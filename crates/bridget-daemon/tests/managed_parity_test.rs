use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{ManagedMarkerStore, group_exists};
use bridget_transport::protocol::{AgentInfo, AttachWindow, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MATRIX_VERSION: &str = "fr-008-v1";
const REDUCED_PROMPT: &str = include_str!("fixtures/prompts/v1-after.txt");
const MATRIX_RUNS_PER_MODE: usize = 3;
const MATRIX_EXPECTED_TURNS: usize = 4;
/// turn_start + update(text) + reasoning(terminal) + turn_end — L4 C3.
const MATRIX_EVENTS_PER_TURN: usize = 4;
const MATRIX_TIMEOUT: Duration = Duration::from_secs(10);
// Seul QUEUE-SLOW doit franchir T/3 et prouver la relance différée. Sous
// contention, appliquer le même délai court aux tours nominaux injectait un
// rappel légitime dans le compteur du faux adaptateur et créait un 5e tour.
const MATRIX_FAST_REPLY_TIMEOUT_SECS: u64 = 30;
const MATRIX_SLOW_REPLY_TIMEOUT_SECS: u64 = 5;
const FROZEN_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";
static MANAGED_BENCH_LOCK: Mutex<()> = Mutex::new(());

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn test_root(label: &str) -> PathBuf {
    PathBuf::from(format!(
        "/tmp/bg909-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn write_fixture(root: &Path) -> PathBuf {
    let adapter = root.join("parity-acp.py");
    fs::create_dir_all(root.join(".config/bridget")).unwrap();
    fs::write(
        &adapter,
        r#"#!/usr/bin/python3
import json
import os
import sys
import time

turn = 0
for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "initialize":
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"protocolVersion":1}}), flush=True)
    elif method == "session/new":
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"sessionId":"parity-session"}}), flush=True)
    elif method == "session/prompt":
        prompt = request["params"]["prompt"][0]["text"]
        if "Carte de reprise Bridget (faits durables, aucune mémoire reconstruite)." in prompt:
            print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"stopReason":"end_turn"}}), flush=True)
            continue
        turn += 1
        if "QUEUE-SLOW" in prompt:
            time.sleep(2.2)
        update = {
            "jsonrpc":"2.0",
            "method":"session/update",
            "params":{
                "sessionId":"parity-session",
                "update":{
                    "sessionUpdate":"agent_message_chunk",
                    "content":{"type":"text","text":f"fixture-response-{turn}"}
                }
            }
        }
        print(json.dumps(update), flush=True)
        print(json.dumps({"jsonrpc":"2.0","id":request["id"],"result":{"stopReason":"end_turn"}}), flush=True)
        if os.environ.get("PARITY_SINGLE_TURN") == "1":
            break
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registry = serde_json::json!({
        "agents": {
            "parity": {
                "command": adapter,
                "protocol": "acp",
                "permissions": "allow",
                "queue_capacity": 8,
                "notify_timeout_secs": 2,
                "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
                "pass_env": ["PARITY_SINGLE_TURN"]
            }
        }
    });
    fs::write(
        root.join(".config/bridget/agents.json"),
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    fs::set_permissions(
        root.join(".config/bridget/agents.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    adapter
}

struct InteractivePromptSession {
    child: Child,
    release: PathBuf,
    done: PathBuf,
    slow_started: PathBuf,
    bootstrap_rejected: PathBuf,
}

impl InteractivePromptSession {
    fn finish(self) -> Vec<String> {
        fs::write(&self.release, b"release").unwrap();
        let output = self.child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "le wrapper interactif a échoué : {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&fs::read(&self.done).unwrap()).unwrap()
    }

    fn finish_without_mcp(self) {
        assert!(
            self.bootstrap_rejected.exists(),
            "le faux Codex n'a pas refusé l'amorçage muté"
        );
        assert!(
            !self.done.exists(),
            "le corpus MCP a été exécuté malgré l'amorçage muté"
        );
        fs::write(&self.release, b"release").unwrap();
        let output = self.child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "le wrapper interactif muté a échoué : {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !self.done.exists(),
            "le corpus MCP a été exécuté après le refus de l'amorçage"
        );
    }
}

fn start_interactive_prompt_session(
    root: &Path,
    name: &str,
    resume_arguments: Option<&[&str]>,
    mutate_resume_bootstrap: bool,
) -> InteractivePromptSession {
    let bin = root.join("prompt-bin");
    let capture = root.join("captured-prompt.json");
    let release = root.join("release-prompt-cli");
    let done = root.join("prompt-corpus-done.json");
    let who = root.join("prompt-who.json");
    let inbox = root.join("prompt-inbox.jsonl");
    let slow_started = root.join("prompt-slow-started");
    let bootstrap_rejected = root.join("prompt-bootstrap-rejected");
    let fake_tmux_root = root.join("fake-tmux");
    let codex = bin.join("codex");
    let tmux = bin.join("tmux");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&fake_tmux_root).unwrap();
    fs::write(
        &codex,
        r#"#!/usr/bin/python3
import json
import os
import re
import subprocess
import sys
import time

capture = os.environ["BRIDGET_PROMPT_CAPTURE"]
temporary = capture + ".tmp"
with open(temporary, "w", encoding="utf-8") as output:
    json.dump(sys.argv[1:], output, ensure_ascii=False)
os.replace(temporary, capture)

if os.environ["BRIDGET_REQUIRE_RESUME_BOOTSTRAP"] == "1":
    prompt = next(
        (argument for argument in sys.argv[1:] if argument.startswith("Tu reprends la session")),
        "",
    )
    if os.environ["BRIDGET_MUTATE_RESUME_BOOTSTRAP"] == "1":
        prompt = prompt.replace("ALL_TOOLS", "OUTILS_ABSENTS")
    required = (
        "Cherche mcp__bridget__* dans ALL_TOOLS via functions.exec",
        "appelle tools.mcp__bridget__bridget_send avec in_reply_to",
        "Utilise le shell bridget seulement si cette recherche ne rend aucun outil.",
    )
    if not all(instruction in prompt for instruction in required):
        with open(os.environ["BRIDGET_PROMPT_BOOTSTRAP_REJECTED"], "w") as signal:
            signal.write("rejected")
        while not os.path.exists(os.environ["BRIDGET_PROMPT_RELEASE"]):
            time.sleep(0.01)
        sys.exit(0)

marker = os.path.join(os.environ["HOME"], ".cache", "bridget", "agent-pids", str(os.getpid()))
deadline = time.monotonic() + 10
while not os.path.exists(marker):
    if time.monotonic() >= deadline:
        raise RuntimeError("marqueur d'identité MCP absent")
    time.sleep(0.01)

override = next(value for value in sys.argv[1:] if value.startswith("mcp_servers.bridget="))
match = re.search(r'command="([^"]+)"', override)
if match is None:
    raise RuntimeError("commande MCP absente de l'argument injecté")
bridget = match.group(1)
mcp = subprocess.Popen(
    [bridget, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    stderr=subprocess.PIPE, text=True,
)

def rpc(message, response=True):
    mcp.stdin.write(json.dumps(message, ensure_ascii=False) + "\n")
    mcp.stdin.flush()
    if response:
        return json.loads(mcp.stdout.readline())

rpc({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{}})
rpc({"jsonrpc":"2.0", "method":"notifications/initialized"}, False)
listed = rpc({"jsonrpc":"2.0", "id":2, "method":"tools/list", "params":{}})
assert any(tool["name"] == "bridget_who" for tool in listed["result"]["tools"])
observed = rpc({
    "jsonrpc":"2.0", "id":3, "method":"tools/call",
    "params":{"name":"bridget_who", "arguments":{}},
})
with open(os.environ["BRIDGET_PROMPT_WHO"] + ".tmp", "w", encoding="utf-8") as output:
    json.dump(observed, output, ensure_ascii=False)
os.replace(os.environ["BRIDGET_PROMPT_WHO"] + ".tmp", os.environ["BRIDGET_PROMPT_WHO"])

processed = []
offset = 0
deadline = time.monotonic() + 30
while len(processed) < 4:
    if time.monotonic() >= deadline:
        raise RuntimeError(f"corpus incomplet: {processed}")
    if not os.path.exists(os.environ["BRIDGET_PROMPT_INBOX"]):
        time.sleep(0.01)
        continue
    with open(os.environ["BRIDGET_PROMPT_INBOX"], encoding="utf-8") as source:
        source.seek(offset)
        lines = source.readlines()
        offset = source.tell()
    for line in lines:
        item = json.loads(line)
        envelope = item["envelope"]
        if "(reply=yes" not in envelope:
            continue
        header = envelope.split("\n", 1)[0]
        request_match = re.search(r"\(reply=yes, id=([^)]+)\)$", header)
        sender_match = re.match(r"💬 (.+?) →", header)
        if request_match is None or sender_match is None:
            raise RuntimeError(f"en-tête Bridget invalide: {header}")
        request_id = request_match.group(1)
        if len(request_id) <= 8:
            raise RuntimeError(f"identifiant Bridget tronqué: {request_id}")
        body = envelope.split("\n", 1)[1].split("\n\n⚠", 1)[0]
        if body == "QUEUE-SLOW":
            with open(os.environ["BRIDGET_PROMPT_SLOW"], "w") as signal:
                signal.write("started")
            time.sleep(2.2)
        response = {
            "TRACKED": "fixture-response-1",
            "QUEUE-SLOW": "fixture-response-slow",
            "QUEUE-NEXT": "fixture-response-next",
        }.get(body, body)
        sent = rpc({
            "jsonrpc":"2.0", "id":10 + len(processed), "method":"tools/call",
            "params":{
                "name":"bridget_send",
                "arguments":{
                    "to":sender_match.group(1),
                    "body":response,
                    "in_reply_to":request_id,
                },
            },
        })
        status = sent["result"]["structuredContent"]["status"]
        # Le dépôt a deux noms honnêtes : « accepted » quand l'accusé aval est
        # déjà consolidé, « in_flight » quand la remise est prise mais pas
        # encore accusée — le cas nominal d'un premier envoi. « outcome_unknown »
        # n'est plus un succès et ne doit plus être toléré ici.
        if status not in ("in_flight", "accepted"):
            raise RuntimeError(f"réponse MCP refusée: {sent}")
        processed.append(body)
with open(os.environ["BRIDGET_PROMPT_DONE"] + ".tmp", "w", encoding="utf-8") as output:
    json.dump(processed, output, ensure_ascii=False)
os.replace(os.environ["BRIDGET_PROMPT_DONE"] + ".tmp", os.environ["BRIDGET_PROMPT_DONE"])

while not os.path.exists(os.environ["BRIDGET_PROMPT_RELEASE"]):
    time.sleep(0.01)
mcp.stdin.close()
mcp.wait(timeout=3)
"#,
    )
    .unwrap();
    fs::write(
        &tmux,
        r#"#!/usr/bin/python3
import json
import os
import sys

args = sys.argv[1:]
root = os.environ["BRIDGET_FAKE_TMUX_ROOT"]
os.makedirs(root, exist_ok=True)
command = args[0]
def option(name):
    return args[args.index(name) + 1]
def buffer_path():
    return os.path.join(root, option("-b").replace("/", "_"))

if command == "display-message":
    print("%prompt-mcp\tfixture:0.0")
elif command == "load-buffer":
    with open(buffer_path(), "w", encoding="utf-8") as output:
        output.write(sys.stdin.read())
elif command == "paste-buffer":
    path = buffer_path()
    with open(path, encoding="utf-8") as source:
        envelope = source.read()
    try:
        with open(os.environ["BRIDGET_FAKE_LAST_SENDER"], encoding="utf-8") as source:
            reply_ref = source.read()
    except FileNotFoundError:
        reply_ref = ""
    record = json.dumps({"envelope":envelope, "reply_ref":reply_ref}, ensure_ascii=False) + "\n"
    fd = os.open(os.environ["BRIDGET_PROMPT_INBOX"], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    os.write(fd, record.encode("utf-8"))
    os.close(fd)
    os.unlink(path)
elif command == "show-buffer":
    sys.exit(1)
elif command == "capture-pane":
    print("›")
elif command == "delete-buffer":
    try:
        os.unlink(buffer_path())
    except FileNotFoundError:
        pass
"#,
    )
    .unwrap();
    fs::set_permissions(&codex, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&tmux, fs::Permissions::from_mode(0o700)).unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
    command.args(["codex", "--name", name]);
    if let Some(arguments) = resume_arguments {
        command.args(arguments);
    }
    let mut child = command
        .env_clear()
        .env("HOME", root)
        .env("PATH", format!("{}:{FROZEN_PATH}", bin.display()))
        .env("USER", "parity-test")
        .env("LANG", "C")
        .env("TMPDIR", "/tmp")
        .env("BRIDGET_CHANNEL", "unix")
        .env("BRIDGET_PROMPT_CAPTURE", &capture)
        .env("BRIDGET_PROMPT_RELEASE", &release)
        .env("BRIDGET_PROMPT_DONE", &done)
        .env("BRIDGET_PROMPT_WHO", &who)
        .env("BRIDGET_PROMPT_INBOX", &inbox)
        .env("BRIDGET_PROMPT_SLOW", &slow_started)
        .env("BRIDGET_PROMPT_BOOTSTRAP_REJECTED", &bootstrap_rejected)
        .env(
            "BRIDGET_REQUIRE_RESUME_BOOTSTRAP",
            if resume_arguments.is_some() { "1" } else { "0" },
        )
        .env(
            "BRIDGET_MUTATE_RESUME_BOOTSTRAP",
            if mutate_resume_bootstrap { "1" } else { "0" },
        )
        .env("BRIDGET_FAKE_TMUX_ROOT", &fake_tmux_root)
        .env(
            "BRIDGET_FAKE_LAST_SENDER",
            root.join(".cache/bridget")
                .join(format!("last-sender-{name}")),
        )
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !capture.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if !capture.exists() {
        let _ = child.kill();
        let output = child.wait_with_output().unwrap();
        panic!(
            "le vrai wrapper MCP n'a pas transmis son prompt : {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let arguments: Vec<String> = serde_json::from_slice(&fs::read(&capture).unwrap()).unwrap();
    if resume_arguments.is_some() {
        let resume = arguments
            .iter()
            .position(|argument| argument == "resume")
            .expect("sous-commande resume absente");
        let session = arguments
            .iter()
            .position(|argument| argument == "bridget-prospective")
            .expect("identifiant de session absent");
        let cd = arguments
            .iter()
            .position(|argument| argument == "--cd")
            .expect("option --cd absente");
        let bootstrap = arguments
            .iter()
            .position(|argument| argument.starts_with("Tu reprends la session"))
            .expect("amorçage Bridget absent de la reprise");
        assert!(resume < session && session < cd && cd + 1 < bootstrap);
        assert_eq!(
            bootstrap,
            arguments.len() - 1,
            "l'amorçage doit être PROMPT"
        );
        let actual = &arguments[bootstrap];
        assert!(actual.contains(name), "identité absente de l'amorçage");
        assert!(actual.contains("mcp__bridget__*"));
        assert!(actual.contains("tools.mcp__bridget__bridget_send"));
        assert!(actual.contains("shell bridget"));
    } else {
        let actual = arguments
            .iter()
            .find(|argument| argument.starts_with("Tu es l'agent"))
            .expect("prompt Bridget absent des arguments du CLI MCP");
        let expected = REDUCED_PROMPT
            .trim_end_matches('\n')
            .replace("agent-fixture", name);
        assert_eq!(
            actual, &expected,
            "le lancement MCP n'utilise pas la fixture réduite"
        );
    }

    if mutate_resume_bootstrap {
        wait_path(
            &bootstrap_rejected,
            "le faux Codex n'a pas refusé l'amorçage muté",
        );
        assert!(!who.exists(), "MCP a été ouvert malgré l'amorçage muté");
    } else {
        let deadline = Instant::now() + MATRIX_TIMEOUT;
        while !who.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        if !who.exists() {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "bridget_who n'a pas été appelé par le faux Codex : {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let observed: serde_json::Value = serde_json::from_slice(&fs::read(who).unwrap()).unwrap();
        assert!(
            observed["result"]["structuredContent"]["agents"]
                .as_array()
                .is_some_and(|agents| agents.iter().any(|agent| agent["name"] == name)),
            "la session MCP capturée n'apparaît pas dans bridget_who: {observed}"
        );
    }

    InteractivePromptSession {
        child,
        release,
        done,
        slow_started,
        bootstrap_rejected,
    }
}

fn write_cached_npx_fixture(root: &Path) {
    let adapter = write_fixture(root);
    let package = root.join("node_modules/parity-acp");
    let binaries = root.join("node_modules/.bin");
    fs::create_dir_all(&package).unwrap();
    fs::create_dir_all(&binaries).unwrap();
    let package_adapter = package.join("index.py");
    fs::copy(adapter, &package_adapter).unwrap();
    fs::set_permissions(&package_adapter, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(
        package.join("package.json"),
        br#"{"name":"parity-acp","version":"1.0.0","bin":{"parity-acp":"index.py"}}"#,
    )
    .unwrap();
    std::os::unix::fs::symlink("../parity-acp/index.py", binaries.join("parity-acp")).unwrap();
    let registry_path = root.join(".config/bridget/agents.json");
    let mut registry: serde_json::Value =
        serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
    registry["agents"]["parity"]["command"] = serde_json::json!("npx");
    registry["agents"]["parity"]["args"] =
        serde_json::json!(["--offline", "--no-install", "parity-acp"]);
    fs::write(
        &registry_path,
        serde_json::to_vec_pretty(&registry).unwrap(),
    )
    .unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();
}

/// Possède le daemon de parité. La garde existe avant le spawn : un échec
/// d'amorçage ne laisse jamais l'enfant sous PID 1. `Drop` ne panique jamais.
struct DaemonProcess {
    child: Option<Child>,
    socket: PathBuf,
}

impl DaemonProcess {
    fn start(root: &Path, billing_key: bool, single_turn: bool) -> Self {
        // Garde créée avant le spawn : le chemin d'erreur précoce (assert prêt,
        // panique injectée) libère encore le processus via Drop.
        let mut process = Self {
            child: None,
            socket: root.join(".cache/bridget/bridget.sock"),
        };
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", root)
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if billing_key {
            command.env("OPENAI_API_KEY", "forbidden-test-key");
        }
        if single_turn {
            command.env("PARITY_SINGLE_TURN", "1");
        }
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut accepting = false;
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                accepting = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            accepting,
            "le daemon de parité n'accepte pas encore les connexions"
        );
        process
    }

    fn stop(mut self) {
        let stopped = stop_daemon_child_best_effort(self.child.as_mut());
        if stopped {
            let _ = self.child.take();
            return;
        }
        // L'enfant reste dans Option pour que Drop retente sans paniquer pendant
        // le dépliage ; l'assertion explicite reste sur le chemin stop() heureux.
        panic!("le daemon de parité n'a pas terminé après SIGTERM");
    }

    fn kill(mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = unsafe { libc::kill(child.id() as i32, libc::SIGKILL) };
            let _ = child.wait();
        }
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let _ = stop_daemon_child_best_effort(self.child.as_mut());
    }
}

/// Arrêt best-effort : jamais de panique (y compris pendant un dépliage).
fn stop_daemon_child_best_effort(child: Option<&mut Child>) -> bool {
    let Some(child) = child else {
        return true;
    };
    match child.try_wait() {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => {}
    }
    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    child.wait().is_ok()
}

/// Compte les daemons dont le HOME est exactement celui de CE test.
/// Un compteur borné au PID du harnais croise les voisins parallèles.
fn daemon_count_for_home(home: &Path) -> usize {
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()
        .expect("ps pour l'oracle de non-fuite");
    assert!(output.status.success(), "ps indisponible pour l'oracle");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (pid, command) = line.split_once(char::is_whitespace)?;
            let command = command.trim_start();
            let mut parts = command.split_whitespace();
            let binary = parts.next()?;
            let argv1 = parts.next()?;
            if !binary.contains("bridget") || argv1 != "daemon" || parts.next().is_some() {
                return None;
            }
            Some(pid.trim())
        })
        .filter(|pid| daemon_home_is(pid, home))
        .count()
}

fn daemon_home_is(pid: &str, home: &Path) -> bool {
    let output = Command::new("lsof").args(["-p", pid, "-Fn"]).output();
    let Ok(output) = output else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix('n'))
        .any(|path| path_is_under_home(path, home))
}

fn path_is_under_home(path: &str, home: &Path) -> bool {
    let home = home.to_string_lossy();
    path == home.as_ref() || path.starts_with(&format!("{home}/"))
}

fn assert_daemon_count_for_home(home: &Path, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if daemon_count_for_home(home) == expected {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        daemon_count_for_home(home),
        expected,
        "daemon orphelin pour le HOME du test: {}",
        home.display()
    );
}

struct CutProxy {
    socket: PathBuf,
    latest_wrapper: Arc<Mutex<Option<(UnixStream, UnixStream)>>>,
    wrapper_connections: Arc<AtomicUsize>,
    stopping: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CutProxy {
    fn start(socket: PathBuf, target: PathBuf) -> Self {
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let latest_wrapper = Arc::new(Mutex::new(None));
        let wrapper_connections = Arc::new(AtomicUsize::new(0));
        let stopping = Arc::new(AtomicBool::new(false));
        let observed_wrapper = Arc::clone(&latest_wrapper);
        let observed_connections = Arc::clone(&wrapper_connections);
        let observed_stop = Arc::clone(&stopping);
        let handle = thread::spawn(move || {
            while !observed_stop.load(Ordering::SeqCst) {
                let client = match listener.accept() {
                    Ok((client, _)) => client,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("accept du proxy impossible: {error}"),
                };
                client.set_nonblocking(false).unwrap();
                let target_stream = connect_target_within_bound(&target);
                target_stream.set_nonblocking(false).unwrap();
                let current_wrapper = Arc::clone(&observed_wrapper);
                let connection_count = Arc::clone(&observed_connections);
                thread::spawn(move || {
                    let mut client_reader = BufReader::new(client.try_clone().unwrap());
                    let mut target_writer = target_stream.try_clone().unwrap();
                    let mut first_line = String::new();
                    if client_reader.read_line(&mut first_line).unwrap() == 0 {
                        return;
                    }
                    target_writer.write_all(first_line.as_bytes()).unwrap();
                    target_writer.flush().unwrap();
                    let is_wrapper = matches!(
                        decode::<WrapperToDaemon>(first_line.trim_end()),
                        Ok(WrapperToDaemon::Register { ref agent_type, .. })
                            if agent_type == "parity" || agent_type == "codex"
                    );
                    if is_wrapper {
                        *current_wrapper
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner()) = Some((
                            client.try_clone().unwrap(),
                            target_stream.try_clone().unwrap(),
                        ));
                        connection_count.fetch_add(1, Ordering::SeqCst);
                    }
                    let mut target_reader = target_stream.try_clone().unwrap();
                    let mut client_writer = client.try_clone().unwrap();
                    let reverse = thread::spawn(move || {
                        let _ = std::io::copy(&mut target_reader, &mut client_writer);
                        let _ = client_writer.shutdown(std::net::Shutdown::Both);
                    });
                    let _ = std::io::copy(&mut client_reader, &mut target_writer);
                    let _ = target_writer.shutdown(std::net::Shutdown::Both);
                    let _ = reverse.join();
                });
            }
        });
        Self {
            socket,
            latest_wrapper,
            wrapper_connections,
            stopping,
            handle: Some(handle),
        }
    }

    fn wrapper_connection_count(&self) -> usize {
        self.wrapper_connections.load(Ordering::SeqCst)
    }

    fn cut_wrapper_and_wait_for_reconnect(&self) {
        let before = self.wrapper_connection_count();
        let sockets = self
            .latest_wrapper
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
            .expect("connexion wrapper à couper");
        shutdown_for_cut(&sockets.0);
        shutdown_for_cut(&sockets.1);
        let deadline = Instant::now() + MATRIX_TIMEOUT;
        while self.wrapper_connection_count() == before && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            self.wrapper_connection_count(),
            before + 1,
            "le wrapper ne s'est pas reconnecté après la coupure réelle"
        );
    }

    fn stop(mut self) {
        self.stopping.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        let _ = fs::remove_file(&self.socket);
    }
}

