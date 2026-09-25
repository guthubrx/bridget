//! SSH local RÉEL, namespace privé, aucun sshd/config/authorized_keys système modifié.
//! Recette opt-in : nécessite OpenSSH et un compte local autorisé à s'authentifier.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::WrapperToDaemon;
use fixture::*;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct SshChild {
    child: Child,
    marker: PathBuf,
}
impl SshChild {
    fn start(command: &mut Command, root: &Path, label: &str) -> Self {
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join(format!("{label}.log")))
            .unwrap();
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .process_group(0)
            .spawn()
            .expect("enfant SSH isolé");
        Self {
            child,
            marker: PathBuf::from(root.file_name().unwrap()),
        }
    }
    fn stop(mut self) {
        self.terminate();
        assert!(
            self.child.try_wait().unwrap().is_some(),
            "processus SSH survivant"
        );
    }

    fn terminate(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        let pid = self.child.id();
        let observed = Command::new("/bin/ps")
            .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
            .output();
        if let Ok(observed) = observed {
            let observed = String::from_utf8_lossy(&observed.stdout);
            if observed
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                == Some(std::process::id())
                && observed.contains(self.marker.to_str().unwrap())
                && !observed.to_lowercase().contains("firefox")
            {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while !matches!(self.child.try_wait(), Ok(Some(_))) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            eprintln!("nettoyage SSH non attesté pour l'enfant {pid}");
        }
    }
}

fn ssh_output(mut command: Command, root: &Path, label: &str) -> std::process::Output {
    let out = root.join(format!("{label}.stdout"));
    let err = root.join(format!("{label}.stderr"));
    let create = |path: &Path| {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .unwrap()
    };
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(create(&out)))
        .stderr(Stdio::from(create(&err)))
        .process_group(0)
        .spawn()
        .unwrap();
    let mut child = SshChild {
        child,
        marker: PathBuf::from(root.file_name().unwrap()),
    };
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.child.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "commande SSH hors budget : {}",
            fs::read_to_string(&err).unwrap()
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    std::process::Output {
        status,
        stdout: fs::read(out).unwrap(),
        stderr: fs::read(err).unwrap(),
    }
}

fn shell_quote(value: &str) -> String {
    assert!(!value.contains('\0'));
    format!("'{}'", value.replace('\'', "'\\''"))
}

struct Remote {
    host: String,
    user: String,
    port: String,
    identity: String,
    known_hosts: String,
    binary: String,
    root: PathBuf,
}
impl Remote {
    fn configured(root: &Path) -> Self {
        let required = |key| {
            std::env::var(key).unwrap_or_else(|_| panic!("paramètre de recette absent : {key}"))
        };
        let parent: String = required("BRIDGET_SSH_REMOTE_PARENT");
        Self {
            host: required("BRIDGET_SSH_REMOTE_HOST"),
            user: required("BRIDGET_SSH_REMOTE_USER"),
            port: required("BRIDGET_SSH_REMOTE_PORT"),
            identity: required("BRIDGET_SSH_IDENTITY"),
            known_hosts: required("BRIDGET_SSH_KNOWN_HOSTS"),
            binary: required("BRIDGET_SSH_REMOTE_BIN"),
            root: Path::new(&parent).join(format!(
                "bg089-{}",
                root.file_name().unwrap().to_str().unwrap()
            )),
        }
    }
    fn socket(&self) -> PathBuf {
        self.root.join("peer.sock")
    }
    fn command(&self, program: &str) -> Command {
        let mut command = Command::new("/usr/bin/ssh");
        command
            .args([
                "-F",
                "/dev/null",
                "-p",
                &self.port,
                "-i",
                &self.identity,
                "-o",
                "IdentitiesOnly=yes",
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=yes",
                "-o",
                &format!("UserKnownHostsFile={}", shell_quote(&self.known_hosts)),
                "-o",
                "GlobalKnownHostsFile=/dev/null",
                "-o",
                "UpdateHostKeys=no",
                "-o",
                "ControlMaster=no",
                "-o",
                "ControlPath=none",
                "-o",
                "ForwardAgent=no",
                "-o",
                "PermitLocalCommand=no",
                "-o",
                "ConnectTimeout=10",
                "-o",
                "ServerAliveInterval=5",
                "-o",
                "ServerAliveCountMax=2",
            ])
            .arg(format!("{}@{}", self.user, self.host))
            .arg(program);
        command
    }
    fn cli(
        &self,
        root: &Path,
        label: &str,
        agent: &str,
        instance: &str,
        args: &[&str],
    ) -> std::process::Output {
        let arguments = args
            .iter()
            .map(|s| shell_quote(s))
            .collect::<Vec<_>>()
            .join(" ");
        let program = format!(
            "env -i PATH=/usr/bin:/bin HOME={home} BRIDGET_HOME={home} BRIDGET_SOCKET={socket} BRIDGET_CHANNEL=ssh-unix BRIDGET_AGENT_ID={agent} BRIDGET_AGENT_INSTANCE_ID={instance} {binary} {arguments}",
            home = shell_quote(self.root.to_str().unwrap()),
            socket = shell_quote(self.socket().to_str().unwrap()),
            agent = shell_quote(agent),
            instance = shell_quote(instance),
            binary = shell_quote(&self.binary)
        );
        ssh_output(self.command(&program), root, label)
    }
    fn tunnel(&self, root: &Path, label: &str) -> SshChild {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/federate-ssh.sh");
        let mut command = Command::new("/bin/bash");
        command
            .arg(script)
            .args([
                "run",
                "--label",
                label,
                "--host",
                &self.host,
                "--user",
                &self.user,
                "--port",
                &self.port,
                "--identity",
                &self.identity,
                "--known-hosts",
                &self.known_hosts,
            ])
            .arg("--root")
            .arg(root.join("state"))
            .arg("--socket")
            .arg(socket(root))
            .arg("--remote-root")
            .arg(&self.root)
            .arg("--remote-socket")
            .arg(self.socket());
        SshChild::start(&mut command, root, label)
    }
}

