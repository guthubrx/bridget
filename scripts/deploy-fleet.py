#!/usr/bin/env python3
"""Déploiement canari puis flotte complète pour Bridget, piloté par agents.json.

Découvre dynamiquement toutes les machines actives déclarées dans agents.json
(y compris les postes portables comme macbook-m1), active d'abord le canari
(maître + un worker), vérifie l'état réel (processus chargés, droits 0600,
bridget status), puis propage aux autres hôtes joignables.

Règles strictes préservées :
- Ne touche jamais à T3 (service `t3` / `com.bridget.t3` toujours exclu).
- Préserve le registre worker Mac et ne supprime aucun binaire encore ouvert.
- Met à jour `update-host-state.json` atomiquement sous `update-host.lock` en 0600
  après arrêt de l'updater, et ne qualifie `rollback` que si l'image précédente
  couvre tous les formats requis par la nouvelle image (`bridget formats`).
- Signale clairement tout hôte injoignable (ex. MacBook en veille) sans bloquer
  le reste de la flotte.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import shlex
import socket
import subprocess
import sys
import time
from typing import Any


EXCLUDED_SERVICES = frozenset({"t3", "com.bridget.t3"})
INSTALL_ROOT_REL = ".local/lib/bridget-communication"
ACTIVE_LINK_REL = ".local/bin/bridget"
CACHE_DIR_REL = ".cache/bridget-core"


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def discover_enabled_machines(agents_data: dict[str, Any]) -> list[str]:
    """Retourne les machine_id actifs déclarés dans agents.json, triés."""
    machines = agents_data.get("machines") or {}
    enabled = [
        machine_id
        for machine_id, meta in machines.items()
        if isinstance(meta, dict) and meta.get("enabled", True) is True
    ]
    return sorted(enabled)


def filter_safe_services(services: list[str]) -> list[str]:
    """Exclut T3 de tout redémarrage automatique (règle permanente AGENTS.md)."""
    return [svc for svc in services if svc not in EXCLUDED_SERVICES]


def rollback_compatible(old_reads: set[str], new_required_reads: set[str]) -> bool:
    """Vrai uniquement si l'ancien binaire sait lire tous les formats requis par le nouveau."""
    if not old_reads or not new_required_reads:
        return False
    return new_required_reads.issubset(old_reads)


def plan_rollout_order(
    machines: list[str],
    master: str,
    canary_worker: str | None = None,
) -> tuple[list[str], list[str]]:
    """Sépare les hôtes en (lot_canari, reste_flotte) dans un ordre déterministe."""
    ordered = list(dict.fromkeys(machines))
    canary: list[str] = []
    if master in ordered:
        canary.append(master)
    if canary_worker and canary_worker in ordered and canary_worker not in canary:
        canary.append(canary_worker)
    elif len(ordered) > len(canary):
        for candidate in ordered:
            if candidate not in canary:
                canary.append(candidate)
                break
    rest = [m for m in ordered if m not in canary]
    return canary, rest


def detect_local_machine_id() -> str | None:
    update_host = Path.home() / CACHE_DIR_REL / "update-host.json"
    if update_host.is_file():
        try:
            data = json.loads(update_host.read_text(encoding="utf-8"))
            host = data.get("host")
            if isinstance(host, str) and host.strip():
                return host.strip()
        except Exception:
            pass
    return socket.gethostname().split(".")[0] or None


def map_systemd_services(machine_id: str, raw_services: list[str]) -> list[str]:
    mapped: list[str] = []
    for svc in filter_safe_services(raw_services):
        if svc == "core":
            mapped.append("bridget-core.service")
        elif svc == "mobile":
            mapped.append("bridget-mobile.service")
        elif svc == "worker":
            mapped.append(f"bridget-worker-{machine_id}.service")
        elif svc.endswith(".service"):
            mapped.append(svc)
        else:
            mapped.append(f"{svc}.service")
    return list(dict.fromkeys(mapped))


