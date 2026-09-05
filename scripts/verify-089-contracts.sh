#!/bin/sh
# Contrôle local uniquement : aucun daemon, fournisseur ou réseau.
set -eu
exec python3 - "$@" <<'PY'
"""O(B) en mémoire/temps pour B octets du corpus borné ; Git reste lecture seule."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

PIN = "dfa2134dcfe2a2522e3ae77d93561e6ae72556b3"
CORPUS = Path("specs/089-communication-core/contracts")
MAX_BYTES = 2 * 1024 * 1024
REQUIRED_FAMILIES = {
    "service-negotiation", "service-deposit-lifecycle", "coordination-v1",
    "coordination-v2", "mcp-dispatch", "journal-turn", "journal-permission",
    "journal-rotation", "message-idempotency", "directory-ledger",
    "managed-lifecycle", "attach-wire", "guichet-claim-reply", "provider-native-wire",
}


def git(root, *args):
    return subprocess.run(
        ["git", "-C", str(root), *args], check=True,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=10,
    ).stdout


def require_ancestor(root, ancestor, head="HEAD"):
    git(root, "merge-base", "--is-ancestor", ancestor, head)


def local_file(root, relative):
    path = Path(relative)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError(f"chemin hors corpus : {relative}")
    target = root / path
    if target.is_symlink() or not target.is_file():
        raise ValueError(f"fichier absent ou symlink : {relative}")
    if not target.resolve().is_relative_to(root.resolve()):
        raise ValueError(f"chemin sortant : {relative}")
    if target.stat().st_size > MAX_BYTES:
        raise ValueError(f"fichier trop grand : {relative}")
    return target


def verify(root, candidates, manifest):
    if set(manifest) != {"version", "source_commit", "entries", "missing_families"}:
        raise ValueError("champs de manifeste inconnus ou manquants")
    if manifest["version"] != 1 or manifest["source_commit"] != PIN:
        raise ValueError("version ou référence Git non épinglée")
    require_ancestor(root, PIN)
    seen = set()
    for entry in manifest["entries"]:
        if set(entry) != {"file", "source", "sha256", "family", "reader"}:
            raise ValueError("champs d'entrée inconnus ou manquants")
        relative = CORPUS / "fixtures" / entry["file"]
        if str(relative) in seen:
            raise ValueError(f"fixture dupliquée : {relative}")
        seen.add(str(relative))
        raw = local_file(candidates, relative).read_bytes()
        if hashlib.sha256(raw).hexdigest() != entry["sha256"]:
            raise ValueError(f"empreinte divergente : {relative}")
        original = git(root, "show", f"{PIN}:{entry['source']}")
        if raw != original:
            raise ValueError(f"octets différents de l'objet Git : {relative}")
        # Fixtures historiques, pas forcément trames entrantes valides : les
        # scénarios négatifs restent des données, jamais normalisées ici.
        for line in raw.splitlines():
            json.loads(line)
        if not raw.endswith(b"\n") or not entry["family"] or not entry["reader"]:
            raise ValueError(f"fixture incomplète : {relative}")
    if not seen:
        raise ValueError("corpus vide")
    missing = sorted(REQUIRED_FAMILIES - {entry["family"] for entry in manifest["entries"]})
    if sorted(manifest["missing_families"]) != missing:
        raise ValueError("les familles manquantes ne correspondent pas aux fixtures présentes")
    return len(seen)


def must_refuse(label, operation):
    try:
        operation()
    except (ValueError, OSError, subprocess.SubprocessError):
        print(f"PASS mutant : {label}")
    else:
        raise ValueError(f"mutant accepté : {label}")


def self_test(root, manifest):
    # Aucune modification du worktree : chaque mutant utilise une copie privée.
    with tempfile.TemporaryDirectory(prefix="bridget-089-corpus-") as temporary:
        candidates = Path(temporary)
        shutil.copytree(root / CORPUS, candidates / CORPUS)
        verify(root, candidates, manifest)
        entry = manifest["entries"][0]
        path = candidates / CORPUS / "fixtures" / entry["file"]
        raw = path.read_bytes()
        path.write_bytes(bytes([raw[0] ^ 1]) + raw[1:])
        must_refuse("un octet modifié", lambda: verify(root, candidates, manifest))
        changed = json.loads(json.dumps(manifest))
        changed["entries"][0]["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        must_refuse("empreinte réécrite avec l'octet", lambda: verify(root, candidates, changed))
        path.write_bytes(raw)
        changed["entries"][0] = dict(entry, source="specs/089-source-absente.jsonl")
        must_refuse("source Git absente", lambda: verify(root, candidates, changed))
        path.unlink()
        must_refuse("fixture absente", lambda: verify(root, candidates, manifest))
        path.write_bytes(raw)
        changed = dict(manifest, missing_families=[], entries=[
            entry for entry in manifest["entries"] if entry["family"] != "coordination-v2"
        ])
        must_refuse("gel déclaré complet sans les familles", lambda: verify(root, candidates, changed))
    current = git(root, "rev-parse", "HEAD").decode().strip()
    if current == PIN:
        raise ValueError("auto-test nécessite un descendant du commit source")
    # Le commit courant existe mais n'est pas ancêtre de la référence :
    # exerce le contrôle réel de Git, pas seulement une référence introuvable.
    must_refuse("commit existant non ancêtre", lambda: require_ancestor(root, current, PIN))


def main():
    parser = argparse.ArgumentParser(description="Vérifier les bytes épinglés de 089")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--require-complete", action="store_true")
    args = parser.parse_args()
    root = Path(git(Path.cwd(), "rev-parse", "--show-toplevel").decode().strip())
    manifest = json.loads(local_file(root, CORPUS / "manifest.json").read_bytes())
    count = verify(root, root, manifest)
    if args.self_test:
        self_test(root, manifest)
    missing = manifest["missing_families"]
    print(f"PASS intégrité : {count} fichiers identiques au commit {PIN}")
    if missing:
        print("GEL INCOMPLET : " + "; ".join(missing))
        if args.require_complete:
            return 1
    else:
        print("Couverture déclarée complète ; les tests consommateurs restent obligatoires")
    return 0


try:
    sys.exit(main())
except (ValueError, OSError, subprocess.SubprocessError) as error:
    print(f"REFUS corpus 089 : {error}", file=sys.stderr)
    sys.exit(1)
PY