/// La moitié du proxy peut déjà avoir atteint EOF quand l'autre la coupe.
/// `NotConnected` confirme alors la même coupure et ne rend pas la matrice
/// moins stricte : la reconnexion suivante reste attendue et contrôlée.
fn shutdown_for_cut(stream: &UnixStream) {
    match stream.shutdown(std::net::Shutdown::Both) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotConnected => {}
        Err(error) => panic!("coupure du proxy impossible: {error}"),
    }
}

/// Le fichier socket du daemon existe avant que son écoute soit atomiquement
/// disponible. Le proxy de test attend donc cette disponibilité bornée au lieu
/// de transformer ce démarrage concurrent en `ConnectionRefused` aléatoire.
fn connect_target_within_bound(target: &Path) -> UnixStream {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        match UnixStream::connect(target) {
            Ok(stream) => return stream,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
                ) && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("connexion du proxy au daemon impossible: {error}"),
        }
    }
}

fn start_daemon_behind_proxy(root: &Path) -> (DaemonProcess, CutProxy) {
    let cache_parent = root.join(".cache");
    let daemon_cache = root.join("daemon-cache");
    let proxy_cache = root.join("proxy-cache");
    fs::create_dir_all(&cache_parent).unwrap();
    fs::create_dir_all(&daemon_cache).unwrap();
    fs::create_dir_all(&proxy_cache).unwrap();
    let cache_link = cache_parent.join("bridget");
    std::os::unix::fs::symlink(&daemon_cache, &cache_link).unwrap();
    let mut daemon = DaemonProcess::start(root, false, false);
    let target = daemon_cache.join("bridget.sock");
    let database = daemon_cache.join("bridget.db");
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !daemon_ledger_is_ready(&database) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(target.exists());
    assert!(
        daemon_ledger_is_ready(&database),
        "le daemon n'a pas achevé l'initialisation de son ledger"
    );
    fs::remove_file(&cache_link).unwrap();
    std::os::unix::fs::symlink(&proxy_cache, &cache_link).unwrap();
    let proxy_socket = proxy_cache.join("bridget.sock");
    let proxy = CutProxy::start(proxy_socket, target);
    daemon.socket = cache_link.join("bridget.sock");
    (daemon, proxy)
}

