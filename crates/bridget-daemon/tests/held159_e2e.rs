//! T001159A/SC004 : fournisseur simulé, mesure avant code159 ; T010 reprendra le même oracle.
//! Copie minimale des fixtures guidance156_e2e ; pas d’API publique/framework nouveau.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;
use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{ManagedMarker, ManagedMarkerStore, process_birth};
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    NativeDelegationRequest, PresenceMode, ProviderPermissions, SpawnPosture,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const PARENT: &str = "15900000-0000-4000-8000-000000000001";
const INSTANCE: &str = "held159-parent-instance";
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
        let birth = process_birth(self.0.id()).expect("naissance du PID propre");
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
        let expected = std::env::current_exe().unwrap();
        eprintln!(
            "arrêt enfant isolé pid={} expected={:?} ps_raw={:?} ps_stderr={:?}",
            self.0.id(),
            expected,
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
        let argv = command.split_whitespace().skip(1).collect::<Vec<_>>();
        let performance_argv = vec![
            expected.to_str().unwrap(),
            "--exact",
            "support::performance_daemon_worker",
            "--ignored",
            "--nocapture",
        ];
        let crash_binary = std::env::var_os("BRIDGET_T010_DAEMON_TEST_BINARY").map(PathBuf::from);
        let crash_argv = crash_binary.as_ref().map(|binary| {
            vec![
                binary.to_str().unwrap(),
                "--exact",
                "daemon::presence_tests::recovery_daemon_child",
                "--ignored",
                "--nocapture",
            ]
        });
        assert!(
            argv == performance_argv || crash_argv.as_ref() == Some(&argv),
            "PID propre d'un des deux workers privés existants uniquement"
        );
        assert_eq!(process_birth(self.0.id()).ok(), Some(birth));
        assert!(!command.to_lowercase().contains("firefox"));
        assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
        thread::sleep(Duration::from_secs(3));
        self.wait(Duration::from_secs(10));
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
}

// T010 : worker unitaire existant. cfg(test) conserve SIGTERM par défaut.
// ROOT fournit le binaire libtest absolu après compilation ; aucune copie de service.
fn spawn_crash_daemon159(root: &Path, sync: Option<&Path>) -> OwnedChild {
    let binary = PathBuf::from(
        std::env::var_os("BRIDGET_T010_DAEMON_TEST_BINARY")
            .expect("ROOT doit fournir le binaire libtest unitaire absolu"),
    );
    assert!(binary.is_absolute());
    support::private_dir(&root.join(".state")).unwrap();
    let isolated = support::isolated_command(root);
    let mut command = Command::new(binary);
    command
        .env_clear()
        .envs(isolated.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))))
        .args([
            "--exact",
            "daemon::presence_tests::recovery_daemon_child",
            "--ignored",
            "--nocapture",
        ])
        .env("BRIDGET_T908_DAEMON_CHILD", "1")
        .env("BRIDGET_T908_ROOT", root)
        .env("BRIDGET_HOME", root.join(".state"))
        .env("BRIDGET_SOCKET", root.join(".state/bridget.sock"))
        .env(
            "BRIDGET_T908_MANAGED_EXECUTABLE",
            env!("CARGO_BIN_EXE_bridget"),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    if let Some(sync) = sync {
        command
            .env("BRIDGET_TEST_SYNC_DIR", sync)
            .env("BRIDGET_T010_DEFAULT_TERM", "1");
    }
    OwnedChild(command.spawn().unwrap())
}

