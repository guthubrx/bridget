#!/usr/bin/env python3
"""Minimal ledger for controlled agentic loops."""

from __future__ import annotations

import argparse
import fcntl
import html
import hashlib
import importlib.util
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import re
from shutil import which as shutil_which
import shlex
import subprocess
import uuid
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any


DEFAULT_ROOT = Path(
    "/Users/moi/Nextcloud/10.Scripts/44.arbor-scaffolding-poc/"
    "harness-lab/orchestration/runs"
)
BRIDGE = Path("/Users/moi/.codex/skills/agent-bridge/scripts/bridge.sh")
BRIDGET = Path(
    os.environ.get(
        "AGENT_LOOP_BRIDGET_BIN",
        "/Users/moi/.local/bin/bridget",
    )
)
BRIDGE_BACKGROUND_ROLE = "agent-loop"
BRIDGE_BACKGROUND_SUBJECTS = {
    "dispatch": "agent-loop-dispatch",
    "heartbeat": "agent-loop-heartbeat",
}
EXACT_TMUX_TARGET = re.compile(r"^(%\d+|[^:\s]+:\d+\.\d+)$")
ROLE_RECOVERY_PROJECT_ROOT = Path(
    "/Users/moi/Nextcloud/10.Scripts/00.Generic/MAICompany"
)
ROLE_RECOVERY_COORDINATION_ROOT = Path("/Users/moi/Documents/MAICompany/coordination")
ROLE_RECOVERY_POLICY_VERSION = "role-recovery-policy-v1"
ROLE_RECOVERY_TERMINAL_RUN_STATUSES = {
    "completed", "failed", "pass", "fail", "blocked", "review", "cancelled", "superseded"
}
ROLE_RECOVERY_FORBIDDEN_PATH_PARTS = {"fixtures", "tests", "specs", "examples"}
RUNTIME_BINDING_MODULE = Path(
    os.environ.get(
        "MAICOMPANY_WATCHDOG_RUNTIME_ROOT",
        "/Users/moi/Documents/MAICompany/worktrees/integration-103-main",
    )
) / "00-company" / "binding_payload.py"
BOOT_ROLE_BINDINGS_PATH = Path(
    os.environ.get(
        "MAICOMPANY_BOOT_ROLE_BINDING",
        "/Users/moi/Documents/MAICompany/coordination/watchdog/boot-role-bindings.json",
    )
)
BOOT_CYCLE_PATTERN = re.compile(r"^boot-[a-f0-9]{32}$")
MISSION_PROGRESS_KINDS = frozenset({
    "worker_ack",
    "task_assigned",
    "source_changed",
    "result_checked",
    "dependency_handoff",
    "blocker_reported",
})
MISSION_EVIDENCE_PREFIXES = {
    "worker_ack": "ack:",
    "task_assigned": "task:",
    "source_changed": "file:",
    "result_checked": "result:",
    "dependency_handoff": "handoff:",
    "blocker_reported": "blocker:",
}
MISSION_DISPOSITIONS = frozenset({
    "accepted",
    "correction_assigned",
    "next_task_assigned",
    "blocked",
})
MISSION_REASON_CODES = frozenset({
    "accepted_in_scope",
    "correction_required",
    "next_work_assigned",
    "external_blocker",
})
WORK_KINDS = frozenset({"generic", "code", "data", "execution_authorization"})


def canonical_boot_context(*, agent: str, pane: str, cli: str) -> dict[str, str]:
    """Résout le contexte boot exact d'une cible depuis le registre durable.

    Le dispatcher et un LaunchAgent ne disposent pas nécessairement des
    variables tmux/MAICOMPANY héritées du pane. Elles ne sont donc qu'un
    contrôle de cohérence : le registre de boot confirmé reste la source de
    vérité. Toute absence, ambiguïté ou contradiction est refusée avant la
    publication d'un binding runtime.
    """
    path = BOOT_ROLE_BINDINGS_PATH
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RuntimeError(f"registre boot illisible: {path}") from exc
    boot_cycle = str(document.get("boot_id") or "")
    entries = document.get("entries")
    if not BOOT_CYCLE_PATTERN.fullmatch(boot_cycle) or not isinstance(entries, list):
        raise RuntimeError("registre boot invalide")
    matches = [
        entry for entry in entries
        if isinstance(entry, dict)
        and str(entry.get("agent_name") or "") == agent
        and str(entry.get("pane_id") or "") == pane
    ]
    if len(matches) != 1:
        raise RuntimeError(f"registre boot ambigu ou absent pour {agent}@{pane}")
    entry = matches[0]
    boot_invocation_id = str(entry.get("invocation_id") or "")
    if (
        str(entry.get("role_status") or "") != "confirmed"
        or str(entry.get("cli") or "") != cli
        or str(entry.get("boot_cycle") or "") != boot_cycle
        or boot_invocation_id != f"{boot_cycle}:{agent}:{pane}"
    ):
        raise RuntimeError(f"registre boot contradictoire pour {agent}@{pane}")
    observed_boot_cycle = str(os.environ.get("MAICOMPANY_BOOT_ID") or "")
    if observed_boot_cycle and observed_boot_cycle != boot_cycle:
        raise RuntimeError("MAICOMPANY_BOOT_ID contradictoire avec le registre boot")
    # L'invocation du processus qui *dispatch* est celle du coordinateur et
    # diffère normalement de celle du worker cible. Ne jamais la propager :
    # l'invocation publiée est toujours celle de l'entrée cible confirmée.
    # Le registre de boot garde le pane tmux littéral (%365), alors que le
    # contrat binding-payload-v2 interdit ce caractère. La projection pane365
    # est déterministe et n'est acceptée qu'après vérification de la valeur
    # brute canonique ci-dessus.
    invocation_id = f"{boot_cycle}:{agent}:pane{pane.lstrip('%')}"
    return {
        "boot_cycle": boot_cycle,
        "invocation_id": invocation_id,
        "source_ref": str(path.resolve()),
    }


