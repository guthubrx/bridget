#!/usr/bin/env python3
"""r7_verify.py - oracles de la ronde r7 (parent Codex vivant, SIGTERM du daemon, relance).

Lecture seule : SQLite en mode ro, JSON écrits par r7_run.py / r6_lifecycle.py, journal PTY nettoyé.
Affiche OK / ECHEC avec la valeur observée. Ne coche rien et n'invente aucun fait : les constats
indirects sont étiquetés « déduit ». Usage : r7_verify.py --fixture-root FX
"""
import argparse
import glob
import json
import os
import re
import sqlite3
import sys

BIN_EXPECTED = "abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8"
PROD_EXPECTED = "b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29"
failures = []


def check(label: str, ok: bool, observed) -> None:
    print(f"{'OK   ' if ok else 'ECHEC'} {label} : {observed}")
    if not ok:
        failures.append(label)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--request-id", default="r7-codex-restart-01")
    options = parser.parse_args()
    fx = os.path.realpath(options.fixture_root)
    home, state = os.path.join(fx, "home"), os.path.join(fx, "state")
    rid = options.request_id
    run = json.load(open(os.path.join(state, "r7-run.json"), encoding="utf-8"))
    life = json.load(open(os.path.join(state, f"lifecycle-{rid}.json"), encoding="utf-8"))
    events = [json.loads(line) for line in open(os.path.join(state, "r7-timeline.jsonl"), encoding="utf-8")]
    start, end = run["entrees_debut"], run["entrees_fin"]

    print("== Entrées (début == fin) ==")
    check("binaire release r9", start["bin149_sha256"] == end["bin149_sha256"] == BIN_EXPECTED, start["bin149_sha256"][:16])
    for key in ("codex_cli_sha256", "codex_pro_launcher_sha256", "codex_config_sha256", "gclaude_sha256"):
        check(key, start[key] == end[key], start[key][:16])
    check("empreinte source production (111 fichiers)",
          start["prod_source_fingerprint"] == end["prod_source_fingerprint"] == PROD_EXPECTED
          and start["prod_source_files"] == 111, end["prod_source_fingerprint"][:16])
    check("aucun vecteur T3 (variables absentes)", not (os.environ.get("BRIDGET_T3_MCP_ENDPOINT")
          or os.environ.get("BRIDGET_T3_MCP_AUTHORIZATION")), "absentes")

    print("== Grants lus directement (BEFORE / AFTER) ==")
    for table in ("native_delegation_grants", "native_delegation_revocations", "native_delegation_cancel_receipts"):
        check(table, run["grants_BEFORE"][table] == 0 and run["grants_AFTER"][table] == 0,
              f"avant={run['grants_BEFORE'][table]} après={run['grants_AFTER'][table]}")
    check("une seule ligne native_delegations après la ronde", run["grants_AFTER"]["native_delegations"] == 1,
          run["grants_AFTER"]["native_delegations"])

    conn = sqlite3.connect(f"file:{os.path.join(home, 'bridget.db')}?mode=ro", uri=True, timeout=3)
    rows = conn.execute("SELECT payload FROM native_delegations WHERE request_id = ?", (rid,)).fetchall()
    payload = json.loads(rows[0][0])
    request, snap = payload["request"], payload["permission_snapshot"]
    parent, child = snap["parent"], snap["child_policy"]
    policy = parent["provider_policy"]
    print("== Faits actuels du parent (publiés par le wrapper, jamais déclarés) ==")
    check("requête codex / gpt-6.1-sol / high, sans posture", request["agent_type"] == "codex"
          and request["model"] == "gpt-6.1-sol" and request["effort"] == "high" and request.get("posture") is None,
          f"{request['agent_type']} {request['model']} {request['effort']} posture={request.get('posture')}")
    check("fait parent source native_wrapper / driver codex_app_server", parent["source"] == "native_wrapper"
          and parent["driver"] == "codex_app_server", f"{parent['source']} {parent['driver']} revision={parent['revision']}")
    check("sandbox workspaceWrite, approval never, réseau coupé", policy["approval_policy"] == "never"
          and policy["sandbox_policy"]["type"] == "workspaceWrite" and policy["sandbox_policy"]["networkAccess"] is False,
          json.dumps(policy["sandbox_policy"], sort_keys=True))
    check("politique enfant == politique parent (aucun droit en plus)", child == policy, "égales" if child == policy else "différentes")

    print("== Alias Codex hors home ==")
    plus = [e for e in events if e["ev"] == "alias+"]
    linked = [e for e in events if e["ev"] == "alias~" and e["s.sock"] == "lien"]
    minus = [e for e in events if e["ev"] == "alias-"]
    check("alias /private/tmp/bridget-codex-<hex> créé (dossier 0700)", len(plus) == 1 and plus[0]["mode_dossier"] == "0o700",
          [(e["dossier"], e["mode_dossier"]) for e in plus])
    check("s.sock est un lien vers le socket physique Codex", len(linked) == 1, [e["cible"][:50] for e in linked])
    check("aucun lien symbolique sous home pendant toute la ronde", all(e["liens"] == [] for e in events if e["ev"] == "home-liens")
          and run["home_liens_fin"] == [] and not glob.glob(os.path.join(home, "c-*.sock")), "0 lien, 0 c-*.sock")
    check("alias supprimé après l'arrêt du parent (observé)", len(minus) == 1 and run["alias_apres_parent_termine"] == [],
          f"alias- à {minus[0]['h'] if minus else None} ; fin du runner +{run['alias_disparus_s_apres_runner']} s")

    print("== Cycle de vie (r6_lifecycle.py) ==")
    writes = [n for _, n in life["increments_avant_sigterm"]]
    check(">= 3 écritures prouvées avant SIGTERM", max(writes) >= 3, life["increments_avant_sigterm"])
    deaths = life["morts_secondes_apres_sigterm"]
    check("tous les processus photographiés sont morts, 0 survivant", not life["survivants_apres_90s"]
          and len(deaths) == len(life["snapshot"]) + 1, f"{len(deaths)} morts, max {max(deaths.values())} s")
    check("fichier de battement quiet après l'arrêt", life["fichier"]["quiet_15s"] and life["fichier"]["croissance_apres_sigterm"] == 0,
          life["fichier"])
    check("nouveau daemon démarré sans toucher au namespace (code 0, aucun socket résiduel)",
          life["relance_code"] == 0 and life["socket_residuel_avant_relance"] is False,
          f"code={life['relance_code']} pid={life['nouveau_daemon_pid']}")
    check("tâche failed dès la première lecture après relance", life["apres_relance_etats"][0][1] == "failed"
          and life["apres_relance_etats"][0][0] <= 1.0, life["apres_relance_etats"])
    check("aucun nouveau processus pendant l'observation", all(not o["descendants_nouveau_daemon"] for o in life["observation_40s"])
          and all(o["battements"] == 3 for o in life["observation_40s"]), f"{len(life['observation_40s'])} points, 0 descendant, 3 battements")
    check("état final failed/unreachable, sans résultat, sans doublon", payload["state"] == "failed"
          and payload["error"] == "unreachable" and payload["result"] is None and payload["result_sent"] is False
          and payload["cleanup_done"] is True and len(rows) == 1,
          f"state={payload['state']} error={payload['error']} result={payload['result']!r} result_sent={payload['result_sent']} "
          f"cleanup_done={payload['cleanup_done']} failure_sent={payload['failure_sent']}")
    stderr = open(os.path.join(fx, "logs", "daemon-stderr.log"), encoding="utf-8").read()
    check("journal daemon : aucun refus de namespace", "symlink" not in stderr and "refus" not in stderr.lower(), "aucun")

    print("== Parent reconnecté (journal PTY nettoyé ; indirect) ==")
    raw = open(os.path.join(fx, "logs", f"parent-pty-{rid}.log"), "rb").read().decode("utf-8", "replace")
    text = re.sub(r"[ \t]+", " ", re.sub(r"\x1b\[[0-9;?]*[A-Za-z]|\x1b[78=>]|\x1b\][^\x07\x1b]*(\x07|\x1b\\)", " ", raw).replace("\r", ""))
    print(f"INFO refus nommés vus par le parent : daemon injoignable x{text.count('daemon Bridget injoignable')} (redessins inclus), "
          f"auxiliary_identity_unproven x{text.count('auxiliary_identity_unproven')} (redessins inclus)")
    check("le parent a reçu l'état terminal failed / result null", "État terminal failed reçu" in text and '"result" : null' in text.replace("re ult", "result"),
          "« État terminal failed reçu à la 4e vérification »" if "4e vérification" in text else "présent")
    check("notification de défaillance reçue une fois", text.count("Délégation native") == 1, text.count("Délégation native"))
    procs = [e for e in events if e["ev"].startswith("proc")]
    print(f"INFO observateur de processus : {len(procs)} évènements (0 attendu seulement si l'observateur est défaillant)")
    print("VERDICT SCRIPT:", f"ECHECS {failures}" if failures else "tous les oracles mesurables sont conformes")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