class HostRunner:
    """Exécute des commandes et transferts sur un hôte local ou SSH."""

    def __init__(self, machine_id: str, ssh_alias: str, is_local: bool, timeout: int = 30):
        self.machine_id = machine_id
        self.ssh_alias = ssh_alias
        self.is_local = is_local
        self.timeout = timeout

    def run(self, script: str, timeout: int | None = None) -> subprocess.CompletedProcess[str]:
        effective = timeout or self.timeout
        if self.is_local:
            return subprocess.run(
                ["bash", "-lc", script],
                capture_output=True,
                text=True,
                timeout=effective,
            )
        return subprocess.run(
            [
                "ssh",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                self.ssh_alias,
                "bash",
                "-s",
            ],
            input=script,
            capture_output=True,
            text=True,
            timeout=effective,
        )

    def copy_binary(self, local_src: Path, remote_dest: str) -> None:
        if self.is_local:
            dest_path = Path(os.path.expanduser(remote_dest))
            dest_path.parent.mkdir(parents=True, exist_ok=True)
            tmp = dest_path.with_name(dest_path.name + ".tmp")
            tmp.write_bytes(local_src.read_bytes())
            os.chmod(tmp, 0o755)
            os.replace(tmp, dest_path)
            return
        parent = os.path.dirname(remote_dest)
        mkdir = self.run(f"mkdir -p {shlex.quote(parent)}")
        if mkdir.returncode != 0:
            raise RuntimeError(f"mkdir échoué sur {self.machine_id}: {mkdir.stderr.strip()}")
        scp = subprocess.run(
            [
                "scp",
                "-q",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
                str(local_src),
                f"{self.ssh_alias}:{remote_dest}.tmp",
            ],
            capture_output=True,
            text=True,
            timeout=120,
        )
        if scp.returncode != 0:
            raise RuntimeError(f"scp échoué vers {self.machine_id}: {scp.stderr.strip()}")
        finalize = self.run(
            f"chmod 755 {shlex.quote(remote_dest + '.tmp')} && mv -f {shlex.quote(remote_dest + '.tmp')} {shlex.quote(remote_dest)}"
        )
        if finalize.returncode != 0:
            raise RuntimeError(
                f"finalisation binaire échouée sur {self.machine_id}: {finalize.stderr.strip()}"
            )


def probe_host(runner: HostRunner) -> dict[str, Any]:
    probe_script = r"""
set -e
OS=$(uname -s)
ARCH=$(uname -m)
HOME_DIR="$HOME"
ACTIVE=$(readlink "$HOME/.local/bin/bridget" 2>/dev/null || true)
if [ -n "$ACTIVE" ] && [ "${ACTIVE#/}" = "$ACTIVE" ]; then
  ACTIVE=$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$HOME/.local/bin/bridget" 2>/dev/null || echo "$ACTIVE")
fi
HOST_CFG=""
if [ -f "$HOME/.cache/bridget-core/update-host.json" ]; then
  HOST_CFG=$(cat "$HOME/.cache/bridget-core/update-host.json")
fi
python3 - "$OS" "$ARCH" "$HOME_DIR" "$ACTIVE" "$HOST_CFG" <<'PY'
import json, sys
os_name, arch, home_dir, active, cfg_raw = sys.argv[1:6]
cfg = {}
if cfg_raw.strip():
    try:
        cfg = json.loads(cfg_raw)
    except Exception:
        cfg = {}
print(json.dumps({
    "os": os_name,
    "arch": arch,
    "home": home_dir,
    "active_binary": active or None,
    "update_host": cfg,
}))
PY
"""
    try:
        res = runner.run(probe_script, timeout=15)
    except subprocess.TimeoutExpired:
        return {"reachable": False, "error": "ssh_timeout"}
    if res.returncode != 0:
        err = (res.stderr or res.stdout or "unreachable").strip().splitlines()
        return {"reachable": False, "error": err[-1] if err else "unreachable"}
    try:
        info = json.loads(res.stdout.strip().splitlines()[-1])
    except Exception as exc:
        return {"reachable": False, "error": f"probe_parse_error:{exc}"}
    info["reachable"] = True
    return info


def check_remote_rollback_compatible(
    runner: HostRunner, old_binary: str | None, new_binary: str
) -> bool:
    if not old_binary or old_binary == new_binary:
        return False
    nonce = secrets.token_hex(16)
    script = f"""
set -e
OLD={shlex.quote(old_binary)}
NEW={shlex.quote(new_binary)}
[ -x "$OLD" ] || exit 11
[ -x "$NEW" ] || exit 12
OLD_REPORT=$("$OLD" formats report --nonce {nonce} 2>/dev/null || true)
NEW_POLICY=$("$NEW" formats policy --json 2>/dev/null || true)
python3 - "$OLD_REPORT" "$NEW_POLICY" <<'PY'
import json, sys
try:
    old = json.loads(sys.argv[1])
    pol = json.loads(sys.argv[2])
except Exception:
    print("false")
    sys.exit(0)
old_reads = set(old.get("reads") or [])
req = set(pol.get("required_reads") or [])
print("true" if (old_reads and req and req.issubset(old_reads)) else "false")
PY
"""
    res = runner.run(script, timeout=20)
    return res.returncode == 0 and res.stdout.strip().splitlines()[-1:] == ["true"]