struct Fixture {
    root: PathBuf,
    state: PathBuf,
    crash_worker: bool,
    native_agent: Option<String>,
    initial_provider: Option<(u32, u64)>,
    carrier_definition: Option<bridget_transport::protocol::ResolvedAgentDefinition>,
    manual_carrier: Option<bridget_daemon::managed_process::RunningManagedChild>,
    daemon: Option<OwnedChild>,
    native: Option<ManagedMarker>,
}
impl Fixture {
    fn socket(&self) -> PathBuf {
        self.state.join("bridget.sock")
    }
    fn with_options(held_queue: Option<bool>, crash_worker: bool, sync: Option<&Path>) -> Self {
        let root = std::env::temp_dir().join(format!(
            "h159-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        for relative in ["", "state", "provider", "tmp", "bin", ".git"] {
            support::private_dir(&root.join(relative)).unwrap();
        }
        let script = if crash_worker {
            provider_codex156()
                .replace("sandbox = json.loads('{\"type\":\"dangerFullAccess\"}')",
                    "sandbox = json.loads('{\"type\":\"dangerFullAccess\"}')\nnative_mode = os.environ.get(\"BRIDGET_NATIVE_CHILD_REQUESTS\") == \"forward\"\nif not native_mode: sandbox = {\"type\": \"readOnly\"}\napproval = \"on-request\" if native_mode else \"never\"")
                .replace("approvalPolicy=\"on-request\"", "approvalPolicy=approval")
                .replace("        assert params[\"approvalPolicy\"] == \"on-request\"\n        assert params[\"approvalsReviewer\"] == \"user\"\n        assert params[\"sandboxPolicy\"] == sandbox",
                    "        if native_mode:\n            assert params[\"approvalPolicy\"] == \"on-request\"\n            assert params[\"approvalsReviewer\"] == \"user\"\n            assert params[\"sandboxPolicy\"] == sandbox\n        else:\n            if \"approvalPolicy\" in params: assert params[\"approvalPolicy\"] == \"never\"\n            if \"sandboxPolicy\" in params: assert params[\"sandboxPolicy\"] == sandbox")
                .replace("        params = frame[\"params\"]\n        assert params[\"threadId\"] == thread_id",
                    "        params = frame[\"params\"]\n        record(\"raw_turn_start159\", params=params)\n        assert params[\"threadId\"] == thread_id")
                .replace("assert os.environ.get(\"BRIDGET_NATIVE_CHILD_REQUESTS\") == \"forward\"",
                    "assert os.environ.get(\"BRIDGET_NATIVE_CHILD_REQUESTS\") in (None, \"forward\")")
                .replace("native_bootstrap=json.loads(os.environ[\"BRIDGET_NATIVE_MISSION_BOOTSTRAP\"]), forward=True",
                    "native_bootstrap=json.loads(os.environ.get(\"BRIDGET_NATIVE_MISSION_BOOTSTRAP\", \"null\")), forward=os.environ.get(\"BRIDGET_NATIVE_CHILD_REQUESTS\") == \"forward\"")
        } else {
            provider_codex156()
        };
        support::private_write(&root.join("provider.py"), &script).unwrap();
        let cli = compile_provider152(&root, &script, "codex");
        let mut registry = json!({"agents":{"fake-codex":{
            "command":cli,"args":["app-server"],"protocol":"codex_app_server","permissions":"deny",
            "queue_capacity":4,"notify_timeout_secs":30,
            "forbidden_env":["ANTHROPIC_API_KEY","OPENAI_API_KEY","T3CODE_HOME","BRIDGET_T3_MCP_ENDPOINT","BRIDGET_T3_MCP_AUTHORIZATION"],"pass_env":[],
            "capabilities":{"execution_paths":["codex_app_server"],"models":{"gpt-5.6-terra":{"efforts":["high"]}}}
        }}});
        if let Some(value) = held_queue {
            registry["agents"]["fake-codex"]["held_queue"] = json!(value);
        } else {
            assert!(registry["agents"]["fake-codex"].get("held_queue").is_none());
        }
        let state = root.join(if crash_worker { ".state" } else { "state" });
        support::private_dir(&state).unwrap();
        support::private_write(
            &state.join("agents.json"),
            serde_json::to_vec(&registry).unwrap(),
        )
        .unwrap();
        support::private_write(&root.join("evidence.jsonl"), []).unwrap();
        let log = root.join("daemon.log");
        support::private_write(&log, []).unwrap();
        let isolated = support::isolated_command(&root);
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .env_clear()
            .envs(isolated.get_envs().filter_map(|(k, v)| v.map(|v| (k, v))));
        let daemon = if crash_worker {
            let local_sync = root.join("sync");
            support::private_dir(&local_sync).unwrap();
            spawn_crash_daemon159(&root, Some(sync.unwrap_or(&local_sync)))
        } else {
            OwnedChild(
                command
                    .args([
                        "--exact",
                        "support::performance_daemon_worker",
                        "--ignored",
                        "--nocapture",
                    ])
                    .env("BRIDGET_PERFORMANCE_DAEMON", "1")
                    .env(
                        "BRIDGET_T908_MANAGED_EXECUTABLE",
                        env!("CARGO_BIN_EXE_bridget"),
                    )
                    .env("TMPDIR", root.join("tmp"))
                    .env("RUST_LOG", "info")
                    .stdout(Stdio::null())
                    .stderr(fs::OpenOptions::new().append(true).open(log).unwrap())
                    .spawn()
                    .unwrap(),
            )
        };
        let f = Self {
            root,
            state,
            crash_worker,
            native_agent: None,
            initial_provider: None,
            carrier_definition: None,
            manual_carrier: None,
            daemon: Some(daemon),
            native: None,
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while std::os::unix::net::UnixStream::connect(f.socket()).is_err() {
            assert!(Instant::now() < deadline, "daemon isolé absent");
            thread::sleep(Duration::from_millis(10));
        }
        f
    }
    // Le crash du wrapper saute ses destructeurs. Les descendants du seul
    // fournisseur synthétique connu sont identifiés AVANT toute terminaison.
    fn stop_native159(&mut self) {
        // Ces PID/birth ont été attestés AVANT la perte possible du wrapper.
        let mut providers = self.initial_provider.take().into_iter().collect::<Vec<_>>();
        if let Some(agent) = &self.native_agent {
            if let Ok(marker) =
                ManagedMarkerStore::at_directory(self.state.join("managed")).load(agent)
            {
                self.native = Some(marker);
            }
        }
        if let Some(marker) = self.native.take() {
            let provider = evidence152(self)
                .into_iter()
                .rev()
                .find(|e| e["event"] == "started" && e["instance_id"] == marker.instance_id)
                .and_then(|e| e["pid"].as_u64())
                .map(|pid| pid as u32)
                .and_then(|pid| process_birth(pid).ok().map(|birth| (pid, birth)));
            if let Some((pid, birth)) = provider {
                // Une preuve déjà conservée reste valable après orphelinage (ppid1).
                if !providers.contains(&(pid, birth)) {
                    let observed = Command::new("/bin/ps")
                        .args([
                            "-ww",
                            "-p",
                            &pid.to_string(),
                            "-o",
                            "ppid=",
                            "-o",
                            "command=",
                        ])
                        .output()
                        .unwrap();
                    if process_birth(pid).ok() == Some(birth) {
                        let command = String::from_utf8_lossy(&observed.stdout);
                        assert_eq!(
                            command
                                .split_whitespace()
                                .next()
                                .and_then(|v| v.parse::<u32>().ok()),
                            Some(marker.pgid),
                            "filiation attestée avant le TERM de cette incarnation"
                        );
                        assert!(
                            command.contains(self.root.join("provider.py").to_str().unwrap())
                                && !command.to_lowercase().contains("firefox")
                        );
                        providers.push((pid, birth));
                    }
                }
            }
            if process_birth(marker.pgid).ok() == Some(marker.birth) {
                let observed = Command::new("/bin/ps")
                    .args(["-ww", "-p", &marker.pgid.to_string(), "-o", "command="])
                    .output()
                    .unwrap();
                let command = String::from_utf8_lossy(&observed.stdout);
                assert!(
                    command.contains(env!("CARGO_BIN_EXE_bridget"))
                        && !command.to_lowercase().contains("firefox")
                );
                if process_birth(marker.pgid).ok() == Some(marker.birth) {
                    assert_eq!(unsafe { libc::kill(marker.pgid as i32, libc::SIGTERM) }, 0);
                    thread::sleep(Duration::from_secs(3));
                    if let Some(running) = self.manual_carrier.as_mut() {
                        assert_eq!(running.child_mut().id(), marker.pgid);
                        assert!(
                            running.try_wait().unwrap().is_some(),
                            "enfant managed propre non récolté après TERM"
                        );
                    }
                    assert_ne!(
                        process_birth(marker.pgid).ok(),
                        Some(marker.birth),
                        "wrapper privé TERM non arrêté"
                    );
                }
            }
        } // Sans marker, les fournisseurs dont la preuve est conservée restent à nettoyer.
        if let Some(running) = self.manual_carrier.as_mut() {
            assert!(
                running.try_wait().unwrap().is_some(),
                "enfant managed propre encore vivant sans marqueur"
            );
        }
        self.manual_carrier.take();
        for (pid, birth) in providers {
            if process_birth(pid).ok() == Some(birth) {
                let observed = Command::new("/bin/ps")
                    .args(["-ww", "-p", &pid.to_string(), "-o", "command="])
                    .output()
                    .unwrap();
                let command = String::from_utf8_lossy(&observed.stdout);
                assert!(
                    command.contains(self.root.join("provider.py").to_str().unwrap())
                        && !command.to_lowercase().contains("firefox")
                );
                if process_birth(pid).ok() == Some(birth) {
                    assert_eq!(unsafe { libc::kill(pid as i32, libc::SIGTERM) }, 0);
                    thread::sleep(Duration::from_secs(3));
                    assert_ne!(
                        process_birth(pid).ok(),
                        Some(birth),
                        "fournisseur privé TERM non arrêté"
                    );
                }
            }
        }
    }
    fn receive(&mut self, parent: &mut support::Client) -> DaemonToWrapper {
        match parent.receive() {
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                ..
            } => {
                parent.send(WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                });
                self.receive(parent)
            }
            DaemonToWrapper::Deliver { .. } | DaemonToWrapper::DeliverExecution { .. } => {
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
        panic!("résultat natif absent")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if self.crash_worker {
            // Couper la reprise avant d'identifier l'incarnation finale empêche un respawn pendant cleanup.
            drop(self.daemon.take());
            self.stop_native159();
            if !thread::panicking() {
                fs::remove_dir_all(&self.root).unwrap();
            }
            return;
        }
        if let Some(marker) = &self.native {
            if process_birth(marker.pgid).ok() == Some(marker.birth) {
                let observed = Command::new("/bin/ps")
                    .args(["-ww", "-p", &marker.pgid.to_string(), "-o", "command="])
                    .output()
                    .unwrap();
                let command = String::from_utf8_lossy(&observed.stdout);
                if process_birth(marker.pgid).ok() == Some(marker.birth) {
                    assert!(
                        command.contains(env!("CARGO_BIN_EXE_bridget"))
                            && !command.to_lowercase().contains("firefox")
                    );
                    // PID leader attesté seulement, jamais signal à un groupe.
                    assert_eq!(unsafe { libc::kill(marker.pgid as i32, libc::SIGTERM) }, 0);
                }
                thread::sleep(Duration::from_secs(3));
                let deadline = Instant::now() + Duration::from_secs(10);
                while process_birth(marker.pgid).ok() == Some(marker.birth)
                    && Instant::now() < deadline
                {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
        drop(self.daemon.take());
        if !thread::panicking() {
            fs::remove_dir_all(&self.root).unwrap();
        }
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
        f.state.join("bridget.db"),
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
fn evidence152(f: &Fixture) -> Vec<Value> {
    let bytes = fs::read(f.root.join("evidence.jsonl")).unwrap();
    // Une écriture concurrente peut laisser uniquement la dernière ligne incomplète.
    bytes
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| line.last() == Some(&b'\n'))
        .map(|line| serde_json::from_slice(line).unwrap())
        .collect()
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

record("started", provider="closed_codex_fixture", pid=os.getpid(), state_root=os.environ["BRIDGET_HOME"], agent_id=os.environ["BRIDGET_AGENT_ID"],
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
        started_ns = time.monotonic_ns()
        record("turn156", params=params, turn_id=turn_id, prompt=prompt, turn_number=turn_number, monotonic_ns=started_ns)
        emit(dict(id=frame["id"], result=dict(turn=dict(id=turn_id), approvalPolicy="on-request", approvalsReviewer="user", sandboxPolicy=sandbox)))
        gate = os.path.join(os.path.dirname(evidence), "release-turn159-" + str(turn_number))
        deadline = time.monotonic() + 30
        while not os.path.exists(gate):
            if time.monotonic() >= deadline:
                raise TimeoutError("tour159 non libéré")
            time.sleep(0.001)
        emit(dict(method="item/agentMessage/delta", params=dict(threadId=thread_id, turnId=turn_id, itemId="answer159", delta="fixture159-completed")))
        completed_ns = time.monotonic_ns()
        emit(dict(method="turn/completed", params=dict(threadId=thread_id, turn=dict(id=turn_id, status="completed", items=[]))))
        record("completed159", turn_number=turn_number, monotonic_ns=completed_ns)
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

// Précondition159 : l'événement provider précède parfois TurnState reçu au daemon.
// Cette attente réelle se situe avant admission/release et hors intervalle SC004.
fn wait_busy159(sender: &mut support::Client, target: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        sender.send(WrapperToDaemon::ListAgents);
        let agents = loop {
            match sender.receive() {
                DaemonToWrapper::AgentList { agents } => break agents,
                DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    message,
                    ..
                } => {
                    assert_eq!(recipient_instance_id, INSTANCE);
                    assert_eq!(message.to, PARENT);
                    assert_eq!(message.from, target);
                    assert!(message.id.starts_with("native-result-"));
                    assert_eq!(message.body, "fixture159-completed");
                    sender.send(WrapperToDaemon::DeliverAcked {
                        delivery_id,
                        delivery_generation,
                    });
                }
                other => panic!("ListAgents ou résultat natif réellement reçu attendu : {other:?}"),
            }
        };
        let state = agents
            .iter()
            .find(|agent| agent.agent_id == target)
            .map(|agent| agent.state.as_str());
        if state == Some("busy") {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "daemon n'atteste pas busy pour {target}: {state:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

// Bootstrap commun T001/T010 : vrai owner, preuve Registered, RegisterAuxiliary et ClientHello.
fn bootstrap159(
    held_queue: Option<bool>,
    crash_worker: bool,
) -> (
    Fixture,
    support::Client,
    support::Client,
    String,
    String,
    PathBuf,
    u64,
) {
    let mut f = Fixture::with_options(held_queue, crash_worker, None);
    let mut parent = support::Client::connect(&f.socket());
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
    });
    let DaemonToWrapper::Registered {
        agent_id,
        credential: Some(credential),
    } = parent.receive()
    else {
        panic!("credential primaire réelle attendue");
    };
    assert_eq!(agent_id, PARENT);
    support::save_fixture_credential(&f.socket(), PARENT, INSTANCE, credential);
    let observation = uuid::Uuid::new_v4().to_string();
    parent.send(WrapperToDaemon::NativePermissionFact { fact:Some(ProviderPermissions { version:1,source:"native_wrapper".into(),run_id:"parent-turn155".into(),
        provider_session_id:"parent-session155".into(),provider_instance_id:INSTANCE.into(),revision:1,driver:"codex_app_server".into(),
        cwd:f.root.to_string_lossy().into_owned(),runtime_mode:"approval-required".into(),interaction_mode:"default".into(),
        provider_policy:json!({"kind":"codex","approval_policy":"on-request","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}) }),
        request_id:None,observation_id:Some(observation.clone()),unavailable_code:None });
    assert!(
        matches!(f.receive(&mut parent),DaemonToWrapper::NativePermissionAcknowledged { observation_id,accepted:true } if observation_id==observation)
    );
    let created = f.request(
        &mut parent,
        NativeDelegationRequest::Delegate {
            fallback: Vec::new(),
            request_id: "latence159-bootstrap".into(),
            agent_type: "fake-codex".into(),
            model: "gpt-5.6-terra".into(),
            effort: Some("high".into()),
            task: "HELD159_INITIAL".into(),
            cwd: f.root.to_string_lossy().into_owned(),
            posture: Some(SpawnPosture::Development),
            reply_timeout: None,
            child_requests: Some("forward".into()),
            workspace: None,
            keep_alive_secs: Some(300),
            brief: None,
        },
    );
    let task_id = created["task_id"].as_str().unwrap();
    let saved = wait_native_ready152(&f, task_id, "");
    let child = saved["child"].as_str().unwrap();
    let instance = saved["child_instance"].as_str().unwrap();
    assert_eq!(saved["owner"], PARENT);
    f.native_agent = Some(child.to_owned());
    f.native = Some(
        ManagedMarkerStore::at_directory(f.state.join("managed"))
            .load(child)
            .unwrap(),
    );
    let mut sender = support::Client::connect(&f.socket());
    sender.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    support::attest_agent(&f.socket(), &mut sender, PARENT, INSTANCE);
    sender.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: support::SCOPE.into(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    let started = evidence152(&f)
        .into_iter()
        .find(|e| e["event"] == "started" && e["instance_id"] == instance)
        .unwrap();
    let provider_pid = started["pid"].as_u64().unwrap();
    if crash_worker {
        let pid = provider_pid as u32;
        let birth = process_birth(pid).unwrap();
        let observed = Command::new("/bin/ps")
            .args([
                "-ww",
                "-p",
                &pid.to_string(),
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
                .and_then(|v| v.parse::<u32>().ok()),
            Some(f.native.as_ref().unwrap().pgid)
        );
        assert!(
            command.contains(f.root.join("provider.py").to_str().unwrap())
                && !command.to_lowercase().contains("firefox")
        );
        assert_eq!(process_birth(pid).ok(), Some(birth));
        f.initial_provider = Some((pid, birth));
    }
    let state_root = PathBuf::from(started["state_root"].as_str().unwrap());
    assert!(state_root.starts_with(&f.root));
    if held_queue == Some(true) {
        wait_busy159(&mut parent, child);
    }
    let child = child.to_owned();
    let instance = instance.to_owned();
    (f, parent, sender, child, instance, state_root, provider_pid)
}

#[test]
fn held159_latency_baseline_codex_simule() {
    measure_latency159(false);
}
#[test]
fn held159_latency_retenue_codex_simule() {
    measure_latency159(true);
}
fn measure_latency159(held_queue: bool) {
    let report = PathBuf::from(
        std::env::var_os(if held_queue {
            "BRIDGET_LATENCY159_AFTER_REPORT"
        } else {
            "BRIDGET_LATENCY159_REPORT"
        })
        .expect("ROOT doit fixer un rapport distinct avant/après pour conserver JSON"),
    );
    assert!(report.is_absolute());
    let (f, mut parent, mut sender, child, instance, state_root, provider_pid) =
        bootstrap159(if held_queue { Some(true) } else { None }, false);
    let db = rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let mut samples = Vec::new();
    for number in 1..=20_u64 {
        let until = Instant::now() + Duration::from_secs(15);
        while !evidence152(&f)
            .iter()
            .any(|e| e["event"] == "turn156" && e["turn_number"] == number)
        {
            assert!(Instant::now() < until);
            thread::sleep(Duration::from_millis(1));
        }
        if held_queue {
            wait_busy159(&mut parent, &child);
        }
        let id = format!("held159-sample-{number:02}");
        let mut message =
            BridgetMessage::new(PARENT, &child, format!("HELD159_SAMPLE_{number:02}"));
        message.id = id.clone();
        sender.send(WrapperToDaemon::SendIdempotent {
            message_id: id.clone(),
            message,
            issued_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });
        let admission = sender.receive();
        let raw_admission = serde_json::to_value(&admission).unwrap();
        eprintln!("HELD159_ADMISSION_JSON {raw_admission}");
        support::private_write(
            &report.with_extension("last-admission.json"),
            serde_json::to_vec_pretty(&raw_admission).unwrap(),
        )
        .unwrap();
        assert!(
            matches!(
                &admission,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind, idempotency_key,
                    issue: IdempotencyIssue::Accepted { .. }
                        | IdempotencyIssue::OutcomeUnknown { delivery_id:Some(_), .. },
                    ..
                } if operation_kind == "send" && idempotency_key == &id
            ),
            "admission préparée n'est pas un ACK fournisseur : {raw_admission}"
        );
        // Le message doit être déjà vu par le wrapper avant de libérer le fournisseur.
        let until = Instant::now() + Duration::from_secs(10);
        let delivery = loop {
            use rusqlite::OptionalExtension;
            let value:Option<String>=db.query_row("SELECT delivery_id FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key=?2",rusqlite::params![support::SCOPE,id],|r|r.get(0)).optional().unwrap();
            if let Some(delivery) = value {
                if held_queue {
                    let (phase, rank): (String, i64) = db
                        .query_row(
                            "SELECT phase,queue_rank FROM send_deliveries WHERE delivery_id=?1",
                            [&delivery],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )
                        .unwrap();
                    if phase == "held" && rank > 0 {
                        break delivery;
                    }
                }
                let receipt = state_root
                    .join("state/bridget/receipts")
                    .join(&instance)
                    .join(format!("receipt-{delivery}"));
                if !held_queue && fs::read_to_string(receipt).is_ok_and(|s| s.starts_with("seen\n"))
                {
                    break delivery;
                }
            }
            assert!(
                Instant::now() < until,
                "message non mis en file avant repos {id}"
            );
            thread::sleep(Duration::from_millis(1));
        };
        assert!(
            !evidence152(&f)
                .iter()
                .any(|e| e["event"] == "turn156" && e["turn_number"] == number + 1)
        );
        support::private_write(&f.root.join(format!("release-turn159-{number}")), []).unwrap();
        let until = Instant::now() + Duration::from_secs(15);
        loop {
            let events = evidence152(&f);
            let end = events
                .iter()
                .find(|e| e["event"] == "completed159" && e["turn_number"] == number);
            let next = events
                .iter()
                .find(|e| e["event"] == "turn156" && e["turn_number"] == number + 1);
            if let (Some(end), Some(next)) = (end, next) {
                assert_eq!(next["instance_id"], instance);
                assert!(
                    next["prompt"]
                        .as_str()
                        .unwrap()
                        .contains(&format!("HELD159_SAMPLE_{number:02}"))
                );
                let end_ns = end["monotonic_ns"].as_u64().unwrap();
                let start_ns = next["monotonic_ns"].as_u64().unwrap();
                let delta_ns = start_ns
                    .checked_sub(end_ns)
                    .expect("mêmehorlogeprovider monotone");
                samples.push(json!({"number":number,"message_id":id,"delivery_id":delivery,"admission":raw_admission,"provider_pid":provider_pid,"completed_ns":end_ns,"next_start_ns":start_ns,"delta_ns":delta_ns}));
                break;
            }
            assert!(Instant::now() < until, "tour suivant absent");
            thread::sleep(Duration::from_millis(1));
        }
    }
    assert_eq!(samples.len(), 20);
    let mut durations = samples
        .iter()
        .map(|s| s["delta_ns"].as_u64().unwrap())
        .collect::<Vec<_>>();
    durations.sort_unstable();
    let evidence = fs::read(f.root.join("evidence.jsonl")).unwrap();
    let result = json!({"version":1,"phase":if held_queue {"after159A"} else {"before159A"},"held_queue":if held_queue {json!(true)} else {json!("absent")},"measure":"provider_before_emit_turn_completed_to_next_turn_start_received","daemon_idle_attestation_measured":false,
        "provider_processes":1,"samples_count":20,"p95_nearest_rank":19,"p95_ns":durations[18],"samples":samples,
        "fixture_source":"guidance156_e2e","root":f.root,"provider_evidence":String::from_utf8(evidence).unwrap(),
        "conditions":{"private_daemon_worker":"support::performance_daemon_worker","circuit_breaker_limit":10000,"circuit_breaker_window_secs":180,"managed_executable":"CARGO_BIN_EXE_bridget","queue_capacity":4,"native_keep_alive_secs":300,"poll_ms":1,"sender":"attested_native_parent","intent":"absent","reply":false,"provider":"simulated_codex_no_network"}});
    support::private_write(&report, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    eprintln!("HELD159_LATENCY_JSON {}", result);
    // Dernier tour terminé avant teardown ; aucune rétention nouvelle.
    support::private_write(&f.root.join("release-turn159-21"), []).unwrap();
    drop(db);
    drop(sender);
    drop(parent);
}

#[test]
fn held159_daemon_arrete_apres_retention_conserve_la_remise() {
    use rusqlite::OptionalExtension;
    use std::os::unix::process::ExitStatusExt;
    let (mut f, parent, mut sender, child, _instance, _state_root, _provider_pid) =
        bootstrap159(Some(true), true);
    let until = Instant::now() + Duration::from_secs(15);
    while !evidence152(&f)
        .iter()
        .any(|e| e["event"] == "turn156" && e["turn_number"] == 1)
    {
        assert!(Instant::now() < until, "premier tour réel absent");
        thread::sleep(Duration::from_millis(1));
    }
    let id = "held159-crash-after-held";
    let mut message = BridgetMessage::new(PARENT, &child, "HELD159_CRASH_RETAINED");
    message.id = id.into();
    sender.send(WrapperToDaemon::SendIdempotent {
        message_id: id.into(),
        message,
        issued_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    });
    let admission = sender.receive();
    assert!(
        matches!(
            admission,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. }
                    | IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(_),
                        ..
                    },
                ..
            }
        ),
        "admission socket réelle attendue"
    );
    let db = rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    let frozen: (String, String, i64, Vec<u8>, i64) = loop {
        let row:Option<(String,String,i64,Vec<u8>,i64)>=db.query_row(
            "SELECT phase,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key=?2",
            rusqlite::params![support::SCOPE,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().unwrap();
        if let Some(row) = row {
            if row.0 == "held" && row.4 > 0 {
                break row;
            }
        }
        assert!(
            Instant::now() < until,
            "message ordinaire attesté non retenu"
        );
        thread::sleep(Duration::from_millis(1));
    };
    assert!(
        !evidence152(&f)
            .iter()
            .any(|e| e["event"] == "turn156" && e["turn_number"] == 2)
    );
    let mut crashed = f.daemon.take().unwrap();
    crashed.stop();
    assert_eq!(
        crashed.0.try_wait().unwrap().unwrap().signal(),
        Some(libc::SIGTERM),
        "worker unitaire cfg(test) réellement interrompu, pas shutdown coopératif"
    );
    drop(sender);
    drop(parent);
    drop(crashed);
    f.daemon = Some(spawn_crash_daemon159(&f.root, None));
    let until = Instant::now() + Duration::from_secs(10);
    while std::os::unix::net::UnixStream::connect(f.socket()).is_err() {
        assert!(Instant::now() < until, "reprise daemon absente");
        thread::sleep(Duration::from_millis(10));
    }
    let after:(String,String,i64,Vec<u8>,i64)=db.query_row(
        "SELECT phase,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key=?2",
        rusqlite::params![support::SCOPE,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        after, frozen,
        "reprise conserve held, identité, génération, bytes et rang"
    );
    // Arrêter le daemon avant de relire le marqueur empêche une génération de teardown.
    if let Some(mut daemon) = f.daemon.take() {
        daemon.stop();
    }
    if let Ok(marker) = ManagedMarkerStore::at_directory(f.state.join("managed")).load(&child) {
        f.native = Some(marker);
    }
}

// Point21 T007 couvert ici seulement : FIFO observée après COMMIT, jamais ouverte après TERM.
#[test]
#[cfg_attr(target_os = "linux", ignore = "watch_marker exige kqueue Darwin/BSD")]
fn held159_release_panne_entre_transaction_et_push_relivree_par_la_reprise() {
    held_crash_after_release159("after_held_release_commit");
}
#[test]
#[cfg_attr(target_os = "linux", ignore = "watch_marker exige kqueue Darwin/BSD")]
fn held159_daemon_arrete_apres_push_reprise_dans_l_ordre() {
    // R17 : ce banc bloque Seen avant injection. Il ne promet pas exactly-once après tout push.
    held_crash_after_release159("after_idempotent_socket_push");
}

#[test]
#[cfg_attr(target_os = "linux", ignore = "watch_marker exige kqueue Darwin/BSD")]
fn held159_daemon_arrete_apres_accuse_ne_reinjecte_pas() {
    held_crash_after_release159("after_delivery_acked");
}

// Le compteur Python repart de1 après réaffectation. Seul l'id de l'enveloppe
// distingue une injection ordinary d'une carte de reprise de mission.
fn ordinary_number159(event: &Value) -> Option<usize> {
    if event["event"] != "turn156" {
        return None;
    }
    let items: Vec<Value> = serde_json::from_str(event["prompt"].as_str()?).ok()?;
    let text = items.first()?.get("text")?.as_str()?;
    let header = text
        .lines()
        .next()?
        .strip_prefix("[Message Bridget : ")?
        .strip_suffix(']')?;
    let header: Value = serde_json::from_str(header).ok()?;
    header["id"]
        .as_str()?
        .strip_prefix("held159-crash-order-")?
        .parse()
        .ok()
}

// Les pannes prescrites T010 partagent le vrai provider/bootstrap et les trois messages SC003.
fn held_crash_after_release159(point: &str) {
    use std::os::unix::process::ExitStatusExt;
    let (mut f, parent, mut sender, child, _instance, _state_root, _provider_pid) =
        bootstrap159(Some(true), true);
    let db = rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let until = Instant::now() + Duration::from_secs(15);
    while !evidence152(&f)
        .iter()
        .any(|e| e["event"] == "turn156" && e["turn_number"] == 1)
    {
        assert!(Instant::now() < until, "tour initial réel absent");
        thread::sleep(Duration::from_millis(1));
    }
    for number in 1..=3 {
        let id = format!("held159-crash-order-{number}");
        let mut message =
            BridgetMessage::new(PARENT, &child, format!("HELD159_CRASH_ORDER_{number}"));
        message.id = id.clone();
        sender.send(WrapperToDaemon::SendIdempotent {
            message_id: id,
            message,
            issued_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        });
        assert!(matches!(
            sender.receive(),
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. }
                    | IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(_),
                        ..
                    },
                ..
            }
        ));
    }
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let count:i64=db.query_row("SELECT COUNT(*) FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' AND phase='held'",
            [support::SCOPE],|r|r.get(0)).unwrap();
        if count == 3 {
            break;
        }
        assert!(
            Instant::now() < until,
            "trois admissions réelles held attendues"
        );
        thread::sleep(Duration::from_millis(1));
    }
    let frozen_rows: Vec<(String,String,i64,Vec<u8>,i64)> = db.prepare(
        "SELECT idempotency_key,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' ORDER BY queue_rank")
        .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))
        .unwrap().collect::<Result<_,_>>().unwrap();
    assert_eq!(frozen_rows.len(), 3);
    let sync = f.root.join("sync");
    if point == "after_idempotent_socket_push" {
        // Les deux barrières précèdent la libération ; aucun writer FIFO n'est ouvert.
        support::arm_checkpoint(
            &sync,
            &[point, "after_receipt_seen_before_injection"],
            point,
        );
        support::arm_checkpoint(
            &sync,
            &["after_receipt_seen_before_injection"],
            "after_receipt_seen_before_injection",
        );
    } else {
        support::arm_checkpoint(&sync, &[point], point);
    }
    support::private_write(&f.root.join("release-turn159-1"), []).unwrap();
    support::watch_marker(&sync, &sync.join(format!("{point}.ready")));
    if point == "after_idempotent_socket_push" {
        support::watch_marker(
            &sync,
            &sync.join("after_receipt_seen_before_injection.ready"),
        );
        assert!(
            evidence152(&f)
                .iter()
                .all(|event| ordinary_number159(event).is_none()),
            "push réel mais aucune injection ancienne avant la barrière Seen"
        );
    }
    let frozen:(String,String,i64,Vec<u8>,i64)=db.query_row(
        "SELECT phase,delivery_id,delivery_generation,message_bytes,turn_gate FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key='held159-crash-order-1'",
        [support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(
        frozen.0,
        if point == "after_delivery_acked" {
            "acked"
        } else {
            "dispatching"
        }
    );
    if point != "after_delivery_acked" {
        assert_eq!(frozen.4, 1, "réservation réellement committée avant push");
    } // Après ACK, TurnState true peut légitimement avoir déjà levé la réservation.
    let others:i64=db.query_row("SELECT COUNT(*) FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key IN ('held159-crash-order-2','held159-crash-order-3') AND phase='held'",
        [support::SCOPE],|r|r.get(0)).unwrap();
    assert_eq!(others, 2, "suivants restent held pendant la réservation");
    let mut crashed = f.daemon.take().unwrap();
    crashed.stop();
    assert_eq!(
        crashed.0.try_wait().unwrap().unwrap().signal(),
        Some(libc::SIGTERM)
    );
    // Aucun writer FIFO : la disposition TERM par défaut établit cette vraie frontière de crash.
    drop(sender);
    drop(parent);
    drop(crashed);
    if point == "after_idempotent_socket_push" {
        // Stop réel avant restart ; ne jamais déverrouiller Seen et injecter dans l'ancien process.
        f.stop_native159();
        assert!(
            evidence152(&f)
                .iter()
                .all(|event| ordinary_number159(event).is_none()),
            "aucune ordinary injectée par l'ancienne incarnation arrêtée"
        );
    }
    // Un gate de l'ancien processus ne doit jamais terminer le tour1 du nouveau.
    for entry in fs::read_dir(&f.root).unwrap() {
        let path = entry.unwrap().path();
        if path
            .file_name()
            .unwrap()
            .to_str()
            .is_some_and(|name| name.starts_with("release-turn159-"))
        {
            fs::remove_file(path).unwrap();
        }
    }
    f.daemon = Some(spawn_crash_daemon159(&f.root, None));
    let until = Instant::now() + Duration::from_secs(10);
    while std::os::unix::net::UnixStream::connect(f.socket()).is_err() {
        assert!(Instant::now() < until, "daemon redémarré absent");
        thread::sleep(Duration::from_millis(10));
    }
    if point == "after_delivery_acked" {
        // Une mission native terminée est volontairement nettoyée au restart.
        // Prouver l'absence réelle, ne pas promettre un autorespawn inexistant.
        let mut observer = support::register_agent_as(&f.socket(), PARENT, INSTANCE);
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            observer.send(WrapperToDaemon::ListAgents);
            let response = f.receive(&mut observer);
            let DaemonToWrapper::AgentList { agents } = response else {
                panic!("ListAgents attendu : {response:?}");
            };
            let state = agents
                .iter()
                .find(|agent| agent.agent_id == child)
                .map(|agent| agent.state.as_str());
            if state.is_none() || state == Some("stopped") {
                break;
            }
            assert!(Instant::now() < until, "cible encore active : {state:?}");
            thread::sleep(Duration::from_millis(10));
        }
        let phase:String=db.query_row("SELECT phase FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key='held159-crash-order-1'",
            [support::SCOPE],|r|r.get(0)).unwrap();
        assert_eq!(phase, "acked");
        let after_rows:Vec<(String,String,i64,Vec<u8>,i64)>=db.prepare(
            "SELECT idempotency_key,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' ORDER BY queue_rank")
            .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))
            .unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(
            after_rows, frozen_rows,
            "id/gen/bytes/rang conservés cible absente"
        );
        let held:i64=db.query_row("SELECT COUNT(*) FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key IN ('held159-crash-order-2','held159-crash-order-3') AND phase='held'",
            [support::SCOPE],|r|r.get(0)).unwrap();
        assert_eq!(held, 2);
        assert_eq!(
            evidence152(&f)
                .iter()
                .filter_map(ordinary_number159)
                .collect::<Vec<_>>(),
            vec![1],
            "ACK ne réinjecte pas ; suivants retenus tant que cible absente"
        );
        f.stop_native159();
        let current = explicit_relaunch159(&mut f, &mut observer, &child, &_instance, false);
        assert_ne!(
            current, _instance,
            "SC015 vraie nouvelle incarnation après daemon puis porteur"
        );
        // La suite vérifie ordinary1 déjàACKED puis les deux held conservés, dans l'ordre.
    }
    // RepriseFleet peut réaffecter une nouvelle incarnation. La génération et
    // les bytes de chaque remise restent figés ; l'ordre ne s'assouplit jamais.
    for number in 1..=3_usize {
        let until = Instant::now() + Duration::from_secs(20);
        let event = loop {
            let events = evidence152(&f);
            let ordinary = events
                .iter()
                .filter_map(|e| ordinary_number159(e).map(|n| (n, e)))
                .collect::<Vec<_>>();
            for (index, (observed, _)) in ordinary.iter().enumerate() {
                assert_eq!(
                    *observed,
                    index + 1,
                    "ordinary1 doit précéder2 ; aucun doublon toléré"
                );
            }
            if let Some((_, event)) = ordinary.get(number - 1) {
                break (*event).clone();
            }
            assert!(
                Instant::now() < until,
                "message ordinaire{number} absent après reprise"
            );
            thread::sleep(Duration::from_millis(1));
        };
        assert!(
            event["prompt"]
                .as_str()
                .unwrap()
                .contains(&format!("HELD159_CRASH_ORDER_{number}"))
        );
        if point == "after_idempotent_socket_push" && number == 1 {
            assert_ne!(
                event["instance_id"].as_str(),
                Some(_instance.as_str()),
                "ordinary1 injectée réellement dans la nouvelle incarnation après les deux arrêts"
            );
        }
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            let phase:String=db.query_row("SELECT phase FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key=?2",
                rusqlite::params![support::SCOPE,format!("held159-crash-order-{number}")],|r|r.get(0)).unwrap();
            if phase == "acked" {
                break;
            }
            assert!(Instant::now() < until, "ACK réel ordinary{number} absent");
            thread::sleep(Duration::from_millis(1));
        }
        let after_rows:Vec<(String,String,i64,Vec<u8>,i64)>=db.prepare(
            "SELECT idempotency_key,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' ORDER BY queue_rank")
            .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))
            .unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(
            after_rows, frozen_rows,
            "id/gen/enveloppe/rang conservés malgré réaffectation"
        );
        let events = evidence152(&f);
        let current = events
            .iter()
            .rev()
            .find(|e| e["event"] == "started" && e["agent_id"] == child)
            .unwrap();
        if event["instance_id"] == current["instance_id"] {
            if number < 3 {
                let count:i64=db.query_row("SELECT COUNT(*) FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' AND phase='held'",
                    [support::SCOPE],|r|r.get(0)).unwrap();
                assert_eq!(
                    count,
                    3 - number as i64,
                    "suivants retenus tant que ce tour réel n'est pas fini"
                );
            }
            let turn = event["turn_number"].as_u64().unwrap();
            support::private_write(&f.root.join(format!("release-turn159-{turn}")), []).unwrap();
        } // Un tour de l'ancien process arrêté n'est jamais relancé par un gate synthétique.
    }
    let events = evidence152(&f);
    let ordinary = events
        .iter()
        .filter_map(ordinary_number159)
        .collect::<Vec<_>>();
    assert_eq!(
        ordinary,
        vec![1, 2, 3],
        "ordreSC003 et injection unique, carte de reprise exclue"
    );
    if let Some(mut daemon) = f.daemon.take() {
        daemon.stop();
    }
    if let Ok(marker) = ManagedMarkerStore::at_directory(f.state.join("managed")).load(&child) {
        f.native = Some(marker);
    }
}

// Relance du porteur existant par la façade : même définition durable et même agent.
// La tâche native terminale reste terminale ; aucun modèle réel/API extérieure.
fn bootstrap_card159(event: &Value) -> bool {
    if event["event"] != "turn156" {
        return false;
    }
    let Some(text) = event["params"]["input"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v["text"].as_str())
    else {
        return false;
    };
    let Some(header) = text
        .lines()
        .next()
        .and_then(|s| s.strip_prefix("[Message Bridget : "))
        .and_then(|s| s.strip_suffix(']'))
    else {
        return false;
    };
    serde_json::from_str::<Value>(header).is_ok_and(|header| header["from"] == "bridget-reprise")
}

// La carte existe réellement dans les captures84178665/eee87d63/d9940 :
// Terminer seulement cette carte ; une file retenue peut déjà avoir consommé le repos.
fn finish_bootstrap_card159(
    f: &mut Fixture,
    primary: &mut support::Client,
    child: &str,
    instance: &str,
    require_idle: bool,
) {
    let until = Instant::now() + Duration::from_secs(10);
    let event = loop {
        if let Some(event) = evidence152(f)
            .into_iter()
            .find(|e| e["instance_id"] == instance && bootstrap_card159(e))
        {
            break event;
        }
        assert!(Instant::now() < until, "carte de reprise effective absente");
        thread::sleep(Duration::from_millis(10));
    };
    let turn = event["turn_number"].as_u64().unwrap();
    let gate = f.root.join(format!("release-turn159-{turn}"));
    support::private_write(&gate, []).unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let events = evidence152(f);
        let completed = events.iter().any(|e| {
            e["event"] == "completed159" && e["instance_id"] == instance && e["turn_number"] == turn
        });
        let ordinary_started = events.iter().any(|e| {
            e["instance_id"] == instance
                && e["turn_number"]
                    .as_u64()
                    .is_some_and(|number| number > turn)
                && ordinary_number159(e).is_some()
        });
        primary.send(WrapperToDaemon::ListAgents);
        let DaemonToWrapper::AgentList { agents } = f.receive(primary) else {
            panic!("AgentList réel attendu");
        };
        let target = agents.iter().find(|agent| agent.agent_id == child);
        if let Some(target) = target {
            assert!(
                matches!(
                    target.agent_type.as_str(),
                    "fake-codex" | "project-discovery-fake-codex"
                ),
                "type effectif inattendu : {}",
                target.agent_type
            );
            if completed
                && (target.state == "connected"
                    || (!require_idle && target.state == "busy" && ordinary_started))
            {
                break;
            }
        }
        assert!(Instant::now() < until, "carte non terminée/repos non reçu");
        thread::sleep(Duration::from_millis(10));
    }
    fs::remove_file(gate).unwrap(); // Plus aucun signal ancien ne traverse une incarnation.
}

fn explicit_relaunch159(
    f: &mut Fixture,
    primary: &mut support::Client,
    child: &str,
    previous: &str,
    require_idle: bool,
) -> String {
    use bridget_transport::protocol::RelaunchOutcome;
    let command = uuid::Uuid::new_v4().to_string();
    primary.send(WrapperToDaemon::RelaunchOrder {
        agent_id: child.into(),
        command_id: command.clone(),
    });
    match f.receive(primary) {
        DaemonToWrapper::RelaunchResult {
            command_id,
            outcome: RelaunchOutcome::Started { agent_id, .. },
        } => {
            assert_eq!(command_id, command);
            assert_eq!(agent_id, child);
        }
        other => panic!("vraie relance explicite refusée : {other:?}"),
    }
    let until = Instant::now() + Duration::from_secs(10);
    let event = loop {
        if let Some(event) = evidence152(f).into_iter().rev().find(|e| {
            e["event"] == "started" && e["agent_id"] == child && e["instance_id"] != previous
        }) {
            break event;
        }
        assert!(Instant::now() < until, "nouveau porteur physique absent");
        thread::sleep(Duration::from_millis(10));
    };
    let marker = ManagedMarkerStore::at_directory(f.state.join("managed"))
        .load(child)
        .unwrap();
    assert_eq!(event["instance_id"], marker.instance_id);
    let pid = event["pid"].as_u64().unwrap() as u32;
    let birth = process_birth(pid).unwrap();
    let observed = Command::new("/bin/ps")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
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
            .and_then(|v| v.parse::<u32>().ok()),
        Some(marker.pgid)
    );
    assert!(
        command.contains(f.root.join("provider.py").to_str().unwrap())
            && !command.to_lowercase().contains("firefox")
    );
    assert_eq!(process_birth(pid).ok(), Some(birth));
    f.initial_provider = Some((pid, birth));
    f.native = Some(marker.clone());
    finish_bootstrap_card159(f, primary, child, &marker.instance_id, require_idle);
    marker.instance_id
}

fn send_ordinary159(sender: &mut support::Client, child: &str, id: &str, body: &str) {
    let mut message = BridgetMessage::new(PARENT, child, body);
    message.id = id.into();
    sender.send(WrapperToDaemon::SendIdempotent {
        message_id: id.into(),
        message,
        issued_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Accepted { .. }
                | IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(_),
                    ..
                },
            ..
        }
    ));
}