fn ack(peer: &mut Client, expected: &str) {
    match receive_delivery(peer) {
        bridget_transport::DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            delivery_generation,
            ..
        } => {
            assert_eq!(message.id, expected);
            peer.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
        }
        other => panic!("remise attendue : {other:?}"),
    }
}

fn child_pids(parent: u32) -> Vec<u32> {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,ppid="])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut pids = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse::<u32>().ok()?;
            (fields.next()?.parse::<u32>().ok()? == parent).then_some(pid)
        })
        .collect::<Vec<_>>();
    pids.sort_unstable();
    pids
}

struct LoadWrapper {
    daemon: Option<DaemonProcess>,
    done: std::sync::mpsc::Receiver<Result<(), String>>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl LoadWrapper {
    fn shutdown(&mut self) -> bool {
        if let Some(daemon) = self.daemon.take() {
            daemon.stop();
        }
        if self.handle.is_none() {
            return true;
        }
        match self.done.recv_timeout(Duration::from_secs(8)) {
            Ok(result) => {
                let joined = self.handle.take().unwrap().join();
                result.is_ok() && joined.is_ok()
            }
            Err(error) => {
                eprintln!("wrapper de mesure non récolté : {error}");
                false
            }
        }
    }
}
impl Drop for LoadWrapper {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // Le garde du daemon nettoie sans paniquer pendant un échec
            // d'oracle ; jamais de double panique qui court-circuite les Drop.
            drop(self.daemon.take());
        } else {
            let _ = self.shutdown();
        }
    }
}

fn unix_ns() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}

fn clock_offset(remote: &Remote, root: &Path, phase: &str) -> (i128, i128) {
    use std::io::{BufRead, Write};
    use std::os::fd::AsRawFd;
    // Une connexion déjà établie : ne pas inclure l'authentification SSH et
    // le lancement Python dans la moitié aller d'une sonde d'horloge.
    let script = "import sys,time\nprint('ready',flush=True)\nfor line in sys.stdin: print(time.time_ns(),flush=True)";
    let log = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(format!("clock-{phase}.stderr")))
        .unwrap();
    let child = remote
        .command(&format!(
            "python3 -u -c {} {}",
            shell_quote(script),
            shell_quote(remote.root.to_str().unwrap())
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log))
        .process_group(0)
        .spawn()
        .unwrap();
    let mut process = SshChild {
        child,
        marker: PathBuf::from(root.file_name().unwrap()),
    };
    let mut input = process.child.stdin.take().unwrap();
    let mut output = std::io::BufReader::new(process.child.stdout.take().unwrap());
    let mut read = || {
        let mut fd = libc::pollfd {
            fd: output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut fd, 1, 10_000) } > 0,
            "sonde horloge muette"
        );
        let mut line = String::new();
        assert!(output.read_line(&mut line).unwrap() > 0);
        line
    };
    assert_eq!(read().trim(), "ready");
    let mut samples = Vec::new();
    for _ in 0..7 {
        let before = unix_ns();
        input.write_all(b"probe\n").unwrap();
        input.flush().unwrap();
        let line = read();
        let after = unix_ns();
        let distant: i128 = line.trim().parse().unwrap();
        samples.push((after - before, distant - (before + after) / 2));
    }
    drop(input);
    process.stop();
    samples.sort_unstable();
    (samples[0].1, samples[0].0 / 2)
}

#[test]
#[ignore = "600 événements / 60 s, deux machines ; namespace BRIDGET_HOME dédié requis"]
fn charge_locale_distante_600_evenements_en_soixante_secondes() {
    use bridget_transport::DaemonToWrapper;
    use std::io::Write;
    assert_eq!(std::env::var("BRIDGET_SSH_LOAD_GATE").as_deref(), Ok("1"));
    // Ce test est lancé seul dans SON processus configuré, sans set_var après
    // démarrage des threads. Le wrapper injectable refuse toute autre racine.
    bridget_daemon::environment::initialize_process().unwrap();
    let namespace = bridget_daemon::environment::Namespace::from_environment().unwrap();
    let root = namespace.root.parent().unwrap().to_path_buf();
    assert!(
        root.file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("b089load-")
    );
    for path in [root.join("provider"), root.join("tmp")] {
        private_dir(&path).unwrap();
    }
    assert_eq!(namespace.socket, socket(&root));
    let remote = Remote::configured(&root);
    let daemon = spawn_performance_daemon(&root);
    let actor = register_agent_as(&socket(&root), ACTOR, "load-actor");
    let probe = bridget_transport::journal::AppendLatencyProbe::install_all_events(
        root.join("state/sessions"),
    );
    let (registry, registry_root, _counter) = registry_with_counting_acp_agent(201);
    let (done_tx, done) = std::sync::mpsc::channel();
    let wrapper_root = root.clone();
    let handle = std::thread::spawn(move || {
        let result = bridget_daemon::wrapper::launch_acp_with(
            "fixture-acp",
            &[],
            Some(ACP_AGENT),
            &registry,
            &socket(&wrapper_root),
            &wrapper_root.join("provider"),
        );
        let _ = done_tx.send(result.map_err(|error| error.to_string()));
    });
    let mut wrapper = LoadWrapper {
        daemon: Some(daemon),
        done,
        handle: Some(handle),
    };
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let mut local = Client::connect(&socket(&root));
    local.send(WrapperToDaemon::RoleHandshake {
        role: bridget_transport::protocol::ConnectionRole::Attach,
    });
    assert!(matches!(
        local.receive(),
        DaemonToWrapper::RoleAccepted { .. }
    ));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        local.send(WrapperToDaemon::Subscribe {
            agent: ACP_AGENT.into(),
            window: bridget_transport::protocol::AttachWindow::Seq(0),
        });
        match local.receive() {
            DaemonToWrapper::Subscribed { .. } => break,
            DaemonToWrapper::AttachRejected {
                reason: bridget_transport::protocol::AttachRefusal::JournalUnavailable,
                ..
            } => assert!(Instant::now() < deadline),
            other => panic!("attache locale : {other:?}"),
        }
    }
    assert!(matches!(
        local.receive(),
        DaemonToWrapper::SnapshotCaughtUp {
            through_seq: None,
            ..
        }
    ));
    local
        .reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(75)))
        .unwrap();
    let rendered_path = root.join("local-render.jsonl");
    let (local_tx, local_rx) = std::sync::mpsc::channel();
    let local_reader = std::thread::spawn(move || {
        let mut rendered = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(rendered_path)
            .unwrap();
        let mut current = Vec::new();
        let mut received = std::collections::BTreeMap::new();
        while received.len() < 600 {
            match local.receive() {
                DaemonToWrapper::JournalFragment {
                    seq,
                    offset,
                    final_fragment,
                    bytes,
                    ..
                } => {
                    assert_eq!(offset, current.len() as u64);
                    current.extend(bytes);
                    if final_fragment {
                        rendered.write_all(&current).unwrap();
                        rendered.write_all(b"\n").unwrap();
                        rendered.flush().unwrap();
                        assert!(
                            received.insert(seq, Instant::now()).is_none(),
                            "doublon local"
                        );
                        current.clear();
                    }
                }
                other => panic!("flux local interrompu : {other:?}"),
            }
        }
        local_tx.send(received).unwrap();
    });
    let tunnel = remote.tunnel(&root, "load-tunnel");
    let deadline = Instant::now() + Duration::from_secs(15);
    for n in 0.. {
        let result = remote.cli(
            &root,
            &format!("load-who-{n}"),
            ACTOR,
            "load-actor",
            &["who"],
        );
        if result.status.success() {
            break;
        }
        assert!(Instant::now() < deadline, "{}", output_text(&result));
    }
    let clock_before = clock_offset(&remote, &root, "before");
    let frames = serde_json::json!([
        WrapperToDaemon::RoleHandshake {
            role: bridget_transport::protocol::ConnectionRole::Attach
        },
        WrapperToDaemon::Subscribe {
            agent: ACP_AGENT.into(),
            window: bridget_transport::protocol::AttachWindow::Seq(0)
        }
    ]);
    let script = r#"import socket,sys,json,time,base64