def select_binary_for_host(
    info: dict[str, Any], linux_bin: Path | None, macos_bin: Path | None
) -> Path | None:
    os_name = info.get("os")
    arch = info.get("arch")
    if os_name == "Linux" and arch == "x86_64":
        return linux_bin
    if os_name == "Darwin" and arch == "arm64":
        return macos_bin
    return None


def deploy_one_host(
    runner: HostRunner,
    info: dict[str, Any],
    src_binary: Path,
    expected_sha: str,
    build_name: str,
    is_master: bool,
    backup_db: bool,
    rollback_policy: str,
    dry_run: bool,
) -> dict[str, Any]:
    home_dir = info["home"]
    os_name = info["os"]
    remote_bin = f"{home_dir}/{INSTALL_ROOT_REL}/{build_name}/bin/bridget"
    previous_bin = info.get("active_binary")
    raw_services = list((info.get("update_host") or {}).get("services") or [])
    if os_name == "Linux":
        if not raw_services:
            raw_services = ["core", "worker", "mobile"] if is_master else ["worker"]
        services = map_systemd_services(runner.machine_id, raw_services)
    else:
        if not raw_services:
            raw_services = [f"com.bridget.worker.{runner.machine_id}"]
        services = filter_safe_services(raw_services)

    plan = {
        "machine_id": runner.machine_id,
        "os": os_name,
        "arch": info.get("arch"),
        "target_binary": remote_bin,
        "previous_binary": previous_bin,
        "services": services,
        "backup_db": bool(is_master and backup_db),
    }
    if dry_run:
        plan["status"] = "dry_run"
        return plan

    # 1. Sauvegarde SQLite optionnelle sur le maître avant activation.
    if is_master and backup_db:
        bak = f"{home_dir}/{CACHE_DIR_REL}/bridget.db.before-{build_name}.bak"
        bak_res = runner.run(
            f"set -e; sqlite3 {shlex.quote(home_dir + '/' + CACHE_DIR_REL + '/bridget.db')} \".backup '{bak}'\"; chmod 600 {shlex.quote(bak)}",
            timeout=60,
        )
        if bak_res.returncode != 0:
            raise RuntimeError(f"sauvegarde bridget.db échouée: {bak_res.stderr.strip()}")
        plan["db_backup"] = bak

    # 2. Copie du binaire et vérification SHA-256.
    runner.copy_binary(src_binary, remote_bin)
    sha_cmd = (
        f"sha256sum {shlex.quote(remote_bin)} | awk '{{print $1}}'"
        if os_name == "Linux"
        else f"shasum -a 256 {shlex.quote(remote_bin)} | awk '{{print $1}}'"
    )
    sha_res = runner.run(sha_cmd, timeout=20)
    actual_sha = (sha_res.stdout or "").strip()
    if sha_res.returncode != 0 or actual_sha != expected_sha:
        raise RuntimeError(
            f"empreinte SHA-256 invalide sur {runner.machine_id}: {actual_sha!r} != {expected_sha!r}"
        )

    # 3. Qualification du rollback (jamais de faux rollback incompatible).
    preserve_existing_rollback = bool(previous_bin and previous_bin == remote_bin)
    if rollback_policy == "null":
        rollback_target = None
        preserve_existing_rollback = False
    elif preserve_existing_rollback:
        rollback_target = None
    else:
        rollback_target = (
            previous_bin
            if check_remote_rollback_compatible(runner, previous_bin, remote_bin)
            else None
        )
    plan["rollback"] = "preserved" if preserve_existing_rollback else rollback_target

    # 4. Activation atomique + mise à jour 0600 de update-host-state.json + redémarrage services.
    rb_py = repr(rollback_target)
    preserve_py = repr(preserve_existing_rollback)
    if os_name == "Linux":
        svc_args = " ".join(shlex.quote(s) for s in services)
        activate_script = f"""
set -e
systemctl --user stop bridget-updates.service 2>/dev/null || true
ln -sfn {shlex.quote(remote_bin)} "$HOME/.local/bin/bridget.next"
mv -Tf "$HOME/.local/bin/bridget.next" "$HOME/.local/bin/bridget"
python3 - <<'PY'
import fcntl, json, os, shutil
os.umask(0o077)
cache = os.path.expanduser("~/.cache/bridget-core")
os.makedirs(cache, exist_ok=True)
lock_path = os.path.join(cache, "update-host.lock")
state_path = os.path.join(cache, "update-host-state.json")
with open(lock_path, "a+") as lf:
    fcntl.flock(lf.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    data = {{}}
    if os.path.exists(state_path):
        with open(state_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        shutil.copy2(state_path, state_path + ".bak")
        os.chmod(state_path + ".bak", 0o600)
    if not {preserve_py}:
        data["rollback"] = {rb_py}
    tmp = state_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
        f.flush()
        os.fsync(f.fileno())
    os.chmod(tmp, 0o600)
    os.replace(tmp, state_path)
    os.chmod(state_path, 0o600)
PY
systemctl --user restart {svc_args} bridget-updates.service
for svc in {svc_args}; do
  state=""
  pid=""
  exe="?"
  for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15; do
    state=$(systemctl --user is-active "$svc" || true)
    pid=$(systemctl --user show -p MainPID --value "$svc" || true)
    exe=$(readlink -f "/proc/$pid/exe" 2>/dev/null || echo "?")
    if [ "$state" = "active" ] && [ "$exe" = {shlex.quote(remote_bin)} ]; then
      break
    fi
    sleep 1
  done
  echo "VERIFY:$svc:$state:$pid:$exe"
  [ "$state" = "active" ] || {{ echo "SERVICE_NOT_ACTIVE:$svc:$state" >&2; exit 21; }}
  [ "$exe" = {shlex.quote(remote_bin)} ] || {{ echo "EXE_MISMATCH:$svc:$exe" >&2; exit 22; }}
done
perms=$(stat -c '%a' "$HOME/.cache/bridget-core/update-host-state.json")
[ "$perms" = "600" ] || {{ echo "BAD_PERMS:$perms" >&2; exit 23; }}
"""
    else:
        kickstarts = "\n".join(
            f'launchctl kickstart -k "gui/${{UID_NUM}}/{svc}"' for svc in services
        )
        verifies = "\n".join(
            f"""
pid=$(launchctl list | awk '$3=="{svc}"{{print $1}}')
[ -n "$pid" ] && [ "$pid" != "-" ] || {{ echo "SERVICE_NOT_RUNNING:{svc}" >&2; exit 21; }}
lsof -p "$pid" 2>/dev/null | grep -Fq {shlex.quote(remote_bin)} || {{ echo "EXE_MISMATCH:{svc}:$pid" >&2; exit 22; }}
echo "VERIFY:{svc}:active:$pid:{remote_bin}"
"""
            for svc in services
        )
        activate_script = f"""
set -e
UID_NUM=$(id -u)
launchctl bootout "gui/${{UID_NUM}}/com.bridget.updates" 2>/dev/null || true
sleep 1
ln -sfn {shlex.quote(remote_bin)} "$HOME/.local/bin/bridget.next"
mv -f "$HOME/.local/bin/bridget.next" "$HOME/.local/bin/bridget"
python3 - <<'PY'
import fcntl, json, os, shutil
os.umask(0o077)
cache = os.path.expanduser("~/.cache/bridget-core")
os.makedirs(cache, exist_ok=True)
lock_path = os.path.join(cache, "update-host.lock")
state_path = os.path.join(cache, "update-host-state.json")
with open(lock_path, "a+") as lf:
    fcntl.flock(lf.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
    data = {{}}
    if os.path.exists(state_path):
        with open(state_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        shutil.copy2(state_path, state_path + ".bak")
        os.chmod(state_path + ".bak", 0o600)
    if not {preserve_py}:
        data["rollback"] = {rb_py}
    tmp = state_path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
        f.flush()
        os.fsync(f.fileno())
    os.chmod(tmp, 0o600)
    os.replace(tmp, state_path)
    os.chmod(state_path, 0o600)
PY
{kickstarts}
if [ -f "$HOME/Library/LaunchAgents/com.bridget.updates.plist" ]; then
  launchctl bootstrap "gui/${{UID_NUM}}" "$HOME/Library/LaunchAgents/com.bridget.updates.plist" 2>/dev/null || true
fi
sleep 2
{verifies}
perms=$(stat -f '%Lp' "$HOME/.cache/bridget-core/update-host-state.json")
[ "$perms" = "600" ] || {{ echo "BAD_PERMS:$perms" >&2; exit 23; }}
"""
    act_res = runner.run(activate_script, timeout=60)
    if act_res.returncode != 0:
        raise RuntimeError(
            f"activation échouée sur {runner.machine_id}: {(act_res.stderr or act_res.stdout).strip()}"
        )
    plan["verified_services"] = [
        line.split(":", 1)[1]
        for line in act_res.stdout.splitlines()
        if line.startswith("VERIFY:")
    ]

    # 5. Vérification santé du maître si applicable.
    if is_master:
        st_res = runner.run('"$HOME/.local/bin/bridget" status', timeout=20)
        if st_res.returncode != 0:
            raise RuntimeError(f"bridget status échoué sur le maître: {st_res.stderr.strip()}")
        plan["master_status_ok"] = True

    # 6. Nettoyage conservateur des anciens binaires inutilisés (garde actif + précédent + ouverts).
    keep_names = {build_name}
    if previous_bin:
        prev_parts = Path(previous_bin).parts
        if "bridget-communication" in prev_parts:
            idx = prev_parts.index("bridget-communication")
            if idx + 1 < len(prev_parts):
                keep_names.add(prev_parts[idx + 1])
    keep_literal = " ".join(shlex.quote(k) for k in sorted(keep_names))
    if os_name == "Linux":
        clean_script = f"""
for d in "$HOME/{INSTALL_ROOT_REL}"/*; do
  [ -d "$d" ] || continue
  name=$(basename "$d")
  skip=0
  for k in {keep_literal}; do [ "$name" = "$k" ] && skip=1; done
  [ "$skip" -eq 1 ] && continue
  in_use=0
  for exe in /proc/[0-9]*/exe; do
    t=$(readlink -f "$exe" 2>/dev/null || true)
    case "$t" in "$d"/*) in_use=1;; esac
  done
  if [ "$in_use" -eq 0 ]; then
    rm -rf "$d"
    echo "REMOVED:$name"
  else
    echo "KEPT_IN_USE:$name"
  fi
done
"""
    else:
        clean_script = f"""
for d in "$HOME/{INSTALL_ROOT_REL}"/*; do
  [ -d "$d" ] || continue
  name=$(basename "$d")
  skip=0
  for k in {keep_literal}; do [ "$name" = "$k" ] && skip=1; done
  [ "$skip" -eq 1 ] && continue
  if lsof +D "$d" >/dev/null 2>&1; then
    echo "KEPT_IN_USE:$name"
  else
    rm -rf "$d"
    echo "REMOVED:$name"
  fi
done
"""
    clean_res = runner.run(clean_script, timeout=30)
    plan["removed_builds"] = [
        line.split(":", 1)[1]
        for line in clean_res.stdout.splitlines()
        if line.startswith("REMOVED:")
    ]
    plan["status"] = "activated"
    return plan


