#!/usr/bin/env python3
"""Ronde Bridget/Maicie en lecture seule : constater, jamais décider."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
from pathlib import Path
from typing import Any

ATTESTED_OUTBOX_STATES = ("accepted", "outcome_unknown", "rejected")


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Ronde passive Bridget/Maicie")
    parser.add_argument("--config", default=str(Path.home() / ".config/maicie/config.json"))
    parser.add_argument("--bridget-bin", default=os.environ.get("BRIDGET_BIN", "bridget"))
    parser.add_argument("--maicie-bin", default=os.environ.get("MAICIE_BIN", "maicie"))
    parser.add_argument("--silent-after-secs", type=int, default=1800)
    parser.add_argument("--exclude", default="bridget,fable,poucave,sol,maicie")
    parser.add_argument("--report-dir", type=Path)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--now", type=int, help="horodatage injecté pour le harnais")
    value = parser.parse_args()
    if value.silent_after_secs < 0:
        parser.error("--silent-after-secs doit être positif ou nul")
    return value


def run_json(command: list[str]) -> tuple[Any | None, str | None]:
    try:
        result = subprocess.run(command, capture_output=True, text=True, check=False)
    except OSError as error:
        return None, str(error)
    if result.returncode:
        return None, ((result.stderr or result.stdout).strip().replace("\n", " ")[:240] or f"sortie {result.returncode}")
    try:
        return json.loads(result.stdout), None
    except json.JSONDecodeError as error:
        return None, f"JSON invalide: {error.msg}"


def run_text(command: list[str]) -> tuple[str | None, str | None]:
    try:
        result = subprocess.run(command, capture_output=True, text=True, check=False)
    except OSError as error:
        return None, str(error)
    if result.returncode:
        return None, ((result.stderr or result.stdout).strip().replace("\n", " ")[:240] or f"sortie {result.returncode}")
    return result.stdout, None


def unavailable(reason: str) -> dict[str, Any]:
    return {"state": "unavailable", "reason": reason}


def read_maicie_read_only(config_path: str) -> tuple[dict[str, Any] | None, str | None]:
    """Lit les index Maicie sur une copie : aucune commande `status` ici.

    `maicie status` réconcilie ses outboxes au démarrage. Une ronde ne doit ni
    émettre ni modifier une source d'autorité. Ouvrir même une base WAL en
    `mode=ro` peut créer/toucher ses sidecars : la base et son WAL sont donc
    copiés dans un répertoire jetable avant toute ouverture SQLite. Le lecteur
    ne peut ainsi écrire que dans cette copie non autoritaire.

    Compteur exposé : `objectives_to_evaluate` = lignes de `objectives` dont
    `state = 'a_evaluer'`. Ce n'est PAS le nombre de délégations `a_evaluer`
    (celles-ci se lisent via `maicie status --json` / greffe). Le résumé texte
    reprend le même sens sous la clé `objectifs_a_evaluer`.
    """
    try:
        with open(config_path, encoding="utf-8") as stream:
            config = json.load(stream)
        source = Path(config["database_path"])
        with tempfile.TemporaryDirectory(prefix="bridget-ronde-maicie-") as directory:
            copied = Path(directory) / source.name
            shutil.copy2(source, copied)
            source_wal = Path(f"{source}-wal")
            if source_wal.exists():
                shutil.copy2(source_wal, Path(f"{copied}-wal"))
            connection = sqlite3.connect(copied)
            try:
                connection.execute("PRAGMA query_only=ON")
                objectives = connection.execute(
                    "SELECT id, state FROM objectives WHERE state = 'a_evaluer' ORDER BY id"
                ).fetchall()
                participants = connection.execute(
                    "SELECT d.payload_json FROM delegations d "
                    "JOIN objectives o ON o.id = d.objective_id "
                    "JOIN delegation_outbox b ON b.delegation_id = d.id "
                    "WHERE o.state = 'en_coordination' AND b.state IN (?, ?, ?) "
                    "ORDER BY d.id"
                    , ATTESTED_OUTBOX_STATES
                ).fetchall()
            finally:
                connection.close()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError, sqlite3.Error) as error:
        return None, str(error)
    active = set()
    for (payload,) in participants:
        try:
            delegation = json.loads(payload)
            participant = delegation.get("participant")
            if isinstance(participant, str):
                active.add(participant)
        except (TypeError, ValueError, json.JSONDecodeError):
            return None, "délégation SQLite invalide"
    return {
        "objectives_to_evaluate": [
            {"objective_id": objective_id, "state": state} for objective_id, state in objectives
        ],
        "objectives_to_evaluate_means": (
            "count of objectives.state=a_evaluer ; not delegation a_evaluer rows"
        ),
        "active_participants": sorted(active),
    }, None


def report(options: argparse.Namespace) -> tuple[str, dict[str, Any]]:
    # `datetime.UTC` n'existe qu'à partir de Python 3.11 ; l'unité launchd
    # utilise le Python système de macOS (3.9 à ce jour).
    utc = dt.timezone.utc
    now = options.now if options.now is not None else int(dt.datetime.now(utc).timestamp())
    observed_at = dt.datetime.fromtimestamp(now, utc).isoformat().replace("+00:00", "Z")
    excluded = {item.strip() for item in options.exclude.split(",") if item.strip()}
    agents_raw, agents_error = run_json([options.bridget_bin, "agents", "--json"])
    requests_raw, requests_error = run_json([options.bridget_bin, "requests", "--all", "--json"])
    maicie_raw, maicie_error = read_maicie_read_only(options.config)
    registry_raw, registry_error = run_text([options.maicie_bin, "registre", "list", "--config", options.config, "--attente"])
    result: dict[str, Any] = {"v": 1, "observed_at": now, "observed_at_iso": observed_at, "decision": "none", "delivery": "none"}

    active: set[str] | None = None
    evaluate: list[dict[str, str]] = []
    if maicie_error:
        result["maicie"] = unavailable(maicie_error)
    else:
        evaluate = maicie_raw["objectives_to_evaluate"]
        active = set(maicie_raw["active_participants"])
        result["maicie"] = {
            "state": "available",
            "objectives_to_evaluate": evaluate,
            "objectives_to_evaluate_means": maicie_raw["objectives_to_evaluate_means"],
            "active_participants": sorted(active),
        }

    if agents_error or not isinstance(agents_raw, list):
        result["agents"] = unavailable(agents_error or "annuaire Bridget invalide")
    else:
        connected = []
        silent = []
        unassigned = []
        for agent in agents_raw:
            if not isinstance(agent, dict):
                continue
            name = agent.get("name")
            if not isinstance(name, str) or name in excluded or agent.get("agent_type") == "mcp" or agent.get("state") != "connected":
                continue
            item = {"name": name, "domain": agent.get("domain"), "last_seen_secs": agent.get("last_seen_secs")}
            connected.append(item)
            if isinstance(item["last_seen_secs"], int) and item["last_seen_secs"] > options.silent_after_secs:
                silent.append(item)
            if active is not None and name not in active:
                unassigned.append(item)
        result["agents"] = {"state": "available", "connected": connected, "unassigned_connected": unassigned if active is not None else None, "silent": silent}

    if requests_error or not isinstance(requests_raw, list):
        result["requests"] = unavailable(requests_error or "projection des demandes invalide")
    else:
        expired = [{"id": request.get("id"), "sender": request.get("sender"), "target": request.get("target"), "deadline_at": request.get("deadline_at")} for request in requests_raw if isinstance(request, dict) and isinstance(request.get("deadline_at"), int) and request["deadline_at"] <= now]
        result["requests"] = {"state": "available", "open_count": len(requests_raw), "expired": expired}
    result["registry"] = unavailable(registry_error) if registry_error else {"state": "available", "view": registry_raw}

    summary = (
        f"RONDE {observed_at} — agents={result['agents']['state']}, maicie={result['maicie']['state']}, "
        f"demandes={result['requests']['state']}, registre={result['registry']['state']}; "
        f"objectifs_a_evaluer={len(evaluate)} (objectifs.state=a_evaluer, pas les délégations), "
        f"demandes_échues={len(result['requests'].get('expired', []))}. "
        f"Constat seulement : aucune décision ni aucun envoi."
    )
    return summary, result


def archive(directory: Path, summary: str, result: dict[str, Any]) -> dict[str, str]:
    directory.mkdir(mode=0o700, parents=True, exist_ok=True)
    stamp = result["observed_at_iso"].replace(":", "-")
    paths = {"text": directory / f"ronde-{stamp}.txt", "json": directory / f"ronde-{stamp}.json"}
    for key, content in (("text", summary + "\n"), ("json", json.dumps(result, ensure_ascii=False, sort_keys=True) + "\n")):
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=directory, prefix=".ronde-", delete=False) as temporary:
            temporary.write(content)
            temporary.flush()
            os.fsync(temporary.fileno())
            temporary_path = Path(temporary.name)
        temporary_path.replace(paths[key])
        paths[key].chmod(0o600)
    return {key: str(path) for key, path in paths.items()}


def main() -> int:
    options = args()
    summary, result = report(options)
    if options.report_dir:
        result["archive"] = archive(options.report_dir, summary, result)
    output = json.dumps(result, ensure_ascii=False, sort_keys=True)
    if options.json:
        print(output)
    else:
        print(summary)
        print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