/// Le fichier SQLite existe dès `Connection::open`, avant les DDL du store.
/// La bascule du proxy attend donc l'observable utile pour le premier Send.
fn daemon_ledger_is_ready(database: &Path) -> bool {
    let flags =
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let Ok(connection) = rusqlite::Connection::open_with_flags(database, flags) else {
        return false;
    };
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'ledger'",
            [],
            |_| Ok(()),
        )
        .is_ok()
}

struct Peer {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
    name: String,
}

impl Peer {
    fn register(socket: &Path, name: &str) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream.set_read_timeout(Some(MATRIX_TIMEOUT)).unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream);
        let mut peer = Self {
            reader,
            writer,
            name: name.to_string(),
        };
        peer.send(&WrapperToDaemon::Register {
            agent_type: "parity-client".to_string(),
            name: Some(name.to_string()),
            host: Some("fixture-host".to_string()),
            transport: Some("unix".to_string()),
            channel: None.into(),
            mode: Some(bridget_transport::protocol::PresenceMode::Acp),
            location: None,
            os: Some("fixture-os".to_string()),
            instance_id: Some(format!("instance-{name}")),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        match peer.recv() {
            DaemonToWrapper::Registered { name: assigned } => peer.name = assigned,
            other => panic!("enregistrement inattendu: {other:?}"),
        }
        peer
    }

    fn attach(socket: &Path) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream.set_read_timeout(Some(MATRIX_TIMEOUT)).unwrap();
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream);
        let mut peer = Self {
            reader,
            writer,
            name: "attach".to_string(),
        };
        peer.send(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        });
        assert!(matches!(
            peer.recv(),
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            }
        ));
        peer
    }

    fn send(&mut self, message: &WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn recv(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "EOF inattendu depuis le daemon");
        decode(line.trim_end()).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ModeObservables {
    replies: Vec<String>,
    request_states: Vec<String>,
    agent_transport: String,
    agent_state: String,
    journal: Vec<String>,
}

fn wait_agent(control: &mut Peer, name: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        control.send(&WrapperToDaemon::ListAgents);
        if let DaemonToWrapper::AgentList { agents } = control.recv()
            && let Some(agent) = agents.into_iter().find(|agent| agent.name == name)
        {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "l'équipier {name} ne s'est pas enregistré"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_busy_reconnected(control: &mut Peer, name: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        let agent = wait_agent(control, name);
        if agent.state == "busy" && agent.reconnect_count >= 1 {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "{name} n'a pas conservé busy après sa reconnexion"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_agent_state(control: &mut Peer, name: &str, expected: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        let agent = wait_agent(control, name);
        if agent.state == expected {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "{name} n'a pas atteint l'état {expected}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn send_tracked(peer: &mut Peer, to: &str, body: &str) -> String {
    send_tracked_with_timeout(peer, to, body, 5)
}

fn send_tracked_with_timeout(peer: &mut Peer, to: &str, body: &str, timeout: u64) -> String {
    let mut message = BridgetMessage::new(&peer.name, to, body);
    message.reply = true;
    message.reply_timeout = Some(timeout);
    let id = message.id.clone();
    peer.send(&WrapperToDaemon::Send(message));
    match peer.recv() {
        DaemonToWrapper::Ack { id: acked } => assert_eq!(acked, id),
        other => panic!("accusé inattendu: {other:?}"),
    }
    id
}

fn wait_path(path: &Path, description: &str) {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while !path.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(path.exists(), "{description}");
}

fn wait_reconnected(control: &mut Peer, name: &str) -> AgentInfo {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    loop {
        let agent = wait_agent(control, name);
        if agent.reconnect_count >= 1 {
            return agent;
        }
        assert!(
            Instant::now() < deadline,
            "{name} ne s'est pas reconnecté dans la même session"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn receive_replies(peer: &mut Peer, expected_ids: &[String]) -> Vec<String> {
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    let mut replies = Vec::new();
    while replies.len() < expected_ids.len() {
        assert!(Instant::now() < deadline, "réponses ACP incomplètes");
        match peer.recv() {
            DaemonToWrapper::Deliver(message) => {
                let in_reply_to = message.in_reply_to.as_deref().unwrap_or_default();
                assert_eq!(in_reply_to, expected_ids[replies.len()]);
                replies.push(message.body);
            }
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                let in_reply_to = message.in_reply_to.as_deref().unwrap_or_default();
                assert_eq!(in_reply_to, expected_ids[replies.len()]);
                replies.push(message.body);
                peer.send(&WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                });
            }
            _ => {}
        }
    }
    replies
}

fn run_interactive_prompt_corpus(
    socket: &Path,
    agent: &str,
    proxy: &CutProxy,
    session: InteractivePromptSession,
) {
    let mut peer = Peer::register(socket, "prompt-sender");

    // Quickstart 007 §1 : c'est exactement la session qui a reçu le prompt
    // réduit qui est visible et interrogée par l'outil MCP bridget_who.
    let initial = wait_agent(&mut peer, agent);
    assert_eq!(initial.agent_type, "codex");
    assert_eq!(initial.transport, "tmux");
    assert!(
        initial
            .channel
            .as_deref()
            .is_some_and(|value| value != "tmux")
    );
    assert_eq!(initial.state, "connected");

    // Quickstart 007 §2 et §3 : la même session répond à une demande
    // suivie, puis restitue un corps riche octet pour octet.
    let first = send_tracked_with_timeout(&mut peer, agent, "TRACKED", 9);
    assert_eq!(
        receive_replies(&mut peer, &[first]),
        vec!["fixture-response-1"]
    );
    let exact = "l'apostrophe d'usage, \"guillemets\", $VAR, `backticks`,\net ce saut de ligne.";
    let second = send_tracked_with_timeout(&mut peer, agent, exact, 9);
    assert_eq!(receive_replies(&mut peer, &[second]), vec![exact]);

    // Quickstart 007 §4 : le faux CLI reste vivant pendant le tour lent,
    // une seconde demande est livrée à la même session, puis la connexion du
    // wrapper est réellement coupée et reprise sans perdre l'ordre FIFO. Le
    // wrapper tmux historique n'a pas de frontière de tour : `busy` et les
    // relances différées restent donc réservés au corpus ACP ci-dessous.
    let slow = send_tracked_with_timeout(&mut peer, agent, "QUEUE-SLOW", 9);
    wait_path(
        &session.slow_started,
        "le même faux Codex n'a pas commencé le tour lent",
    );
    assert_eq!(wait_agent(&mut peer, agent).state, "connected");
    let next = send_tracked_with_timeout(&mut peer, agent, "QUEUE-NEXT", 9);
    proxy.cut_wrapper_and_wait_for_reconnect();
    let reconnected = wait_reconnected(&mut peer, agent);
    assert_eq!(reconnected.transport, "tmux");
    assert!(
        reconnected
            .channel
            .as_deref()
            .is_some_and(|value| value != "tmux")
    );
    assert_eq!(reconnected.state, "connected");
    assert_eq!(
        receive_replies(&mut peer, &[slow.clone(), next]),
        vec!["fixture-response-slow", "fixture-response-next"]
    );

    peer.send(&WrapperToDaemon::ListRequests {
        sender: peer.name.clone(),
        limit: 200,
    });
    match peer.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), MATRIX_EXPECTED_TURNS);
            let slow_request = requests
                .iter()
                .find(|request| request.id == slow)
                .expect("demande lente absente du ledger interactif");
            assert_eq!(slow_request.deferred_reminder_level, None);
            assert!(requests.iter().all(|request| request.state == "answered"));
        }
        other => panic!("liste des demandes du prompt inattendue: {other:?}"),
    }

    let processed = session.finish();
    assert_eq!(
        processed,
        vec!["TRACKED", exact, "QUEUE-SLOW", "QUEUE-NEXT"]
    );
}

fn normalized_entry(bytes: &[u8]) -> String {
    let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let event = value["event"].as_str().unwrap();
    let payload = &value["payload"];
    match event {
        "turn_start" => format!(
            "turn_start|reply={}|body={}",
            payload["reply"].as_bool().unwrap(),
            payload["body"].as_str().unwrap()
        ),
        "update" => format!("update|{}", payload["text"].as_str().unwrap_or_default()),
        "reasoning" => format!(
            "reasoning|available={}",
            payload["available"].as_bool().unwrap_or(false)
        ),
        "turn_end" => format!(
            "turn_end|{}|routed={}",
            payload["stop_reason"].as_str().unwrap_or_default(),
            payload.get("routed_to").is_some()
        ),
        other => format!("{other}|{payload}"),
    }
}

fn collect_journal(socket: &Path, agent: &str) -> Vec<String> {
    let mut attach = Peer::attach(socket);
    attach.send(&WrapperToDaemon::Subscribe {
        agent: agent.to_string(),
        window: AttachWindow::Seq(0),
    });
    let mut fragments: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
    let mut complete = BTreeMap::new();
    let mut caught_up = false;
    let deadline = Instant::now() + MATRIX_TIMEOUT;
    while (!caught_up || complete.len() < MATRIX_EXPECTED_TURNS * MATRIX_EVENTS_PER_TURN)
        && Instant::now() < deadline
    {
        match attach.recv() {
            DaemonToWrapper::Subscribed { .. } => {}
            DaemonToWrapper::JournalFragment {
                seq,
                offset,
                final_fragment,
                bytes,
                ..
            } => {
                let current = fragments.entry(seq).or_default();
                assert_eq!(
                    offset as usize,
                    current.len(),
                    "offset non contigu pour seq {seq}"
                );
                current.extend(bytes);
                if final_fragment {
                    complete.insert(seq, fragments.remove(&seq).unwrap());
                }
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => caught_up = true,
            DaemonToWrapper::Gap {
                from_seq, to_seq, ..
            } => {
                panic!("Gap inattendu dans le corpus de parité: {from_seq}..={to_seq}")
            }
            other => panic!("frame attach inattendue: {other:?}"),
        }
    }
    assert!(caught_up, "SnapshotCaughtUp absent");
    let bootstrap_message_ids = complete
        .values()
        .filter_map(|bytes| {
            let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
            (value["event"] == "turn_start" && value["payload"]["from"] == "bridget-reprise")
                .then(|| value["message_id"].as_str().map(str::to_string))
                .flatten()
        })
        .collect::<BTreeSet<_>>();
    let business: Vec<Vec<u8>> = complete
        .into_values()
        .filter(|bytes| {
            let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
            !value["message_id"]
                .as_str()
                .is_some_and(|message_id| bootstrap_message_ids.contains(message_id))
        })
        .collect();
    assert_eq!(
        business.len(),
        MATRIX_EXPECTED_TURNS * MATRIX_EVENTS_PER_TURN,
        "la fixture doit produire {MATRIX_EVENTS_PER_TURN} événements par tour métier (start+text+reasoning+end), hors carte de reprise"
    );
    // Preuve d'unicité : exactement un event=reasoning par message_id, available=false
    // (fixture parity sans thought_chunk — cas Gemini).
    let mut reasoning_per_message = BTreeMap::<String, usize>::new();
    for bytes in &business {
        let value: serde_json::Value = serde_json::from_slice(bytes).unwrap();
        if value["event"] == "reasoning" {
            let message_id = value["message_id"]
                .as_str()
                .expect("reasoning sans message_id")
                .to_string();
            *reasoning_per_message.entry(message_id).or_default() += 1;
            assert_eq!(
                value["payload"],
                serde_json::json!({ "available": false }),
                "reasoning terminal sans pensée = available:false, sans summary/raw"
            );
        }
    }
    assert_eq!(
        reasoning_per_message.len(),
        MATRIX_EXPECTED_TURNS,
        "un message_id métier sans reasoning terminal: {reasoning_per_message:?}"
    );
    assert!(
        reasoning_per_message.values().all(|count| *count == 1),
        "doublon event=reasoning sur un tour: {reasoning_per_message:?}"
    );
    business
        .iter()
        .map(|bytes| normalized_entry(bytes))
        .collect()
}

fn run_corpus(socket: &Path, agent: &str, run: usize, proxy: &CutProxy) -> ModeObservables {
    let mut peer = Peer::register(socket, &format!("parity-sender-{run}"));
    // Quickstart 007 §1 : l'équipier est visible en ACP, prêt à recevoir.
    // Attendre connected : la carte de reprise peut encore tourner (busy) —
    // L4 ajoute un event=reasoning qui allonge cette fenêtre.
    let initial_agent = wait_agent_state(&mut peer, agent, "connected");
    assert_eq!(initial_agent.transport, "acp");
    assert_eq!(initial_agent.state, "connected");

    // Quickstart 007 §2 : une demande suivie reçoit sa réponse et se clôt.
    let first =
        send_tracked_with_timeout(&mut peer, agent, "TRACKED", MATRIX_FAST_REPLY_TIMEOUT_SECS);
    let mut replies = receive_replies(&mut peer, &[first]);
    // Quickstart 007 §3 : le corps riche traverse le transport octet pour octet.
    let exact = "l'apostrophe d'usage, \"guillemets\", $VAR, `backticks`,\net ce saut de ligne.";
    let second = send_tracked_with_timeout(&mut peer, agent, exact, MATRIX_FAST_REPLY_TIMEOUT_SECS);
    replies.extend(receive_replies(&mut peer, &[second]));

    // Quickstart 007 §4 : FIFO pendant un tour, relance différée et
    // reconnexion conservant l'état busy.
    let slow = send_tracked_with_timeout(
        &mut peer,
        agent,
        "QUEUE-SLOW",
        MATRIX_SLOW_REPLY_TIMEOUT_SECS,
    );
    let next = send_tracked_with_timeout(
        &mut peer,
        agent,
        "QUEUE-NEXT",
        MATRIX_FAST_REPLY_TIMEOUT_SECS,
    );
    let busy = wait_agent(&mut peer, agent);
    assert_eq!(busy.state, "busy", "le tour lent doit être observable");
    proxy.cut_wrapper_and_wait_for_reconnect();
    let reconnected = wait_busy_reconnected(&mut peer, agent);
    assert_eq!(reconnected.transport, "acp");
    replies.extend(receive_replies(&mut peer, &[slow.clone(), next]));
    let final_agent = wait_agent_state(&mut peer, agent, "connected");
    assert_eq!(
        replies,
        vec![
            "fixture-response-1",
            "fixture-response-2",
            "fixture-response-3",
            "fixture-response-4"
        ]
    );

    peer.send(&WrapperToDaemon::ListRequests {
        sender: peer.name.clone(),
        limit: 200,
    });
    let request_states: Vec<String> = match peer.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), MATRIX_EXPECTED_TURNS);
            let slow_request = requests
                .iter()
                .find(|request| request.id == slow)
                .expect("demande lente absente du ledger");
            assert!(
                slow_request.deferred_reminder_level.is_some(),
                "la relance différée doit être consignée pendant le tour busy"
            );
            requests.into_iter().map(|request| request.state).collect()
        }
        other => panic!("liste de demandes inattendue: {other:?}"),
    };
    assert!(request_states.iter().all(|state| state == "answered"));

    let journal = collect_journal(socket, agent);
    let turn_starts = journal
        .iter()
        .filter(|entry| entry.starts_with("turn_start|"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        turn_starts,
        vec![
            "turn_start|reply=true|body=TRACKED".to_string(),
            format!("turn_start|reply=true|body={exact}"),
            "turn_start|reply=true|body=QUEUE-SLOW".to_string(),
            "turn_start|reply=true|body=QUEUE-NEXT".to_string(),
        ],
        "l'oracle de corps doit détecter une perte commune aux deux modes"
    );

    ModeObservables {
        replies,
        request_states,
        agent_transport: final_agent.transport,
        agent_state: final_agent.state,
        journal,
    }
}

fn stop_managed(control: &mut Peer, name: &str, run: usize) {
    control.send(&WrapperToDaemon::StopOrder {
        name: name.to_string(),
        command_id: format!("stop-parity-{run}"),
    });
    let response = control.recv();
    assert!(
        matches!(
            response,
            DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
                ..
            }
        ),
        "arrêt géré inattendu pour {name}: {response:?}"
    );
}

