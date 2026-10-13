//! T018/T023159B : réemploi local des fixtures privées held159_e2e, aucun modèle réel.
//! Copie minimale des fixtures guidance156_e2e ; pas d’API publique/framework nouveau.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;
use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{ManagedMarker, ManagedMarkerStore, process_birth};
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    NativeDelegationRequest, PresenceMode, ProviderPermissions, QueueEditAction, SpawnPosture,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    io::Write,
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
        let facade_argv = argv.first().copied() == Some(env!("CARGO_BIN_EXE_bridget"))
            && (argv.as_slice() == [env!("CARGO_BIN_EXE_bridget"), "mcp"]
                || argv.get(1).copied() == Some("queue")
                    && matches!(argv.get(2).copied(), Some("cancel" | "reorder" | "promote"))
                    && argv
                        .windows(2)
                        .any(|pair| pair[0] == "--request" && pair[1].starts_with("sc010-")));
        assert!(
            argv == performance_argv || crash_argv.as_ref() == Some(&argv) || facade_argv,
            "PID propre d'un worker ou d'une façade privée SC010 uniquement"
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
                    assert_ne!(
                        process_birth(marker.pgid).ok(),
                        Some(marker.birth),
                        "wrapper privé TERM non arrêté"
                    );
                }
            }
        } // Sans marker, les fournisseurs dont la preuve est conservée restent à nettoyer.
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
import select

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
active_turn = None
while True:
    if active_turn is not None:
        number, identifier = active_turn
        gate = os.path.join(os.path.dirname(evidence), "release-turn159-" + str(number))
        if os.path.exists(gate):
            emit(dict(method="item/agentMessage/delta", params=dict(threadId=thread_id, turnId=identifier, itemId="answer159", delta="fixture159-completed")))
            emit(dict(method="turn/completed", params=dict(threadId=thread_id, turn=dict(id=identifier, status="completed", items=[]))))
            record("completed159", turn_number=number, monotonic_ns=time.monotonic_ns())
            active_turn = None
    ready, _, _ = select.select([sys.stdin], [], [], 0.001)
    if not ready:
        continue
    line = sys.stdin.readline()
    if not line:
        break
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
        active_turn = (turn_number, turn_id)
    elif method == "turn/steer":
        params = frame["params"]
        assert params["threadId"] == thread_id
        assert params["expectedTurnId"] == turn_id
        record("steer156", params=params, turn_id=turn_id)
        emit(dict(id=frame["id"], result={}))
        # Le lecteur confirme la visibilité sur item/started ; le seul result ne la prouve pas.
        steered_item = dict(id="steered159-" + params["clientUserMessageId"], clientId=params["clientUserMessageId"], type="userMessage", content=params["input"])
        emit(dict(method="item/started", params=dict(threadId=thread_id, turnId=turn_id, item=steered_item)))
        emit(dict(method="item/completed", params=dict(threadId=thread_id, turnId=turn_id, item=steered_item)))
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
                DaemonToWrapper::Deliver {
                    message,
                    mid_turn_ok,
                } => {
                    // Notice durable contractuelle reprise après le COMMIT de cancel.
                    assert_eq!(message.from, "bridget");
                    assert_eq!(message.to, PARENT);
                    assert_eq!(mid_turn_ok, Some(true));
                    assert_eq!(
                        message.body,
                        format!(
                            "RETIRÉ DE LA FILE : votre message edit159-cancel-target pour {target} a été retiré avant livraison (cancelled_in_queue). Il ne sera pas livré."
                        )
                    );
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
        queue_fallback: false,
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
            machine_id: None,
            project_id: None,
            resume_on_limit: None,
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
        capabilities: vec![
            ClientCapability::SendIdempotent,
            ClientCapability::Lookup,
            ClientCapability::QueueListV1,
            ClientCapability::QueueEditV1,
        ],
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

// Ces fonctions de composition ont cinq consommateurs T018/T023.
fn edit159_frame(
    target: &str,
    action: QueueEditAction,
    request: &str,
    message: &str,
    before: Option<&str>,
) -> WrapperToDaemon {
    WrapperToDaemon::QueueEditRequest {
        request_id: request.into(),
        target: target.into(),
        action,
        message_id: message.into(),
        before_message_id: before.map(str::to_owned),
    }
}
fn edit159_reply(
    sender: &mut support::Client,
    request: &str,
    refusal: Option<&str>,
    promoted: bool,
) -> Value {
    let raw = sender.receive();
    let value = serde_json::to_value(&raw).unwrap();
    let mut expected = json!({"type":"QueueEditResult", "request_id":request,
        "state":if refusal.is_some(){"refused"}else{"applied"}});
    if let Some(code) = refusal {
        expected["refusal"] = json!(code);
    }
    if promoted {
        expected["effect"] = json!("unconfirmed");
    }
    assert_eq!(value, expected, "réponse wire réelle: {raw:?}");
    value
}
fn edit159_db(f: &Fixture) -> rusqlite::Connection {
    rusqlite::Connection::open_with_flags(
        f.state.join("bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap()
}
fn edit159_row(
    db: &rusqlite::Connection,
    key: &str,
) -> (String, String, i64, Vec<u8>, i64, Vec<u8>) {
    db.query_row("SELECT d.phase,d.delivery_id,d.delivery_generation,d.message_bytes,d.queue_rank,r.canonical_bytes FROM send_deliveries d JOIN idempotency_records r ON r.issuer_scope=d.issuer_scope AND r.operation_kind=d.operation_kind AND r.idempotency_key=d.idempotency_key WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
        rusqlite::params![support::SCOPE,key], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).unwrap()
}
fn edit159_send(sender: &mut support::Client, target: &str, id: &str) {
    let mut message = BridgetMessage::new(PARENT, target, format!("EDIT159_BODY_{id}"));
    message.id = id.into();
    sender.send(WrapperToDaemon::SendIdempotent {
        message_id: id.into(),
        message,
        issued_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
    });
    let raw = sender.receive();
    assert!(
        matches!(
            raw,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. }
                    | IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(_),
                        ..
                    },
                ..
            }
        ),
        "admission réelle: {raw:?}"
    );
}
fn edit159_observed(f: &Fixture, event_kind: &str) -> Vec<String> {
    evidence152(f)
        .iter()
        .filter(|e| e["event"] == event_kind)
        .filter_map(|event| {
            let items = event["params"]["input"].as_array()?;
            let text = items.first()?["text"].as_str()?;
            let header = text
                .lines()
                .next()?
                .strip_prefix("[Message Bridget : ")?
                .strip_suffix(']')?;
            let header: Value = serde_json::from_str(header).ok()?;
            let id = header["id"].as_str()?;
            id.starts_with("edit159-").then(|| id.to_owned())
        })
        .collect()
}
fn edit159_finish_turn(f: &Fixture, event: &Value) {
    support::private_write(
        &f.root.join(format!(
            "release-turn159-{}",
            event["turn_number"].as_u64().unwrap()
        )),
        [],
    )
    .unwrap();
}
fn edit159_wait_input(f: &Fixture, expected: &[&str]) {
    // FR024 : ACK depuis plus de 30 s, puis prochain tick (1 s) et marge de charge bornée.
    let until = Instant::now() + Duration::from_secs(40);
    loop {
        let actual = edit159_observed(f, "turn156");
        let wanted = expected.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(
            wanted.starts_with(&actual),
            "ordre/injection unique: {actual:?} vs {wanted:?}"
        );
        if actual == wanted {
            return;
        }
        assert!(
            Instant::now() < until,
            "entrée fournisseur absente: {actual:?} vs {wanted:?}"
        );
        thread::sleep(Duration::from_millis(1));
    }
}
fn edit159_current_turn(f: &Fixture) -> Value {
    evidence152(f)
        .into_iter()
        .rev()
        .find(|e| e["event"] == "turn156")
        .expect("tour fournisseur réel")
}
fn edit159_reconnect(f: &mut Fixture) -> (support::Client, support::Client) {
    let until = Instant::now() + Duration::from_secs(15);
    while std::os::unix::net::UnixStream::connect(f.socket()).is_err() {
        assert!(Instant::now() < until, "daemon privé redémarré absent");
        thread::sleep(Duration::from_millis(10));
    }
    let primary = support::register_agent_as(&f.socket(), PARENT, INSTANCE);
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
        capabilities: vec![
            ClientCapability::SendIdempotent,
            ClientCapability::Lookup,
            ClientCapability::QueueListV1,
            ClientCapability::QueueEditV1,
        ],
    });
    assert!(matches!(
        sender.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    (primary, sender)
}
#[test]
fn edit159_e2e_cancel_arret_avant_transaction() {
    edit159_cancel_boundary("before_queue_cancel_transaction");
}
#[test]
fn edit159_e2e_cancel_echec_interne_transaction() {
    edit159_cancel_boundary("after_queue_cancel_delivery_write");
}
#[test]
fn edit159_e2e_cancel_arret_apres_commit() {
    edit159_cancel_boundary("after_queue_cancel_commit");
}
fn edit159_cancel_boundary(point: &str) {
    let (mut f, parent, mut sender, child, instance, _state, _pid) = bootstrap159(Some(true), true);
    let db = edit159_db(&f);
    for id in ["edit159-cancel-target", "edit159-cancel-neighbor"] {
        edit159_send(&mut sender, &child, id);
    }
    let before = edit159_row(&db, "edit159-cancel-target");
    let neighbor = edit159_row(&db, "edit159-cancel-neighbor");
    assert_eq!(before.0, "held");
    assert_eq!(neighbor.0, "held");
    let task_id: String = db
        .query_row(
            "SELECT task_id FROM native_delegations WHERE json_extract(payload,'$.child')=?1",
            [&child],
            |r| r.get(0),
        )
        .unwrap();
    let sync = f.root.join("sync");
    support::arm_checkpoint(&sync, &[point], point);
    let request = edit159_frame(
        &child,
        QueueEditAction::Cancel,
        "edit159-cancel-request",
        "edit159-cancel-target",
        None,
    );
    let internal = point == "after_queue_cancel_delivery_write";
    if internal {
        let writer = rusqlite::Connection::open(f.state.join("bridget.db")).unwrap();
        writer.execute_batch("CREATE TRIGGER edit159_cancel_abort BEFORE INSERT ON queue_edits WHEN NEW.request_id='edit159-cancel-request' BEGIN SELECT RAISE(ABORT,'edit159 internal rollback'); END;").unwrap();
    }
    sender.send(request.clone());
    support::watch_marker(&sync, &sync.join(format!("{point}.ready")));
    if internal {
        // Vrai échec SQLite après écriture livraison, avant INSERT du reçu.
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .open(sync.join(format!("{point}.fifo")))
            .unwrap()
            .write_all(b"x")
            .unwrap();
        let raw = sender.receive();
        assert!(
            matches!(&raw,DaemonToWrapper::Nack{id,reason} if id=="edit159-cancel-request" && reason=="queue_edit_storage_unavailable"),
            "échec interne réel, aucun Applied: {raw:?}"
        );
    }
    let committed = point == "after_queue_cancel_commit";
    assert_eq!(
        edit159_row(&db, "edit159-cancel-target").0,
        if committed { "cancelled" } else { "held" }
    );
    let receipts:i64=db.query_row("SELECT COUNT(*) FROM queue_edits WHERE issuer_scope=?1 AND request_id='edit159-cancel-request'",[support::SCOPE],|r|r.get(0)).unwrap();
    assert_eq!(receipts, i64::from(committed));
    assert_eq!(edit159_row(&db, "edit159-cancel-neighbor"), neighbor);
    assert!(edit159_observed(&f, "turn156").is_empty());
    drop(f.daemon.take()); // SIGTERM individuel attesté, jamais groupe.
    drop(sender);
    drop(parent);
    // Le démarrage du daemon réconcilie et arrête le porteur natif ancien.
    // Pause de montage pendant l'arrêt : isole le retrait durable de la course
    // de libération A. Ne prouve PAS un retry avant libération hors pause.
    let private_control = rusqlite::Connection::open(f.state.join("bridget.db")).unwrap();
    let control_before = bridget_daemon::referent_control::read(&private_control).unwrap();
    let paused = bridget_daemon::referent_control::set(
        &private_control,
        bridget_daemon::referent_control::ControlMutation {
            command_id: "edit159-private-pause-after-stop",
            expected_generation: control_before.generation,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: Some("fixture T018 : retrait avant reprise autonome"),
            actor: "fixture-private-init",
            now: support::issued_at(),
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    assert!(paused.paused);
    drop(private_control);
    for suffix in ["fifo", "ready"] {
        let _ = fs::remove_file(sync.join(format!("{point}.{suffix}")));
    }
    if internal {
        rusqlite::Connection::open(f.state.join("bridget.db"))
            .unwrap()
            .execute_batch("DROP TRIGGER edit159_cancel_abort;")
            .unwrap();
    }
    f.daemon = Some(spawn_crash_daemon159(&f.root, None));
    let (mut parent, mut sender) = edit159_reconnect(&mut f);
    assert!(bridget_daemon::referent_control::read(&db).unwrap().paused);
    assert_eq!(
        edit159_row(&db, "edit159-cancel-target").0,
        if committed { "cancelled" } else { "held" }
    );
    assert_eq!(edit159_row(&db, "edit159-cancel-neighbor"), neighbor);
    // L'auteur réellement réattesté retire son message même hors ligne et en pause.
    sender.send(request.clone());
    let first = edit159_reply(&mut sender, "edit159-cancel-request", None, false);
    sender.send(request);
    assert_eq!(
        edit159_reply(&mut sender, "edit159-cancel-request", None, false),
        first
    );
    let after = edit159_row(&db, "edit159-cancel-target");
    assert_eq!(after.0, "cancelled");
    assert_eq!(after.5, before.5);
    assert_eq!(
        (&after.1, after.2, &after.3, after.4),
        (&before.1, before.2, &before.3, before.4)
    );
    assert_eq!(edit159_row(&db, "edit159-cancel-neighbor"), neighbor);
    // Levée par la véritable API du principal humain (SPEC087), pas UPDATE SQL.
    let mut human = support::Client::connect(&f.socket());
    human.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    });
    assert!(matches!(
        human.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    human.send(WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        // Périmètre canonique fixe de bridget-control-cli, communication.rs:128–139.
        // Même principal humain087 et même UID privé, aucun credential agent inventé.
        issuer_scope: "012_scope_88c53ef080a4199f1145b8edd4373646".into(),
        capabilities: vec![ClientCapability::ControlStateV1],
    });
    assert!(matches!(
        human.receive(),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    human.send(WrapperToDaemon::ControlStateRead { version: 1 });
    let raw = human.receive();
    let DaemonToWrapper::ControlState { state: control, .. } = raw else {
        panic!("contrôle humain réellement lu : {raw:?}")
    };
    assert!(control.paused);
    human.send(WrapperToDaemon::ControlStateSet {
        version: 1,
        command_id: uuid::Uuid::new_v4().to_string(),
        expected_generation: control.generation,
        paused: Some(false),
        auto_objectives_cap: None,
        reason: Some("fixture T018 : reprise après retrait validé".into()),
        agent_posture: None,
        auto_reassignment: None,
    });
    let raw = human.receive();
    assert!(
        matches!(&raw, DaemonToWrapper::ControlState { state, .. } if !state.paused),
        "levée réelle de pause : {raw:?}"
    );
    let saved = wait_native_ready152(&f, &task_id, &instance);
    assert_eq!(saved["child"], child);
    assert_eq!(saved["owner"], PARENT);
    assert_ne!(
        saved["child_instance"], instance,
        "nouveau porteur après réconciliation native152"
    );
    let fresh_provider = evidence152(&f)
        .into_iter()
        .rev()
        .find(|event| {
            event["event"] == "started" && event["instance_id"] == saved["child_instance"]
        })
        .unwrap();
    let fresh_pid = fresh_provider["pid"].as_u64().unwrap() as u32;
    assert_ne!(fresh_pid as u64, _pid);
    let fresh_birth = process_birth(fresh_pid).expect("nouveau fournisseur réellement vivant");
    f.native = Some(
        ManagedMarkerStore::at_directory(f.state.join("managed"))
            .load(&child)
            .unwrap(),
    );
    assert_eq!(
        f.native.as_ref().unwrap().instance_id,
        saved["child_instance"].as_str().unwrap()
    );
    f.initial_provider = Some((fresh_pid, fresh_birth));
    wait_busy159(&mut parent, &child);
    assert_eq!(edit159_row(&db, "edit159-cancel-target").0, "cancelled");
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    edit159_wait_input(&f, &["edit159-cancel-neighbor"]);
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    let until = Instant::now() + Duration::from_secs(10);
    while edit159_row(&db, "edit159-cancel-neighbor").0 != "acked" {
        assert!(Instant::now() < until, "ACK voisin absent");
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        edit159_observed(&f, "turn156"),
        vec!["edit159-cancel-neighbor"]
    );
    assert!(edit159_observed(&f, "steer156").is_empty());
    assert_eq!(edit159_row(&db, "edit159-cancel-target").0, "cancelled");
}
#[test]
fn edit159_e2e_recette_complete_reglage_ouvert() {
    let (f, _parent, mut sender, child, instance, _state, _pid) = bootstrap159(Some(true), true);
    let db = edit159_db(&f);
    for id in ["edit159-A", "edit159-B", "edit159-C", "edit159-D"] {
        edit159_send(&mut sender, &child, id);
        assert_eq!(edit159_row(&db, id).0, "held");
    }
    let originals =
        ["edit159-A", "edit159-B", "edit159-C", "edit159-D"].map(|id| edit159_row(&db, id));

    // SC010 : mêmes refus par les deux véritables façades, sans modèle réel.
    // Le pair est enregistré réellement ; le destinataire conserve sa preuve wrapper.
    let peer_id = "15900000-0000-4000-8000-000000000099";
    let peer_instance = "sc010-peer-instance159";
    let mut peer = support::Client::connect(&f.socket());
    peer.send(WrapperToDaemon::Register {
        identity_version: 2,
        agent_type: "fake-codex".into(),
        agent_id: peer_id.into(),
        host: Some("t1209".into()),
        transport: Some("codex_app_server".into()),
        channel: None.into(),
        mode: Some(PresenceMode::Acp),
        location: None,
        os: Some("test".into()),
        instance_id: Some(peer_instance.into()),
        domain: None,
        turn_in_progress: false,
        turn_attest: false,
        queue_fallback: false,
        journal_available: None,
    });
    let DaemonToWrapper::Registered {
        agent_id,
        credential: Some(credential),
    } = peer.receive()
    else {
        panic!("Registered réel du pair requis")
    };
    assert_eq!(agent_id, peer_id);
    support::save_fixture_credential(&f.socket(), peer_id, peer_instance, credential);
    let peer_name = f.state.join("sc010-peer-name");
    let recipient_name = f.state.join("sc010-recipient-name");
    support::private_write(&peer_name, peer_id).unwrap();
    support::private_write(&recipient_name, &child).unwrap();
    let receipt_before: i64 = db
        .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r.get(0))
        .unwrap();
    let observed_before = [
        edit159_observed(&f, "turn156"),
        edit159_observed(&f, "steer156"),
    ];
    let delegated_directory = f.state.join("delegated-pids");
    let delegated_path = delegated_directory.join(std::process::id().to_string());
    for actor in [
        "peer",
        "recipient",
        "bare-context-invalid",
        "child-proof-fixture",
    ] {
        if actor == "child-proof-fixture" {
            // Origine issue d'une fixture locale privée, non signée ; pas une interop T3.
            // Même structure149, avec PID/naissance réels et parent réellement enregistré.
            support::private_dir(&delegated_directory).unwrap();
            assert!(
                !delegated_path.exists(),
                "ne pas remplacer un marqueur étranger"
            );
            let marker = bridget_daemon::mcp_identity::DelegatedPidMarker {
                pid: std::process::id(),
                birth: process_birth(std::process::id()).unwrap(),
                parent_instance_id: peer_instance.into(),
                parent_name_file: peer_name.clone(),
                provider: "codex".into(),
                child_ref: "0123456789abcdef".into(),
                owner: "sc010-private-fixture159".into(),
            };
            support::private_write(&delegated_path, serde_json::to_vec(&marker).unwrap()).unwrap();
        }
        for action in ["cancel", "reorder", "promote"] {
            for surface in ["cli", "mcp"] {
                let request_id = format!("sc010-{surface}-{actor}-{action}");
                let mut command = support::isolated_command(&f.root);
                command
                    .env("BRIDGET_HOME", &f.state)
                    .env("BRIDGET_SOCKET", f.socket());
                match actor {
                    "peer" => {
                        command
                            .env("BRIDGET_AGENT_ID_FILE", &peer_name)
                            .env("BRIDGET_AGENT_INSTANCE_ID", peer_instance);
                    }
                    "recipient" => {
                        command
                            .env("BRIDGET_AGENT_ID_FILE", &recipient_name)
                            .env("BRIDGET_AGENT_INSTANCE_ID", &instance);
                    }
                    "bare-context-invalid" => {
                        // Ce contexte non attestable n'est pas le CLI humain Attach autorisé.
                        command
                            .env("BRIDGET_AGENT_ID_FILE", f.state.join("sc010-name-absent"))
                            .env("BRIDGET_AGENT_INSTANCE_ID", "sc010-invalid-instance");
                    }
                    "child-proof-fixture" => {}
                    _ => unreachable!(),
                }
                let stdout_path = f.root.join(format!("{request_id}.stdout"));
                let stderr_path = f.root.join(format!("{request_id}.stderr"));
                support::private_write(&stdout_path, []).unwrap();
                support::private_write(&stderr_path, []).unwrap();
                command
                    .stdout(
                        fs::OpenOptions::new()
                            .write(true)
                            .open(&stdout_path)
                            .unwrap(),
                    )
                    .stderr(
                        fs::OpenOptions::new()
                            .write(true)
                            .open(&stderr_path)
                            .unwrap(),
                    );
                let mut args = json!({"target":child,"action":action,"message_id":"edit159-A","request_id":request_id});
                if action == "reorder" {
                    args["before_message_id"] = json!("edit159-B");
                }
                let status;
                let reply;
                if surface == "cli" {
                    command.args([
                        "queue",
                        action,
                        "--target",
                        &child,
                        "--message",
                        "edit159-A",
                        "--request",
                        &request_id,
                    ]);
                    if action == "reorder" {
                        command.args(["--before", "edit159-B"]);
                    }
                    let mut process = OwnedChild(command.spawn().unwrap());
                    status = process.wait(Duration::from_secs(15));
                    let bytes = fs::read(&stdout_path).unwrap();
                    reply = serde_json::from_slice::<Value>(&bytes).ok();
                } else {
                    command.arg("mcp").stdin(Stdio::piped());
                    let mut process = OwnedChild(command.spawn().unwrap());
                    let mut input = process.0.stdin.take().unwrap();
                    for frame in [
                        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"sc010-private","version":"1"}}}),
                        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_queue","arguments":args}}),
                    ] {
                        writeln!(input, "{frame}").unwrap();
                    }
                    input.flush().unwrap();
                    let deadline = Instant::now() + Duration::from_secs(15);
                    let frame = loop {
                        if let Some(frame) = fs::read(&stdout_path)
                            .unwrap()
                            .split(|byte| *byte == b'\n')
                            .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
                            .find(|frame| frame["id"] == 2)
                        {
                            break frame;
                        }
                        assert!(
                            Instant::now() < deadline,
                            "réponse MCP SC010 absente : {request_id}"
                        );
                        thread::sleep(Duration::from_millis(10));
                    };
                    drop(input); // Pas d'EOF avant la réponse : serve cancel les appels en vol.
                    status = process.wait(Duration::from_secs(15));
                    reply = Some(frame);
                }
                let stderr = fs::read_to_string(&stderr_path).unwrap();
                if actor == "bare-context-invalid" || actor == "child-proof-fixture" {
                    let code = if actor == "bare-context-invalid" {
                        "identity_not_found"
                    } else if surface == "cli" {
                        "delegated_mcp_only"
                    } else {
                        "delegated_tool_forbidden"
                    };
                    if surface == "cli" {
                        assert!(!status.success());
                        assert!(stderr.contains(code), "{request_id}: {stderr}");
                    } else {
                        assert!(
                            reply.as_ref().unwrap().to_string().contains(code),
                            "{request_id}: {reply:?}"
                        );
                    }
                } else if surface == "cli" {
                    assert!(status.success(), "{request_id}: {stderr}");
                    let value = reply.as_ref().expect("résultat CLI final");
                    assert_eq!(value["request_id"], request_id);
                    assert_eq!(value["state"], "refused");
                    assert!(
                        value["refusal"]
                            .as_str()
                            .is_some_and(|code| code.starts_with("intent_not_permitted")),
                        "{request_id}: {value}"
                    );
                } else {
                    let value = reply.as_ref().unwrap();
                    if value["result"]["isError"] == true || value.get("error").is_some() {
                        let text = value.to_string();
                        assert!(
                            text.contains("intent_not_permitted")
                                || text.contains("capability_denied"),
                            "{request_id}: {value}"
                        );
                    } else {
                        let result: Value = serde_json::from_str(
                            value["result"]["content"][0]["text"]
                                .as_str()
                                .expect("résultat MCP textuel"),
                        )
                        .unwrap();
                        assert_eq!(result["request_id"], request_id);
                        assert_eq!(result["state"], "refused");
                        assert!(
                            result["refusal"]
                                .as_str()
                                .is_some_and(|code| code.starts_with("intent_not_permitted")),
                            "{request_id}: {result}"
                        );
                    }
                }
                for (i, id) in ["edit159-A", "edit159-B", "edit159-C", "edit159-D"]
                    .iter()
                    .enumerate()
                {
                    assert_eq!(
                        edit159_row(&db, id),
                        originals[i],
                        "SC010 : aucune phase/rang/enveloppe/canon changés {request_id}"
                    );
                }
                let receipts: i64 = db
                    .query_row("SELECT COUNT(*) FROM queue_edits", [], |r| r.get(0))
                    .unwrap();
                assert_eq!(
                    receipts, receipt_before,
                    "SC010 : aucun reçu écrit {request_id}"
                );
                assert_eq!(
                    [
                        edit159_observed(&f, "turn156"),
                        edit159_observed(&f, "steer156")
                    ],
                    observed_before,
                    "SC010 : aucun départ {request_id}"
                );
            }
        }
        if actor == "child-proof-fixture" {
            fs::remove_file(&delegated_path).unwrap();
        }
    }
    sender.send(WrapperToDaemon::QueueList {
        target: child.clone(),
    });
    let raw = sender.receive();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = raw
    else {
        panic!("liste wire réelle: {raw:?}")
    };
    assert_eq!(queue_bound, Some(4));
    assert_eq!(
        entries
            .iter()
            .map(|e| e["message_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["edit159-A", "edit159-B", "edit159-C", "edit159-D"]
    );
    assert!(entries.iter().all(|e| e["state"] == "held"));
    let reorder = edit159_frame(
        &child,
        QueueEditAction::Reorder,
        "edit159-reorder",
        "edit159-B",
        Some("edit159-A"),
    );
    sender.send(reorder.clone());
    let result = edit159_reply(&mut sender, "edit159-reorder", None, false);
    sender.send(reorder);
    assert_eq!(
        edit159_reply(&mut sender, "edit159-reorder", None, false),
        result
    );
    let cancel = edit159_frame(
        &child,
        QueueEditAction::Cancel,
        "edit159-cancel",
        "edit159-C",
        None,
    );
    sender.send(cancel.clone());
    let result = edit159_reply(&mut sender, "edit159-cancel", None, false);
    sender.send(cancel);
    assert_eq!(
        edit159_reply(&mut sender, "edit159-cancel", None, false),
        result
    );
    let promote = edit159_frame(
        &child,
        QueueEditAction::Promote,
        "edit159-promote",
        "edit159-D",
        None,
    );
    sender.send(promote.clone());
    let result = edit159_reply(&mut sender, "edit159-promote", None, true);
    sender.send(promote);
    assert_eq!(
        edit159_reply(&mut sender, "edit159-promote", None, true),
        result
    );
    let until = Instant::now() + Duration::from_secs(10);
    while edit159_observed(&f, "steer156") != ["edit159-D"] {
        assert!(
            Instant::now() < until,
            "guidage réel D absent/doublé: {:?}",
            edit159_observed(&f, "steer156")
        );
        thread::sleep(Duration::from_millis(1));
    }
    assert!(edit159_observed(&f, "turn156").is_empty());
    assert_eq!(edit159_row(&db, "edit159-C").0, "cancelled");
    let promoted_phase = edit159_row(&db, "edit159-D").0;
    assert!(
        matches!(promoted_phase.as_str(), "dispatching" | "acked"),
        "D libéré au porteur : {promoted_phase}"
    );
    for (i, id) in ["edit159-A", "edit159-B", "edit159-C", "edit159-D"]
        .iter()
        .enumerate()
    {
        let row = edit159_row(&db, id);
        assert_eq!(row.5, originals[i].5, "canon initial figé {id}");
        assert_eq!(
            (&row.1, row.2, &row.3),
            (&originals[i].1, originals[i].2, &originals[i].3),
            "canon/enveloppe/ID/génération figés {id}"
        );
    }
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    edit159_wait_input(&f, &["edit159-B"]);
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    edit159_wait_input(&f, &["edit159-B", "edit159-A"]);
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    let until = Instant::now() + Duration::from_secs(10);
    while edit159_row(&db, "edit159-A").0 != "acked" {
        assert!(Instant::now() < until, "ACK A absent");
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        edit159_observed(&f, "turn156"),
        vec!["edit159-B", "edit159-A"]
    );
    assert_eq!(edit159_observed(&f, "steer156"), vec!["edit159-D"]);
    let count:i64=db.query_row("SELECT COUNT(*) FROM queue_edits WHERE issuer_scope=?1 AND request_id IN ('edit159-reorder','edit159-cancel','edit159-promote')",[support::SCOPE],|r|r.get(0)).unwrap();
    assert_eq!(count, 3);
}
#[test]
fn edit159_e2e_recette_reglage_ferme_not_held() {
    let (f, mut parent, mut sender, child, _instance, _state, _pid) =
        bootstrap159(Some(false), true);
    wait_busy159(&mut parent, &child);
    let db = edit159_db(&f);
    edit159_send(&mut sender, &child, "edit159-closed");
    let before = edit159_row(&db, "edit159-closed");
    assert_ne!(before.0, "held");
    for (action, key) in [
        (QueueEditAction::Cancel, "edit159-closed-cancel"),
        (QueueEditAction::Reorder, "edit159-closed-reorder"),
        (QueueEditAction::Promote, "edit159-closed-promote"),
    ] {
        let request = edit159_frame(&child, action, key, "edit159-closed", None);
        sender.send(request.clone());
        let first = edit159_reply(&mut sender, key, Some("not_held"), false);
        sender.send(request);
        assert_eq!(
            edit159_reply(&mut sender, key, Some("not_held"), false),
            first
        );
        let after = edit159_row(&db, "edit159-closed");
        assert_eq!(after.5, before.5);
        assert_eq!(
            (&after.1, after.2, &after.3, after.4),
            (&before.1, before.2, &before.3, before.4)
        );
    }
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    edit159_wait_input(&f, &["edit159-closed"]);
    edit159_finish_turn(&f, &edit159_current_turn(&f));
    assert!(edit159_observed(&f, "steer156").is_empty());
}
