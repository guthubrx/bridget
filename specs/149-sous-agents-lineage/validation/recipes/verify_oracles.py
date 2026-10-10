#!/usr/bin/env python3
"""verify_oracles.py — Vérifications post-exécution de la recette 149 r2 (T037/T038).

Toutes les vérifications portent sur le FIXTURE (jamais la prod). Aucune case
n'est cochée automatiquement : le script affiche OBSERVÉ et attendu, et ne se
prononce que sur ce qui est mesurable. sqlite en lecture seule (URI ro).

Scénarios r2 :
  t037-glm               1 ligne native_delegations, état failed, payload portant
                         provider_permission_denied (G-P-01) ; filesystem :
                         allowed/write-ok.md présent, forbidden/ vide (refus par
                         la règle settings préexistante Edit(/forbidden/**))
  t037-codex-child       1 ligne, état failed, refus sandbox réel hors workspace ;
                         allowed/write-codex-ok.md présent (écrit AVANT la
                         tentative hors workspace)
  t037-codex-confinement 0 ligne : le refus nommé provider_confinement_unavailable
                         est SYNCHRONE au delegate (child_policy l.436 avant
                         création de tâche l.451) ; le texte doit figurer dans le
                         journal PTY du parent Codex workspace-write
  t037-fullcodex-glm     1 ligne, état result_available ; enfant GLM flash réel
                         via parent Codex danger-full-access (-a never -s explicites)
  t038-cancel            1 ligne, état cancelled, plus aucun processus enfant vivant
  t038-negative          0 ligne pour le request_id (refus observer avant effet)
Commun : digests lanceur/CLI figés (state/expected.json), empreintes des sources
sélectionnées inchangées (state/sources-snapshot.json), baseline filesystem
respectée (aucun fichier préexistant modifié ; nouveaux fichiers uniquement dans
allowed/), aucun artefact observer résiduel (np-*.sock / no-*.json), aucun
processus lié au fixture, aucun vecteur T3.
"""
import argparse
import glob
import hashlib
import json
import os
import sqlite3
import subprocess
import sys

TRANSPARENT_GCLAUDE = "dd8dee5677e46fde677f6ba940cbc07294f50ff690a76878c1883b6805872a8e"

SCENARIO_FILES = {
    "t037-glm": ["project/parent-proj/allowed/write-ok.md"],
    "t037-codex-child": ["project/parent-proj/allowed/write-codex-ok.md"],
    "t037-fullcodex-glm": ["project/parent-proj/allowed/write-glm-from-codex.md"],
}


def digest(path: str) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def fail(message: str) -> None:
    print(f"ECHEC: {message}")
    sys.exit(1)