def active_existing_tmux_binding(task: dict[str, Any]) -> tuple[bool, str, dict[str, str]]:
    """Valide la cible d'un dispatch tmux contre le registre boot actif.

    Le registre est la seule preuve durable autorisant le ciblage d'un pane
    existant. L'absence, une entrée incohérente ou plusieurs entrées actives
    refusent donc le dispatch avant toute résolution bridge ou transport.
    """
    target = str(task.get("assigned_agent") or task.get("agent_target") or "").strip()
    if not target:
        return False, "target_out_of_binding", {}
    try:
        document = json.loads(BOOT_ROLE_BINDINGS_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False, "target_out_of_binding", {}
    boot_cycle = str(document.get("boot_id") or "")
    entries = document.get("entries")
    if not BOOT_CYCLE_PATTERN.fullmatch(boot_cycle) or not isinstance(entries, list):
        return False, "target_out_of_binding", {}

    target_alias = normalize_agent_alias(target)
    candidates: list[dict[str, str]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        agent = str(entry.get("agent_name") or "").strip()
        pane = str(entry.get("pane_id") or "").strip()
        cli = str(entry.get("cli") or "").strip().lower()
        invocation_id = str(entry.get("invocation_id") or "").strip()
        if target_alias not in {normalize_agent_alias(agent), normalize_agent_alias(pane)}:
            continue
        if (
            str(entry.get("role_status") or "") != "confirmed"
            or not agent
            or not re.fullmatch(r"%\d+", pane)
            or cli not in {"codex", "claude"}
            or str(entry.get("boot_cycle") or "") != boot_cycle
            or invocation_id != f"{boot_cycle}:{agent}:{pane}"
        ):
            continue
        candidates.append(
            {
                "agent_name": agent,
                "pane_id": pane,
                "cli": cli,
                "boot_cycle": boot_cycle,
                "source_ref": str(BOOT_ROLE_BINDINGS_PATH.resolve()),
            }
        )
    if len(candidates) == 1:
        return True, "", candidates[0]
    if len(candidates) > 1:
        return False, "target_ambiguous", {}
    return False, "target_out_of_binding", {}


def runtime_binding_producer() -> Any:
    """Charge le producteur v2 partagé; aucun payload local divergent."""
    if not RUNTIME_BINDING_MODULE.is_file():
        raise RuntimeError(f"producteur binding absent: {RUNTIME_BINDING_MODULE}")
    spec = importlib.util.spec_from_file_location("mai_runtime_binding_payload", RUNTIME_BINDING_MODULE)
    if spec is None or spec.loader is None:
        raise RuntimeError("producteur binding non chargeable")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def iso_after_seconds(seconds: int, *, base: str | None = None) -> str:
    origin = parse_ts(base) if base else datetime.now(timezone.utc)
    if origin is None:
        origin = datetime.now(timezone.utc)
    return (origin + timedelta(seconds=max(0, seconds))).isoformat().replace("+00:00", "Z")


def input_snapshot(task: dict[str, Any]) -> dict[str, str]:
    """Capture les octets des inputs au dispatch pour invalider une revue dérivée."""
    snapshot: dict[str, str] = {}
    for raw in task.get("input_paths", []) or []:
        path = Path(str(raw))
        if not path.is_file():
            snapshot[str(path)] = "missing"
            continue
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                digest.update(chunk)
        snapshot[str(path)] = digest.hexdigest()
    return snapshot


def input_snapshot_changed(task: dict[str, Any]) -> list[str]:
    expected = task.get("input_sha256") or {}
    if not expected:
        return []
    actual = input_snapshot(task)
    return sorted(path for path, digest in expected.items() if actual.get(path) != digest)


def progress_snapshot(task: dict[str, Any]) -> dict[str, str]:
    return input_snapshot({"input_paths": list(dict.fromkeys(
        (task.get("input_paths") or []) + (task.get("output_paths") or [])
    ))})


def result_identity(task: dict[str, Any], result: dict[str, Any]) -> tuple[str, str, str]:
    """Identité stable d'une livraison et de son reçu de continuation."""
    task_id = str(task.get("task_id") or "")
    attempt_id = str(task.get("attempt_id") or task.get("attempts") or "1")
    result_hash = hashlib.sha256(
        json.dumps(result, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    ).hexdigest()
    result_id = hashlib.sha256(f"mai-result-v1:{task_id}:{attempt_id}:{result_hash}".encode("utf-8")).hexdigest()
    idempotency_key = f"result-continuation:{task_id}:{attempt_id}:{result_id}"
    return result_id, result_hash, idempotency_key


def write_continuation_receipt(rd: Path, task: dict[str, Any], result: dict[str, Any]) -> dict[str, Any]:
    result_id, result_hash, idempotency_key = result_identity(task, result)
    receipt = {
        "schema_version": "mai-result-continuation-v1",
        "result_id": result_id,
        "task_id": str(task.get("task_id") or ""),
        "attempt_id": str(task.get("attempt_id") or task.get("attempts") or "1"),
        "result_hash": result_hash,
        "dispatch_epoch": int(task.get("dispatch_epoch") or task.get("attempts") or 0),
        "binding_generation": str(task.get("binding_generation") or "unknown"),
        "fencing_epoch": int(task.get("fencing_epoch") or task.get("lease_epoch") or 0),
        "state": "open",
        "owner": str(task.get("assigned_agent") or task.get("agent_target") or "unknown"),
        "due_at": str(task.get("next_action_due") or task.get("deadline") or now()),
        "created_by_tick": now(),
        "idempotency_key": idempotency_key,
    }
    write_json(rd / "receipts" / f"{result_id}.json", receipt)
    return receipt


def process_exists(pid: Any) -> bool:
    try:
        value = int(pid)
        if value <= 0:
            return False
        os.kill(value, 0)
        probe = subprocess.run(["ps", "-p", str(value), "-o", "stat="], capture_output=True, text=True)
        if not probe.stdout.strip() or "Z" in probe.stdout.strip():
            return False
        return True
    except (TypeError, ValueError, OSError):
        return False


def pane_worker_active(pane: str) -> bool | None:
    """Vérifie qu'un pane existing_tmux exécute encore un worker CLI."""
    if not pane:
        return None
    try:
        out = subprocess.run(
            ["tmux", "display-message", "-p", "-t", pane, "#{pane_current_command}"],
            capture_output=True, text=True, timeout=2, check=False,
        ).stdout.strip().lower()
    except (OSError, subprocess.TimeoutExpired):
        return None
    if not out:
        return None
    if not any(token in out for token in ("codex", "claude", "gemini", "python")):
        return False
    # Un pane Codex revenu au prompt après ACK n'est plus une exécution active.
    try:
        screen = subprocess.run(
            ["tmux", "capture-pane", "-p", "-t", pane, "-S", "-20"],
            capture_output=True, text=True, timeout=2, check=False,
        ).stdout
    except (OSError, subprocess.TimeoutExpired):
        return None
    return "Working" in screen or "working" in screen


def bridge_background_environment(emission: str) -> dict[str, str]:
    """Déclare l'émetteur agent-loop au bridge sans élargir ses cibles.

    La cible reste résolue par bridge.sh puis contrôlée contre la capacité
    signée exacte. Cette fonction ne porte donc aucune autorisation de cible.
    """
    subject = BRIDGE_BACKGROUND_SUBJECTS.get(emission)
    if subject is None:
        raise ValueError(f"unknown bridge background emission: {emission}")
    env = os.environ.copy()
    # L'orchestrateur peut être lancé depuis un pane, mais ses émissions sont
    # bien des processus background. Éviter que le contexte hérité contourne
    # le garde de capacité ou revendique l'identité du pane coordinateur.
    env.pop("TMUX_PANE", None)
    env["BRIDGE_BACKGROUND_SUBJECT"] = subject
    env["BRIDGE_BACKGROUND_ROLE"] = BRIDGE_BACKGROUND_ROLE
    env["BRIDGE_FROM"] = subject
    env["BRIDGE_ROLE"] = BRIDGE_BACKGROUND_ROLE
    return env


def bridget_background_environment() -> dict[str, str]:
    """Retire toute identité T3 héritée avant un envoi CLI autonome.

    Le client idempotent utilise sa propre portée de remise, sans créer
    d'équipier CLI éphémère. Il n'emprunte jamais l'identité de la session T3
    qui a installé le LaunchAgent.
    """
    env = os.environ.copy()
    for key in (
        "TMUX_PANE",
        "BRIDGET_AGENT_ID",
        "BRIDGET_AGENT_ID_FILE",
        "BRIDGET_AGENT_INSTANCE_ID",
        "BRIDGET_DELEGATED_PID",
        "BRIDGET_DELEGATED_REF",
    ):
        env.pop(key, None)
    return env


def parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def age_seconds(value: str | None) -> int:
    ts = parse_ts(value)
    if not ts:
        return 0
    return max(0, int((datetime.now(timezone.utc) - ts).total_seconds()))


def age_bucket(age_sec: int, notice_sec: int) -> int:
    if notice_sec <= 0:
        return 0
    return max(0, age_sec // notice_sec)


def load_json(path: Path, default: Any = None) -> Any:
    if not path.exists():
        return default
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    # Le fichier canonique ne doit jamais être observé à moitié écrit : le
    # watchdog peut le lire pendant la collecte, y compris après un crash.
    payload = json.dumps(data, ensure_ascii=False, indent=2) + "\n"
    tmp = path.with_name(f".{path.name}.tmp-{os.getpid()}")
    tmp.write_text(payload, encoding="utf-8")
    os.replace(tmp, path)


def append_event(run_dir: Path, event: dict[str, Any]) -> None:
    event.setdefault("ts", now())
    path = run_dir / "events.jsonl"
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(event, ensure_ascii=False, sort_keys=True) + "\n")


def run_dir(args: argparse.Namespace) -> Path:
    return Path(args.root).expanduser() / args.run_id


def parse_json_arg(raw: str | None, fallback: Any) -> Any:
    if not raw:
        return fallback
    try:
        return json.loads(raw)
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid JSON argument: {exc}") from exc


def csv(value: str | None) -> list[str]:
    if not value:
        return []
    return [item.strip() for item in value.split(",") if item.strip()]


def list_field(value: Any) -> list[str]:
    if value is None or value == "":
        return []
    if isinstance(value, str):
        return csv(value)
    if isinstance(value, list):
        return [str(item).strip() for item in value if str(item).strip()]
    return [str(value).strip()] if str(value).strip() else []


def normalize_agent_alias(value: str) -> str:
    return value.lower().replace(" ", "")


def is_exact_tmux_target(target: str) -> bool:
    return bool(EXACT_TMUX_TARGET.match(target.strip()))


def tmux_display_pane_id(target: str) -> str:
    direct = subprocess.run(
        ["tmux", "display-message", "-p", "-t", target, "#{pane_id}"],
        text=True,
        capture_output=True,
    )
    if direct.returncode == 0 and direct.stdout.strip():
        return direct.stdout.strip()
    return ""


def as_bool(value: str | bool) -> bool:
    if isinstance(value, bool):
        return value
    value = value.lower().strip()
    if value in {"true", "1", "yes", "y"}:
        return True
    if value in {"false", "0", "no", "n"}:
        return False
    raise argparse.ArgumentTypeError("expected true/false")


def cmd_init(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    if (rd / "run.json").exists() and not args.force:
        raise SystemExit(f"run already exists: {rd}")
    for name in ["tasks", "results", "sessions", "toolchain-requests"]:
        (rd / name).mkdir(parents=True, exist_ok=True)
    created_at = now()
    policies = {
        "max_live_roles": 3,
        "allow_spawn_tmux": False,
        "allowed_backends": ["local", "existing_tmux", "existing_bridget", "background_process", "llm_process", "codex_thread"],
        "max_attempts_default": 2,
        "max_concurrent_tasks": 2,
        "max_orchestrators": 1,
        "orchestrator_selection": "latest",
        "default_agent_provider": "codex_exec",
        "default_agent_model": "",
        "spawn_tmux_uses_default_agent": False,
    }
    policies.update(mission_control_defaults())
    policies.update(parse_json_arg(args.policies_json, {}))
    data = {
        "schema_version": "agent-loop-run-v1",
        "run_id": args.run_id,
        "created_at": created_at,
        "updated_at": created_at,
        "status": "open",
        "mission_control_enabled_at": created_at,
        "prompt_initial": args.prompt_initial or "",
        "objective": args.objective,
        "acceptance": args.acceptance or "",
        "domain": args.domain or "generic",
        "domain_context": parse_json_arg(args.domain_context_json, {}),
        "policies": policies,
    }
    write_json(rd / "run.json", data)
    append_event(rd, {"event": "run.created", "run_id": args.run_id, "objective": args.objective})
    print(str(rd))
    return 0


def cmd_add_task(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    run = load_json(rd / "run.json")
    if not run:
        raise SystemExit(f"missing run.json: {rd}")
    if run.get("status") == "closed":
        raise SystemExit("run is closed")
    if task_path(rd, args.task_id).exists():
        raise SystemExit("task already exists; use relaunch instead of overwriting it")
    policies = run.get("policies", {})
    created_at = now()
    timeout_sec = int(args.timeout_sec or 0)
    due_sec = timeout_sec or int(policies.get("task_due_sec", 3600))
    work_kind = str(getattr(args, "work_kind", "generic") or "generic")
    if work_kind not in WORK_KINDS:
        raise SystemExit(f"invalid work-kind: {work_kind}")
    owner = str(getattr(args, "owner", "") or args.agent_target or "").strip()
    if not owner:
        if args.backend in {"existing_bridget", "existing_tmux"}:
            raise SystemExit("delegated task requires an assigned owner")
        owner = "local" if args.backend == "local" else "runner"
    task = {
        "schema_version": "agent-loop-task-v1",
        "task_id": args.task_id,
        "created_at": created_at,
        "updated_at": created_at,
        "status": "pending",
        "objective": args.objective,
        "acceptance": args.acceptance,
        "delegate": as_bool(args.delegate),
        "backend": args.backend,
        "agent_target": args.agent_target or "",
        "assigned_agent": owner,
        "agent_role": getattr(args, "agent_role", "worker") or "worker",
        "context": getattr(args, "context", "fresh") or "fresh",
        "resume_from": getattr(args, "resume_from", "") or "",
        "depends_on": csv(args.depends_on),
        "parallelizable": as_bool(args.parallelizable),
        "priority": args.priority,
        "conflicts_with": csv(args.conflicts_with),
        "parent_task_id": args.parent_task_id or "",
        "spawn_reason": args.spawn_reason or "",
        "max_attempts": args.max_attempts,
        "reply_channel": args.reply_channel,
        "runner_command": args.runner_command or "",
        "timeout_sec": timeout_sec,
        "due_at": str(getattr(args, "due_at", "") or iso_after_seconds(due_sec, base=created_at)),
        "expected_result": str(getattr(args, "expected_result", "") or args.acceptance),
        "work_kind": work_kind,
        "keep_alive": as_bool(args.keep_alive),
        "agent_provider": args.agent_provider or "",
        "agent_model": args.agent_model or "",
        "agent_command_template": args.agent_command_template or "",
        "cwd": getattr(args, "cwd", "") or "",
        "input_paths": csv(args.input_paths),
        "output_paths": csv(args.output_paths),
        "forbidden_paths": csv(args.forbidden_paths),
    }
    write_json(rd / "tasks" / f"{args.task_id}.json", task)
    append_event(rd, {"event": "task.created", "task_id": args.task_id, "backend": args.backend})
    print(str(rd / "tasks" / f"{args.task_id}.json"))
    return 0


def cmd_record_event(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    details = parse_json_arg(args.details_json, {})
    event = {"event": args.event, "task_id": args.task_id or "", "status": args.status or "", "message": args.message or ""}
    event.update(details)
    append_event(rd, event)
    print(str(rd / "events.jsonl"))
    return 0


def cmd_record_session(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    data = {
        "schema_version": "agent-loop-session-v1",
        "recorded_at": now(),
        "role": args.role,
        "backend": args.backend,
        "session_id": args.session_id or "",
        "jsonl_path": args.jsonl_path or "",
        "pane": args.pane or "",
        "agent_name": args.agent_name or "",
        "notes": args.notes or "",
    }
    safe_id = args.session_id or f"{args.role}-{data['recorded_at'].replace(':', '-')}"
    write_json(rd / "sessions" / f"{safe_id}.json", data)
    append_event(rd, {"event": "session.recorded", "role": args.role, "backend": args.backend, "session_id": args.session_id or ""})
    print(str(rd / "sessions" / f"{safe_id}.json"))
    return 0


def task_path(rd: Path, task_id: str) -> Path:
    return rd / "tasks" / f"{task_id}.json"


def load_task(rd: Path, task_id: str) -> dict[str, Any]:
    task = load_json(task_path(rd, task_id))
    if not task:
        raise SystemExit(f"missing task: {task_path(rd, task_id)}")
    return task


def update_task(
    rd: Path, task: dict[str, Any], *,
    expected_statuses: set[str] | frozenset[str] | None = None,
    touch: bool = True, **fields: Any,
) -> bool:
    """Fusion sous verrou : O(taille JSON + k log k), k preuves triées pour l'anti-rejeu."""
    path = task_path(rd, task["task_id"])
    with path.with_suffix(".lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        latest = load_json(path, task)
        if expected_statuses is not None and latest.get("status") not in expected_statuses:
            raise SystemExit("task changed concurrently; reload before updating")
        proof_hash = fields.get("last_progress_hash")
        if proof_hash:
            seen = set(latest.get("progress_seen_hashes") or [])
            if latest.get("last_progress_hash"):
                seen.add(latest["last_progress_hash"])
            if proof_hash in seen:
                return False
            fields["progress_seen_hashes"] = sorted(seen | {proof_hash})
        latest.update(fields)
        if touch:
            latest["updated_at"] = now()
        write_json(path, latest)
        task.clear()
        task.update(latest)
    return True


def _role_recovery_is_enforced(rd: Path) -> bool:
    run = load_json(rd / "run.json", {}) or {}
    if bool(run.get("policies", {}).get("role_recovery_enforced")):
        return True
    try:
        rd.resolve().relative_to(ROLE_RECOVERY_COORDINATION_ROOT.resolve())
        return True
    except ValueError:
        return False


def _role_recovery_cli_family(task: dict[str, Any]) -> str:
    declared = str(task.get("cli_family") or "").strip().lower()
    if declared in {"codex", "claude"}:
        return declared
    provider = str(task.get("agent_provider") or "").strip().lower()
    if provider:
        return "claude" if provider in {"gclaude", "claude", "claude_print"} else "codex"

    # Une cible logique sans provider explicite doit rester portable : le
    # registre boot confirmé est la seule autorité disponible avant le
    # dispatch. Une entrée absente ou ambiguë ne devient jamais une permission
    # implicite ; le contrôle fail-closed de canonical_boot_context la refusera.
    target = str(task.get("assigned_agent") or task.get("agent_target") or "").strip()
    if target:
        try:
            document = json.loads(BOOT_ROLE_BINDINGS_PATH.read_text(encoding="utf-8"))
            entries = document.get("entries") if isinstance(document, dict) else None
            matches = [
                entry for entry in entries if isinstance(entry, dict)
                and str(entry.get("agent_name") or "") == target
                and str(entry.get("role_status") or "") == "confirmed"
            ] if isinstance(entries, list) else []
            if len(matches) == 1:
                cli = str(matches[0].get("cli") or "").strip().lower()
                if cli in {"codex", "claude"}:
                    return cli
        except (OSError, json.JSONDecodeError):
            pass
    return "codex"


def evaluate_role_recovery_dispatch(
    rd: Path,
    task: dict[str, Any],
    *,
    current_agent: str,
    cli_family: str,
    project_root: Path | None = None,
    coordination_root: Path | None = None,
) -> dict[str, Any]:
    """Construit en mémoire un verdict borné; aucune nouvelle autorité n'est persistée."""
    project = (project_root or ROLE_RECOVERY_PROJECT_ROOT).resolve()
    coordination = (coordination_root or ROLE_RECOVERY_COORDINATION_ROOT).resolve()
    run_file = rd / "run.json"
    task_file = task_path(rd, str(task.get("task_id") or "missing"))
    roles_file = project / "40-registry" / "agents" / "roles.json"
    policy_file = project / "AGENTS.md"
    contracts_dir = project / "specs" / "101-role-prewarm-recovery" / "contracts"
    provenance_contract = contracts_dir / "mai-task-provenance-verdict-v1.schema.json"
    assertion_contract = contracts_dir / "mai-role-recovery-assertion-v1.schema.json"

    run = load_json(run_file, {}) or {}
    roles_doc = load_json(roles_file, {}) or {}
    assigned = str(task.get("assigned_agent") or "").strip()
    current = str(current_agent or "").strip()
    role = str(task.get("agent_role") or "").strip()
    context = str(task.get("context") or "fresh").strip()
    status = str(task.get("status") or "missing").strip()
    reasons: list[str] = []

    resolved_run = rd.resolve()
    resolved_parts = {part.lower() for part in resolved_run.parts}
    source_class = "coordination_task"
    try:
        resolved_run.relative_to(coordination)
    except ValueError:
        reasons.append("path_outside_coordination")
        source_class = "unknown"
    if resolved_parts & ROLE_RECOVERY_FORBIDDEN_PATH_PARTS:
        reasons.append("test_or_document_path")
        source_class = "test_fixture"
    if not run:
        reasons.append("run_missing")
    run_terminal = str(run.get("status") or "").lower() in ROLE_RECOVERY_TERMINAL_RUN_STATUSES
    if run_terminal:
        reasons.append("run_terminal")
    if task.get("schema_version") not in {"agent-loop-task-v1", "agent-loop-task-v2"}:
        reasons.append("task_invalid")
    if status not in {"dispatched", "in_progress", "running"}:
        reasons.append("task_not_consumable")
    if not assigned:
        reasons.append("assigned_agent_missing")
    elif normalize_agent_alias(assigned) != normalize_agent_alias(current):
        reasons.append("assigned_agent_mismatch")
    if not role or role not in (roles_doc.get("roles") or {}):
        reasons.append("role_source_missing")
    if context not in {"fresh", "continuation"}:
        reasons.append("task_invalid")
    active_siblings = []
    for sibling_file in (rd / "tasks").glob("*.json"):
        sibling = load_json(sibling_file, {}) or {}
        if sibling.get("task_id") == task.get("task_id"):
            continue
        if (
            str(sibling.get("status") or "") in {"dispatched", "in_progress", "running"}
            and normalize_agent_alias(str(sibling.get("assigned_agent") or ""))
            == normalize_agent_alias(current)
        ):
            active_siblings.append(str(sibling.get("task_id") or sibling_file.stem))
    if active_siblings:
        reasons.append("ambiguous_active_tasks")

    snapshot_path: Path | None = None
    if context == "continuation":
        resume_from = str(task.get("resume_from") or "").strip()
        if resume_from:
            candidate = Path(resume_from)
            snapshot_path = candidate if candidate.is_absolute() else rd / candidate
        snapshot = load_json(snapshot_path, {}) if snapshot_path else {}
        try:
            inside_run = bool(snapshot_path and snapshot_path.resolve().is_relative_to(rd.resolve()))
        except (OSError, ValueError):
            inside_run = False
        if (
            not inside_run
            or not snapshot
            or snapshot.get("schema_version") != "task-snapshot-v1"
            or snapshot.get("task_id") != task.get("task_id")
        ):
            reasons.append("resume_snapshot_missing")

    policy_text = policy_file.read_text(encoding="utf-8") if policy_file.exists() else ""
    if f"policy={ROLE_RECOVERY_POLICY_VERSION}" not in policy_text:
        reasons.append("policy_version_unknown")
    for contract, expected in (
        (provenance_contract, "mai-task-provenance-verdict-v1.schema.json"),
        (assertion_contract, "mai-role-recovery-assertion-v1.schema.json"),
    ):
        document = load_json(contract, {}) or {}
        if not str(document.get("$id") or "").endswith(expected):
            reasons.append("policy_version_unknown")

    reasons = list(dict.fromkeys(reasons))
    provenance_reasons = [
        reason for reason in reasons
        if reason in {
            "path_outside_coordination", "test_or_document_path", "run_missing", "run_terminal", "task_invalid",
            "task_not_consumable", "assigned_agent_missing", "assigned_agent_mismatch",
            "resume_snapshot_missing", "ambiguous_active_tasks",
        }
    ][:8]
    task_schema_version = str(task.get("schema_version") or "missing")
    if task_schema_version not in {"agent-loop-task-v1", "agent-loop-task-v2", "missing"}:
        task_schema_version = "invalid"
    contract_statuses = {
        "pending", "ready", "dispatched", "in_progress", "running", "blocked",
        "blocked-cause-known", "completed", "failed", "pass", "review", "cancelled",
        "superseded", "missing",
    }
    contract_status = status if status in contract_statuses else "invalid"
    provenance = {
        "schema_version": "mai-task-provenance-verdict-v1",
        "evaluated_at": now(),
        "candidate_ref": str(task_file.resolve()),
        "task_schema_version": task_schema_version,
        "task_status": contract_status,
        "assigned_agent": assigned or None,
        "current_agent": current or "unknown",
        "assignment_matches_current_agent": bool(assigned and normalize_agent_alias(assigned) == normalize_agent_alias(current)),
        "run_terminal": run_terminal,
        "source_class": source_class,
        "verdict": "valid" if not provenance_reasons else "invalid",
        "reason_codes": provenance_reasons,
    }
    if source_class == "coordination_task" and "path_outside_coordination" not in reasons:
        provenance["task_file"] = str(task_file.resolve())
    assertion_reasons: list[str] = []
    if provenance["verdict"] != "valid":
        assertion_reasons.append("task_provenance_invalid")
    if "role_source_missing" in reasons:
        assertion_reasons.append("role_source_missing")
    if "assigned_agent_missing" in reasons:
        assertion_reasons.append("binding_missing")
    if "assigned_agent_mismatch" in reasons:
        assertion_reasons.append("binding_mismatch")
    if "resume_snapshot_missing" in reasons:
        assertion_reasons.append("resume_snapshot_missing")
    if "policy_version_unknown" in reasons:
        assertion_reasons.append("policy_version_unknown")
    allowed = not reasons
    binding_file = Path('/Users/moi/Documents/MAICompany/coordination/watchdog/boot-role-bindings.json')
    source_refs = [str(run_file.resolve()), str(roles_file.resolve()), str(policy_file.resolve())]
    if binding_file.is_file():
        source_refs.append(str(binding_file.resolve()))
    assertion: dict[str, Any] = {
        "schema_version": "mai-role-recovery-assertion-v1",
        "observed_at": now(),
        "cli_family": cli_family if cli_family in {"codex", "claude"} else "codex",
        "agent_id": current or "unknown",
        "role": role or "unknown",
        "run_id": str(run.get("run_id") or rd.name),
        "task_id": str(task.get("task_id") or "missing"),
        "context": context if context in {"fresh", "continuation"} else "fresh",
        "policy_version": ROLE_RECOVERY_POLICY_VERSION,
        "source_refs": source_refs,
        "task_provenance": provenance,
        "verdict": "confirmed" if allowed else "unknown",
        "reason_codes": assertion_reasons or (["task_provenance_invalid"] if not allowed else []),
        "next_action": "continue_assigned_task" if allowed else "request_canonical_task",
        "procedural_enforcement": "machine_enforced_v1",
        "enforcement_targets": {
            "v1_mode": "machine_enforced",
            "future_primary_caller": "agent_loop.py::cmd_dispatch",
            "future_independent_observer": "coordination_watchdog.py",
        },
        "raw_pane_text_used": False,
        "binding_generation": hashlib.sha256(
            "|".join(f"{ref}:{hashlib.sha256(Path(ref).read_bytes()).hexdigest()}" for ref in source_refs if Path(ref).is_file()).encode()
        ).hexdigest(),
    }
    if snapshot_path and context == "continuation":
        assertion["snapshot_ref"] = str(snapshot_path.resolve())
    return {
        "allowed": allowed,
        "reason_codes": reasons,
        "provenance": provenance,
        "assertion": assertion,
    }


def result_path(rd: Path, task_id: str) -> Path:
    return rd / "results" / f"{task_id}.result.json"


def archive_previous_result(rd: Path, task_id: str) -> str:
    rp = result_path(rd, task_id)
    if not rp.exists():
        return ""
    suffix = now().replace(":", "").replace("-", "").replace(".", "")
    archived = rp.with_name(f"{task_id}.previous-{suffix}.result.json")
    rp.rename(archived)
    append_event(rd, {"event": "task.previous_result_archived", "task_id": task_id, "result_path": str(archived)})
    return str(archived)


def log_dir(rd: Path, task_id: str) -> Path:
    path = rd / "logs" / task_id
    path.mkdir(parents=True, exist_ok=True)
    return path


def tail_text(path: Path, limit: int = 8000) -> str:
    if not path.exists():
        return ""
    data = path.read_bytes()
    if len(data) <= limit:
        return data.decode("utf-8", errors="replace")
    return data[-limit:].decode("utf-8", errors="replace")


def normalize_result(
    *,
    task_id: str,
    status: str,
    message: str = "",
    worker: str = "",
    terminal: bool = True,
    blockers: list[dict[str, Any]] | None = None,
    output_paths: list[str] | None = None,
    metrics: dict[str, Any] | None = None,
    next_recommendation: str = "",
    logs: dict[str, str] | None = None,
) -> dict[str, Any]:
    return {
        "schema_version": "agent-loop-result-v2",
        "task_id": task_id,
        "created_at": now(),
        "status": status,
        "terminal": terminal,
        "message": message,
        "blockers": blockers or [],
        "output_paths": output_paths or [],
        "metrics": metrics or {},
        "next_recommendation": next_recommendation,
        "logs": logs or {},
        "worker": worker,
    }


def write_task_result(
    rd: Path,
    task: dict[str, Any],
    *,
    status: str,
    message: str = "",
    worker: str = "",
    blockers: list[dict[str, Any]] | None = None,
    output_paths: list[str] | None = None,
    metrics: dict[str, Any] | None = None,
    next_recommendation: str = "",
    logs: dict[str, str] | None = None,
) -> Path:
    rp = result_path(rd, task["task_id"])
    result = normalize_result(
        task_id=task["task_id"],
        status=status,
        message=message,
        worker=worker,
        blockers=blockers,
        output_paths=output_paths,
        metrics=metrics,
        next_recommendation=next_recommendation,
        logs=logs,
    )
    write_json(rp, result)
    update_task(rd, task, status=status, result_path=str(rp), collected_at=now())
    write_continuation_receipt(rd, task, result)
    append_event(rd, {"event": "task.result_written", "task_id": task["task_id"], "status": status, "result_path": str(rp)})
    return rp


def bridge_discover() -> list[dict[str, str]]:
    if not BRIDGE.exists():
        raise SystemExit(f"missing agent-bridge script: {BRIDGE}")
    proc = subprocess.run([str(BRIDGE), "discover"], check=True, text=True, capture_output=True)
    agents: list[dict[str, str]] = []
    for line in proc.stdout.splitlines()[1:]:
        parts = line.split()
        if len(parts) < 4:
            continue
        pane, addr, typ, name = parts[:4]
        pane_name = "" if len(parts) < 5 or parts[4] == "—" else parts[4]
        agents.append({"pane": pane, "addr": addr, "type": typ, "name": name, "pane_name": pane_name})
    return agents


def bridget_agents() -> list[dict[str, Any]]:
    """Lit le registre vivant de Bridget sans identité d'agent héritée."""
    if not BRIDGET.is_file():
        raise SystemExit(f"missing Bridget binary: {BRIDGET}")
    try:
        proc = subprocess.run(
            [str(BRIDGET), "agents", "--json"],
            check=True,
            text=True,
            capture_output=True,
            timeout=10,
            env=bridget_background_environment(),
        )
        payload = json.loads(proc.stdout)
    except subprocess.TimeoutExpired as exc:
        raise SystemExit("Bridget agents timed out") from exc
    except subprocess.CalledProcessError as exc:
        raise SystemExit(f"Bridget agents failed: rc={exc.returncode}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit("Bridget agents returned invalid JSON") from exc
    if not isinstance(payload, list) or not all(isinstance(item, dict) for item in payload):
        raise SystemExit("Bridget agents returned an invalid directory")
    return payload


def bridget_agents_by_id() -> dict[str, dict[str, Any]]:
    return {
        str(agent.get("agent_id")): agent
        for agent in bridget_agents()
        if agent.get("agent_id")
    }


def bridget_agent_connected(agent: dict[str, Any]) -> bool:
    """Occupé signifie connecté : la file T3 reçoit les actions sans réveil forcé."""
    return str(agent.get("state") or "").lower() in {"connected", "busy"}


def resolve_bridget_target(
    target: str,
    agents: list[dict[str, Any]] | None = None,
    *,
    require_connected: bool = True,
) -> dict[str, Any]:
    """Résout un UUID Bridget, un préfixe UUID ou un nom humain unique."""
    target = str(target or "").strip()
    if not target:
        raise SystemExit("empty Bridget target")
    directory = agents if agents is not None else bridget_agents()
    target_lc = target.lower()
    exact_ids = [agent for agent in directory if str(agent.get("agent_id") or "").lower() == target_lc]
    prefix_ids = [
        agent for agent in directory
        if len(target) >= 6 and str(agent.get("agent_id") or "").lower().startswith(target_lc)
    ]
    names = [
        agent for agent in directory
        if str(agent.get("display_name") or "").strip().lower() == target_lc
    ]
    matches = exact_ids or prefix_ids or names
    if len(matches) != 1:
        reason = "ambiguous" if matches else "not found"
        raise SystemExit(f"Bridget target {reason}: {target}")
    agent = matches[0]
    if require_connected and not bridget_agent_connected(agent):
        raise SystemExit(
            f"Bridget target is not connected: {agent.get('agent_id')} "
            f"state={agent.get('state') or 'unknown'}"
        )
    return agent


def send_bridget_message(agent_id: str, message: str, *, replay: dict[str, Any] | None = None) -> None:
    """Envoie un message sans réponse et sans usurper une identité T3."""
    # Client idempotent, jamais une inscription de faux équipier CLI temporaire.
    replay = replay or {"id": uuid.uuid4().hex, "issued_at": int(datetime.now(timezone.utc).timestamp()),
                        "issuer_scope": "agent-loop-background"}
    command = [str(BRIDGET), "send", "--to", agent_id, "--hops", "4",
               "--id", replay["id"], "--issued-at", str(replay["issued_at"]), "--issuer-scope", replay["issuer_scope"]]
    subprocess.run(
        command + ["--", message],
        check=True,
        text=True,
        capture_output=True,
        timeout=130,
        env=bridget_background_environment(),
    )


def agent_by_pane() -> dict[str, dict[str, str]]:
    return {agent["pane"]: agent for agent in bridge_discover()}


def resolve_agent_target(target: str) -> str:
    if not target:
        raise SystemExit("empty agent target")
    target_lc = normalize_agent_alias(target)
    agents = bridge_discover()
    matches = []
    for agent in agents:
        aliases = {
            agent["pane"],
            agent["addr"],
            agent["name"],
            agent["pane_name"],
            agent["addr"].replace(":", "."),
            agent["addr"].replace("agents:", "w"),
            agent["addr"].replace("agents:", "w").replace(".", ""),
        }
        normalized = {normalize_agent_alias(a) for a in aliases if a}
        if target_lc in normalized:
            matches.append(agent)
    if len(matches) == 1:
        return matches[0]["pane"]
    if len(matches) > 1:
        raise SystemExit(f"agent target ambiguous: {target}. matches: {matches}")
    # tmux accepts fuzzy targets; only call it for exact pane ids or addresses.
    if is_exact_tmux_target(target):
        direct = tmux_display_pane_id(target)
        if direct:
            return direct
    available = ", ".join(f"{a['pane']}/{a['addr']}/{a['name']}/{a['pane_name'] or '-'}" for a in agents)
    raise SystemExit(f"agent target not found: {target}. available: {available}")


def bridge_model_info(target: str) -> dict[str, str]:
    if not BRIDGE.exists():
        return {}
    proc = subprocess.run(
        [str(BRIDGE), "model", "--to", target],
        text=True,
        capture_output=True,
    )
    if proc.returncode != 0:
        return {}
    lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    if len(lines) < 2:
        return {}
    parts = lines[-1].split(maxsplit=3)
    if len(parts) < 4:
        return {}
    return {
        "agent_pane": parts[0],
        "agent_name": parts[1],
        "agent_type": parts[2],
        "agent_model": parts[3],
    }


def validate_authorized_recipient(task: dict[str, Any]) -> tuple[bool, str, list[str]]:
    authorized = list_field(task.get("authorized_agents"))
    if not authorized:
        return True, "", []
    assigned = str(task.get("assigned_agent") or task.get("agent_target") or "").strip()
    authorized_norm = {normalize_agent_alias(item) for item in authorized}
    if assigned and normalize_agent_alias(assigned) in authorized_norm:
        return True, assigned, authorized
    return False, assigned, authorized


def refuse_invalid_recipient(rd: Path, task: dict[str, Any], assigned: str, authorized: list[str]) -> int:
    reason = "invalid_recipient"
    update_task(
        rd,
        task,
        status="blocked",
        blocked_reason=reason,
        assigned_agent=assigned,
        authorized_agents=authorized,
    )
    append_event(
        rd,
        {
            "event": "task.dispatch_refused",
            "task_id": task.get("task_id", ""),
            "reason": reason,
            "assigned_agent": assigned,
            "authorized_agents": authorized,
        },
    )
    print(invalid_recipient_message(assigned, authorized))
    return 2


def invalid_recipient_message(assigned: str, authorized: list[str]) -> str:
    return f"BLOCKED invalid_recipient assigned_agent={assigned or '-'} authorized_agents={','.join(authorized) or '-'}"


def session_files(rd: Path) -> list[Path]:
    return sorted((rd / "sessions").glob("*.json"))


def session_is_live(
    session: dict[str, Any],
    live: dict[str, dict[str, str]],
    bridget_live: dict[str, dict[str, Any]] | None = None,
) -> bool:
    if session.get("backend") == "existing_bridget":
        agent_id = str(session.get("agent_id") or "")
        agent = (bridget_live or {}).get(agent_id, {})
        return bridget_agent_connected(agent)
    pane = session.get("pane") or ""
    if pane and pane in live:
        return True
    name = (session.get("agent_name") or "").lower()
    pane_name = (session.get("pane_name") or "").lower()
    return any(
        (name and agent.get("name", "").lower() == name)
        or (pane_name and agent.get("pane_name", "").lower() == pane_name)
        for agent in live.values()
    )


def maybe_close_spawned_pane(rd: Path, task: dict[str, Any]) -> None:
    if task.get("dispatch_backend") != "spawn_tmux":
        return
    if task.get("keep_alive"):
        return
    if task.get("pane_closed_at"):
        return
    pane = task.get("pane") or ""
    if not pane:
        return
    proc = subprocess.run(["tmux", "display-message", "-p", "-t", pane, "#{pane_id}"], text=True, capture_output=True)
    if proc.returncode != 0:
        append_event(rd, {"event": "tmux_pane.close_skipped", "task_id": task.get("task_id", ""), "pane": pane, "reason": "pane_not_found"})
        update_task(rd, task, pane_closed_at=now(), pane_close_status="not_found")
        return
    kill = subprocess.run(["tmux", "kill-pane", "-t", pane], text=True, capture_output=True)
    if kill.returncode == 0:
        update_task(rd, task, pane_closed_at=now(), pane_close_status="closed")
        append_event(rd, {"event": "tmux_pane.closed", "task_id": task.get("task_id", ""), "pane": pane})
    else:
        update_task(rd, task, pane_closed_at=now(), pane_close_status="not_found_after_check")
        append_event(rd, {"event": "tmux_pane.close_skipped", "task_id": task.get("task_id", ""), "pane": pane, "reason": "not_found_after_check"})


def find_session_jsonls(limit: int = 80) -> list[dict[str, Any]]:
    roots = [
        ("codex", Path("/Users/moi/.codex/sessions")),
        ("claude", Path("/Users/moi/.claude/sessions")),
    ]
    items: list[dict[str, Any]] = []
    for source, root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file() or path.suffix not in {".jsonl", ".json"}:
                continue
            try:
                stat = path.stat()
            except OSError:
                continue
            items.append(
                {
                    "source": source,
                    "path": str(path),
                    "mtime": datetime.fromtimestamp(stat.st_mtime, timezone.utc).isoformat().replace("+00:00", "Z"),
                    "size": stat.st_size,
                }
            )
    items.sort(key=lambda item: item["mtime"], reverse=True)
    return items[:limit]


# Alias explicite et borné : la doctrine historique nomme l'orchestrateur
# « orchestrator », tandis que la taxonomie MAICompany le nomme
# « coordinateur ». Les deux désignent la même responsabilité. Tous les
# autres rôles conservent une comparaison exacte.
ORCHESTRATOR_ROLE_ALIASES = frozenset({"orchestrator", "coordinateur"})


def role_alias_set(role: str) -> frozenset[str]:
    if role in ORCHESTRATOR_ROLE_ALIASES:
        return ORCHESTRATOR_ROLE_ALIASES
    return frozenset({role})


def live_session_for_role(rd: Path, role: str) -> dict[str, Any] | None:
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    selection = policies.get("orchestrator_selection", "latest")
    aliases = role_alias_set(role)
    sessions = [load_json(path, {}) for path in session_files(rd)]
    candidates = [
        session for session in sessions
        if session.get("role") in aliases and session.get("status") == "live"
    ]
    needs_tmux = any(session.get("backend") != "existing_bridget" for session in candidates)
    needs_bridget = any(session.get("backend") == "existing_bridget" for session in candidates)
    live = agent_by_pane() if needs_tmux else {}
    bridget_live = bridget_agents_by_id() if needs_bridget else {}
    matches: list[dict[str, Any]] = []
    for session in candidates:
        if session_is_live(session, live, bridget_live):
            matches.append(session)
    if not matches:
        return None
    if len(matches) > int(policies.get("max_orchestrators", 1)) and selection == "error":
        # La sélection est une lecture pure, y compris pendant un dry-run.
        # Le heartbeat journalise l'étape injoignable et sa décision de suite.
        return None
    if selection == "first":
        return sorted(matches, key=lambda item: item.get("recorded_at", ""))[0]
    return sorted(matches, key=lambda item: item.get("recorded_at", ""), reverse=True)[0]


def cmd_attach_agent(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    backend = getattr(args, "backend", "existing_tmux")
    if backend == "existing_bridget":
        agent = resolve_bridget_target(args.target)
        agent_id = str(agent["agent_id"])
        session_id = args.session_id or f"{args.role}-{agent_id[:12]}"
        data = {
            "schema_version": "agent-loop-session-v1",
            "recorded_at": now(),
            "updated_at": now(),
            "role": args.role,
            "backend": backend,
            "status": "live",
            "session_id": session_id,
            "jsonl_path": args.jsonl_path or "",
            "agent_id": agent_id,
            "agent_name": agent.get("display_name", ""),
            "agent_type": agent.get("agent_type", ""),
            "transport": agent.get("transport", ""),
            "domain": agent.get("domain", ""),
            "notes": args.notes or "",
        }
        path = rd / "sessions" / f"{session_id}.json"
        write_json(path, data)
        append_event(
            rd,
            {
                "event": "session.attached",
                "role": args.role,
                "session_id": session_id,
                "backend": backend,
                "agent_id": agent_id,
            },
        )
        print(str(path))
        return 0
    pane = resolve_agent_target(args.target)
    live = agent_by_pane()
    agent = live.get(pane, {})
    session_id = args.session_id or f"{args.role}-{pane.lstrip('%')}"
    data = {
        "schema_version": "agent-loop-session-v1",
        "recorded_at": now(),
        "updated_at": now(),
        "role": args.role,
        "backend": "existing_tmux",
        "status": "live",
        "session_id": session_id,
        "jsonl_path": args.jsonl_path or "",
        "pane": pane,
        "addr": agent.get("addr", ""),
        "agent_type": agent.get("type", ""),
        "agent_name": agent.get("name", args.target),
        "pane_name": agent.get("pane_name", ""),
        "notes": args.notes or "",
    }
    write_json(rd / "sessions" / f"{session_id}.json", data)
    append_event(rd, {"event": "session.attached", "role": args.role, "session_id": session_id, "pane": pane})
    print(str(rd / "sessions" / f"{session_id}.json"))
    return 0


def cmd_refresh_sessions(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    sessions = [(path, load_json(path, {})) for path in session_files(rd)]
    needs_tmux = any(session.get("backend") != "existing_bridget" for _, session in sessions)
    needs_bridget = any(session.get("backend") == "existing_bridget" for _, session in sessions)
    live = agent_by_pane() if needs_tmux else {}
    bridget_live = bridget_agents_by_id() if needs_bridget else {}
    changed = 0
    for path, session in sessions:
        old_status = session.get("status", "")
        new_status = "live" if session_is_live(session, live, bridget_live) else "stale"
        if old_status != new_status:
            session["status"] = new_status
            session["updated_at"] = now()
            write_json(path, session)
            append_event(rd, {"event": "session.status_changed", "session_id": session.get("session_id", path.stem), "status": new_status})
            changed += 1
    print(f"REFRESHED sessions changed={changed}")
    return 0


def cmd_worker_result(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    task = load_task(rd, args.task_id)
    blockers = parse_json_arg(args.blockers_json, [])
    metrics = parse_json_arg(args.metrics_json, {})
    output_paths = csv(args.output_paths)
    logs = parse_json_arg(args.logs_json, {})
    rp = write_task_result(
        rd,
        task,
        status=args.status,
        message=args.message,
        worker=args.worker or "agent_loop.py worker-result",
        blockers=blockers,
        output_paths=output_paths,
        metrics=metrics,
        next_recommendation=args.next_recommendation,
        logs=logs,
    )
    print(str(rp))
    return 0


def cmd_ack(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    task = load_task(rd, args.task_id)
    status = str(task.get("status") or "")
    if status == "running":
        print(f"ALREADY_ACKNOWLEDGED {args.task_id}")
        return 0
    if status != "dispatched":
        raise SystemExit(f"task status is {status!r}; expected 'dispatched' before ack")
    worker = (args.worker or task.get("agent_target") or task.get("resolved_target") or "").strip()
    if task_agent_identity(task) and worker != task_agent_identity(task):
        raise SystemExit("ack worker does not match the assigned owner")
    acknowledged_at = now()
    update_task(
        rd,
        task,
        expected_statuses={"dispatched"},
        status="running",
        acknowledged_at=acknowledged_at,
        acknowledged_by=worker,
        started_at=acknowledged_at,
        last_progress_at=acknowledged_at,
        last_progress_kind="worker_ack",
        last_progress_ref=f"ack:{worker or 'unknown'}",
    )
    append_event(
        rd,
        {
            "event": "task.acknowledged",
            "task_id": args.task_id,
            "worker": worker,
            "resolved_target": task.get("resolved_target", ""),
        },
    )
    print(f"ACKNOWLEDGED {args.task_id} worker={worker or '-'}")
    return 0


def validate_progress_reference(kind: str, evidence_ref: str) -> None:
    if kind not in MISSION_PROGRESS_KINDS:
        raise SystemExit(f"invalid progress kind: {kind}")
    expected = MISSION_EVIDENCE_PREFIXES[kind]
    if not evidence_ref.startswith(expected) or not evidence_ref[len(expected):].strip():
        raise SystemExit(f"evidence-ref for {kind} must start with {expected!r} and contain a value")


def cmd_progress(args: argparse.Namespace) -> int:
    """Vérifie une preuve : O(octets des fichiers déclarés ou de l'artefact), puis fusion atomique."""
    rd = run_dir(args)
    task = load_task(rd, args.task_id)
    if str(task.get("status") or "") not in ACTIVE_TASK_STATUSES:
        raise SystemExit(f"task status is {task.get('status')!r}; progress requires an active task")
    kind = str(args.kind or "").strip()
    evidence_ref = str(args.evidence_ref or "").strip()
    validate_progress_reference(kind, evidence_ref)
    worker = str(args.worker or task_agent_identity(task) or "unknown").strip()
    if worker != task_agent_identity(task):
        raise SystemExit("progress worker does not match the assigned owner")
    reference = evidence_ref.split(":", 1)[1]
    if kind != "source_changed" and (not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", reference) or ".." in reference):
        raise SystemExit("invalid evidence-ref identifier")
    if kind == "worker_ack":
        raise SystemExit("use ack for initial acknowledgement; repeated ACK is not progress")
    if kind == "source_changed":
        baseline = task.get("progress_baseline", {})
        actual = progress_snapshot(task)
        if reference not in actual or actual[reference] == "missing":
            raise SystemExit("source evidence-ref must name an existing declared input/output file")
        if reference not in baseline or actual[reference] == baseline[reference]:
            raise SystemExit("source evidence-ref has no verified change from the dispatch snapshot")
        proof = actual[reference]
    elif kind in {"task_assigned", "dependency_handoff"}:
        related = load_json(task_path(rd, reference), {})
        if not related or related.get("status") not in ACTIVE_TASK_STATUSES:
            raise SystemExit("task evidence-ref requires an actually dispatched or acknowledged task")
        if kind == "dependency_handoff" and task_agent_identity(related) == worker:
            raise SystemExit("handoff evidence-ref requires a different dependency owner")
        proof = notify_signature([{
            "event": kind, "task_id": reference, "status": related.get("status"),
            "evidence_marker": related.get("acknowledged_at") or related.get("dispatched_at"),
        }])
    elif kind == "result_checked":
        result = load_json(result_path(rd, reference), {})
        if result.get("task_id") != reference or result.get("status") not in TERMINAL_STATUSES:
            raise SystemExit("result evidence-ref requires an existing terminal result")
        proof = json.dumps(result, sort_keys=True, ensure_ascii=False)
    else:
        blocker = load_json(rd / "decisions" / f"{reference}.json", {})
        check_at = parse_ts(blocker.get("next_check"))
        if not blocker.get("owner") or not blocker.get("reason_code") or not check_at:
            raise SystemExit("blocker evidence-ref requires a decision with owner, reason_code and next_check")
        if age_seconds(blocker.get("next_check")) > 0:
            raise SystemExit("blocker next_check must be in the future")
        proof = json.dumps(blocker, sort_keys=True, ensure_ascii=False)
    proof_hash = hashlib.sha256(f"{kind}:{reference}:{proof}".encode("utf-8")).hexdigest()
    if task.get("last_progress_hash") == proof_hash:
        print(f"NO_OP unchanged_progress {args.task_id}")
        return 0
    observed_at = now()
    changed = update_task(
        rd,
        task,
        expected_statuses=ACTIVE_TASK_STATUSES,
        last_progress_at=observed_at,
        last_progress_kind=kind,
        last_progress_ref=evidence_ref,
        last_progress_hash=proof_hash,
        last_progress_by=worker,
    )
    if not changed:
        print(f"NO_OP unchanged_progress {args.task_id}")
        return 0
    append_event(
        rd,
        {
            "event": "task.progress_recorded",
            "task_id": args.task_id,
            "worker": worker,
            "kind": kind,
            "evidence_hash": proof_hash,
        },
    )
    print(f"PROGRESS {args.task_id} kind={kind} evidence_hash={proof_hash}")
    return 0


def dispatch_message(rd: Path, task: dict[str, Any]) -> str:
    reply_channel = task.get("reply_channel", "file")
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    provider = task.get("agent_provider") or policies.get("default_agent_provider") or ""
    model = task.get("agent_model") or policies.get("default_agent_model") or ""
    ack_command = (
        f"python3 {shlex.quote(str(Path(__file__).resolve()))} ack "
        f"--root {shlex.quote(str(rd.parent))} --run-id {shlex.quote(rd.name)} "
        f"--task-id {shlex.quote(task['task_id'])} "
        f"--worker {shlex.quote(str(task.get('agent_target') or ''))}"
    )
    progress_command = (
        f"python3 {shlex.quote(str(Path(__file__).resolve()))} progress "
        f"--root {shlex.quote(str(rd.parent))} --run-id {shlex.quote(rd.name)} "
        f"--task-id {shlex.quote(task['task_id'])} --worker {shlex.quote(str(task.get('agent_target') or ''))} "
        "--kind TYPE --evidence-ref PREFIX:PREUVE"
    )
    return (
        "[agent-loop task]\n"
        f"run_dir: {rd}\n"
        f"task_file: {task_path(rd, task['task_id'])}\n"
        f"result_file: {result_path(rd, task['task_id'])}\n"
        f"reply_channel: {reply_channel}\n"
        f"agent_provider: {provider}\n"
        f"agent_model: {model}\n"
        f"objective: {task.get('objective','')}\n"
        f"acceptance: {task.get('acceptance','')}\n"
        f"expected_result: {task.get('expected_result') or task.get('acceptance','')}\n"
        f"due_at: {task.get('due_at','')}\n"
        "Instruction prioritaire: TU ES LE WORKER CIBLE, pas un relais. "
        "Ne reponds jamais que la tache a deja ete relayee: le transport ne vaut pas prise en charge. "
        f"Avant tout travail, execute exactement cet ACK: {ack_command}\n"
        f"Ensuite, pour chaque progrès vérifiable, utilise: {progress_command}\n"
        "Ensuite lis la tache, respecte forbidden_paths, puis rends un resultat selon reply_channel. "
        "file=result.json seulement; bridge=reponse courte via bridge; both=result.json puis resume court avec chemin."
    )


def run_local_command(rd: Path, task: dict[str, Any], command: str) -> int:
    task_id = task["task_id"]
    logs = log_dir(rd, task_id)
    stdout_path = logs / "stdout.log"
    stderr_path = logs / "stderr.log"
    timeout = int(task.get("timeout_sec") or 0) or None
    started_at = now()
    update_task(rd, task, status="running", started_at=started_at, dispatch_backend="local", attempts=int(task.get("attempts", 0)) + 1)
    append_event(rd, {"event": "task.started", "task_id": task_id, "backend": "local"})
    try:
        proc = subprocess.run(
            ["zsh", "-lc", command],
            cwd=str(rd),
            text=True,
            capture_output=True,
            timeout=timeout,
        )
        stdout_path.write_text(proc.stdout, encoding="utf-8")
        stderr_path.write_text(proc.stderr, encoding="utf-8")
        status = "pass" if proc.returncode == 0 else "fail"
        metrics = {"returncode": proc.returncode}
        message = f"local command exited {proc.returncode}"
    except subprocess.TimeoutExpired as exc:
        stdout_path.write_text(exc.stdout or "", encoding="utf-8")
        stderr_path.write_text(exc.stderr or "", encoding="utf-8")
        status = "blocked"
        metrics = {"timeout_sec": timeout}
        message = f"local command timeout after {timeout}s"
    write_task_result(
        rd,
        load_task(rd, task_id),
        status=status,
        message=message,
        worker="agent_loop.py local",
        metrics=metrics,
        logs={"stdout": str(stdout_path), "stderr": str(stderr_path), "stdout_tail": tail_text(stdout_path), "stderr_tail": tail_text(stderr_path)},
    )
    print(f"LOCAL_DONE {task_id} status={status}")
    return 0 if status == "pass" else 1


def start_background_process(rd: Path, root: str, run_id: str, task: dict[str, Any], command: str) -> int:
    task_id = task["task_id"]
    logs = log_dir(rd, task_id)
    stdout_path = logs / "stdout.log"
    stderr_path = logs / "stderr.log"
    wrapper = (
        "set +e\n"
        f"cd {shlex.quote(str(rd))}\n"
        f"({command}) > {shlex.quote(str(stdout_path))} 2> {shlex.quote(str(stderr_path))}\n"
        "rc=$?\n"
        "if [ \"$rc\" -eq 0 ]; then result_status=pass; else result_status=fail; fi\n"
        f"{shlex.quote(sys.executable)} {shlex.quote(str(Path(__file__)))} worker-result "
        f"--root {shlex.quote(str(Path(root).expanduser()))} "
        f"--run-id {shlex.quote(run_id)} "
        f"--task-id {shlex.quote(task_id)} "
        "--worker background_process "
        "--status \"$result_status\" "
        "--message \"background command exited $rc\" "
        f"--metrics-json '{{\"returncode\":'\"$rc\"'}}' "
        f"--logs-json {shlex.quote(json.dumps({'stdout': str(stdout_path), 'stderr': str(stderr_path)}, ensure_ascii=False))}\n"
    )
    proc = subprocess.Popen(["zsh", "-lc", wrapper], cwd=str(rd), start_new_session=True)
    update_task(
        rd,
        task,
        status="running",
        dispatched_at=now(),
        dispatch_backend="background_process",
        pid=proc.pid,
        logs={"stdout": str(stdout_path), "stderr": str(stderr_path)},
        attempts=int(task.get("attempts", 0)) + 1,
    )
    append_event(rd, {"event": "task.dispatched", "task_id": task_id, "backend": "background_process", "pid": proc.pid})
    print(f"DISPATCHED background_process pid={proc.pid}")
    return 0


def task_cwd(rd: Path, task: dict[str, Any]) -> Path:
    """Retourne le répertoire de travail effectif d'une tâche agent."""
    return Path(task.get("cwd") or rd).expanduser().resolve()


def declared_output_roots(cwd: Path, task: dict[str, Any]) -> list[Path]:
    """Normalise les racines d'écriture déclarées par la tâche.

    Un chemin de fichier est ramené à son dossier parent, car ``--add-dir``
    autorise des répertoires. Les chemins sans extension restent des racines.
    """
    roots: set[Path] = set()
    for path in task.get("output_paths", []):
        if not path:
            continue
        candidate = Path(path).expanduser()
        candidate = (candidate if candidate.is_absolute() else cwd / candidate).resolve()
        roots.add(candidate.parent if (candidate.exists() and candidate.is_file()) or candidate.suffix else candidate)
    return sorted(roots, key=str)


def path_is_covered(path: Path, roots: list[Path]) -> bool:
    return any(path == root or root in path.parents for root in roots)


def writable_roots_for_llm(rd: Path, task: dict[str, Any], provider: str) -> tuple[Path, list[Path]] | None:
    """Détermine les racines d'écriture réellement garanties au worker.

    Codex est le seul provider non-tmux pour lequel agent-loop sait déclarer
    ces racines de façon fiable (``-C`` et ``--add-dir``). Les autres providers
    et les templates libres ne donnent pas cette garantie : on refuse donc le
    dispatch avant de créer un worker.
    """
    cwd = task_cwd(rd, task)
    if not cwd.is_dir():
        return None
    if provider != "codex_exec" or task.get("agent_command_template"):
        return None

    output_roots = declared_output_roots(cwd, task)
    if any(not root.is_dir() for root in output_roots):
        return None

    # Le résultat autoritaire est écrit par l'agent dans le run_dir ; il doit
    # rester accessible même si le cwd de travail est un worktree externe.
    roots = [cwd, rd.resolve(), *output_roots]
    effective: list[Path] = []
    for root in roots:
        if not path_is_covered(root, effective):
            effective.append(root)
    if not all(path_is_covered(path, effective) for path in [cwd, *output_roots]):
        return None
    return cwd, effective


def refuse_writable_root_unavailable(rd: Path, task: dict[str, Any], provider: str) -> int:
    """Bloque un dispatch avant tout lancement de worker ou création de PID."""
    update_task(
        rd,
        task,
        status="blocked",
        blocked_reason="writable_root_unavailable",
        writable_root_provider=provider,
    )
    append_event(
        rd,
        {
            "event": "task.dispatch_refused",
            "task_id": task["task_id"],
            "reason": "writable_root_unavailable",
            "provider": provider,
        },
    )
    print("BLOCKED llm_process: writable_root_unavailable", file=sys.stderr)
    return 2


def render_agent_command(
    provider: str,
    model: str,
    prompt_file: Path,
    final_path: Path,
    stdout_path: Path,
    stderr_path: Path,
    template: str,
    cwd: Path | None = None,
    writable_roots: list[Path] | None = None,
) -> str:
    values = {
        "model": model,
        "prompt_file": str(prompt_file),
        "final_path": str(final_path),
        "stdout_path": str(stdout_path),
        "stderr_path": str(stderr_path),
    }
    if template:
        return template.format(**{k: shlex.quote(v) for k, v in values.items()})
    if provider == "codex_exec":
        if not shutil_which("codex"):
            raise SystemExit("codex CLI not found")
        if cwd is None or writable_roots is None:
            raise SystemExit("writable_root_unavailable")
        parts = ["codex", "exec", "--json", "--skip-git-repo-check", "-C", str(cwd), "-o", str(final_path)]
        parts.extend(flag for root in writable_roots if root != cwd for flag in ("--add-dir", str(root)))
        if model:
            parts.extend(["--model", model])
        parts.extend(["-"])
        return " ".join(shlex.quote(part) for part in parts) + f" < {shlex.quote(str(prompt_file))}"
    if provider == "claude_print":
        if not shutil_which("claude"):
            raise SystemExit("claude CLI not found")
        parts = ["claude", "--print", "--permission-mode", "bypassPermissions", "--output-format", "stream-json"]
        if model:
            parts.extend(["--model", model])
        parts.extend(["--"])
        return " ".join(shlex.quote(part) for part in parts) + f" < {shlex.quote(str(prompt_file))}"
    if provider == "gclaude":
        # gclaude est defini dans /Users/moi/.env.local : glm puis Claude CLI
        # via proxy Anthropic Z.AI. On source ce fichier explicitement pour ne
        # pas dependre d'un alias interactif charge par hasard.
        selected_model = model or "glm-5.2"
        model_arg = f" --model {shlex.quote(selected_model)}"
        inner = (
            f"source /Users/moi/.env.local >/dev/null 2>&1; "
            f"glm >/dev/null; "
            f"claude --print --dangerously-skip-permissions --permission-mode bypassPermissions "
            f"--output-format stream-json{model_arg} < {shlex.quote(str(prompt_file))}"
        )
        return f"zsh -lic {shlex.quote(inner)}"
    raise SystemExit(f"unknown agent provider: {provider}")


def start_llm_process(rd: Path, root: str, run_id: str, task: dict[str, Any]) -> int:
    task_id = task["task_id"]
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    provider = task.get("agent_provider") or policies.get("default_agent_provider") or "codex_exec"
    model = task.get("agent_model") or policies.get("default_agent_model") or ""
    writable_config = writable_roots_for_llm(rd, task, provider)
    if writable_config is None:
        return refuse_writable_root_unavailable(rd, task, provider)
    cwd, writable_roots = writable_config
    logs = log_dir(rd, task_id)
    stdout_path = logs / "codex.stdout.jsonl"
    stderr_path = logs / "codex.stderr.log"
    final_path = logs / "codex.final.txt"
    prompt_file = logs / "prompt.txt"
    prompt = (
        dispatch_message(rd, task)
        + "\n\n"
        + f"Tu es un agent non-tmux lance par agent-loop via provider={provider}. "
        + "Tu dois produire le resultat au chemin result_file si la tache demande file ou both. "
        + "Reponds court en final."
    )
    prompt_file.write_text(prompt, encoding="utf-8")
    try:
        command = render_agent_command(
            provider,
            model,
            prompt_file,
            final_path,
            stdout_path,
            stderr_path,
            task.get("agent_command_template") or "",
            cwd,
            writable_roots,
        )
    except SystemExit as exc:
        update_task(rd, task, status="blocked", blocked_reason=str(exc))
        append_event(rd, {"event": "task.blocked", "task_id": task_id, "reason": str(exc)})
        print(f"BLOCKED llm_process: {exc}")
        return 2
    wrapper = (
        "set +e\n"
        f"cd {shlex.quote(str(rd))}\n"
        f"{command} > {shlex.quote(str(stdout_path))} 2> {shlex.quote(str(stderr_path))}\n"
        "rc=$?\n"
        "if [ -f "
        + shlex.quote(str(result_path(rd, task_id)))
        + " ]; then exit 0; fi\n"
        f"if [ ! -s {shlex.quote(str(final_path))} ] && [ -s {shlex.quote(str(stdout_path))} ]; then tail -c 12000 {shlex.quote(str(stdout_path))} > {shlex.quote(str(final_path))}; fi\n"
        "if [ \"$rc\" -eq 0 ]; then result_status=review; else result_status=fail; fi\n"
        f"{shlex.quote(sys.executable)} {shlex.quote(str(Path(__file__)))} worker-result "
        f"--root {shlex.quote(str(Path(root).expanduser()))} "
        f"--run-id {shlex.quote(run_id)} "
        f"--task-id {shlex.quote(task_id)} "
        f"--worker {shlex.quote(provider)} "
        "--status \"$result_status\" "
        "--message \"LLM process finished without task-authored result.json; see logs\" "
        f"--logs-json {shlex.quote(json.dumps({'prompt': str(prompt_file), 'stdout': str(stdout_path), 'stderr': str(stderr_path), 'final': str(final_path)}, ensure_ascii=False))}\n"
    )
    proc = subprocess.Popen(["zsh", "-lc", wrapper], cwd=str(rd), start_new_session=True)
    update_task(
        rd,
        task,
        status="running",
        dispatched_at=now(),
        dispatch_backend="llm_process",
        agent_provider=provider,
        agent_model=model,
        pid=proc.pid,
        logs={"prompt": str(prompt_file), "stdout": str(stdout_path), "stderr": str(stderr_path), "final": str(final_path)},
        attempts=int(task.get("attempts", 0)) + 1,
    )
    append_event(rd, {"event": "task.dispatched", "task_id": task_id, "backend": "llm_process", "provider": provider, "model": model, "pid": proc.pid})
    print(f"DISPATCHED llm_process provider={provider} model={model or '-'} pid={proc.pid}")
    return 0


def render_interactive_agent_command(provider: str, model: str, prompt: str) -> str:
    if provider == "gclaude":
        selected_model = model or "glm-5.2"
        inner = (
            "source /Users/moi/.env.local >/dev/null 2>&1; "
            "glm >/dev/null; "
            "claude --dangerously-skip-permissions --permission-mode bypassPermissions "
            f"--model {shlex.quote(selected_model)} {shlex.quote(prompt)}"
        )
        return f"zsh -lic {shlex.quote(inner)}"
    if provider == "claude_print":
        selected_model = f" --model {shlex.quote(model)}" if model else ""
        return (
            "claude --dangerously-skip-permissions --permission-mode bypassPermissions"
            f"{selected_model} {shlex.quote(prompt)}"
        )
    if provider == "codex_exec":
        selected_model = f" --model {shlex.quote(model)}" if model else ""
        return f"codex{selected_model} {shlex.quote(prompt)}"
    raise SystemExit(f"unknown interactive agent provider: {provider}")


def cmd_dispatch(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    run = load_json(rd / "run.json", {}) or {}
    task = load_task(rd, args.task_id)
    if run.get("status") == "closed":
        raise SystemExit("run is closed")
    role_verdict: dict[str, Any] | None = None
    if task.get("backend") == "existing_tmux" and _role_recovery_is_enforced(rd):
        target = str(task.get("agent_target") or "")
        prospective = dict(task)
        prospective["status"] = "dispatched"
        role_verdict = evaluate_role_recovery_dispatch(
            rd,
            prospective,
            current_agent=target,
            cli_family=_role_recovery_cli_family(task),
        )
        if not role_verdict["allowed"]:
            reason_codes = role_verdict["reason_codes"]
            if args.dry_run:
                print("BLOCKED role recovery gate: " + ",".join(reason_codes))
                return 2
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason="role_recovery_gate_failed",
                role_recovery_reason_codes=reason_codes,
                role_recovery_checked_at=now(),
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_refused",
                    "task_id": args.task_id,
                    "reason": "role_recovery_gate_failed",
                    "reason_codes": reason_codes,
                },
            )
            print(
                "BLOCKED role recovery gate: " + ",".join(reason_codes),
                file=sys.stderr,
            )
            return 2
    errors = collect_errors(rd)
    if errors:
        for err in errors:
            print(f"ERROR {err}", file=sys.stderr)
        return 1
    gate_allowed, gate_reason = gate_preflight_verdict(rd, task)
    if not gate_allowed:
        if args.dry_run:
            print("BLOCKED gate_preflight: " + gate_reason)
            return 2
        update_task(
            rd,
            task,
            status="blocked",
            blocked_reason=gate_reason,
            gate_preflight_checked_at=now(),
        )
        append_event(
            rd,
            {
                "event": "task.dispatch_refused",
                "task_id": args.task_id,
                "reason": gate_reason,
            },
        )
        print("BLOCKED gate_preflight: " + gate_reason, file=sys.stderr)
        return 2
    backend = task.get("backend")
    if (
        task.get("status") == "blocked"
        and int(task.get("attempts", 0)) >= int(task.get("max_attempts", 2))
        and not args.force
    ):
        raise SystemExit("task max_attempts reached; supersede it or use --force")
    if task.get("status") not in {"pending", "blocked"} and not args.force:
        raise SystemExit(f"task status is {task.get('status')!r}; use --force to dispatch anyway")
    if backend == "local":
        command = args.runner_command or task.get("runner_command") or ""
        if not command:
            update_task(rd, task, status="blocked", blocked_reason="local_dispatch_requires_runner_command")
            append_event(rd, {"event": "task.blocked", "task_id": args.task_id, "reason": "local_dispatch_requires_runner_command"})
            print("BLOCKED local task: missing runner_command.")
            return 2
        if args.dry_run:
            print(command)
            return 0
        return run_local_command(rd, task, command)
    if backend == "existing_bridget":
        target = str(task.get("agent_target") or "").strip()
        if not target:
            raise SystemExit(f"{args.task_id}: existing_bridget requires agent_target")
        authorized, assigned, authorized_agents = validate_authorized_recipient(task)
        if not authorized:
            if args.dry_run:
                print(invalid_recipient_message(assigned, authorized_agents))
                return 2
            return refuse_invalid_recipient(rd, task, assigned, authorized_agents)
        msg = dispatch_message(rd, task)
        if args.dry_run:
            print(msg)
            print(f"declared_target: {target}")
            return 0
        agent = resolve_bridget_target(target)
        resolved_target = str(agent["agent_id"])
        previous_result_path = archive_previous_result(rd, args.task_id)
        attempts = int(task.get("attempts", 0)) + 1
        update_task(
            rd,
            task,
            run_id=str(run.get("run_id") or rd.name),
            status="dispatched",
            dispatched_at=now(),
            dispatch_backend="existing_bridget",
            resolved_target=resolved_target,
            collected_at="",
            result_path="",
            previous_result_path=previous_result_path,
            acknowledged_at="",
            acknowledged_by="",
            started_at="",
            attempts=attempts,
            attempt_id=str(task.get("attempt_id") or f"{args.task_id}-attempt-{attempts}"),
            dispatch_epoch=int(task.get("dispatch_epoch") or attempts),
            fencing_epoch=int(task.get("fencing_epoch") or task.get("lease_epoch") or attempts),
            binding_generation=str(task.get("binding_generation") or "bridget-directory"),
            input_sha256=input_snapshot(task),
            progress_baseline=progress_snapshot(task),
            last_progress_at="", last_progress_hash="", progress_seen_hashes=[], dispositioned_at="", disposition={},
            bridget_agent_id=resolved_target,
            agent_name=agent.get("display_name", ""),
            agent_type=agent.get("agent_type", ""),
            agent_model=agent.get("model", ""),
            agent_transport=agent.get("transport", ""),
            agent_domain=agent.get("domain", ""),
        )
        try:
            send_bridget_message(resolved_target, msg)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired, OSError) as exc:
            returncode = getattr(exc, "returncode", -1)
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason="dispatch_transport_failed",
                transport_returncode=returncode,
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_failed",
                    "task_id": args.task_id,
                    "backend": "existing_bridget",
                    "target": target,
                    "resolved_target": resolved_target,
                    "returncode": returncode,
                },
            )
            print(f"BLOCKED existing_bridget transport failed rc={returncode}", file=sys.stderr)
            return 2
        append_event(
            rd,
            {
                "event": "task.dispatched",
                "task_id": args.task_id,
                "backend": "existing_bridget",
                "target": target,
                "resolved_target": resolved_target,
                "agent_type": agent.get("agent_type", ""),
                "agent_model": agent.get("model", ""),
                "transport": agent.get("transport", ""),
            },
        )
        print(
            f"DISPATCHED existing_bridget {target} -> {resolved_target} "
            f"({agent.get('agent_type') or '?'}, {agent.get('model') or '?'})"
        )
        return 0
    if backend == "existing_tmux":
        target = task.get("agent_target")
        if not target:
            raise SystemExit(f"{args.task_id}: existing_tmux requires agent_target")
        authorized, assigned, authorized_agents = validate_authorized_recipient(task)
        if not authorized:
            if args.dry_run:
                print(invalid_recipient_message(assigned, authorized_agents))
                return 2
            return refuse_invalid_recipient(rd, task, assigned, authorized_agents)
        binding_allowed, binding_reason, binding = active_existing_tmux_binding(task)
        if not binding_allowed:
            if args.dry_run:
                print(f"BLOCKED active binding: {binding_reason}")
                return 2
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason=binding_reason,
                binding_checked_at=now(),
                binding_source_ref=str(BOOT_ROLE_BINDINGS_PATH),
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_refused",
                    "task_id": args.task_id,
                    "reason": binding_reason,
                    "target": target,
                },
            )
            print(f"BLOCKED active binding: {binding_reason}", file=sys.stderr)
            return 2
        msg = dispatch_message(rd, task)
        if args.dry_run:
            print(msg)
            print(f"declared_target: {target}")
            print(f"active_binding: {binding['agent_name']}@{binding['pane_id']}")
            if role_verdict:
                print(
                    "role_recovery_gate: confirmed "
                    f"policy={ROLE_RECOVERY_POLICY_VERSION}"
                )
            return 0
        resolved_target = resolve_agent_target(target)
        if str(resolved_target).startswith("%") and resolved_target != binding["pane_id"]:
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason="target_out_of_binding",
                binding_checked_at=now(),
                binding_source_ref=binding["source_ref"],
                resolved_target=resolved_target,
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_refused",
                    "task_id": args.task_id,
                    "reason": "target_out_of_binding",
                    "target": target,
                    "resolved_target": resolved_target,
                    "bound_pane": binding["pane_id"],
                },
            )
            print("BLOCKED active binding: target_out_of_binding", file=sys.stderr)
            return 2
        agent_info = bridge_model_info(resolved_target)
        resolved_agent = str(agent_info.get("agent_name") or "").strip()
        assigned_agent = str(task.get("assigned_agent") or "").strip()
        if (
            resolved_agent
            and assigned_agent
            and normalize_agent_alias(resolved_agent) != normalize_agent_alias(assigned_agent)
        ):
            if args.dry_run:
                print(
                    f"BLOCKED resolved recipient mismatch assigned={assigned_agent} resolved={resolved_agent}"
                )
                return 2
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason="role_recovery_resolved_recipient_mismatch",
                role_recovery_reason_codes=["assigned_agent_mismatch"],
                role_recovery_checked_at=now(),
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_refused",
                    "task_id": args.task_id,
                    "reason": "role_recovery_resolved_recipient_mismatch",
                    "assigned_agent": assigned_agent,
                    "resolved_agent": resolved_agent,
                },
            )
            print(
                f"BLOCKED resolved recipient mismatch assigned={assigned_agent} resolved={resolved_agent}",
                file=sys.stderr,
            )
            return 2
        previous_result_path = archive_previous_result(rd, args.task_id)
        update_fields = {
            "run_id": str(run.get("run_id") or rd.name),
            "status": "dispatched",
            "dispatched_at": now(),
            "dispatch_backend": "existing_tmux",
            "resolved_target": resolved_target,
            "collected_at": "",
            "result_path": "",
            "previous_result_path": previous_result_path,
            "acknowledged_at": "",
            "acknowledged_by": "",
            "started_at": "",
            "attempts": int(task.get("attempts", 0)) + 1,
            "attempt_id": str(task.get("attempt_id") or f"{args.task_id}-attempt-{int(task.get('attempts', 0)) + 1}"),
            "dispatch_epoch": int(task.get("dispatch_epoch") or int(task.get("attempts", 0)) + 1),
            "fencing_epoch": int(task.get("fencing_epoch") or task.get("lease_epoch") or int(task.get("attempts", 0)) + 1),
            "binding_generation": str(task.get("binding_generation") or "dispatch-pending"),
            "active_boot_binding": binding,
            "input_sha256": input_snapshot(task),
            "progress_baseline": progress_snapshot(task),
            "last_progress_at": "", "last_progress_hash": "", "progress_seen_hashes": [], "dispositioned_at": "", "disposition": {},
        }
        if role_verdict:
            update_fields.update(
                role_recovery_gate="confirmed",
                role_recovery_checked_at=role_verdict["assertion"]["observed_at"],
                role_recovery_policy_version=ROLE_RECOVERY_POLICY_VERSION,
                binding_generation=role_verdict["assertion"]["binding_generation"],
                binding_source_refs=role_verdict["assertion"].get("source_refs", []),
            )
            pane_pid = "0"; cli_pid = "0"; cli_start = "unknown"; tty = "unknown"; executable = str(role_verdict["assertion"].get("cli_family") or "unknown")
            try:
                if not str(resolved_target).startswith('%'):
                    raise RuntimeError('runtime pane metadata unavailable')
                pane_info = subprocess.run(["tmux", "display-message", "-p", "-t", resolved_target, "#{pane_pid}"], capture_output=True, text=True, timeout=2)
                pane_pid = pane_info.stdout.strip()
                child = subprocess.run(["pgrep", "-P", pane_pid], capture_output=True, text=True, timeout=2).stdout.split()
                cli_pid = child[0] if child else pane_pid
                raw_start = subprocess.run(["ps", "-p", cli_pid, "-o", "lstart="], capture_output=True, text=True, timeout=2).stdout.strip()
                raw_exec = subprocess.run(["ps", "-p", cli_pid, "-o", "comm="], capture_output=True, text=True, timeout=2).stdout.strip()
                raw_tty = subprocess.run(["ps", "-p", cli_pid, "-o", "tty="], capture_output=True, text=True, timeout=2).stdout.strip()
                if raw_start:
                    cli_start = datetime.strptime(raw_start, "%a %b %d %H:%M:%S %Y").replace(tzinfo=timezone.utc).isoformat().replace("+00:00", "Z")
                if raw_exec: executable = os.path.realpath(raw_exec)
                if raw_tty and raw_tty != "??": tty = raw_tty if raw_tty.startswith("/dev/") else "/dev/" + raw_tty
            except (OSError, RuntimeError, subprocess.SubprocessError):
                pass
            try:
                producer = runtime_binding_producer()
            except Exception as exc:
                update_task(
                    rd, task, status="blocked", blocked_reason="binding_producer_unavailable",
                    binding_error=str(exc), role_recovery_gate="failed",
                )
                append_event(rd, {
                    "event": "task.dispatch_refused",
                    "task_id": args.task_id,
                    "reason": "binding_producer_unavailable",
                    "error": str(exc),
                })
                print("BLOCKED binding producer unavailable", file=sys.stderr)
                return 2
            binding_run_id = str(run.get("run_id") or rd.name)
            if not binding_run_id.startswith("maicompany-"):
                binding_run_id = f"maicompany-{binding_run_id}"
            if str(resolved_target).startswith('%'):
                try:
                    boot_context = canonical_boot_context(
                        agent=str(role_verdict["assertion"].get("agent_id") or ""),
                        pane=resolved_target,
                        cli=str(role_verdict["assertion"].get("cli_family") or ""),
                    )
                    if boot_context["source_ref"] not in update_fields["binding_source_refs"]:
                        update_fields["binding_source_refs"].append(boot_context["source_ref"])
                    binding_payload = producer.produce_binding_payload(
                        run_id=binding_run_id, task_id=args.task_id,
                        attempt_id=update_fields["attempt_id"],
                        dispatch_epoch=update_fields["dispatch_epoch"],
                        fencing_epoch=update_fields["fencing_epoch"],
                        agent=str(role_verdict["assertion"].get("agent_id") or ""),
                        pane=resolved_target,
                        cli=str(role_verdict["assertion"].get("cli_family") or ""),
                        binding_sources=update_fields.get("binding_source_refs", []),
                        invocation_id=boot_context["invocation_id"],
                        boot_cycle=boot_context["boot_cycle"],
                        cli_pid=int(cli_pid) if str(cli_pid).isdigit() else 0,
                        cli_start_time=cli_start, executable=executable, tty=tty,
                    )
                    producer.register_runtime_binding(binding_payload, root=ROLE_RECOVERY_COORDINATION_ROOT)
                except Exception as exc:
                    update_task(
                        rd, task, status="blocked", blocked_reason="binding_payload_invalid",
                        binding_error=str(exc), role_recovery_gate="failed",
                    )
                    append_event(rd, {
                        "event": "task.dispatch_refused",
                        "task_id": args.task_id,
                        "reason": "binding_payload_invalid",
                        "error": str(exc),
                    })
                    print("BLOCKED binding payload invalid", file=sys.stderr)
                    return 2
            else:
                # Cible abstraite utilisée uniquement par les tests hors tmux;
                # aucun binding runtime n'est publié sans pane concret.
                binding_payload = {"schema_version": "mai-binding-payload-v2", "run_id": binding_run_id,
                    "task_id": args.task_id, "attempt_id": update_fields["attempt_id"],
                    "dispatch_epoch": update_fields["dispatch_epoch"], "fencing_epoch": update_fields["fencing_epoch"],
                    "agent": role_verdict["assertion"].get("agent_id"), "pane": resolved_target,
                    "cli": role_verdict["assertion"].get("cli_family"), "binding_sources": update_fields.get("binding_source_refs", []),
                    "invocation_id": "test-binding", "boot_cycle": "test-binding", "cli_pid": 1,
                    "cli_start_time": "1970-01-01T00:00:00Z", "executable": "/usr/bin/test", "tty": "/dev/ttys0"}
            update_fields["binding_payload"] = binding_payload
            update_fields["binding_generation"] = hashlib.sha256(json.dumps(binding_payload, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
            if str(resolved_target).startswith('%'):
                valid = True
                try:
                    producer.validate_binding_payload(binding_payload)
                except Exception:
                    valid = False
                if not valid:
                    update_task(rd, task, status="blocked", blocked_reason="binding_payload_invalid", role_recovery_gate="failed")
                    print("BLOCKED binding payload invalid", file=sys.stderr)
                    return 2
        update_fields.update({k: v for k, v in agent_info.items() if v})
        update_task(rd, task, **update_fields)
        cmd = [str(BRIDGE), "send", "--to", resolved_target, "--no-reply", "--hops", "1", msg]
        try:
            subprocess.run(cmd, check=True, env=bridge_background_environment("dispatch"))
        except subprocess.CalledProcessError as exc:
            update_task(
                rd,
                task,
                status="blocked",
                blocked_reason="dispatch_transport_failed",
                transport_returncode=exc.returncode,
            )
            append_event(
                rd,
                {
                    "event": "task.dispatch_failed",
                    "task_id": args.task_id,
                    "backend": "existing_tmux",
                    "target": target,
                    "resolved_target": resolved_target,
                    "returncode": exc.returncode,
                },
            )
            print(f"BLOCKED existing_tmux transport failed rc={exc.returncode}", file=sys.stderr)
            return 2
        event = {
            "event": "task.dispatched",
            "task_id": args.task_id,
            "backend": "existing_tmux",
            "target": target,
            "resolved_target": resolved_target,
        }
        event.update({k: v for k, v in agent_info.items() if v})
        append_event(rd, event)
        model = agent_info.get("agent_model") or "?"
        typ = agent_info.get("agent_type") or "?"
        print(f"DISPATCHED existing_tmux {target} -> {resolved_target} ({typ}, {model})")
        return 0
    if backend == "background_process":
        command = args.runner_command or task.get("runner_command") or ""
        if not command:
            update_task(rd, task, status="blocked", blocked_reason="background_process_requires_runner_command")
            append_event(rd, {"event": "task.blocked", "task_id": args.task_id, "reason": "background_process_requires_runner_command"})
            print("BLOCKED background_process task: missing runner_command.")
            return 2
        if args.dry_run:
            print(command)
            return 0
        previous_result_path = archive_previous_result(rd, args.task_id)
        task["previous_result_path"] = previous_result_path
        task["collected_at"] = ""
        task["result_path"] = ""
        return start_background_process(rd, args.root, args.run_id, task, command)
    if backend == "spawn_tmux":
        run = load_json(rd / "run.json", {})
        policies = run.get("policies", {})
        if not policies.get("allow_spawn_tmux", False):
            print("BLOCKED_BY_POLICY: spawn_tmux requested but allow_spawn_tmux=false", file=sys.stderr)
            return 1
        command = args.spawn_command
        provider = task.get("agent_provider") or ""
        model = task.get("agent_model") or ""
        if not provider and policies.get("spawn_tmux_uses_default_agent", False):
            provider = policies.get("default_agent_provider", "")
            model = model or policies.get("default_agent_model", "")
        if not command:
            if provider:
                command = render_interactive_agent_command(provider, model, dispatch_message(rd, task))
            else:
                command = (
                    f"{shlex.quote(sys.executable)} {shlex.quote(str(Path(__file__)))} "
                    f"worker-result --root {shlex.quote(str(Path(args.root).expanduser()))} "
                    f"--run-id {shlex.quote(args.run_id)} --task-id {shlex.quote(args.task_id)} "
                    "--status pass --message 'spawn_tmux test worker completed'; sleep 2"
                )
        if args.dry_run:
            print(command)
            return 0
        previous_result_path = archive_previous_result(rd, args.task_id)
        tmux_cmd = ["tmux", "split-window", "-d", "-P", "-F", "#{pane_id}", "-c", str(Path.cwd()), "zsh", "-lc", command]
        pane = subprocess.check_output(tmux_cmd, text=True).strip()
        update_task(
            rd,
            task,
            status="running",
            dispatched_at=now(),
            dispatch_backend="spawn_tmux",
            pane=pane,
            agent_provider=provider,
            agent_model=model,
            collected_at="",
            result_path="",
            previous_result_path=previous_result_path,
        )
        append_event(
            rd,
            {
                "event": "task.dispatched",
                "task_id": args.task_id,
                "backend": "spawn_tmux",
                "pane": pane,
                "agent_provider": provider,
                "agent_model": model,
            },
        )
        print(f"DISPATCHED spawn_tmux {pane}")
        return 0
    if backend in {"codex_thread", "llm_process"}:
        run = load_json(rd / "run.json", {})
        policies = run.get("policies", {})
        provider = task.get("agent_provider") or policies.get("default_agent_provider") or "codex_exec"
        if writable_roots_for_llm(rd, task, provider) is None:
            if args.dry_run:
                print("BLOCKED llm_process: writable_root_unavailable", file=sys.stderr)
                return 2
            return refuse_writable_root_unavailable(rd, task, provider)
        if args.dry_run:
            print(dispatch_message(rd, task))
            model = task.get("agent_model") or policies.get("default_agent_model", "")
            print(f"backend: llm_process provider={provider} model={model or '-'}")
            return 0
        previous_result_path = archive_previous_result(rd, args.task_id)
        task["previous_result_path"] = previous_result_path
        task["collected_at"] = ""
        task["result_path"] = ""
        return start_llm_process(rd, args.root, args.run_id, task)
    raise SystemExit(f"dispatch not implemented for backend {backend!r}")


TERMINAL_STATUSES = {"pass", "fail", "blocked", "review"}
ACTIVE_TASK_STATUSES = {"dispatched", "running"}


def task_status_map(rd: Path) -> dict[str, str]:
    return {p.stem: load_json(p, {}).get("status", "") for p in (rd / "tasks").glob("*.json")}


def deps_satisfied(task: dict[str, Any], statuses: dict[str, str]) -> bool:
    return all(statuses.get(dep) == "pass" for dep in task.get("depends_on", []))


def conflicts_active(task: dict[str, Any], statuses: dict[str, str]) -> list[str]:
    return [tid for tid in task.get("conflicts_with", []) if statuses.get(tid) in ACTIVE_TASK_STATUSES]


GATE_PREFLIGHT_VALUES = {"refuse_if_no_passing_review", "bootstrap_self_test"}


def review_passed(rd: Path, review_task_id: str) -> bool:
    """Indique si la revue liée autorise explicitement l'implémentation."""
    if not review_task_id:
        return False
    result = load_json(result_path(rd, review_task_id), {}) or {}
    if str(result.get("status", "")).lower() == "pass":
        return True
    if result.get("implementation_dispatch_allowed") is True:
        return True
    verdict = str(result.get("business_verdict") or result.get("verdict") or "").upper()
    return (
        verdict.startswith("PASS")
        or verdict == "GO"
        or verdict.startswith("GO_PLAN")
        or verdict.startswith("GO_IMPLEMENT")
    )


def gate_preflight_verdict(rd: Path, task: dict[str, Any]) -> tuple[bool, str]:
    """Refuse une tâche protégée tant que sa revue n'est pas passée.

    ``bootstrap_self_test`` est réservé à T099 pour éviter la circularité du
    premier préflight ; les tâches sans préflight conservent leur comportement
    historique.
    """
    gate = task.get("gate_preflight")
    if not gate or gate not in GATE_PREFLIGHT_VALUES:
        return True, ""
    if gate == "bootstrap_self_test":
        return True, ""
    review_id = str(task.get("gate_preflight_review_task_id") or "")
    if not review_passed(rd, review_id):
        return False, "gate_preflight_no_passing_review"
    return True, ""


def collect_task_results(rd: Path) -> list[dict[str, Any]]:
    collected = []
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        tid = task.get("task_id", path.stem)
        rp = result_path(rd, tid)
        if task.get("status") in ACTIVE_TASK_STATUSES and rp.exists() and not task.get("collected_at"):
            result = load_json(rp, {})
            status = result.get("status")
            changed_inputs = input_snapshot_changed(task)
            if changed_inputs:
                status = "blocked"
                result["validation_error"] = "input_changed_after_dispatch"
                result["changed_input_paths"] = changed_inputs
                write_json(rp, result)
            elif status not in TERMINAL_STATUSES:
                status = "blocked"
                result["validation_error"] = "invalid_or_missing_status"
                write_json(rp, result)
            update_task(rd, task, status=status, collected_at=now(), result_path=str(rp))
            receipt = write_continuation_receipt(rd, {**task, "task_id": tid}, result)
            event = {"event": "task.result_collected", "task_id": tid, "status": status, "result_path": str(rp)}
            event["result_id"] = receipt["result_id"]
            event["continuation_receipt"] = str(rd / "receipts" / f"{receipt['result_id']}.json")
            append_event(rd, event)
            maybe_close_spawned_pane(rd, {**task, "status": status})
            collected.append(event)
        elif task.get("status") in TERMINAL_STATUSES and rp.exists() and not task.get("pane_closed_at"):
            maybe_close_spawned_pane(rd, task)
    return collected


def cmd_reconcile(args: argparse.Namespace) -> int:
    """Réconcilie les faits durables sans inventer une clôture."""
    rd = run_dir(args)
    events = collect_task_results(rd)
    report: list[dict[str, Any]] = list(events)
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        status = task.get("status")
        tid = task.get("task_id", path.stem)
        rp = result_path(rd, tid)
        if status in ACTIVE_TASK_STATUSES and not rp.exists():
            item = {"event": "task.reconciliation_needed", "task_id": tid, "status": status, "reason": "active_without_result"}
            append_event(rd, item)
            report.append(item)
        elif status not in ACTIVE_TASK_STATUSES and status not in TERMINAL_STATUSES and status not in {"pending", "superseded"}:
            item = {"event": "task.invalid_status", "task_id": tid, "status": status, "reason": "status_outside_enum"}
            append_event(rd, item)
            report.append(item)
    print(json.dumps({"run_id": args.run_id, "reconciled": report}, ensure_ascii=False, indent=2))
    return 0


def mission_control_defaults() -> dict[str, Any]:
    return {
        "ack_notice_sec": 120,
        "progress_notice_sec": 300,
        "reminder_interval_sec": 300,
        "escalation_after_reminders": 2,
        "escalation_role": "root",
        "heartbeat_digest_limit": 8,
        "task_due_sec": 3600,
        "paused_work_kinds": [],
    }


def cmd_migrate_run(args: argparse.Namespace) -> int:
    """Migration additive : O(n log n + octets des fichiers déclarés des tâches actives)."""
    rd = run_dir(args)
    run_path = rd / "run.json"
    run = load_json(run_path, None)
    if not run:
        raise SystemExit(f"missing run.json: {rd}")
    if run.get("mission_control_enabled_at"):
        print(f"ALREADY_MIGRATED {args.run_id} enabled_at={run['mission_control_enabled_at']}")
        return 0
    enabled_at = now()
    policies = run.setdefault("policies", {})
    for key, value in mission_control_defaults().items():
        policies.setdefault(key, value)
    run["status"] = run.get("status") or "open"
    run["mission_control_enabled_at"] = enabled_at
    run["updated_at"] = enabled_at
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        if task.get("status") in ACTIVE_TASK_STATUSES and not task.get("progress_baseline"):
            update_task(rd, task, touch=False, progress_baseline=progress_snapshot(task))
    write_json(run_path, run)
    append_event(rd, {"event": "run.mission_control_enabled", "enabled_at": enabled_at})
    print(f"MIGRATED {args.run_id} enabled_at={enabled_at}")
    return 0


def task_requires_disposition(run: dict[str, Any], task: dict[str, Any]) -> bool:
    if task.get("status") not in TERMINAL_STATUSES or task.get("dispositioned_at"):
        return False
    enabled = parse_ts(run.get("mission_control_enabled_at"))
    collected = parse_ts(task.get("collected_at") or task.get("updated_at"))
    return bool(enabled and collected and collected >= enabled)


def close_task_receipts(rd: Path, task_id: str, decision: str) -> None:
    for path in sorted((rd / "receipts").glob("*.json")):
        receipt = load_json(path, {})
        if receipt.get("task_id") != task_id or receipt.get("state") != "open":
            continue
        receipt["state"] = "closed"
        receipt["decision"] = decision
        receipt["closed_at"] = now()
        write_json(path, receipt)


def cmd_disposition(args: argparse.Namespace) -> int:
    """Décision distincte du verdict : O(r log r), r étant le nombre de reçus existants."""
    rd = run_dir(args)
    task = load_task(rd, args.task_id)
    if task.get("status") not in TERMINAL_STATUSES:
        raise SystemExit(f"task status is {task.get('status')!r}; disposition requires a terminal result")
    decision = str(args.decision or "").strip()
    reason_code = str(args.reason_code or "").strip()
    if decision not in MISSION_DISPOSITIONS:
        raise SystemExit(f"invalid disposition: {decision}")
    if reason_code not in MISSION_REASON_CODES:
        raise SystemExit(f"invalid reason-code: {reason_code}")
    owner = str(args.owner or "").strip()
    next_check = str(args.next_check or "").strip()
    successor_task_id = str(args.successor_task_id or "").strip()
    if decision in {"correction_assigned", "next_task_assigned", "blocked"} and not owner:
        raise SystemExit(f"disposition {decision} requires --owner")
    if decision == "blocked" and not next_check:
        raise SystemExit("disposition blocked requires --next-check")
    if decision == "blocked" and (not parse_ts(next_check) or parse_ts(next_check) <= datetime.now(timezone.utc)):
        raise SystemExit("disposition blocked requires a future --next-check")
    if decision in {"correction_assigned", "next_task_assigned"}:
        if not successor_task_id or successor_task_id == args.task_id:
            raise SystemExit("assigned disposition requires a different --successor-task-id")
        successor = load_json(task_path(rd, successor_task_id), {})
        if successor.get("status") not in ACTIVE_TASK_STATUSES or task_agent_identity(successor) != owner:
            raise SystemExit("successor must be dispatched or acknowledged by the declared owner")
    dispositioned_at = now()
    disposition = {
        "decision": decision,
        "reason_code": reason_code,
        "owner": owner,
        "next_check": next_check,
        "successor_task_id": successor_task_id,
    }
    update_task(rd, task, disposition=disposition, dispositioned_at=dispositioned_at)
    close_task_receipts(rd, args.task_id, decision)
    append_event(
        rd,
        {
            "event": "task.disposition_recorded",
            "task_id": args.task_id,
            "verdict": task.get("status"),
            "decision": decision,
            "reason_code": reason_code,
            "owner": owner,
            "next_check": next_check,
            "successor_task_id": successor_task_id,
        },
    )
    print(f"DISPOSITIONED {args.task_id} verdict={task.get('status')} decision={decision}")
    return 0


def run_open_obligations(rd: Path) -> list[str]:
    """Contrôle les obligations en O(n log n), un fichier indépendant par tâche."""
    run = load_json(rd / "run.json", {})
    obligations: list[str] = []
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        tid = str(task.get("task_id") or path.stem)
        status = str(task.get("status") or "")
        if status in ACTIVE_TASK_STATUSES or status == "pending":
            obligations.append(f"{tid}:{status}")
        elif task_requires_disposition(run, task):
            obligations.append(f"{tid}:disposition_required")
        elif status in {"review", "blocked", "fail"} and not task.get("dispositioned_at") and not task.get("successor_task_id"):
            obligations.append(f"{tid}:legacy_disposition_required")
        elif task.get("disposition", {}).get("decision") == "blocked":
            obligations.append(f"{tid}:blocked_followup")
    return obligations


def cmd_close_run(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    run_path = rd / "run.json"
    run = load_json(run_path, None)
    if not run:
        raise SystemExit(f"missing run.json: {rd}")
    obligations = run_open_obligations(rd)
    if obligations:
        sample = ", ".join(obligations[:8])
        raise SystemExit(f"open obligations prevent closure: {sample}")
    closed_at = now()
    run["status"] = "closed"
    run["closed_at"] = closed_at
    run["close_reason"] = str(args.reason or "").strip()
    run["updated_at"] = closed_at
    write_json(run_path, run)
    append_event(rd, {"event": "run.closed", "reason": run["close_reason"]})
    print(f"CLOSED {args.run_id}")
    return 0


def cmd_resolve_task(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    task = load_json(task_path(rd, args.task_id), None)
    if not task:
        raise SystemExit(f"task not found: {args.task_id}")
    run = load_json(rd / "run.json", {})
    if run.get("mission_control_enabled_at") and task.get("status") in TERMINAL_STATUSES:
        raise SystemExit("terminal verdict is immutable; use disposition or relaunch")
    if args.status not in TERMINAL_STATUSES | {"superseded"}:
        raise SystemExit(f"invalid disposition: {args.status}")
    update_task(rd, task, status=args.status, disposition_reason=args.reason, collected_at=task.get("collected_at") or now())
    append_event(rd, {"event": "task.dispositioned", "task_id": args.task_id, "status": args.status, "reason": args.reason})
    print(f"DISPOSITIONED {args.task_id} status={args.status}")
    return 0


def cmd_relaunch(args: argparse.Namespace) -> int:
    """Crée une reprise canonique sans réactiver une tâche terminale."""
    rd = run_dir(args)
    source = load_json(task_path(rd, args.task_id), None)
    if not source:
        raise SystemExit(f"task not found: {args.task_id}")
    if source.get("status") not in {"blocked", "fail", "review"}:
        raise SystemExit("relaunch requires blocked, fail or review source")
    if source.get("successor_task_id"):
        print(source["successor_task_id"])
        return 0
    successor = f"{args.task_id}-relaunch-{int(source.get('attempts') or 0) + 1}"
    task = dict(source)
    task.update({
        "task_id": successor,
        "parent_task_id": args.task_id,
        "status": "pending",
        "created_at": now(),
        "updated_at": now(),
        "collected_at": None,
        "result_path": None,
        "dispatched_at": None,
        "pid": None,
        "attempts": 0,
        "relaunch_policy": None,
        "acknowledged_at": "", "acknowledged_by": "", "started_at": "",
        "last_progress_at": "", "last_progress_hash": "", "last_progress_ref": "",
        "progress_seen_hashes": [],
        "dispositioned_at": "", "disposition": {},
    })
    if args.agent_target:
        task["agent_target"] = args.agent_target
    if args.assigned_agent:
        task["assigned_agent"] = args.assigned_agent
    write_json(task_path(rd, successor), task)
    update_task(rd, source, successor_task_id=successor,
                disposition={"decision": "next_task_assigned", "successor_task_id": successor},
                dispositioned_at=now())
    close_task_receipts(rd, args.task_id, "next_task_assigned")
    append_event(rd, {"event": "task.relaunch_created", "task_id": args.task_id, "successor_task_id": successor, "reason": args.reason})
    print(f"RELAUNCH_CREATED {successor}")
    return 0


def ready_tasks(rd: Path) -> list[dict[str, Any]]:
    statuses = task_status_map(rd)
    run = load_json(rd / "run.json", {})
    paused_work_kinds = set(run.get("policies", {}).get("paused_work_kinds", []))
    ready = []
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        if task.get("status") != "pending":
            continue
        if str(task.get("work_kind") or "generic") in paused_work_kinds:
            continue
        if not deps_satisfied(task, statuses):
            continue
        if conflicts_active(task, statuses):
            continue
        gate_allowed, _gate_reason = gate_preflight_verdict(rd, task)
        if not gate_allowed:
            continue
        ready.append(task)
    return sorted(ready, key=lambda t: (int(t.get("priority", 100)), t.get("created_at", ""), t.get("task_id", "")))


def running_tasks(rd: Path) -> list[dict[str, Any]]:
    return [
        load_json(path, {})
        for path in sorted((rd / "tasks").glob("*.json"))
        if load_json(path, {}).get("status") in ACTIVE_TASK_STATUSES
    ]


def task_agent_identity(task: dict[str, Any]) -> str:
    """Return the stable worker identity used to serialize work per agent."""
    return str(
        task.get("assigned_agent")
        or task.get("agent_target")
        or task.get("resolved_target")
        or ""
    ).strip()


def ready_tasks_for_idle_agents(rd: Path, ready: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Select at most one ready task per idle agent within global capacity."""
    running = running_tasks(rd)
    busy_agents = {
        agent_id
        for task in running
        if (agent_id := task_agent_identity(task))
    }
    run = load_json(rd / "run.json", {})
    max_concurrent = int(run.get("policies", {}).get("max_concurrent_tasks", 3))
    slots = max(0, max_concurrent - len(running))
    selected: list[dict[str, Any]] = []
    selected_lanes: set[str] = set()
    for task in ready:
        if len(selected) >= slots:
            break
        agent_id = task_agent_identity(task)
        lane = agent_id or f"task:{task.get('task_id', '')}"
        if agent_id in busy_agents or lane in selected_lanes:
            continue
        selected.append(task)
        selected_lanes.add(lane)
    return selected


def heartbeat_notice_sec(rd: Path) -> int:
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    return int(policies.get("heartbeat_notice_sec", policies.get("running_task_notice_sec", 300)))


def task_age(task: dict[str, Any], *fields: str) -> int:
    for field in fields:
        if parse_ts(task.get(field)):
            return age_seconds(task.get(field))
    return age_seconds(task.get("updated_at") or task.get("created_at"))


def ready_task_alerts(rd: Path, ready: list[dict[str, Any]], notice_sec: int) -> list[dict[str, Any]]:
    if notice_sec <= 0:
        return []
    alerts: list[dict[str, Any]] = []
    for task in ready:
        age = task_age(task, "ready_at", "created_at", "updated_at")
        alerts.append(
            {
                "event": "task.ready_waiting",
                "task_id": task.get("task_id", ""),
                "status": "pending",
                "backend": task.get("backend", ""),
                "delegate": task.get("delegate"),
                "age_sec": age,
                "age_bucket": age_bucket(age, notice_sec),
                "agent_target": task.get("agent_target", ""),
                "notice_sec": notice_sec,
                "due_at": task.get("due_at", ""),
                "evidence_marker": str(task.get("updated_at") or task.get("created_at") or "legacy"),
                "routing": "coordinator",
            }
        )
    return alerts


def collected_task_alerts(rd: Path, notice_sec: int) -> list[dict[str, Any]]:
    if notice_sec <= 0:
        return []
    actionable_statuses = TERMINAL_STATUSES
    run = load_json(rd / "run.json", {})
    alerts: list[dict[str, Any]] = []
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        status = str(task.get("status") or "").strip().lower()
        if status not in actionable_statuses:
            continue
        if run.get("mission_control_enabled_at") and not task_requires_disposition(run, task):
            continue
        if not run.get("mission_control_enabled_at") and status not in {"review", "blocked", "fail"}:
            continue
        tid = task.get("task_id", path.stem)
        rp = task.get("result_path") or str(result_path(rd, tid))
        age = task_age(task, "collected_at", "updated_at", "created_at")
        alerts.append(
            {
                "event": "task.collected_needs_arbitration",
                "task_id": tid,
                "status": status,
                "age_sec": age,
                "age_bucket": age_bucket(age, notice_sec),
                "result_path": rp,
                "notice_sec": notice_sec,
                "due_at": task.get("due_at", ""),
                "evidence_marker": str(task.get("collected_at") or task.get("updated_at") or "legacy"),
                "routing": "coordinator",
            }
        )
    return alerts


def relaunch_required_alerts(rd: Path, notice_sec: int) -> list[dict[str, Any]]:
    """Expose une reprise obligatoire pour les livraisons sans suite."""
    alerts: list[dict[str, Any]] = []
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        status = str(task.get("status") or "").strip().lower()
        if status not in {"blocked", "fail", "review"} or task.get("successor_task_id"):
            continue
        if not task.get("relaunch_policy"):
            continue
        age = task_age(task, "collected_at", "updated_at", "created_at")
        if age < max(1, notice_sec):
            continue
        alerts.append({
            "event": "task.relaunch_required",
            "task_id": task.get("task_id", path.stem),
            "status": status,
            "age_sec": age,
            "age_bucket": age_bucket(age, notice_sec),
            "reason": "terminal_delivery_without_successor",
            "relaunch_policy": task.get("relaunch_policy"),
            "due_at": task.get("due_at", ""),
            "evidence_marker": str(task.get("collected_at") or task.get("updated_at") or "legacy"),
            "routing": "coordinator",
        })
    return alerts


def continuation_task_alerts(rd: Path) -> list[dict[str, Any]]:
    """Agrège les décisions attendues en O(n log n), sans contenu de résultat."""
    alerts: list[dict[str, Any]] = []
    legacy_ids: list[str] = []
    run = load_json(rd / "run.json", {})
    enabled = parse_ts(run.get("mission_control_enabled_at"))
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        tid = str(task.get("task_id") or path.stem)
        disposition = task.get("disposition", {})
        check_at = parse_ts(disposition.get("next_check"))
        if disposition.get("decision") == "blocked" and check_at and check_at <= datetime.now(timezone.utc):
            alerts.append({
                "event": "task.blocker_check_due", "task_id": tid, "status": task.get("status"),
                "evidence_marker": str(task.get("dispositioned_at")), "routing": "coordinator",
                "due_at": disposition.get("next_check"), "age_sec": age_seconds(disposition.get("next_check")),
            })
        collected = parse_ts(task.get("collected_at") or task.get("updated_at"))
        if (enabled and collected and collected < enabled and task.get("status") in {"review", "blocked", "fail"}
                and not task.get("dispositioned_at") and not task.get("successor_task_id")):
            legacy_ids.append(tid)
    if legacy_ids:
        alerts.append({
            "event": "run.legacy_decisions_open", "task_id": "", "status": "open",
            "evidence_marker": hashlib.sha256("\n".join(legacy_ids).encode()).hexdigest(),
            "routing": "coordinator", "obligation_count": len(legacy_ids), "age_sec": 0,
        })
    return alerts


def running_task_alerts(rd: Path, notice_sec: int | None = None) -> list[dict[str, Any]]:
    """Parcours O(n log n), annuaire Bridget lu une fois et indexé par UUID."""
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    ack_notice_sec = int(notice_sec if notice_sec is not None else policies.get("ack_notice_sec", 120))
    progress_notice_sec = int(notice_sec if notice_sec is not None else policies.get("progress_notice_sec", 300))
    if ack_notice_sec <= 0 and progress_notice_sec <= 0:
        return []
    tasks = running_tasks(rd)
    needs_tmux = any(task.get("backend") == "existing_tmux" for task in tasks)
    needs_bridget = any(task.get("backend") == "existing_bridget" for task in tasks)
    live = agent_by_pane() if needs_tmux else {}
    bridget_live = bridget_agents_by_id() if needs_bridget else {}
    alerts: list[dict[str, Any]] = []
    for task in tasks:
        tid = task.get("task_id", "")
        if result_path(rd, tid).exists():
            continue
        status = str(task.get("status") or "")
        if status == "dispatched":
            observed_at = task.get("dispatched_at") or task.get("updated_at")
            threshold = ack_notice_sec
            event_name = "task.dispatched_no_ack"
        else:
            observed_at = task.get("last_progress_at") or task.get("acknowledged_at") or task.get("started_at") or task.get("updated_at")
            threshold = progress_notice_sec
            event_name = "task.running_no_progress"
        age = age_seconds(observed_at)
        target = task.get("resolved_target") or task.get("pane") or ""
        if task.get("backend") == "existing_bridget":
            agent_live = bool(
                target
                and target in bridget_live
                and bridget_agent_connected(bridget_live[target])
            )
        else:
            agent_live = bool(target and target in live)
        pid = task.get("pid")
        process_dead = bool(pid) and not process_exists(pid)
        pane_active = pane_worker_active(target) if task.get("backend") == "existing_tmux" else None
        pane_dead = pane_active is False
        process_dead = process_dead or pane_dead
        timeout_sec = int(task.get("timeout_sec") or 0)
        total_age = age_seconds(task.get("dispatched_at") or task.get("started_at"))
        due = parse_ts(task.get("due_at"))
        timed_out = (timeout_sec > 0 and total_age >= timeout_sec) or bool(due and due <= datetime.now(timezone.utc))
        if age < threshold and not process_dead and not timed_out:
            continue
        # Un processus worker disparu est une anomalie immédiate : ne pas
        # attendre le TTL d'âge, sinon une tâche peut rester running invisible.
        if process_dead:
            age = max(age, threshold)
            event_name = "task.worker_lost"
        elif timed_out:
            event_name = "task.timeout"
        alert = {
            "event": event_name,
            "task_id": tid,
            "status": status,
            "age_sec": age,
            "age_bucket": age_bucket(age, max(1, threshold)),
            "agent_target": task.get("agent_target", ""),
            "resolved_target": target,
            "agent_live": agent_live,
            "process_dead": process_dead,
            "pane_worker_active": pane_active,
            "timed_out": timed_out,
            "notice_sec": threshold,
            "evidence_marker": str(observed_at or "legacy"),
            "routing": "worker",
            "backend": task.get("backend", ""),
            "due_at": task.get("due_at", ""),
        }
        alerts.append(alert)
    return alerts


def scheduler_candidates(rd: Path, limit: int) -> tuple[list[dict[str, Any]], str]:
    run = load_json(rd / "run.json", {})
    policies = run.get("policies", {})
    max_concurrent = int(policies.get("max_concurrent_tasks", 2))
    running = running_tasks(rd)
    slots = max(0, max_concurrent - len(running))
    if slots <= 0:
        return [], f"max_concurrent_tasks reached ({max_concurrent})"
    selected: list[dict[str, Any]] = []
    active_ids = {task.get("task_id") for task in running}
    for task in ready_tasks(rd):
        if len(selected) >= min(limit, slots):
            break
        if running and not bool(task.get("parallelizable")):
            continue
        conflicts = set(task.get("conflicts_with", []))
        selected_ids = {item.get("task_id") for item in selected}
        if conflicts & active_ids:
            continue
        if conflicts & selected_ids:
            continue
        if task.get("task_id") in {conflict for item in selected for conflict in item.get("conflicts_with", [])}:
            continue
        selected.append(task)
    reason = "ready" if selected else "no ready task after dependency/conflict/concurrency checks"
    return selected, reason


def cmd_run_next(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    errors = collect_errors(rd)
    if errors:
        for err in errors:
            print(f"ERROR {err}", file=sys.stderr)
        return 1
    collected = collect_task_results(rd)
    if collected:
        for event in collected:
            print(f"COLLECTED {event.get('task_id')} status={event.get('status')}")
    candidates, reason = scheduler_candidates(rd, args.limit)
    if not candidates:
        print(f"NO_READY {reason}")
        return 0
    for task in candidates:
        print(f"SCHEDULE {task['task_id']} backend={task.get('backend')}")
        if not args.dry_run:
            dispatch_args = argparse.Namespace(
                root=args.root,
                run_id=args.run_id,
                task_id=task["task_id"],
                dry_run=False,
                force=False,
                spawn_command=args.spawn_command,
                runner_command=args.runner_command,
            )
            rc = cmd_dispatch(dispatch_args)
            if rc not in {0, 2}:
                return rc
    return 0


def heartbeat_state_path(rd: Path) -> Path:
    return rd / "heartbeat-state.json"


def notify_signature(events: list[dict[str, Any]]) -> str:
    payload = {
        "events": [
            (
                e.get("event"),
                e.get("task_id"),
                e.get("status"),
                e.get("backend"),
                e.get("agent_target"),
                e.get("evidence_marker"),
            )
            for e in events
        ],
    }
    return json.dumps(payload, sort_keys=True, ensure_ascii=False)


def issue_key(event: dict[str, Any]) -> str:
    return f"{event.get('event') or 'unknown'}:{event.get('task_id') or 'run'}"


def advance_issue_state(
    event: dict[str, Any],
    previous: dict[str, Any] | None,
    *,
    interval_elapsed: bool,
    escalation_after: int,
) -> dict[str, Any] | None:
    marker = str(event.get("evidence_marker") or event.get("status") or "legacy")
    same_observation = bool(previous and previous.get("evidence_marker") == marker)
    if same_observation and previous and previous.get("deferred"):
        return {**previous, "event": event, "last_notified_at": now()}
    if same_observation and not interval_elapsed:
        return None
    old_count = int(previous.get("reminder_count", 0)) if same_observation and previous else 0
    max_count = max(1, int(escalation_after)) + 1
    if old_count >= max_count:
        return None
    reminder_count = old_count + 1
    routing = str(event.get("routing") or "coordinator")
    if reminder_count == 1 and routing == "worker":
        audience = "worker"
    elif reminder_count <= max(1, int(escalation_after)):
        audience = "coordinator"
    else:
        audience = "escalation"
    return {
        "issue_key": issue_key(event),
        "event": event,
        "evidence_marker": marker,
        "reminder_count": reminder_count,
        "audience": audience,
        "last_notified_at": now(),
    }


def session_for_worker_event(event: dict[str, Any]) -> dict[str, Any] | None:
    if event.get("agent_live") is False:
        return None
    target = str(event.get("resolved_target") or event.get("agent_target") or "").strip()
    if not target:
        return None
    backend = str(event.get("backend") or "")
    if backend == "existing_bridget":
        return {"backend": backend, "agent_id": target, "status": "live"}
    return {"backend": backend or "existing_tmux", "pane": target, "status": "live"}


def session_recipient_key(session: dict[str, Any]) -> str:
    return f"{session.get('backend')}:{session.get('agent_id') or session.get('pane')}"


def mission_digest(rd: Path, audience: str, items: list[dict[str, Any]], limit: int) -> str:
    audiences = {str(item.get("audience") or audience) for item in items}
    lines = [
        f"[agent-loop {rd.name}]",
        f"run_dir: {rd}",
        f"audience: {','.join(sorted(audiences))}",
        f"actions: {len(items)}",
        f"suivi: {rd / 'heartbeat-state.json'}",
    ]
    for item in items[:max(1, limit)]:
        event = item["event"]
        line = (
            f"- task={event.get('task_id') or '-'} event={event.get('event')} "
            f"status={event.get('status') or '-'} age_sec={event.get('age_sec', 0)} "
            f"reminder={item.get('reminder_count')}"
        )
        if event.get("due_at"):
            line += f" due_at={event.get('due_at')}"
        lines.append(line)
    if len(items) > limit:
        lines.append(f"- autres_actions={len(items) - limit}")
    if "worker" in audiences:
        lines.append(f"Action: utilise ack ou progress dans {Path(__file__).resolve()} pour la tâche indiquée.")
    if "coordinator" in audiences:
        lines.append("Action: attribue, arbitre ou relance. Une intention future ne compte pas.")
    if "escalation" in audiences:
        lines.append("Action: arbitre la coordination ou réattribue la mission.")
    return "\n".join(lines)


def send_orchestrator_message(rd: Path, session: dict[str, Any], message: str, *, replay: dict[str, Any] | None = None) -> None:
    if session.get("backend") == "existing_bridget":
        agent_id = str(session.get("agent_id") or "")
        if not agent_id:
            raise RuntimeError("existing_bridget orchestrator has no agent_id")
        if replay:
            send_bridget_message(agent_id, message, replay=replay)
        else:
            send_bridget_message(agent_id, message)
        return
    pane = session.get("pane") or resolve_agent_target(session.get("agent_name") or session.get("pane_name") or "")
    cmd = [str(BRIDGE), "send", "--to", pane, "--no-reply", "--hops", "1", message]
    subprocess.run(cmd, check=True, env=bridge_background_environment("heartbeat"))


def heartbeat_message(rd: Path, events: list[dict[str, Any]], ready: list[dict[str, Any]]) -> str:
    lines = [
        "[agent-loop heartbeat]",
        f"run_dir: {rd}",
        f"events: {len(events)}",
        f"ready_tasks: {len(ready)}",
    ]
    for event in events[:8]:
        detail = f"- event {event.get('event')} task={event.get('task_id')} status={event.get('status')}"
        if "age_sec" in event:
            detail += f" age_sec={event.get('age_sec')} age_bucket={event.get('age_bucket')}"
        if event.get("agent_target"):
            detail += f" target={event.get('agent_target')}"
        if "agent_live" in event:
            detail += f" live={event.get('agent_live')}"
        if event.get("result_path"):
            detail += f" result={event.get('result_path')}"
        lines.append(detail)
    for task in ready[:8]:
        lines.append(f"- ready {task.get('task_id')} backend={task.get('backend')} delegate={task.get('delegate')}")
    lines.append("Instruction: lis les taches/resultats utiles et decide la prochaine action. N'ouvre pas de pane tmux sauf policy explicite.")
    return "\n".join(lines)


def decision_request_path_for_orchestrator_missing(rd: Path) -> Path:
    return rd / "decisions" / "orchestrator-missing.json"


def write_orchestrator_missing_decision_request(
    rd: Path,
    role: str,
    *,
    ready_count: int,
    collected_count: int,
    running_alert_count: int,
) -> Path:
    path = decision_request_path_for_orchestrator_missing(rd)
    existing = load_json(path, {})
    if existing and existing.get("schema_version") == "decision-request-v1" and existing.get("status") == "open":
        return path
    run = load_json(rd / "run.json", {})
    request = {
        "schema_version": "decision-request-v1",
        "id": "agent-loop-orchestrator-missing",
        "created_at": now(),
        "from_agent": "agent-loop-heartbeat",
        "run_dir": str(rd),
        "subject": "Orchestrateur agent-loop introuvable",
        "why_escalated": (
            "Le heartbeat a detecte des actions en attente, mais aucune session "
            f"vivante avec le role '{role}' n'est rattachee au run. "
            f"ready={ready_count}, collected={collected_count}, running_alerts={running_alert_count}."
        ),
        "level": "decision_requise",
        "category": "arbitrage",
        "options": [
            "Rattacher un orchestrateur vivant puis relancer le heartbeat.",
            "Fermer ou archiver le run si ces actions sont obsoletes.",
        ],
        "status": "open",
        "answer": "",
        "answered_at": None,
    }
    if run.get("run_id"):
        request["id"] = f"agent-loop-orchestrator-missing-{run['run_id']}"
    write_json(path, request)
    return path


def cmd_heartbeat_legacy(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    errors = collect_errors(rd)
    if errors:
        for err in errors:
            print(f"ERROR {err}", file=sys.stderr)
        return 1
    if not args.dry_run:
        cmd_refresh_sessions(argparse.Namespace(root=args.root, run_id=args.run_id))
    if not args.dry_run:
        collect_task_results(rd)
    ready = ready_tasks_for_idle_agents(rd, ready_tasks(rd))
    notice_sec = heartbeat_notice_sec(rd)
    ready_alerts = ready_task_alerts(rd, ready, notice_sec)
    collected_alerts = collected_task_alerts(rd, notice_sec)
    running_alerts = running_task_alerts(rd, notice_sec)
    relaunch_alerts = relaunch_required_alerts(rd, notice_sec)
    actionable_events = collected_alerts + running_alerts + ready_alerts + relaunch_alerts
    actionable = actionable_events
    if not actionable:
        if args.verbose:
            print("NO_OP")
        return 0

    # Même moteur de remise que v2, sans inventer ACK/disposition pour le legacy.
    return notify_mission_events(args, actionable_events)


def write_mission_escalation_request(rd: Path, items: list[dict[str, Any]], role: str) -> Path:
    keys = sorted(str(item.get("issue_key") or "") for item in items)
    proofs = sorted((str(item.get("issue_key") or ""), str(item.get("evidence_marker") or "")) for item in items)
    digest = hashlib.sha256(json.dumps(proofs).encode("utf-8")).hexdigest()[:16]
    path = rd / "decisions" / f"mission-escalation-{digest}.json"
    existing = load_json(path, {})
    if existing.get("status") == "open":
        return path
    request = {
        "schema_version": "decision-request-v1",
        "id": f"mission-escalation-{digest}",
        "created_at": now(),
        "from_agent": "agent-loop-mission-control",
        "run_dir": str(rd),
        "subject": "Escalade de missions sans progrès",
        "why_escalated": f"Le rôle {role!r} est injoignable après deux rappels sans progrès.",
        "level": "decision_requise",
        "category": "coordination",
        "issue_keys": keys,
        "status": "open",
        "answer": "",
        "answered_at": None,
    }
    write_json(path, request)
    return path


def open_run_alerts(rd: Path, ready: list[dict[str, Any]]) -> list[dict[str, Any]]:
    run = load_json(rd / "run.json", {})
    if run.get("status", "open") == "closed" or ready:
        return []
    active = running_tasks(rd)
    if len(active) >= int(run.get("policies", {}).get("max_concurrent_tasks", 2)):
        return []
    obligations = run_open_obligations(rd)
    # Des workers actifs avec aucune tâche en attente sont un état normal.
    waiting = [item for item in obligations if not item.endswith(":running") and not item.endswith(":dispatched")]
    if not waiting:
        return []
    marker = hashlib.sha256("\n".join(waiting).encode("utf-8")).hexdigest()
    return [{
        "event": "run.open_no_ready",
        "task_id": "",
        "status": "open",
        "age_sec": 0,
        "evidence_marker": marker,
        "routing": "coordinator",
        "obligation_count": len(waiting),
    }]


def cmd_heartbeat(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    if args.dry_run:
        return heartbeat_tick(args)
    if not rd.is_dir():
        raise SystemExit(f"missing run directory: {rd}")
    with (rd / ".heartbeat.lock").open("a") as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print("NO_OP heartbeat_already_running")
            return 0
        return heartbeat_tick(args)


def heartbeat_tick(args: argparse.Namespace) -> int:
    """Contrôle O(n log n + dépendances) : scans triés et trois vagues au plus."""
    rd = run_dir(args)
    run = load_json(rd / "run.json", {})
    if run.get("status") == "closed":
        print("NO_OP run_closed")
        return 0
    if not run.get("mission_control_enabled_at"):
        return cmd_heartbeat_legacy(args)
    errors = collect_errors(rd)
    if errors:
        for err in errors:
            print(f"ERROR {err}", file=sys.stderr)
        return 1
    if not args.dry_run:
        cmd_refresh_sessions(argparse.Namespace(root=args.root, run_id=args.run_id))
        collect_task_results(rd)
    ready = ready_tasks_for_idle_agents(rd, ready_tasks(rd))
    notice_sec = heartbeat_notice_sec(rd)
    # Les résultats sont relus depuis leur état durable. L'événement de collecte
    # ponctuel ne crée pas une deuxième anomalie au passage suivant.
    events = (
        collected_task_alerts(rd, notice_sec)
        + running_task_alerts(rd)
        + ready_task_alerts(rd, ready, notice_sec)
        + relaunch_required_alerts(rd, notice_sec)
        + continuation_task_alerts(rd)
        + open_run_alerts(rd, ready)
    )
    return notify_mission_events(args, events, run=run)


def notify_mission_events(
    args: argparse.Namespace, events: list[dict[str, Any]], *,
    run: dict[str, Any] | None = None, role_resolver=None, transport=None,
    clock: str | None = None, initial_issues: dict[str, Any] | None = None,
) -> int:
    """Moteur partagé O(n log n) : détection métier séparée de la remise durable.

    Les adaptateurs fournissent des anomalies et des preuves stables, jamais
    leur propre boucle d'envoi. Ils ne collectent ni ne valident ici une tâche.
    """
    rd = run_dir(args)
    run = load_json(rd / "run.json", {}) if run is None else run
    policies = run.get("policies", {})
    interval = max(1, int(policies.get("reminder_interval_sec", 300)))
    escalation_after = max(1, int(policies.get("escalation_after_reminders", 2)))
    limit = max(1, int(policies.get("heartbeat_digest_limit", 8)))
    orchestrator_role = args.orchestrator_role or policies.get("orchestrator_role", "orchestrator")
    escalation_role = str(policies.get("escalation_role", "root"))
    resolve_role = role_resolver or live_session_for_role
    deliver = transport or send_orchestrator_message
    tick_at = clock or now()
    state_path = heartbeat_state_path(rd)
    state = load_json(state_path, {})
    legacy_outbox = "active_keys" not in state and "occurrences" not in state
    previous = state.get("issues", initial_issues or {})
    prior_occurrences = dict(state.get("occurrences", {}))
    current_events = {issue_key(event): event for event in events}
    current_keys = set(current_events)
    active_keys = set(state.get("active_keys", previous))
    occurrences = dict(state.get("occurrences", {}))
    for key in current_keys:
        if key not in active_keys:
            occurrences[key] = int(occurrences.get(key, 0)) + 1
        else:
            occurrences.setdefault(key, 0)  # Reprise des anciens tokens en vol.
    issues = {key: value for key, value in previous.items() if key in current_keys}
    state = {
        "schema_version": "mission-control-heartbeat-v2",
        "issues": issues,
        "outbox": state.get("outbox", {}),
        "active_keys": sorted(current_keys),
        "occurrences": occurrences,
        "recovery_blocks": state.get("recovery_blocks", {}),
    }

    def persist() -> None:
        if not args.dry_run:
            state["updated_at"] = tick_at
            write_json(state_path, state)

    role_sessions: dict[str, dict[str, Any] | None] = {}
    groups: dict[str, dict[str, Any]] = {}
    unresolved: list[dict[str, Any]] = []
    sent_recipients: set[str] = set()
    failed_recipients: set[str] = set()

    def route(item: dict[str, Any]) -> None:
        while True:
            item = {**item, "last_notified_at": tick_at}
            audience = item["audience"]
            if audience == "worker":
                session = session_for_worker_event(item["event"])
            else:
                role = str(orchestrator_role) if audience == "coordinator" else escalation_role
                if role not in role_sessions:
                    role_sessions[role] = resolve_role(rd, role)
                session = role_sessions[role]
            if session:
                recipient = session_recipient_key(session)
                if recipient in state["recovery_blocks"]:
                    # Une ancienne remise ambiguë ne justifie jamais un nouvel
                    # identifiant vers le même destinataire. ROOT arbitre.
                    if audience == "escalation":
                        unresolved.append(item)
                        return
                    item = {**item, "audience": "escalation", "reminder_count": escalation_after + 1}
                    continue
                if recipient in failed_recipients:
                    # L'alerte peut commencer au coordinateur (résultat ou
                    # file prête). Ne pas retenter le même destinataire en
                    # passant d'un compteur à l'autre : atteindre ROOT ici.
                    fallback = advance_issue_state(
                        item["event"], item, interval_elapsed=True,
                        escalation_after=escalation_after,
                    )
                    if fallback is None:
                        unresolved.append(item)
                        return
                    item = fallback
                    continue
                group = groups.setdefault(recipient, {"session": session, "audience": audience, "items": []})
                group["items"].append(item)
                return
            if not args.dry_run:
                append_event(rd, {
                    "event": "heartbeat.recipient_unreachable",
                    "issue_key": item["issue_key"], "audience": audience,
                })
            fallback = advance_issue_state(
                item["event"], item, interval_elapsed=True, escalation_after=escalation_after
            )
            if fallback is None:
                unresolved.append(item)
                return
            item = fallback

    for event in events:
        key = issue_key(event)
        old = previous.get(key)
        elapsed = not old or (age_seconds(old.get("last_notified_at")) if clock is None else max(0, (parse_ts(tick_at) - parse_ts(old.get("last_notified_at") or tick_at)).total_seconds())) >= interval
        if old and old.get("recipient_unreachable") and old.get("evidence_marker") == str(event.get("evidence_marker") or event.get("status") or "legacy"):
            if escalation_role not in role_sessions:
                role_sessions[escalation_role] = resolve_role(rd, escalation_role)
            item = {**old, "event": event, "last_notified_at": tick_at} if role_sessions[escalation_role] else None
        else:
            item = advance_issue_state(event, old, interval_elapsed=elapsed, escalation_after=escalation_after)
        if item:
            item["last_notified_at"] = tick_at
        if item:
            route(item)

    def batch_token(recipient: str, items: list[dict[str, Any]], *, legacy=False, generations=None) -> str:
        generations = occurrences if generations is None else generations
        proofs = sorted(
            (item["issue_key"], item["evidence_marker"], item["reminder_count"])
            + (() if legacy or not generations.get(item["issue_key"], 0) else (generations[item["issue_key"]],))
            for item in items
        )
        return hashlib.sha256(json.dumps([str(rd.resolve()), recipient, proofs], sort_keys=True).encode("utf-8")).hexdigest()

    finalized_groups = None

    def legacy_batch_finalized(recipient: str, batch: dict[str, Any]) -> bool:
        nonlocal finalized_groups
        if finalized_groups is None:
            finalized_groups = {}
            for item in previous.values():
                if not item.get("event") or not item.get("last_notified_at") or item.get("deferred") or item.get("recipient_unreachable"):
                    continue
                audience = item.get("audience")
                if audience == "worker":
                    session = session_for_worker_event(item["event"])
                else:
                    role = str(orchestrator_role) if audience == "coordinator" else escalation_role
                    if role not in role_sessions:
                        role_sessions[role] = resolve_role(rd, role)
                    session = role_sessions[role]
                if session:
                    key = (session_recipient_key(session), int(parse_ts(item["last_notified_at"]).timestamp()))
                    finalized_groups.setdefault(key, []).append(item)
        saved = finalized_groups.get((recipient, batch.get("issued_at")), [])
        return bool(saved and batch["id"] in {
            batch_token(recipient, saved, legacy=True),
            batch_token(recipient, saved, generations=prior_occurrences),
        })

    # La vague de reprise précède les nouveautés. Un nouveau fait ne peut
    # remplacer ni le corps ni la clé d'une remise dont l'issue est inconnue.
    for recipient, batch in list(state["outbox"].items()):
        receipt = load_json(rd / "receipts" / f"notification-{batch['id']}.json", {})
        if receipt.get("state") == "accepted" and "items" not in batch:
            if recipient not in groups or legacy_batch_finalized(recipient, batch):
                continue  # Reçu et état final sain : historique, pas lot en vol.
        fresh = groups.pop(recipient, {})
        saved_items = batch.get("items")
        if saved_items is None:
            candidates = fresh.get("items", [])
            matches_legacy = bool(candidates and legacy_outbox and batch["id"] == batch_token(recipient, candidates, legacy=True))
            if matches_legacy or (candidates and batch["id"] == batch_token(recipient, candidates)):
                saved_items = candidates
                if matches_legacy:
                    for item in saved_items:
                        occurrences[item["issue_key"]] = 0
            else:
                # Pas d'inférence depuis le texte ni de recherche exponentielle
                # de sous-ensembles. L'association exige une décision explicite.
                saved_items = []
                decision_path = rd / "decisions" / f"outbox-recovery-{batch['id']}.json"
                state["recovery_blocks"][recipient] = {"delivery_id": batch["id"], "decision_request_path": str(decision_path)}
                if not args.dry_run and not decision_path.exists():
                    write_json(decision_path, {
                        "schema_version": "decision-request-v1", "id": f"outbox-recovery-{batch['id']}",
                        "created_at": tick_at, "from_agent": "agent-loop-mission-control",
                        "subject": "Reprise d'une ancienne remise ambiguë",
                        "why_escalated": "Le lot figé ne possède pas les preuves nécessaires pour l'associer aux anomalies courantes.",
                        "delivery_id": batch["id"], "recipient": recipient,
                        "status": "open", "answer": "", "answered_at": None,
                    })
        backend, _, target = recipient.partition(":")
        session = batch.get("session") or fresh.get("session") or {
            "backend": backend, "agent_id" if backend == "existing_bridget" else "pane": target,
        }
        groups[recipient] = {
            "session": session, "audience": batch.get("audience", fresh.get("audience", "coordinator")),
            "items": saved_items, "waiting_items": fresh.get("items", []), "replay_batch": batch,
        }
    if not groups and not unresolved:
        if issues != previous or current_keys != active_keys or (initial_issues and not state_path.exists()):
            persist()
        if args.verbose:
            print("NO_OP")
        return 0

    # Trois vagues au plus (worker, coordinateur, escalade). Le registre
    # complet est écrit une fois par vague, pas une fois par destinataire.
    while groups:
        wave = list(groups.items())
        groups.clear()
        for recipient, group in wave:
            if group.get("replay_batch") or recipient in sent_recipients or recipient in failed_recipients:
                continue
            token = batch_token(recipient, group["items"])
            batch = state["outbox"].get(recipient, {})
            if batch.get("id") != token:
                batch = {
                    "id": token, "issued_at": int(parse_ts(tick_at).timestamp()),
                    "issuer_scope": "agent-loop-" + hashlib.sha256(str(rd.resolve()).encode()).hexdigest(),
                    "message": mission_digest(rd, group["audience"], group["items"], limit),
                    "session": {key: group["session"][key] for key in ("backend", "agent_id", "pane") if key in group["session"]},
                    "audience": group["audience"],
                    "items": [{**item, "occurrence": occurrences[item["issue_key"]]} for item in group["items"]],
                }
                state["outbox"][recipient] = batch
        persist()
        for recipient, group in wave:
            if recipient in sent_recipients or recipient in failed_recipients:
                for item in group["items"]:
                    issues[item["issue_key"]] = {**item, "deferred": True}
                continue
            batch = state["outbox"][recipient]
            receipt_path = rd / "receipts" / f"notification-{batch['id']}.json"
            receipt = load_json(receipt_path, {})
            if args.dry_run:
                print(batch["message"])
                sent_recipients.add(recipient)
                continue
            try:
                if receipt.get("state") != "accepted":
                    deliver(rd, group["session"], batch["message"], replay=batch)
                    write_json(receipt_path, {
                        "schema_version": "mission-notification-v2", "state": "accepted",
                        "delivery_id": batch["id"], "recipient": recipient, "accepted_at": tick_at,
                    })
            except (OSError, RuntimeError, subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
                failed_recipients.add(recipient)
                for item in group.get("waiting_items", group["items"]):
                    append_event(rd, {
                        "event": "heartbeat.transport_failed",
                        "issue_key": item["issue_key"], "audience": item["audience"],
                        "failure_kind": "subprocess" if isinstance(exc, subprocess.CalledProcessError) else "timeout" if isinstance(exc, subprocess.TimeoutExpired) else "io" if isinstance(exc, OSError) else "runtime",
                        "returncode": exc.returncode if isinstance(exc, subprocess.CalledProcessError) else None,
                    })
                    fallback = advance_issue_state(
                        item["event"], item, interval_elapsed=True, escalation_after=escalation_after
                    )
                    if fallback:
                        route(fallback)
                    else:
                        unresolved.append(item)
                continue
            for item in group["items"]:
                key = item["issue_key"]
                current = current_events.get(key)
                if not current or str(current.get("evidence_marker") or current.get("status") or "legacy") != item["evidence_marker"] or int(item.get("occurrence", occurrences[key])) != occurrences[key]:
                    continue  # Le reçu ancien ne valide pas une preuve nouvelle.
                newer = issues.get(key, {})
                if newer.get("evidence_marker") == item["evidence_marker"] and int(newer.get("reminder_count", 0)) > item["reminder_count"]:
                    continue  # Une reprise ancienne ne redescend pas l'escalade.
                issues[item["issue_key"]] = {
                    "issue_key": item["issue_key"], "event": item["event"],
                    "evidence_marker": item["evidence_marker"], "reminder_count": item["reminder_count"],
                    "audience": item["audience"], "last_notified_at": tick_at,
                }
            sent_recipients.add(recipient)
            state["outbox"].pop(recipient, None)
            replayed = {(item["issue_key"], item["evidence_marker"], int(item.get("occurrence", occurrences.get(item["issue_key"], 0)))) for item in group["items"]}
            for item in group.get("waiting_items", []):
                if (item["issue_key"], item["evidence_marker"], occurrences[item["issue_key"]]) in replayed:
                    continue
                if recipient in state["recovery_blocks"]:
                    route(item)
                else:
                    issues[item["issue_key"]] = {**item, "deferred": True}
            append_event(rd, {
                "event": "heartbeat.digest_sent", "audience": group["audience"],
                "recipient": recipient, "action_count": len(group["items"]), "delivery_id": batch["id"],
            })
    if unresolved and not args.dry_run:
        decision_path = write_mission_escalation_request(rd, unresolved, escalation_role)
        for item in unresolved:
            issues[item["issue_key"]] = {
                "issue_key": item["issue_key"], "event": item["event"],
                "evidence_marker": item["evidence_marker"], "reminder_count": escalation_after + 1,
                "audience": "escalation", "last_notified_at": tick_at,
                "decision_request_path": str(decision_path), "recipient_unreachable": True,
            }
        append_event(rd, {
            "event": "heartbeat.escalation_decision_requested",
            "role": escalation_role, "decision_request_path": str(decision_path),
            "action_count": len(unresolved),
        })
    persist()
    print(f"NOTIFIED_DIGESTS {len(sent_recipients)}" if sent_recipients else "ESCALATION_RECORDED")
    return 0


def collect_errors(rd: Path) -> list[str]:
    errors: list[str] = []
    run = load_json(rd / "run.json")
    if not run:
        return [f"missing run.json: {rd}"]
    policies = run.get("policies", {})
    allowed = set(policies.get("allowed_backends", ["local", "existing_tmux", "existing_bridget"]))
    allow_spawn = bool(policies.get("allow_spawn_tmux", False))
    task_ids = {p.stem for p in (rd / "tasks").glob("*.json")}
    for path in sorted((rd / "tasks").glob("*.json")):
        task = load_json(path, {})
        tid = task.get("task_id", path.stem)
        backend = task.get("backend")
        delegate = bool(task.get("delegate"))
        if backend not in {"local", "existing_tmux", "existing_bridget", "spawn_tmux", "background_process", "codex_thread", "llm_process", "external_runner"}:
            errors.append(f"{tid}: unknown backend {backend!r}")
        if backend not in allowed and not (backend == "spawn_tmux" and allow_spawn):
            errors.append(f"{tid}: backend {backend!r} blocked by policy")
        if not delegate and backend != "local":
            errors.append(f"{tid}: delegate=false requires backend=local")
        if delegate and backend == "local":
            errors.append(f"{tid}: delegate=true cannot use backend=local")
        if backend == "spawn_tmux" and not allow_spawn:
            errors.append(f"{tid}: spawn_tmux requested but allow_spawn_tmux=false")
        if backend == "spawn_tmux" and not task.get("spawn_reason"):
            errors.append(f"{tid}: spawn_tmux requires spawn_reason")
        if backend in {"local", "background_process"} and not task.get("runner_command"):
            errors.append(f"{tid}: backend {backend!r} requires runner_command")
        if task.get("reply_channel", "file") not in {"file", "bridge", "both"}:
            errors.append(f"{tid}: invalid reply_channel {task.get('reply_channel')!r}")
        if int(task.get("attempts", 0)) >= int(task.get("max_attempts", 2)) and task.get("status") == "pending":
            errors.append(f"{tid}: max_attempts reached")
        if not task.get("objective"):
            errors.append(f"{tid}: missing objective")
        if not task.get("acceptance"):
            errors.append(f"{tid}: missing acceptance")
        for dep in task.get("depends_on", []):
            if dep not in task_ids:
                errors.append(f"{tid}: missing dependency {dep}")
    return errors


def cmd_validate(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    errors = collect_errors(rd)
    if errors:
        for err in errors:
            print(f"ERROR {err}", file=sys.stderr)
        return 1
    print(f"PASS {rd}")
    return 0


def read_events(rd: Path, limit: int = 60) -> list[dict[str, Any]]:
    path = rd / "events.jsonl"
    if not path.exists():
        return []
    lines = path.read_text(encoding="utf-8").splitlines()[-limit:]
    events = []
    for line in lines:
        try:
            events.append(json.loads(line))
        except json.JSONDecodeError:
            events.append({"event": "parse_error", "raw": line})
    return events


def status_badge(status: str) -> str:
    classes = {
        "pass": "ok",
        "dispatched": "warn",
        "running": "warn",
        "pending": "info",
        "blocked": "stop",
        "fail": "stop",
        "review": "warn",
        "stale": "stop",
        "live": "ok",
    }
    cls = classes.get(status, "neutral")
    return f'<span class="badge {cls}">{html.escape(status or "-")}</span>'


def render_dashboard(rd: Path, refresh_sec: int = 0) -> str:
    run = load_json(rd / "run.json", {})
    tasks = [load_json(path, {}) for path in sorted((rd / "tasks").glob("*.json"))]
    sessions = [load_json(path, {}) for path in sorted((rd / "sessions").glob("*.json"))]
    events = read_events(rd)
    results = [load_json(path, {}) for path in sorted((rd / "results").glob("*.result.json"))]
    ready, ready_reason = scheduler_candidates(rd, 20)
    meta_refresh = f'<meta http-equiv="refresh" content="{refresh_sec}">' if refresh_sec else ""
    css = """
      body{font:14px/1.45 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;margin:0;background:#f6f7f9;color:#17202c}
      header{background:#101828;color:#fff;padding:22px 28px} main{max-width:1180px;margin:0 auto;padding:22px}
      section{background:#fff;border:1px solid #d8dee8;border-radius:8px;padding:18px;margin-bottom:16px}
      h1{margin:0 0 6px;font-size:25px} h2{margin:0 0 12px;font-size:18px}
      table{width:100%;border-collapse:collapse;border:1px solid #d8dee8;border-radius:8px;overflow:hidden}
      th,td{padding:9px 10px;border-bottom:1px solid #d8dee8;text-align:left;vertical-align:top}
      th{background:#f1f4f8}.badge{display:inline-block;border-radius:999px;padding:2px 8px;font-weight:650;font-size:12px}
      .ok{background:#ecfdf3;color:#147a4a}.warn{background:#fff6e6;color:#9a5b00}.stop{background:#fff1f0;color:#b42318}
      .info{background:#eef4ff;color:#2457d6}.neutral{background:#f2f4f7;color:#344054}
      code{font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;background:#eef1f6;border:1px solid #d9dee7;border-radius:5px;padding:1px 5px}
      .path{word-break:break-all;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;font-size:12px}
      .grid{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px}.card{border:1px solid #d8dee8;border-radius:8px;padding:12px;background:#fbfcfe}
      @media(max-width:800px){.grid{grid-template-columns:1fr}main{padding:14px}}
    """
    counts: dict[str, int] = {}
    for task in tasks:
        counts[task.get("status", "unknown")] = counts.get(task.get("status", "unknown"), 0) + 1
    task_rows = "\n".join(
        "<tr>"
        f"<td><code>{html.escape(t.get('task_id',''))}</code></td>"
        f"<td>{status_badge(t.get('status',''))}</td>"
        f"<td>{html.escape(t.get('backend',''))}</td>"
        f"<td>{html.escape(t.get('reply_channel','file'))}</td>"
        f"<td>{html.escape(t.get('objective',''))}</td>"
        f"<td>{html.escape(', '.join(t.get('depends_on', [])))}</td>"
        "</tr>"
        for t in tasks
    )
    session_rows = "\n".join(
        "<tr>"
        f"<td>{html.escape(s.get('role',''))}</td>"
        f"<td>{status_badge(s.get('status',''))}</td>"
        f"<td>{html.escape(s.get('backend',''))}</td>"
        f"<td><code>{html.escape(s.get('pane',''))}</code></td>"
        f"<td>{html.escape(s.get('agent_name',''))}</td>"
        "</tr>"
        for s in sessions
    )
    event_rows = "\n".join(
        "<tr>"
        f"<td>{html.escape(e.get('ts',''))}</td>"
        f"<td>{html.escape(e.get('event',''))}</td>"
        f"<td><code>{html.escape(e.get('task_id',''))}</code></td>"
        f"<td>{html.escape(e.get('status','') or e.get('reason',''))}</td>"
        "</tr>"
        for e in reversed(events)
    )
    result_rows = "\n".join(
        "<tr>"
        f"<td><code>{html.escape(r.get('task_id',''))}</code></td>"
        f"<td>{status_badge(r.get('status',''))}</td>"
        f"<td>{html.escape(r.get('message',''))}</td>"
        f"<td>{html.escape(r.get('next_recommendation',''))}</td>"
        "</tr>"
        for r in results
    )
    ready_items = "".join(f"<li><code>{html.escape(t.get('task_id',''))}</code> {html.escape(t.get('backend',''))}</li>" for t in ready)
    count_cards = "".join(f'<div class="card"><strong>{html.escape(k)}</strong><br>{v}</div>' for k, v in sorted(counts.items()))
    return f"""<!doctype html>
<html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">{meta_refresh}
<title>Agent Loop Dashboard - {html.escape(run.get('run_id',''))}</title><style>{css}</style></head>
<body><header><h1>Agent Loop Dashboard</h1><div>{html.escape(run.get('run_id',''))} · {html.escape(run.get('objective',''))}</div></header>
<main>
<section><h2>Résumé</h2><div class="grid">{count_cards or '<div class="card">Aucune tâche</div>'}<div class="card"><strong>ready</strong><br>{len(ready)}</div><div class="card"><strong>sessions</strong><br>{len(sessions)}</div><div class="card"><strong>results</strong><br>{len(results)}</div></div><p>Scheduler: {html.escape(ready_reason)}</p><ul>{ready_items}</ul></section>
<section><h2>Tâches</h2><table><thead><tr><th>ID</th><th>Statut</th><th>Backend</th><th>Réponse</th><th>Objectif</th><th>Dépendances</th></tr></thead><tbody>{task_rows}</tbody></table></section>
<section><h2>Sessions</h2><table><thead><tr><th>Rôle</th><th>Statut</th><th>Backend</th><th>Pane</th><th>Agent</th></tr></thead><tbody>{session_rows}</tbody></table></section>
<section><h2>Résultats</h2><table><thead><tr><th>Tâche</th><th>Statut</th><th>Message</th><th>Recommandation</th></tr></thead><tbody>{result_rows}</tbody></table></section>
<section><h2>Derniers événements</h2><table><thead><tr><th>Temps</th><th>Événement</th><th>Tâche</th><th>Statut / raison</th></tr></thead><tbody>{event_rows}</tbody></table></section>
<section><h2>Chemin</h2><p class="path">{html.escape(str(rd))}</p></section>
</main></body></html>
"""


def cmd_dashboard(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    if not (rd / "run.json").exists():
        raise SystemExit(f"missing run.json: {rd}")
    output = Path(args.output).expanduser() if args.output else rd / "dashboard.html"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(render_dashboard(rd, args.refresh_sec), encoding="utf-8")
    print(str(output))
    return 0


def cmd_serve_dashboard(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    if not (rd / "run.json").exists():
        raise SystemExit(f"missing run.json: {rd}")

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802
            body = render_dashboard(rd, args.refresh_sec).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, format: str, *values: Any) -> None:
            append_event(rd, {"event": "dashboard.request", "client": self.client_address[0], "path": self.path})

    server = ThreadingHTTPServer((args.host, args.port), Handler)
    url = f"http://{args.host}:{args.port}/"
    append_event(rd, {"event": "dashboard.server_started", "url": url})
    print(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()
        append_event(rd, {"event": "dashboard.server_stopped", "url": url})
    return 0


def cmd_index_sessions(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    sessions = find_session_jsonls(args.limit)
    out = rd / "sessions" / "jsonl-index.json"
    write_json(out, {"schema_version": "agent-loop-jsonl-index-v1", "created_at": now(), "sessions": sessions})
    append_event(rd, {"event": "sessions.jsonl_indexed", "count": len(sessions), "path": str(out)})
    print(str(out))
    return 0


def cmd_create_toolchain_request(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    request_id = args.request_id or f"toolchain-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    path = rd / "toolchain-requests" / f"{request_id}.json"
    data = {
        "schema_version": "agent-loop-toolchain-request-v1",
        "request_id": request_id,
        "created_at": now(),
        "status": "open",
        "severity": args.severity,
        "title": args.title,
        "description": args.description,
        "evidence_paths": csv(args.evidence_paths),
        "suggested_owner": args.suggested_owner,
        "resolution": "",
        "resolved_at": "",
    }
    write_json(path, data)
    append_event(rd, {"event": "toolchain_request.created", "request_id": request_id, "severity": args.severity})
    print(str(path))
    return 0


def cmd_resolve_toolchain_request(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    path = rd / "toolchain-requests" / f"{args.request_id}.json"
    data = load_json(path)
    if not data:
        raise SystemExit(f"missing toolchain request: {path}")
    data["status"] = args.status
    data["resolution"] = args.resolution
    data["resolved_at"] = now()
    write_json(path, data)
    append_event(rd, {"event": "toolchain_request.resolved", "request_id": args.request_id, "status": args.status})
    print(str(path))
    return 0


def cmd_example(args: argparse.Namespace) -> int:
    rd = run_dir(args)
    if rd.exists() and not args.force:
        raise SystemExit(f"example run already exists: {rd}")
    init_args = argparse.Namespace(
        root=args.root,
        run_id=args.run_id,
        force=True,
        objective="Tester une boucle simple de revue keyframes sans spawn tmux.",
        prompt_initial="Demo agent-loop: audit dimensions, identite, revue visuelle optionnelle.",
        acceptance="Les trois taches ont un verdict ou un blocage explicite.",
        domain="horizon_episode",
        domain_context_json='{"example": true, "scene_id": "SC-001"}',
        policies_json='{"allow_spawn_tmux": false, "allowed_backends": ["local", "existing_tmux", "background_process"], "max_concurrent_tasks": 2}',
    )
    cmd_init(init_args)
    examples = [
        ("dimensions", False, "local", "Verifier les dimensions keyframes.", "verdict JSON dimensions pass/fail", ""),
        ("identity", False, "local", "Calculer le gate identite acteur.", "verdict JSON identite pass/review/fail", "dimensions"),
        ("visual-review", True, "existing_tmux", "Demander une revue visuelle courte a un agent existant.", "rapport court avec ok/ko et blockers", "identity"),
    ]
    for tid, delegate, backend, objective, acceptance, dep in examples:
        add_args = argparse.Namespace(
            root=args.root,
            run_id=args.run_id,
            task_id=tid,
            objective=objective,
            acceptance=acceptance,
            delegate=str(delegate).lower(),
            backend=backend,
            agent_target="conversation1_w1" if backend == "existing_tmux" else "",
            depends_on=dep,
            parallelizable="false",
            priority=100,
            conflicts_with="",
            parent_task_id="",
            spawn_reason="",
            max_attempts=1,
            reply_channel="file",
            runner_command=(
                f"{shlex.quote(sys.executable)} {shlex.quote(str(Path(__file__)))} "
                f"worker-result --root {shlex.quote(args.root)} --run-id {shlex.quote(args.run_id)} "
                f"--task-id {shlex.quote(tid)} --status pass --message {shlex.quote(objective)}"
            ) if backend == "local" else "",
            timeout_sec=0,
            keep_alive="false",
            agent_provider="",
            agent_model="",
            agent_command_template="",
            input_paths="",
            output_paths="",
            forbidden_paths="/Users/moi/.codex/skills" if backend != "local" else "",
        )
        cmd_add_task(add_args)
    cmd_validate(argparse.Namespace(root=args.root, run_id=args.run_id))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.set_defaults(func=None)
    sub = parser.add_subparsers(dest="cmd", required=True)

    def add_common(p: argparse.ArgumentParser) -> None:
        p.add_argument("--root", default=str(DEFAULT_ROOT))
        p.add_argument("--run-id", required=True)

    p = sub.add_parser("init")
    add_common(p)
    p.add_argument("--objective", required=True)
    p.add_argument("--prompt-initial", default="")
    p.add_argument("--acceptance", default="")
    p.add_argument("--domain", default="generic")
    p.add_argument("--domain-context-json", default="")
    p.add_argument("--policies-json", default="")
    p.add_argument("--force", action="store_true")
    p.set_defaults(func=cmd_init)

    p = sub.add_parser("add-task")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--objective", required=True)
    p.add_argument("--acceptance", required=True)
    p.add_argument("--delegate", required=True)
    p.add_argument("--backend", required=True)
    p.add_argument("--agent-target", default="")
    p.add_argument("--agent-role", default="worker")
    p.add_argument("--context", default="fresh", choices=["fresh", "continuation"])
    p.add_argument("--resume-from", default="")
    p.add_argument("--depends-on", default="")
    p.add_argument("--parallelizable", default="false")
    p.add_argument("--priority", type=int, default=100)
    p.add_argument("--conflicts-with", default="")
    p.add_argument("--parent-task-id", default="")
    p.add_argument("--spawn-reason", default="")
    p.add_argument("--max-attempts", type=int, default=2)
    p.add_argument("--reply-channel", default="file", choices=["file", "bridge", "both"])
    p.add_argument("--runner-command", default="")
    p.add_argument("--timeout-sec", type=int, default=0)
    p.add_argument("--keep-alive", default="false")
    p.add_argument("--agent-provider", default="")
    p.add_argument("--agent-model", default="")
    p.add_argument("--agent-command-template", default="")
    p.add_argument("--cwd", default="")
    p.add_argument("--input-paths", default="")
    p.add_argument("--output-paths", default="")
    p.add_argument("--forbidden-paths", default="")
    p.add_argument("--owner", default="")
    p.add_argument("--due-at", default="")
    p.add_argument("--expected-result", default="")
    p.add_argument("--work-kind", default="generic", choices=sorted(WORK_KINDS))
    p.set_defaults(func=cmd_add_task)

    p = sub.add_parser("record-event")
    add_common(p)
    p.add_argument("--event", required=True)
    p.add_argument("--task-id", default="")
    p.add_argument("--status", default="")
    p.add_argument("--message", default="")
    p.add_argument("--details-json", default="")
    p.set_defaults(func=cmd_record_event)

    p = sub.add_parser("record-session")
    add_common(p)
    p.add_argument("--role", required=True)
    p.add_argument("--backend", required=True)
    p.add_argument("--session-id", default="")
    p.add_argument("--jsonl-path", default="")
    p.add_argument("--pane", default="")
    p.add_argument("--agent-name", default="")
    p.add_argument("--notes", default="")
    p.set_defaults(func=cmd_record_session)

    p = sub.add_parser("attach-agent")
    add_common(p)
    p.add_argument("--role", required=True)
    p.add_argument("--target", required=True)
    p.add_argument("--backend", default="existing_tmux", choices=["existing_tmux", "existing_bridget"])
    p.add_argument("--session-id", default="")
    p.add_argument("--jsonl-path", default="")
    p.add_argument("--notes", default="")
    p.set_defaults(func=cmd_attach_agent)

    p = sub.add_parser("refresh-sessions")
    add_common(p)
    p.set_defaults(func=cmd_refresh_sessions)

    p = sub.add_parser("dispatch")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--force", action="store_true")
    p.add_argument("--spawn-command", default="")
    p.add_argument("--runner-command", default="")
    p.set_defaults(func=cmd_dispatch)

    p = sub.add_parser("run-next")
    add_common(p)
    p.add_argument("--limit", type=int, default=1)
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--spawn-command", default="")
    p.add_argument("--runner-command", default="")
    p.set_defaults(func=cmd_run_next)

    p = sub.add_parser("ack")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--worker", default="")
    p.set_defaults(func=cmd_ack)

    p = sub.add_parser("progress")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--worker", default="")
    p.add_argument("--kind", required=True, choices=sorted(MISSION_PROGRESS_KINDS))
    p.add_argument("--evidence-ref", required=True)
    p.set_defaults(func=cmd_progress)

    p = sub.add_parser("worker-result")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--status", default="pass", choices=["pass", "fail", "blocked", "review"])
    p.add_argument("--message", default="")
    p.add_argument("--worker", default="")
    p.add_argument("--blockers-json", default="")
    p.add_argument("--metrics-json", default="")
    p.add_argument("--output-paths", default="")
    p.add_argument("--next-recommendation", default="")
    p.add_argument("--logs-json", default="")
    p.set_defaults(func=cmd_worker_result)

    p = sub.add_parser("heartbeat")
    add_common(p)
    p.add_argument("--orchestrator-role", default="")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--force", action="store_true")
    p.add_argument("--verbose", action="store_true")
    p.set_defaults(func=cmd_heartbeat)

    p = sub.add_parser("migrate-run")
    add_common(p)
    p.set_defaults(func=cmd_migrate_run)

    p = sub.add_parser("reconcile")
    add_common(p)
    p.set_defaults(func=cmd_reconcile)

    p = sub.add_parser("resolve-task")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--status", required=True, choices=["pass", "fail", "blocked", "review", "superseded"])
    p.add_argument("--reason", required=True)
    p.set_defaults(func=cmd_resolve_task)

    p = sub.add_parser("disposition")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--decision", required=True, choices=sorted(MISSION_DISPOSITIONS))
    p.add_argument("--reason-code", required=True, choices=sorted(MISSION_REASON_CODES))
    p.add_argument("--owner", default="")
    p.add_argument("--next-check", default="")
    p.add_argument("--successor-task-id", default="")
    p.set_defaults(func=cmd_disposition)

    p = sub.add_parser("close-run")
    add_common(p)
    p.add_argument("--reason", required=True)
    p.set_defaults(func=cmd_close_run)

    p = sub.add_parser("relaunch")
    add_common(p)
    p.add_argument("--task-id", required=True)
    p.add_argument("--reason", default="terminal delivery without successor")
    p.add_argument("--agent-target", default="")
    p.add_argument("--assigned-agent", default="")
    p.set_defaults(func=cmd_relaunch)

    p = sub.add_parser("validate")
    add_common(p)
    p.set_defaults(func=cmd_validate)

    p = sub.add_parser("dashboard")
    add_common(p)
    p.add_argument("--output", default="")
    p.add_argument("--refresh-sec", type=int, default=0)
    p.set_defaults(func=cmd_dashboard)

    p = sub.add_parser("serve-dashboard")
    add_common(p)
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=8765)
    p.add_argument("--refresh-sec", type=int, default=5)
    p.set_defaults(func=cmd_serve_dashboard)

    p = sub.add_parser("index-sessions")
    add_common(p)
    p.add_argument("--limit", type=int, default=80)
    p.set_defaults(func=cmd_index_sessions)

    p = sub.add_parser("create-toolchain-request")
    add_common(p)
    p.add_argument("--request-id", default="")
    p.add_argument("--severity", default="medium", choices=["low", "medium", "high", "critical"])
    p.add_argument("--title", required=True)
    p.add_argument("--description", required=True)
    p.add_argument("--evidence-paths", default="")
    p.add_argument("--suggested-owner", default="")
    p.set_defaults(func=cmd_create_toolchain_request)

    p = sub.add_parser("resolve-toolchain-request")
    add_common(p)
    p.add_argument("--request-id", required=True)
    p.add_argument("--status", default="resolved", choices=["resolved", "rejected", "duplicate"])
    p.add_argument("--resolution", required=True)
    p.set_defaults(func=cmd_resolve_toolchain_request)

    p = sub.add_parser("example")
    add_common(p)
    p.add_argument("--force", action="store_true")
    p.set_defaults(func=cmd_example)
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
