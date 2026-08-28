#!/usr/bin/env python3
"""Ronde Bridget/Maicie en lecture seule : constater, jamais décider."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
from pathlib import Path
from typing import Any

ATTESTED_OUTBOX_STATES = ("accepted", "outcome_unknown", "rejected")

VP_CLASSES = ("production", "verification", "instruction", "repair", "indeterminate")
VP_CLASSIFIER_VERSION = "vp-manifest-v1"
VP_BASELINE_START_EXCLUSIVE = 1787788534
VP_BASELINE_END_INCLUSIVE = 1787874934
VP_BASELINE_COUNT = 169
VP_BASELINE_ID_BYTES = 6253
VP_BASELINE_SHA256 = "e2a279624ecd4fbd9ca7f8f97effa45777c15b6b3b9574f3d43a9d90298d5a55"
VP_ROOT_REPUBLISH = "lot:maicie-republish-before-migrate"

# Annotations historiques volontairement petites et réfutables. Elles sont
# issues de la lecture des buts durables, jamais d'un motif lexical ni du champ
# `origin` actuellement trompeur. Tout identifiant absent reste indéterminé.
VP_ANNOTATIONS: dict[str, dict[str, str]] = {
    "17441d4f-244b-41dc-8c0f-a26110a55276": {
        "class": "production",
        "root_id": VP_ROOT_REPUBLISH,
        "basis": "mandat initial de mise en oeuvre de la republication",
    },
    "d105d9c4-8f67-482f-8f53-5f6725229f74": {
        "class": "verification",
        "root_id": VP_ROOT_REPUBLISH,
        "basis": "revue explicite du lot de republication",
    },
    "3344a60b-6f3f-4cd3-ac07-84ab40483cef": {
        "class": "verification",
        "root_id": VP_ROOT_REPUBLISH,
        "basis": "amendement cause par le verdict de revue",
    },
    "f11f889b-4f22-482a-9960-fb45348bdd01": {
        "class": "verification",
        "root_id": VP_ROOT_REPUBLISH,
        "basis": "seconde revue explicite du meme lot",
    },
    "83240941-9a69-4879-8bd7-95b5534c01c3": {
        "class": "verification",
        "root_id": VP_ROOT_REPUBLISH,
        "basis": "correction des charges produites par la revue",
    },
}


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


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def objective_rows(connection: sqlite3.Connection) -> list[dict[str, Any]]:
    rows = connection.execute(
        "SELECT id, payload_json FROM objectives ORDER BY id"
    ).fetchall()
    objectives: list[dict[str, Any]] = []
    for objective_id, payload_json in rows:
        payload = json.loads(payload_json)
        created_at = payload.get("cree_at") if isinstance(payload, dict) else None
        goal = payload.get("but") if isinstance(payload, dict) else None
        if (
            not isinstance(objective_id, str)
            or not isinstance(created_at, int)
            or isinstance(created_at, bool)
        ):
            raise ValueError("objectif sans identifiant ou horodatage exploitable")
        if payload.get("id") != objective_id:
            raise ValueError(f"identifiant divergent dans le payload de {objective_id}")
        if not isinstance(goal, str):
            raise ValueError(f"objectif {objective_id} sans but textuel exploitable")
        objectives.append({"id": objective_id, "created_at": created_at, "goal": goal})
    return objectives


def select_objective_window(
    rows: list[dict[str, Any]], start_exclusive: int, end_inclusive: int
) -> list[dict[str, Any]]:
    return sorted(
        (row for row in rows if start_exclusive < row["created_at"] <= end_inclusive),
        key=lambda row: (row["created_at"], row["id"]),
    )


def ratio_result(
    numerator: int, denominator: int, complete: bool, reason: str
) -> dict[str, Any]:
    if not complete:
        return unavailable(reason)
    if denominator == 0:
        return unavailable("denominator_zero")
    return {
        "state": "available",
        "numerator": numerator,
        "denominator": denominator,
        "value": numerator / denominator,
    }


def measure_objective_population(
    rows: list[dict[str, Any]], window: dict[str, Any]
) -> dict[str, Any]:
    id_bytes = "".join(f"{row['id']}\n" for row in rows).encode("ascii")
    entries: list[dict[str, Any]] = []
    objective_counts = {name: 0 for name in VP_CLASSES}
    roots = {name: set() for name in VP_CLASSES}
    classified = 0
    rooted = 0

    for row in rows:
        annotation = VP_ANNOTATIONS.get(row["id"])
        if annotation is None:
            classification = "indeterminate"
            root_id = None
            basis = "causalite_historique_indisponible"
        else:
            classification = annotation["class"]
            root_id = annotation["root_id"]
            basis = annotation["basis"]
        if classification not in VP_CLASSES:
            raise ValueError(f"classe inconnue pour {row['id']}: {classification}")
        objective_counts[classification] += 1
        if classification != "indeterminate":
            classified += 1
        if root_id is not None:
            rooted += 1
            roots[classification].add(root_id)
        entries.append(
            {
                "objective_id": row["id"],
                "class": classification,
                "root_id": root_id,
                "basis": basis,
                "goal_sha256": sha256_bytes(row["goal"].encode("utf-8")),
            }
        )

    total = len(rows)
    classification_complete = classified == total
    root_complete = rooted == total
    root_counts = {name: len(values) for name, values in roots.items()}
    objective_ratios = {
        "verification_per_production": ratio_result(
            objective_counts["verification"],
            objective_counts["production"],
            classification_complete,
            "classification_coverage_incomplete",
        ),
        "verification_per_production_and_repair": ratio_result(
            objective_counts["verification"],
            objective_counts["production"] + objective_counts["repair"],
            classification_complete,
            "classification_coverage_incomplete",
        ),
    }
    root_measure_complete = classification_complete and root_complete
    root_reason = (
        "classification_coverage_incomplete"
        if not classification_complete
        else "root_coverage_incomplete"
    )
    root_ratios = {
        "verification_per_production": ratio_result(
            root_counts["verification"],
            root_counts["production"],
            root_measure_complete,
            root_reason,
        ),
        "verification_per_production_and_repair": ratio_result(
            root_counts["verification"],
            len(roots["production"] | roots["repair"]),
            root_measure_complete,
            root_reason,
        ),
    }
    expansion = {
        name: (
            objective_counts[name] / root_counts[name] if root_counts[name] else None
        )
        for name in VP_CLASSES
    }
    manifest_bytes = json.dumps(
        entries, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")
    return {
        "window": window,
        "population": {
            "count": total,
            "id_bytes": len(id_bytes),
            "sha256": sha256_bytes(id_bytes),
        },
        "manifest": {"sha256": sha256_bytes(manifest_bytes), "entries": entries},
        "coverage": {
            "classification": {
                "classified": classified,
                "total": total,
                "ratio": classified / total if total else 0.0,
            },
            "root": {
                "rooted": rooted,
                "total": total,
                "ratio": rooted / total if total else 0.0,
            },
        },
        "objectives": {"counts": objective_counts, "ratios": objective_ratios},
        "roots": {
            "counts": root_counts,
            "ratios": root_ratios,
            "expansion_factor": expansion,
        },
    }


def measure_verification_production(
    connection: sqlite3.Connection, now: int
) -> dict[str, Any]:
    rows = objective_rows(connection)
    baseline = measure_objective_population(
        select_objective_window(
            rows, VP_BASELINE_START_EXCLUSIVE, VP_BASELINE_END_INCLUSIVE
        ),
        {
            "kind": "frozen_24h",
            "start": VP_BASELINE_START_EXCLUSIVE,
            "start_inclusive": False,
            "end": VP_BASELINE_END_INCLUSIVE,
            "end_inclusive": True,
            "timezone": "Europe/Paris",
        },
    )
    expected = {
        "count": VP_BASELINE_COUNT,
        "id_bytes": VP_BASELINE_ID_BYTES,
        "sha256": VP_BASELINE_SHA256,
    }
    baseline["reference"] = {
        "expected": expected,
        "matches": baseline["population"] == expected,
    }
    if baseline["reference"]["matches"]:
        baseline["state"] = "available"
    else:
        baseline["state"] = "unavailable"
        baseline["reason"] = "frozen_population_mismatch"
    rolling_start = now - 24 * 60 * 60
    rolling = measure_objective_population(
        select_objective_window(rows, rolling_start, now),
        {
            "kind": "rolling_24h",
            "start": rolling_start,
            "start_inclusive": False,
            "end": now,
            "end_inclusive": True,
            "timezone": "UTC",
        },
    )
    rolling["state"] = "available"
    return {
        "state": "available",
        "version": 1,
        "classifier_version": VP_CLASSIFIER_VERSION,
        "as_of": now,
        "target": None,
        "origin_used": False,
        "populations": {
            "human_baseline_2026_08_27": baseline,
            "rolling_24h": rolling,
        },
    }


def read_maicie_read_only(
    config_path: str, now: int
) -> tuple[dict[str, Any] | None, str | None]:
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
                try:
                    verification_production = measure_verification_production(
                        connection, now
                    )
                except (
                    TypeError,
                    ValueError,
                    json.JSONDecodeError,
                    sqlite3.Error,
                ) as error:
                    verification_production = unavailable(str(error))
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
        "verification_production": verification_production,
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
    maicie_raw, maicie_error = read_maicie_read_only(options.config, now)
    registry_raw, registry_error = run_text([options.maicie_bin, "registre", "list", "--config", options.config, "--attente"])
    result: dict[str, Any] = {"v": 1, "observed_at": now, "observed_at_iso": observed_at, "decision": "none", "delivery": "none"}

    active: set[str] | None = None
    evaluate: list[dict[str, str]] = []
    if maicie_error:
        result["maicie"] = unavailable(maicie_error)
        result["verification_production"] = unavailable(maicie_error)
    else:
        evaluate = maicie_raw["objectives_to_evaluate"]
        active = set(maicie_raw["active_participants"])
        result["maicie"] = {
            "state": "available",
            "objectives_to_evaluate": evaluate,
            "objectives_to_evaluate_means": maicie_raw["objectives_to_evaluate_means"],
            "active_participants": sorted(active),
        }
        result["verification_production"] = maicie_raw["verification_production"]

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

    vp = result["verification_production"]
    if vp["state"] == "available":
        rolling_vp = vp["populations"]["rolling_24h"]
        classified_vp = rolling_vp["coverage"]["classification"]
        rooted_vp = rolling_vp["coverage"]["root"]
        vp_summary = (
            f"vp_classes={classified_vp['classified']}/{classified_vp['total']}, "
            f"vp_racines={rooted_vp['rooted']}/{rooted_vp['total']}"
        )
    else:
        vp_summary = "vp=unavailable"
    summary = (
        f"RONDE {observed_at} — agents={result['agents']['state']}, maicie={result['maicie']['state']}, "
        f"demandes={result['requests']['state']}, registre={result['registry']['state']}; "
        f"objectifs_a_evaluer={len(evaluate)} (objectifs.state=a_evaluer, pas les délégations), "
        f"demandes_échues={len(result['requests'].get('expired', []))}, {vp_summary}. "
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