fn spawn_managed(control: &mut Peer, root: &Path, name: &str, command_id: &str, persistent: bool) {
    let now = unix_now();
    control.send(&WrapperToDaemon::SpawnOrder {
        agent_type: "parity".to_string(),
        name: Some(name.to_string()),
        cwd: root.to_string_lossy().into_owned(),
        persistent,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 10,
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::SpawnAccepted { name: accepted, .. } if accepted == name
    ));
}

fn wait_named_agents(socket: &Path, expected: &[String], absent: &[String]) {
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut control = Peer::register(socket, "persistence-observer");
    loop {
        control.send(&WrapperToDaemon::ListAgents);
        let agents = match control.recv() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("annuaire de persistance inattendu: {other:?}"),
        };
        let ready = expected.iter().all(|name| {
            agents
                .iter()
                .any(|agent| agent.name == *name && agent.state == "connected")
        });
        let excluded = absent
            .iter()
            .all(|name| agents.iter().all(|agent| agent.name != *name));
        if ready && excluded {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "annuaire persistant incomplet: attendu={expected:?}, absent={absent:?}, reçu={agents:?}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn process_group_members(pgid: u32) -> Vec<(u32, String)> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,pgid=,command="])
        .output()
        .expect("instantané ps");
    assert!(output.status.success(), "ps doit réussir");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let pid = fields.next()?.parse::<u32>().ok()?;
            let group = fields.next()?.parse::<u32>().ok()?;
            let command = fields.collect::<Vec<_>>().join(" ");
            (group == pgid).then_some((pid, command))
        })
        .collect()
}

