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

La ronde expose aussi les références distantes locales non fusionnées. Cette
vue Git est bornée, sans fetch et sans écriture dans le dépôt observé. La
greffe ne portant aucun verdict structuré exploitable, le blocage métier reste
explicitement indéterminé au lieu d'être déduit de messages libres.
"""

from __future__ import annotations

import argparse
import json
import math
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
DEFAULT_GIT_REPO = str(Path(__file__).resolve().parent.parent)
DEFAULT_GIT_TIMEOUT_SECS = 5.0
BRANCH_BACKLOG_UNAVAILABLE = "greffe sans etat exploitable"
BRANCH_REFS_SCOPE = "refs locales sans fetch"
BRANCH_AGE_BASIS = "age du commit de tete uniquement"
BRANCH_DELIVERY_LIMIT = "une ref distante ne prouve pas une livraison"


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
    parser.add_argument("--git-bin", default=os.environ.get("GIT_BIN", "git"))
    parser.add_argument(
        "--git-repo", default=os.environ.get("BRIDGET_REPO", DEFAULT_GIT_REPO)
    )
    parser.add_argument(
        "--git-timeout-secs",
        type=float,
        default=DEFAULT_GIT_TIMEOUT_SECS,
        help="budget total de la vue branches (défaut: 5 secondes)",
    )
    parser.add_argument("--now", type=int, help="horodatage injecté pour le harnais")
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
    if not math.isfinite(value.git_timeout_secs) or value.git_timeout_secs <= 0:
        parser.error("--git-timeout-secs doit être strictement positif")
    if value.now is not None and value.now < 0:
        parser.error("--now doit être positif ou nul")
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


def _git_detail(result: subprocess.CompletedProcess[str]) -> str:
    return (result.stderr or result.stdout).strip().replace("\n", " ")[
        :180
    ] or f"sortie {result.returncode}"


def _run_git(
    repository: Path,
    arguments: list[str],
    *,
    deadline: float,
    git_bin: str,
    extra_env: dict[str, str] | None = None,
) -> tuple[subprocess.CompletedProcess[str] | None, str | None]:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        return None, "delai Git depasse"
    environment = os.environ.copy()
    for variable in (
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    ):
        environment.pop(variable, None)
    environment["GIT_OPTIONAL_LOCKS"] = "0"
    if extra_env:
        environment.update(extra_env)
    try:
        result = subprocess.run(
            [git_bin, *arguments],
            cwd=repository,
            env=environment,
            capture_output=True,
            text=True,
            errors="replace",
            check=False,
            timeout=remaining,
        )
    except subprocess.TimeoutExpired:
        return None, "delai Git depasse"
    except OSError as error:
        return None, f"Git inaccessible: {error}"
    return result, None


def read_branch_backlog(
    repository_path: str,
    *,
    now: int | None = None,
    timeout_secs: float = DEFAULT_GIT_TIMEOUT_SECS,
    git_bin: str = "git",
) -> tuple[dict[str, Any] | None, str | None]:
    """Vue Git locale des branches non fusionnées, bornée et sans mutation.

    Complexité : O(n log n), n = nombre de refs distantes locales ; O(n)
    commandes Git, toutes incluses dans une seule échéance murale.
    """
    if not math.isfinite(timeout_secs) or timeout_secs <= 0:
        return None, "budget Git invalide"
    repository = Path(repository_path).resolve()
    deadline = time.monotonic() + timeout_secs
    stamp = int(time.time()) if now is None else now
    main_ref = "origin/main"
    full_main_ref = "refs/remotes/origin/main"

    main_result, error = _run_git(
        repository,
        ["rev-parse", "--verify", f"{full_main_ref}^{{commit}}"],
        deadline=deadline,
        git_bin=git_bin,
    )
    if error:
        return None, error
    assert main_result is not None
    if main_result.returncode:
        return None, f"{main_ref} indisponible: {_git_detail(main_result)}"
    main_head = main_result.stdout.strip()
    if not main_head:
        return None, f"{main_ref} vide"

    refs_result, error = _run_git(
        repository,
        [
            "for-each-ref",
            "--sort=refname",
            "--format=%(refname)\t%(refname:short)\t%(objectname)\t%(committerdate:unix)\t%(symref)",
            "refs/remotes/origin/",
        ],
        deadline=deadline,
        git_bin=git_bin,
    )
    if error:
        return None, error
    assert refs_result is not None
    if refs_result.returncode:
        return None, f"refs distantes indisponibles: {_git_detail(refs_result)}"

    objects_result, error = _run_git(
        repository,
        ["rev-parse", "--git-path", "objects"],
        deadline=deadline,
        git_bin=git_bin,
    )
    if error:
        return None, error
    assert objects_result is not None
    if objects_result.returncode:
        return None, f"object store indisponible: {_git_detail(objects_result)}"
    objects_path = Path(objects_result.stdout.strip())
    if not objects_path.is_absolute():
        objects_path = repository / objects_path
    objects_path = objects_path.resolve()
    if not objects_path.is_dir():
        return None, f"object store absent: {objects_path}"

    temp_root = Path(tempfile.gettempdir()).resolve()
    try:
        inside_repository = os.path.commonpath(
            [str(temp_root), str(repository)]
        ) == str(repository)
    except ValueError:
        inside_repository = False
    if inside_repository:
        return None, "repertoire temporaire Git situe dans le depot observe"

    parsed_refs: list[tuple[str, str, int]] = []
    for line in refs_result.stdout.splitlines():
        fields = line.split("\t")
        if len(fields) != 5:
            return None, "sortie for-each-ref invalide"
        full_ref, short_ref, head, commit_ts_raw, symref = fields
        if full_ref == full_main_ref or symref:
            continue
        try:
            commit_ts = int(commit_ts_raw)
        except ValueError:
            return None, f"date Git invalide pour {short_ref}"
        parsed_refs.append((short_ref, head, commit_ts))
    if time.monotonic() >= deadline:
        return None, "delai Git depasse"

    lots: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(
        prefix="bridget-idle-merge-", dir=temp_root
    ) as directory:
        temporary_objects = Path(directory) / "objects"
        temporary_objects.mkdir(mode=0o700)
        merge_environment = {
            "GIT_OBJECT_DIRECTORY": str(temporary_objects),
            "GIT_ALTERNATE_OBJECT_DIRECTORIES": str(objects_path),
        }

        for short_ref, head, commit_ts in parsed_refs:
            ancestor_result, error = _run_git(
                repository,
                ["merge-base", "--is-ancestor", head, main_head],
                deadline=deadline,
                git_bin=git_bin,
            )
            if error:
                return None, error
            assert ancestor_result is not None
            if ancestor_result.returncode == 0:
                continue
            if ancestor_result.returncode != 1:
                return (
                    None,
                    f"ascendance invalide pour {short_ref}: {_git_detail(ancestor_result)}",
                )

            base_result, error = _run_git(
                repository,
                ["merge-base", main_head, head],
                deadline=deadline,
                git_bin=git_bin,
            )
            if error:
                return None, error
            assert base_result is not None
            base_sha = base_result.stdout.strip()
            if base_result.returncode == 1 and not base_sha:
                base_state = "perimee"
                blocking = "base perimee, a rebaser"
            elif base_result.returncode:
                return (
                    None,
                    f"base invalide pour {short_ref}: {_git_detail(base_result)}",
                )
            else:
                base_state = "dans_main"
                blocking = f"indetermine — {BRANCH_BACKLOG_UNAVAILABLE}"

            merge_arguments = ["merge-tree", "--write-tree", "--no-messages"]
            if base_state == "perimee":
                merge_arguments.append("--allow-unrelated-histories")
            merge_arguments.extend((main_head, head))
            merge_result, error = _run_git(
                repository,
                merge_arguments,
                deadline=deadline,
                git_bin=git_bin,
                extra_env=merge_environment,
            )
            if error:
                return None, error
            assert merge_result is not None
            if merge_result.returncode == 0:
                textual_merge = "sans conflit textuel"
            elif merge_result.returncode == 1:
                textual_merge = "en conflit"
                if base_state == "dans_main":
                    blocking = "en conflit"
            else:
                return (
                    None,
                    f"merge-tree invalide pour {short_ref}: {_git_detail(merge_result)}",
                )

            lots.append(
                {
                    "ref": short_ref,
                    "head": head,
                    "age_secs": max(0, stamp - commit_ts),
                    "base_state": base_state,
                    "base_sha": base_sha or None,
                    "textual_merge": textual_merge,
                    "verdict_state": "indisponible",
                    "blocking": blocking,
                }
            )

    lots.sort(key=lambda item: (-item["age_secs"], item["ref"]))
    if time.monotonic() >= deadline:
        return None, "delai Git depasse"
    return {
        "state": "partial",
        "reason": BRANCH_BACKLOG_UNAVAILABLE,
        "main_ref": main_ref,
        "main_head": main_head,
        "refs_scope": BRANCH_REFS_SCOPE,
        "age_basis": BRANCH_AGE_BASIS,
        "delivery_limit": BRANCH_DELIVERY_LIMIT,
        "timeout_secs": timeout_secs,
        "lots": lots,
    }, None


def _format_age(seconds: int) -> str:
    if seconds < 3600:
        return f"{seconds // 60}min"
    if seconds < 48 * 3600:
        return f"{seconds // 3600}h"
    return f"{seconds // 86400}j"


def format_branch_backlog(
    branch_backlog: dict[str, Any] | None, *, branch_error: str | None
) -> str:
    lines: list[str] = []
    if branch_error or branch_backlog is None:
        lines.append(
            f"BACKLOG BRANCHES INDISPONIBLE ({(branch_error or 'inconnu')[:120]})"
        )
    else:
        lines.append(f"BACKLOG BRANCHES INDISPONIBLE ({branch_backlog['reason']})")
        lots = branch_backlog["lots"]
        if lots:
            lines.append("LOTS LIVRES NON MERGES :")
            for lot in lots:
                if lot["base_state"] == "dans_main":
                    base = f"base {lot['base_sha'][:12]} dans main"
                else:
                    base = "base perimee"
                lines.append(
                    f"  {lot['ref']} {_format_age(lot['age_secs'])} | {base} | "
                    f"{lot['textual_merge']} | verdict indisponible | {lot['blocking']}"
                )
        else:
            lines.append("LOTS LIVRES NON MERGES : aucun dans les refs locales")
    lines.append(
        "LIMITES BRANCHES : refs locales sans fetch ; age du commit de tete uniquement ; "
        "une ref distante ne prouve pas une livraison"
    )
    return "\n".join(lines)


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
    # Énoncé volontairement littéral : l'âge est celui de la plus vieille
    # remise encore en file, PAS « bloqué depuis ». Un agent qui repart après
    # déblocage garde son ancienneté tant que cette remise n'est pas consommée
    # (mesuré 2026-08-25, jc2 : travaille, compteur 50 min — le signal était
    # vrai, la lecture « est bloqué depuis » mentait).
    lines.append(
        "BLOQUES (plus vieille remise non consommee) : "
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
        backlog_ages, backlog_error = read_backlog_ages_from_bridget_copy(
            options.bridget_db, now=options.now
        )
        if backlog_ages is None:
            backlog_error = backlog_error or "indisponible"
            backlog_ages = {}

    try:
        branch_backlog, branch_error = read_branch_backlog(
            options.git_repo,
            now=options.now,
            timeout_secs=options.git_timeout_secs,
            git_bin=options.git_bin,
        )
    except OSError as error:
        branch_backlog = None
        branch_error = f"temporaire Git indisponible: {error}"

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
            "branch_backlog": (
                branch_backlog
                if branch_backlog is not None
                else {
                    "state": "unavailable",
                    "reason": branch_error or "inconnu",
                    "refs_scope": BRANCH_REFS_SCOPE,
                    "age_basis": BRANCH_AGE_BASIS,
                    "delivery_limit": BRANCH_DELIVERY_LIMIT,
                }
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
        text += "\n" + format_branch_backlog(branch_backlog, branch_error=branch_error)
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