s=socket.socket(socket.AF_UNIX); s.settimeout(75); s.connect(sys.argv[1]); f=s.makefile('rb'); deadline=time.monotonic()+75
def receive():
 s.settimeout(max(0.001,deadline-time.monotonic())); raw=f.readline(16*1024*1024)
 assert raw.endswith(b'\n'); return json.loads(raw)
for request in json.loads(sys.argv[2]):
 s.sendall((json.dumps(request,separators=(',',':'))+'\n').encode()); reply=receive()
 assert reply['type'] in ('RoleAccepted','Subscribed'), reply
assert receive()['type']=='SnapshotCaughtUp'
print(json.dumps({'ready':True}),flush=True)
seen=set(); line=bytearray()
while len(seen)<600:
 event=receive(); assert event['type']=='JournalFragment',event
 assert event['offset']==len(line); line.extend(base64.b64decode(event['bytes']))
 if event['final']:
  assert event['seq'] not in seen; seen.add(event['seq'])
  sys.stdout.buffer.write(line+b'\n'); sys.stdout.buffer.flush()
  print(json.dumps({'rendered_seq':event['seq'],'rendered_ns':time.time_ns()}),flush=True); line.clear()
"#;
    let program = format!(
        "python3 -c {} {} {}",
        shell_quote(script),
        shell_quote(remote.socket().to_str().unwrap()),
        shell_quote(&frames.to_string())
    );
    let mut capture = SshChild::start(&mut remote.command(&program), &root, "load-remote");
    ready(&mut capture, &root.join("load-remote.log"), || {
        fs::read_to_string(root.join("load-remote.log"))
            .unwrap()
            .lines()
            .any(|l| serde_json::from_str::<serde_json::Value>(l).is_ok_and(|v| v["ready"] == true))
    });
    let anchor_ns = unix_ns();
    let anchor = Instant::now();
    let started = Instant::now();
    let mut sender = negotiate_client(&socket(&root));
    let mut submitted = Vec::new();
    for turn in 0..200 {
        // 200 tours, trois lignes chacun : 600 événements à 10/s en moyenne,
        // par petits groupes de trois. Ce profil est publié, pas présenté comme
        // un événement uniformément espacé de 100 ms.
        if let Some(delay) =
            (started + Duration::from_millis(300) * turn).checked_duration_since(Instant::now())
        {
            std::thread::sleep(delay);
        }
        assert!(started.elapsed() < Duration::from_secs(70));
        let id = format!("089-load-{turn}");
        let mut message = BridgetMessage::new(ACTOR, ACP_AGENT, format!("charge numérotée {turn}"));
        message.id = id.clone();
        let command = WrapperToDaemon::SendIdempotent {
            message,
            message_id: id,
            issued_at: issued_at(),
        };
        sender.send(command.clone());
        let result = sender.receive();
        match result {
            DaemonToWrapper::IdempotencyResult {
                issue: bridget_transport::protocol::IdempotencyIssue::Accepted { .. },
                ..
            } => {}
            DaemonToWrapper::IdempotencyResult {
                issue: bridget_transport::protocol::IdempotencyIssue::OutcomeUnknown { .. },
                ..
            } => {}
            other => panic!("tour {turn} refusé : {other:?}"),
        }
        // Remise différée du protocole : ne pas sérialiser la cadence sur
        // l'ACK, ni confondre OutcomeUnknown avec Accepted. Tous les terminaux
        // sont contrôlés après la phase d'émission, avec ces mêmes bytes.
        submitted.push(command);
    }
    if let Some(delay) = (started + Duration::from_secs(60)).checked_duration_since(Instant::now())
    {
        std::thread::sleep(delay);
    }
    let local_times = local_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    local_reader.join().unwrap();
    for command in &submitted {
        assert!(
            matches!(
                retry_command_issue(&socket(&root), command.clone()),
                bridget_transport::protocol::IdempotencyIssue::Accepted { .. }
            ),
            "remise non terminale après rendu"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let remote_status = loop {
        if let Some(status) = capture.child.try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "réception distante incomplète");
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(
        remote_status.success(),
        "{}",
        fs::read_to_string(root.join("load-remote.log")).unwrap()
    );
    let clock_after = clock_offset(&remote, &root, "after");
    let drift = (clock_after.0 - clock_before.0).abs();
    assert!(
        drift <= clock_before.1 + clock_after.1,
        "horloge incompatible avec les bornes de mesure"
    );
    let remote_output = fs::read_to_string(root.join("load-remote.log")).unwrap();
    let mut remote_times = std::collections::BTreeMap::new();
    let mut remote_bytes = Vec::new();
    for line in remote_output.lines() {
        let entry: serde_json::Value = serde_json::from_str(line).unwrap();
        if let Some(seq) = entry["rendered_seq"].as_u64() {
            assert!(
                remote_times
                    .insert(seq, entry["rendered_ns"].as_i64().unwrap() as i128)
                    .is_none()
            );
        } else if entry["v"] == 1 {
            remote_bytes.extend_from_slice(line.as_bytes());
            remote_bytes.push(b'\n');
        } else {
            assert_eq!(entry["ready"], true);
        }
    }
    let expected = (1..=600).collect::<Vec<_>>();
    assert_eq!(local_times.keys().copied().collect::<Vec<_>>(), expected);
    assert_eq!(remote_times.keys().copied().collect::<Vec<_>>(), expected);
    assert_eq!(
        fs::read(root.join("local-render.jsonl")).unwrap(),
        remote_bytes,
        "rendu brut identique"
    );
    let samples = probe.take_samples();
    assert_eq!(samples.len(), 600, "chaque append réellement échantillonné");
    let uncertainty = clock_before.1.max(clock_after.1) + drift;
    let mut local_ns = Vec::new();
    let mut remote_raw_ns = Vec::new();
    let mut remote_corrected_ns = Vec::new();
    for sample in &samples {
        local_ns.push(
            local_times[&sample.seq]
                .checked_duration_since(sample.completed_at)
                .expect("rendu avant append")
                .as_nanos() as i128,
        );
        let append_ns = anchor_ns + sample.completed_at.duration_since(anchor).as_nanos() as i128;
        let raw = remote_times[&sample.seq] - append_ns;
        remote_raw_ns.push(raw);
        remote_corrected_ns.push(raw - clock_before.0);
    }
    let percentile = |values: &[i128]| {
        let mut copy = values.to_vec();
        copy.sort_unstable();
        copy[(copy.len() * 95).div_ceil(100) - 1]
    };
    let local_p95 = percentile(&local_ns);
    let remote_p95 = percentile(&remote_corrected_ns);
    let report = serde_json::json!({"v":1,"criterion":"SC-08912","turns":200,"journal_events":600,"duration_s":60,"profile":"3 événements par tour, 1 tour/300ms","local_received":local_times.len(),"remote_received":remote_times.len(),"local_p95_ns":local_p95,"local_max_ns":local_ns.iter().max(),"remote_raw_p95_ns":percentile(&remote_raw_ns),"remote_corrected_p95_ns":remote_p95,"remote_corrected_max_ns":remote_corrected_ns.iter().max(),"clock_offset_before_ns":clock_before.0,"clock_offset_after_ns":clock_after.0,"clock_uncertainty_ns":uncertainty,"remote_p95_upper_bound_ns":remote_p95+uncertainty,"remote_raw_negative_count":remote_raw_ns.iter().filter(|v|**v<0).count(),"local_samples_ns":local_ns,"remote_raw_samples_ns":remote_raw_ns,"remote_corrected_samples_ns":remote_corrected_ns});
    private_write(
        &root.join("load-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!(
        "rapport charge {} : local p95={}ms, distant corrigé p95={}ms, borne haute={}ms, incertitude={}ms, 600/600 deux vues",
        root.join("load-report.json").display(),
        local_p95 as f64 / 1e6,
        remote_p95 as f64 / 1e6,
        (remote_p95 + uncertainty) as f64 / 1e6,
        uncertainty as f64 / 1e6
    );
    assert!(local_p95 < 1_000_000_000 && *local_ns.iter().max().unwrap() < 3_000_000_000);
    assert!(remote_p95 + uncertainty < 3_000_000_000);
    drop(sender);
    drop(actor);
    capture.stop();
    tunnel.stop();
    assert!(wrapper.shutdown());
    fs::remove_dir_all(registry_root).unwrap();
    cleanup_remote_namespace(&remote, &root);
}

#[test]
#[ignore = "deux machines autorisées requises ; paramètres BRIDGET_SSH_REMOTE_* explicites"]
fn deux_machines_cli_reel_demande_reponse_ledger_journal() {
    remote_exchange(false);
}

#[test]
#[ignore = "recette SSH distante : coupe/reprise du seul tunnel de test"]
fn deux_machines_coupure_reprise_curseur_et_lacune() {
    remote_exchange(true);
}

fn remote_exchange(reconnect: bool) {
    assert_eq!(std::env::var("BRIDGET_SSH_REMOTE_GATE").as_deref(), Ok("1"));
    let root = fs::canonicalize(test_root("089-ssh-remote")).unwrap();
    let remote = Remote::configured(&root);
    let historical = if reconnect {
        let mut date = Command::new("/bin/date");
        #[cfg(target_os = "macos")]
        date.args(["-v-1d", "+%F"]);
        #[cfg(not(target_os = "macos"))]
        date.args(["-d", "yesterday", "+%F"]);
        let date = date.output().unwrap();
        assert!(date.status.success());
        let directory = root.join("state/sessions").join(ACP_AGENT);
        private_dir(&directory).unwrap();
        let file = directory.join(format!(
            "{}.jsonl",
            String::from_utf8(date.stdout).unwrap().trim()
        ));
        private_write(
            &file,
            include_bytes!("../../../fixtures/journal-source-unusual-v1.jsonl"),
        )
        .unwrap();
        Some(file)
    } else {
        None
    };
    let through_seq = if reconnect { 8 } else { 3 };
    let daemon = spawn_daemon(&root, None);
    let mut actor = register_agent_as(&socket(&root), ACTOR, "shared-cli-mcp-instance");
    let mut recipient = register_recipient_as(&socket(&root), "089-remote-recipient");
    let (registry, registry_root, counter) = registry_with_counting_acp_agent(2);
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let mut tunnel = Some(remote.tunnel(&root, "remote-tunnel"));
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut attempt = 0;
    loop {
        let who = remote.cli(
            &root,
            &format!("who-{attempt}"),
            ACTOR,
            "shared-cli-mcp-instance",
            &["who"],
        );
        if who.status.success() {
            assert!(String::from_utf8_lossy(&who.stdout).contains("fixture-acp"));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "tunnel jamais prêt : {}",
            output_text(&who)
        );
        attempt += 1;
    }
    let directory = remote.cli(
        &root,
        "agents",
        ACTOR,
        "shared-cli-mcp-instance",
        &["agents", "--json"],
    );
    assert!(directory.status.success(), "{}", output_text(&directory));
    let directory: serde_json::Value = serde_json::from_slice(&directory.stdout).unwrap();
    assert!(
        directory.to_string().contains(ACTOR),
        "identité UUID absente de la projection machine"
    );
    let provider_pids = child_pids(wrapper.0.id());
    assert!(
        !provider_pids.is_empty(),
        "fournisseur réel absent derrière le wrapper"
    );
    let timestamp = issued_at().to_string();
    let question = [
        "send",
        "--to",
        RECIPIENT,
        "--reply",
        "--timeout",
        "60",
        "--id",
        "089-ssh-question",
        "--issued-at",
        &timestamp,
        "--issuer-scope",
        SCOPE,
        "--",
        "Question SSH : été $VAR ' intact\nligne 2",
    ];
    let request = remote.cli(
        &root,
        "request",
        ACTOR,
        "shared-cli-mcp-instance",
        &question,
    );
    assert!(request.status.success(), "{}", output_text(&request));
    ack(&mut recipient, "089-ssh-question");
    let reply = [
        "send",
        "--to",
        ACTOR,
        "--in-reply-to",
        "089-ssh-question",
        "--id",
        "089-ssh-answer",
        "--issued-at",
        &timestamp,
        "--issuer-scope",
        SCOPE,
        "--",
        "Réponse distante corrélée — intacte",
    ];
    let response = remote.cli(&root, "reply", RECIPIENT, "089-remote-recipient", &reply);
    assert!(response.status.success(), "{}", output_text(&response));
    ack(&mut actor, "089-ssh-answer");
    let replay = remote.cli(
        &root,
        "request-replay",
        ACTOR,
        "shared-cli-mcp-instance",
        &question,
    );
    assert!(replay.status.success(), "{}", output_text(&replay));
    assert!(String::from_utf8_lossy(&replay.stdout).starts_with("OK: accepted "));
    let reply_replay = remote.cli(
        &root,
        "reply-replay",
        RECIPIENT,
        "089-remote-recipient",
        &reply,
    );
    assert!(
        reply_replay.status.success(),
        "{}",
        output_text(&reply_replay)
    );
    assert!(String::from_utf8_lossy(&reply_replay.stdout).starts_with("OK: accepted "));
    let store = fixture_store(&root.join("state/bridget.db")).unwrap();
    assert_eq!(store.recent_messages(100).unwrap().len(), 2);
    let mut observer = Client::connect(&socket(&root));
    observer.send(WrapperToDaemon::ListRequests {
        sender: ACTOR.into(),
        limit: 20,
    });
    match observer.receive() {
        bridget_transport::DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].state, "answered");
        }
        other => panic!("projection des demandes : {other:?}"),
    }
    let local = run_isolated(&root, &["ledger", "--limit", "20"], false);
    let distant = remote.cli(
        &root,
        "ledger",
        ACTOR,
        "shared-cli-mcp-instance",
        &["ledger", "--limit", "20"],
    );
    assert!(
        local.status.success() && distant.status.success(),
        "{}",
        output_text(&distant)
    );
    assert_eq!(local.stdout, distant.stdout);
    let prompt = remote.cli(
        &root,
        "prompt",
        ACTOR,
        "shared-cli-mcp-instance",
        &[
            "send",
            "--to",
            ACP_AGENT,
            "--id",
            "089-ssh-journal",
            "--issued-at",
            &timestamp,
            "--issuer-scope",
            SCOPE,
            "--",
            "Tour au vrai wrapper depuis Linux",
        ],
    );
    assert!(prompt.status.success(), "{}", output_text(&prompt));
    // Client public indépendant côté Linux : pas de fichier/base locale lu.
    let script = r#"import socket,sys,json,time