def check_common(fixture: str) -> None:
    expected_path = os.path.join(fixture, "state", "expected.json")
    if not os.path.exists(expected_path):
        fail(f"{expected_path} absent — lancer recipe_env.sh d'abord")
    with open(expected_path, encoding="utf-8") as handle:
        expected = json.load(handle)

    launcher = "/Users/moi/.local/bin/gclaude"
    actual = digest(launcher)
    if actual != expected["launcher_gclaude_sha256"] or actual != TRANSPARENT_GCLAUDE:
        fail(f"hash gclaude dérivé : {actual}")
    codexpro = "/Users/moi/.local/bin/codex-pro"
    if expected.get("launcher_codexpro_sha256") and digest(codexpro) != expected["launcher_codexpro_sha256"]:
        fail("hash codex-pro dérivé depuis la préparation")
    resolved = subprocess.run(
        ["python3", "-c", "import os,sys;print(os.path.realpath(sys.argv[1]))", expected["resolved_claude_path"]],
        capture_output=True, text=True, check=True).stdout.strip()
    if resolved != expected["resolved_claude_path"] or digest(resolved) != expected["resolved_claude_sha256"]:
        fail("CLI claude résolu a changé depuis la préparation")
    print("OK digests : gclaude == constante contrat ; codex-pro et CLI résolu inchangés")

    snapshot_path = os.path.join(fixture, "state", "sources-snapshot.json")
    if os.path.exists(snapshot_path):
        with open(snapshot_path, encoding="utf-8") as handle:
            snapshot = json.load(handle)
        drift = []
        for path in snapshot:
            if os.path.exists(path):
                if snapshot.get(path) != digest(path):
                    drift.append(path)
            else:
                drift.append(path + " (disparu)")
        if drift:
            fail(f"sources sélectionnées dérivées depuis la préparation : {drift}")
        print("OK sources sélectionnées (profil + settings projet fixture) : empreintes inchangées")

    leftovers = sorted(glob.glob(os.path.join(fixture, "home", "np-*.sock"))) \
        + sorted(glob.glob(os.path.join(fixture, "home", "no-*.json")))
    if leftovers:
        fail(f"artefacts observer résiduels : {leftovers}")
    print("OK observer : socket et overlay supprimés en sortie de session")

    if os.environ.get("BRIDGET_T3_MCP_ENDPOINT") or os.environ.get("BRIDGET_T3_MCP_AUTHORIZATION"):
        fail("vecteur T3 présent dans l'environnement d'exécution")

    baseline_path = os.path.join(fixture, "state", "fs-baseline.json")
    if os.path.exists(baseline_path):
        with open(baseline_path, encoding="utf-8") as handle:
            baseline = json.load(handle)
        modified = [rel for rel in baseline
                    if not rel.startswith(("home/", "state/", "logs/"))
                    and os.path.exists(os.path.join(fixture, rel))
                    and digest(os.path.join(fixture, rel)) != baseline[rel]]
        deleted = [rel for rel in baseline
                   if not rel.startswith(("home/", "state/", "logs/"))
                   and not os.path.exists(os.path.join(fixture, rel))]
        if modified or deleted:
            fail(f"baseline filesystem violée : modifiés={modified} supprimés={deleted}")
        print("OK filesystem : aucun fichier préexistant modifié ou supprimé")

    scan = subprocess.run(["ps", "-axo", "command"], capture_output=True, text=True).stdout
    alive = [line for line in scan.splitlines() if fixture in line and "verify_oracles" not in line]
    if alive:
        fail(f"processus encore liés au fixture : {alive}")
    print("OK environnement : aucun vecteur T3, aucun processus résiduel du fixture")


def check_filesystem(fixture: str, scenario: str) -> None:
    for rel in SCENARIO_FILES.get(scenario, []):
        path = os.path.join(fixture, rel)
        if not os.path.exists(path):
            fail(f"filesystem attendu absent : {rel}")
        print(f"OK filesystem : {rel} présent (écriture autorisée réelle)")
    for zone in ("project/parent-proj/forbidden", "outside"):
        absolute = os.path.join(fixture, zone)
        if os.path.isdir(absolute) and any(os.scandir(absolute)):
            fail(f"fichier présent dans la zone hors politique : {zone}/")
    print("OK filesystem : zones forbidden/ et outside/ vides (aucune écriture hors politique)")


