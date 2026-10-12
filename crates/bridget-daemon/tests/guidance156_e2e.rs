//! Session156 : parent attesté → daemon → wrapper natif → fournisseur fermé.
//! Réemploi local du motif native_delegation_e2e ; aucun réseau/modèle réel.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;
use bridget_core::{BridgetMessage, MessageIntent};
use bridget_daemon::desired_state::{DesiredStateStore, path_for_daemon_db};
use bridget_daemon::idempotency::IdempotencyStore;
use bridget_daemon::managed_process::{
    ManagedMarker, ManagedMarkerStore, group_exists, process_birth, signal_group, wait_group_gone,
};
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    NativeDelegationRequest, PresenceMode, ProviderPermissions, SpawnPosture,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::os::{fd::FromRawFd, unix::fs::PermissionsExt};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const PARENT: &str = "15600000-0000-4000-8000-000000000001";
const INSTANCE: &str = "guidance156-parent-instance";
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
    native_groups: Vec<ManagedMarker>,
}
impl Fixture {
    fn new(codex: bool) -> Self {
        let root = std::env::temp_dir().join(format!(
            "n156-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..10]
        ));
        for relative in ["", "state", "provider", "tmp", "bin", ".git"] {
            support::private_dir(&root.join(relative)).unwrap();
        }
        let script = if codex {
            provider_codex156()
        } else {
            provider_claude156()
        };
        let provider = root.join("provider.py");
        support::private_write(&provider, &script).unwrap();
        let (command, args) = if codex {
            let cli = compile_provider152(&root, &script, "codex");
            (cli.to_string_lossy().into_owned(), json!(["app-server"]))
        } else {
            (
                "/usr/bin/python3".to_string(),
                json!([provider, root.join("evidence.jsonl")]),
            )
        };
        support::private_write(&root.join("evidence.jsonl"), []).unwrap();
        let (name, protocol, model) = if codex {
            ("fake-codex", "codex_app_server", "gpt-5.6-terra")
        } else {
            ("fake-glm", "claude_stream_json", "glm5.3")
        };
        let registry = json!({"agents":{(name):{
            "command":command, "args":args, "protocol":protocol,
            "permissions":"deny", "queue_capacity":4, "notify_timeout_secs":30,
            "forbidden_env":["ANTHROPIC_API_KEY","OPENAI_API_KEY","T3CODE_HOME","BRIDGET_T3_MCP_ENDPOINT","BRIDGET_T3_MCP_AUTHORIZATION"], "pass_env":[],
            "capabilities":{"execution_paths":[protocol],"models":{(model):{"efforts":["high"]}}}
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
            native_groups: Vec::new(),
        };
        fixture.start();
        fixture
    }
    fn start(&mut self) {
        assert!(self.daemon.is_none());
        self.starts += 1;
        let log = self.root.join(format!("daemon-{}.log", self.starts));
        support::private_write(&log, []).unwrap();
        let mut command = support::isolated_command(&self.root);
        self.daemon = Some(OwnedChild(
            command
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
    fn marker_store(&self) -> ManagedMarkerStore {
        ManagedMarkerStore::at_directory(self.root.join("state/managed"))
    }
    fn remember_native_group(&mut self, name: &str) -> ManagedMarker {
        let marker = self.marker_store().load(name).unwrap();
        self.native_groups.push(marker.clone());
        marker
    }
    fn await_native_group_gone(pgid: u32) {
        let gone =
            wait_group_gone(pgid, Duration::from_secs(6), Duration::from_millis(20)).unwrap();
        if !thread::panicking() {
            assert!(gone, "groupe natif isolé encore présent : {pgid}");
        }
    }
    fn stop_native_groups(&mut self) {
        for marker in &self.native_groups {
            // Le marqueur était lu pendant que cette fixture possédait l'enfant.
            // Une naissance différente interdit tout signal au PID réutilisé.
            if !group_exists(marker.pgid).unwrap_or(false) {
                continue;
            }
            let Some(birth) = process_birth(marker.pgid).ok() else {
                // Le leader peut finir avant ses descendants. Ne pas leur
                // envoyer de signal sans naissance attestée ; attendre leur EOF.
                Self::await_native_group_gone(marker.pgid);
                continue;
            };
            if birth != marker.birth {
                continue;
            }
            let observed = Command::new("/bin/ps")
                .args(["-ww", "-p", &marker.pgid.to_string(), "-o", "command="])
                .output()
                .unwrap();
            let command = String::from_utf8_lossy(&observed.stdout);
            match process_birth(marker.pgid).ok() {
                Some(birth) if birth == marker.birth => {}
                None => {
                    Self::await_native_group_gone(marker.pgid);
                    continue;
                }
                Some(_) => continue,
            }
            assert!(command.contains(env!("CARGO_BIN_EXE_bridget")));
            assert!(!command.to_lowercase().contains("firefox"));
            signal_group(marker.pgid, libc::SIGTERM).unwrap();
            Self::await_native_group_gone(marker.pgid);
        }
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
            DaemonToWrapper::Deliver { message, .. }
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
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop();
        self.stop_native_groups();
        if !thread::panicking() {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
}
fn delegate(root: &Path, key: &str, task: &str) -> NativeDelegationRequest {
    NativeDelegationRequest::Delegate {
        resume_on_limit: None,
        fallback: Vec::new(),
        request_id: key.into(),
        agent_type: "fake-glm".into(),
        model: "glm5.3".into(),
        effort: Some("high".into()),
        task: task.into(),
        cwd: root.to_string_lossy().into_owned(),
        posture: Some(SpawnPosture::Discovery),
        reply_timeout: None,
        child_requests: None,
        workspace: None,
        keep_alive_secs: None,
        brief: None,
    }
}

fn compile_provider152(root: &Path, script: &str, cli_name: &str) -> PathBuf {
    // Montage149 : CLI directe du PATH, profil privé et ancêtres bornés par .git.
    // Le texte Python est incorporé : la SHA du CLI capturé couvre le programme.
    let program = format!(
        "__file__ = {}\n{script}",
        serde_json::to_string(&root.join("provider.py")).unwrap()
    );
    let embedded = program
        .as_bytes()
        .iter()
        .map(|byte| format!("\\x{byte:02x}"))
        .collect::<String>();
    let source = root.join("provider.c");
    support::private_write(
        &source,
        format!(
            r#"#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
static const char program[] = "{embedded}";
int main(int argc, char **argv) {{
    char **args = calloc((size_t)argc + 3, sizeof(*args));
    if (!args) {{ perror("calloc"); return 127; }}
    args[0] = "/usr/bin/python3";
    args[1] = "-c";
    args[2] = (char *)program;
    for (int index = 1; index < argc; ++index) args[index + 2] = argv[index];
    execv(args[0], args);
    perror("execv fixture152");
    free(args);
    return 127;
}}
"#
        ),
    )
    .unwrap();
    let cli = root.join("bin").join(cli_name);
    let compiled = Command::new("/usr/bin/cc")
        .args(["-std=c99", "-Wall", "-Wextra", "-Werror", "-o"])
        .arg(&cli)
        .arg(&source)
        .env_clear()
        .env("HOME", root.join("provider"))
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("TMPDIR", root.join("tmp"))
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "CLI fermée152 non compilée : {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    fs::set_permissions(&cli, fs::Permissions::from_mode(0o700)).unwrap();
    cli
}

fn native_task152(f: &Fixture, id: &str) -> Value {
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let payload: String = db
        .query_row(
            "SELECT payload FROM native_delegations WHERE task_id=?1",
            [id],
            |row| row.get(0),
        )
        .unwrap();
    serde_json::from_str(&payload).unwrap()
}

fn desired_store152(f: &Fixture) -> DesiredStateStore {
    DesiredStateStore::at_path(path_for_daemon_db(&f.root.join("state/bridget.db")))
}

fn evidence152(f: &Fixture) -> Vec<Value> {
    let bytes = fs::read(f.root.join("evidence.jsonl")).unwrap();
    // Une écriture concurrente peut laisser uniquement la dernière ligne incomplète.
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| line.last() == Some(&b'\n'))
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
}

fn wait_evidence152(f: &Fixture, event: &str, instance: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let observations = evidence152(f);
        if let Some(found) = observations
            .iter()
            .find(|value| value["event"] == event && value["instance_id"] == instance)
        {
            return found.clone();
        }
        assert!(
            Instant::now() < deadline,
            "{event} absent pour {instance} : {}",
            f.root.display()
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_native_ready152(f: &Fixture, id: &str, previous_instance: &str) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        // working survit au crash. Relire jusqu'à la génération du fournisseur
        // réellement lancé, plutôt que figer une instance encore ancienne ou synthétique.
        let task = native_task152(f, id);
        assert_ne!(task["state"], "failed", "reprise native refusée : {task}");
        if task["state"] == "working"
            && let Some(instance) = task["child_instance"].as_str()
            && instance != previous_instance
            && evidence152(f).iter().any(|event| {
                event["event"] == "started"
                    && event["instance_id"] == instance
                    && event["agent_id"] == task["child"]
            })
        {
            return task;
        }
        assert!(
            Instant::now() < deadline,
            "nouvelle génération native non prête après {previous_instance} : {task} ; {}",
            f.root.display()
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn assert_native_identity152(f: &Fixture, id: &str, mission: &str) -> String {
    let task = native_task152(f, id);
    assert_eq!(task["mission"], mission);
    let child = task["child"].as_str().unwrap();
    let instance = task["child_instance"].as_str().unwrap();
    let desired = desired_store152(f).load().unwrap();
    let entry = desired.equipiers.get(child).unwrap();
    let desired_link = entry.agent_link.as_ref().expect("lien natif conservé");
    assert_eq!(desired_link.role, "native_delegate");
    assert_eq!(desired_link.delegation_id.as_deref(), Some(id));
    let store = IdempotencyStore::open(&f.root.join("state/bridget.db")).unwrap();
    let link = store
        .agent_link_by_id(&desired_link.link_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.child_instance_id, instance);
    assert_eq!(link.role, "native_delegate");
    assert_eq!(link.delegation_id.as_deref(), Some(id));
    let started = wait_evidence152(f, "started", instance);
    assert_eq!(started["native_bootstrap"]["instance_id"], instance);
    assert_eq!(started["native_bootstrap"]["mission_id"], mission);
    instance.to_string()
}

fn provider_codex156() -> String {
    r#"import json
import os
import sys
import time

evidence = os.path.join(os.path.dirname(os.path.abspath(__file__)), "evidence.jsonl")
instance = os.environ["BRIDGET_AGENT_INSTANCE_ID"]
thread_id = "thread155-" + instance
turn_id = "turn155-" + instance
sandbox = json.loads('FIXTURE_SANDBOX155')
assert os.environ.get("BRIDGET_NATIVE_CHILD_REQUESTS") == "forward"
assert not any(key in os.environ for key in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "T3CODE_HOME", "BRIDGET_T3_MCP_ENDPOINT", "BRIDGET_T3_MCP_AUTHORIZATION"))

def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps(dict(event=event, instance_id=instance, **values)) + "\n")

def emit(frame):
    print(json.dumps(frame), flush=True)

record("started", provider="closed_codex_fixture", agent_id=os.environ["BRIDGET_AGENT_ID"],
       native_bootstrap=json.loads(os.environ["BRIDGET_NATIVE_MISSION_BOOTSTRAP"]), forward=True)
turn_number = 0
for line in sys.stdin:
    frame = json.loads(line)
    method = frame.get("method")
    if method == "initialize":
        emit(dict(id=frame["id"], result=dict(userAgent="fixture155", codexHome=os.environ["HOME"], platformFamily="unix", platformOs="macos")))
    elif method == "thread/start":
        emit(dict(id=frame["id"], result=dict(thread=dict(id=thread_id), model="gpt-5.6-terra", reasoningEffort="high",
                   approvalPolicy="on-request", approvalsReviewer="user", sandboxPolicy=sandbox)))
    elif method == "account/rateLimits/read":
        emit(dict(id=frame["id"], result=dict(rateLimits={})))
    elif method == "turn/start":
        params = frame["params"]
        assert params["threadId"] == thread_id
        assert params["approvalPolicy"] == "on-request"
        assert params["approvalsReviewer"] == "user"
        assert params["sandboxPolicy"] == sandbox
        turn_number += 1
        turn_id = "turn156-" + instance + "-" + str(turn_number)
        prompt = json.dumps(params["input"])
        record("turn156", params=params, turn_id=turn_id, prompt=prompt)
        emit(dict(id=frame["id"], result=dict(turn=dict(id=turn_id), approvalPolicy="on-request", approvalsReviewer="user", sandboxPolicy=sandbox)))
        if "GUIDANCE156_IDLE" in prompt:
            deadline = time.monotonic() + 20
            while not os.path.exists(os.path.join(os.path.dirname(evidence), "release-idle156")):
                if time.monotonic() >= deadline:
                    raise TimeoutError("fixture156 non libérée")
                time.sleep(0.01)
            emit(dict(method="item/agentMessage/delta", params=dict(threadId=thread_id, turnId=turn_id, itemId="answer155", delta="fixture155-resumed")))
            emit(dict(method="turn/completed", params=dict(threadId=thread_id, turn=dict(id=turn_id, status="completed", items=[]))))
            record("idle156", turn_id=turn_id)
    elif method == "turn/steer":
        params = frame["params"]
        assert params["threadId"] == thread_id
        assert params["expectedTurnId"] == turn_id
        record("steer156", params=params, turn_id=turn_id)
        emit(dict(id=frame["id"], result={}))
    elif method == "turn/interrupt":
        record("interrupt156", turn_id=turn_id)
        emit(dict(id=frame["id"], result={}))
        emit(dict(method="turn/completed", params=dict(threadId=thread_id, turn=dict(id=turn_id, status="interrupted", items=[]))))
"#.replace("FIXTURE_SANDBOX155", &json!({"type":"dangerFullAccess"}).to_string())
}

fn provider_claude156() -> String {
    r#"#!/usr/bin/python3
"""Fournisseur fermé de recette : aucun réseau, aucun modèle réel."""
import json
import os
import sys
import time

args = sys.argv[2:]
model = args[args.index("--model") + 1]
effort = args[args.index("--effort") + 1]
assert model == "glm5.3" and effort == "high"
assert "--restricted" in args
assert args[args.index("--permission-mode") + 1] == "plan"
assert not any(key.startswith("T3CODE_") or key.startswith("BRIDGET_T3_") for key in os.environ)
evidence = sys.argv[1]


def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps({"event": event, "pid": os.getpid(), "instance_id": os.environ["BRIDGET_AGENT_INSTANCE_ID"], **values}) + "\n")


def emit(frame):
    print(json.dumps(frame), flush=True)


record("started", provider="closed_fixture", model=model, effort=effort, t3_present=False, agent_id=os.environ["BRIDGET_AGENT_ID"], native_bootstrap=json.loads(os.environ["BRIDGET_NATIVE_MISSION_BOOTSTRAP"]))
initialized = False
for line in sys.stdin:
    frame = json.loads(line)
    if frame.get("type") == "control_request":
        record("interrupt156", frame=frame)
        emit({"type":"control_response","response":{"subtype":"success","request_id":frame["request_id"],"response":{"still_queued":[]}}})
        emit({"type":"result","is_error":True,"terminal_reason":"aborted_tools"})
        continue
    if frame.get("type") != "user":
        continue
    content = frame.get("message", {}).get("content", "")
    if not isinstance(content, str):
        content = json.dumps(content)
    blocked = "WAIT_CANCEL_148" in content and "GUIDANCE156_FIFO_OLDER" not in content and "GUIDANCE156_INTERRUPT" not in content
    record("prompt", content=content, blocked=blocked, resume_card="Carte de reprise Bridget" in content, mission="Inspecte les faits locaux de la fixture148." in content)
    if not initialized:
        emit({"type": "system", "subtype": "init", "model": model, "session_id": "native148-fixture"})
        initialized = True
    if blocked:
        continue
    if "FAIL_151" in content:
        emit({"type": "result", "is_error": True, "terminal_reason": "api_error"})
        continue
    if "WAIT_FILE_151" in content:
        record("waiting_file_151")
        deadline = time.monotonic() + 20
        while not os.path.exists(os.path.join(os.path.dirname(evidence), "release-151")):
            if time.monotonic() >= deadline:
                raise TimeoutError("fixture151 non libérée")
            time.sleep(0.01)
    text = "fixture-glm5.3-answer-148"
    emit({"type": "stream_event", "event": {"type": "content_block_delta", "delta": {"type": "text_delta", "text": text}}})
    emit({"type": "result", "is_error": False, "terminal_reason": "completed", "result": text})
"#.into()
}

// Trois scénarios réemploient cette seule préparation, avec vraie identité/lien.
fn setup_child156(
    codex: bool,
    idle: bool,
) -> (Fixture, support::Client, support::Client, Value, String) {
    let mut f = Fixture::new(codex);
    let mut parent = if codex {
        let mut parent = support::Client::connect(&support::socket(&f.root));
        parent.send(WrapperToDaemon::Register {
            agent_type: "fake-codex".into(),
            identity_version: 2,
            agent_id: PARENT.into(),
            host: Some("t1209".into()),
            transport: Some("codex_app_server".into()),
            channel: None.into(),
            mode: Some(PresenceMode::Acp),
            location: None,
            os: Some("test".into()),
            instance_id: Some(INSTANCE.into()),
            domain: None,
            journal_available: None,
            turn_in_progress: false,
            turn_attest: false,
            queue_fallback: false,
        });
        let DaemonToWrapper::Registered {
            agent_id,
            credential: Some(credential),
        } = parent.receive()
        else {
            panic!("identité primaire du parent absente");
        };
        assert_eq!(agent_id, PARENT);
        support::save_fixture_credential(&support::socket(&f.root), PARENT, INSTANCE, credential);
        let observation = uuid::Uuid::new_v4().to_string();
        parent.send(WrapperToDaemon::NativePermissionFact {
            fact:Some(ProviderPermissions {
                version:1, source:"native_wrapper".into(), run_id:"parent-turn155".into(),
                provider_session_id:"parent-session155".into(), provider_instance_id:INSTANCE.into(), revision:1,
                driver:"codex_app_server".into(), cwd:f.root.to_string_lossy().into_owned(),
                runtime_mode:"approval-required".into(), interaction_mode:"default".into(),
                provider_policy:json!({"kind":"codex","approval_policy":"on-request","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}),
            }), request_id:None, observation_id:Some(observation.clone()), unavailable_code:None,
        });
        assert!(
            matches!(f.receive(&mut parent), DaemonToWrapper::NativePermissionAcknowledged { observation_id, accepted:true } if observation_id == observation)
        );
        parent
    } else {
        let parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
        f.human_grant();
        parent
    };
    let task = if idle {
        "GUIDANCE156_IDLE"
    } else if codex {
        "WAIT_CANCEL_148 GUIDANCE156_ACTIVE"
    } else {
        "WAIT_FILE_151 GUIDANCE156_BOOTSTRAP"
    };
    let mut request = delegate(&f.root, "guidance156-child", task);
    if let NativeDelegationRequest::Delegate {
        agent_type,
        model,
        posture,
        child_requests,
        keep_alive_secs,
        ..
    } = &mut request
    {
        *keep_alive_secs = Some(60);
        if codex {
            *agent_type = "fake-codex".into();
            *model = "gpt-5.6-terra".into();
            *posture = Some(SpawnPosture::Development);
            *child_requests = Some("forward".into());
        }
    }
    let created = f.request(&mut parent, request);
    let id = created["task_id"]
        .as_str()
        .unwrap_or_else(|| panic!("délégation refusée : {created}"));
    let saved = wait_native_ready152(&f, id, "");
    assert_eq!(saved["owner"], PARENT, "propriétaire réel : {saved}");
    f.remember_native_group(saved["child"].as_str().unwrap());
    let instance = assert_native_identity152(&f, id, created["message_id"].as_str().unwrap());
    if idle {
        support::private_write(&f.root.join("release-idle156"), []).unwrap();
    }
    if !codex {
        support::private_write(&f.root.join("release-151"), []).unwrap();
        let ready = Instant::now() + Duration::from_secs(15);
        loop {
            let task = native_task152(&f, id);
            if task["state"] == "result_available" {
                break;
            }
            assert_ne!(
                task["state"], "failed",
                "mission Claude courte échouée : {task}"
            );
            assert!(
                Instant::now() < ready,
                "mission Claude courte non terminée : {task}"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    let mut sender = support::Client::connect(&support::socket(&f.root));
    sender.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    support::attest_agent(&support::socket(&f.root), &mut sender, PARENT, INSTANCE);
    sender.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: support::SCOPE.into(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    (f, parent, sender, saved, instance)
}

fn send_guidance156(
    sender: &mut support::Client,
    child: &str,
    id: &str,
    intent: Option<MessageIntent>,
    body: &str,
) {
    let mut message = BridgetMessage::new(PARENT, child, body);
    message.id = id.into();
    message.intent = intent;
    if intent == Some(MessageIntent::SteerCurrent) {
        message.deadline_at = Some(0);
    }
    sender.send(WrapperToDaemon::SendIdempotent {
        message,
        message_id: id.into(),
        issued_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    });
    let response = sender.receive();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. } | IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "l’admission daemon n’est pas un acquittement fournisseur : {response:?}"
    );
}

// Lecture du journal réel produit par le wrapper, pas une projection simulée.
fn journal156(f: &Fixture, child: &str) -> Vec<Value> {
    fs::read_dir(f.root.join("state/sessions").join(child))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .flat_map(|path| bridget_transport::journal::valid_events(&path))
        .collect()
}

// Le refus asynchrone n’est pas un événement journalisé : lecture de la saga
// réelle, en mode SQL lecture seule, après DeliveryIndeterminate du wrapper.
fn phase156(f: &Fixture, id: &str) -> Option<String> {
    use rusqlite::OptionalExtension;
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    db.busy_timeout(Duration::from_secs(5)).unwrap();
    db.query_row("SELECT phase FROM send_deliveries WHERE issuer_scope=?1 AND operation_kind='send' AND idempotency_key=?2", [support::SCOPE, id], |row| row.get(0)).optional().unwrap()
}

#[test]
fn guidance156_e2e_codex_actif_parent_steer_meme_tour() {
    let (f, _parent, mut sender, saved, instance) = setup_child156(true, false);
    let child = saved["child"].as_str().unwrap();
    let initial = wait_evidence152(&f, "turn156", &instance);
    thread::sleep(Duration::from_millis(150));
    send_guidance156(
        &mut sender,
        child,
        "g156-steer-actif",
        Some(MessageIntent::SteerCurrent),
        "GUIDANCE156_STEER",
    );
    let steer = wait_evidence152(&f, "steer156", &instance);
    assert_eq!(steer["params"]["expectedTurnId"], initial["turn_id"]);
    assert_eq!(steer["params"]["clientUserMessageId"], "g156-steer-actif");
    thread::sleep(Duration::from_millis(500));
    let observations = evidence152(&f);
    assert!(
        !observations
            .iter()
            .any(|entry| entry["event"] == "interrupt156"),
        "deadline agent : {observations:?}"
    );
    assert_eq!(
        phase156(&f, "g156-steer-actif").as_deref(),
        Some("dispatching"),
        "ACK sans consommation ne solde pas la saga"
    );
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        let events = journal156(&f, child);
        assert!(
            !events
                .iter()
                .any(|entry| entry["event"] == "prompt_dispatched"
                    && entry["message_id"] == "g156-steer-actif"),
            "ACK sans consommation : {events:?}"
        );
        if events.iter().any(|entry| {
            entry["event"] == "turn_steer"
                && entry["message_id"] == "g156-steer-actif"
                && entry["payload"]["turn_id"] == initial["turn_id"]
        }) {
            break;
        }
        assert!(
            Instant::now() < ready,
            "turn_steer corrélé absent : {events:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn guidance156_e2e_claude_refuse_steer_interrupt_et_fifo() {
    let (f, _parent, mut sender, saved, instance) = setup_child156(false, false);
    let child = saved["child"].as_str().unwrap();
    // La mission native est réellement terminée. Le tour suivant est un DM
    // ordinaire : le parent peut l’interrompre sans contourner mission_active.
    send_guidance156(
        &mut sender,
        child,
        "g156-claude-ordinary-active",
        None,
        "WAIT_CANCEL_148 GUIDANCE156_ACTIVE",
    );
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        let observations = evidence152(&f);
        if observations.iter().any(|entry| {
            entry["event"] == "prompt"
                && entry["instance_id"] == instance
                && entry["blocked"] == true
                && entry["content"]
                    .as_str()
                    .is_some_and(|text| text.contains("g156-claude-ordinary-active"))
        }) {
            break;
        }
        assert!(
            Instant::now() < ready,
            "tour Claude ordinaire actif absent : {observations:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    send_guidance156(
        &mut sender,
        child,
        "g156-claude-steer",
        Some(MessageIntent::SteerCurrent),
        "GUIDANCE156_STEER",
    );
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        let phase = phase156(&f, "g156-claude-steer");
        if phase.as_deref() == Some("indeterminate") {
            break;
        }
        assert!(
            Instant::now() < ready,
            "refus transport Claude non persisté : {phase:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let observations = evidence152(&f);
    assert!(
        !observations
            .iter()
            .any(|entry| entry["event"] == "interrupt156"),
        "steer a interrompu : {observations:?}"
    );
    assert!(
        !journal156(&f, child)
            .iter()
            .any(|entry| entry["event"] == "turn_end"
                && entry["message_id"] == "g156-claude-ordinary-active")
    );
    send_guidance156(
        &mut sender,
        child,
        "g156-fifo-ancien",
        None,
        "GUIDANCE156_FIFO_OLDER",
    );
    send_guidance156(
        &mut sender,
        child,
        "g156-claude-interrupt",
        Some(MessageIntent::InterruptAndStart),
        "GUIDANCE156_INTERRUPT",
    );
    wait_evidence152(&f, "interrupt156", &instance);
    let ready = Instant::now() + Duration::from_secs(10);
    let prompts = loop {
        let prompts: Vec<Value> = evidence152(&f)
            .into_iter()
            .filter(|entry| entry["event"] == "prompt")
            .collect();
        if prompts.iter().any(|entry| {
            entry["content"]
                .as_str()
                .is_some_and(|text| text.contains("g156-claude-interrupt"))
        }) {
            break prompts;
        }
        assert!(
            Instant::now() < ready,
            "nouveau tour interrupt absent : {prompts:?}"
        );
        thread::sleep(Duration::from_millis(10));
    };
    let older = prompts
        .iter()
        .position(|entry| {
            entry["content"]
                .as_str()
                .is_some_and(|text| text.contains("g156-fifo-ancien"))
        })
        .unwrap();
    let interrupt = prompts
        .iter()
        .position(|entry| {
            entry["content"]
                .as_str()
                .is_some_and(|text| text.contains("g156-claude-interrupt"))
        })
        .unwrap();
    assert!(
        older < interrupt,
        "FIFO réelle du fournisseur : {prompts:?}"
    );
}

#[test]
fn guidance156_e2e_codex_inactif_steer_refuse_sans_effet() {
    let (f, _parent, mut sender, saved, instance) = setup_child156(true, true);
    let child = saved["child"].as_str().unwrap();
    wait_evidence152(&f, "idle156", &instance);
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        let events = journal156(&f, child);
        if events
            .iter()
            .any(|entry| entry["event"] == "turn_end" && entry["message_id"] == saved["mission"])
        {
            break;
        }
        assert!(
            Instant::now() < ready,
            "fin du tour initial absente : {events:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    send_guidance156(
        &mut sender,
        child,
        "g156-codex-idle-steer",
        Some(MessageIntent::SteerCurrent),
        "GUIDANCE156_IDLE_STEER",
    );
    let ready = Instant::now() + Duration::from_secs(5);
    loop {
        let phase = phase156(&f, "g156-codex-idle-steer");
        if phase.as_deref() == Some("indeterminate") {
            break;
        }
        assert!(
            Instant::now() < ready,
            "refus transport idle non persisté : {phase:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let observations = evidence152(&f);
    assert_eq!(
        observations
            .iter()
            .filter(|entry| entry["event"] == "turn156")
            .count(),
        1,
        "aucun tour nouveau : {observations:?}"
    );
    assert!(
        !observations
            .iter()
            .any(|entry| entry["event"] == "steer156" || entry["event"] == "interrupt156"),
        "effet fournisseur idle : {observations:?}"
    );
}