deadline=time.monotonic()+10
s=socket.socket(socket.AF_UNIX); s.settimeout(10); s.connect(sys.argv[1]); f=s.makefile('rb')
def receive():
 s.settimeout(max(0.001,deadline-time.monotonic()))
 raw=f.readline(16*1024*1024)
 assert raw.endswith(b'\n'), 'trame incomplete'
 return raw,json.loads(raw)
for request in json.loads(sys.argv[2]):
 s.sendall((json.dumps(request,separators=(',',':'))+'\n').encode())
 raw,event=receive(); assert event['type'] in ('RoleAccepted','Subscribed'), event
caught=False; seq=0
while not (caught and seq>=int(sys.argv[3])):
 raw,event=receive(); sys.stdout.buffer.write(raw); sys.stdout.buffer.flush()
 if event['type']=='SnapshotCaughtUp': caught=True
 elif event['type']=='JournalFragment':
  if event['final']: seq=event['seq']
 elif event['type']!='Gap': raise AssertionError(event)
"#;
    let capture_from = |label: &str, from| {
        let frames = serde_json::json!([
            WrapperToDaemon::RoleHandshake {
                role: bridget_transport::protocol::ConnectionRole::Attach
            },
            WrapperToDaemon::Subscribe {
                agent: ACP_AGENT.into(),
                window: bridget_transport::protocol::AttachWindow::Seq(from)
            }
        ]);
        let program = format!(
            "python3 -c {} {} {} {through_seq}",
            shell_quote(script),
            shell_quote(remote.socket().to_str().unwrap()),
            shell_quote(&frames.to_string())
        );
        ssh_output(remote.command(&program), &root, label)
    };
    let capture = capture_from("journal", 0);
    assert!(capture.status.success(), "{}", output_text(&capture));
    let mut lines = std::collections::BTreeMap::<u64, Vec<u8>>::new();
    for raw in capture
        .stdout
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
    {
        match bridget_transport::protocol::decode::<bridget_transport::DaemonToWrapper>(
            std::str::from_utf8(raw).unwrap(),
        )
        .unwrap()
        {
            bridget_transport::DaemonToWrapper::JournalFragment {
                seq, offset, bytes, ..
            } => {
                let line = lines.entry(seq).or_default();
                assert_eq!(line.len() as u64, offset);
                line.extend(bytes);
            }
            bridget_transport::DaemonToWrapper::SnapshotCaughtUp { .. } => {}
            other => panic!("fait inattendu : {other:?}"),
        }
    }
    let expected = if reconnect {
        vec![5, 6, 7, 8]
    } else {
        vec![1, 2, 3]
    };
    assert_eq!(lines.keys().copied().collect::<Vec<_>>(), expected);
    let journal = root.join("state/sessions").join(ACP_AGENT).join(format!(
        "{}.jsonl",
        bridget_transport::journal::current_host_date()
    ));
    for raw in fs::read(journal)
        .unwrap()
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
    {
        let parsed: serde_json::Value = serde_json::from_slice(raw).unwrap();
        assert_eq!(lines[&parsed["seq"].as_u64().unwrap()], raw);
    }
    assert_eq!(fs::read(&counter).unwrap(), b"x");
    if let Some(historical) = historical {
        // Identité de la socket AVANT coupure ; une autre socket du même compte
        // ne serait pas une preuve suffisante pour permettre son effacement.
        let metadata_program = format!(
            "python3 -c {} {}",
            shell_quote(
                "import os,sys,json; s=os.lstat(sys.argv[1]); print(json.dumps([s.st_dev,s.st_ino]))"
            ),
            shell_quote(remote.socket().to_str().unwrap())
        );
        let metadata = ssh_output(remote.command(&metadata_program), &root, "socket-before");
        assert!(metadata.status.success(), "{}", output_text(&metadata));
        let identity: Vec<u64> = serde_json::from_slice(&metadata.stdout).unwrap();
        tunnel.take().unwrap().stop();
        let offline = remote.cli(
            &root,
            "offline",
            ACTOR,
            "shared-cli-mcp-instance",
            &["ledger"],
        );
        assert!(!offline.status.success());
        assert!(offline.stdout.is_empty());
        let disconnected = capture_from("journal-offline", through_seq);
        assert!(!disconnected.status.success());
        assert!(
            disconnected.stdout.is_empty(),
            "aucun fragment/fraîcheur fabriqué hors connexion"
        );
        // Le nettoyage est une opération de recette explicitement autorisée,
        // pas un StreamLocalBindUnlink automatique dans le produit.
        let unlink = r#"import os,sys,socket,stat,json
p=sys.argv[1]; wanted=json.loads(sys.argv[2]); m=os.lstat(p)
assert [m.st_dev,m.st_ino]==wanted and m.st_uid==os.getuid() and stat.S_ISSOCK(m.st_mode)
s=socket.socket(socket.AF_UNIX); s.settimeout(2)
try: s.connect(p)
except ConnectionRefusedError: pass
else: raise AssertionError('socket encore vivante')
s.close(); m=os.lstat(p); assert [m.st_dev,m.st_ino]==wanted; os.unlink(p)
"#;
        let removed = ssh_output(
            remote.command(&format!(
                "python3 -c {} {} {}",
                shell_quote(unlink),
                shell_quote(remote.socket().to_str().unwrap()),
                shell_quote(&serde_json::to_string(&identity).unwrap())
            )),
            &root,
            "stale-cleanup",
        );
        assert!(removed.status.success(), "{}", output_text(&removed));
        tunnel = Some(remote.tunnel(&root, "resumed-tunnel"));
        let deadline = Instant::now() + Duration::from_secs(15);
        for attempt in 0.. {
            let probe = remote.cli(
                &root,
                &format!("resumed-who-{attempt}"),
                ACTOR,
                "shared-cli-mcp-instance",
                &["who"],
            );
            if probe.status.success() {
                break;
            }
            assert!(Instant::now() < deadline, "{}", output_text(&probe));
        }
        // Même commande, même canon : le fournisseur ne reçoit PAS un tour neuf.
        let replay = remote.cli(
            &root,
            "prompt-replay",
            ACTOR,
            "shared-cli-mcp-instance",
            &[
                "send",
                "--to",
                ACP_AGENT,
                "--id",
                "089-ssh-journal",
                "--issued-at",
                &timestamp,
                "--issuer-scope",
                SCOPE,
                "--",
                "Tour au vrai wrapper depuis Linux",
            ],
        );
        assert!(replay.status.success());
        assert!(String::from_utf8_lossy(&replay.stdout).starts_with("OK: accepted "));
        let resumed = capture_from("journal-resumed", through_seq);
        assert!(resumed.status.success(), "{}", output_text(&resumed));
        let mut resumed_bytes = Vec::new();
        let mut caught_up = 0;
        for raw in resumed
            .stdout
            .split(|b| *b == b'\n')
            .filter(|s| !s.is_empty())
        {
            match bridget_transport::protocol::decode::<bridget_transport::DaemonToWrapper>(
                std::str::from_utf8(raw).unwrap(),
            )
            .unwrap()
            {
                bridget_transport::DaemonToWrapper::JournalFragment {
                    seq, offset, bytes, ..
                } => {
                    assert_eq!(seq, through_seq);
                    assert_eq!(offset, resumed_bytes.len() as u64);
                    resumed_bytes.extend(bytes);
                }
                bridget_transport::DaemonToWrapper::SnapshotCaughtUp {
                    through_seq: seq, ..
                } => {
                    assert_eq!(seq, Some(through_seq));
                    caught_up += 1;
                }
                other => panic!("reprise sans trou attendu : {other:?}"),
            }
        }
        assert_eq!(caught_up, 1);
        assert_eq!(resumed_bytes, lines[&through_seq]);
        fs::remove_file(historical).unwrap(); // rétention simulée d'UNE fixture
        let gap = capture_from("journal-gap", 5);
        assert!(gap.status.success(), "{}", output_text(&gap));
        let first = std::str::from_utf8(&gap.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap();
        // Mutant : Gap→Unavailable ou absence de Gap => cet oracle distant casse.
        assert!(matches!(
            bridget_transport::protocol::decode::<bridget_transport::DaemonToWrapper>(first)
                .unwrap(),
            bridget_transport::DaemonToWrapper::Gap {
                from_seq: 5,
                to_seq: 5,
                ..
            }
        ));
        assert_eq!(fs::read(&counter).unwrap(), b"x");
        assert_eq!(
            store
                .recent_messages(100)
                .unwrap()
                .iter()
                .filter(|m| m.id == "089-ssh-journal")
                .count(),
            1
        );
        observer.send(WrapperToDaemon::ListAgents);
        match observer.receive() {
            bridget_transport::DaemonToWrapper::AgentList { agents } => {
                for agent_id in [ACTOR, ACP_AGENT] {
                    let active = agents.iter().find(|a| a.agent_id == agent_id).unwrap();
                    let before = directory
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|a| a["agent_id"] == agent_id)
                        .unwrap();
                    assert_eq!(active.connection_id, before["connection_id"]);
                }
                assert_eq!(
                    child_pids(wrapper.0.id()),
                    provider_pids,
                    "pas de remplacement fournisseur indu par la coupure"
                );
            }
            other => panic!("annuaire absent : {other:?}"),
        }
    }
    assert_no_delivery(&mut actor);
    assert_no_delivery(&mut recipient);
    drop(store);
    drop(actor);
    drop(recipient);
    drop(observer);
    tunnel.take().unwrap().stop();
    daemon.stop();
    assert_eq!(wrapper.join(), Ok(()));
    cleanup_remote_namespace(&remote, &root);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(registry_root).unwrap();
}