fn assert_npx_descendant(pgid: u32) -> Vec<(u32, String)> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let members = process_group_members(pgid);
        let has_npx = members.iter().any(|(_, command)| {
            command.contains("npx")
                || command == "npm"
                || command.contains("npm exec")
                || command.contains("npm-cli.js exec")
        });
        if members.len() >= 2 && has_npx {
            return members;
        }
        assert!(
            Instant::now() < deadline,
            "le groupe {pgid} doit contenir le wrapper et son descendant npx: {members:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

fn wait_groups_gone(pgids: &[u32]) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let survivors = pgids
            .iter()
            .copied()
            .filter(|pgid| group_exists(*pgid).unwrap_or(false))
            .collect::<Vec<_>>();
        if survivors.is_empty() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "groupes encore vivants après arrêt: {survivors:?}"
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn marker_pgids(root: &Path, names: &[String]) -> Vec<u32> {
    let store = ManagedMarkerStore::for_home(root);
    names
        .iter()
        .map(|name| store.load(name).unwrap().pgid)
        .collect()
}

#[test]
fn prompt_reduit_rejoue_le_corpus_dans_la_meme_session() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(REDUCED_PROMPT.contains("reply=yes"));
    assert!(REDUCED_PROMPT.contains("reply=no"));
    assert!(!REDUCED_PROMPT.contains("bridget send"));

    let root = test_root("prompt-corpus");
    write_fixture(&root);
    let (daemon, proxy) = start_daemon_behind_proxy(&root);
    let prompt_name = "prompt-mcp";
    let prompt_session = start_interactive_prompt_session(&root, prompt_name, None, false);
    run_interactive_prompt_corpus(&daemon.socket, prompt_name, &proxy, prompt_session);
    daemon.stop();
    proxy.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_codex_rejoue_la_panne_mcp_et_clot_les_demandes_liees() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("resume-mcp");
    write_fixture(&root);
    let (daemon, proxy) = start_daemon_behind_proxy(&root);
    let name = "resume-mcp";
    let root_argument = root.to_string_lossy().into_owned();
    let resume_arguments = [
        "--yolo",
        "resume",
        "bridget-prospective",
        "--cd",
        root_argument.as_str(),
    ];
    let session = start_interactive_prompt_session(&root, name, Some(&resume_arguments), false);

    run_interactive_prompt_corpus(&daemon.socket, name, &proxy, session);

    daemon.stop();
    proxy.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_codex_sans_amorcage_ne_decouvre_pas_mcp() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("resume-mcp-mutation");
    write_fixture(&root);
    let (daemon, proxy) = start_daemon_behind_proxy(&root);
    let name = "resume-mcp-mutation";
    let root_argument = root.to_string_lossy().into_owned();
    let resume_arguments = [
        "--yolo",
        "resume",
        "bridget-prospective",
        "--cd",
        root_argument.as_str(),
    ];
    let session = start_interactive_prompt_session(&root, name, Some(&resume_arguments), true);
    let mut peer = Peer::register(&daemon.socket, "prompt-mutation-sender");
    wait_agent(&mut peer, name);

    let request_id = send_tracked_with_timeout(&mut peer, name, "MUTATION-NO-MCP", 30);
    peer.send(&WrapperToDaemon::ListRequests {
        sender: peer.name.clone(),
        limit: 20,
    });
    match peer.recv() {
        DaemonToWrapper::RequestList { requests } => {
            let request = requests
                .iter()
                .find(|request| request.id == request_id)
                .expect("demande mutée absente du ledger");
            assert_eq!(request.state, "open");
        }
        other => panic!("liste des demandes mutées inattendue: {other:?}"),
    }
    session.finish_without_mcp();

    daemon.stop();
    proxy.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn matrice_fr008_compare_le_meme_corpus_et_les_frames_attach() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert_eq!(MATRIX_VERSION, "fr-008-v1");
    assert_eq!(MATRIX_RUNS_PER_MODE, 3);

    for run in 0..MATRIX_RUNS_PER_MODE {
        let root = test_root(&format!("matrix-{run}"));
        let adapter = write_fixture(&root);
        let (daemon, proxy) = start_daemon_behind_proxy(&root);

        let terminal_name = format!("parity-terminal-{run}");
        let mut terminal = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args([
                "--",
                adapter.to_str().unwrap(),
                "--equipier",
                "--name",
                &terminal_name,
            ])
            .env_clear()
            .env("HOME", &root)
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp")
            .env("BRIDGET_CHANNEL", "unix")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let terminal_observables = run_corpus(&daemon.socket, &terminal_name, run * 2, &proxy);
        unsafe {
            libc::kill(terminal.id() as i32, libc::SIGTERM);
        }
        let _ = terminal.wait();

        let managed_name = format!("parity-managed-{run}");
        let mut control = Peer::register(&daemon.socket, &format!("spawn-client-{run}"));
        let now = unix_now();
        control.send(&WrapperToDaemon::SpawnOrder {
            agent_type: "parity".to_string(),
            name: Some(managed_name.clone()),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id: format!("spawn-parity-{run}"),
            issued_at: now,
            deadline_at: now + 8,
        });
        assert!(matches!(
            control.recv(),
            DaemonToWrapper::SpawnAccepted { ref name, .. } if name == &managed_name
        ));
        let managed_observables = run_corpus(&daemon.socket, &managed_name, run * 2 + 1, &proxy);

        assert_eq!(
            terminal_observables, managed_observables,
            "{MATRIX_VERSION}: divergence au run {run}"
        );
        stop_managed(&mut control, &managed_name, run);
        daemon.stop();
        proxy.stop();
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn matrice_fr008_compare_la_garde_de_facturation() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("billing");
    let adapter = write_fixture(&root);
    let daemon = DaemonProcess::start(&root, true, false);

    let terminal = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args([
            "--",
            adapter.to_str().unwrap(),
            "--equipier",
            "--name",
            "billing-terminal",
        ])
        .env_clear()
        .env("HOME", &root)
        .env("PATH", FROZEN_PATH)
        .env("USER", "parity-test")
        .env("LANG", "C")
        .env("TMPDIR", "/tmp")
        .env("BRIDGET_CHANNEL", "unix")
        .env("OPENAI_API_KEY", "forbidden-test-key")
        .output()
        .unwrap();
    assert!(!terminal.status.success());
    assert!(String::from_utf8_lossy(&terminal.stderr).contains("OPENAI_API_KEY"));

    let mut control = Peer::register(&daemon.socket, "billing-client");
    let now = unix_now();
    control.send(&WrapperToDaemon::SpawnOrder {
        agent_type: "parity".to_string(),
        name: Some("billing-managed".to_string()),
        cwd: root.to_string_lossy().into_owned(),
        persistent: false,
        command_id: "spawn-billing".to_string(),
        issued_at: now,
        deadline_at: now + 5,
    });
    assert!(matches!(
        control.recv(),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::BillingGuard { ref variable },
            ..
        } if variable == "OPENAI_API_KEY"
    ));

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sc001_vingt_spawns_survivent_a_la_fermeture_du_client_et_repondent() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    const SPAWNS: usize = 20;
    const SPAWN_P95_LIMIT: Duration = Duration::from_secs(10);
    const GLOBAL_BENCH_TIMEOUT: Duration = Duration::from_secs(120);

    let root = test_root("sc001");
    write_cached_npx_fixture(&root);
    let daemon = DaemonProcess::start(&root, false, true);
    let deadline = Instant::now() + GLOBAL_BENCH_TIMEOUT;
    let mut sender = Peer::register(&daemon.socket, "sc001-sender");
    let mut spawn_latencies = Vec::with_capacity(SPAWNS);

    for index in 0..SPAWNS {
        assert!(Instant::now() < deadline, "timeout global SC-001");
        let name = format!("sc001-agent-{index}");
        let command_id = format!("sc001-spawn-{index}");
        let now = unix_now();
        let started = Instant::now();
        let mut ordering_terminal =
            Peer::register(&daemon.socket, &format!("sc001-orderer-{index}"));
        ordering_terminal.send(&WrapperToDaemon::SpawnOrder {
            agent_type: "parity".to_string(),
            name: Some(name.clone()),
            cwd: root.to_string_lossy().into_owned(),
            persistent: false,
            command_id,
            issued_at: now,
            deadline_at: now + 10,
        });
        match ordering_terminal.recv() {
            DaemonToWrapper::SpawnAccepted { name: accepted, .. } => {
                assert_eq!(accepted, name)
            }
            other => panic!("spawn SC-001 {index} inattendu: {other:?}"),
        }
        spawn_latencies.push(started.elapsed());

        // Ce drop ferme réellement la socket du terminal donneur d'ordre.
        // L'échange suivant ne possède donc plus aucun lien avec ce client.
        drop(ordering_terminal);

        let request = send_tracked(&mut sender, &name, &format!("SC001-{index}"));
        assert_eq!(
            receive_replies(&mut sender, &[request]),
            vec!["fixture-response-1"]
        );
    }

    sender.send(&WrapperToDaemon::ListRequests {
        sender: sender.name.clone(),
        limit: 200,
    });
    match sender.recv() {
        DaemonToWrapper::RequestList { requests } => {
            assert_eq!(requests.len(), SPAWNS);
            assert!(requests.iter().all(|request| request.state == "answered"));
        }
        other => panic!("liste finale SC-001 inattendue: {other:?}"),
    }

    spawn_latencies.sort_unstable();
    let p95_index = (SPAWNS * 95).div_ceil(100).saturating_sub(1);
    let p95 = spawn_latencies[p95_index];
    let max = *spawn_latencies.last().unwrap();
    eprintln!(
        "SC-001 009: {SPAWNS}/{SPAWNS} spawns et échanges, p95={p95:?}, max={max:?}, total={:?}",
        GLOBAL_BENCH_TIMEOUT - deadline.saturating_duration_since(Instant::now())
    );
    assert!(p95 < SPAWN_P95_LIMIT, "p95 spawn SC-001={p95:?}");
    assert!(Instant::now() < deadline, "timeout global SC-001");

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sc005_sc006_persistance_arrets_cooperatifs_et_reconciliation_sigkill() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    const CYCLES: usize = 3;
    const FLEET_SIZE: usize = 3;
    const GLOBAL_TIMEOUT: Duration = Duration::from_secs(120);

    let root = test_root("persistence");
    write_cached_npx_fixture(&root);
    let persistent = (0..FLEET_SIZE)
        .map(|index| format!("persistent-{index}"))
        .collect::<Vec<_>>();
    let ephemeral = (0..FLEET_SIZE)
        .map(|index| format!("ephemeral-{index}"))
        .collect::<Vec<_>>();
    let deadline = Instant::now() + GLOBAL_TIMEOUT;

    let mut daemon = DaemonProcess::start(&root, false, false);
    let mut control = Peer::register(&daemon.socket, "persistence-orderer");
    for (index, name) in persistent.iter().enumerate() {
        spawn_managed(
            &mut control,
            &root,
            name,
            &format!("persistent-spawn-{index}"),
            true,
        );
    }
    for (index, name) in ephemeral.iter().enumerate() {
        spawn_managed(
            &mut control,
            &root,
            name,
            &format!("ephemeral-spawn-{index}"),
            false,
        );
    }
    wait_named_agents(&daemon.socket, &persistent, &[]);
    wait_named_agents(&daemon.socket, &ephemeral, &[]);

    let mut cooperative_pgids = Vec::new();
    for cycle in 0..CYCLES {
        assert!(Instant::now() < deadline, "timeout global SC-005/SC-006");
        let active_names = if cycle == 0 {
            persistent
                .iter()
                .chain(ephemeral.iter())
                .cloned()
                .collect::<Vec<_>>()
        } else {
            persistent.clone()
        };
        let pgids = marker_pgids(&root, &active_names);
        for pgid in &pgids {
            let _ = assert_npx_descendant(*pgid);
        }
        cooperative_pgids.push(pgids.clone());
        daemon.stop();
        wait_groups_gone(&pgids);

        daemon = DaemonProcess::start(&root, false, false);
        wait_named_agents(&daemon.socket, &persistent, &ephemeral);
    }

    let pre_crash_pgids = marker_pgids(&root, &persistent);
    daemon.kill();
    assert!(
        pre_crash_pgids
            .iter()
            .any(|pgid| group_exists(*pgid).unwrap_or(false)),
        "SIGKILL doit laisser au moins un groupe à réconcilier"
    );
    daemon = DaemonProcess::start(&root, false, false);
    wait_groups_gone(&pre_crash_pgids);
    wait_named_agents(&daemon.socket, &persistent, &ephemeral);
    let post_crash_pgids = marker_pgids(&root, &persistent);
    assert!(
        post_crash_pgids
            .iter()
            .all(|pgid| !pre_crash_pgids.contains(pgid)),
        "la reprise doit créer une génération distincte"
    );

    let mut control = Peer::register(&daemon.socket, "persistence-stopper");
    for (index, name) in persistent.iter().enumerate() {
        stop_managed(&mut control, name, 10_000 + index);
    }
    daemon.stop();
    daemon = DaemonProcess::start(&root, false, false);
    wait_named_agents(&daemon.socket, &[], &persistent);

    eprintln!(
        "SC-005/SC-006 009: cycles={CYCLES}, persistants={persistent:?}, éphémères={ephemeral:?}, pgid_coopératifs={cooperative_pgids:?}, pgid_avant_sigkill={pre_crash_pgids:?}, pgid_après_reprise={post_crash_pgids:?}"
    );
    assert!(Instant::now() < deadline, "timeout global SC-005/SC-006");
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

/// La branche forcée traverse la même garde que le happy-path. Elle verrouille
/// la fuite qui laissait des `bridget daemon` sous PID 1 après panique.
/// Compteur borné au HOME de CE test (pas au PID du harnais). Le verrou
/// sérialise seulement la contention CPU/IO avec les autres bancs lourds.
#[test]
fn daemon_process_nettoie_apres_une_panique_injectee() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root_slot = Mutex::new(None::<PathBuf>);
    let mid = AtomicUsize::new(usize::MAX);
    let failed = catch_unwind(AssertUnwindSafe(|| {
        let root = test_root("drop-oracle");
        fs::create_dir_all(&root).unwrap();
        *root_slot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(root.clone());
        // Compteur borné à CE HOME : un voisin parallèle ne peut pas le fausser.
        assert_eq!(
            daemon_count_for_home(&root),
            0,
            "le HOME du test doit être vide avant le spawn"
        );
        let _daemon = DaemonProcess::start(&root, false, false);
        // Premier temps : le spawn doit apparaître au compteur (sinon l'égalité
        // avant/après ne prouverait rien).
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut seen = daemon_count_for_home(&root);
        while seen != 1 && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(20));
            seen = daemon_count_for_home(&root);
        }
        mid.store(seen, Ordering::SeqCst);
        panic!("échec injecté après le spawn : la garde doit nettoyer");
    }));
    assert!(
        failed.is_err(),
        "la branche d'échec doit réellement paniquer"
    );
    assert_eq!(
        mid.load(Ordering::SeqCst),
        1,
        "premier temps : le daemon spawné doit être compté"
    );
    let root = root_slot
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
        .expect("HOME du test créé avant la panique");
    // Second temps : après Drop (dépliage), plus aucun daemon sur CE HOME.
    assert_daemon_count_for_home(&root, 0);
    let _ = fs::remove_dir_all(root);
}


