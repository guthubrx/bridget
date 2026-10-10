#!/usr/bin/env python3
"""r6_lifecycle.py - scénario B du smoke r6 (T038, cycle de vie du daemon), session 149.

Pendant qu'un parent Codex PTY réel (lancé à part par run_parent_pty.py) a délégué à un enfant
Codex réel qui écrit un battement toutes les 3 s :
  1. attend >= 3 écritures PROUVÉES dans allowed/heartbeat.txt ;
  2. photographie l'arbre des descendants du daemon fixture (PID + date de début) ;
  3. envoie SIGTERM au SEUL daemon fixture (PID vérifié) - jamais le parent PTY ni le wrapper ;
  4. mesure la mort de chaque PID photographié, l'arrêt du fichier, les restes ;
  5. relance un daemon fixture privé et observe : tâche failed/unreachable, aucun nouveau
     processus, aucune nouvelle écriture, une seule ligne en base.
Aucun kill -9, aucun groupe, aucun pkill. Lecture SQLite en mode ro. Journal JSON dans
<fixture>/state/lifecycle-<request_id>.json.
"""
import argparse
import json
import os
import re
import signal
import sqlite3
import subprocess
import sys
import time

ROW = re.compile(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(\w{3}\s+\w{3}\s+\d+\s+[\d:]{8}\s+\d{4})\s+(.*)$")
HERE = os.path.dirname(os.path.abspath(__file__))


def log(message: str) -> None:
    print(f"[{time.strftime('%H:%M:%S')}] {message}", flush=True)


def ps_rows() -> dict:
    out = subprocess.run(["ps", "-axo", "pid=,ppid=,pgid=,lstart=,command="],
                         capture_output=True, text=True).stdout
    rows = {}
    for line in out.splitlines():
        match = ROW.match(line)
        if match:
            pid, ppid, pgid, lstart, command = match.groups()
            rows[int(pid)] = {"pid": int(pid), "ppid": int(ppid), "pgid": int(pgid),
                              "lstart": " ".join(lstart.split()), "command": command}
    return rows


def descendants(rows: dict, root: int) -> list:
    found, frontier = [], [root]
    while frontier:
        current = frontier.pop()
        for row in rows.values():
            if row["ppid"] == current and row["pid"] not in [f["pid"] for f in found]:
                found.append(row)
                frontier.append(row["pid"])
    return found


def kind_of(command: str) -> str:
    if "heartbeat.txt" in command:
        return "commande-battement"
    if "app-server" in command and "codex" in command:
        return "fournisseur-codex-app-server"
    if "bridget" in command:
        return "wrapper-bridget"
    return "autre"


def line_count(path: str) -> int:
    try:
        with open(path, "rb") as handle:
            return handle.read().count(b"\n")
    except FileNotFoundError:
        return 0


def db_rows(home: str, request_id: str):
    conn = sqlite3.connect(f"file:{os.path.join(home, 'bridget.db')}?mode=ro", uri=True, timeout=3)
    try:
        return conn.execute("SELECT task_id, payload FROM native_delegations WHERE request_id = ?",
                            (request_id,)).fetchall()
    finally:
        conn.close()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--bin149", required=True)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--wait-writes", type=int, default=3)
    parser.add_argument("--delegation-timeout", type=float, default=420.0)
    options = parser.parse_args()

    fixture = os.path.realpath(options.fixture_root)
    home = os.path.join(fixture, "home")
    heartbeat = os.path.join(fixture, "project", "parent-proj", "allowed", "heartbeat.txt")
    pidfile = os.path.join(fixture, "state", "daemon.pid")
    report = {"request_id": options.request_id, "fixture": fixture}
    out_path = os.path.join(fixture, "state", f"lifecycle-{options.request_id}.json")

    def save() -> None:
        fd = os.open(out_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w") as handle:
            json.dump(report, handle, indent=1, ensure_ascii=False)

    with open(pidfile, encoding="utf-8") as handle:
        daemon_pid = int(handle.read().strip())

    # 1. attente de >= N écritures prouvées
    t_start = time.monotonic()
    increments = []  # (secondes depuis début, nombre de lignes)
    seen = 0
    log(f"attente de {options.wait_writes} écritures dans {heartbeat}")
    while True:
        count = line_count(heartbeat)
        if count != seen:
            seen = count
            increments.append((round(time.monotonic() - t_start, 2), count))
        if count >= options.wait_writes:
            break
        if time.monotonic() - t_start > options.delegation_timeout:
            report["verdict"] = "BLOQUE: aucune écriture suffisante avant le délai"
            report["increments_avant_sigterm"] = increments
            save()
            log(report["verdict"])
            return 3
        time.sleep(0.2)
    # laisse une 4e écriture arriver si elle est imminente : non, on agit dès que >= N
    log(f"écritures prouvées : {seen} (premier incrément à {increments[0][0]} s)")
    report["increments_avant_sigterm"] = increments

    # 2. photographie
    rows = ps_rows()
    snapshot = descendants(rows, daemon_pid)
    for row in snapshot:
        row["kind"] = kind_of(row["command"])
        row["command"] = row["command"][:170]
    extra = [r for r in rows.values()
             if "heartbeat.txt" in r["command"] and r["pid"] not in [s["pid"] for s in snapshot]]
    for row in extra:
        row["kind"] = "commande-battement(hors arbre)"
        row["command"] = row["command"][:170]
    snapshot += extra
    report["daemon_pid"] = daemon_pid
    report["daemon_lstart"] = rows[daemon_pid]["lstart"] if daemon_pid in rows else None
    report["snapshot"] = snapshot
    kinds = sorted({s["kind"] for s in snapshot})
    log(f"photographie : {len(snapshot)} processus descendants, types {kinds}")
    needed = {"wrapper-bridget", "fournisseur-codex-app-server", "commande-battement"}
    missing = needed - {s["kind"].split("(")[0] for s in snapshot}
    if missing:
        report["avertissement"] = f"types absents de la photographie : {sorted(missing)}"
        log(report["avertissement"])

    # 3. SIGTERM au daemon seul, PID revérifié
    env_text = subprocess.run(["ps", "-Eww", "-p", str(daemon_pid), "-o", "command="],
                              capture_output=True, text=True).stdout
    if f"BRIDGET_HOME={home}" not in env_text or " daemon" not in env_text or "firefox" in env_text.lower():
        report["verdict"] = "BLOQUE: le PID du pidfile n'est pas le daemon de ce fixture"
        save()
        log(report["verdict"])
        return 4
    lines_at_sigterm = line_count(heartbeat)
    t_sig = time.monotonic()
    epoch_sig = time.time()
    os.kill(daemon_pid, signal.SIGTERM)
    log(f"SIGTERM envoyé au daemon fixture pid={daemon_pid} ; battements={lines_at_sigterm}")
    report["sigterm"] = {"epoch": epoch_sig, "lignes_au_sigterm": lines_at_sigterm}

    # 4. mesure des morts
    deaths = {}
    last_count, last_change_t = lines_at_sigterm, 0.0
    deadline = t_sig + 90
    while time.monotonic() < deadline:
        now = ps_rows()
        for row in snapshot:
            key = row["pid"]
            current = now.get(key)
            if key not in deaths and (current is None or current["lstart"] != row["lstart"]):
                deaths[key] = round(time.monotonic() - t_sig, 2)
        count = line_count(heartbeat)
        if count != last_count:
            last_count, last_change_t = count, round(time.monotonic() - t_sig, 2)
        if daemon_pid not in now or now[daemon_pid]["lstart"] != report["daemon_lstart"]:
            deaths.setdefault(daemon_pid, round(time.monotonic() - t_sig, 2))
        if all(s["pid"] in deaths for s in snapshot) and daemon_pid in deaths:
            break
        time.sleep(0.1)
    report["morts_secondes_apres_sigterm"] = {str(k): v for k, v in sorted(deaths.items())}
    report["survivants_apres_90s"] = [s for s in snapshot if s["pid"] not in deaths]
    log(f"morts mesurées : {report['morts_secondes_apres_sigterm']}")

    # fichier quiet : 15 s (5 battements) après la dernière mort
    quiet_start = time.monotonic()
    lines_after_deaths = line_count(heartbeat)
    while time.monotonic() - quiet_start < 15:
        time.sleep(0.5)
    lines_final = line_count(heartbeat)
    report["fichier"] = {"lignes_au_sigterm": lines_at_sigterm,
                         "lignes_a_la_derniere_mort": lines_after_deaths,
                         "lignes_15s_plus_tard": lines_final,
                         "derniere_ecriture_apres_sigterm_s": last_change_t,
                         "croissance_apres_sigterm": lines_final - lines_at_sigterm,
                         "quiet_15s": lines_final == lines_after_deaths}
    log(f"fichier : {report['fichier']}")

    rows = ps_rows()
    residual = [r for r in rows.values()
                if (fixture in r["command"] or "heartbeat.txt" in r["command"])
                and "r6_lifecycle" not in r["command"]]
    parent_tree = [{"pid": r["pid"], "command": r["command"][:140]} for r in residual]
    report["processus_mentionnant_le_fixture_apres_arret_daemon"] = parent_tree
    log(f"processus mentionnant le fixture (le parent PTY attendu seul) : {len(parent_tree)}")

    # 5. état en base, daemon arrêté
    try:
        base = db_rows(home, options.request_id)
        report["etat_base_daemon_arrete"] = [json.loads(p).get("state") for _, p in base]
    except Exception as error:  # noqa: BLE001 - constat de recette
        report["etat_base_daemon_arrete"] = f"illisible: {error}"
    log(f"état en base, daemon arrêté : {report['etat_base_daemon_arrete']}")
    save()

    # 6. relance privée
    sock = os.path.join(home, "bridget.sock")
    for _ in range(20):
        if not os.path.exists(sock):
            break
        time.sleep(0.25)
    report["socket_residuel_avant_relance"] = os.path.exists(sock)
    if os.path.exists(sock):
        log("socket encore présent après l'arrêt : constat consigné, pas de suppression automatique")
        report["verdict"] = "BLOQUE: socket résiduel après SIGTERM daemon"
        save()
        return 5
    code = subprocess.run([sys.executable, os.path.join(HERE, "r6_daemon.py"), "start",
                           "--fixture-root", fixture, "--bin149", options.bin149]).returncode
    report["relance_code"] = code
    with open(pidfile, encoding="utf-8") as handle:
        new_pid = int(handle.read().strip())
    report["nouveau_daemon_pid"] = new_pid
    t_restart = time.monotonic()
    log(f"daemon relancé pid={new_pid} code={code}")

    first_terminal = None
    states = []
    while time.monotonic() - t_restart < 90:
        try:
            base = db_rows(home, options.request_id)
            state = json.loads(base[0][1]).get("state") if base else None
        except Exception as error:  # noqa: BLE001
            state = f"err:{error}"
        if not states or states[-1][1] != state:
            states.append((round(time.monotonic() - t_restart, 2), state))
        if state in ("failed", "cancelled", "result_available", "completed", "expired"):
            first_terminal = round(time.monotonic() - t_restart, 2)
            break
        time.sleep(0.3)
    report["apres_relance_etats"] = states
    report["apres_relance_premier_terminal_s"] = first_terminal
    log(f"états après relance : {states}")

    # observation 40 s : aucun nouveau processus fournisseur, aucune écriture, une ligne
    obs = []
    lines_before = line_count(heartbeat)
    for _ in range(8):
        time.sleep(5)
        rows = ps_rows()
        tree = descendants(rows, new_pid)
        obs.append({"t": round(time.monotonic() - t_restart, 1),
                    "descendants_nouveau_daemon": [{"pid": r["pid"], "kind": kind_of(r["command"]),
                                                    "command": r["command"][:120]} for r in tree],
                    "battements": line_count(heartbeat)})
    report["observation_40s"] = obs
    base = db_rows(home, options.request_id)
    final_payload = json.loads(base[0][1]) if base else {}
    report["fin"] = {"lignes_pour_request_id": len(base),
                     "etat": final_payload.get("state"),
                     "erreur": final_payload.get("error"),
                     "resultat_present": final_payload.get("result") is not None,
                     "result_sent": final_payload.get("result_sent"),
                     "cleanup_done": final_payload.get("cleanup_done"),
                     "task_id": final_payload.get("task_id"),
                     "battements_avant_observation": lines_before,
                     "battements_apres_observation": line_count(heartbeat)}
    log(f"fin : {report['fin']}")
    report["verdict"] = "MESURES COMPLETES (interprétation dans le rapport)"
    save()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