fn cleanup_remote_namespace(remote: &Remote, root: &Path) {
    // Pas de rm -rf distant : suppression de la seule socket créée, après
    // échec de connexion, propriétaire/type vérifiés ; racine vide exigée.
    let cleanup = r#"import os,sys,socket,stat
p=sys.argv[1]; r=sys.argv[2]
if os.path.lexists(p):
 m=os.lstat(p); assert stat.S_ISSOCK(m.st_mode) and m.st_uid==os.getuid()
 s=socket.socket(socket.AF_UNIX); s.settimeout(2)
 try: s.connect(p)
 except ConnectionRefusedError: pass
 else: raise AssertionError('socket encore vivante')
 s.close(); assert os.lstat(p).st_ino==m.st_ino; os.unlink(p)
# Le bootstrap CLI crée seulement ce répertoire temporaire privé et vide.
temporary=os.path.join(r,'tmp')
if os.path.lexists(temporary):
 m=os.lstat(temporary); assert stat.S_ISDIR(m.st_mode) and m.st_uid==os.getuid() and stat.S_IMODE(m.st_mode)==0o700
 os.rmdir(temporary)
os.rmdir(r)
"#;
    let cleaned = ssh_output(
        remote.command(&format!(
            "python3 -c {} {} {}",
            shell_quote(cleanup),
            shell_quote(remote.socket().to_str().unwrap()),
            shell_quote(remote.root.to_str().unwrap())
        )),
        root,
        "cleanup",
    );
    assert!(cleaned.status.success(), "{}", output_text(&cleaned));
}

