use maicie::app::{DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::bridget_client::IdempotencyIssue;
use maicie::config::DurationClasses;
use maicie::domain::ClasseDuree;
use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

const OBJECTIVE_COUNT: usize = 100;
const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS: usize = 21;
const STATUS_P95_BUDGET: Duration = Duration::from_millis(250);

/// SC-008 : le coût mesuré couvre la vraie commande `maicie status`, la
/// projection SQLite de 100 objectifs et une capture Attach publique réelle.
/// Les 100 objectifs partagent l'unique équipier délégué : la déduplication
/// des abonnements est donc elle aussi exercée, sans simuler le runtime.
///
/// Ce n'est volontairement pas un test ordinaire : le p95 dépend du noyau, de
/// la charge et du coût de création du binaire enfant. La commande explicite
/// produit 21 mesures brutes et peut écrire sa référence locale versionnée si
/// `BRIDGET_PERF_REPORT` désigne son fichier JSON.
#[test]
#[ignore = "mesure locale explicite SC-008 ; voir specs/011-maicie-orchestration/implementation.md"]
fn status_sur_cent_objectifs_respecte_le_budget_p95() {
    let fixture = BenchmarkFixture::new();
    fixture.seed_accepted_objectives(OBJECTIVE_COUNT);

    let (ready_tx, ready_rx) = mpsc::channel();
    let socket = fixture.socket.clone();
    let server =
        thread::spawn(move || serve_statuses(&socket, WARMUP_RUNS + MEASURED_RUNS, ready_tx));
    ready_rx.recv().expect("serveur de benchmark prêt");

    run_status(&fixture, OBJECTIVE_COUNT);
    let mut samples = Vec::with_capacity(MEASURED_RUNS);
    for _ in 0..MEASURED_RUNS {
        let started = Instant::now();
        run_status(&fixture, OBJECTIVE_COUNT);
        samples.push(started.elapsed());
    }
    server.join().expect("serveur de benchmark termine");

    let p95 = percentile_95(&samples);
    let report = json!({
        "v": 1,
        "criterion": "SC-008",
        "commit": git_commit(),
        "machine": machine_reference(),
        "system": system_reference(),
        "load_1m": load_average(),
        "charge": "campagne locale explicite ; autres charges à consigner par l'opérateur",
        "objectives": OBJECTIVE_COUNT,
        "warmup_runs": WARMUP_RUNS,
        "samples_ms": samples.iter().map(duration_ms).collect::<Vec<_>>(),
        "p95_ms": duration_ms(&p95),
        "budget_ms": duration_ms(&STATUS_P95_BUDGET),
    });
    eprintln!("SC-008 rapport={report}");
    write_optional_report(&report);
    assert!(
        p95 < STATUS_P95_BUDGET,
        "p95 status = {:?}, budget = {:?}",
        p95,
        STATUS_P95_BUDGET
    );
}

fn duration_ms(duration: &Duration) -> f64 {
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
    // `getloadavg` est disponible sur les deux plateformes cibles locales de
    // Bridget. Sa valeur est informative dans une campagne, jamais un oracle.
    (unsafe { libc::getloadavg(values.as_mut_ptr(), 3) } > 0).then_some(values[0])
}

fn write_optional_report(report: &Value) {
    let Some(path) = std::env::var_os("BRIDGET_PERF_REPORT") else {
        return;
    };
    fs::write(
        path,
        serde_json::to_vec_pretty(report).expect("rapport JSON"),
    )
    .expect("écriture référence locale SC-008");
}

fn run_status(fixture: &BenchmarkFixture, expected_objectives: usize) {
    let output = Command::new(env!("CARGO_BIN_EXE_maicie"))
        .args([
            "status",
            "--config",
            fixture.config.to_str().expect("configuration UTF-8"),
            "--json",
        ])
        .output()
        .expect("exécution maicie status");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: Value = serde_json::from_slice(&output.stdout).expect("sortie status JSON");
    assert_eq!(
        value["coordination"].as_array().map(Vec::len),
        Some(expected_objectives)
    );
    assert_eq!(value["runtime"][0]["agent"], "benchmark-agent");
    assert_eq!(value["runtime"][0]["stream_state"], "fresh");
}

fn percentile_95(samples: &[Duration]) -> Duration {
    assert!(!samples.is_empty(), "un p95 exige au moins un échantillon");
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let nearest_rank = (sorted.len() * 95).div_ceil(100).max(1);
    sorted[nearest_rank - 1]
}

fn serve_statuses(socket: &std::path::Path, runs: usize, ready: mpsc::Sender<()>) {
    let _ = fs::remove_file(socket);
    let listener = UnixListener::bind(socket).expect("socket benchmark");
    ready.send(()).expect("signal prêt");
    for run in 0..runs {
        accept_empty_guichet(&listener);
        accept_empty_coordination(&listener);
        accept_client(&listener);
        send_agent_list(&listener);
        send_snapshot(&listener, run);
    }
}

fn accept_empty_coordination(listener: &UnixListener) {
    let (stream, _) = listener.accept().expect("connexion coordination status");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(
        hello["capabilities"],
        json!(["maicie_guichet", "coordination_events_v2"])
    );
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome","version":1,"horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet","coordination_events_v2"]
        }),
    );
    assert_eq!(read_json(&mut reader)["type"], "coordination_subscribe");
    write_json(
        &mut writer,
        json!({"type":"coordination_snapshot_caught_up","v":2}),
    );
}

