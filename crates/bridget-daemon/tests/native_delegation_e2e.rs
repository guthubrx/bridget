//! Session148 : vrai daemon/superviseur/wrapper, fournisseur Claude/GLM fermé.
//! Aucun T3, secret utilisateur, réseau fournisseur ou modèle réel.
#[allow(dead_code)]
#[path = "support/idempotent.rs"]
mod support;

use bridget_core::BridgetMessage;
use bridget_daemon::desired_state::{DesiredStateStore, path_for_daemon_db};
use bridget_daemon::fleet::{FleetConfig, FleetSupervisor, SpawnOrder, SpawnSubmission};
use bridget_daemon::idempotency::IdempotencyStore;
use bridget_daemon::managed_process::{
    BootstrapReady, ManagedMarker, ManagedMarkerStore, group_exists, process_birth, signal_group,
    wait_group_gone,
};
use bridget_transport::protocol::{NativeDelegationRequest, SpawnOwnership, SpawnPosture};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::fs;
use std::os::fd::FromRawFd;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const PARENT: &str = "14800000-0000-4000-8000-000000000001";
const INSTANCE: &str = "native148-parent-instance";
const ANSWER: &str = "fixture-glm5.3-answer-148";

// Le harnais existant fournit environnement fermé, client, Register et preuve.
// Ses watchdogs et nettoyages SIGKILL ne sont pas utilisés par cette recette.
struct OwnedChild(Child);
impl OwnedChild {
    fn crash(&mut self) {
        assert!(self.0.try_wait().unwrap().is_none());
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
        assert_eq!(
            command
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<u32>().ok()),
            Some(std::process::id())
        );
        assert!(command.contains(env!("CARGO_BIN_EXE_bridget")));
        assert!(!command.to_lowercase().contains("firefox"));
        // SIGABRT interrompt le daemon sans son arrêt propre, sans SIGKILL.
        assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGABRT) }, 0);
        assert!(!self.wait(Duration::from_secs(10)).success());
    }
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
    native_recovery152: bool,
}
impl Fixture {
    fn new() -> Self {
        Self::new_with_notify_timeout(2)
    }
    fn new_with_notify_timeout(notify_timeout_secs: u64) -> Self {
        Self::new_with_provider(notify_timeout_secs, false)
    }
    fn new_with_provider(notify_timeout_secs: u64, native_recovery152: bool) -> Self {
        let root = fs::canonicalize("/tmp").unwrap().join(format!(
            "n148-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..10]
        ));
        for relative in ["", "state", "provider", "tmp"] {
            support::private_dir(&root.join(relative)).unwrap();
        }
        let provider = root.join("provider.py");
        let (command, args) = if native_recovery152 {
            let script = provider_recovery152();
            support::private_write(&provider, &script).unwrap();
            for relative in ["bin", ".git", "provider/.claude"] {
                support::private_dir(&root.join(relative)).unwrap();
            }
            support::private_write(
                &root.join("provider/.claude/settings.json"),
                serde_json::to_vec(&json!({"permissions":{
                    "defaultMode":"plan",
                    "deny":["Bash","Edit","Write","NotebookEdit","WebFetch","WebSearch"]
                }}))
                .unwrap(),
            )
            .unwrap();
            let cli = compile_provider152(&root, &script);
            (cli.to_string_lossy().into_owned(), json!([]))
        } else {
            support::private_write(
                &provider,
                include_bytes!("fixtures/native_delegation_148.py"),
            )
            .unwrap();
            (
                "/usr/bin/python3".to_string(),
                json!([provider, root.join("evidence.jsonl")]),
            )
        };
        support::private_write(&root.join("evidence.jsonl"), []).unwrap();
        let mut registry = json!({"agents":{"fake-glm":{
            "command":command, "args":args,
            "protocol":"claude_stream_json", "permissions":"deny", "queue_capacity":4,"notify_timeout_secs":notify_timeout_secs,
            "forbidden_env":["ANTHROPIC_API_KEY","OPENAI_API_KEY","T3CODE_HOME","BRIDGET_T3_MCP_ENDPOINT","BRIDGET_T3_MCP_AUTHORIZATION"],"pass_env":[],
            "capabilities":{"execution_paths":["claude_stream_json"],"models":{"glm5.3":{"efforts":["high"]}}}
        }}});
        if native_recovery152 {
            registry["agents"]["fake-glm"]["claude_config_dir"] =
                json!(root.join("provider/.claude"));
        }
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
            native_recovery152,
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
        if self.native_recovery152 {
            command.env(
                "PATH",
                format!(
                    "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                    self.root.join("bin").display()
                ),
            );
        }
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
    fn crash(&mut self) {
        self.daemon.as_mut().unwrap().crash();
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
        self.stop_native_groups();
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
        keep_alive_secs: None,
        brief: None,
    }
}

// T004/T048 : adapter la fixture fermée sur disque, sans nouveau hook du daemon.
// Le MCP est lancé par le fournisseur : il porte la vraie identité de l'enfant.
fn compile_provider152(root: &Path, script: &str) -> PathBuf {
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
    let cli = root.join("bin/claude");
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

fn provider_recovery152() -> String {
    let started = r#"record("started", provider="closed_fixture", model=model, effort=effort, t3_present=False)"#;
    let identity = r#"record("started", provider="closed_fixture", model=model, effort=effort, t3_present=False,
       instance_id=os.environ.get("BRIDGET_AGENT_INSTANCE_ID"),
       agent_id=os.environ.get("BRIDGET_AGENT_ID"),
       argv=sys.argv[1:],
       native_bootstrap=json.loads(os.environ.get("BRIDGET_NATIVE_MISSION_BOOTSTRAP", "null")))"#;
    let resumed_init = r#"initialized = "--resume" in args
if initialized:
    emit({"type": "system", "subtype": "init", "model": model,
          "session_id": args[args.index("--resume") + 1]})
for line in sys.stdin:"#;
    let permissions = r#"mode = args[args.index("--permission-mode") + 1]
assert mode == "plan"
denied = {"Bash", "Edit", "Write", "NotebookEdit", "WebFetch", "WebSearch"}
profile = os.environ["CLAUDE_CONFIG_DIR"]
with open(os.path.join(profile, "settings.json"), encoding="utf-8") as settings_file:
    permissions = json.load(settings_file)["permissions"]
assert permissions["defaultMode"] == mode
assert denied.issubset(set(permissions["deny"]))
tools = ["Read", "Glob", "Grep"]
if "--tools" in args:
    tools = [tool for tool in args[args.index("--tools") + 1].split(",") if tool]
assert set(tools).issubset({"Read", "Glob", "Grep"})
if "--restricted" not in args:
    assert args[args.index("--permission-prompts") + 1] == "none"
    assert denied.issubset(set(args[args.index("--disallowedTools") + 1].split(",")))"#;
    let observed_init = r#"def emit(frame):
    if frame.get("type") == "system" and frame.get("subtype") == "init":
        frame.update(permissionMode=mode, tools=tools, cwd=os.getcwd())
        record("init152", instance_id=os.environ.get("BRIDGET_AGENT_INSTANCE_ID"),
               frame=frame, config_dir=profile, argv=sys.argv[1:])
    print(json.dumps(frame), flush=True)"#;
    let nested = r#"    if "NESTED_152" in content and os.path.exists(os.path.join(os.path.dirname(evidence), "delegate-152")):
        import subprocess
        import select
        mcp = subprocess.Popen([BRIDGET152, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.DEVNULL, text=True)
        def rpc152(frame):
            mcp.stdin.write(json.dumps(frame) + "\n")
            mcp.stdin.flush()
            if not select.select([mcp.stdout], [], [], 8)[0]:
                raise TimeoutError("MCP natif152 sans réponse")
            return json.loads(mcp.stdout.readline())
        initialized152 = rpc152({"jsonrpc":"2.0", "id":0, "method":"initialize", "params":{}})
        assert "protocolVersion" in initialized152["result"]
        mcp.stdin.write(json.dumps({"jsonrpc":"2.0", "method":"notifications/initialized"}) + "\n")
        mcp.stdin.flush()
        delegated152 = rpc152({"jsonrpc":"2.0", "id":1, "method":"tools/call",
                              "params":{"name":"bridget_delegate", "arguments":{
                                  "request_id":"native152-grandchild", "agent_type":"fake-glm",
                                  "model":"glm5.3", "effort":"high", "task":"WAIT_CANCEL_148",
                                  "cwd":os.path.dirname(evidence), "posture":"discovery",
                                  "reply_timeout":60}}})
        record("nested152", instance_id=os.environ.get("BRIDGET_AGENT_INSTANCE_ID"),
               response=delegated152)
        mcp.stdin.close()
        mcp.wait(timeout=8)
    if blocked:
        record("blocked152", instance_id=os.environ.get("BRIDGET_AGENT_INSTANCE_ID"))
        continue"#;
    let provider = include_str!("fixtures/native_delegation_148.py")
        .replacen("args = sys.argv[2:]", "args = sys.argv[1:]", 1)
        .replacen(
            "evidence = sys.argv[1]",
            "evidence = os.path.join(os.path.dirname(os.path.abspath(__file__)), \"evidence.jsonl\")",
            1,
        )
        .replacen(
            "assert \"--restricted\" in args\nassert args[args.index(\"--permission-mode\") + 1] == \"plan\"",
            permissions,
            1,
        )
        .replacen(
            "def emit(frame):\n    print(json.dumps(frame), flush=True)",
            observed_init,
            1,
        )
        .replacen(started, identity, 1)
        .replacen("initialized = False\nfor line in sys.stdin:", resumed_init, 1)
        .replacen("    if blocked:\n        continue", nested, 1)
        // T019 réemploie ce fournisseur : mêmes permissions et réponses148/152.
        .replacen(
            "record(\"prompt\", blocked=blocked,",
            "record(\"prompt\", instance_id=os.environ.get(\"BRIDGET_AGENT_INSTANCE_ID\"), content=content, blocked=blocked,",
            1,
        )
        .replacen(
            "    text = \"fixture-glm5.3-answer-148\"",
            r#"    text = "fixture-glm5.3-answer-148"
    # La carte peut citer une mission passée ; retenir le marqueur récent.
    for round153 in (3, 2, 1):
        if f"RONDE_153_{round153}" in content:
            text = f"fixture-153-result-{round153}"
            break"#,
            1,
        );
    assert!(provider.contains("native_bootstrap="));
    assert!(provider.contains("delegated152 = rpc152"));
    provider.replacen(
        "#!/usr/bin/python3\n",
        &format!(
            "#!/usr/bin/python3\nBRIDGET152 = {}\n",
            serde_json::to_string(env!("CARGO_BIN_EXE_bridget")).unwrap()
        ),
        1,
    )
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

fn working_native152(nested: bool) -> (Fixture, support::Client, Value) {
    let mut f = Fixture::new_with_provider(15, true);
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    f.human_grant();
    let task = if nested {
        "WAIT_CANCEL_148 NESTED_152"
    } else {
        "WAIT_CANCEL_148"
    };
    let created = f.request(&mut parent, delegate(&f.root.clone(), "recovery-152", task));
    let id = created["task_id"].as_str().expect("mission admise");
    f.status_until(&mut parent, id, "working");
    let saved = native_task152(&f, id);
    f.remember_native_group(saved["child"].as_str().unwrap());
    assert_native_identity152(&f, id, created["message_id"].as_str().unwrap());
    // Le prompt initial a franchi le contrôle du fichier avant le crash.
    wait_evidence152(&f, "blocked152", saved["child_instance"].as_str().unwrap());
    (f, parent, created)
}

#[test]
fn native152_rouge_panne_mission_working() {
    let (mut f, mut parent, created) = working_native152(true);
    let id = created["task_id"].as_str().unwrap();
    let mission = created["message_id"].as_str().unwrap();
    let original = native_task152(&f, id);
    let mut previous_instance = original["child_instance"].as_str().unwrap().to_string();
    let mut grandchild_before = None::<Value>;
    support::private_write(&f.root.join("delegate-152"), []).unwrap();
    for restart in 1..=2 {
        drop(parent);
        f.crash();
        // Chaque démarrage relit les magasins réels du même daemon.
        f.start();
        parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
        let resumed = f.status_until(&mut parent, id, "working");
        assert_eq!(resumed["task_id"], id);
        assert_eq!(resumed["message_id"], mission);
        let task = wait_native_ready152(&f, id, &previous_instance);
        f.remember_native_group(task["child"].as_str().unwrap());
        assert_eq!(task["restart_resumes"], restart);
        assert_eq!(task["root_owner_agent_id"], original["root_owner_agent_id"]);
        assert_eq!(
            task["origin_owner_instance"],
            original["origin_owner_instance"]
        );
        assert_eq!(task["parent_task_id"], original["parent_task_id"]);
        let instance = assert_native_identity152(&f, id, mission);
        assert_ne!(
            instance, previous_instance,
            "nouvelle génération native requise"
        );
        let nested = wait_evidence152(&f, "nested152", &instance);
        assert_ne!(nested["response"]["result"]["isError"], true, "{nested}");
        let result = &nested["response"]["result"]["structuredContent"];
        let child_id = result["task_id"]
            .as_str()
            .expect("descendant admis après reprise");
        if let Some(previous) = &grandchild_before {
            assert_eq!(child_id, previous["task_id"].as_str().unwrap());
        }
        let previous_child_instance = grandchild_before
            .as_ref()
            .and_then(|child| child["child_instance"].as_str())
            .unwrap_or("");
        let child = wait_native_ready152(&f, child_id, previous_child_instance);
        assert_eq!(child["parent_task_id"], id);
        assert_eq!(child["root_owner_agent_id"], PARENT);
        assert_eq!(child["restart_resumes"], restart - 1);
        if let Some(previous) = &grandchild_before {
            assert_eq!(child["mission"], previous["mission"]);
            assert_eq!(
                child["origin_owner_instance"],
                previous["origin_owner_instance"]
            );
        } else {
            grandchild_before = Some(child.clone());
        }
        f.remember_native_group(child["child"].as_str().unwrap());
        assert_native_identity152(&f, child_id, child["mission"].as_str().unwrap());
        previous_instance = instance;
    }
}

enum NativeBoundary152 {
    FleetAhead,
    LinkMissing,
    MarkerMismatch,
    MarkerReconciled,
}

fn native_identity_boundary152(boundary: NativeBoundary152) {
    let (mut f, parent, created) = working_native152(false);
    let id = created["task_id"].as_str().unwrap();
    let mission = created["message_id"].as_str().unwrap();
    let before = native_task152(&f, id);
    let name = before["child"].as_str().unwrap();
    let marker = f.marker_store().load(name).unwrap();
    drop(parent);
    f.crash();
    let fails_task = matches!(
        boundary,
        NativeBoundary152::LinkMissing | NativeBoundary152::MarkerMismatch
    );
    match boundary {
        NativeBoundary152::FleetAhead => {
            let entry = desired_store152(&f)
                .load()
                .unwrap()
                .equipiers
                .remove(name)
                .unwrap();
            let link = entry.agent_link.as_ref().unwrap();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let supervisor = FleetSupervisor::open(
                &f.root.join("state/bridget.db"),
                desired_store152(&f),
                FleetConfig::default(),
            )
            .unwrap();
            let order = SpawnOrder {
                posture: None,
                agent_type: entry.agent_type.clone(),
                project: entry.project.clone(),
                requested_name: Some(name.to_string()),
                cwd: entry.cwd.clone(),
                persistent: true,
                command_id: format!("boundary152-{}", uuid::Uuid::new_v4()),
                issued_at: now,
                deadline_at: now + 60,
                ownership: Some(SpawnOwnership {
                    parent_instance_id: link.parent_instance_id.clone(),
                    parent_execution_id: link.parent_execution_id.clone(),
                    objective_id: link.objective_id.clone(),
                    delegation_id: link.delegation_id.clone(),
                    project: link.project.clone(),
                    role: link.role.clone(),
                    max_children: Some(16),
                    max_depth: Some(8),
                }),
            };
            let SpawnSubmission::Start(lease) = supervisor.request_recovery(&order, now).unwrap()
            else {
                panic!("réservation de frontière native refusée");
            };
            supervisor
                .mark_starting(&lease, now, entry.resolved_definition.as_ref().unwrap())
                .unwrap();
            supervisor
                .register_connected(&lease, &lease.instance_id, now)
                .unwrap();
            f.marker_store()
                .persist(
                    name,
                    &BootstrapReady {
                        pid: marker.pgid,
                        pgid: marker.pgid,
                        birth: marker.birth,
                        instance_id: lease.instance_id.clone(),
                        command_id: lease.command_id.clone(),
                        generation: lease.generation,
                    },
                )
                .unwrap();
            assert_eq!(
                native_task152(&f, id)["child_instance"],
                before["child_instance"]
            );
            assert_ne!(
                lease.instance_id,
                before["child_instance"].as_str().unwrap()
            );
        }
        NativeBoundary152::LinkMissing => {
            let store = desired_store152(&f);
            let mut entry = store.load().unwrap().equipiers.remove(name).unwrap();
            entry.agent_link = None;
            store.upsert(name.to_string(), entry).unwrap();
            assert!(store.load().unwrap().equipiers[name].agent_link.is_none());
        }
        NativeBoundary152::MarkerMismatch => {
            f.marker_store()
                .persist(
                    name,
                    &BootstrapReady {
                        pid: marker.pgid,
                        pgid: marker.pgid,
                        birth: marker.birth,
                        instance_id: uuid::Uuid::new_v4().to_string(),
                        command_id: marker.command_id.clone(),
                        generation: marker.generation,
                    },
                )
                .unwrap();
            assert_ne!(
                f.marker_store().load(name).unwrap().instance_id,
                marker.instance_id
            );
        }
        NativeBoundary152::MarkerReconciled => {
            // La disparition du groupe précède le retrait du marqueur.
            f.stop_native_groups();
            assert!(!group_exists(marker.pgid).unwrap());
            f.marker_store().remove(name).unwrap();
            assert!(!f.root.join(format!("state/managed/{name}.json")).exists());
        }
    }
    f.start();
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    if fails_task {
        let failed = f.request(
            &mut parent,
            NativeDelegationRequest::Status { task_id: id.into() },
        );
        assert_eq!(failed["status"], "failed", "{failed}");
        assert_eq!(
            native_task152(&f, id)["error"],
            "native_restart_identity_mismatch"
        );
    } else {
        f.status_until(&mut parent, id, "working");
        wait_native_ready152(&f, id, before["child_instance"].as_str().unwrap());
        f.remember_native_group(name);
        assert_native_identity152(&f, id, mission);
    }
}

#[test]
fn native152_rouge_identite_frontieres_flotte_neuve_tache_ancienne() {
    native_identity_boundary152(NativeBoundary152::FleetAhead);
}

#[test]
fn native152_rouge_identite_frontieres_lien_absent() {
    native_identity_boundary152(NativeBoundary152::LinkMissing);
}

#[test]
fn native152_rouge_identite_frontieres_marqueur_lisible_different() {
    native_identity_boundary152(NativeBoundary152::MarkerMismatch);
}

#[test]
fn native152_rouge_identite_frontieres_marqueur_deja_reconcilie() {
    native_identity_boundary152(NativeBoundary152::MarkerReconciled);
}

#[test]
fn native153_e2e_trois_rondes() {
    // Réemploi152 : CLI/profil privés réellement capturés, sans T3 ni modèle.
    let mut f = Fixture::new_with_provider(15, true);
    let mut parent = support::register_agent_as(&support::socket(&f.root), PARENT, INSTANCE);
    f.human_grant();
    let db = rusqlite::Connection::open_with_flags(
        f.root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let (generation, mut previous_seq): (String, u64) = db
        .query_row(
            "SELECT generation,seq FROM native_delegation_projection_meta WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let instructions = [
        "RONDE_153_1 Revue des faits locaux.",
        "RONDE_153_2 Corrige les seuls points du delta.",
        "RONDE_153_3 Nouvelle revue des corrections.",
    ];
    let delta = json!({
        "objective":"Corriger les points relevés dans la revue.",
        "summary":"DELTA_BRIEF_153 : ajuster les seuls points signalés.",
        "next_step":"Relire les corrections, sans reprendre tout le dossier."
    });
    let mut request = delegate(&f.root, "three-rounds-153", instructions[0]);
    let NativeDelegationRequest::Delegate {
        keep_alive_secs, ..
    } = &mut request
    else {
        unreachable!()
    };
    // La borne haute évite l'expiration des premiers des16 enfants pendant
    // les admissions séquentielles. Leur fin reste explicite et contrôlée.
    *keep_alive_secs = Some(7200);
    let created = f.request(&mut parent, request);
    let id = created["task_id"]
        .as_str()
        .expect("mission admise")
        .to_string();
    let child = created["child_agent_id"].as_str().unwrap().to_string();
    f.status_until(&mut parent, &id, "result_available");
    let initial = native_task152(&f, &id);
    let instance = initial["child_instance"].as_str().unwrap().to_string();
    let marker = f.remember_native_group(&child);
    assert_eq!(
        assert_native_identity152(&f, &id, created["message_id"].as_str().unwrap()),
        instance
    );
    let mut missions = Vec::new();
    let mut result_ids = Vec::new();
    // Trois rondes bornées. Chaque attente reprend la borne20s du harnais.
    for (index, instruction) in instructions.iter().enumerate() {
        let round = index + 1;
        let admitted = if index == 0 {
            created.clone()
        } else {
            f.request(
                &mut parent,
                NativeDelegationRequest::Continue {
                    task_id: id.clone(),
                    request_id: format!("three-rounds-153-r{round}"),
                    task: (*instruction).into(),
                    brief: (index == 1).then(|| delta.clone()),
                },
            )
        };
        assert_eq!(admitted["task_id"], id, "{admitted}");
        assert_eq!(admitted["child_agent_id"], child, "{admitted}");
        let mission = admitted["message_id"].as_str().unwrap().to_string();
        let mut completed = f.status_until(&mut parent, &id, "result_available");
        let deadline = Instant::now() + Duration::from_secs(20);
        while completed["continuable"] != true
            || f.replies
                .iter()
                .all(|reply| reply.in_reply_to.as_deref() != Some(mission.as_str()))
        {
            assert!(
                Instant::now() < deadline,
                "remise ronde{round} absente : {completed}; {}",
                f.root.display()
            );
            completed = f.request(
                &mut parent,
                NativeDelegationRequest::Status {
                    task_id: id.clone(),
                },
            );
            assert_ne!(completed["status"], "failed", "{completed}");
            thread::sleep(Duration::from_millis(20));
        }
        let expected_result = format!("fixture-153-result-{round}");
        assert_eq!(completed["round"], round as u64);
        assert_eq!(completed["result"], expected_result);
        let saved = native_task152(&f, &id);
        assert_eq!(saved["child"], child);
        assert_eq!(saved["child_instance"], instance);
        assert_eq!(saved["round"], round as u64);
        assert_eq!(saved["result_sent"], true);
        assert_eq!(saved["cleanup_done"], false);
        assert_eq!(saved["guard_released"], false);
        assert!(saved["idle_deadline_at"].is_i64());
        assert_eq!(f.marker_store().load(&child).unwrap().pgid, marker.pgid);
        assert!(group_exists(marker.pgid).unwrap());
        let replies: Vec<_> = f
            .replies
            .iter()
            .filter(|reply| reply.in_reply_to.as_deref() == Some(mission.as_str()))
            .collect();
        assert_eq!(replies.len(), 1);
        let expected_id = if index == 0 {
            format!("native-result-{id}")
        } else {
            format!("native-result-{id}-r{round}")
        };
        assert_eq!(replies[0].id, expected_id);
        assert_eq!(replies[0].from, child);
        assert_eq!(replies[0].to, PARENT);
        assert_eq!(replies[0].body, expected_result);
        missions.push(mission);
        result_ids.push(replies[0].id.clone());
        let (observed_generation, seq): (String, u64) = db
            .query_row(
                "SELECT generation,seq FROM native_delegation_projection_meta WHERE singleton=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(observed_generation, generation);
        assert!(
            seq > previous_seq,
            "mutation visible absente à la ronde{round}"
        );
        previous_seq = seq;
        assert_eq!(
            db.query_row("SELECT COUNT(*) FROM native_delegations", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        if index == 1 {
            let body = saved["round_body"].as_str().unwrap();
            assert!(body.contains("DELTA_BRIEF_153"));
            assert!(body.contains("\n\n--- Consigne ---\n"));
            assert!(body.ends_with(*instruction));
        } else if index == 2 {
            assert_eq!(saved["round_body"], *instruction);
        }
    }
    assert_eq!(
        missions
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    assert_eq!(
        result_ids
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
    let evidence = evidence152(&f);
    let starts: Vec<_> = evidence
        .iter()
        .filter(|event| event["event"] == "started" && event["instance_id"] == instance)
        .collect();
    assert_eq!(starts.len(), 1, "une seule instance de fournisseur");
    let prompts: Vec<_> = evidence
        .iter()
        .filter(|event| event["event"] == "prompt" && event["instance_id"] == instance)
        .collect();
    assert_eq!(prompts.len(), 3, "trois missions réellement reçues");
    for (prompt, instruction) in prompts.iter().zip(instructions) {
        assert_eq!(
            prompt["pid"], starts[0]["pid"],
            "même processus fournisseur"
        );
        assert!(prompt["content"].as_str().unwrap().contains(instruction));
    }
    assert!(
        prompts[1]["content"]
            .as_str()
            .unwrap()
            .contains("DELTA_BRIEF_153")
    );
    let ended = f.request(
        &mut parent,
        NativeDelegationRequest::Cancel {
            task_id: id.clone(),
        },
    );
    assert_eq!(ended["status"], "result_available");
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let saved = native_task152(&f, &id);
        assert_eq!(saved["guard_released"], true);
        assert_eq!(saved["idle_deadline_at"], Value::Null);
        if saved["cleanup_done"] == true && !group_exists(marker.pgid).unwrap() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "fin explicite non confirmée ; {}",
            f.root.display()
        );
        f.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: id.clone(),
            },
        );
        thread::sleep(Duration::from_millis(20));
    }
    // Les16 suivants ont tous rendu leur résultat et restent réellement vivants.
    let mut idle_tasks = Vec::new();
    for index in 0..16 {
        let mut request = delegate(&f.root, &format!("idle153-{index}"), "Revue inactive153.");
        let NativeDelegationRequest::Delegate {
            keep_alive_secs, ..
        } = &mut request
        else {
            unreachable!()
        };
        *keep_alive_secs = Some(7200);
        let admitted = f.request(&mut parent, request);
        let idle_id = admitted["task_id"].as_str().unwrap().to_string();
        let mut completed = f.status_until(&mut parent, &idle_id, "result_available");
        let deadline = Instant::now() + Duration::from_secs(20);
        while completed["continuable"] != true {
            assert!(
                Instant::now() < deadline,
                "garde inactive153 absente : {completed}"
            );
            completed = f.request(
                &mut parent,
                NativeDelegationRequest::Status {
                    task_id: idle_id.clone(),
                },
            );
            assert_ne!(completed["status"], "failed", "{completed}");
            thread::sleep(Duration::from_millis(20));
        }
        let saved = native_task152(&f, &idle_id);
        assert_eq!(saved["state"], "result_available");
        assert_eq!(saved["result"], ANSWER);
        assert_eq!(saved["guarded"], true);
        assert_eq!(saved["guard_released"], false);
        assert_eq!(saved["cleanup_done"], false);
        assert!(saved["idle_deadline_at"].is_i64());
        let idle_marker = f.remember_native_group(saved["child"].as_str().unwrap());
        assert!(group_exists(idle_marker.pgid).unwrap());
        idle_tasks.push(idle_id);
    }
    let held: i64 = db.query_row(
        "SELECT COUNT(*) FROM native_delegations WHERE json_extract(payload,'$.state')='result_available' AND json_extract(payload,'$.guarded')=1 AND json_extract(payload,'$.cleanup_done')=0",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(held, 16);
    for idle_marker in f.native_groups.iter().skip(1) {
        assert!(group_exists(idle_marker.pgid).unwrap());
    }
    let mut seventeenth = delegate(&f.root, "idle153-17", "Délégation au plafond153.");
    let NativeDelegationRequest::Delegate {
        keep_alive_secs, ..
    } = &mut seventeenth
    else {
        unreachable!()
    };
    *keep_alive_secs = Some(7200);
    let refused = f.request(&mut parent, seventeenth);
    assert_eq!(refused["code"], "task_limit", "{refused}");
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM native_delegations", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        17
    );
    assert_eq!(
        evidence152(&f)
            .iter()
            .filter(|event| event["event"] == "started")
            .count(),
        17
    );
    for task_id in idle_tasks {
        let ended = f.request(&mut parent, NativeDelegationRequest::Cancel { task_id });
        assert_eq!(ended["status"], "result_available");
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
