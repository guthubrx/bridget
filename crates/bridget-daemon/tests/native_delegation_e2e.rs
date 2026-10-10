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
            "protocol":"claude_stream_json", "permissions":"deny", "queue_capacity":4,"notify_timeout_secs":2,
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
    }
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
    assert_eq!(recovered, finished);
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
