#!/usr/bin/env python3
"""r6_verify.py - oracles du smoke réel r6 (A: parent GLM -> enfant GLM ; B: cycle de vie Codex).

Lecture seule (SQLite ro, fichiers, ps). Affiche OBSERVÉ / ATTENDU, ne coche rien.
Usage : r6_verify.py --fixture-root FX --bin149 BIN
"""
import argparse
import collections
import glob
import hashlib
import json
import os
import sqlite3
import subprocess
import sys

CONTRACT_GCLAUDE = "dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e"
CONTRACT_CLI = "c9b5341637becbd423ddffc5b254afb645682a3868cb708bbc6cc0e7bb419937"
BIN_EXPECTED = "0a29ad9b2cdb88b1c19f95d9a9bfd1cd89292e269a92fa440864a25bdfa5dde6"
PROD_EXPECTED = "3943009ca82db59d850d913b55b18c7a0a17c91c3143ffaeb39ee6cdd600526d"
WORKTREE = "/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage"

failures = []


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def check(label, ok, observed):
    print(f"{'OK  ' if ok else 'ECHEC'} {label} : {observed}")
    if not ok:
        failures.append(label)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--bin149", required=True)
    options = parser.parse_args()
    fx = os.path.realpath(options.fixture_root)
    home = os.path.join(fx, "home")
    conn = sqlite3.connect(f"file:{os.path.join(home, 'bridget.db')}?mode=ro", uri=True, timeout=3)

    print("== Intégrité des entrées ==")
    check("binaire r8 (SHA256 fin)", sha(options.bin149) == BIN_EXPECTED, sha(options.bin149))
    check("lanceur gclaude == constante contrat", sha("/Users/moi/.local/bin/gclaude") == CONTRACT_GCLAUDE,
          sha("/Users/moi/.local/bin/gclaude")[:16])
    resolved = os.path.realpath("/Users/moi/.local/bin/claude")
    check("CLI claude résolu == mesure r5/r6", sha(resolved) == CONTRACT_CLI, f"{resolved} {sha(resolved)[:16]}")
    fingerprint = json.loads(subprocess.run(
        [sys.executable, "-I", os.path.join(WORKTREE, "specs/149-sous-agents-lineage/validation/native-r5-fingerprint.py")],
        capture_output=True, text=True).stdout)
    check("empreinte production (fin)", fingerprint["prod"]["value"] == PROD_EXPECTED,
          f"{fingerprint['prod']['value'][:16]} ({fingerprint['prod']['fileCount']} fichiers)")
    check("aucun vecteur T3 dans l'environnement du vérificateur",
          not (os.environ.get("BRIDGET_T3_MCP_ENDPOINT") or os.environ.get("BRIDGET_T3_MCP_AUTHORIZATION")), "absent")

    print("== Grants (AFTER) ==")
    grants = conn.execute("SELECT COUNT(*) FROM native_delegation_grants").fetchone()[0]
    revoc = conn.execute("SELECT COUNT(*) FROM native_delegation_revocations").fetchone()[0]
    cancels = conn.execute("SELECT COUNT(*) FROM native_delegation_cancel_receipts").fetchone()[0]
    check("native_delegation_grants", grants == 0, grants)
    check("native_delegation_revocations", revoc == 0, revoc)
    check("native_delegation_cancel_receipts (aucune annulation émise)", cancels == 0, cancels)

    def row(request_id):
        rows = conn.execute("SELECT task_id, owner_instance, request_id, payload FROM native_delegations "
                            "WHERE request_id = ?", (request_id,)).fetchall()
        return rows

    print("== A : parent GLM PTY -> enfant GLM glm-5.3-flash ==")
    rows = row("r6-glm-smoke-01")
    check("une seule ligne corrélée par request_id", len(rows) == 1, len(rows))
    task_id, owner, owner_instance, text = rows[0]
    p = json.loads(text)
    check("état result_available, sans erreur", p["state"] == "result_available" and p["error"] is None,
          f"{p['state']} error={p['error']}")
    check("résultat unique relayé (result_sent) et nettoyage fait", p["result_sent"] is True and p["cleanup_done"] is True,
          f"result_sent={p['result_sent']} cleanup_done={p['cleanup_done']} failure_sent={p['failure_sent']}")
    request = p["request"]
    check("requête: glm / glm-5.3-flash exact, sans posture", request["agent_type"] == "glm"
          and request["model"] == "glm-5.3-flash" and request.get("posture") is None,
          f"agent={request['agent_type']} model={request['model']} posture={request.get('posture')} "
          f"effective_posture={p['effective_posture']}")
    snap = p["permission_snapshot"]
    parent, child = snap["parent"], snap["child_policy"]
    check("fait parent issu du wrapper (source native_wrapper), mode réel", parent["source"] == "native_wrapper"
          and parent["runtime_mode"] == "auto" and parent["interaction_mode"] == "default",
          f"source={parent['source']} runtime_mode={parent['runtime_mode']} interaction_mode={parent['interaction_mode']} "
          f"revision={parent['revision']} driver={parent['driver']}")
    lc = child["launch_context"]
    check("lanceur gclaude figé dans la politique enfant", lc["cli_revision"] == "sha256:" + CONTRACT_GCLAUDE
          and lc["cli_path"] == "/Users/moi/.local/bin/gclaude", f"{lc['cli_path']} {lc['cli_revision'][:23]}…")
    check("CLI résolu figé", lc["resolved_cli_revision"] == "sha256:" + CONTRACT_CLI,
          f"{lc['resolved_cli_path']} {lc['resolved_cli_revision'][:23]}…")
    check("profil .claude-glm", lc["config_dir"] == "/Users/moi/.claude-glm", lc["config_dir"])
    sources = lc["permission_sources"]
    kinds = collections.Counter(s["kind"] for s in sources)
    check("17 sources de permission héritées", len(sources) == 17, f"{len(sources)} {dict(kinds)}")
    check("politique enfant == politique parent (héritage exact)", child == parent["provider_policy"],
          "égales" if child == parent["provider_policy"] else "différentes")
    path = os.path.join(fx, "project", "parent-proj", "allowed", "child-smoke.md")
    content = open(path, encoding="utf-8").read() if os.path.exists(path) else None
    check("fichier allowed/child-smoke.md écrit", content == "smoke r6 child ok",
          repr(content) + (f" {len(content)} octets" if content else ""))
    check("zones forbidden/ et outside/ vides",
          not any(os.scandir(os.path.join(fx, "project", "parent-proj", "forbidden")))
          and not any(os.scandir(os.path.join(fx, "outside"))), "vides")
    tdir = glob.glob("/Users/moi/.claude-glm/projects/*r6*" + os.path.basename(fx).split(".")[-1] + "*")
    for directory in tdir:
        for f in sorted(glob.glob(directory + "/*.jsonl")):
            models, tools = collections.Counter(), []
            for line in open(f, encoding="utf-8"):
                try:
                    o = json.loads(line)
                except ValueError:
                    continue
                m = o.get("message") if isinstance(o.get("message"), dict) else {}
                if m.get("model"):
                    models[m["model"]] += 1
                if isinstance(m.get("content"), list):
                    tools += [b.get("name") for b in m["content"] if isinstance(b, dict) and b.get("type") == "tool_use"]
            role = "parent" if os.path.basename(f).startswith(parent["provider_session_id"]) else "enfant"
            print(f"     transcrit {role}: modèles={dict(models)} outils={tools}")

    print("== B : parent Codex -> enfant Codex gpt-6.1-sol high ; SIGTERM daemon puis relance ==")
    rows = row("r6-codex-life-01")
    check("une seule ligne corrélée par request_id (après relance)", len(rows) == 1, len(rows))
    p = json.loads(rows[0][3])
    check("état final failed/unreachable, sans faux résultat", p["state"] == "failed" and p["error"] == "unreachable"
          and p["result"] is None and p["result_sent"] is False,
          f"state={p['state']} error={p['error']} result={p['result']!r} result_sent={p['result_sent']} "
          f"cleanup_done={p['cleanup_done']} failure_sent={p['failure_sent']}")
    request = p["request"]
    check("requête: codex / gpt-6.1-sol / effort high", request["agent_type"] == "codex"
          and request["model"] == "gpt-6.1-sol" and request.get("effort") == "high",
          f"{request['agent_type']} {request['model']} {request.get('effort')} posture={request.get('posture')}")
    snap = p["permission_snapshot"]
    parent, child = snap["parent"], snap["child_policy"]
    print(f"     parent: source={parent['source']} mode={parent.get('runtime_mode')} driver={parent.get('driver')} "
          f"policy={json.dumps(parent.get('provider_policy'), sort_keys=True)[:260]}")
    print(f"     enfant: {json.dumps(child, sort_keys=True)[:260]}")
    check("politique enfant == politique parent", child == parent.get("provider_policy"),
          "égales" if child == parent.get("provider_policy") else "différentes")
    hb = os.path.join(fx, "project", "parent-proj", "allowed", "heartbeat.txt")
    hb_lines = open(hb).read().count("\n")
    check("fichier battement figé à 3 lignes (aucune reprise)", hb_lines == 3, hb_lines)
    for name in ("lifecycle-r6-codex-life-01.json", "restart-r6-codex-life-01.json"):
        data = json.load(open(os.path.join(fx, "state", name)))
        if name.startswith("lifecycle"):
            deaths = data["morts_secondes_apres_sigterm"]
            kinds = {str(s["pid"]): s["kind"] for s in data["snapshot"]}
            kinds[str(data["daemon_pid"])] = "daemon"
            print(f"     photographie: {len(data['snapshot'])} descendants ; morts (s après SIGTERM): max={max(deaths.values())} "
                  f"survivants={len(data['survivants_apres_90s'])}")
            for pid, t in sorted(deaths.items(), key=lambda x: x[1]):
                print(f"       pid {pid} {kinds.get(pid, '?')}: {t} s")
            print(f"     fichier: {data['fichier']}")
            print(f"     base daemon arrêté: {data['etat_base_daemon_arrete']} ; relance 1 code={data['relance_code']}")
        else:
            print(f"     liens symboliques: {data['liens_symboliques_dans_home']} retiré={data.get('lien_retire')}")
            print(f"     relance 2 code={data['relance_code']} états={data['etats']} fin={data['fin']}")
    print("VERDICT SCRIPT:", "ECHECS " + str(failures) if failures else "tous les oracles mesurables sont conformes")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