// Les deux cas arrêt/restart du porteur réemploient le même SpawnOrder de production.
// Ils n'ouvrent pas une délégation dont la garde terminale arrêterait le porteur.
fn bootstrap_carrier159(busy: bool) -> (Fixture, support::Client, support::Client, String, String) {
    let mut f = Fixture::with_options(Some(true), true, None);
    let mut primary = support::register_agent_as(&f.socket(), PARENT, INSTANCE);
    let wanted = uuid::Uuid::new_v4().to_string();
    f.native_agent = Some(wanted.clone());
    let command = uuid::Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    primary.send(WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "fake-codex".into(),
        project: None,
        ownership: None,
        agent_id: Some(wanted),
        cwd: f.root.to_string_lossy().into_owned(),
        persistent: false,
        command_id: command.clone(),
        issued_at: now,
        deadline_at: now + 60,
    });
    let child = match f.receive(&mut primary) {
        DaemonToWrapper::SpawnAccepted {
            command_id,
            agent_id,
            definition,
        } => {
            assert_eq!(command_id, command);
            f.carrier_definition =
                Some(definition.expect("définition durable du nouveau Spawn réel"));
            agent_id
        }
        other => panic!("porteur autonome réel non lancé : {other:?}"),
    };
    f.native_agent = Some(child.clone());
    let until = Instant::now() + Duration::from_secs(10);
    let event = loop {
        if let Some(event) = evidence152(&f)
            .into_iter()
            .rev()
            .find(|e| e["event"] == "started" && e["agent_id"] == child)
        {
            break event;
        }
        assert!(Instant::now() < until, "provider autonome absent");
        thread::sleep(Duration::from_millis(10));
    };
    let marker = ManagedMarkerStore::at_directory(f.state.join("managed"))
        .load(&child)
        .unwrap();
    assert_eq!(event["instance_id"], marker.instance_id);
    let pid = event["pid"].as_u64().unwrap() as u32;
    let birth = process_birth(pid).unwrap();
    let observed = Command::new("/bin/ps")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
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
            .and_then(|v| v.parse::<u32>().ok()),
        Some(marker.pgid)
    );
    assert!(
        command.contains(f.root.join("provider.py").to_str().unwrap())
            && !command.to_lowercase().contains("firefox")
    );
    assert_eq!(process_birth(pid).ok(), Some(birth));
    f.initial_provider = Some((pid, birth));
    let instance = marker.instance_id.clone();
    f.native = Some(marker);
    let mut sender = support::Client::connect(&f.socket());
    sender.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    support::attest_agent(&f.socket(), &mut sender, PARENT, INSTANCE);
    sender.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: support::SCOPE.into(),
        capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    finish_bootstrap_card159(&mut f, &mut primary, &child, &instance, true);
    if busy {
        send_ordinary159(
            &mut sender,
            &child,
            "held159-carrier-initial",
            "HELD159_INITIAL",
        );
        wait_busy159(&mut primary, &child);
    }
    (f, primary, sender, child, instance)
}