fn accept_empty_guichet(listener: &UnixListener) {
    let (stream, _) = listener.accept().expect("connexion service guichet status");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["service"], "maicie");
    assert_eq!(hello["capabilities"], json!(["maicie_guichet"]));
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"guichet_claim_next","v":1})
    );
    write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
}

fn accept_client(listener: &UnixListener) {
    let (stream, _) = listener.accept().expect("connexion client status");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    assert_eq!(read_json(&mut reader)["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
}

fn send_agent_list(listener: &UnixListener) {
    let (stream, _) = listener.accept().expect("connexion annuaire status");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(read_json(&mut reader), json!({"type":"ListAgents"}));
    write_json(
        &mut writer,
        json!({"type":"AgentList","agents":[{
            "name":"benchmark-agent","agent_type":"codex","connection_id":"benchmark-1",
            "host":"local","transport":"acp","state":"connected",
            "last_seen_secs":0,"reconnect_count":0
        }]}),
    );
}

fn send_snapshot(listener: &UnixListener, run: usize) {
    let (stream, _) = listener.accept().expect("connexion Attach status");
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"attach"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"attach"}));
    assert_eq!(read_json(&mut reader)["type"], "Subscribe");
    let subscription_id = format!("benchmark-{run}");
    write_json(
        &mut writer,
        json!({"type":"Subscribed","subscription_id":subscription_id}),
    );
    write_json(
        &mut writer,
        json!({"type":"SnapshotCaughtUp","subscription_id":subscription_id}),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().expect("clone socket")),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("lecture JSONL");
    serde_json::from_str(&line).expect("trame JSONL")
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    serde_json::to_writer(&mut *writer, &value).expect("écriture JSON");
    writer.write_all(b"\n").expect("délimiteur JSONL");
    writer.flush().expect("flush JSONL");
}

struct BenchmarkFixture {
    root: PathBuf,
    database: PathBuf,
    socket: PathBuf,
    config: PathBuf,
}

impl BenchmarkFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("maicie-status-benchmark-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("répertoire benchmark");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("permissions benchmark");
        let database = root.join("maicie.sqlite3");
        let socket = std::env::temp_dir().join(format!("mc-bench-{}.sock", Uuid::new_v4()));
        let config = root.join("maicie.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version":1,
                "bridget_socket":socket,
                "database_path":database,
                "durations":{"short_secs":30,"normal_secs":60,"long_secs":90},
                "status_capture_budget_ms":200,
                "profiles":[]
            }))
            .expect("configuration JSON"),
        )
        .expect("configuration benchmark");
        Self {
            root,
            database,
            socket,
            config,
        }
    }

    fn seed_accepted_objectives(&self, count: usize) {
        let mut store = MaicieStore::open(&self.database).expect("store benchmark");
        let candidates = [DelegationCandidate {
            name: "benchmark-agent".to_string(),
            tags: Vec::new(),
            available: true,
            dnd: false,
        }];
        for index in 0..count {
            let goal = format!("objectif benchmark {index}");
            let key = format!("status-benchmark-{index}");
            let created = delegate(
                &mut store,
                DurationClasses {
                    short_secs: 30,
                    normal_secs: 60,
                    long_secs: 90,
                },
                "maicie",
                &candidates,
                &DelegateRequest {
                    goal: &goal,
                    explicit_target: Some("benchmark-agent"),
                    required_tags: &[],
                    duration: ClasseDuree::Normale,
                    reply: false,
                    constat_id: None,
                    suite: maicie::domain::SuiteObjective::Aucune,
                    depends_on: &[],
                    references: &[],
                    idempotency_key: &key,
                    now: 100,
                    retry_until: 150,
                    dedup_retained_until: 200,
                    max_frame_bytes: 256 * 1024,
                },
            )
            .expect("délégation benchmark");
            let DelegateResult::Created(created) = created else {
                panic!("chaque clé benchmark est nouvelle");
            };
            store
                .record_lookup_issue(
                    created.message_id.unwrap(),
                    &IdempotencyIssue::Accepted { expires_at: 150 },
                    110,
                )
                .expect("issue terminale benchmark");
        }
    }
}

impl Drop for BenchmarkFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
        let _ = fs::remove_dir_all(&self.root);
    }
}
