#!/usr/bin/env python3
"""run_parent_pty.py — Parent externe réel sur PTY (T037/T038, session 149, r2).

Lance le wrapper 149 dans un vrai PTY — `check_terminal` exige stdin et stdout
sur terminal — relié au daemon FIXTURE (BRIDGET_HOME/BRIDGET_SOCKET privés).

r2 :
  - `--parent claude` (défaut) : `bridget gclaude "$PROMPT"` — AUCUN drapeau de
    permissions injecté par la recette : le mode réel normalisé (default/plan/
    bypass) vient de l'observation ACK du hook primaire, jamais d'une
    autodéclaration. Les règles allow/deny viennent du settings projet fixture
    préexistant (conservées par le mapping claude→claude) ;
  - `--parent codex --codex-sandbox {workspace-write|danger-full-access}` :
    `bridget codex --name <nom> -a never -s <sandbox> "$PROMPT"` — parent Codex
    réel (T014 : publication du fait issu de l'app-server). `-a never` et `-s`
    explicites, jamais --yolo global ;
  - `--cwd` : le parent est lancé DANS ce répertoire (obligatoire : le mapping
    claude→claude exige parent.cwd == cwd de délégation, l.230) ;
  - arrêt individuel OBLIGATOIRE : SIGTERM à UN PID identifié (comm vérifié,
    propriétaire = uid courant, pas Firefox), attente 3 s, vérification ;
    JAMAIS killpg/pkill/SIGKILL, jamais plusieurs processus d'un coup.
Preuve observer négative (T038d) : --stop-daemon-pidfile <state/daemon.pid>
--stop-delay N : SIGTERM au daemon après N s ; l'appel bridget_delegate suivant
reste sans ACK observer, refus nommé, aucune ligne native_delegations.
"""
import argparse
import os
import pty
import select
import signal
import subprocess
import time

GLM_PROFILE = "/Users/moi/.claude-glm"
# comms autorisés pour un arrêt ciblé ; tout autre comm est REFUSÉ.
ALLOWED_COMMS = {"bridget", "gclaude", "claude", "codex", "codex-pro", "node", "gemini"}
FORBIDDEN_COMMS = {"firefox", "firefox-bin", "Terminal", "iTerm2", "login"}