#[test]
fn held159_porteur_arrete_sans_message_en_cours() {
    let (mut f, mut primary, _sender, child, instance) = bootstrap_carrier159(false);
    primary.send(WrapperToDaemon::ListAgents);
    let DaemonToWrapper::AgentList { agents } = f.receive(&mut primary) else {
        panic!("AgentList réel attendu");
    };
    assert!(
        agents
            .iter()
            .any(|agent| agent.agent_id == child && agent.state == "connected")
    );
    assert!(
        evidence152(&f)
            .iter()
            .filter(|event| event["event"] == "turn156")
            .all(bootstrap_card159),
        "aucun message ordinaire avant l'arrêt à vide"
    );
    f.stop_native159();
    let current = explicit_relaunch159(&mut f, &mut primary, &child, &instance, true);
    assert_ne!(current, instance);
    assert!(
        evidence152(&f)
            .iter()
            .filter(|event| event["event"] == "turn156")
            .all(bootstrap_card159),
        "seules cartes réellement terminées, aucun ordinaire inventé à vide"
    );
}

#[test]
fn held159_porteur_arrete_avec_message_en_cours_preserve_ordre() {
    let (mut f, mut primary, mut sender, child, instance) = bootstrap_carrier159(true);
    let db = rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    for number in 1..=2 {
        send_ordinary159(
            &mut sender,
            &child,
            &format!("held159-crash-order-{number}"),
            &format!("HELD159_CRASH_ORDER_{number}"),
        );
    }
    let frozen:Vec<(String,i64,Vec<u8>,i64)>=db.prepare("SELECT delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' AND phase='held' ORDER BY queue_rank")
        .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap();
    assert_eq!(frozen.len(), 2);
    assert!(
        evidence152(&f)
            .iter()
            .filter_map(ordinary_number159)
            .next()
            .is_none()
    );
    f.stop_native159();
    let current = explicit_relaunch159(&mut f, &mut primary, &child, &instance, false);
    assert_ne!(current, instance);
    for number in 1..=2_usize {
        let until = Instant::now() + Duration::from_secs(20);
        let event = loop {
            let events = evidence152(&f);
            let ordinary = events
                .iter()
                .filter_map(|e| ordinary_number159(e).map(|n| (n, e)))
                .collect::<Vec<_>>();
            for (index, (n, _)) in ordinary.iter().enumerate() {
                assert_eq!(*n, index + 1, "ordre sans doublon");
            }
            if let Some((_, event)) = ordinary.get(number - 1) {
                break (*event).clone();
            }
            assert!(Instant::now() < until, "message retenu{number} non repris");
            thread::sleep(Duration::from_millis(1));
        };
        assert_eq!(event["instance_id"], current);
        let after:Vec<(String,i64,Vec<u8>,i64)>=db.prepare("SELECT delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' ORDER BY queue_rank")
            .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(after, frozen);
        if number == 1 {
            let held:i64=db.query_row("SELECT COUNT(*) FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key='held159-crash-order-2' AND phase='held'",[support::SCOPE],|r|r.get(0)).unwrap();
            assert_eq!(held, 1);
        }
        support::private_write(
            &f.root.join(format!(
                "release-turn159-{}",
                event["turn_number"].as_u64().unwrap()
            )),
            [],
        )
        .unwrap();
    }
    assert_eq!(
        evidence152(&f)
            .iter()
            .filter_map(ordinary_number159)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
}

// T010 : vraie interruption du porteur après Seen durable et avant toute injection.
// La reconstruction physique reprend l'identité exacte, jamais une nouvelle mission.
#[test]
#[cfg_attr(target_os = "linux", ignore = "watch_marker exige kqueue Darwin/BSD")]
fn held159_porteur_arrete_apres_seen_reprise_meme_instance_indeterminee() {
    use bridget_daemon::managed_process::{
        ManagedIdentity, ManagedLaunch, spawn_managed_bootstrap_with_stderr,
    };
    use std::io::Read;
    use std::os::unix::process::ExitStatusExt;
    let (mut f, mut primary, mut sender, child, instance) = bootstrap_carrier159(true);
    let db = rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    primary.send(WrapperToDaemon::ListAgents);
    let DaemonToWrapper::AgentList { agents } = f.receive(&mut primary) else {
        panic!("AgentList réel attendu");
    };
    let target = agents
        .iter()
        .find(|agent| agent.agent_id == child)
        .expect("porteur réel présent");
    assert_eq!(target.state, "busy");
    let agent_type = target.agent_type.clone();
    let definition = f.carrier_definition.clone().unwrap();
    let old = ManagedMarkerStore::at_directory(f.state.join("managed"))
        .load(&child)
        .unwrap();
    let old_provider = f.initial_provider.unwrap().0;
    assert_eq!(old.instance_id, instance);
    for number in 1..=2 {
        send_ordinary159(
            &mut sender,
            &child,
            &format!("held159-crash-order-{number}"),
            &format!("HELD159_CRASH_ORDER_{number}"),
        );
    }
    let frozen:Vec<(String,String,i64,Vec<u8>,i64)>=db.prepare("SELECT idempotency_key,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' AND phase='held' ORDER BY queue_rank")
        .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap().collect::<Result<_,_>>().unwrap();
    assert_eq!(frozen.len(), 2);
    let sync = f.root.join("sync");
    let point = "after_receipt_seen_before_injection";
    support::arm_checkpoint(&sync, &[point], point);
    let initial = evidence152(&f)
        .into_iter()
        .find(|e| {
            e["event"] == "turn156"
                && e["instance_id"] == instance
                && e["params"]["input"][0]["text"]
                    .as_str()
                    .is_some_and(|text| {
                        text.lines()
                            .next()
                            .unwrap_or_default()
                            .contains("\"id\":\"held159-carrier-initial\"")
                    })
        })
        .unwrap();
    support::private_write(
        &f.root.join(format!(
            "release-turn159-{}",
            initial["turn_number"].as_u64().unwrap()
        )),
        [],
    )
    .unwrap();
    support::watch_marker(&sync, &sync.join(format!("{point}.ready")));
    let before:(String,String,i64,i64)=db.query_row("SELECT phase,recipient_instance_id,delivery_generation,turn_gate FROM send_deliveries WHERE delivery_id=?1",[&frozen[0].1],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).unwrap();
    assert_eq!(
        before,
        ("dispatching".into(), instance.clone(), frozen[0].2, 1)
    );
    let receipt = f
        .state
        .join("state/bridget/receipts")
        .join(&instance)
        .join(format!("receipt-{}", frozen[0].1));
    let seen = fs::read(&receipt).unwrap();
    assert!(seen.starts_with(b"seen\n"));
    assert!(
        evidence152(&f)
            .iter()
            .filter_map(ordinary_number159)
            .next()
            .is_none(),
        "Seen précède réellement l'injection"
    );
    let held: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id=?1 AND phase='held'",
            [&frozen[1].1],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 1);
    // Couper le daemon d'abord : aucune relève Fleet concurrente de l'ancien wrapper.
    let mut crashed = f.daemon.take().unwrap();
    crashed.stop();
    assert_eq!(
        crashed.0.try_wait().unwrap().unwrap().signal(),
        Some(libc::SIGTERM)
    );
    drop(crashed);
    drop(sender);
    drop(primary);
    f.stop_native159();
    assert_ne!(process_birth(old.pgid).ok(), Some(old.birth));
    assert_eq!(
        fs::read(&receipt).unwrap(),
        seen,
        "aucun reçu forgé ou effacé"
    );
    // Désarmer l'ancienne FIFO sans writer après TERM, retirer les gates consommés.
    fs::remove_file(sync.join(format!("{point}.fifo"))).unwrap();
    fs::remove_file(sync.join(format!("{point}.ready"))).unwrap();
    for entry in fs::read_dir(&f.root).unwrap() {
        let path = entry.unwrap().path();
        if path
            .file_name()
            .unwrap()
            .to_str()
            .is_some_and(|name| name.starts_with("release-turn159-"))
        {
            fs::remove_file(path).unwrap();
        }
    }
    // Issue indéterminée constatée avant un futur tick ; pas de promesse sur les futurs repos réels.
    let reported = "after_delivery_indeterminate";
    support::arm_checkpoint(&sync, &[reported], reported);
    // La réconciliation startup termine tous les anciens markers vivants.
    // Démarrer AVANT le nouveau bootstrap ; le porteur générique est non persistant.
    f.daemon = Some(spawn_crash_daemon159(&f.root, Some(&sync)));
    let until = Instant::now() + Duration::from_secs(10);
    while std::os::unix::net::UnixStream::connect(f.socket()).is_err() {
        assert!(Instant::now() < until, "daemon repris absent");
        thread::sleep(Duration::from_millis(10));
    }
    let fleet: Value =
        serde_json::from_slice(&fs::read(f.state.join("bridget.fleet.json")).unwrap()).unwrap();
    assert_eq!(fleet["equipiers"][&child]["persistent"], false);
    assert_eq!(fleet["equipiers"][&child]["lifecycle_state"], "stopped");
    assert!(
        ManagedMarkerStore::at_directory(f.state.join("managed"))
            .load(&child)
            .is_err(),
        "ancien marker mort reconcilié avant le nouveau bootstrap"
    );
    assert_eq!(
        evidence152(&f)
            .iter()
            .filter(|e| e["event"] == "started" && e["agent_id"] == child)
            .count(),
        1,
        "aucun autorespawn non persistant"
    );
    let isolated = support::isolated_command(&f.root);
    let mut env = isolated
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|value| (key.to_string_lossy().into_owned(), value.to_owned()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    env.insert("BRIDGET_HOME".into(), f.state.clone().into_os_string());
    env.insert("BRIDGET_SOCKET".into(), f.socket().into_os_string());
    env.insert(
        "BRIDGET_TEST_SYNC_DIR".into(),
        sync.clone().into_os_string(),
    );
    env.insert("BRIDGET_T010_DEFAULT_TERM".into(), "1".into());
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity: ManagedIdentity {
            instance_id: old.instance_id.clone(),
            command_id: old.command_id.clone(),
            generation: old.generation,
        },
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".into(),
            agent_type,
            child.clone(),
            serde_json::to_string(&definition).unwrap(),
        ],
        cwd: f.root.clone(),
        env,
    };
    let stderr = f.root.join("same-instance-wrapper.log");
    support::private_write(&stderr, []).unwrap();
    let mut bootstrap = spawn_managed_bootstrap_with_stderr(
        &launch,
        Stdio::from(fs::OpenOptions::new().append(true).open(&stderr).unwrap()),
    )
    .unwrap();
    bootstrap
        .set_ready_timeout(Duration::from_secs(10))
        .unwrap();
    let marked = bootstrap
        .wait_ready()
        .unwrap()
        .persist_marker(
            &ManagedMarkerStore::at_directory(f.state.join("managed")),
            &child,
        )
        .unwrap();
    let current = ManagedMarkerStore::at_directory(f.state.join("managed"))
        .load(&child)
        .unwrap();
    assert_eq!(
        (
            &current.instance_id,
            &current.command_id,
            current.generation
        ),
        (&old.instance_id, &old.command_id, old.generation)
    );
    assert_ne!(current.pgid, old.pgid);
    assert_eq!(process_birth(current.pgid).unwrap(), current.birth);
    let mut running = marked.release().unwrap();
    running.set_status_nonblocking().unwrap();
    f.native = Some(current.clone());
    f.manual_carrier = Some(running);
    let until = Instant::now() + Duration::from_secs(10);
    let started = loop {
        if let Some(event) = evidence152(&f).into_iter().rev().find(|e| {
            e["event"] == "started"
                && e["instance_id"] == instance
                && e["pid"]
                    .as_u64()
                    .is_some_and(|pid| pid as u32 != old_provider)
        }) {
            break event;
        }
        assert!(
            Instant::now() < until,
            "nouveau fournisseur même incarnation absent"
        );
        thread::sleep(Duration::from_millis(10));
    };
    let pid = started["pid"].as_u64().unwrap() as u32;
    let birth = process_birth(pid).unwrap();
    let observed = Command::new("/bin/ps")
        .args([
            "-ww",
            "-p",
            &pid.to_string(),
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
        Some(current.pgid)
    );
    assert!(
        command.contains(f.root.join("provider.py").to_str().unwrap())
            && !command.to_lowercase().contains("firefox")
    );
    assert_eq!(process_birth(pid).ok(), Some(birth));
    f.initial_provider = Some((pid, birth));
    support::watch_marker(&sync, &sync.join(format!("{reported}.ready")));
    let after:(String,String,i64)=db.query_row("SELECT phase,recipient_instance_id,delivery_generation FROM send_deliveries WHERE delivery_id=?1",[&frozen[0].1],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
    assert_eq!(
        after,
        ("indeterminate".into(), instance.clone(), frozen[0].2),
        "rejeu Seen réel, aucun ACK d'injection fabriqué"
    );
    let held: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM send_deliveries WHERE delivery_id=?1 AND phase='held'",
            [&frozen[1].1],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 1);
    let after_rows:Vec<(String,String,i64,Vec<u8>,i64)>=db.prepare("SELECT idempotency_key,delivery_id,delivery_generation,message_bytes,queue_rank FROM send_deliveries WHERE issuer_scope=?1 AND idempotency_key LIKE 'held159-crash-order-%' ORDER BY queue_rank")
        .unwrap().query_map([support::SCOPE],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap().collect::<Result<_,_>>().unwrap();
    assert_eq!(after_rows, frozen);
    assert_eq!(fs::read(&receipt).unwrap(), seen);
    assert!(
        evidence152(&f)
            .iter()
            .filter_map(ordinary_number159)
            .next()
            .is_none(),
        "ni réinjection du Seen ni faux tour terminal"
    );
    // EOF = aucun StartupFailed ; le vrai report daemon ci-dessus prouve Register.
    let mut status = String::new();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        match f
            .manual_carrier
            .as_mut()
            .unwrap()
            .status_reader()
            .read_to_string(&mut status)
        {
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < until, "canal managed-status encore ouvert");
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("statut managed réel illisible : {error}"),
        }
    }
    assert!(status.is_empty(), "StartupFailed effectif : {status}");
    f.manual_carrier.as_mut().unwrap().close_status();
    if let Some(mut daemon) = f.daemon.take() {
        daemon.stop();
    } // Aucun writer de la FIFO après TERM.
}
