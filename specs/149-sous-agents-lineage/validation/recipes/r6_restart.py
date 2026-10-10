#!/usr/bin/env python3
"""r6_restart.py - phase « relance privée » du scénario B (smoke r6, session 149).

Reprend la phase 6 de r6_lifecycle.py quand la première relance a été refusée par la validation
d'environnement du daemon (« symlink d'état interdit » : le parent Codex TUI vivant laisse
home/c-<instance>.sock, un lien symbolique, dans le namespace). Ce script :
  - consigne chaque lien symbolique présent dans home/ (nom + cible), SANS supprimer la cible ;
  - ne retire QUE ce lien (os.unlink sur le lien lui-même, jamais sur sa cible) quand
    --remove-parent-socket-link est passé : le TUI déjà connecté n'en a plus besoin ;
  - relance le daemon fixture (r6_daemon.py start), puis observe sans rien envoyer :
    états de la tâche, arbre des descendants du nouveau daemon, battements, lignes en base.
Journal JSON : <fixture>/state/restart-<request_id>.json.
"""
import argparse
import json
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from r6_lifecycle import db_rows, descendants, kind_of, line_count, log, ps_rows  # noqa: E402


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--bin149", required=True)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--remove-parent-socket-link", action="store_true")
    parser.add_argument("--observe", type=float, default=45.0)
    options = parser.parse_args()

    fixture = os.path.realpath(options.fixture_root)
    home = os.path.join(fixture, "home")
    heartbeat = os.path.join(fixture, "project", "parent-proj", "allowed", "heartbeat.txt")
    pidfile = os.path.join(fixture, "state", "daemon.pid")
    report = {"request_id": options.request_id}
    out_path = os.path.join(fixture, "state", f"restart-{options.request_id}.json")

    links = []
    for name in sorted(os.listdir(home)):
        path = os.path.join(home, name)
        if os.path.islink(path):
            links.append({"nom": name, "cible": os.readlink(path)})
    report["liens_symboliques_dans_home"] = links
    log(f"liens symboliques dans home : {links}")
    if links and options.remove_parent_socket_link:
        for link in links:
            if link["nom"].startswith("c-") and link["nom"].endswith(".sock"):
                os.unlink(os.path.join(home, link["nom"]))  # le lien seulement
                log(f"lien retiré (cible intacte) : {link['nom']}")
        report["lien_retire"] = True
    base_before = db_rows(home, options.request_id)
    report["etat_avant"] = [json.loads(p).get("state") for _, p in base_before]
    lines_before = line_count(heartbeat)

    code = subprocess.run([sys.executable, os.path.join(HERE, "r6_daemon.py"), "start",
                           "--fixture-root", fixture, "--bin149", options.bin149]).returncode
    report["relance_code"] = code
    with open(pidfile, encoding="utf-8") as handle:
        new_pid = int(handle.read().strip())
    report["nouveau_daemon_pid"] = new_pid
    report["relance_epoch"] = time.time()
    t0 = time.monotonic()
    log(f"relance code={code} pid={new_pid}")
    if code != 0:
        report["verdict"] = "BLOQUE: relance refusée"
        fd = os.open(out_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd, "w") as handle:
            json.dump(report, handle, indent=1, ensure_ascii=False)
        return 1

    states, first_terminal, trees = [], None, []
    last_tree = 0.0
    while time.monotonic() - t0 < options.observe:
        try:
            base = db_rows(home, options.request_id)
            state = json.loads(base[0][1]).get("state") if base else None
        except Exception as error:  # noqa: BLE001
            state = f"err:{error}"
        if not states or states[-1][1] != state:
            states.append((round(time.monotonic() - t0, 2), state))
            log(f"état {state} à +{states[-1][0]} s")
        if first_terminal is None and state in ("failed", "cancelled", "result_available", "completed", "expired"):
            first_terminal = round(time.monotonic() - t0, 2)
        if time.monotonic() - last_tree >= 3:
            last_tree = time.monotonic()
            tree = descendants(ps_rows(), new_pid)
            trees.append({"t": round(time.monotonic() - t0, 1),
                          "descendants": [{"pid": r["pid"], "kind": kind_of(r["command"]),
                                           "command": r["command"][:120]} for r in tree],
                          "battements": line_count(heartbeat)})
        time.sleep(0.3)
    report["etats"] = states
    report["premier_terminal_s"] = first_terminal
    report["arbres_nouveau_daemon"] = trees
    base = db_rows(home, options.request_id)
    payload = json.loads(base[0][1]) if base else {}
    conn_total = None
    try:
        import sqlite3
        conn = sqlite3.connect(f"file:{os.path.join(home, 'bridget.db')}?mode=ro", uri=True, timeout=3)
        conn_total = conn.execute("SELECT COUNT(*) FROM native_delegations").fetchone()[0]
        conn.close()
    except Exception as error:  # noqa: BLE001
        conn_total = f"err:{error}"
    report["fin"] = {"lignes_pour_request_id": len(base),
                     "lignes_native_delegations_total": conn_total,
                     "etat": payload.get("state"), "erreur": payload.get("error"),
                     "resultat_present": payload.get("result") is not None,
                     "result_sent": payload.get("result_sent"), "cleanup_done": payload.get("cleanup_done"),
                     "failure_sent": payload.get("failure_sent"), "task_id": payload.get("task_id"),
                     "battements_avant_relance": lines_before, "battements_apres": line_count(heartbeat),
                     "max_descendants_observes": max((len(t["descendants"]) for t in trees), default=0)}
    log(f"fin : {report['fin']}")
    fd = os.open(out_path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as handle:
        json.dump(report, handle, indent=1, ensure_ascii=False)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
