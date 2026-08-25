#!/usr/bin/env python3
"""Agents daemon classés pour la ronde : partition complète, jamais d'omission.

Écrit le 2026-08-24 pour remplacer le croisement à l'œil. Corrigé le 2026-08-25 :
un agent vu par le daemon qui n'entrait dans aucune branche (busy hors greffe,
domaine hors allowlist) disparaissait — « LIBRES : aucun » mentait. Tout agent
du daemon appartient désormais à exactement une catégorie ; l'inclassable va
dans INDETERMINES avec la raison, jamais dans le silence.

Propriété mécanique (BLOQUÉS) : un agent connecté peut cesser de consommer ses
remises sans que la présence (heartbeat) ni la mission greffe ne le signalent.
Signal : âge de la plus vieille remise `send_deliveries.phase=dispatching`
jointe au ledger (copie seule).
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any


DEFAULT_EXCLUDE = "bridget,fable,poucave,sol,maicie"
# Domaines explicitement hors flotte Bridget (autres projets). Un domaine
# perdu (None) ou un domaine de lot (`*-lot`) reste dans le périmètre LIBRES.
EXTERNAL_DOMAINS = frozenset({"46.Thunderbridge", "30.infra", "moi"})

# Coupure BLOQUÉS — mesurée, pas choisie.
# Snapshot live 2026-08-25 ~09:05 (copie ~/.cache/bridget/bridget.db) :
#   max âges ledger-join = 6.2 / 13.3 / 14.6 min → PAS d'écart franc
#   (les deux figés du matin étaient déjà débloqués).
# Séparation mesurée le matin même (même requête SQL sur copie) :
#   sains 2–7 min · figés 44 min et >60 min. Trou franc = [7, 44].
# Coupure = milieu du trou observé : (7+44)/2 ≈ 25.5 → 25 min = 1500 s.
# Une compile de ~10 min reste sous le seuil (exigence anti fausse alerte).
BLOCKED_AFTER_SECS = 1500
DEFAULT_BRIDGET_DB = str(Path.home() / ".cache/bridget/bridget.db")


def args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Partition des agents daemon "
            "(libres / muets / occupés / bloqués / indéterminés / morts)"
        )
    )
    parser.add_argument("--config", default=str(Path.home() / ".config/maicie/config.json"))
    parser.add_argument("--bridget-bin", default=os.environ.get("BRIDGET_BIN", "bridget"))
    parser.add_argument("--bridget-db", default=os.environ.get("BRIDGET_DB", DEFAULT_BRIDGET_DB))
    parser.add_argument("--silent-after-secs", type=int, default=1800)
    parser.add_argument("--blocked-after-secs", type=int, default=BLOCKED_AFTER_SECS)
    parser.add_argument("--exclude", default=DEFAULT_EXCLUDE)
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--agents-json",
        help="fixture agents (harnais) ; sinon `bridget agents --json`",
    )
    parser.add_argument(
        "--occupied-json",
        help="fixture participants occupés (harnais) ; sinon lecture copie Maicie",
    )
    parser.add_argument(
        "--backlog-json",
        help="fixture {agent: age_secs} remises non accusées (harnais)",
    )
    value = parser.parse_args()
    if value.silent_after_secs < 0:
        parser.error("--silent-after-secs doit être positif ou nul")
    if value.blocked_after_secs < 0:
        parser.error("--blocked-after-secs doit être positif ou nul")
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


def read_occupied_from_maicie_copy(config_path: str) -> tuple[set[str] | None, str | None]:
    """Participants en coordination, lus sur une COPIE de la base (jamais la prod)."""
    try:
        with open(config_path, encoding="utf-8") as stream:
            config = json.load(stream)
        source = Path(config["database_path"])
        with tempfile.TemporaryDirectory(prefix="bridget-idle-maicie-") as directory:
            copied = Path(directory) / source.name
            shutil.copy2(source, copied)
            source_wal = Path(f"{source}-wal")
            if source_wal.exists():
                shutil.copy2(source_wal, Path(f"{copied}-wal"))
            connection = sqlite3.connect(copied)
            try:
                connection.execute("PRAGMA query_only=ON")
                rows = connection.execute(
                    "SELECT d.payload_json FROM delegations d JOIN objectives o ON o.id = d.objective_id "
                    "WHERE o.state = 'en_coordination' ORDER BY d.id"
                ).fetchall()
            finally:
                connection.close()
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError, sqlite3.Error) as error:
        return None, str(error)
    occupied: set[str] = set()
    for (payload,) in rows:
        try:
            delegation = json.loads(payload)
            participant = delegation.get("participant")
            if isinstance(participant, str):
                occupied.add(participant)
        except (TypeError, ValueError, json.JSONDecodeError):
            return None, "délégation SQLite invalide"
    return occupied, None


def read_backlog_ages_from_bridget_copy(
    database_path: str, *, now: int | None = None
) -> tuple[dict[str, int] | None, str | None]:
    """Âge (secs) de la plus vieille remise non accusée, par cible — COPIE seule.

    Jointure ledger obligatoire : sans `ledger.ts` l'âge n'est pas fiable
    (les `dispatching` orphelins sans ligne ledger donnent des âges fantômes
    via expires_at). Requête = celle qui a séparé sains/figés le 2026-08-25.
    """
    try:
        source = Path(database_path)
        with tempfile.TemporaryDirectory(prefix="bridget-idle-db-") as directory:
            copied = Path(directory) / source.name
            shutil.copy2(source, copied)
            for suffix in ("-wal", "-shm"):
                side = Path(f"{source}{suffix}")
                if side.exists():
                    shutil.copy2(side, Path(f"{copied}{suffix}"))
            connection = sqlite3.connect(copied)
            try:
                connection.execute("PRAGMA query_only=ON")
                stamp = int(time.time()) if now is None else now
                rows = connection.execute(
                    "SELECT l.target, MAX(? - l.ts) "
                    "FROM send_deliveries d "
                    "JOIN ledger l ON l.id = d.idempotency_key "
                    "WHERE d.phase = 'dispatching' "
                    "GROUP BY l.target",
                    (stamp,),
                ).fetchall()
            finally:
                connection.close()
    except (OSError, TypeError, ValueError, sqlite3.Error) as error:
        return None, str(error)
    ages: dict[str, int] = {}
    for target, age in rows:
        if isinstance(target, str) and isinstance(age, int):
            ages[target] = age
    return ages, None


def domain_in_fleet_scope(domain: Any) -> bool:
    if domain in ("bridget", None):
        return True
    if isinstance(domain, str) and domain.endswith("-lot"):
        return True
    if domain in EXTERNAL_DOMAINS:
        return False
    # Domaine inconnu : pas LIBRE silencieux — l'appelant mettra INDETERMINE.
    return False


def classify(
    agents: list[dict[str, Any]],
    occupied: set[str],
    *,
    exclude: set[str],
    silent_after_secs: int,
    backlog_ages: dict[str, int] | None = None,
    blocked_after_secs: int = BLOCKED_AFTER_SECS,
) -> dict[str, Any]:
    """Partition complète des agents vus par le daemon + morts hors daemon.

    Propriété : libres ∪ muets ∪ occupes ∪ bloques ∪ indetermines = noms daemon,
    sans recouvrement. Les morts = occupied − daemon (hors partition daemon).

    BLOQUÉS prime sur les autres catégories daemon : présence + mission ne
    suffisent pas si les remises ne sont plus consommées.
    """
    by_name: dict[str, dict[str, Any]] = {}
    for agent in agents:
        if not isinstance(agent, dict):
            continue
        name = agent.get("name")
        if not isinstance(name, str) or not name:
            continue
        by_name[name] = agent

    daemon_names = set(by_name)
    backlog_ages = backlog_ages or {}
    libres: list[tuple[str, int]] = []
    muets: list[tuple[str, int]] = []
    occupes: list[str] = []
    bloques: list[tuple[str, int]] = []
    indetermines: list[tuple[str, str]] = []

    for name, agent in sorted(by_name.items()):
        age = backlog_ages.get(name)
        if isinstance(age, int) and age > blocked_after_secs:
            bloques.append((name, age))
            continue

        if name in occupied:
            occupes.append(name)
            continue

        reasons: list[str] = []
        if name in exclude:
            reasons.append("hors-perimetre")
        if agent.get("agent_type") == "mcp":
            reasons.append("type=mcp")

        state = agent.get("state")
        domain = agent.get("domain")
        last_seen = agent.get("last_seen_secs", 0)
        if not isinstance(last_seen, int):
            last_seen = 0

        if state == "busy":
            reasons.append("busy-sans-mission-greffe")
        elif state != "connected":
            reasons.append(f"state={state!r}")

        if not domain_in_fleet_scope(domain):
            reasons.append(f"domaine={domain!r}")

        if reasons:
            seen_r: set[str] = set()
            ordered: list[str] = []
            for reason in reasons:
                if reason not in seen_r:
                    seen_r.add(reason)
                    ordered.append(reason)
            indetermines.append((name, "+".join(ordered)))
            continue

        if last_seen > silent_after_secs:
            muets.append((name, last_seen))
        else:
            libres.append((name, last_seen))

    morts = sorted(occupied - daemon_names)
    return {
        "daemon_count": len(daemon_names),
        "libres": libres,
        "muets": muets,
        "occupes": occupes,
        "bloques": bloques,
        "indetermines": indetermines,
        "morts": morts,
        "maicie_occupied_count": len(occupied),
        "blocked_after_secs": blocked_after_secs,
    }


def partition_oracle(result: dict[str, Any], daemon_names: set[str]) -> tuple[bool, str]:
    """L'oracle voit l'omission : somme des catégories daemon == |daemon|, sans recouvrement."""
    buckets = {
        "libres": {name for name, _ in result["libres"]},
        "muets": {name for name, _ in result["muets"]},
        "occupes": set(result["occupes"]),
        "bloques": {name for name, _ in result.get("bloques", [])},
        "indetermines": {name for name, _ in result["indetermines"]},
    }
    covered: set[str] = set()
    for label, names in buckets.items():
        overlap = covered & names
        if overlap:
            return False, f"recouvrement dans {label}: {sorted(overlap)}"
        covered |= names

    missing = daemon_names - covered
    if missing:
        return False, f"omission: {sorted(missing)}"
    extra = covered - daemon_names
    if extra:
        return False, f"hors-daemon dans partition: {sorted(extra)}"
    if len(covered) != len(daemon_names):
        return False, f"cardinal {len(covered)} != daemon {len(daemon_names)}"
    for dead in result.get("morts", []):
        if dead in daemon_names:
            return False, f"mort encore présent au daemon: {dead}"
    return True, f"partition ok ({len(daemon_names)} agents)"


def classify_legacy(
    agents: list[dict[str, Any]],
    occupied: set[str],
    *,
    exclude: set[str],
    silent_after_secs: int,
) -> dict[str, Any]:
    """Ancienne logique (2026-08-24) — volontairement buggy, pour le contrôle positif."""
    libres: list[tuple[str, int]] = []
    muets: list[tuple[str, int]] = []
    for agent in agents:
        name = agent.get("name")
        if not isinstance(name, str):
            continue
        if name in exclude or agent.get("agent_type") == "mcp":
            continue
        if agent.get("state") != "connected":
            continue
        if agent.get("domain") not in ("bridget", None):
            continue
        last_seen = agent.get("last_seen_secs", 0)
        if not isinstance(last_seen, int):
            last_seen = 0
        if name not in occupied:
            (muets if last_seen > silent_after_secs else libres).append((name, last_seen))
    presents = {a["name"] for a in agents if isinstance(a.get("name"), str)}
    # Ancien affichage : OCCUPES = tout le greffe (recouvre MORTS) ; pas d'indéterminés.
    return {
        "daemon_count": len(presents),
        "libres": libres,
        "muets": muets,
        "occupes": sorted(occupied),
        "bloques": [],
        "indetermines": [],
        "morts": sorted(occupied - presents),
        "maicie_occupied_count": len(occupied),
    }


def format_text(result: dict[str, Any], *, maicie_error: str | None) -> str:
    lines: list[str] = []
    if maicie_error:
        lines.append(f"MAICIE INDISPONIBLE ({maicie_error[:60]}) — vue agents seule, missions inconnues")
    lines.append(
        "LIBRES (vivants, sans mission) : "
        + (", ".join(name for name, _ in result["libres"]) or "aucun")
    )
    lines.append(
        "MUETS (>30 min sans signal)    : "
        + (", ".join(f"{name} {secs // 60}min" for name, secs in result["muets"]) or "aucun")
    )
    lines.append(
        "OCCUPES                        : " + (", ".join(result["occupes"]) or "aucun")
    )
    lines.append(
        "BLOQUES (remises non consommees): "
        + (
            ", ".join(f"{name} {secs // 60}min" for name, secs in result["bloques"])
            or "aucun"
        )
    )
    if result["indetermines"]:
        lines.append(
            "INDETERMINES                  : "
            + ", ".join(f"{name} ({reason})" for name, reason in result["indetermines"])
        )
    else:
        lines.append("INDETERMINES                  : aucun")
    if result["morts"]:
        lines.append(
            "MORTS (mission active, ABSENTS du daemon) : " + ", ".join(result["morts"])
        )
    return "\n".join(lines)


def main() -> int:
    options = args()
    exclude = {item.strip() for item in options.exclude.split(",") if item.strip()}
    maicie_error: str | None = None
    backlog_error: str | None = None

    if options.agents_json:
        agents = json.loads(Path(options.agents_json).read_text(encoding="utf-8"))
    else:
        agents, agents_error = run_json([options.bridget_bin, "agents", "--json"])
        if agents_error or not isinstance(agents, list):
            print(f"annuaire Bridget indisponible: {agents_error or 'payload invalide'}", flush=True)
            return 1

    if options.occupied_json:
        occupied = set(json.loads(Path(options.occupied_json).read_text(encoding="utf-8")))
    else:
        occupied, maicie_error = read_occupied_from_maicie_copy(options.config)
        if occupied is None:
            maicie_error = maicie_error or "indisponible"
            occupied = set()

    if options.backlog_json:
        raw = json.loads(Path(options.backlog_json).read_text(encoding="utf-8"))
        backlog_ages = {str(k): int(v) for k, v in raw.items()}
    else:
        backlog_ages, backlog_error = read_backlog_ages_from_bridget_copy(options.bridget_db)
        if backlog_ages is None:
            backlog_error = backlog_error or "indisponible"
            backlog_ages = {}

    assert isinstance(agents, list)
    typed_agents = [a for a in agents if isinstance(a, dict)]
    result = classify(
        typed_agents,
        occupied,
        exclude=exclude,
        silent_after_secs=options.silent_after_secs,
        backlog_ages=backlog_ages,
        blocked_after_secs=options.blocked_after_secs,
    )
    daemon_names = {a["name"] for a in typed_agents if isinstance(a.get("name"), str)}
    ok, detail = partition_oracle(result, daemon_names)
    if not ok:
        print(f"ORACLE PARTITION ROUGE: {detail}", flush=True)
        return 2

    if options.json:
        payload = {
            "v": 1,
            "libres": [{"name": n, "last_seen_secs": s} for n, s in result["libres"]],
            "muets": [{"name": n, "last_seen_secs": s} for n, s in result["muets"]],
            "occupes": result["occupes"],
            "bloques": [{"name": n, "oldest_unacked_secs": s} for n, s in result["bloques"]],
            "indetermines": [{"name": n, "reason": r} for n, r in result["indetermines"]],
            "morts": result["morts"],
            "daemon_count": result["daemon_count"],
            "blocked_after_secs": result["blocked_after_secs"],
            "partition": detail,
            "maicie": (
                {"state": "unavailable", "reason": maicie_error}
                if maicie_error
                else {"state": "available", "occupied_count": result["maicie_occupied_count"]}
            ),
            "backlog": (
                {"state": "unavailable", "reason": backlog_error}
                if backlog_error
                else {"state": "available", "targets": len(backlog_ages)}
            ),
        }
        print(json.dumps(payload, ensure_ascii=False, sort_keys=True))
    else:
        text = format_text(result, maicie_error=maicie_error)
        if backlog_error:
            text = (
                f"BACKLOG INDISPONIBLE ({backlog_error[:60]}) — BLOQUES non calculables\n"
                + text
            )
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
