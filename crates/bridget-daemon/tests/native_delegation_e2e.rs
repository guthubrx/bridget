//! Session148 : vrai daemon/superviseur/wrapper, fournisseur Claude/GLM fermé.
//! Aucun T3, secret utilisateur, réseau fournisseur ou modèle réel.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{NativeDelegationRequest, SpawnPosture};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::fs;
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const PARENT: &str = "14800000-0000-4000-8000-000000000001";
const INSTANCE: &str = "native148-parent-instance";
const ANSWER: &str = "fixture-glm5.3-answer-148";

// Le harnais existant fournit environnement fermé, client, Register et preuve.
// Ses watchdogs et nettoyages SIGKILL ne sont pas utilisés par cette recette.
struct OwnedChild(Child);
impl OwnedChild {
    fn wait(&mut self, budget: Duration) -> ExitStatus {
        let deadline = Instant::now() + budget;
        loop {
            if let Some(status) = self.0.try_wait().unwrap() {
                return status;
            }
            assert!(
                Instant::now() < deadline,
                "enfant isolé {} bloqué",
                self.0.id()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(&mut self) {
        if self.0.try_wait().ok().flatten().is_some() {
            return;
        }
        let observed = Command::new("/bin/ps")
            .args([
                "-ww",
                "-p",
                &self.0.id().to_string(),
                "-o",
                "ppid=",
                "-o",
                "command=",
            ])
            .output()
            .unwrap();
        let command = String::from_utf8_lossy(&observed.stdout);
        eprintln!(
            "arrêt enfant isolé pid={} expected={:?} ps_raw={:?} ps_stderr={:?}",
            self.0.id(),
            env!("CARGO_BIN_EXE_bridget"),
            command,
            String::from_utf8_lossy(&observed.stderr)
        );
        // EOF peut terminer MCP pendant ps : un zombie ne porte plus son argv.
        if self.0.try_wait().unwrap().is_some() {
            return;
        }
        assert_eq!(
            command
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<u32>().ok()),
            Some(std::process::id())
        );
        assert!(
            command.contains(env!("CARGO_BIN_EXE_bridget"))
                && !command.to_lowercase().contains("firefox")
        );
        assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
        self.wait(Duration::from_secs(10));
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
}

struct Fixture {
    root: PathBuf,
    daemon: Option<OwnedChild>,
    starts: usize,
    replies: Vec<BridgetMessage>,
}
impl Fixture {
    fn new() -> Self {
        Self::new_with_notify_timeout(2)
    }
    fn new_with_notify_timeout(notify_timeout_secs: u64) -> Self {
        let root = fs::canonicalize("/tmp").unwrap().join(format!(
            "n148-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..10]
        ));
        for relative in ["", "state", "provider", "tmp"] {
            support::private_dir(&root.join(relative)).unwrap();
        }
        support::private_write(
            &root.join("provider.py"),
            include_bytes!("fixtures/native_delegation_148.py"),
        )
        .unwrap();
        support::private_write(&root.join("evidence.jsonl"), []).unwrap();
        let registry = json!({"agents":{"fake-glm":{
            "command":"/usr/bin/python3", "args":[root.join("provider.py"),root.join("evidence.jsonl")],
            "protocol":"claude_stream_json", "permissions":"deny", "queue_capacity":4,"notify_timeout_secs":notify_timeout_secs,
            "forbidden_env":["ANTHROPIC_API_KEY","OPENAI_API_KEY","T3CODE_HOME","BRIDGET_T3_MCP_ENDPOINT","BRIDGET_T3_MCP_AUTHORIZATION"],"pass_env":[],
            "capabilities":{"execution_paths":["claude_stream_json"],"models":{"glm5.3":{"efforts":["high"]}}}
        }}});
        support::private_write(
            &root.join("state/agents.json"),
            serde_json::to_vec(&registry).unwrap(),
        )
        .unwrap();
        let mut fixture = Self {
            root,
            daemon: None,
            starts: 0,
            replies: Vec::new(),
        };
        fixture.start();
        fixture
    }
    fn start(&mut self) {
        assert!(self.daemon.is_none());
        self.starts += 1;
        let log = self.root.join(format!("daemon-{}.log", self.starts));
        support::private_write(&log, []).unwrap();
        self.daemon = Some(OwnedChild(
            support::isolated_command(&self.root)
                .arg("daemon")
                .env("TMPDIR", self.root.join("tmp"))
                .env("RUST_LOG", "info")
                .stdout(Stdio::null())
                .stderr(fs::OpenOptions::new().append(true).open(log).unwrap())
                .spawn()
                .unwrap(),
        ));
        let deadline = Instant::now() + Duration::from_secs(10);
        while std::os::unix::net::UnixStream::connect(support::socket(&self.root)).is_err() {
            assert!(
                Instant::now() < deadline,
                "daemon isolé absent : {}",
                self.root.display()
            );
            assert!(
                self.daemon
                    .as_mut()
                    .unwrap()
                    .0
                    .try_wait()
                    .unwrap()
                    .is_none(),
                "daemon refusé : {}",
                self.root.display()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn stop(&mut self) {
        drop(self.daemon.take());
    }
    fn human_grant(&self) {
        let (mut master, mut slave) = (-1, -1);
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            0
        );
        let _master = unsafe { fs::File::from_raw_fd(master) };
        let slave = unsafe { fs::File::from_raw_fd(slave) };
        let log = self.root.join("grant.log");
        support::private_write(&log, []).unwrap();
        let mut child = OwnedChild(
            support::isolated_command(&self.root)
                .args([
                    "delegate-grant",
                    PARENT,
                    "--cwd",
                    self.root.to_str().unwrap(),
                    "--posture",
                    "discovery",
                ])
                .env("TMPDIR", self.root.join("tmp"))
                .stdin(slave.try_clone().unwrap())
                .stdout(slave)
                .stderr(fs::OpenOptions::new().append(true).open(&log).unwrap())
                .spawn()
                .unwrap(),
        );
        assert!(
            child.wait(Duration::from_secs(10)).success(),
            "grant humain refusé : {}",
            fs::read_to_string(log).unwrap()
        );
    }
    fn receive(&mut self, parent: &mut support::Client) -> DaemonToWrapper {
        match parent.receive() {
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                parent.send(WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                });
                self.replies.push(message);
                self.receive(parent)
            }
            DaemonToWrapper::Deliver(message)
            | DaemonToWrapper::DeliverExecution { message, .. } => {
                self.replies.push(message);
                self.receive(parent)
            }
            other => other,
        }
    }
    fn request(&mut self, parent: &mut support::Client, request: NativeDelegationRequest) -> Value {
        parent.send(WrapperToDaemon::NativeDelegation { request });
        for _ in 0..128 {
            if let DaemonToWrapper::NativeDelegationResult { result } = self.receive(parent) {
                return result;
            }
        }
        panic!("réponse native absente");
    }
    fn status_until(&mut self, parent: &mut support::Client, task: &str, expected: &str) -> Value {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let status = self.request(
                parent,
                NativeDelegationRequest::Status {
                    task_id: task.into(),
                },
            );
            if status["status"] == expected {
                return status;
            }
            assert_ne!(
                status["status"],
                "failed",
                "échec core natif : {status} ; {}",
                self.root.display()
            );
            assert!(
                Instant::now() < deadline,
                "état natif non atteint : {status} ; {}",
                self.root.display()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
    fn evidence(&self) -> Vec<Value> {
        fs::read_to_string(self.root.join("evidence.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop();
        if !thread::panicking() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}
fn delegate(root: &Path, key: &str, task: &str) -> NativeDelegationRequest {
    NativeDelegationRequest::Delegate {
        request_id: key.into(),
        agent_type: "fake-glm".into(),
        model: "glm5.3".into(),
        effort: Some("high".into()),
        task: task.into(),
        cwd: root.to_string_lossy().into_owned(),
        posture: Some(SpawnPosture::Discovery),
        reply_timeout: None,
    }
}

#[test]
fn native151_e2e_deleguer_puis_attendre_et_message_unique_apres_expiration() {
    let mut f = Fixture::new_with_notify_timeout(15);
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    f.human_grant();
    let created = f.request(
        &mut parent,
        delegate(&f.root.clone(), "wait-151", "WAIT_FILE_151"),
    );
    let id = created["task_id"].as_str().unwrap().to_string();
    let fixture_deadline = Instant::now() + Duration::from_secs(10);
    while f
        .evidence()
        .iter()
        .all(|event| event["event"] != "waiting_file_151")
    {
        assert!(
            Instant::now() < fixture_deadline,
            "fournisseur synthétique non bloqué sur WAIT_FILE_151"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let expired = f.request(
        &mut parent,
        NativeDelegationRequest::Wait {
            task_ids: vec![id.clone()],
            wait_timeout: Some(1),
        },
    );
    assert_eq!(expired["timed_out"], true, "{expired}");
    assert!(f.replies.is_empty());
    let db_path = f.root.join("state/bridget.db");
    let observed_task = id.clone();
    let (observation_ready, observation_started) = std::sync::mpsc::sync_channel(0);
    let (ready, ready_at, capture_before, capture_after) = thread::scope(|scope| {
        let observer = scope.spawn(move || {
            let db = rusqlite::Connection::open_with_flags(
                db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            let capture_deadline = Instant::now() + Duration::from_secs(10);
            let mut last_null = None;
            loop {
                // Début de lecture : un snapshot NULL peut finir après un commit concurrent.
                let query_started = Instant::now();
                let result: Option<String> = db
                    .query_row(
                        "SELECT json_extract(payload,'$.result') FROM native_delegations WHERE task_id=?1",
                        [&observed_task],
                        |row| row.get(0),
                    )
                    .unwrap();
                let query_finished = Instant::now();
                match result {
                    None => {
                        if last_null.is_none() {
                            observation_ready.send(()).unwrap();
                        }
                        last_null = Some(query_started);
                    }
                    Some(result) => {
                        assert_eq!(result, ANSWER);
                        return (
                            last_null.expect("capture non observée : première lecture déjà complète"),
                            query_finished,
                        );
                    }
                }
                assert!(
                    Instant::now() < capture_deadline,
                    "réponse enfant non capturée dans la base privée"
                );
                thread::sleep(Duration::from_millis(5));
            }
        });
        observation_started
            .recv_timeout(Duration::from_secs(5))
            .expect("observateur SQLite absent avant la libération");
        support::private_write(&f.root.join("release-151"), b"go").unwrap();
        let ready = f.request(
            &mut parent,
            NativeDelegationRequest::Wait {
                task_ids: vec![id.clone()],
                wait_timeout: Some(5),
            },
        );
        let ready_at = Instant::now();
        let (capture_before, capture_after) = observer.join().unwrap();
        (ready, ready_at, capture_before, capture_after)
    });
    assert_eq!(ready["timed_out"], false, "{ready}");
    assert_eq!(ready["ready"][0]["summary"], ANSWER);
    assert!(ready["ready"][0].get("result").is_none());
    let ready_elapsed = ready_at.duration_since(capture_before);
    assert!(ready_elapsed <= Duration::from_secs(2), "{ready_elapsed:?}");
    eprintln!(
        "SC001 : borne antérieure de capture SQLite -> attente rendue en {} ms, capture encadrée sur {} ms, zéro Status avant résultat",
        ready_elapsed.as_millis(),
        capture_after.duration_since(capture_before).as_millis()
    );
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let status = f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: id.clone(),
            },
        );
        if status["mailbox"]["state"] == "delivered" {
            break;
        }
        assert!(Instant::now() < deadline, "ACK absent : {status}");
        thread::sleep(Duration::from_millis(20));
    }
    let replies: Vec<_> = f
        .replies
        .iter()
        .filter(|m| m.in_reply_to.as_deref() == created["message_id"].as_str())
        .collect();
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].body, ANSWER);
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    // tick confirme les arrêts supervisés après la remise, puis persiste ce fait.
    let cleanup_deadline = Instant::now() + Duration::from_secs(10);
    let task = loop {
        let payload: String = db
            .query_row(
                "SELECT payload FROM native_delegations WHERE task_id=?1",
                [&id],
                |r| r.get(0),
            )
            .unwrap();
        let task: Value = serde_json::from_str(&payload).unwrap();
        if task["cleanup_done"] == true {
            break task;
        }
        assert!(
            Instant::now() < cleanup_deadline,
            "nettoyage supervisé non confirmé : {task} ; {}",
            f.root.display()
        );
        thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(task["cleanup_done"], true);
    assert_eq!(task["result_sent"], true);
    parent.send(WrapperToDaemon::Unregister);
}

#[test]
fn native151_e2e_boite_echec_pending_claimed_delivered() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::AsRawFd;

    // Lire une fois les deux exemples concernés. Seuls les IDs seront remplacés.
    let section = include_str!("../../../skills/bridget/SKILL.md")
        .split_once("### Attendre et déclarer la lecture - contrat 151\n")
        .unwrap()
        .1
        .split("### Privilégier le même projet — contrat 138\n")
        .next()
        .unwrap();
    let mut published = section
        .split("```json\n")
        .skip(1)
        .flat_map(|block| {
            serde_json::Deserializer::from_str(block.split("```").next().unwrap())
                .into_iter::<Value>()
        })
        .map(Result::unwrap)
        .filter(|example| {
            matches!(
                example["name"].as_str(),
                Some("bridget_task_wait" | "bridget_task_ack")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        published,
        vec![
            json!({"name":"bridget_task_wait","arguments":{"task_ids":["<task_id>"],"wait_timeout":600}}),
            json!({"name":"bridget_task_ack","arguments":{"task_ids":["<task_id>"]}}),
        ]
    );
    let mut f = Fixture::new_with_notify_timeout(15);
    let socket = support::socket(&f.root);
    let mut parent = support::register_agent_as(&socket, PARENT, INSTANCE);
    f.human_grant();
    let mut auxiliary = support::Client::connect(&socket);
    support::attest_agent(&socket, &mut auxiliary, PARENT, INSTANCE);

    parent.send(WrapperToDaemon::TurnState { in_progress: true });
    // Même socket : cette réponse prouve que TurnState a précédé la délégation.
    let barrier = f.request(
        &mut parent,
        NativeDelegationRequest::Status {
            task_id: "synthetic-before-delegation151".into(),
        },
    );
    assert_eq!(barrier["code"], "task_unavailable", "{barrier}");
    let created = f.request(
        &mut auxiliary,
        delegate(&f.root.clone(), "mailbox-failure-151", "FAIL_151"),
    );
    let id = created["task_id"].as_str().unwrap().to_string();
    for example in &mut published {
        example["arguments"]["task_ids"][0] = json!(id);
    }
    let name_file = f.root.join("state/mcp-agent-name-published151");
    support::private_write(&name_file, PARENT).unwrap();
    let mut mcp = OwnedChild(
        support::isolated_command(&f.root)
            .arg("mcp")
            .env("BRIDGET_AGENT_ID_FILE", &name_file)
            .env("BRIDGET_AGENT_INSTANCE_ID", INSTANCE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut input = mcp.0.stdin.take().unwrap();
    let mut output = BufReader::new(mcp.0.stdout.take().unwrap());
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{}})
    )
    .unwrap();
    input.flush().unwrap();
    let mut ready_fd = libc::pollfd {
        fd: output.get_ref().as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert!(
        unsafe { libc::poll(&mut ready_fd, 1, 8000) } > 0,
        "initialize publié absent"
    );
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    let initialized: Value = serde_json::from_str(&line).unwrap();
    assert!(
        initialized["result"]["protocolVersion"].is_string(),
        "{initialized}"
    );
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","method":"notifications/initialized"})
    )
    .unwrap();
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":published[0]})
    )
    .unwrap();
    input.flush().unwrap();
    assert!(
        unsafe { libc::poll(&mut ready_fd, 1, 8000) } > 0,
        "TaskWait publié absent"
    );
    line.clear();
    output.read_line(&mut line).unwrap();
    let waited: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(waited["id"], 1);
    assert_ne!(waited["result"]["isError"], true, "{waited}");
    let ready = &waited["result"]["structuredContent"];
    eprintln!(
        "JSON TaskWait publié exécuté sur vrai MCP : paramètres={}, réponse={ready}",
        published[0]
    );
    assert_eq!(ready["timed_out"], false, "{ready}");
    assert_eq!(ready["ready"][0]["status"], "failed", "{ready}");
    assert_eq!(ready["ready"][0]["mailbox"]["state"], "pending", "{ready}");
    let pending = f.request(
        &mut auxiliary,
        NativeDelegationRequest::Status {
            task_id: id.clone(),
        },
    );
    assert_eq!(pending["mailbox"]["state"], "pending", "{pending}");

    parent.send(WrapperToDaemon::TurnState { in_progress: false });
    // Lecture brute : aucun ACK implicite de Fixture.receive avant l'état claimed.
    let notice_deadline = Instant::now() + Duration::from_secs(8);
    let (delivery_id, delivery_generation, notice) = loop {
        let remaining = notice_deadline.saturating_duration_since(Instant::now());
        assert!(
            !remaining.is_zero(),
            "avis natif absent après huit secondes"
        );
        parent
            .reader
            .get_ref()
            .set_read_timeout(Some(remaining))
            .unwrap();
        match parent.receive() {
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                assert_eq!(message.id, format!("native-failure-{id}"));
                break (delivery_id, delivery_generation, message);
            }
            DaemonToWrapper::Deliver(message) => {
                assert_eq!(message.from, "bridget");
                assert_eq!(message.to, PARENT);
                assert!(!message.reply);
                assert!(
                    message
                        .body
                        .contains(created["message_id"].as_str().unwrap())
                );
                eprintln!("Avis hérité de demande observé avant l'avis natif : {message:?}");
                f.replies.push(message);
            }
            other => panic!("avis d'échec idempotent attendu : {other:?}"),
        }
    };
    parent
        .reader
        .get_ref()
        .set_read_timeout(Some(support::CLIENT_READ_TIMEOUT))
        .unwrap();
    assert_eq!(notice.id, format!("native-failure-{id}"));
    let claimed = f.request(
        &mut auxiliary,
        NativeDelegationRequest::Status {
            task_id: id.clone(),
        },
    );
    assert_eq!(claimed["mailbox"]["state"], "claimed", "{claimed}");
    f.replies.push(notice);
    parent.send(WrapperToDaemon::DeliverAcked {
        delivery_id,
        delivery_generation,
    });
    let delivered_deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let delivered = f.request(
            &mut auxiliary,
            NativeDelegationRequest::Status {
                task_id: id.clone(),
            },
        );
        if delivered["mailbox"]["state"] == "delivered" {
            break;
        }
        assert!(Instant::now() < delivered_deadline, "{delivered}");
        thread::sleep(Duration::from_millis(20));
    }

    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let cleanup_deadline = Instant::now() + Duration::from_secs(10);
    let task = loop {
        let payload: String = db
            .query_row(
                "SELECT payload FROM native_delegations WHERE task_id=?1",
                [&id],
                |row| row.get(0),
            )
            .unwrap();
        let task: Value = serde_json::from_str(&payload).unwrap();
        if task["cleanup_done"] == true {
            break task;
        }
        assert!(Instant::now() < cleanup_deadline, "{task}");
        thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(task["cleanup_done"], true);
    assert_eq!(task["failure_sent"], true);
    // Deux ticks réels supplémentaires : collecter toute remise dupliquée sur le primaire.
    let quiet_deadline = Instant::now() + Duration::from_millis(2100);
    while Instant::now() < quiet_deadline {
        let status = f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: id.clone(),
            },
        );
        assert_eq!(status["mailbox"]["state"], "delivered", "{status}");
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        f.replies
            .iter()
            .filter(|message| message.id == format!("native-failure-{id}"))
            .count(),
        1
    );
    eprintln!("SC006 : pending -> claimed -> delivered, un avis, nettoyage durable confirmé");
    // L'accusé de lecture vient après toute preuve de remise et de nettoyage.
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":published[1]})
    )
    .unwrap();
    input.flush().unwrap();
    assert!(
        unsafe { libc::poll(&mut ready_fd, 1, 8000) } > 0,
        "Ack publié absent"
    );
    line.clear();
    output.read_line(&mut line).unwrap();
    let acknowledged: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(acknowledged["id"], 2);
    assert_ne!(acknowledged["result"]["isError"], true, "{acknowledged}");
    assert_eq!(
        acknowledged["result"]["structuredContent"],
        json!({"version":1,"acknowledged":[id],"already":[]})
    );
    let after_ack = f.request(
        &mut auxiliary,
        NativeDelegationRequest::Status {
            task_id: id.clone(),
        },
    );
    assert_eq!(after_ack["mailbox"]["state"], "acknowledged", "{after_ack}");
    let acknowledged_at: Option<i64> = db
        .query_row(
            "SELECT json_extract(payload,'$.acknowledged_at') FROM native_delegations WHERE task_id=?1",
            [&id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(acknowledged_at.is_some(), "date de lecture durable absente");
    eprintln!(
        "JSON Ack publié exécuté après SC006 : paramètres={}, réponse={acknowledged}",
        published[1]
    );
    drop(input);
    assert!(mcp.wait(Duration::from_secs(6)).success());
    parent.send(WrapperToDaemon::Unregister);
}

#[test]
fn native151_e2e_mcp_eof_ferme_quatre_attentes_sans_perdre_le_resultat() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::fd::AsRawFd;
    let mut f = Fixture::new_with_notify_timeout(15);
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    f.human_grant();
    let created = f.request(
        &mut parent,
        delegate(&f.root.clone(), "eof-151", "WAIT_FILE_151"),
    );
    let id = created["task_id"].as_str().unwrap().to_string();
    let name_file = f.root.join("state/mcp-agent-name");
    support::private_write(&name_file, PARENT).unwrap();
    let mut mcp = OwnedChild(
        support::isolated_command(&f.root)
            .arg("mcp")
            .env("BRIDGET_AGENT_ID_FILE", &name_file)
            .env("BRIDGET_AGENT_INSTANCE_ID", INSTANCE)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let mut input = mcp.0.stdin.take().unwrap();
    let mut output = BufReader::new(mcp.0.stdout.take().unwrap());
    writeln!(input,"{}",json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"fixture151","version":"1"}}})).unwrap();
    input.flush().unwrap();
    let mut line = String::new();
    output.read_line(&mut line).unwrap();
    assert!(line.contains("protocolVersion"), "{line}");
    writeln!(
        input,
        "{}",
        json!({"jsonrpc":"2.0","method":"notifications/initialized"})
    )
    .unwrap();
    for call_id in [1, 2, 3, 4] {
        writeln!(input,"{}",json!({"jsonrpc":"2.0","id":call_id,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":[id],"wait_timeout":3600}}})).unwrap();
    }
    writeln!(input,"{}",json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":[id],"wait_timeout":3600}}})).unwrap();
    input.flush().unwrap();
    let mut ready_fd = libc::pollfd {
        fd: output.get_ref().as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    assert!(
        unsafe { libc::poll(&mut ready_fd, 1, 5000) } > 0,
        "capacité MCP non rendue"
    );
    line.clear();
    output.read_line(&mut line).unwrap();
    assert!(
        line.contains("wait_capacity"),
        "quatre attentes ouvertes : {line}"
    );
    thread::sleep(Duration::from_millis(250));
    let start = Instant::now();
    drop(input);
    assert!(mcp.wait(Duration::from_secs(6)).success());
    let eof_elapsed = start.elapsed();
    assert!(eof_elapsed < Duration::from_secs(6));
    eprintln!(
        "MCP151 EOF : quatre attentes de 3600 s fermées en {} ms",
        eof_elapsed.as_millis()
    );
    let mut remaining = String::new();
    output.read_to_string(&mut remaining).unwrap();
    assert!(
        remaining.trim().is_empty(),
        "les quatre attentes doivent rester ouvertes jusqu'à EOF : {remaining}"
    );
    support::private_write(&f.root.join("release-151"), b"go").unwrap();
    let ready = f.request(
        &mut parent,
        NativeDelegationRequest::Wait {
            task_ids: vec![id.clone()],
            wait_timeout: Some(5),
        },
    );
    assert_eq!(ready["timed_out"], false, "{ready}");
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let status = f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: id.clone(),
            },
        );
        if status["mailbox"]["state"] == "delivered" {
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        f.replies
            .iter()
            .filter(|m| m.in_reply_to.as_deref() == created["message_id"].as_str())
            .count(),
        1
    );
    parent.send(WrapperToDaemon::Unregister);
}

#[test]
fn native148_sans_t3_retry_reponse_correllee_annulation_et_reprise_durable() {
    let mut f = Fixture::new();
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    let denied = f.request(
        &mut parent,
        delegate(&f.root.clone(), "without-grant", "mission locale"),
    );
    assert_eq!(denied["code"], "delegation_grant_required");
    f.human_grant();
    let request = delegate(
        &f.root,
        "retry-same-mission",
        "Inspecte les faits locaux de la fixture148.",
    );
    let first = f.request(&mut parent, request.clone());
    assert_eq!(first["version"], 1);
    let task = first["task_id"]
        .as_str()
        .expect("mission admise")
        .to_string();
    for _ in 0..10 {
        let replay = f.request(&mut parent, request.clone());
        assert_eq!(replay["task_id"], first["task_id"]);
        assert_eq!(replay["child_agent_id"], first["child_agent_id"]);
        assert_eq!(replay["message_id"], first["message_id"]);
    }
    let finished = f.status_until(&mut parent, &task, "result_available");
    assert_eq!(finished["result"], ANSWER);
    let deadline = Instant::now() + Duration::from_secs(5);
    while f
        .replies
        .iter()
        .all(|reply| reply.in_reply_to.as_deref() != first["message_id"].as_str())
    {
        f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: task.clone(),
            },
        );
        assert!(Instant::now() < deadline, "réponse corrélée non livrée");
        thread::sleep(Duration::from_millis(20));
    }
    let replies: Vec<_> = f
        .replies
        .iter()
        .filter(|reply| reply.in_reply_to.as_deref() == first["message_id"].as_str())
        .collect();
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].from, first["child_agent_id"].as_str().unwrap());
    assert_eq!(replies[0].to, PARENT);
    assert_eq!(replies[0].body, ANSWER);
    let evidence = f.evidence();
    assert_eq!(
        evidence
            .iter()
            .filter(|event| event["event"] == "started")
            .count(),
        1
    );
    assert_eq!(
        evidence
            .iter()
            .filter(|event| event["event"] == "prompt")
            .count(),
        1
    );
    let prompt = evidence
        .iter()
        .find(|event| event["event"] == "prompt")
        .unwrap();
    assert_eq!(
        prompt["resume_card"], true,
        "carte conservée dans l'unique mission"
    );
    assert_eq!(prompt["mission"], true, "mission originale conservée");
    assert_eq!(evidence[0]["model"], "glm5.3");
    assert_eq!(evidence[0]["t3_present"], false);
    assert!(!f.root.join("provider/.t3").exists());
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM native_delegations", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(db);

    let blocked_request = delegate(&f.root, "cancel-mission", "WAIT_CANCEL_148");
    let blocked = f.request(&mut parent, blocked_request);
    let blocked_id = blocked["task_id"].as_str().unwrap().to_string();
    let deadline = Instant::now() + Duration::from_secs(15);
    while f.evidence().iter().all(|event| event["blocked"] != true) {
        let status = f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: blocked_id.clone(),
            },
        );
        assert_ne!(
            status["status"], "failed",
            "mission bloquée refusée : {status}"
        );
        assert!(Instant::now() < deadline, "faux fournisseur non sollicité");
        thread::sleep(Duration::from_millis(20));
    }
    f.request(
        &mut parent,
        NativeDelegationRequest::Cancel {
            task_id: blocked_id.clone(),
        },
    );
    let cancelled = f.status_until(&mut parent, &blocked_id, "cancelled");
    assert_eq!(cancelled["result"], Value::Null);
    assert!(
        f.replies
            .iter()
            .all(|reply| reply.in_reply_to.as_deref() != blocked["message_id"].as_str())
    );
    // Arrêt officiel de l'enfant terminé avant redémarrage du daemon.
    parent.send(WrapperToDaemon::StopOrder {
        agent_id: first["child_agent_id"].as_str().unwrap().into(),
        command_id: "native148-stop-completed".into(),
    });
    loop {
        if let DaemonToWrapper::StopResult { outcome, .. } = f.receive(&mut parent) {
            assert!(matches!(
                outcome,
                bridget_transport::StopOutcome::Stopped | bridget_transport::StopOutcome::NotFound
            ));
            break;
        }
    }
    drop(parent);
    f.stop();
    f.start();
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    let recovered = f.request(&mut parent, request);
    let mut recovered_stable = recovered.clone();
    let mut finished_stable = finished.clone();
    // La boîte reflète maintenant l'ACK reçu entre les deux lectures ; les autres clés restent stables.
    recovered_stable.as_object_mut().unwrap().remove("mailbox");
    finished_stable.as_object_mut().unwrap().remove("mailbox");
    assert_eq!(recovered_stable, finished_stable);
    assert_eq!(recovered["mailbox"]["state"], "delivered");
    let cancelled = f.request(
        &mut parent,
        NativeDelegationRequest::Status {
            task_id: blocked_id,
        },
    );
    assert_eq!(cancelled["status"], "cancelled");
    assert_eq!(
        f.evidence()
            .iter()
            .filter(|event| event["event"] == "started")
            .count(),
        2
    );
    parent.send(WrapperToDaemon::Unregister);
}