const ABANDON_CAUSE_HARDCODED: &str =
    "abandon relance fournisseur après 2 tentatives (processus fournisseur terminé)";

fn provider_pids_in_group(pgid: u32) -> Vec<u32> {
    process_group_members(pgid)
        .into_iter()
        .filter_map(|(pid, command)| {
            let is_wrapper = command.contains("managed-wrapper");
            let is_provider = command.contains("parity-acp")
                || command.contains("python")
                || command.contains("npx")
                || command.contains("npm");
            if !is_wrapper && is_provider {
                Some(pid)
            } else {
                None
            }
        })
        .collect()
}

fn wrapper_pid_in_group(pgid: u32) -> Option<u32> {
    process_group_members(pgid)
        .into_iter()
        .find_map(|(pid, command)| command.contains("managed-wrapper").then_some(pid))
}

fn kill_provider_sigterm(pgid: u32) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let providers = provider_pids_in_group(pgid);
        if let Some(pid) = providers.into_iter().next() {
            assert_eq!(
                unsafe { libc::kill(pid as i32, libc::SIGTERM) },
                0,
                "SIGTERM fournisseur {pid}"
            );
            return pid;
        }
        assert!(
            Instant::now() < deadline,
            "aucun PID fournisseur dans le groupe {pgid}: {:?}",
            process_group_members(pgid)
        );
        thread::sleep(Duration::from_millis(25));
    }
}

