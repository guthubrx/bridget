#!/usr/bin/env python3
"""r7_run.py - ronde r7 (session 149) : parent Codex PTY vivant pendant SIGTERM + relance du daemon.

Orchestre, dans une fixture privée neuve et SANS T3, les scripts r6 réutilisés tels quels :
  - r6_daemon.py start|stop      (daemon fixture, environnement minimal, SIGTERM vérifié)
  - run_parent_pty.py            (vrai parent Codex gpt-6.1-sol high, -a never -s workspace-write)
  - r6_lifecycle.py              (>= 3 écritures, photographie, SIGTERM du SEUL daemon, relance)
et ajoute un observateur en lecture seule (r7_timeline.jsonl) qui consigne, avec l'heure en
millisecondes : alias Codex hors home (/private/tmp/bridget-codex-<hex>/s.sock), liens c-*.sock
sous home, processus de la fixture (PID + date de début), battements, état de la ligne en base.

Aucun lien supprimé à la main, aucun kill -9, aucun groupe, aucun pkill. Les environnements des
processus ne sont jamais enregistrés (seule la ligne de commande tronquée l'est).
Usage : r7_run.py --fixture-root FX --bin149 BIN --request-id ID
"""
import argparse
import glob
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import sys
import threading
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from r6_lifecycle import db_rows, line_count  # noqa: E402

ALIAS_GLOB = "/private/tmp/bridget-codex-*"
ALIAS_RE = re.compile(r"^/private/tmp/bridget-codex-[0-9a-f]{32}$")
PS_RE = re.compile(r"^\s*(\d+)\s+(\d+)\s+(\w{3}\s+\w{3}\s+\d+\s+[\d:]{8}\s+\d{4})\s+(.*)$")
WORKTREE = os.path.realpath(os.path.join(HERE, "../../../.."))
FINGERPRINT = os.path.join(WORKTREE, "specs/149-sous-agents-lineage/validation/native-r5-fingerprint.py")