def load_agents_json(agents_json: Path | None, master: str, master_ssh: str) -> dict[str, Any]:
    if agents_json is not None:
        return json.loads(agents_json.read_text(encoding="utf-8"))
    res = subprocess.run(
        [
            "ssh",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            master_ssh,
            "cat ~/.cache/bridget-core/agents.json",
        ],
        capture_output=True,
        text=True,
        timeout=15,
    )
    if res.returncode != 0:
        raise RuntimeError(
            f"impossible de lire agents.json depuis {master} ({master_ssh}): {res.stderr.strip()}"
        )
    return json.loads(res.stdout)


def parse_ssh_map(entries: list[str]) -> dict[str, str]:
    mapping: dict[str, str] = {}
    for item in entries:
        if "=" not in item:
            raise ValueError(f"--ssh-map invalide (attendu machine_id=alias): {item}")
        k, v = item.split("=", 1)
        mapping[k.strip()] = v.strip()
    return mapping


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--build-name", required=True, help="Nom de répertoire de build (ex. 357-fix-r1)")
    parser.add_argument("--linux-bin", type=Path, default=None, help="Chemin local du binaire Linux x86_64")
    parser.add_argument("--macos-bin", type=Path, default=None, help="Chemin local du binaire macOS arm64")
    parser.add_argument("--agents-json", type=Path, default=None, help="Fichier agents.json local (sinon lu sur --master)")
    parser.add_argument("--master", default="cartae", help="machine_id du maître (défaut: cartae)")
    parser.add_argument("--canary", default="sysle2", help="machine_id du worker canari (défaut: sysle2)")
    parser.add_argument("--local-machine", default=None, help="machine_id exécuté en local sans SSH")
    parser.add_argument("--ssh-map", action="append", default=[], help="Override SSH machine_id=alias")
    parser.add_argument("--rollback-policy", choices=("auto", "null"), default="auto")
    parser.add_argument("--backup-db", action="store_true", help="Sauvegarder bridget.db sur le maître avant redémarrage")
    parser.add_argument("--canary-only", action="store_true", help="S'arrêter après validation du lot canari")
    parser.add_argument("--dry-run", action="store_true", help="Planifier et sonder sans modifier aucun hôte")
    parser.add_argument("--json", action="store_true", help="Afficher le bilan complet au format JSON")
    args = parser.parse_args(argv)

    ssh_map = parse_ssh_map(args.ssh_map)
    local_machine = args.local_machine or detect_local_machine_id()
    linux_sha = sha256_file(args.linux_bin) if args.linux_bin else None
    macos_sha = sha256_file(args.macos_bin) if args.macos_bin else None

    agents_data = load_agents_json(args.agents_json, args.master, ssh_map.get(args.master, args.master))
    machines = discover_enabled_machines(agents_data)
    canary_ids, rest_ids = plan_rollout_order(machines, args.master, args.canary)

    report: dict[str, Any] = {
        "build_name": args.build_name,
        "dry_run": args.dry_run,
        "discovered_machines": machines,
        "canary_plan": canary_ids,
        "fleet_plan": rest_ids,
        "activated": [],
        "unreachable": [],
        "skipped_missing_binary": [],
        "failed": [],
        "canary_ok": False,
    }

    def run_phase(ids: list[str], phase: str) -> bool:
        ok = True
        for mid in ids:
            runner = HostRunner(
                machine_id=mid,
                ssh_alias=ssh_map.get(mid, mid),
                is_local=(mid == local_machine),
            )
            info = probe_host(runner)
            if not info.get("reachable"):
                report["unreachable"].append({"machine_id": mid, "phase": phase, "error": info.get("error")})
                if mid == args.master:
                    ok = False
                continue
            src = select_binary_for_host(info, args.linux_bin, args.macos_bin)
            if src is None:
                report["skipped_missing_binary"].append(
                    {"machine_id": mid, "os": info.get("os"), "arch": info.get("arch")}
                )
                continue
            expected_sha = linux_sha if src == args.linux_bin else macos_sha
            assert expected_sha is not None
            try:
                result = deploy_one_host(
                    runner=runner,
                    info=info,
                    src_binary=src,
                    expected_sha=expected_sha,
                    build_name=args.build_name,
                    is_master=(mid == args.master),
                    backup_db=args.backup_db,
                    rollback_policy=args.rollback_policy,
                    dry_run=args.dry_run,
                )
                result["phase"] = phase
                report["activated"].append(result)
            except Exception as exc:
                report["failed"].append({"machine_id": mid, "phase": phase, "error": str(exc)})
                ok = False
                if phase == "canary":
                    return False
        return ok

    canary_ok = run_phase(canary_ids, "canary")
    report["canary_ok"] = canary_ok
    if canary_ok and not args.canary_only:
        run_phase(rest_ids, "fleet")

    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print(f"[deploy-fleet] build={args.build_name} dry_run={args.dry_run} canary_ok={canary_ok}")
        for item in report["activated"]:
            print(
                f"  OK [{item['phase']}] {item['machine_id']} ({item['os']}/{item['arch']}) "
                f"services={item['services']} rollback={item.get('rollback')}"
            )
        for item in report["unreachable"]:
            print(f"  INJOIGNABLE [{item['phase']}] {item['machine_id']} : {item['error']}")
        for item in report["skipped_missing_binary"]:
            print(f"  IGNORE (binaire absent) {item['machine_id']} ({item['os']}/{item['arch']})")
        for item in report["failed"]:
            print(f"  ECHEC [{item['phase']}] {item['machine_id']} : {item['error']}")

    return 0 if (canary_ok and not report["failed"]) else 1


if __name__ == "__main__":
    sys.exit(main())