/// Oracle : MEURT si un agent persistant joignable avant le kill ne revient pas
/// sans redémarrage du daemon. Prouve d'abord la joignabilité (sinon projection vide).
#[test]
fn TEMOIN_persistant_tue_redevient_joignable_sans_redemarrer_le_daemon() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("persist-relaunch");
    write_fixture(&root);
    // Borne courte pour le test, backoff minimal.
    let mut daemon = {
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut process = DaemonProcess {
            child: None,
            socket: root.join(".cache/bridget/bridget.sock"),
        };
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", &root)
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp/bt")
            .env("BRIDGET_PROVIDER_RELAUNCH_MAX", "5")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            UnixStream::connect(&process.socket).is_ok(),
            "daemon relance non prêt"
        );
        process
    };

    let name = "essai-persist-relaunch".to_string();
    let mut control = Peer::register(&daemon.socket, "persist-orderer");
    spawn_managed(&mut control, &root, &name, "persist-relaunch-1", true);
    wait_named_agents(&daemon.socket, &[name.clone()], &[]);

    // Preuve de joignabilité AVANT le kill — sinon l'oracle passe sur une projection vide.
    let mut sender = Peer::register(&daemon.socket, "persist-sender");
    let before = send_tracked(&mut sender, &name, "AVANT-KILL");
    assert_eq!(
        receive_replies(&mut sender, &[before]),
        vec!["fixture-response-1"],
        "joignable avant kill"
    );

    let pgids = marker_pgids(&root, &[name.clone()]);
    assert_eq!(pgids.len(), 1);
    let wrapper_before = wrapper_pid_in_group(pgids[0]).expect("wrapper avant kill");
    let killed = kill_provider_sigterm(pgids[0]);
    assert_ne!(killed, wrapper_before, "on tue le fournisseur, pas le wrapper");

    // Le wrapper doit survivre (propriété nommée par Maicie).
    let survive_deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < survive_deadline {
        if unsafe { libc::kill(wrapper_before as i32, 0) } != 0 {
            panic!("wrapper {wrapper_before} mort avec le fournisseur {killed}");
        }
        thread::sleep(Duration::from_millis(50));
    }

    wait_named_agents(&daemon.socket, &[name.clone()], &[]);
    let after = send_tracked(&mut sender, &name, "APRES-KILL");
    assert_eq!(
        receive_replies(&mut sender, &[after]),
        vec!["fixture-response-1"],
        "joignable après kill sans redémarrage daemon"
    );
    assert_eq!(
        unsafe { libc::kill(wrapper_before as i32, 0) },
        0,
        "même wrapper encore vivant après reprise"
    );

    stop_managed(&mut control, &name, 1);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