def sha(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def now_ms() -> str:
    t = time.time()
    return time.strftime("%H:%M:%S", time.localtime(t)) + f".{int((t % 1) * 1000):03d}"


def log(message: str) -> None:
    print(f"[{now_ms()}] {message}", flush=True)


def aliases() -> dict:
    found = {}
    for path in sorted(glob.glob(ALIAS_GLOB)):
        if not ALIAS_RE.match(path):
            continue  # dossiers « bridget-codex-native-… » des tests : étrangers
        sock = os.path.join(path, "s.sock")
        try:
            meta = os.lstat(sock)
            kind = "lien" if os.path.islink(sock) else "socket" if (meta.st_mode & 0o170000) == 0o140000 else "autre"
            target = os.readlink(sock) if kind == "lien" else None
        except FileNotFoundError:
            kind, target = "absent", None
        found[path] = {"s.sock": kind, "cible": target, "mode_dossier": oct(os.lstat(path).st_mode & 0o777)}
    return found


def home_links(home: str) -> list:
    return sorted(name for name in os.listdir(home) if os.path.islink(os.path.join(home, name)))


def fixture_procs(home: str) -> dict:
    """{(pid, lstart): {ppid, kind, cmd}} des processus portant BRIDGET_HOME=<home> (env lu, jamais écrit)."""
    out = subprocess.run(["ps", "-Eww", "-axo", "pid=,ppid=,lstart=,command="],
                         capture_output=True, text=True).stdout
    found = {}
    marker = f"BRIDGET_HOME={home}"
    for line in out.splitlines():
        match = PS_RE.match(line)
        if not match or marker not in line:
            continue
        pid, ppid, lstart, command = match.groups()
        cmd = command.split(" ")[0:14]
        head = " ".join(cmd)[:130]
        words = command.split()
        if len(words) > 1 and words[1] == "daemon":
            kind = "daemon"
        elif "heartbeat.txt" in head:
            kind = "commande-battement"
        elif "app-server" in head:
            kind = "codex-app-server"
        elif len(words) > 2 and words[1] == "codex" and "--name" in words:
            kind = "parent-wrapper"
        elif "managed-wrapper" in head:
            kind = "enfant-wrapper"
        elif os.path.basename(words[0]) == "codex":
            kind = "codex-tui"
        else:
            kind = "autre"
        found[(int(pid), " ".join(lstart.split()))] = {"ppid": int(ppid), "kind": kind, "cmd": head}
    return found


class Observer(threading.Thread):
    def __init__(self, fixture: str, request_id: str, out_path: str):
        super().__init__(daemon=True)
        self.fixture, self.home, self.request_id = fixture, os.path.join(fixture, "home"), request_id
        self.heartbeat = os.path.join(fixture, "project", "parent-proj", "allowed", "heartbeat.txt")
        self.out = open(os.open(out_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w")
        self.stop = threading.Event()
        self.events = []

    def emit(self, ev: str, **data) -> None:
        event = {"t": round(time.time(), 3), "h": now_ms(), "ev": ev, **data}
        self.events.append(event)
        self.out.write(json.dumps(event, ensure_ascii=False) + "\n")
        self.out.flush()

    def state(self):
        try:
            rows = db_rows(self.home, self.request_id)
        except Exception:  # noqa: BLE001 - base verrouillée ou absente : pas de constat
            return "illisible"
        if not rows:
            return None
        payload = json.loads(rows[0][1])
        return f"{payload.get('state')}/{payload.get('error')}"

    def run(self) -> None:
        procs, seen_alias, seen_links, beats, state = {}, {}, None, 0, "init"
        while not self.stop.is_set():
            current = fixture_procs(self.home)
            for key in current.keys() - procs.keys():
                self.emit("proc+", pid=key[0], lstart=key[1], **current[key])
            for key in procs.keys() - current.keys():
                self.emit("proc-", pid=key[0], lstart=key[1], kind=procs[key]["kind"])
            procs = current
            alias_now = aliases()
            for path in alias_now.keys() - seen_alias.keys():
                self.emit("alias+", dossier=path, **alias_now[path])
            for path in seen_alias.keys() - alias_now.keys():
                self.emit("alias-", dossier=path)
            for path in alias_now.keys() & seen_alias.keys():
                if alias_now[path] != seen_alias[path]:
                    self.emit("alias~", dossier=path, **alias_now[path])
            seen_alias = alias_now
            links = home_links(self.home)
            if links != seen_links:
                self.emit("home-liens", liens=links)
                seen_links = links
            count = line_count(self.heartbeat)
            if count != beats:
                self.emit("battement", lignes=count)
                beats = count
            new_state = self.state()
            if new_state != state:
                self.emit("etat-base", etat=new_state)
                state = new_state
            self.stop.wait(0.4)
        self.out.close()


def grants(home: str) -> dict:
    conn = sqlite3.connect(f"file:{os.path.join(home, 'bridget.db')}?mode=ro", uri=True, timeout=3)
    try:
        return {table: conn.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
                for table in ("native_delegation_grants", "native_delegation_revocations",
                              "native_delegation_cancel_receipts", "native_delegations")}
    finally:
        conn.close()


def inputs(bin149: str) -> dict:
    codex_real = os.path.realpath("/opt/homebrew/bin/codex")
    prod = json.loads(subprocess.run([sys.executable, "-I", FINGERPRINT], capture_output=True, text=True).stdout)["prod"]
    return {"bin149_sha256": sha(bin149),
            "codex_cli": codex_real, "codex_cli_sha256": sha(codex_real),
            "codex_pro_launcher_sha256": sha("/Users/moi/.local/bin/codex-pro"),
            "gclaude_sha256": sha("/Users/moi/.local/bin/gclaude"),
            "codex_config_sha256": sha("/Users/moi/.codex/config.toml"),
            "prod_source_fingerprint": prod["value"], "prod_source_files": prod["fileCount"]}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--bin149", required=True)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--pty-timeout", type=float, default=900.0)
    options = parser.parse_args()
    os.umask(0o077)
    fixture = os.path.realpath(options.fixture_root)
    home = os.path.join(fixture, "home")
    state_dir = os.path.join(fixture, "state")
    cwd = os.path.join(fixture, "project", "parent-proj")
    rid = options.request_id
    report = {"request_id": rid, "fixture": fixture, "debut": time.time()}
    out_path = os.path.join(state_dir, "r7-run.json")

    def save() -> None:
        with open(os.open(out_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as handle:
            json.dump(report, handle, indent=1, ensure_ascii=False)

    for var in ("BRIDGET_T3_MCP_ENDPOINT", "BRIDGET_T3_MCP_AUTHORIZATION"):
        if os.environ.get(var):
            log(f"REFUS : {var} présent")
            return 2
    report["entrees_debut"] = inputs(options.bin149)
    report["alias_preexistants_etrangers"] = sorted(aliases())
    log(f"entrées au début : {report['entrees_debut']}")

    code = subprocess.run([sys.executable, "-I", os.path.join(HERE, "r6_daemon.py"), "start",
                           "--fixture-root", fixture, "--bin149", options.bin149]).returncode
    if code != 0:
        report["verdict"] = "BLOQUE: daemon fixture non démarré"
        save()
        return 1
    with open(os.path.join(state_dir, "daemon.pid"), encoding="utf-8") as handle:
        report["daemon1_pid"] = int(handle.read().strip())
    report["grants_BEFORE"] = grants(home)
    log(f"grants BEFORE (lus directement) : {report['grants_BEFORE']}")

    observer = Observer(fixture, rid, os.path.join(state_dir, "r7-timeline.jsonl"))
    observer.start()
    prompt = os.path.join(HERE, "prompts", "r6_parent_codex_lifecycle.md")
    pty_log = open(os.path.join(fixture, "logs", "r7-pty-runner.out"), "w")
    t_launch = time.time()
    pty = subprocess.Popen([sys.executable, "-I", os.path.join(HERE, "run_parent_pty.py"),
                            "--bin149", options.bin149, "--fixture-root", fixture, "--prompt-file", prompt,
                            "--request-id", rid, "--parent", "codex", "--codex-sandbox", "workspace-write",
                            "--cwd", cwd, "--type-prompt", "--exit-when-terminal",
                            "--timeout", str(options.pty_timeout)],
                           stdout=pty_log, stderr=subprocess.STDOUT, env={
                               "PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": os.environ["HOME"],
                               "LANG": os.environ.get("LANG", "fr_FR.UTF-8"), "TERM": "xterm-256color"})
    log(f"parent PTY lancé (runner pid {pty.pid})")
    life_log = open(os.path.join(fixture, "logs", "r7-lifecycle.out"), "w")
    life = subprocess.Popen([sys.executable, "-I", os.path.join(HERE, "r6_lifecycle.py"),
                             "--fixture-root", fixture, "--bin149", options.bin149, "--request-id", rid],
                            stdout=life_log, stderr=subprocess.STDOUT, env={
                                "PATH": os.environ.get("PATH", "/usr/bin:/bin"), "HOME": os.environ["HOME"]})
    life_code = life.wait()
    report["lifecycle_code"] = life_code
    log(f"r6_lifecycle terminé code={life_code} (+{time.time() - t_launch:.0f} s)")
    pty_code = pty.wait()
    report["pty_runner_code"] = pty_code
    report["duree_session_pty_s"] = round(time.time() - t_launch, 1)
    t_pty_end = time.time()
    log(f"runner PTY terminé code={pty_code}")

    # le parent terminé : l'alias et son dossier doivent disparaître seuls
    deadline = time.time() + 20
    while time.time() < deadline and any(a not in report["alias_preexistants_etrangers"] for a in aliases()):
        time.sleep(0.4)
    observer.stop.set()
    observer.join()
    report["alias_apres_parent_termine"] = sorted(a for a in aliases() if a not in report["alias_preexistants_etrangers"])
    report["alias_disparus_s_apres_runner"] = round(time.time() - t_pty_end, 1)
    report["home_liens_fin"] = home_links(home)
    report["home_contenu_fin"] = sorted(os.listdir(home))
    left = fixture_procs(home)
    report["processus_fixture_restants_avant_arret_daemon"] = [
        {"pid": k[0], "kind": v["kind"], "cmd": v["cmd"]} for k, v in left.items()]
    code = subprocess.run([sys.executable, "-I", os.path.join(HERE, "r6_daemon.py"), "stop",
                           "--fixture-root", fixture, "--bin149", options.bin149]).returncode
    report["daemon_final_stop_code"] = code
    report["grants_AFTER"] = grants(home)
    report["entrees_fin"] = inputs(options.bin149)
    report["processus_fixture_restants_final"] = [
        {"pid": k[0], "kind": v["kind"]} for k, v in fixture_procs(home).items()]
    report["fin"] = time.time()
    report["evenements"] = len(observer.events)
    save()
    log(f"rapport : {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