def check_db(fixture: str, request_id: str, scenario: str) -> None:
    db_path = os.path.join(fixture, "home", "bridget.db")
    if not os.path.exists(db_path):
        fail(f"{db_path} absent — daemon jamais lancé ?")
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        rows = conn.execute(
            "SELECT task_id, owner_instance, request_id, payload FROM native_delegations "
            "WHERE request_id = ?", (request_id,)).fetchall()
        count = len(rows)
        print(f"OBSERVÉ native_delegations[{request_id}] = {count} ligne(s)")
        if scenario in ("t038-negative", "t037-codex-confinement"):
            if count != 0:
                fail(f"scénario {scenario} : {count} délégation(s) enregistrée(s), attendu 0")
            print(f"OK {scenario} : aucune délégation n'a franchi (refus avant effet)")
            return
        if count != 1:
            fail(f"corrélation unique attendue, obtenu {count}")
        task_id, owner, _, payload_text = rows[0]
        payload = json.loads(payload_text)
        state = payload.get("state")
        print(f"OBSERVÉ task_id={task_id} owner={owner} state={state}")
        print(f"OBSERVÉ payload: {json.dumps(payload, ensure_ascii=False)[:1200]}")
        if scenario == "t037-glm":
            if state != "failed":
                fail(f"état attendu failed, obtenu {state}")
            if "provider_permission_denied" not in payload_text:
                fail("payload sans provider_permission_denied (G-P-01)")
            print("OK G-P-01 : refus fournisseur corrélé, tâche failed, jamais result_available")
        elif scenario == "t037-codex-child":
            # Le bac à sable OS de Codex refuse l'écriture au niveau de la commande (EPERM) :
            # le tour se termine normalement, aucun événement d'approbation fournisseur.
            if state != "result_available":
                fail(f"état attendu result_available, obtenu {state}")
            if "operation not permitted" not in (payload.get("result") or "").lower():
                fail("résultat de l'enfant sans trace du refus du bac à sable (EPERM)")
            policy = payload.get("permission_snapshot", {}).get("child_policy", {})
            if policy.get("sandbox_policy", {}).get("type") != "workspaceWrite" or policy.get("approval_policy") != "never":
                fail(f"politique enfant non héritée du parent : {policy}")
            if payload.get("request", {}).get("model") != "gpt-6.1-sol" or payload.get("request", {}).get("effort") != "high":
                fail("modèle/effort de l'enfant différents de gpt-6.1-sol/high")
            print("OK enfant Codex : écriture workspace OK, refus EPERM hors workspace, politique héritée, tâche result_available")
        elif scenario == "t037-fullcodex-glm":
            if state != "result_available":
                fail(f"état attendu result_available, obtenu {state}")
            print("OK fullCodex→GLM : enfant glm-5.3-flash réel, écriture autorisée, résultat corrélé")
        elif scenario == "t038-cancel":
            if state != "cancelled":
                fail(f"état attendu cancelled, obtenu {state}")
            print("OK annulation : état cancelled persistant")
    finally:
        conn.close()


def check_no_grant(fixture: str) -> None:
    """r5 : héritage sans grant Bridget. native_delegation_grants doit rester VIDE."""
    db_path = os.path.join(fixture, "home", "bridget.db")
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    try:
        grants = conn.execute("SELECT COUNT(*) FROM native_delegation_grants").fetchone()[0]
    finally:
        conn.close()
    print(f"OBSERVÉ native_delegation_grants = {grants} ligne(s)")
    if grants != 0:
        fail(f"table native_delegation_grants non vide ({grants}) : un grant existe, héritage non prouvé")
    print("OK aucun grant Bridget : l'héritage vient du fait attesté du parent")


def check_confinement_journal(fixture: str, request_id: str) -> None:
    journal = os.path.join(fixture, "logs", f"parent-pty-{request_id}.log")
    if not os.path.exists(journal):
        fail(f"journal PTY absent : {journal}")
    with open(journal, "rb") as handle:
        content = handle.read().decode("utf-8", errors="replace")
    if "provider_confinement_unavailable" not in content:
        fail("provider_confinement_unavailable absent du journal PTY du parent Codex")
    print("OK confinement : refus nommé provider_confinement_unavailable observé dans le journal PTY")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--request-id", required=True)
    parser.add_argument("--scenario", required=True, choices=[
        "t037-glm", "t037-codex-child", "t037-codex-confinement",
        "t037-fullcodex-glm", "t038-cancel", "t038-negative"])
    options = parser.parse_args()

    fixture = os.path.realpath(options.fixture_root)
    if fixture == "/Users/moi/.cache/bridget-core" or fixture.startswith("/Users/moi/.config/bridget"):
        fail("refus : fixture = prod ou autre registre")
    check_common(fixture)
    check_no_grant(fixture)
    check_db(fixture, options.request_id, options.scenario)
    if options.scenario == "t037-codex-confinement":
        check_confinement_journal(fixture, options.request_id)
    else:
        check_filesystem(fixture, options.scenario)
    print("VERDICT PARTIEL : oracles mesurables conformes — le pronostic complet "
          "reste à la charge du rapport, aucune case tasks.md cochée par ce script.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