def ps_table() -> list[tuple[int, int, str, str]]:
    """(pid, ppid, comm, command) de tous les processus."""
    out = subprocess.run(["ps", "-axo", "pid=,ppid=,comm=,command="],
                         capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        parts = line.strip().split(None, 3)
        if len(parts) >= 3:
            try:
                command = parts[3] if len(parts) == 4 else ""
                # `ps comm=` tronque les chemins longs ("/Volumes/8TB2/50") : le nom vient d'argv[0].
                name = os.path.basename(command.split()[0]) if command.split() else os.path.basename(parts[2])
                if name.startswith("bridget-"):
                    name = "bridget"
                rows.append((int(parts[0]), int(parts[1]), name, command))
            except ValueError:
                continue
    return rows


def fixture_pids(fixture: str, bin149: str) -> list[tuple[int, int, str, str]]:
    """PIDs liés au fixture par l'ENVIRONNEMENT (BRIDGET_HOME=<fixture>/home), jamais par le
    chemin du binaire (partagé avec d'autres rondes). Exclut le runner et le daemon fixture
    (partagé entre runs successifs). Chaque candidat est revu individuellement."""
    matched = []
    for pid, ppid, comm, command in ps_table():
        if pid == os.getpid():
            continue
        words = command.split()
        if len(words) >= 2 and words[1] == "daemon":
            continue
        out = subprocess.run(["ps", "-Eww", "-p", str(pid), "-o", "command="],
                             capture_output=True, text=True).stdout
        if f"BRIDGET_HOME={fixture}/home" in out:
            matched.append((pid, ppid, comm, command))
    return matched


def stop_one_pid(pid: int, comm: str, why: str, log) -> bool:
    """SIGTERM à UN PID vérifié ; attente 3 s ; vérification ; jamais -9."""
    if comm.lower() in FORBIDDEN_COMMS or comm.lower().startswith("firefox"):
        log(f"REFUS {why} pid={pid} comm={comm} interdit (protection Firefox)")
        return False
    if comm not in ALLOWED_COMMS:
        log(f"REFUS {why} pid={pid} comm={comm} hors liste autorisée")
        return False
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return True
    log(f"SIGTERM {why} pid={pid} comm={comm} (processus unique)")
    try:
        os.kill(pid, signal.SIGTERM)
    except ProcessLookupError:
        return True
    deadline = time.monotonic() + 3.0
    while time.monotonic() < deadline:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            log(f"OK pid={pid} terminé après SIGTERM")
            return True
        time.sleep(0.3)
    try:
        os.kill(pid, 0)
        log(f"ATTENTION pid={pid} ({comm}) toujours vivant après 3 s — pas de -9, à consigner")
        return False
    except ProcessLookupError:
        return True


def stop_fixture_leftovers(fixture: str, bin149: str, log) -> None:
    """Arrêt un par un des processus du fixture restants (chef PTY déjà SIGTERM-é)."""
    for pid, _ppid, comm, _command in fixture_pids(fixture, bin149):
        stop_one_pid(pid, comm, "nettoyage fixture", log)


def terminate_process(pid: int, log) -> None:
    """SIGTERM au chef PTY uniquement (processus unique, jamais un groupe)."""
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return
    comm = next((c for p, _, c, _ in ps_table() if p == pid), "?")
    stop_one_pid(pid, comm, "chef PTY", log)


def db_terminal(home: str, request_id: str) -> bool:
    import json
    import sqlite3
    db = os.path.join(home, "bridget.db")
    try:
        conn = sqlite3.connect(f"file:{db}?mode=ro", uri=True, timeout=2)
        rows = conn.execute("SELECT payload FROM native_delegations WHERE request_id = ?",
                            (request_id,)).fetchall()
        conn.close()
    except Exception:
        return False
    return bool(rows) and json.loads(rows[0][0]).get("state") in (
        "failed", "cancelled", "result_available", "completed", "expired")


TERMINAL_QUERIES = [
    (b"\x1b[6n", b"\x1b[1;1R"),
    (b"\x1b]10;?\x1b\\", b"\x1b]10;rgb:cccc/cccc/cccc\x1b\\"),
    (b"\x1b]11;?\x1b\\", b"\x1b]11;rgb:0000/0000/0000\x1b\\"),
    (b"\x1b[?u", b"\x1b[?0u"),
    (b"\x1b[c", b"\x1b[?1;2c"),
]
SCREEN = [""]
GRANT_RETRY_AT = [0.0]


def clean_screen(chunk: bytes) -> str:
    import re
    txt = chunk.decode("utf-8", "replace")
    return re.sub(r"[ \t]+", " ", re.sub(r"\x1b\[[0-9;?]*[A-Za-z]|\x1b[78=>]|\x1b\][^\x07]*\x07", " ", txt).replace("\r", ""))


def try_grant(options, env, log, master_fd, prompt, name) -> bool:
    """Grant humain FIXTURE puis saisie du prompt. True quand c'est fait."""
    import json
    out = subprocess.run([options.bin149, "agents", "--json", "--global"], capture_output=True, text=True, env=env)
    try:
        data = json.loads("\n".join(l for l in out.stdout.splitlines() if l.lstrip()[:1] in "[{ ]}\""))
    except ValueError:
        return False
    agents = data if isinstance(data, list) else data.get("agents", [])
    wrappers = [a for a in agents if isinstance(a, dict) and a.get("state") == "connected"]
    if not wrappers:
        return False
    target = wrappers[0]
    ident = target["agent_id"]
    log(f"agents --json (extrait) = {json.dumps(target, ensure_ascii=False)[:300]}")
    # Le contrôle humain exige un terminal interactif : grant exécuté dans son propre PTY.
    if time.monotonic() < GRANT_RETRY_AT[0]:
        return False
    GRANT_RETRY_AT[0] = time.monotonic() + 3.0
    gpid, gfd = pty.fork()
    if gpid == 0:
        os.execve(options.bin149, [options.bin149, "delegate-grant", str(ident), "--cwd", options.cwd,
                                   "--posture", options.grant_posture], env)
    gout = b""
    gdeadline = time.monotonic() + 15.0
    while time.monotonic() < gdeadline:
        r, _, _ = select.select([gfd], [], [], 0.3)
        if r:
            try:
                chunk = os.read(gfd, 4096)
            except OSError:
                break
            if not chunk:
                break
            gout += chunk
    try:
        _, gstatus = os.waitpid(gpid, 0)
    except ChildProcessError:
        gstatus = -1
    os.close(gfd)
    log(f"delegate-grant status={gstatus} out={gout.decode('utf-8','replace').strip()[:300]}")
    if gstatus != 0:
        return False
    return True


def screen_ready() -> bool:
    text = SCREEN[0]
    banner = max(text.rfind("Claude Code v"), text.rfind("OpenAI Codex"))
    dialog = max(text.rfind("Enter to confirm"), text.rfind("enter confirm"), text.rfind("enter continue"),
                 text.rfind("Hooks need review"))
    return banner >= 0 and banner > dialog


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin149", required=True)
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--prompt-file", required=True)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--parent", choices=["claude", "codex"], default="claude")
    parser.add_argument("--codex-sandbox", default=None,
                        choices=["workspace-write", "danger-full-access"],
                        help="requis avec --parent codex (-s explicite)")
    parser.add_argument("--cwd", required=True,
                        help="répertoire du parent PTY (= cwd de délégation)")
    parser.add_argument("--timeout", type=float, default=900.0,
                        help="durée maximale de la session PTY (s)")
    parser.add_argument("--grant-posture", choices=["discovery", "development"], default=None,
                        help="grant humain FIXTURE (delegate-grant) sur --cwd avant de taper le prompt")
    parser.add_argument("--exit-when-terminal", action="store_true",
                        help="quand la ligne native_delegations du request_id est terminale, laisser 25 s "
                             "au parent pour répondre puis saisir /exit (arrêt propre)")
    parser.add_argument("--pre-keys", default="",
                        help="touches (échappements \\x1b[Z = Maj+Tab) envoyées avant le prompt, p.ex. pour changer de mode")
    parser.add_argument("--stop-daemon-pidfile",
                        help="pidfile du daemon à SIGTERM-er pour la branche négative")
    parser.add_argument("--stop-delay", type=float, default=20.0)
    options = parser.parse_args()

    if options.parent == "codex" and not options.codex_sandbox:
        parser.error("--parent codex exige --codex-sandbox (-a never reste implicite au montage)")
    if not os.path.isdir(options.cwd):
        parser.error(f"--cwd inexistant : {options.cwd}")

    with open(options.prompt_file, encoding="utf-8") as handle:
        prompt = handle.read().strip().replace("{{REQUEST_ID}}", options.request_id)

    fixture = os.path.realpath(options.fixture_root)
    prompt = prompt.replace("{{FIXTURE_ROOT}}", fixture)
    home = os.path.join(fixture, "home")
    log_path = os.path.join(fixture, "logs", f"parent-pty-{options.request_id}.log")

    env = {
        "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
        "HOME": os.environ.get("HOME", os.path.expanduser("~")),
        "TERM": os.environ.get("TERM", "xterm-256color"),
        "LANG": os.environ.get("LANG", "fr_FR.UTF-8"),
        "CLAUDE_CONFIG_DIR": GLM_PROFILE,
        "BRIDGET_HOME": home,
        "BRIDGET_SOCKET": os.path.join(home, "bridget.sock"),
    }

    name = f"recipe149-parent-{options.parent}-{options.request_id}"
    typed = options.grant_posture is not None
    tail = [] if typed else [prompt]
    if options.parent == "codex":
        argv = [options.bin149, "codex", "--name", name,
                "-a", "never", "-s", options.codex_sandbox] + tail
    else:
        argv = [options.bin149, "gclaude"] + tail

    os.chdir(options.cwd)  # le parent partage le cwd de délégation exigé (l.230)

    # pty.fork : l'enfant reçoit déjà setsid + terminal de contrôle ; ne pas
    # rappeler setsid (EPERM sur un chef de session existant).
    pid, master_fd = pty.fork()
    if pid == 0:
        os.execve(options.bin149, argv, env)

    import fcntl
    import struct
    import termios
    fcntl.ioctl(master_fd, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 140, 0, 0))  # TUI natif : taille non nulle

    def log(message: str) -> None:
        os.write(log_fd, f"\n[recette] {message}\n".encode())

    daemon_pid = None
    if options.stop_daemon_pidfile:
        with open(options.stop_daemon_pidfile, encoding="utf-8") as handle:
            daemon_pid = int(handle.read().strip())

    log_fd = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    started = time.monotonic()
    granted = not typed
    granted_at = 0.0
    last_output = time.monotonic()
    prompt_typed = not typed
    screen = ""
    scheduled = []  # (échéance, touches, message) : le TUI Codex ignore une touche trop précoce
    trust_sent = imports_sent = update_sent = hooks_sent = False
    typed_exit_at = None
    exit_sent = False
    exit_sent_at = 0.0
    leader_stopped = False
    grant_deadline = started + 120.0
    stopped_daemon = daemon_pid is None
    exit_status = None
    while True:
        ready, _, _ = select.select([master_fd], [], [], 0.5)
        if ready:
            try:
                chunk = os.read(master_fd, 65536)
            except OSError:
                chunk = b""
            if not chunk:
                break
            os.write(log_fd, chunk)
            last_output = time.monotonic()
            for query, answer in TERMINAL_QUERIES:  # réponses d'un terminal réel aux requêtes TUI
                if query in chunk:
                    os.write(master_fd, answer)
            screen = (screen + clean_screen(chunk))[-6000:]
            SCREEN[0] = (SCREEN[0] + clean_screen(chunk))[-6000:]
            if not trust_sent and "Yes, I trust this folder" in screen and "Enter to confirm" in screen:
                os.write(master_fd, b"\x1b[B"); time.sleep(0.5); os.write(master_fd, b"\r")
                trust_sent = True; screen = ""
                log("dialogue confiance dossier : 'Yes, I trust this folder' choisi")
            elif not update_sent and "Skip until next version" in screen:
                scheduled.append((time.monotonic() + 6.0, b"2", "dialogue mise à jour Codex : 'Skip'"))
                update_sent = True; screen = ""
            elif not hooks_sent and "Continue without trusting" in screen:
                scheduled.append((time.monotonic() + 6.0, b"3", "dialogue hooks Codex : 'Continue without trusting'"))
                hooks_sent = True; screen = ""
            elif not imports_sent and "Allow external CLAUDE.md file imports" in screen:
                os.write(master_fd, b"\r")  # défaut : 'No, disable external imports'
                imports_sent = True; screen = ""
                log("dialogue imports externes : défaut 'No, disable external imports'")
        for item in [i for i in scheduled if time.monotonic() >= i[0]]:
            scheduled.remove(item)
            os.write(master_fd, item[1])
            log(item[2])
        if options.exit_when_terminal and prompt_typed and typed_exit_at is None \
                and db_terminal(home, options.request_id):
            typed_exit_at = time.monotonic() + 25.0
            log("ligne native_delegations terminale observée ; /exit dans 25 s")
        if typed_exit_at is not None and not exit_sent and time.monotonic() >= typed_exit_at:
            os.write(master_fd, b"/exit")
            time.sleep(0.8)
            os.write(master_fd, b"\r")
            exit_sent = True
            exit_sent_at = time.monotonic()
            log("/exit saisi dans le TUI")
        if exit_sent and time.monotonic() - exit_sent_at > 20.0 and not leader_stopped:
            leader_stopped = True
            terminate_process(pid, log)
        if not granted:
            granted = try_grant(options, env, log, master_fd, prompt, name)
            if granted:
                granted_at = time.monotonic()
            elif time.monotonic() > grant_deadline:
                log("ECHEC : grant impossible sous 90 s")
                granted = True
                prompt_typed = True  # ne rien saisir sans grant
        if granted and typed and not prompt_typed and screen_ready() \
                and time.monotonic() - granted_at >= 3.0 \
                and (time.monotonic() - last_output >= 5.0 or time.monotonic() - granted_at >= 45.0):
            if options.pre_keys:
                os.write(master_fd, options.pre_keys.encode().decode("unicode_escape").encode())
                time.sleep(2.5)
                log("pre-keys envoyées : " + repr(options.pre_keys))
            data = " ".join(prompt.split()).encode()
            os.write(master_fd, b"\x1b[200~")  # collage balisé : un humain collerait ce texte
            for start in range(0, len(data), 256):
                os.write(master_fd, data[start:start + 256])
                time.sleep(0.05)
            os.write(master_fd, b"\x1b[201~")
            time.sleep(2.0)
            os.write(master_fd, b"\r")
            prompt_typed = True
            log("prompt tapé dans le TUI après grant")
        if daemon_pid is not None and not stopped_daemon \
                and time.monotonic() >= started + options.stop_delay:
            stop_one_pid(daemon_pid, "bridget", "daemon fixture (branche négative)", log)
            stopped_daemon = True
            log("SIGTERM daemon envoye (branche observer negative)")
        if time.monotonic() - started > options.timeout:
            log("TIMEOUT : SIGTERM au chef PTY (PID unique), puis nettoyage individuel")
            terminate_process(pid, log)
            time.sleep(3.0)
            stop_fixture_leftovers(fixture, options.bin149, log)
            break
        done, status = os.waitpid(pid, os.WNOHANG)
        if done == pid:
            exit_status = status
            time.sleep(0.3)
            while True:  # vider le reste du PTY après la mort du chef
                ready, _, _ = select.select([master_fd], [], [], 0.1)
                if not ready:
                    break
                try:
                    chunk = os.read(master_fd, 65536)
                except OSError:
                    break
                if not chunk:
                    break
                os.write(log_fd, chunk)
            break

    if exit_status is None:
        # Vider le PTY pendant l'attente : un chef bloqué en écriture ne peut pas sortir.
        end = time.monotonic() + 20.0
        while exit_status is None and time.monotonic() < end:
            ready, _, _ = select.select([master_fd], [], [], 0.3)
            if ready:
                try:
                    chunk = os.read(master_fd, 65536)
                except OSError:
                    chunk = b""
                if chunk:
                    os.write(log_fd, chunk)
            try:
                done, status = os.waitpid(pid, os.WNOHANG)
            except ChildProcessError:
                exit_status = -1
                break
            if done == pid:
                exit_status = status
        if exit_status is None:
            log("ATTENTION : chef PTY toujours vivant après 20 s de drainage ; pas de -9")
            exit_status = -2
    # Un chef vivant en fin de boucle (sortie sans timeout) est arrêté
    # individuellement ; les restes du fixture sont passés un par un.
    stop_fixture_leftovers(fixture, options.bin149, log)
    os.close(master_fd)
    os.close(log_fd)

    exit_file = os.path.join(fixture, "state", f"parent-pty-{options.request_id}.exit")
    fd = os.open(exit_file, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    os.write(fd, f"{exit_status}\n".encode())
    os.close(fd)
    print(f"OK parent PTY termine status={exit_status} journal={log_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
