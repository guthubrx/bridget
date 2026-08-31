#!/usr/bin/env python3
"""Déclenche une occurrence de la ronde globale via le contrat Bridget local."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
import time
from pathlib import Path

INTERVAL_SECS = 7 * 60


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(
        description="Déclenche la ronde uniquement pour les projets activés."
    )
    result.add_argument(
        "--bridget-bin",
        default=shutil.which("bridget") or "bridget",
        help="binaire Bridget à invoquer",
    )
    result.add_argument("--now", type=int, help="instant Unix de test")
    result.add_argument("--json", action="store_true", help="sortie machine")
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    now = int(time.time()) if args.now is None else args.now
    if now < 0:
        parser().error("--now doit être positif ou nul")
    occurrence_at = now - (now % INTERVAL_SECS)
    binary = Path(args.bridget_bin)
    if binary.is_absolute() and (not binary.is_file() or not binary.stat().st_mode & 0o111):
        parser().error("--bridget-bin doit désigner un fichier exécutable")

    completed = subprocess.run(
        [
            args.bridget_bin,
            "project-round",
            "dispatch",
            "--occurrence",
            str(occurrence_at),
            "--json",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stderr)
        return completed.returncode
    try:
        outcomes = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        print(f"sortie Bridget invalide: {error}", file=sys.stderr)
        return 1
    if not isinstance(outcomes, list):
        print("sortie Bridget invalide: une liste est attendue", file=sys.stderr)
        return 1

    if args.json:
        print(json.dumps(outcomes, ensure_ascii=False, separators=(",", ":")))
    elif outcomes:
        print(
            f"Ronde {occurrence_at}: {len(outcomes)} projet(s) traité(s)."
        )
    else:
        print(f"Ronde {occurrence_at}: aucun projet activé.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