impl Drop for SshChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn ready(child: &mut SshChild, log: &Path, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            child.child.try_wait().unwrap().is_none(),
            "enfant arrêté : {}",
            fs::read_to_string(log).unwrap()
        );
        if predicate() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "borne de démarrage : {}",
            fs::read_to_string(log).unwrap()
        );
        std::thread::sleep(Duration::from_millis(5)); // observation de disponibilité, pas délai métier
    }
}

#[test]
#[ignore = "recette réelle opt-in : BRIDGET_SSH_LOCAL_GATE=1, sshd local requis"]
fn ssh_unix_reel_garde_annuaire_ledger_permissions_et_coupure_honnete() {
    assert_eq!(std::env::var("BRIDGET_SSH_LOCAL_GATE").as_deref(), Ok("1"));
    let root = fs::canonicalize(test_root("089-ssh")).unwrap();
    // StrictModes parcourt les parents d'AuthorizedKeysFile : /tmp, partagé,
    // est refusé par sshd même si son sous-répertoire est privé. Clés de TEST
    // sous target, parents possédés ; jamais ~/.ssh/authorized_keys.
    let keys = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("ssh-089-{}", uuid::Uuid::new_v4()));
    private_dir(&keys).unwrap();
    let keys = fs::canonicalize(keys).unwrap();
    for key in ["host", "client"] {
        let status = Command::new("/usr/bin/ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(keys.join(key))
            .status()
            .unwrap();
        assert!(status.success());
    }
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let user = Command::new("/usr/bin/id").arg("-un").output().unwrap();
    let user = String::from_utf8(user.stdout).unwrap().trim().to_owned();
    assert!(user.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'));
    // OpenSSH 10 session.c::do_authenticated désactive aussi l'ACL Unix si
    // AllowTcpForwarding=no. Autoriser la direction remote, mais borner le TCP
    // à loopback:1 (non utilisé, non accessible à ce sshd sans privilèges).
    // Source : openssh-portable V_10_0_P2/session.c, lignes 329–344.
    let config = format!(
        "ListenAddress 127.0.0.1\nPort {port}\nHostKey {host}\nPidFile {pid}\nAuthorizedKeysFile {client}\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nPubkeyAuthentication yes\nUsePAM no\nStrictModes yes\nAllowUsers {user}\nPermitRootLogin no\nAllowTcpForwarding remote\nPermitListen 127.0.0.1:1\nPermitOpen none\nAllowStreamLocalForwarding yes\nStreamLocalBindMask 0177\nStreamLocalBindUnlink no\nX11Forwarding no\nPermitUserEnvironment no\nPermitUserRC no\nLogLevel VERBOSE\n",
        host = keys.join("host").display(),
        pid = root.join("sshd.pid").display(),
        client = keys.join("client.pub").display()
    );
    private_write(&root.join("sshd_config"), config).unwrap();
    let public_key = fs::read_to_string(keys.join("host.pub")).unwrap();
    private_write(
        &keys.join("known_hosts"),
        format!("[127.0.0.1]:{port} {public_key}"),
    )
    .unwrap();
    let mut server = SshChild::start(
        Command::new("/usr/sbin/sshd")
            .args(["-D", "-e", "-f"])
            .arg(root.join("sshd_config")),
        &root,
        "sshd",
    );
    ready(&mut server, &root.join("sshd.log"), || {
        TcpStream::connect(("127.0.0.1", port)).is_ok()
    });

    let store = fixture_store(&root.join("state/bridget.db")).unwrap();
    let mut message = BridgetMessage::new(ACTOR, RECIPIENT, "SSH maître : été $VAR\nligne intacte");
    message.id = "089-ssh-ledger".into();
    store.record_message(&message, &message.id).unwrap();
    drop(store);
    let daemon = spawn_daemon(&root, None);
    let actor = register_agent_as(&socket(&root), ACTOR, "089-ssh-actor");
    let remote_root = root.join("remote");
    let remote_socket = remote_root.join("peer.sock");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/federate-ssh.sh");
    let mut command = Command::new("/bin/bash");
    command
        .arg(script)
        .args([
            "run",
            "--label",
            "089-local",
            "--host",
            "127.0.0.1",
            "--user",
            &user,
            "--port",
            &port.to_string(),
            "--identity",
        ])
        .arg(keys.join("client"))
        .arg("--known-hosts")
        .arg(keys.join("known_hosts"))
        .arg("--root")
        .arg(root.join("state"))
        .arg("--socket")
        .arg(socket(&root))
        .arg("--remote-root")
        .arg(&remote_root)
        .arg("--remote-socket")
        .arg(&remote_socket);
    let mut tunnel = SshChild::start(&mut command, &root, "tunnel");
    ready(&mut tunnel, &root.join("tunnel.log"), || {
        fs::metadata(&remote_socket).is_ok()
    });
    let metadata = fs::symlink_metadata(&remote_socket).unwrap();
    assert!(metadata.file_type().is_socket());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(
        fs::metadata(&remote_root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let mut direct = Client::connect(&socket(&root));
    let mut forwarded = Client::connect(&remote_socket);
    direct.send(WrapperToDaemon::ListAgents);
    forwarded.send(WrapperToDaemon::ListAgents);
    assert_eq!(
        serde_json::to_value(direct.receive()).unwrap(),
        serde_json::to_value(forwarded.receive()).unwrap()
    );
    let local = run_isolated(&root, &["ledger", "--limit", "20"], false);
    let mut distant_command = isolated_command(&root);
    distant_command
        .env("BRIDGET_HOME", &remote_root)
        .env("BRIDGET_SOCKET", &remote_socket)
        .env("BRIDGET_CHANNEL", "ssh-unix")
        .args(["ledger", "--limit", "20"]);
    let distant = run_command(distant_command);
    assert!(
        local.status.success() && distant.status.success(),
        "{}",
        output_text(&distant)
    );
    assert_eq!(
        local.stdout, distant.stdout,
        "même source et mêmes octets à travers SSH"
    );
    assert!(!remote_root.join("bridget.db").exists());
    drop(forwarded);
    tunnel.stop(); // Seul le tunnel de TEST est arrêté, pas le daemon ni l'agent.
    let mut offline_command = isolated_command(&root);
    offline_command
        .env("BRIDGET_HOME", &remote_root)
        .env("BRIDGET_SOCKET", &remote_socket)
        .args(["ledger"]);
    let offline = run_command(offline_command);
    assert!(
        !offline.status.success(),
        "une coupure ne devient pas un ledger vide"
    );
    assert!(offline.stdout.is_empty());
    assert!(!remote_root.join("bridget.db").exists());
    direct.send(WrapperToDaemon::ListAgents);
    assert!(
        serde_json::to_string(&direct.receive())
            .unwrap()
            .contains(ACTOR)
    );
    // Si OpenSSH conserve une socket stale, elle n'autorise jamais unlink
    // automatique : la recette de reconnexion T027 vérifie cette frontière.
    eprintln!(
        "socket après arrêt tunnel : existe={}",
        remote_socket.exists()
    );
    drop(actor);
    drop(direct);
    daemon.stop();
    server.stop();
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&keys).unwrap();
}