/// Oracle : MEURT si l'abandon après N tentatives n'est pas nommé en dur.
#[test]
fn TEMOIN_abandon_apres_N_tentatives_est_nomme() {
    let _serial = MANAGED_BENCH_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = test_root("persist-abandon");
    let adapter = write_fixture(&root);
    let mut daemon = {
        let binary = env!("CARGO_BIN_EXE_bridget");
        let mut process = DaemonProcess {
            child: None,
            socket: root.join(".cache/bridget/bridget.sock"),
        };
        let mut command = Command::new(binary);
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", &root)
            .env("PATH", FROZEN_PATH)
            .env("USER", "parity-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp/bt")
            .env("BRIDGET_PROVIDER_RELAUNCH_MAX", "2")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        process.child = Some(command.spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&process.socket).is_ok() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        process
    };

    let name = "essai-persist-abandon".to_string();
    let mut control = Peer::register(&daemon.socket, "abandon-orderer");
    spawn_managed(&mut control, &root, &name, "persist-abandon-1", true);
    wait_named_agents(&daemon.socket, &[name.clone()], &[]);

    let mut sender = Peer::register(&daemon.socket, "abandon-sender");
    let before = send_tracked(&mut sender, &name, "AVANT-ABANDON");
    assert_eq!(
        receive_replies(&mut sender, &[before]),
        vec!["fixture-response-1"]
    );

    let pgids = marker_pgids(&root, &[name.clone()]);
    // Remplacer le fournisseur par un binaire qui refuse de démarrer.
    fs::write(&adapter, "#!/bin/sh\necho refuse-auth >&2\nexit 1\n").unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let _ = kill_provider_sigterm(pgids[0]);

    // Le daemon conserve l'entrée en state=stopped après Unregister — ne pas
    // exiger la disparition du roster. La propriété mesurée : plus joignable.
    let gone_deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let mut observer = Peer::register(&daemon.socket, "abandon-observer");
        observer.send(&WrapperToDaemon::ListAgents);
        let agents = match observer.recv() {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("liste inattendue: {other:?}"),
        };
        let still_connected = agents
            .iter()
            .any(|agent| agent.name == name && agent.state == "connected");
        if !still_connected {
            break;
        }
        assert!(
            Instant::now() < gone_deadline,
            "agent encore connected après abandon attendu: {agents:?}"
        );
        thread::sleep(Duration::from_millis(100));
    }

    // Cause nommée EN DUR dans stderr géré.
    let stderr_root = root.join(".cache/bridget/managed-stderr");
    let mut found = false;
    let scan_deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < scan_deadline && !found {
        if stderr_root.exists() {
            for entry in walkdir_files(&stderr_root) {
                if let Ok(content) = fs::read_to_string(&entry) {
                    if content.contains(ABANDON_CAUSE_HARDCODED) {
                        found = true;
                        break;
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        found,
        "cause d'abandon absente des stderr gérés; attendu en dur: {ABANDON_CAUSE_HARDCODED}"
    );

    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

fn walkdir_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}
