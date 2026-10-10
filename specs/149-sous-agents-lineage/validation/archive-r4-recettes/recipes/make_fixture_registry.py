#!/usr/bin/env python3
"""make_fixture_registry.py — Registre fixture privé de la recette 149 r2 (T037/T038).

Clone CONTRÔLÉ du registre PROD lisible (lecture seule), avec des écarts voulus
et uniquement eux :
  1. glm : modèle pinné exact `glm-5.3-flash` dans args ET capabilities.models
     (aucun repli glm-5.3) — le registre prod porte glm-5.3 ;
  2. codex (clone de codex-pro) : conservé tel quel (commande, protocol
     codex_app_server, modèle gpt-6.1-sol) ; l'effort `high` passe par le
     paramètre `effort` du delegate (catalog prouvé : ~/.codex/config.toml
     actif déclare model=gpt-6.1-sol + model_reasoning_effort=high ; CLI
     codex 0.161.0 valide --sandbox read-only|workspace-write|danger-full-access).
Aucun secret : le clone ne copie que command/args/protocol/config_dir/
capabilities/pass_env/forbidden_env/mcp ; aucun jeton (scan heuristique avant
écriture). Le fichier de sortie reçoit le mode 0600. Idempotent via --force.
"""
import argparse
import json
import os
import re
import sys

PROD_REGISTRY = "/Users/moi/.cache/bridget-core/agents.json"
STRUCTURAL_KEYS = ["command", "args", "protocol", "claude_config_dir",
                   "capabilities", "pass_env", "forbidden_env", "mcp"]


def fail(message: str) -> int:
    print(f"ECHEC: {message}", file=sys.stderr)
    return 1


def scan_secrets(obj, path=""):
    """Heuristique : refuse toute valeur ressemblant à un credential."""
    findings = []
    if isinstance(obj, dict):
        for key, value in obj.items():
            if re.search(r"token|secret|password|authorization|api[_-]?key",
                         key, re.I) and not isinstance(value, (dict, list)):
                findings.append(f"{path}/{key}")
            else:
                findings += scan_secrets(value, f"{path}/{key}")
    elif isinstance(obj, list):
        for index, value in enumerate(obj):
            findings += scan_secrets(value, f"{path}[{index}]")
    elif isinstance(obj, str) and len(obj) > 64 and not obj.startswith("/"):
        findings.append(f"{path} (chaîne longue suspecte)")
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--home", required=True,
                        help="BRIDGET_HOME fixture (agents.json y est écrit)")
    parser.add_argument("--force", action="store_true",
                        help="remplacer un registre fixture existant")
    options = parser.parse_args()

    home = os.path.realpath(options.home)
    if home.rstrip("/").endswith("bridget-core") or home.startswith("/Users/moi/.config/bridget"):
        return fail("refus : cible = registre prod ou autre registre")
    try:
        with open(PROD_REGISTRY, encoding="utf-8") as handle:
            prod = json.load(handle)
    except OSError as error:
        return fail(f"registre prod illisible (lecture seule seule permise) : {error}")

    agents = {}
    glm = prod.get("agents", {}).get("glm")
    codex = prod.get("agents", {}).get("codex-pro")
    if glm is None or codex is None:
        return fail("registre prod sans entrées glm/codex-pro attendues")

    # Écart 1 — glm : modèle exact glm-5.3-flash, aucun repli.
    glm_clone = {key: glm[key] for key in STRUCTURAL_KEYS if key in glm}
    glm_clone["args"] = [
        "glm-5.3-flash" if value == "glm-5.3" else value for value in glm_clone.get("args", [])
    ]
    if "--model" not in glm_clone["args"] or "glm-5.3-flash" not in glm_clone["args"]:
        return fail("args glm prod sans paire --model glm-5.3 attendue")
    models = glm_clone.get("capabilities", {}).get("models")
    if not isinstance(models, dict):
        return fail("capabilities.models glm absent du registre prod")
    glm_clone["capabilities"]["models"] = {"glm-5.3-flash": {}}
    agents["glm"] = glm_clone

    # Écart 2 — codex : clone fidèle de codex-pro (protocole codex_app_server).
    agents["codex"] = {key: codex[key] for key in STRUCTURAL_KEYS if key in codex}
    # Écart 3 — l'effort `high` doit être DÉCLARÉ (registry.rs for_delegation : sans
    # `efforts`, tout effort est refusé effort_unavailable). Constat r4 : le registre
    # prod déclare gpt-6.1-sol sans efforts ; la TUI Codex tourne pourtant en `high`.
    agents["codex"]["capabilities"] = json.loads(json.dumps(agents["codex"]["capabilities"]))
    agents["codex"]["capabilities"]["models"]["gpt-6.1-sol"] = {"efforts": ["high"]}

    findings = scan_secrets(agents)
    if findings:
        return fail(f"clone refusé : valeurs sensibles détectées : {findings}")

    target = os.path.join(home, "agents.json")
    flags = os.O_WRONLY | os.O_CREAT | (os.O_TRUNC if options.force else os.O_EXCL)
    try:
        fd = os.open(target, flags, 0o600)
    except FileExistsError:
        return fail(f"refus : {target} existe déjà (utiliser --force)")
    with os.fdopen(fd, "w") as handle:
        json.dump({"agents": agents}, handle, indent=1, sort_keys=True)
        handle.write("\n")
    os.chmod(target, 0o600)
    print(f"OK registre fixture : {target} (0600 ; glm=glm-5.3-flash exact, codex=gpt-6.1-sol, sans secrets)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
