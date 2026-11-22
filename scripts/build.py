#!/usr/bin/env python3
"""Cargo avec entretien avant et après exécution ; jamais de cargo clean global."""

import argparse
import fcntl
import hashlib
import json
import math
import os
import platform
import re
import selectors
import shlex
import signal
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
from collections import namedtuple


CACHE_DIRS = ("deps", "build", "incremental", ".fingerprint")
OWNER = ".bridget-build-owner.json"
BUILD_SETTINGS = Path.home().resolve() / ".config/bridget/build.json"
ROOT_ANCHOR = ".bridget-build-root.json"
SYSTEM_DEVICE = Path("/").stat().st_dev
GIB = 1024 ** 3
VALID_OWNER_SHA_RE = re.compile(r"^[0-9a-f]{40}([0-9a-f]{24})?$")
KNOWN_HEAVY_TARGETS = ("macos-arm64", "linux-x86_64-glibc2.36")
REMOTE_LOCK_FILENAME = ".heavy-work.lock"
HEAVY_WORK_RENEW_INTERVAL_SECS = 20.0
HEAVY_WORK_HEARTBEAT_INTERVAL_SECS = 10.0
HEAVY_WORK_HEARTBEAT_TIMEOUT_SECS = 30.0
HEAVY_WORK_KILL_GRACE_SECS = 10.0
HEAVY_WORK_RESULT_PREFIX = "BRIDGET_HEAVY_WORK_RESULT:"


def log(message):
    print(f"[build Bridget] {message}", file=sys.stderr, flush=True)


def plain_dir(path):
    return path.is_dir() and not path.is_symlink() and path.absolute() == path.resolve()


def git_ceiling_env(repo):
    env = dict(os.environ, LC_ALL="C")
    parent = str(Path(repo).resolve().parent)
    existing = env.get("GIT_CEILING_DIRECTORIES")
    env["GIT_CEILING_DIRECTORIES"] = f"{parent}:{existing}" if existing else parent
    return env


def worktrees(repo):
    result = subprocess.run(
        ["git", "-C", str(repo), "worktree", "list", "--porcelain", "-z"],
        capture_output=True, check=False, timeout=10, env=git_ceiling_env(repo),
    )
    if result.returncode:
        return [repo]  # Paquet source sans Git : aucun autre projet à examiner.
    return [Path(os.fsdecode(line[9:])) for line in result.stdout.split(b"\0")
            if line.startswith(b"worktree ")]


def owner_id(repo):
    result = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--path-format=absolute", "--git-common-dir"],
        text=True, capture_output=True, check=False, timeout=10, env=git_ceiling_env(repo),
    )
    return str(Path(result.stdout.strip()).resolve()) if result.returncode == 0 else str(repo)


def canonical_owner_id(repo):
    """Identifiant canonique partagé entre local et distant : SHA du commit racine."""
    env_owner = os.environ.get("BRIDGET_BUILD_OWNER", "").strip().lower()
    if env_owner and VALID_OWNER_SHA_RE.fullmatch(env_owner):
        return env_owner
    result = subprocess.run(
        ["git", "-C", str(repo), "rev-list", "--max-parents=0", "HEAD"],
        text=True, capture_output=True, check=False, timeout=10, env=git_ceiling_env(repo),
    )
    if result.returncode == 0:
        for line in result.stdout.splitlines():
            candidate = line.strip().lower()
            if VALID_OWNER_SHA_RE.fullmatch(candidate):
                return candidate
    return None


def staging_owner_id(repo):
    return canonical_owner_id(repo) or owner_id(repo)


def remote_cache_root():
    """Résolution dédiée de la racine de cache distante (§3.2.4).
    Si build.json existe et est invalide, refuse sans repli."""
    external = external_staging_parent()
    return external if external is not None else staging_parent()


def remote_cache_dir_name(repo_slug, target):
    safe_slug = re.sub(r"[^A-Za-z0-9._-]+", "-", str(repo_slug)).strip("-") or "repo"
    safe_target = re.sub(r"[^A-Za-z0-9._-]+", "-", str(target)).strip("-") or "target"
    return f"bridget-build-{safe_slug}-{safe_target}"


def staging_parent():
    # Ne pas étendre ce périmètre à XDG_CACHE_HOME ou à un chemin fourni au nettoyeur.
    return Path.home().resolve() / ".cache"


def owned_json(path, private=False):
    """Lire un petit marqueur du compte, sans suivre son dernier composant."""
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, encoding="utf-8") as stream:
        info = os.fstat(stream.fileno())
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid()
                or info.st_size > 4096 or (private and info.st_mode & 0o077)):
            raise ValueError("fichier du compte requis, privé pour la configuration")
        raw = stream.read(4097)
        if len(raw) > 4096:
            raise ValueError("marqueur trop grand")
        return json.loads(raw)


def external_staging_parent():
    if not BUILD_SETTINGS.exists() and not BUILD_SETTINGS.is_symlink():
        return None
    try:
        settings = owned_json(BUILD_SETTINGS, private=True)
        if (not isinstance(settings, dict) or set(settings) != {"version", "root", "root_id"}
                or type(settings["version"]) is not int or settings["version"] != 1
                or not isinstance(settings["root"], str)
                or not isinstance(settings["root_id"], str)
                or not re.fullmatch(r"[0-9a-f]{32}", settings["root_id"])):
            raise ValueError("format de configuration non reconnu")
        root = Path(settings["root"])
        if (not plain_dir(root) or not root.is_absolute() or root.stat().st_uid != os.getuid()
                or root.stat().st_mode & 0o077):
            raise ValueError("volume absent ou racine réelle privée 0700 indisponible")
        if root.stat().st_dev == SYSTEM_DEVICE:
            raise ValueError("la racine configurée est sur le système de fichiers système")
        if owned_json(root / ROOT_ANCHOR, private=True) != {"version": 1, "id": settings["root_id"]}:
            raise ValueError("ancre du volume différente de la configuration")
        return root
    except (OSError, ValueError) as error:
        raise ValueError(
            f"configuration de compilation refusée ({error}). Vérifiez {BUILD_SETTINGS}, "
            "le montage et son ancre privée ; aucun repli vers le disque système. "
            "Les autres travaux autorisés suivent leur propre état."
        ) from error


def staging_parents():
    try:
        external = external_staging_parent()
    except ValueError as error:
        log(f"Racine externe ignorée pour cet entretien seulement : {error}")
        external = None
    return tuple(dict.fromkeys([staging_parent(), *([external] if external else [])]))


def configure_build_target(repo, arguments):
    root = external_staging_parent()
    if root is None:
        return
    target = os.environ.get("CARGO_TARGET_DIR")
    for index, argument in enumerate(arguments):
        if argument.startswith("--target-dir="):
            target = argument.split("=", 1)[1]
        elif argument == "--target-dir" and index + 1 < len(arguments):
            target = arguments[index + 1]
    if not target:
        key = hashlib.sha256(owner_id(repo).encode()).hexdigest()[:16]
        target = str(root / f"bridget-build-{key}")
        os.environ["CARGO_TARGET_DIR"] = target
    path = Path(os.path.abspath(repo / target))
    if (".." in Path(target).parts or path.parent != root or not path.name.startswith("bridget-build-")
            or path.name == "bridget-build-" or path.is_symlink()
            or (path.exists() and (not plain_dir(path) or path.stat().st_uid != os.getuid()))):
        raise ValueError(
            "cible Cargo hors de la racine configurée ou indirecte. Retirez l’ancien "
            "CARGO_TARGET_DIR/--target-dir pour utiliser le cache configuré, ou choisissez "
            "un enfant direct bridget-build-* de cette racine. Aucun repli automatique."
        )
    return root


def is_staging(path, parents):
    return (plain_dir(path) and path.parent in parents
            and path.name.startswith("bridget-build-") and path.name != "bridget-build-")


def register_staging(repo, arguments):
    """Enregistrer uniquement un cache temporaire explicitement utilisé par Cargo."""
    target = os.environ.get("CARGO_TARGET_DIR")
    for index, argument in enumerate(arguments):
        if argument.startswith("--target-dir="):
            target = argument.split("=", 1)[1]
        elif argument == "--target-dir" and index + 1 < len(arguments):
            target = arguments[index + 1]
    if not target:
        return
    path = Path(os.path.abspath(repo / target))
    parents = staging_parents()
    if (path.parent not in parents or not path.name.startswith("bridget-build-")
            or path.name == "bridget-build-"):
        return
    # Créer le cache avant Cargo : son mode par défaut pourrait le rendre
    # traversable et empêcher toute observation limitée au compte propriétaire.
    path.parent.mkdir(mode=0o700, exist_ok=True)
    if not plain_dir(path.parent):
        return
    try:
        path.mkdir(mode=0o700)
    except FileExistsError:
        pass  # Ne pas changer les droits d'un cache préexistant.
    if not is_staging(path, parents):
        return
    marker = path / OWNER
    if marker.exists() or marker.is_symlink():
        return  # Ne jamais s'approprier un cache déjà marqué, même par un autre dépôt.
    with marker.open("x", encoding="utf-8") as stream:
        json.dump({"version": 1, "repo": staging_owner_id(repo)}, stream)


def owned_staging(repo):
    identities = {owner_id(repo)}
    canon = canonical_owner_id(repo)
    if canon:
        identities.add(canon)
    parents = staging_parents()
    for parent in parents:
        for path in parent.glob("bridget-build-*"):
            marker = path / OWNER
            if not is_staging(path, parents) or marker.is_symlink() or not marker.is_file():
                continue
            try:
                data = owned_json(marker)
            except (OSError, ValueError):
                continue
            if (isinstance(data, dict) and set(data) == {"version", "repo"}
                    and data.get("version") == 1 and data.get("repo") in identities):
                yield path


def profiles(repo):
    roots = [root / "target" for root in worktrees(repo) if plain_dir(root)]
    roots.extend(owned_staging(repo))
    seen = set()
    for root in roots:
        if not plain_dir(root):
            continue
        # Liste fermée : pas de parcours arbitraire du home ou d'un target-dir externe.
        for name in ("debug", "release"):
            path = root / name
            lock = path / ".cargo-lock"
            if (path not in seen and plain_dir(path) and lock.is_file()
                    and not lock.is_symlink()):
                seen.add(path)
                yield path


def measure(profile):
    """O(nombre de fichiers), sans suivre de liens ni changer de volume."""
    size, newest = 0, 0
    device = profile.stat().st_dev
    for name in CACHE_DIRS:
        root = profile / name
        if not root.exists():
            continue
        if not plain_dir(root):
            raise ValueError(f"cache indirect refusé : {root}")
        for folder, dirs, files in os.walk(root, followlinks=False):
            for item in [Path(folder)] + [Path(folder) / n for n in dirs + files]:
                info = item.lstat()
                if info.st_dev != device:
                    raise ValueError(f"autre volume refusé : {item}")
                newest = max(newest, info.st_mtime)
                if stat.S_ISREG(info.st_mode):
                    size += info.st_size
    return size, newest


def opened_paths(scopes=()):
    paths = busy_paths(scopes, ignore_own=tuple(scope / ".cargo-lock" for scope in scopes))
    if paths is None:
        raise RuntimeError("observation des processus incomplète ; nettoyage ignoré")
    return [str(path) for path in paths]


def rmtree_no_follow(parent_fd, name):
    """Suppression récursive sûre ancrée par descripteur (Python 3.9+, sans suivre de lien)."""
    fd = os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent_fd)
    try:
        for entry in os.listdir(fd):
            info = os.stat(entry, dir_fd=fd, follow_symlinks=False)
            if stat.S_ISDIR(info.st_mode):
                rmtree_no_follow(fd, entry)
            else:
                os.unlink(entry, dir_fd=fd)
    finally:
        os.close(fd)
    os.rmdir(name, dir_fd=parent_fd)


def prune(repo, max_bytes, max_age, dry_run=False, only=None):
    candidates = []
    for profile in profiles(repo):
        try:
            size, newest = measure(profile)
            if size:
                candidates.append((newest, str(profile), size))
        except (OSError, ValueError) as error:
            log(str(error))
    total = sum(size for _, _, size in candidates)
    removed = 0
    for newest, name, size in sorted(candidates):
        profile = Path(name)
        if only is not None and profile.parent != only:
            continue
        if only is None and total <= max_bytes and time.time() - newest < max_age:
            continue
        # Ce fichier reste en place : supprimer le verrou permettrait deux verrous indépendants.
        try:
            fd = os.open(profile / ".cargo-lock", os.O_RDWR | os.O_NOFOLLOW)
            with os.fdopen(fd, "r+") as lock:
                try:
                    fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    log(f"compilation active, conservée : {profile}")
                    continue
                if not plain_dir(profile):
                    raise ValueError(f"profil déplacé ou indirect : {profile}")
                # Recontrôler sous verrou : une compilation peut avoir fini depuis le scan.
                current_size, current_newest = measure(profile)
                if current_size != size or current_newest != newest:
                    log(f"cache modifié pendant le contrôle, conservé : {profile}")
                    continue
                if any(p == name or p.startswith(name + "/") for p in opened_paths((profile,))):
                    log(f"fichiers utilisés, conservés : {profile}")
                    continue
                if dry_run:
                    log(f"simulation : {size / GIB:.2f} Gio supprimables dans {profile}")
                else:
                    directory = os.open(profile, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
                    try:
                        for child in CACHE_DIRS:
                            try:
                                info = os.stat(child, dir_fd=directory, follow_symlinks=False)
                            except FileNotFoundError:
                                continue
                            if stat.S_ISDIR(info.st_mode):
                                rmtree_no_follow(directory, child)
                            else:
                                raise ValueError(f"cache changé ou indirect : {profile / child}")
                    finally:
                        os.close(directory)
                    log(f"cache supprimé : {size / GIB:.2f} Gio dans {profile}")
                total -= size
                removed += size
        except (OSError, ValueError) as error:
            log(f"cache conservé ou nettoyage incomplet : {error}")
    log(f"{'Simulation' if dry_run else 'Nettoyage'} : {removed / GIB:.2f} Gio de caches ; "
        f"{total / GIB:.2f} Gio restants (tailles logiques, clones APFS compris)")
    if total > max_bytes:
        log("budget dépassé : caches protégés conservés, aucune suppression forcée")
    return removed


def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as stream:
        while True:
            chunk = stream.read(65536)
            if not chunk:
                break
            hasher.update(chunk)
    return hasher.digest()


def installed(repo, target, binary, dry_run):
    target = Path(os.path.abspath(target))
    if target in {root / "target" for root in worktrees(repo)}:
        return prune(repo, *limits(), dry_run=dry_run)
    if target not in set(owned_staging(repo)):
        raise ValueError("seul un staging enregistré de ce dépôt peut être purgé après installation")
    source = target / "release" / "bridget"
    binary = Path(binary).resolve(strict=True)
    if (binary.is_relative_to(target) or source.is_symlink()
            or not source.is_file() or digest(source) != digest(binary)):
        raise ValueError("installation non attestée : binaire différent ou encore dans le staging")
    # Garder le petit exécutable et les verrous, jamais des Gio d'intermédiaires.
    return prune(repo, 0, 0, dry_run, only=target)


def limits():
    size = float(os.environ.get("BRIDGET_BUILD_CACHE_GIB", "10"))
    days = float(os.environ.get("BRIDGET_BUILD_CACHE_DAYS", "7"))
    if not all(math.isfinite(n) and n >= 0 for n in (size, days)):
        raise ValueError("limites de cache invalides : valeurs finies positives ou nulles requises")
    return size * GIB, days * 86400


# Session 116 : constitution XVI.3, un worktree fusionné est retiré sous 24 h.
WORKTREE_MIN_AGE = 24 * 3600


Worktree = namedtuple("Worktree", "path head locked prunable")


def worktree_entries(repo, budget=None):
    """Worktrees Git, dépôt principal en premier ; RuntimeError si le registre est illisible."""
    result = git_ok(repo, "worktree", "list", "--porcelain", "-z", budget=budget)
    if result.returncode:
        raise RuntimeError(f"registre Git illisible : {repo}")
    entries, path, head, locked, prunable = [], None, None, False, False
    for field in result.stdout.split(b"\0") + [b""]:
        if field.startswith(b"worktree "):
            path = Path(os.fsdecode(field[9:]))
        elif field.startswith(b"HEAD "):
            head = field[5:].decode()
        elif field == b"locked" or field.startswith(b"locked "):
            locked = True
        elif field == b"prunable" or field.startswith(b"prunable "):
            prunable = True
        elif not field and path is not None:
            entries.append(Worktree(path, head, locked, prunable))
            path, head, locked, prunable = None, None, False, False
    if len(entries) > MAX_WORKTREE_ENTRIES:
        raise RuntimeError("registre Git trop grand")
    if not entries:
        raise RuntimeError(f"registre Git vide : {repo}")
    return entries


LSOF_OPEN_FILES = ["lsof", "-n", "-P", "-F", "pftDn"]
OBSERVATION_SECONDS = 30
MAX_OBSERVATION_BYTES = 8 * 1024 * 1024
MAX_WORKTREE_ENTRIES = 4096


class ObservationBudget:
    """Un seul budget monotone et des faits réutilisables pendant un passage."""
    def __init__(self, seconds=OBSERVATION_SECONDS):
        self.started = time.monotonic()
        self.deadline = self.started + seconds
        self.failures = set()
        self.git_calls = 0
        self.facts = {}
        self.process_reader = "unavailable"
        self.exempt_mounts = 0
        self.owner_private_scope = False

    def remaining(self):
        return max(0, self.deadline - time.monotonic())


def observed_run(arguments, budget=None, timeout=15, limit=MAX_OBSERVATION_BYTES, env=None):
    """Captures bornées ; aucune donnée partielle n'est interprétée comme complète."""
    budget = budget or ObservationBudget()
    end = min(budget.deadline, time.monotonic() + timeout)
    output = [bytearray(), bytearray()]
    child = None
    failure = None
    try:
        if time.monotonic() >= end:
            raise TimeoutError
        child = subprocess.Popen(arguments, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                 start_new_session=True, env=env if env is not None else dict(os.environ, LC_ALL="C"))
        with selectors.DefaultSelector() as selector:
            for index, stream in enumerate((child.stdout, child.stderr)):
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, index)
            while selector.get_map():
                remaining = end - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError
                for key, _ in selector.select(min(remaining, 0.1)):
                    chunk = os.read(key.fd, min(65536, limit - len(output[key.data]) + 1))
                    if not chunk:
                        selector.unregister(key.fileobj)
                    else:
                        output[key.data].extend(chunk)
                        if sum(map(len, output)) > limit:
                            raise OverflowError
            child.wait(timeout=max(0.001, end-time.monotonic()))
        return subprocess.CompletedProcess(arguments, child.returncode, bytes(output[0]), bytes(output[1]))
    except (TimeoutError, subprocess.TimeoutExpired):
        failure = "deadline"
    except OverflowError:
        failure = "output_limit"
    except (OSError, ValueError):
        failure = "tool_unavailable"
    finally:
        if child is not None:
            if child.poll() is None or failure is not None:
                # Groupe créé par cet appel uniquement ; jamais un PID découvert ailleurs.
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                    child.wait(timeout=1)
                except ProcessLookupError:
                    pass
                except (subprocess.TimeoutExpired, OSError):
                    budget.failures.add("process_stop_unconfirmed")
            for stream in (child.stdout, child.stderr):
                stream.close()
    budget.failures.add(failure)
    return subprocess.CompletedProcess(arguments, 124, b"", b"")


def process_output_complete(stdout, exempt=(), diagnostics=None):
    # Les types explicitement non fichiers n'ont pas de chemin à protéger.
    nonfiles = {"a_inode", "netlink", "unix", "IPv4", "IPv6", "sock", "pack", "PIPE",
                "KQUEUE", "PSXSEM", "PSXSHM", "systm", "NPOLICY", "FSEVENTS", "NEXUS", "CHAN", "rte"}
    pid, record = None, {}
    seen = False

    def refused():
        # Métadonnées fermées seulement : jamais le nom ou le chemin du fichier.
        if diagnostics is not None and not diagnostics:
            descriptor = record.get("f", "")
            if descriptor not in {"cwd", "rtd", "txt", "mem", "DEL", "NOFD"} and not re.fullmatch(r"[0-9]{1,8}[ruw]?", descriptor):
                descriptor = "unknown"
            kind = record.get("t", "unknown")
            if kind not in nonfiles | {"REG", "DIR", "CHR", "BLK", "FIFO", "LINK", "VREG", "VDIR", "unknown"}:
                kind = "unknown"
            code = "cwd_unavailable_enoent" if record.get("f") == "cwd" and record.get("n") == "cwd|rtd info error: No such file or directory" else "process_record_unverifiable"
            diagnostics.append({"code": code, "pid": int(pid) if pid and pid.isascii() and pid.isdigit() and len(pid) <= 10 else None,
                                "descriptor": descriptor, "type": kind})
        return False

    def valid():
        kind, name = record.get("t"), record.get("n", "")
        if "f" not in record:
            return False
        if kind in nonfiles:
            return True
        # macOS : un descripteur révoqué (revoke(2), terminal fermé) désigne un
        # nœud mort sans chemin ; il ne peut ouvrir aucun fichier du cache.
        if kind is None and name == "(revoked)":
            return True
        if kind == "FIFO" and name == "pipe":
            return True
        if kind == "REG" and name in {"net", "mnt", "pid", "user", "uts", "ipc", "cgroup", "time"}:
            return True
        if kind == "REG" and name == "anon_inode:[io_uring] (stat: No such file or directory)":
            return True
        # Une cible lue mais disparue du disque reste un chemin occupé, jamais une absence.
        if kind == "REG" and name.startswith("/"):
            name = name.removesuffix(" (stat: No such file or directory)")
        if kind == "unknown" and record.get("f") == "txt" and name == f"/proc/{pid}/exe":
            try:
                raw = Path(f"/proc/{pid}/stat").read_text()
                return bool(int(raw[raw.rfind(")")+2:].split()[6]) & 0x200000)
            except FileNotFoundError:
                return not Path(f"/proc/{pid}").exists()
            except (OSError, ValueError, IndexError):
                return False
        if kind not in {"REG", "DIR", "CHR", "BLK", "FIFO", "LINK", "VREG", "VDIR"}:
            return False
        if not name.startswith("/") or any(token in name for token in
                ("Permission denied", "(readlink:", "(opendir:", "(stat:", "(-e ", "(+e ")):
            return False
        path = Path(name.removesuffix(" (deleted)"))
        return not any(path == Path(m) or Path(m) in path.parents for m in exempt)

    for raw in stdout.splitlines():
        if not raw:
            continue
        try:
            field, value = chr(raw[0]), os.fsdecode(raw[1:])
            if field in {"p", "f"}:
                if record and not valid():
                    return refused()
                record = {}
            if field == "p":
                if not value.isdigit():
                    return refused()
                pid = value
            elif field in {"f", "t", "D", "n"}:
                if pid is None or field in record:
                    return refused()
                record[field] = value
                seen = True
            else:
                return refused()
        except (ValueError, UnicodeError):
            return refused()
    if not seen or record and not valid():
        return refused()
    return True


def parse_open_paths(stdout, ignore_own=()):
    # Le verrou Cargo détenu par ce nettoyeur reste sur disque ; aucun autre usage n'est exclu.
    paths, pid = [], None
    for line in stdout.splitlines():
        if line.startswith(b"p"):
            pid = line[1:]
        elif line.startswith(b"n/"):
            name = line[1:].removesuffix(b" (stat: No such file or directory)").removesuffix(b" (deleted)")
            path = Path(os.fsdecode(name))
            if pid != str(os.getpid()).encode() or path not in ignore_own:
                paths.append(path)
    return paths


def mount_snapshot():
    with open("/proc/self/mountinfo", "rb") as stream:
        data = stream.read(1024 * 1024 + 1)
    if len(data) > 1024 * 1024:
        raise ValueError("mount inventory too large")
    mounts = {}
    for line in data.decode().splitlines():
        fields = line.split()
        if len(fields) < 10 or "-" not in fields:
            raise ValueError("mount inventory incomplete")
        name = re.sub(r"\\([0-7]{3})", lambda m: chr(int(m[1], 8)), fields[4])
        point = Path(name)
        root = re.sub(r"\\([0-7]{3})", lambda m: chr(int(m[1], 8)), fields[3])
        mounts.setdefault(point, []).append((fields[2], Path(root), fields[fields.index("-")+1]))
    return data, mounts


def validate_mount_scopes(scopes, mounts, aliases=None):
    """Sans liste `aliases`, un second montage du même contenu reste refusé.
    Avec une liste, chaque chemin alias y est ajouté avec son chemin canonique
    pour que les fichiers ouverts par l'alias restent comptés."""
    protected = set()
    roots = [path.resolve(strict=True) for path in scopes]
    for root in roots:
        info = root.stat()
        device = f"{os.major(info.st_dev)}:{os.minor(info.st_dev)}"
        parents = [m for m in mounts if m == root or m in root.parents]
        if not parents:
            raise ValueError("mount identity unavailable")
        parent = max(parents, key=lambda m: len(m.parts))
        if len(mounts[parent]) != 1 or mounts[parent][0][0] != device or mounts[parent][0][2].startswith("fuse"):
            raise ValueError("mount identity ambiguous")
        identity = mounts[parent][0][1] / root.relative_to(parent)
        for other, layers in mounts.items():
            if other == parent:
                continue
            if root == other or root in other.parents:
                raise ValueError("nested mount in observed scope")
            for dev, source, _kind in layers:
                if dev == device and (identity == source or identity in source.parents or source in identity.parents):
                    if aliases is None:
                        raise ValueError("alias mount may hide usage")
                    if identity == source:
                        aliases.append((other, root))
                    elif identity in source.parents:
                        aliases.append((other, root / source.relative_to(identity)))
                    else:
                        aliases.append((other / identity.relative_to(source), root))
        protected.add(device)
    return roots, protected


def foreign_mount_exemptions(stderr, scopes, mounts):
    """Seuls des stat impossibles sur des périphériques étrangers sont exemptables."""
    lines = stderr.decode(errors="strict").splitlines()
    if not scopes or not lines or len(lines) % 2:
        raise ValueError("unclassified diagnostic")
    roots, protected = validate_mount_scopes(scopes, mounts)
    exempt = []
    for index in range(0, len(lines), 2):
        match = re.fullmatch(r"lsof: WARNING: can't stat\(\) \S+ file system (/.*)", lines[index])
        if not match or lines[index+1].strip() != "Output information may be incomplete.":
            raise ValueError("unclassified diagnostic")
        mount = Path(match[1])
        if mount not in mounts or len(mounts[mount]) != 1 or mounts[mount][0][0] in protected:
            raise ValueError("relevant mount unavailable")
        # lsof -e compare des préfixes bruts, donc refuser aussi les faux voisins de chaîne.
        if any(str(root).startswith(str(mount)) or str(mount).startswith(str(root)) for root in roots):
            raise ValueError("overlapping mount unavailable")
        exempt.append(str(mount))
    return sorted(set(exempt))


def owner_private_scope(scope):
    """Seuls ce compte et root peuvent atteindre ce dossier : un ancêtre réel lui
    appartient et ne laisse ni groupe ni autres le traverser."""
    try:
        path = Path(scope)
        if path.is_symlink() or path.resolve() != path.absolute():
            return False
        for item in (path, *path.parents):
            info = os.lstat(item)
            if stat.S_ISDIR(info.st_mode) and info.st_uid == os.geteuid() and not info.st_mode & 0o011:
                return True
    except OSError:
        return False
    return False


# Gardiens de session protégés par le système (non inspectables sans root) :
# ils tiennent la session SSH ou PAM et n'ouvrent jamais de cache de compilation.
SESSION_HELPERS = {"sshd", "(sd-pam)"}
BUILD_PROTECTED_COMMS = frozenset({
    "cargo", "rustc", "rustdoc", "clippy-driver", "bridget",
    "cc", "clang", "gcc", "c++", "clang++", "g++", "ld", "lld", "ld64", "ar", "ranlib", "libtool",
})


def process_comm(pid):
    """Lit le nom court du processus pour distinguer un outil de build d'un processus étranger."""
    if not pid or not pid.isdigit():
        return None
    if sys.platform.startswith("linux"):
        try:
            name = Path(f"/proc/{pid}/comm").read_text().strip()
            return Path(name).name.lower() if name else None
        except OSError:
            return None
    try:
        result = subprocess.run(
            ["ps", "-p", pid, "-o", "comm="],
            capture_output=True, text=True, timeout=2,
        )
        if result.returncode != 0:
            return None
        name = result.stdout.strip()
        return Path(name).name.lower() if name else None
    except (OSError, subprocess.SubprocessError):
        return None


def drop_foreign_unverifiable_records(stdout, scopes, comm_reader=None):
    """Ignore les descripteurs morts/transitoires (cwd ENOENT ou fd sans type)
    de processus clairement étrangers aux builds Bridget/Cargo lorsque l'observation
    est bornée à `scopes`. Fail-closed si la commande est illisible, si c'est un
    outil de build/Bridget, ou si le processus ouvre déjà un chemin dans `scopes`."""
    if not scopes:
        return stdout
    roots = {Path(scope).resolve() for scope in scopes}
    reader = comm_reader or process_comm
    blocks = []
    cur_pid, cur_lines = None, []
    for raw in stdout.splitlines(keepends=True):
        if raw.startswith(b"p"):
            if cur_lines:
                blocks.append((cur_pid, cur_lines))
            cur_pid = raw[1:].strip().decode(errors="replace")
            cur_lines = [raw]
        else:
            cur_lines.append(raw)
    if cur_lines:
        blocks.append((cur_pid, cur_lines))

    kept = []
    for pid, lines in blocks:
        header = []
        records = []
        cur = []
        for line in lines:
            if line.startswith(b"f"):
                if cur:
                    records.append(cur)
                cur = [line]
            elif records or cur:
                cur.append(line)
            else:
                header.append(line)
        if cur:
            records.append(cur)

        def _is_foreign_dead_record(rec):
            fields = {}
            for line in rec:
                if line:
                    fields[chr(line[0])] = os.fsdecode(line[1:].rstrip(b"\r\n"))
            fd = fields.get("f", "")
            kind = fields.get("t")
            name = fields.get("n", "")
            if fd == "cwd" and name == "cwd|rtd info error: No such file or directory":
                return True
            if kind in (None, "unknown") and re.fullmatch(r"[0-9]{1,8}[ruw]?", fd) and not name.startswith("/"):
                return True
            return False

        if not any(_is_foreign_dead_record(rec) for rec in records):
            kept.extend(lines)
            continue

        touches_scope = False
        for rec in records:
            for line in rec:
                if line.startswith(b"n"):
                    candidate = os.fsdecode(line[1:].rstrip(b"\r\n")).removesuffix(" (deleted)")
                    if candidate.startswith("/"):
                        path = Path(candidate)
                        if path in roots or not roots.isdisjoint(path.parents):
                            touches_scope = True
                            break
            if touches_scope:
                break

        comm = None if touches_scope else reader(pid)
        if touches_scope or comm is None or comm in BUILD_PROTECTED_COMMS:
            kept.extend(lines)
            continue

        kept.extend(header)
        for rec in records:
            if not _is_foreign_dead_record(rec):
                kept.extend(rec)
    return b"".join(kept)


def drop_session_helpers(stdout):
    """Mode de repli seulement : retirer les blocs des gardiens de session."""
    kept, skip = [], False
    for raw in stdout.splitlines(keepends=True):
        if raw.startswith(b"p"):
            pid = raw[1:].strip().decode(errors="replace")
            try:
                comm = Path(f"/proc/{pid}/comm").read_text().strip()
            except OSError:
                comm = ""
            skip = comm in SESSION_HELPERS
        if not skip:
            kept.append(raw)
    return b"".join(kept)


def busy_paths(scopes=(), budget=None, ignore_own=()):
    budget = budget or ObservationBudget()
    command = LSOF_OPEN_FILES
    before, fuse = None, ()
    diagnostics = []
    try:
        if any(not str(scope).isascii() or not str(scope).isprintable() or "\\" in str(scope) or "^" in str(scope) for scope in scopes):
            raise ValueError("escaped scope unavailable")
        if sys.platform.startswith("linux"):
            before, mounts = mount_snapshot()
            fuse = tuple(str(point) for point, layers in mounts.items()
                         if any(kind.startswith("fuse") for _, _, kind in layers))
            # Lecture seule avec une autorisation existante ; aucun mot de passe ni sudoers.
            command = ["/usr/bin/lsof", "-n", "-P", "-F", "pftDn"]
            if os.geteuid() != 0:
                command = ["sudo", "-n", "--", *command]
        result = observed_run(command, budget, timeout=30)
        if (result.returncode and command[0] == "sudo" and scopes
                and all(owner_private_scope(scope) for scope in scopes)):
            # Sans sudo : un dossier privé n'est accessible qu'à ce compte (et à
            # root). Les processus de ce compte sont entièrement lisibles sans
            # privilège ; hors de ce cas, la garde reste entière.
            command = ["/usr/bin/lsof", "-u", str(os.geteuid()), "-n", "-P", "-F", "pftDn"]
            result = observed_run(command, budget, timeout=30)
            result.stdout = drop_session_helpers(result.stdout)
            budget.owner_private_scope = True
        if result.returncode:
            budget.failures.add("process_access_unavailable")
            return None
        if result.stderr.strip():
            if before is None:
                raise ValueError("diagnostic without mount identity")
            exempt = foreign_mount_exemptions(result.stderr, scopes, mounts)
            result = observed_run([*command, *(arg for name in exempt for arg in ("-e", name))],
                                  budget, timeout=30)
            filtered_stdout = drop_foreign_unverifiable_records(result.stdout, scopes)
            if result.returncode or result.stderr.strip() or not process_output_complete(filtered_stdout, (*fuse, *exempt), diagnostics):
                if not diagnostics:
                    log(f"Conserver les caches : lecture lsof après exclusion non qualifiée (code={result.returncode}, diagnostics={len(result.stderr)} octets). Faire vérifier ce lecteur et ses diagnostics sur cette machine, sans retirer la garde.")
                raise ValueError("incomplete scoped observation")
            result.stdout = filtered_stdout
            budget.exempt_mounts = len(exempt)
        else:
            filtered_stdout = drop_foreign_unverifiable_records(result.stdout, scopes)
            if not process_output_complete(filtered_stdout, fuse, diagnostics):
                raise ValueError("incomplete process records")
            result.stdout = filtered_stdout
        aliases = []
        if before is not None and scopes:
            validate_mount_scopes(scopes, mounts, aliases)
        if before is not None and mount_snapshot()[0] != before:
            raise ValueError("mount inventory changed")
        if not budget.remaining():
            raise ValueError("observation expired")
        paths = parse_open_paths(result.stdout, ignore_own)
        budget.process_reader = "system_lsof" if before is None else "linux_system_lsof"
        if aliases:
            # Un fichier ouvert par un second montage du même contenu compte
            # sous son chemin canonique.
            translated = []
            for path in paths:
                for alias, canonical in aliases:
                    if path == alias or alias in path.parents:
                        path = canonical / path.relative_to(alias)
                        break
                translated.append(path)
            paths = translated
        if scopes:
            roots = {path.resolve() for path in scopes}
            paths = [path for path in paths if path in roots or not roots.isdisjoint(path.parents)]
        return paths
    except (OSError, ValueError, UnicodeError):
        if diagnostics:
            log("Conserver les caches : observation d'un processus incomplète " + json.dumps(diagnostics[0], ensure_ascii=False) + ". Faire vérifier ce processus et son descripteur avec son propriétaire ; ne pas arrêter un autre travail pour nettoyer. Les autres travaux autorisés suivent leur propre état.")
        budget.failures.add("process_observation_incomplete")
        return None


def git_ok(repo, *arguments, budget=None):
    budget = budget or ObservationBudget()
    budget.git_calls += 1
    return observed_run(["git", "-c", "core.fsmonitor=false", "-C", str(repo), *arguments],
                        budget, timeout=15, limit=1024 * 1024, env=git_ceiling_env(repo))


# Aligné sur MAX_WORKTREES dans crates/bridget-daemon/src/workspace_git.rs.
WORKTREE_LIMIT = 128
WORKTREE_WARNING_AT = 96
MOTIF_CANDIDATE = "candidat_a_examiner"
NOTE_CANDIDAT = ("Un candidat n'est pas une preuve que la tâche est close "
                 "et n'autorise pas la suppression.")
WORKTREE_RECOVERY = {'principal': 'Conservez le checkout principal.',
 'hors_perimetre': 'Cet espace ne compte pas dans le dossier indiqué ; examinez-le dans son propre '
                   'périmètre.',
 'locked': 'Identifiez le propriétaire du verrou avant toute action.',
 'nested': 'Examinez les espaces imbriqués et leurs usages avant leur parent.',
 'dirty_ignored': 'Préservez les modifications et les fichiers non suivis ou ignorés.',
 'non_integre': 'Intégrez ou archivez les commits avant tout retrait.',
 'recent': 'Conservez le travail récent ; le délai minimal est de 24 heures depuis le dernier '
           'commit.',
 'processus_actif': 'Attendez la fin du processus qui utilise ce dossier.',
 'lecture_inconnue': 'Rétablissez une observation complète puis relancez cet inventaire.',
 'candidat_a_examiner': 'Vérifiez tâches, fils, branches et fichiers avant un retrait Git sans '
                        'force.',
 'lien_symbolique': 'Conservez ce chemin symbolique ; vérifiez son emplacement réel.',
 'chemin_absent': 'Vérifiez le registre Git et le stockage ; ne recréez ni ne purgez '
                  'automatiquement.'}


NOTE_CAPACITE = ("scoped_count compte les inscriptions Git sous le dossier .worktrees. "
                 "Les réservations et dossiers retenus de l'admission Bridget ne sont pas observés ici : "
                 "ce nombre ne garantit ni égalité avec l'admission, ni capacité disponible.")


def integration_reason(repo, head, base_ref, budget=None):
    budget = budget or ObservationBudget()
    key = ("base", str(repo), base_ref)
    if key not in budget.facts:
        result = git_ok(repo, "rev-parse", "--verify", "--quiet", "--end-of-options", base_ref + "^{commit}", budget=budget)
        budget.facts[key] = result.stdout.strip().decode() if not result.returncode else None
    base = budget.facts[key]
    if not base or not budget.remaining():
        return "lecture_inconnue"
    key = ("ancestor", str(repo), head, base)
    if key not in budget.facts:
        budget.facts[key] = git_ok(repo, "merge-base", "--is-ancestor", "--", head, base, budget=budget).returncode
    code = budget.facts[key]
    return None if code == 0 else "non_integre" if code == 1 else "lecture_inconnue"


def age_reason(repo, head, now, budget=None):
    budget = budget or ObservationBudget()
    key = ("time", str(repo), head)
    if key not in budget.facts:
        result = git_ok(repo, "log", "-1", "--format=%ct", head, budget=budget)
        try:
            budget.facts[key] = int(result.stdout.strip()) if not result.returncode else None
        except ValueError:
            budget.facts[key] = None
    stamp = budget.facts[key]
    return "lecture_inconnue" if stamp is None else "recent" if now-stamp < WORKTREE_MIN_AGE else None


def dirty_reason(path, budget=None):
    status = git_ok(path, "--no-optional-locks", "status", "--porcelain",
                    "--untracked-files=normal", "--ignored", budget=budget)
    if status.returncode:
        return "lecture_inconnue"
    return "dirty_ignored" if status.stdout.strip() else None


def nested_worktrees(paths):
    known, nested = set(paths), set()
    for path in known:
        for parent in path.parents:
            if parent in known:
                nested.update((path, parent))
    return nested


def capacity_state(count):
    if count >= WORKTREE_LIMIT:
        return "limit"
    return "warning" if count >= WORKTREE_WARNING_AT else "normal"


def worktree_context(repo, scope=None, budget=None):
    entries = worktree_entries(repo, budget)
    base = ((scope if scope is not None else entries[0].path) / ".worktrees").resolve()
    return entries[0], base, [(entry, entry.path.resolve()) for entry in entries[1:]]


def worktree_reasons(repo, entry, real, others, base, base_ref, now, busy, launch, budget=None):
    """Motifs de blocage d'un worktree secondaire, produits à la demande.

    Seul moteur de décision : l'inventaire liste tous les motifs, le retrait
    s'arrête au premier."""
    budget = budget or ObservationBudget()
    if not budget.remaining():
        budget.failures.add("deadline")
        yield "lecture_inconnue"
        return
    if entry.locked:
        yield "locked"
    for part in (entry.path, *entry.path.parents):
        if part.is_symlink():
            yield "lien_symbolique"
            return
        if part.resolve() == base:
            break
    if not entry.path.is_dir():
        yield "chemin_absent"
        return
    if base not in real.parents:
        yield "hors_perimetre"
    if real in others:
        yield "nested"
    if entry.prunable or not entry.head:
        yield "lecture_inconnue"
    else:
        yield from (reason for reason in (integration_reason(repo, entry.head, base_ref, budget),
                                          age_reason(repo, entry.head, now, budget)) if reason)
    dirty = dirty_reason(real, budget)
    if dirty:
        yield dirty
    if busy is None:
        yield "lecture_inconnue"
    elif launch == real or real in launch.parents or real in busy:
        yield "processus_actif"


def inventory(repo, base_ref, now, launch, budget=None):
    """Observations en lecture seule ; RuntimeError si le registre Git est illisible."""
    budget = budget or ObservationBudget()
    principal, base, secondary = worktree_context(repo, repo.resolve(), budget)
    scopes = tuple(real for _, real in secondary if real.is_dir())
    busy = busy_paths(scopes, budget) if scopes else []
    busy = None if busy is None else {ancestor for opened in busy for ancestor in (opened, *opened.parents)}
    others = nested_worktrees(real for _, real in secondary)
    rows = [{"path": str(principal.path), "head": principal.head, "locked": principal.locked,
             "prunable": principal.prunable, "motifs": ["principal"]}]
    for entry, real in secondary:
        reasons = worktree_reasons(repo, entry, real, others, base, base_ref, now, busy, launch, budget)
        motifs = list(dict.fromkeys(reasons)) or [MOTIF_CANDIDATE]
        rows.append({"path": str(entry.path), "head": entry.head, "locked": entry.locked,
                     "prunable": entry.prunable, "motifs": motifs})
    for row in rows:
        row["recovery"] = [WORKTREE_RECOVERY[reason] for reason in row["motifs"]]
    scoped = sum(1 for entry, _ in secondary if base == entry.path or base in entry.path.parents)
    return {
        "version": 1,
        "repo": str(repo.resolve()),
        "base_ref": base_ref,
        "capacity": {"scope": str(base), "scoped_count": scoped,
                     "git_total": len(secondary) + 1, "limit": WORKTREE_LIMIT,
                     "warning_at": WORKTREE_WARNING_AT, "state": capacity_state(scoped),
                     "limite": NOTE_CAPACITE},
        "processus": "inconnu" if busy is None else "complet",
        "observation": {"seconds": round(time.monotonic()-budget.started, 3),
                        "budget_seconds": round(budget.deadline-budget.started, 3),
                        "capture_limit_bytes": MAX_OBSERVATION_BYTES,
                        "git_capture_limit_bytes": 1024 * 1024,
                        "entry_limit": MAX_WORKTREE_ENTRIES,
                        "git_calls": budget.git_calls, "reader": budget.process_reader,
                        "exempt_foreign_mounts": budget.exempt_mounts,
                        "incomplete": sorted(budget.failures)},
        "note": NOTE_CANDIDAT,
        "worktrees": rows,
    }


def report_worktrees(repo, base_ref, as_json):
    try:
        report = inventory(repo, base_ref, time.time(), Path.cwd().resolve())
    except (OSError, RuntimeError, ValueError) as error:
        if as_json:
            print(json.dumps({"version": 1, "erreur": "inventaire_git_indisponible"}))
        else:
            log(f"inventaire Git indisponible : {error}")
        return 1
    if as_json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
        return 0
    capacity = report["capacity"]
    print(f"Capacité : {capacity['scoped_count']}/{capacity['limit']} sous {capacity['scope']} "
          f"({capacity['state']}) ; registre Git : {capacity['git_total']} au total")
    print(capacity["limite"])
    for row in report["worktrees"]:
        print(f"{row['path']} : {', '.join(row['motifs'])}")
        print("  " + " ".join(row["recovery"]))
    print(report["note"])
    return 0


def prune_merged_worktrees(repo, dry_run=False, now=None, base_ref="main"):
    """Retire les worktrees de `.worktrees/` intégrés à base_ref, sans aucun motif de blocage.

    Même moteur que l'inventaire. `git worktree remove` sans --force refuse un worktree
    sale. La branche est conservée, donc rien n'est perdu."""
    now = time.time() if now is None else now
    budget = ObservationBudget()
    try:
        _, base, secondary = worktree_context(repo, budget=budget)
    except RuntimeError as error:
        log(f"worktrees fusionnés non retirés : {error}")
        return []
    if not secondary:
        return []
    busy = busy_paths(tuple(real for _, real in secondary if real.is_dir()), budget)
    if busy is None:
        log("worktrees fusionnés non retirés : fichiers ouverts inconnus")
        return []
    busy = {ancestor for opened in busy for ancestor in (opened, *opened.parents)}
    launch = Path.cwd().resolve()
    others = nested_worktrees(real for _, real in secondary)
    removed = []
    for entry, real in secondary:
        reasons = worktree_reasons(repo, entry, real, others, base, base_ref, now, busy, launch, budget)
        if next(reasons, None):
            continue
        if dry_run:
            removed.append(real)
        elif not git_ok(repo, "worktree", "remove", str(real), budget=budget).returncode:
            removed.append(real)
    if removed:
        verb = "retirables" if dry_run else "retirés"
        log(f"worktrees fusionnés {verb} : " + ", ".join(p.name for p in removed))
    return removed


def maintain_build_caches(repo, arguments):
    """Entretenir les caches sans changer le résultat de Cargo."""
    errors = []
    try:
        register_staging(repo, arguments)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        log(f"staging non enregistré : {error}")
        errors.append(f"register_staging: {error}")
    try:
        prune(repo, *limits())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"entretien des caches différé : {error}")
        errors.append(f"prune: {error}")
    return errors


# ---------------------------------------------------------------------------
# Spécification 312 (Lot 1a) — Déport des compilations release hors du poste
# ---------------------------------------------------------------------------


def local_glibc_version():
    """Retourne (major, minor) de la glibc locale sous Linux, ou None."""
    if not sys.platform.startswith("linux"):
        return None
    try:
        raw = os.confstr("CS_GNU_LIBC_VERSION")
        if raw:
            match = re.search(r"(\d+)\.(\d+)", raw)
            if match:
                return (int(match.group(1)), int(match.group(2)))
    except (AttributeError, OSError, ValueError):
        pass
    lib, ver = platform.libc_ver()
    if lib == "glibc" and ver:
        match = re.search(r"(\d+)\.(\d+)", ver)
        if match:
            return (int(match.group(1)), int(match.group(2)))
    return None


def detect_local_target():
    """Identifie la cible canonique native de la machine locale."""
    machine = platform.machine().lower()
    if sys.platform == "darwin" and machine in ("arm64", "aarch64"):
        return "macos-arm64"
    if sys.platform.startswith("linux") and machine in ("x86_64", "amd64"):
        return "linux-x86_64-glibc2.36"
    return None


def check_local_target_compatible(target, host_platform=None, host_machine=None, host_glibc=None):
    """Garde de compilation locale (§3.3.2, §7.12) : même OS, même arch, glibc <= cible."""
    plat = sys.platform if host_platform is None else host_platform
    arch = (platform.machine() if host_machine is None else host_machine).lower()
    if target not in KNOWN_HEAVY_TARGETS:
        return False, (
            f"cible inconnue « {target} » (cibles reconnues : {', '.join(KNOWN_HEAVY_TARGETS)}). "
            "Spécifiez une cible valide ou utilisez --host <machine>."
        )
    if target == "macos-arm64":
        if plat == "darwin" and arch in ("arm64", "aarch64"):
            return True, ""
        return False, (
            f"compilation locale impossible pour {target} sur cette machine ({plat}/{arch}). "
            "Relancez sans --local ou avec --host <machine-macos>."
        )
    if target == "linux-x86_64-glibc2.36":
        if not plat.startswith("linux") or arch not in ("x86_64", "amd64"):
            return False, (
                f"compilation locale impossible pour {target} sur cette machine ({plat}/{arch}). "
                "Relancez sans --local ou avec --host <machine-linux-glibc2.36>."
            )
        glibc = local_glibc_version() if host_glibc is None else host_glibc
        if glibc is None:
            return False, (
                f"version glibc locale indéterminée pour {target}. "
                "Relancez avec --host <machine-linux-glibc2.36>."
            )
        if glibc > (2, 36):
            return False, (
                f"compilation locale refusée pour {target} : glibc locale {glibc[0]}.{glibc[1]} > 2.36. "
                "Relancez sans --local ou avec --host <machine-linux-glibc2.36>."
            )
        return True, ""
    return False, f"cible non prise en charge en local : {target}"


def discrete_cargo_command(cargo_bin, cargo_args, env):
    """Prépare la commande Cargo en mode discret (§3.3.2)."""
    run_env = dict(env)
    cpu_count = os.cpu_count() or 4
    jobs = max(1, min(2, cpu_count // 4))
    run_env["CARGO_BUILD_JOBS"] = str(jobs)
    prefix = []
    if sys.platform == "darwin":
        if shutil.which("taskpolicy"):
            prefix = ["taskpolicy", "-b"]
    elif sys.platform.startswith("linux"):
        if shutil.which("nice"):
            prefix.extend(["nice", "-n", "15"])
        if shutil.which("ionice"):
            prefix.extend(["ionice", "-c", "3"])
    return [*prefix, cargo_bin, *cargo_args], run_env


def is_release_build_command(cargo_args):
    """Périmètre Lot 1a : seul `cargo build --release` est déporté."""
    if not cargo_args or cargo_args[0] != "build":
        return False
    for index, arg in enumerate(cargo_args[1:], start=1):
        if arg in ("--release", "-r", "--profile=release"):
            return True
        if arg == "--profile" and index + 1 < len(cargo_args) and cargo_args[index + 1] == "release":
            return True
    return False


class CargoWrapperOptions:
    def __init__(self):
        self.local = False
        self.host = None
        self.build_target = None
        self.ssh_host = None
        self.ssh_user = None
        self.ssh_port = None
        self.ssh_key = None
        self.ssh_known_hosts = None


def parse_cargo_wrapper_options(raw_args):
    """Extrait les options Bridget placées avant la sous-commande Cargo (build/test/check/clippy)."""
    opts = CargoWrapperOptions()
    index = 0
    while index < len(raw_args):
        arg = raw_args[index]
        if arg in ("build", "test", "check", "clippy"):
            return opts, raw_args[index:]
        if arg == "--local":
            opts.local = True
            index += 1
        elif arg == "--host" and index + 1 < len(raw_args):
            opts.host = raw_args[index + 1]
            index += 2
        elif arg.startswith("--host="):
            opts.host = arg.split("=", 1)[1]
            index += 1
        elif arg == "--build-target" and index + 1 < len(raw_args):
            opts.build_target = raw_args[index + 1]
            index += 2
        elif arg.startswith("--build-target="):
            opts.build_target = arg.split("=", 1)[1]
            index += 1
        elif arg == "--ssh-host" and index + 1 < len(raw_args):
            opts.ssh_host = raw_args[index + 1]
            index += 2
        elif arg == "--ssh-user" and index + 1 < len(raw_args):
            opts.ssh_user = raw_args[index + 1]
            index += 2
        elif arg == "--ssh-port" and index + 1 < len(raw_args):
            opts.ssh_port = int(raw_args[index + 1])
            index += 2
        elif arg == "--ssh-key" and index + 1 < len(raw_args):
            opts.ssh_key = raw_args[index + 1]
            index += 2
        elif arg == "--ssh-known-hosts" and index + 1 < len(raw_args):
            opts.ssh_known_hosts = raw_args[index + 1]
            index += 2
        else:
            return opts, raw_args[index:]
    return opts, []


def validate_relative_home_file(rel_path, home=None, label="fichier SSH"):
    """Vérifie un chemin relatif à $HOME : pas de .., pas de lien symbolique, régulier, 0600, UID courant."""
    if not isinstance(rel_path, str) or not rel_path or rel_path.startswith("/") or rel_path.startswith("~"):
        raise ValueError(f"{label} doit être un chemin relatif à $HOME : {rel_path!r}")
    if any(ord(ch) < 0x20 or ch == "\x7f" for ch in rel_path):
        raise ValueError(f"{label} contient un caractère de contrôle interdit")
    rel = Path(rel_path)
    if ".." in rel.parts or not rel.parts:
        raise ValueError(f"{label} contient une traversée interdite : {rel_path!r}")
    base = Path.home().resolve() if home is None else Path(home).resolve()
    current = base
    for part in rel.parts:
        current = current / part
        if current.is_symlink():
            raise ValueError(f"{label} passe par un lien symbolique interdit : {current}")
    try:
        info = os.lstat(current)
    except OSError as error:
        raise ValueError(f"{label} introuvable ({current}) : {error}") from error
    if not stat.S_ISREG(info.st_mode):
        raise ValueError(f"{label} n'est pas un fichier régulier : {current}")
    if info.st_uid != os.getuid():
        raise ValueError(f"{label} n'appartient pas au compte courant : {current}")
    if info.st_mode & 0o077:
        raise ValueError(
            f"{label} doit avoir des droits stricts 0600 ({oct(info.st_mode & 0o777)}) : {current}. "
            f"Exécutez : chmod 600 {current}"
        )
    return current


def ssh_command_prefix(ssh_profile, home=None):
    """Construit la commande SSH stricte (§3.1, §3.2.4) sans transfert d'agent ni config globale."""
    host = ssh_profile.get("host", "")
    user = ssh_profile.get("user", "")
    port = int(ssh_profile.get("port", 22))
    if not host or host.startswith("-") or any(c.isspace() for c in host):
        raise ValueError(f"hôte SSH invalide : {host!r}")
    if not user or user.startswith("-") or any(c.isspace() for c in user):
        raise ValueError(f"utilisateur SSH invalide : {user!r}")
    if not (1 <= port <= 65535):
        raise ValueError(f"port SSH invalide : {port}")
    identity = validate_relative_home_file(ssh_profile.get("identity_rel", ""), home=home, label="clé SSH")
    known_hosts = validate_relative_home_file(
        ssh_profile.get("known_hosts_rel", ""), home=home, label="known_hosts SSH"
    )
    return [
        "ssh",
        "-F", "/dev/null",
        "-o", "BatchMode=yes",
        "-o", "StrictHostKeyChecking=yes",
        "-o", "IdentitiesOnly=yes",
        "-o", "ForwardAgent=no",
        "-o", "ServerAliveInterval=15",
        "-o", "ServerAliveCountMax=3",
        "-o", "ConnectTimeout=10",
        "-p", str(port),
        "-i", str(identity),
        "-o", f"UserKnownHostsFile={known_hosts}",
        f"{user}@{host}",
    ]


def validate_git_source_for_offload(repo):
    """Vérifie un commit Git exact, propre et déjà ancêtre de la branche distante suivie (§3.2.4)."""
    env = git_ceiling_env(repo)
    head = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--verify", "HEAD"],
        text=True, capture_output=True, check=False, timeout=10, env=env,
    )
    if head.returncode != 0:
        raise ValueError("dépôt Git ou commit HEAD introuvable ; relancez avec --local")
    head_sha = head.stdout.strip().lower()
    if not VALID_OWNER_SHA_RE.fullmatch(head_sha):
        raise ValueError(f"SHA HEAD invalide : {head_sha!r}")

    diff_work = subprocess.run(
        ["git", "-C", str(repo), "diff", "--quiet", "HEAD"],
        capture_output=True, check=False, timeout=10, env=env,
    )
    diff_idx = subprocess.run(
        ["git", "-C", str(repo), "diff", "--cached", "--quiet"],
        capture_output=True, check=False, timeout=10, env=env,
    )
    if diff_work.returncode != 0 or diff_idx.returncode != 0:
        raise ValueError(
            "arbre de travail ou index modifié : le déport refuse d'archiver un état non commité. "
            "Commitez et poussez vos changements, ou relancez explicitement avec --local."
        )

    # Vérifier que HEAD est déjà ancêtre de la branche distante suivie (sans fetch réseau).
    remote_refs = []
    upstream = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"],
        text=True, capture_output=True, check=False, timeout=10, env=env,
    )
    if upstream.returncode == 0 and upstream.stdout.strip():
        remote_refs.append(upstream.stdout.strip())
    for fallback_ref in ("origin/main", "github/main"):
        check_ref = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--verify", "--quiet", f"{fallback_ref}^{{commit}}"],
            capture_output=True, check=False, timeout=10, env=env,
        )
        if check_ref.returncode == 0 and fallback_ref not in remote_refs:
            remote_refs.append(fallback_ref)

    pushed_ref = None
    for ref in remote_refs:
        anc = subprocess.run(
            ["git", "-C", str(repo), "merge-base", "--is-ancestor", head_sha, ref],
            capture_output=True, check=False, timeout=10, env=env,
        )
        if anc.returncode == 0:
            pushed_ref = ref
            break
    if pushed_ref is None:
        checked = ", ".join(remote_refs) if remote_refs else "aucune référence distante locale"
        raise ValueError(
            f"le commit {head_sha[:12]} n'est pas encore ancêtre de la branche distante ({checked}). "
            "Poussez le commit d'abord, ou relancez explicitement avec --local."
        )

    untracked_res = subprocess.run(
        ["git", "-C", str(repo), "ls-files", "--others", "--exclude-standard"],
        text=True, capture_output=True, check=False, timeout=10, env=env,
    )
    untracked = [line for line in untracked_res.stdout.splitlines() if line.strip()] if untracked_res.returncode == 0 else []

    root_sha = canonical_owner_id(repo)
    if not root_sha:
        raise ValueError("commit racine Git introuvable pour BRIDGET_BUILD_OWNER")

    return {
        "head_sha": head_sha,
        "sha12": head_sha[:12],
        "root_sha": root_sha,
        "pushed_ref": pushed_ref,
        "untracked": untracked,
    }


def create_git_archive(repo, head_sha, dest_tar):
    """Crée une archive tar reproductible d'un commit exact et calcule son SHA-256."""
    env = git_ceiling_env(repo)
    res = subprocess.run(
        ["git", "-C", str(repo), "archive", "--format=tar", "-o", str(dest_tar), head_sha],
        capture_output=True, text=True, check=False, timeout=120, env=env,
    )
    if res.returncode != 0:
        raise RuntimeError(f"git archive a échoué : {res.stderr.strip()}")
    return dest_tar, digest(Path(dest_tar)).hex()


def read_pinned_rust_channel(repo):
    """Lit `channel = "..."` dans rust-toolchain.toml si présent."""
    toolchain = Path(repo) / "rust-toolchain.toml"
    if not toolchain.is_file():
        return None
    text = toolchain.read_text(encoding="utf-8", errors="replace")
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1).strip() if match else None


def find_bridget_bin():
    explicit = os.environ.get("BRIDGET_BIN")
    if explicit:
        return explicit if Path(explicit).is_file() else None
    found = shutil.which("bridget")
    if found:
        return found
    local_bin = Path.home() / ".local/bin/bridget"
    return str(local_bin) if local_bin.is_file() else None


def call_bridget_heavy_work(subcommand, extra_args=()):
    """Appelle `bridget heavy-work <subcommand> ... --json` ; renvoie (ok, payload_ou_erreur)."""
    bridget_bin = find_bridget_bin()
    if not bridget_bin:
        return False, "bridget_cli_unavailable"
    cmd = [bridget_bin, "heavy-work", subcommand, *extra_args, "--json"]
    try:
        res = subprocess.run(cmd, capture_output=True, text=True, check=False, timeout=15)
    except (OSError, subprocess.SubprocessError) as error:
        return False, f"bridget_cli_error:{error}"
    if res.returncode != 0:
        err = (res.stderr or res.stdout or "").strip()
        first_line = next((line.strip() for line in err.splitlines() if line.strip()), "")
        if len(first_line) > 160:
            first_line = first_line[:157] + "..."
        return False, first_line or f"bridget_cli_exit_{res.returncode}"
    try:
        return True, json.loads(res.stdout)
    except json.JSONDecodeError as error:
        return False, f"bridget_cli_invalid_json:{error}"


REMOTE_PREFLIGHT_SCRIPT = r"""
import json, os, platform, re, shutil, stat, subprocess, sys
from pathlib import Path

target = sys.argv[1]
expected_rust = sys.argv[2]
repo_slug = sys.argv[3]

plat = sys.platform
arch = platform.machine().lower()
glibc = None
if plat.startswith("linux"):
    try:
        raw = os.confstr("CS_GNU_LIBC_VERSION")
        m = re.search(r"(\d+)\.(\d+)", raw or "")
        if m:
            glibc = [int(m.group(1)), int(m.group(2))]
    except Exception:
        pass
    if glibc is None:
        lib, ver = platform.libc_ver()
        if lib == "glibc" and ver:
            m = re.search(r"(\d+)\.(\d+)", ver)
            if m:
                glibc = [int(m.group(1)), int(m.group(2))]

if target == "macos-arm64":
    if not (plat == "darwin" and arch in ("arm64", "aarch64")):
        print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"hôte {plat}/{arch} incompatible avec {target}"}))
        sys.exit(0)
elif target == "linux-x86_64-glibc2.36":
    if not (plat.startswith("linux") and arch in ("x86_64", "amd64")):
        print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"hôte {plat}/{arch} incompatible avec {target}"}))
        sys.exit(0)
    if glibc is None or tuple(glibc) > (2, 36):
        print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"glibc distante {glibc} > 2.36 pour {target}"}))
        sys.exit(0)
else:
    print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"cible inconnue {target}"}))
    sys.exit(0)

env = dict(os.environ)
cargo_bin_dir = str(Path.home() / ".cargo/bin")
env["PATH"] = cargo_bin_dir + os.pathsep + env.get("PATH", "")
rustc = shutil.which("rustc", path=env["PATH"])
cargo = shutil.which("cargo", path=env["PATH"])
if not rustc or not cargo:
    print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": "rustc ou cargo introuvable sur la machine distante"}))
    sys.exit(0)
ver_out = subprocess.run([rustc, "--version"], capture_output=True, text=True, env=env, timeout=10)
if ver_out.returncode != 0:
    print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": "rustc --version en échec"}))
    sys.exit(0)
rustc_ver = ver_out.stdout.strip()
if expected_rust and expected_rust not in rustc_ver:
    # Si rustup gère la toolchain épinglée, vérifier si elle est installée.
    rustup = shutil.which("rustup", path=env["PATH"])
    ok_toolchain = False
    if rustup:
        tl = subprocess.run([rustup, "toolchain", "list"], capture_output=True, text=True, env=env, timeout=10)
        if tl.returncode == 0 and any(line.startswith(expected_rust) for line in tl.stdout.splitlines()):
            ok_toolchain = True
    if not ok_toolchain:
        print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"toolchain Rust {expected_rust} absente (observé : {rustc_ver})"}))
        sys.exit(0)

# Résolution stricte de la racine de cache distante (§3.2.4) : jamais de repli si build.json invalide.
home = Path.home().resolve()
settings_path = home / ".config/bridget/build.json"
def read_owned_json(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, encoding="utf-8") as st:
        info = os.fstat(st.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_size > 4096 or (info.st_mode & 0o077):
            raise ValueError("configuration non privée ou propriétaire invalide")
        return json.loads(st.read(4097))

if settings_path.exists() or settings_path.is_symlink():
    try:
        cfg = read_owned_json(settings_path)
        if (not isinstance(cfg, dict) or set(cfg) != {"version", "root", "root_id"}
                or type(cfg["version"]) is not int or cfg["version"] != 1
                or not isinstance(cfg["root"], str) or not isinstance(cfg["root_id"], str)
                or not re.fullmatch(r"[0-9a-f]{32}", cfg["root_id"])):
            raise ValueError("format build.json invalide")
        root = Path(cfg["root"])
        if not root.is_dir() or root.is_symlink() or root.absolute() != root.resolve() or not root.is_absolute():
            raise ValueError("racine build.json absente ou indirecte")
        st = root.stat()
        if st.st_uid != os.getuid() or (st.st_mode & 0o077) or st.st_dev == Path("/").stat().st_dev:
            raise ValueError("racine build.json non privée ou sur disque système")
        if read_owned_json(root / ".bridget-build-root.json") != {"version": 1, "id": cfg["root_id"]}:
            raise ValueError("ancre de volume différente")
        cache_root = root
    except Exception as error:
        print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"build.json distant invalide sans repli : {error}"}))
        sys.exit(0)
else:
    cache_root = home / ".cache"
    cache_root.mkdir(mode=0o700, exist_ok=True)

usage = shutil.disk_usage(cache_root)
if usage.free < 2 * 1024 ** 3:
    print(json.dumps({"ok": False, "failure_kind": "prerequisite_missing", "reason": f"espace libre insuffisant sous {cache_root} ({usage.free / (1024**3):.2f} Gio < 2 Gio)"}))
    sys.exit(0)

safe_slug = re.sub(r"[^A-Za-z0-9._-]+", "-", repo_slug).strip("-") or "repo"
safe_target = re.sub(r"[^A-Za-z0-9._-]+", "-", target).strip("-") or "target"
cache_dir = cache_root / f"bridget-build-{safe_slug}-{safe_target}"
print(json.dumps({
    "ok": True,
    "os": plat,
    "arch": arch,
    "glibc": glibc,
    "rustc": rustc_ver,
    "cache_root": str(cache_root),
    "cache_dir": str(cache_dir),
    "free_bytes": usage.free,
}))
"""


REMOTE_GUARDIAN_SCRIPT = r"""#!/usr/bin/env python3
import fcntl
import hashlib
import json
import os
import platform
import selectors
import shutil
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

RESULT_PREFIX = "BRIDGET_HEAVY_WORK_RESULT:"


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as stream:
        while True:
            chunk = stream.read(65536)
            if not chunk:
                break
            h.update(chunk)
    return h.hexdigest()


def kill_process_group(proc, grace_secs):
    if proc is None or proc.poll() is not None:
        return
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    deadline = time.monotonic() + grace_secs
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            return
        time.sleep(0.1)
    if proc.poll() is None:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def main():
    cfg_path = Path(sys.argv[1])
    cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
    work_dir = Path(cfg["work_dir"])
    archive_path = Path(cfg["archive_path"])
    cache_dir = Path(cfg["cache_dir"])
    expected_archive_sha = cfg["archive_sha256"]
    owner_sha = cfg["owner_sha"]
    build_id = cfg["build_id"]
    target = cfg["target"]
    cargo_args = cfg["cargo_args"]
    heartbeat_timeout = float(cfg.get("heartbeat_timeout_secs", 30.0))
    kill_grace = float(cfg.get("kill_grace_secs", 10.0))
    custom_command = cfg.get("custom_command")

    src_dir = work_dir / "src"
    phase_times = {}
    aborted_reason = [None]
    active_proc = [None]
    stop_heartbeat = threading.Event()
    proc_lock = threading.Lock()

    def heartbeat_watcher():
        last_beat = time.monotonic()
        sel = selectors.DefaultSelector()
        try:
            sel.register(sys.stdin.fileno(), selectors.EVENT_READ)
            while not stop_heartbeat.is_set():
                events = sel.select(timeout=0.5)
                if events:
                    data = os.read(sys.stdin.fileno(), 4096)
                    if not data:
                        aborted_reason[0] = "stdin_eof"
                        with proc_lock:
                            kill_process_group(active_proc[0], kill_grace)
                        return
                    last_beat = time.monotonic()
                elif time.monotonic() - last_beat >= heartbeat_timeout:
                    aborted_reason[0] = "heartbeat_timeout"
                    with proc_lock:
                        kill_process_group(active_proc[0], kill_grace)
                    return
        except OSError:
            aborted_reason[0] = "stdin_error"
            with proc_lock:
                kill_process_group(active_proc[0], kill_grace)
        finally:
            try:
                sel.close()
            except Exception:
                pass

    hb_thread = threading.Thread(target=heartbeat_watcher, daemon=True)
    hb_thread.start()

    result = {
        "ok": False,
        "error": None,
        "binary_path": None,
        "binary_sha256": None,
        "build_info": None,
        "phases": phase_times,
        "maintenance_warnings": [],
    }
    try:
        t0 = time.monotonic()
        if not custom_command:
            actual_archive_sha = sha256_file(archive_path)
            if actual_archive_sha != expected_archive_sha:
                raise RuntimeError(
                    f"SHA-256 archive invalide : attendu {expected_archive_sha}, reçu {actual_archive_sha}"
                )
            src_dir.mkdir(mode=0o700, exist_ok=True)
            tar_res = subprocess.run(
                ["tar", "--no-same-owner", "-xf", str(archive_path), "-C", str(src_dir)],
                capture_output=True, text=True, check=False,
            )
            if tar_res.returncode != 0:
                raise RuntimeError(f"extraction archive en échec : {tar_res.stderr.strip()}")
            phase_times["extract_s"] = round(time.monotonic() - t0, 2)

        cache_dir.parent.mkdir(mode=0o700, exist_ok=True)
        cache_dir.mkdir(mode=0o700, exist_ok=True)
        owner_marker = cache_dir / ".bridget-build-owner.json"
        if not owner_marker.exists() and not owner_marker.is_symlink():
            try:
                with owner_marker.open("x", encoding="utf-8") as stream:
                    json.dump({"version": 1, "repo": owner_sha}, stream)
            except FileExistsError:
                pass

        lock_path = cache_dir / ".heavy-work.lock"
        lock_fd = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
        with os.fdopen(lock_fd, "r+") as lock_file:
            t_lock = time.monotonic()
            fcntl.flock(lock_file, fcntl.LOCK_EX)
            phase_times["lock_wait_s"] = round(time.monotonic() - t_lock, 2)

            if aborted_reason[0]:
                raise RuntimeError(f"interrompu avant lancement ({aborted_reason[0]})")

            env = dict(os.environ)
            cargo_bin_dir = str(Path.home() / ".cargo/bin")
            env["PATH"] = cargo_bin_dir + os.pathsep + env.get("PATH", "")
            env["CARGO_TARGET_DIR"] = str(cache_dir)
            env["BRIDGET_BUILD_ID"] = build_id
            env["BRIDGET_BUILD_OWNER"] = owner_sha
            if sys.platform == "darwin":
                env.setdefault("MACOSX_DEPLOYMENT_TARGET", "11.0")

            if custom_command:
                cmd = list(custom_command)
                run_cwd = str(work_dir)
            else:
                cargo_bin = shutil.which("cargo", path=env["PATH"]) or "cargo"
                cmd = [cargo_bin, *cargo_args]
                if sys.platform == "darwin" and Path("/usr/bin/caffeinate").is_file():
                    cmd = ["/usr/bin/caffeinate", "-i", *cmd]
                run_cwd = str(src_dir)

            t_compile = time.monotonic()
            with proc_lock:
                if aborted_reason[0]:
                    raise RuntimeError(f"interrompu ({aborted_reason[0]})")
                proc = subprocess.Popen(
                    cmd,
                    cwd=run_cwd,
                    env=env,
                    stdin=subprocess.DEVNULL,
                    stdout=sys.stderr,
                    stderr=sys.stderr,
                    start_new_session=True,
                )
                active_proc[0] = proc
            rc = proc.wait()
            with proc_lock:
                active_proc[0] = None
            phase_times["compile_s"] = round(time.monotonic() - t_compile, 2)
            if aborted_reason[0]:
                raise RuntimeError(f"compilation arrêtée par le gardien ({aborted_reason[0]})")
            if rc != 0:
                raise RuntimeError(f"cargo a échoué avec le code {rc}")

            if custom_command:
                result["ok"] = True
                return

            t_check = time.monotonic()
            binary_path = cache_dir / "release/bridget"
            if not binary_path.is_file() or binary_path.is_symlink():
                raise RuntimeError(f"binaire attendu introuvable : {binary_path}")

            if target.startswith("linux-"):
                abi_script = src_dir / "scripts/check-linux-abi.py"
                abi_res = subprocess.run(
                    [sys.executable, str(abi_script), str(binary_path)],
                    capture_output=True, text=True, check=False, env=env,
                )
                if abi_res.returncode != 0:
                    raise RuntimeError(f"contrôle ABI Linux refusé : {abi_res.stderr.strip() or abi_res.stdout.strip()}")
            elif target.startswith("macos-"):
                cs_res = subprocess.run(
                    ["codesign", "--verify", "--strict", str(binary_path)],
                    capture_output=True, text=True, check=False, env=env,
                )
                if cs_res.returncode != 0:
                    raise RuntimeError(f"codesign --verify --strict refusé : {cs_res.stderr.strip() or cs_res.stdout.strip()}")

            bi_res = subprocess.run(
                [str(binary_path), "build-info", "--json"],
                capture_output=True, text=True, check=False, env=env, timeout=15,
            )
            if bi_res.returncode != 0:
                raise RuntimeError(f"bridget build-info --json en échec : {bi_res.stderr.strip()}")
            bi = json.loads(bi_res.stdout)
            if bi.get("build_id") != build_id:
                raise RuntimeError(f"build_id inattendu : attendu {build_id}, reçu {bi.get('build_id')}")
            expected_os = "macos" if target.startswith("macos-") else "linux"
            if bi.get("os") != expected_os:
                raise RuntimeError(f"OS binaire inattendu : attendu {expected_os}, reçu {bi.get('os')}")

            binary_sha = sha256_file(binary_path)
            phase_times["verify_s"] = round(time.monotonic() - t_check, 2)

            # Entretien des caches distant non bloquant (§4.5).
            try:
                clean_res = subprocess.run(
                    [sys.executable, str(src_dir / "scripts/build.py"), "clean"],
                    cwd=str(src_dir),
                    env=env,
                    capture_output=True,
                    text=True,
                    check=False,
                    timeout=60,
                )
                if clean_res.returncode != 0:
                    result["maintenance_warnings"].append(
                        f"entretien distant code {clean_res.returncode}: {clean_res.stderr.strip()[:200]}"
                    )
            except Exception as maint_err:
                result["maintenance_warnings"].append(f"entretien distant : {maint_err}")

            result["ok"] = True
            result["binary_path"] = str(binary_path)
            result["binary_sha256"] = binary_sha
            result["build_info"] = bi
    except Exception as error:
        result["error"] = str(error)
    finally:
        stop_heartbeat.set()
        # Ne supprimer que le dossier source temporaire et l'archive (jamais le cache ni .heavy-work.lock).
        if not custom_command:
            shutil.rmtree(src_dir, ignore_errors=True)
            try:
                if archive_path.exists():
                    archive_path.unlink()
            except OSError:
                pass
        print(RESULT_PREFIX + json.dumps(result, ensure_ascii=False), flush=True)


if __name__ == "__main__":
    main()
"""


def run_remote_preflight(ssh_profile, target, expected_rust_channel, repo_slug, home=None):
    """Pré-vérification SSH rapide avant transfert (§3.2.3.5)."""
    ssh_prefix = ssh_command_prefix(ssh_profile, home=home)
    remote_cmd = (
        f"python3 -c {shlex.quote(REMOTE_PREFLIGHT_SCRIPT)} "
        f"{shlex.quote(target)} {shlex.quote(expected_rust_channel or '')} {shlex.quote(repo_slug)}"
    )
    try:
        res = subprocess.run(
            [*ssh_prefix, remote_cmd],
            capture_output=True,
            text=True,
            check=False,
            timeout=20,
        )
    except subprocess.TimeoutExpired:
        return {"ok": False, "failure_kind": "unreachable", "reason": "délai dépassé à la pré-vérification SSH"}
    except OSError as error:
        return {"ok": False, "failure_kind": "unreachable", "reason": f"échec SSH : {error}"}
    if res.returncode != 0:
        stderr = res.stderr.strip()
        return {
            "ok": False,
            "failure_kind": "unreachable",
            "reason": f"machine injoignable (SSH code {res.returncode}) : {stderr[:200]}",
        }
    try:
        return json.loads(res.stdout.strip().splitlines()[-1])
    except (IndexError, json.JSONDecodeError) as error:
        return {
            "ok": False,
            "failure_kind": "prerequisite_missing",
            "reason": f"sortie de pré-vérification illisible ({error})",
        }


def resolve_local_output_binary_path(repo, cargo_args, target, native_target):
    """Emplacement local du binaire rapatrié (§3.2.4, §7.12)."""
    target_dir = os.environ.get("CARGO_TARGET_DIR")
    for index, argument in enumerate(cargo_args):
        if argument.startswith("--target-dir="):
            target_dir = argument.split("=", 1)[1]
        elif argument == "--target-dir" and index + 1 < len(cargo_args):
            target_dir = cargo_args[index + 1]
    base = Path(os.path.abspath(repo / target_dir)) if target_dir else (repo / "target")
    if target == native_target:
        return base / "release" / "bridget", True
    return base / target / "release" / "bridget", False


def _lease_renewer_loop(lease_id, stop_event):
    while not stop_event.wait(HEAVY_WORK_RENEW_INTERVAL_SECS):
        ok, info = call_bridget_heavy_work("renew", ["--lease", lease_id])
        if not ok:
            log(f"avertissement : renouvellement du bail {lease_id} échoué ({info})")


def _release_lease_safely(lease_id):
    if not lease_id:
        return
    ok, info = call_bridget_heavy_work("release", ["--lease", lease_id])
    if not ok:
        log(f"avertissement : libération du bail {lease_id} non confirmée ({info})")


def execute_remote_build(
    repo,
    cargo_args,
    target,
    native_target,
    machine_id,
    ssh_profile,
    preflight,
    git_source,
    decision_reason,
    lease_id=None,
    home=None,
):
    """Exécute la compilation release sur la machine distante sous gardien et rapatriement vérifié."""
    ssh_prefix = ssh_command_prefix(ssh_profile, home=home)
    stop_renew = threading.Event()
    renew_thread = None
    if lease_id:
        renew_thread = threading.Thread(
            target=_lease_renewer_loop, args=(lease_id, stop_renew), daemon=True
        )
        renew_thread.start()

    remote_work_dir = None
    phases = {}
    try:
        with tempfile.TemporaryDirectory(prefix="bridget-heavy-local-") as tmp_str:
            tmp_dir = Path(tmp_str)
            archive_local = tmp_dir / "source.tar"

            t_archive = time.monotonic()
            _, archive_sha = create_git_archive(repo, git_source["head_sha"], archive_local)
            phases["archive_s"] = round(time.monotonic() - t_archive, 2)

            # Créer un dossier temporaire privé 0700 sur l'hôte distant.
            mk_res = subprocess.run(
                [*ssh_prefix, "umask 077 && mktemp -d /tmp/bridget-heavy-work-XXXXXX"],
                capture_output=True, text=True, check=False, timeout=20,
            )
            if mk_res.returncode != 0 or not mk_res.stdout.strip():
                raise RuntimeError(f"impossible de créer le dossier temporaire distant : {mk_res.stderr.strip()}")
            remote_work_dir = mk_res.stdout.strip().splitlines()[-1]
            remote_archive = f"{remote_work_dir}/source.tar"
            remote_guardian = f"{remote_work_dir}/guardian.py"
            remote_cfg = f"{remote_work_dir}/config.json"

            cfg_payload = json.dumps({
                "work_dir": remote_work_dir,
                "archive_path": remote_archive,
                "cache_dir": preflight["cache_dir"],
                "archive_sha256": archive_sha,
                "owner_sha": git_source["root_sha"],
                "build_id": git_source["sha12"],
                "target": target,
                "cargo_args": cargo_args,
                "heartbeat_timeout_secs": HEAVY_WORK_HEARTBEAT_TIMEOUT_SECS,
                "kill_grace_secs": HEAVY_WORK_KILL_GRACE_SECS,
            }).encode("utf-8")

            t_upload = time.monotonic()
            with archive_local.open("rb") as archive_stream:
                up_arch = subprocess.run(
                    [*ssh_prefix, f"umask 077 && cat > {shlex.quote(remote_archive)}"],
                    stdin=archive_stream, capture_output=True, check=False, timeout=300,
                )
            if up_arch.returncode != 0:
                raise RuntimeError(f"envoi de l'archive en échec : {up_arch.stderr.decode(errors='replace').strip()}")

            up_guard = subprocess.run(
                [*ssh_prefix, f"umask 077 && cat > {shlex.quote(remote_guardian)} && chmod 700 {shlex.quote(remote_guardian)}"],
                input=REMOTE_GUARDIAN_SCRIPT.encode("utf-8"),
                capture_output=True, check=False, timeout=30,
            )
            if up_guard.returncode != 0:
                raise RuntimeError(f"dépôt du gardien distant en échec : {up_guard.stderr.decode(errors='replace').strip()}")

            up_cfg = subprocess.run(
                [*ssh_prefix, f"umask 077 && cat > {shlex.quote(remote_cfg)}"],
                input=cfg_payload, capture_output=True, check=False, timeout=30,
            )
            if up_cfg.returncode != 0:
                raise RuntimeError(f"dépôt de la configuration distante en échec : {up_cfg.stderr.decode(errors='replace').strip()}")
            phases["upload_s"] = round(time.monotonic() - t_upload, 2)

            log(
                f"Déport vers {machine_id} pour {target} (commit {git_source['sha12']}, "
                f"motif : {decision_reason})"
            )

            # Lancer le gardien par son chemin (jamais bash -s) afin de garder stdin ouvert pour le battement.
            guardian_cmd = [
                *ssh_prefix,
                f"python3 {shlex.quote(remote_guardian)} {shlex.quote(remote_cfg)}",
            ]
            proc = subprocess.Popen(
                guardian_cmd,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=sys.stderr,
            )
            stop_hb = threading.Event()

            def send_heartbeats():
                try:
                    while not stop_hb.is_set() and proc.poll() is None:
                        proc.stdin.write(b"ping\n")
                        proc.stdin.flush()
                        stop_hb.wait(HEAVY_WORK_HEARTBEAT_INTERVAL_SECS)
                except (BrokenPipeError, OSError, ValueError):
                    pass

            hb_sender = threading.Thread(target=send_heartbeats, daemon=True)
            hb_sender.start()
            try:
                stdout_bytes = proc.stdout.read()
                proc.wait()
            finally:
                stop_hb.set()
                hb_sender.join(timeout=2)
                try:
                    proc.stdin.close()
                except OSError:
                    pass
                try:
                    proc.stdout.close()
                except OSError:
                    pass

            remote_result = None
            for line in stdout_bytes.decode(errors="replace").splitlines():
                if line.startswith(HEAVY_WORK_RESULT_PREFIX):
                    remote_result = json.loads(line[len(HEAVY_WORK_RESULT_PREFIX):])
            if not remote_result or not remote_result.get("ok"):
                err = (remote_result or {}).get("error") or f"gardien distant terminé avec code {proc.returncode}"
                raise RuntimeError(f"compilation distante sur {machine_id} échouée : {err}")

            phases.update(remote_result.get("phases") or {})
            remote_binary = remote_result["binary_path"]
            expected_bin_sha = remote_result["binary_sha256"]

            # Rapatrier le binaire et revérifier son SHA-256 avant pose atomique.
            dest_path, is_native = resolve_local_output_binary_path(repo, cargo_args, target, native_target)
            dest_path.parent.mkdir(parents=True, exist_ok=True)
            staged_local = dest_path.parent / f".bridget-download-{os.getpid()}.tmp"
            t_download = time.monotonic()
            try:
                with staged_local.open("wb") as out_stream:
                    dl_res = subprocess.run(
                        [*ssh_prefix, f"cat {shlex.quote(remote_binary)}"],
                        stdout=out_stream, stderr=subprocess.PIPE, check=False, timeout=300,
                    )
                if dl_res.returncode != 0:
                    raise RuntimeError(f"rapatriement du binaire en échec : {dl_res.stderr.decode(errors='replace').strip()}")
                actual_bin_sha = digest(staged_local).hex()
                if actual_bin_sha != expected_bin_sha:
                    raise RuntimeError(
                        f"SHA-256 après rapatriement incohérent : attendu {expected_bin_sha}, reçu {actual_bin_sha}"
                    )
                os.chmod(staged_local, 0o755)
                os.replace(staged_local, dest_path)
            finally:
                if staged_local.exists():
                    try:
                        staged_local.unlink()
                    except OSError:
                        pass
            phases["download_s"] = round(time.monotonic() - t_download, 2)

            if is_native:
                try:
                    register_staging(repo, cargo_args)
                except (OSError, ValueError) as error:
                    log(f"avertissement : staging local non enregistré : {error}")

            maint_warnings = remote_result.get("maintenance_warnings") or []
            maint_note = "ok" if not maint_warnings else "; ".join(maint_warnings)
            log(
                f"Succès déporté sur {machine_id} ({target}) -> {dest_path} | "
                f"build_id={remote_result['build_info']['build_id']} | "
                f"sha256={expected_bin_sha[:12]} | phases={json.dumps(phases)} | "
                f"entretien_distant={maint_note}"
            )
            return 0
    finally:
        stop_renew.set()
        if renew_thread is not None:
            renew_thread.join(timeout=2)
        _release_lease_safely(lease_id)
        if remote_work_dir:
            try:
                subprocess.run(
                    [*ssh_prefix, f"rm -rf {shlex.quote(remote_work_dir)}"],
                    capture_output=True, check=False, timeout=15,
                )
            except (OSError, subprocess.SubprocessError):
                pass


def run_local_cargo(repo, cargo_args, discrete=False, lease_id=None):
    """Exécute Cargo localement avec entretien avant/après, mode discret éventuel et bail éventuel."""
    try:
        build_root = configure_build_target(repo, cargo_args)
        root_identity = (build_root.stat().st_dev, build_root.stat().st_ino) if build_root else None
    except (OSError, ValueError) as error:
        log(f"Cette compilation n’a pas été lancée : {error}")
        return 2
    cargo_bin = os.environ.get("BRIDGET_CARGO") or shutil.which("cargo")
    if not cargo_bin:
        fallback = Path.home() / ".cargo/bin/cargo"
        cargo_bin = str(fallback) if fallback.is_file() else "cargo"
    maint_errors = maintain_build_caches(repo, cargo_args)
    if build_root:
        try:
            if (external_staging_parent() != build_root
                    or (build_root.stat().st_dev, build_root.stat().st_ino) != root_identity):
                raise ValueError("la racine configurée a changé pendant l’entretien préalable")
        except (OSError, ValueError) as error:
            log(f"Cette compilation n’a pas été lancée : {error}. Rétablissez le volume et sa configuration ; aucun repli.")
            return 2

    stop_renew = threading.Event()
    renew_thread = None
    if lease_id:
        renew_thread = threading.Thread(
            target=_lease_renewer_loop, args=(lease_id, stop_renew), daemon=True
        )
        renew_thread.start()
    try:
        if discrete:
            cmd, run_env = discrete_cargo_command(cargo_bin, cargo_args, os.environ)
            log(f"Compilation locale en mode discret (CARGO_BUILD_JOBS={run_env['CARGO_BUILD_JOBS']})")
            result = subprocess.run(cmd, cwd=repo, env=run_env, check=False)
        else:
            result = subprocess.run([cargo_bin, *cargo_args], cwd=repo, check=False)
    finally:
        stop_renew.set()
        if renew_thread is not None:
            renew_thread.join(timeout=2)
        _release_lease_safely(lease_id)

    maint_errors.extend(maintain_build_caches(repo, cargo_args))
    if maint_errors:
        log(f"Bilan entretien des caches (non bloquant) : {'; '.join(maint_errors)}")
    if build_root:
        try:
            if (external_staging_parent() != build_root
                    or (build_root.stat().st_dev, build_root.stat().st_ino) != root_identity):
                raise ValueError("la racine configurée a changé pendant Cargo")
        except (OSError, ValueError) as error:
            log(f"Résultat de cette compilation à vérifier : {error}. "
                "Conservez les fichiers produits ; ne relancez pas avant le retour du volume. "
                "Aucun retrait sur une cible non qualifiée.")
            return result.returncode or 2
    return result.returncode


def run_cargo_action(repo, raw_arguments, parser):
    try:
        opts, cargo_args = parse_cargo_wrapper_options(raw_arguments)
    except ValueError as error:
        log(f"Option invalide : {error}")
        return 2
    if not cargo_args or cargo_args[0] not in ("build", "test", "check", "clippy"):
        parser.error("commande Cargo attendue : build, test, check ou clippy")

    if not is_release_build_command(cargo_args):
        if opts.build_target or opts.host:
            log(
                "Le déport du Lot 1a couvre uniquement `cargo build --release`. "
                "Retirez --host / --build-target pour cette commande locale."
            )
            return 2
        return run_local_cargo(repo, cargo_args, discrete=False)

    native_target = detect_local_target()
    requested_target = opts.build_target or native_target
    if not requested_target:
        log(
            "Cible native indéterminée sur cette machine. "
            "Précisez --build-target <macos-arm64|linux-x86_64-glibc2.36>."
        )
        return 2
    if opts.local and opts.host:
        log("Options incompatibles : --local et --host ne peuvent pas être combinés.")
        return 2

    # Cas 1 : --local explicite (§3.3.2, §3.3.3)
    if opts.local:
        ok_compat, compat_err = check_local_target_compatible(requested_target)
        if not ok_compat:
            log(f"Refus de compilation locale : {compat_err}")
            return 2
        discrete = True
        ok_status, status_payload = call_bridget_heavy_work("status")
        if ok_status and isinstance(status_payload, dict) and status_payload.get("configured"):
            local_host = platform.node().split(".", 1)[0]
            for machine in status_payload.get("machines", []):
                if machine.get("machine_id") == local_host:
                    role = machine.get("role")
                    if role == "excluded":
                        log(
                            f"Refus : la machine locale « {local_host} » a le rôle « exclue » dans la politique maître. "
                            "Tout travail lourd local est interdit ; relancez sans --local ou avec --host <machine>."
                        )
                        return 2
                    if role == "open":
                        discrete = False
                    break
        return run_local_cargo(repo, cargo_args, discrete=discrete)

    # Cas 2 : décision via le daemon maître (ou repli sûr / --host sans daemon)
    owner_sha = canonical_owner_id(repo) or hashlib.sha256(owner_id(repo).encode()).hexdigest()
    expected_rust = read_pinned_rust_channel(repo)
    repo_slug = repo.name
    exclusions = []
    wait_started = None

    while True:
        acquire_args = ["--target", requested_target, "--repo-owner", owner_sha]
        if opts.host:
            acquire_args.extend(["--host", opts.host])
        for machine_id, failure_kind in exclusions:
            acquire_args.extend(["--exclude", f"{machine_id}:{failure_kind}"])

        ok_rpc, payload = call_bridget_heavy_work("acquire", acquire_args)
        if not ok_rpc:
            # Sans daemon joignable (§3.3.3, §4.3, §7.9)
            if opts.host:
                missing = [
                    flag for flag, val in (
                        ("--ssh-host", opts.ssh_host),
                        ("--ssh-user", opts.ssh_user),
                        ("--ssh-port", opts.ssh_port),
                        ("--ssh-key", opts.ssh_key),
                        ("--ssh-known-hosts", opts.ssh_known_hosts),
                    )
                    if not val
                ]
                if missing:
                    log(
                        f"Daemon maître injoignable ({payload}) : --host {opts.host} exige les paramètres SSH "
                        f"explicites ({', '.join(missing)})."
                    )
                    return 2
                log(
                    f"Avertissement : daemon maître injoignable ({payload}) — le rôle « exclue » de {opts.host} "
                    "ne peut pas être vérifié. La sérialisation reste assurée par le verrou distant."
                )
                ssh_profile = {
                    "host": opts.ssh_host,
                    "user": opts.ssh_user,
                    "port": opts.ssh_port,
                    "identity_rel": opts.ssh_key,
                    "known_hosts_rel": opts.ssh_known_hosts,
                }
                try:
                    git_source = validate_git_source_for_offload(repo)
                    for path_u in git_source["untracked"][:5]:
                        log(f"Avertissement : fichier non suivi exclu de l'archive : {path_u}")
                    preflight = run_remote_preflight(ssh_profile, requested_target, expected_rust, repo_slug)
                    if not preflight.get("ok"):
                        log(f"Pré-vérification distante refusée sur {opts.host} : {preflight.get('reason')}")
                        return 2
                    return execute_remote_build(
                        repo, cargo_args, requested_target, native_target, opts.host,
                        ssh_profile, preflight, git_source, "--host sans daemon", lease_id=None,
                    )
                except (OSError, ValueError, RuntimeError) as error:
                    log(f"Déport vers {opts.host} refusé : {error}")
                    return 2

            ok_compat, compat_err = check_local_target_compatible(requested_target)
            if not ok_compat:
                log(f"Daemon maître injoignable ({payload}) et {compat_err}")
                return 2
            log(
                f"Daemon maître injoignable ({payload}) : défaut sûr actif (compilation locale en mode discret). "
                "Pour forcer une machine distante sans daemon, utilisez --host <machine> avec les paramètres SSH."
            )
            return run_local_cargo(repo, cargo_args, discrete=True)

        decision = payload.get("decision") if isinstance(payload, dict) else None
        kind = decision.get("kind") if isinstance(decision, dict) else None

        if kind == "policy_absent":
            ok_compat, compat_err = check_local_target_compatible(requested_target)
            if not ok_compat:
                log(f"Politique de travaux lourds absente sur le maître et {compat_err}")
                return 2
            log("Politique de travaux lourds absente sur le maître : défaut sûr actif (compilation locale en mode discret).")
            return run_local_cargo(repo, cargo_args, discrete=True)

        if kind == "run_local_lease":
            lease_id = decision.get("lease_id")
            ok_compat, compat_err = check_local_target_compatible(requested_target)
            if not ok_compat:
                _release_lease_safely(lease_id)
                log(f"Refus : {compat_err}")
                return 2
            log(f"Exécution locale sous bail ({decision.get('machine_id')}) : {decision.get('reason')}")
            return run_local_cargo(repo, cargo_args, discrete=False, lease_id=lease_id)

        if kind == "run_remote":
            lease_id = decision.get("lease_id")
            machine_id = decision["machine_id"]
            ssh_profile = decision["ssh"]
            reason = decision.get("reason", "")
            try:
                git_source = validate_git_source_for_offload(repo)
            except ValueError as error:
                _release_lease_safely(lease_id)
                log(f"Déport refusé : {error}")
                return 2
            if git_source["untracked"]:
                preview = ", ".join(git_source["untracked"][:5])
                suffix = "…" if len(git_source["untracked"]) > 5 else ""
                log(f"Avertissement : {len(git_source['untracked'])} fichier(s) non suivi(s) exclu(s) de l'archive ({preview}{suffix}).")

            try:
                preflight = run_remote_preflight(ssh_profile, requested_target, expected_rust, repo_slug)
            except ValueError as error:
                _release_lease_safely(lease_id)
                log(f"Profil SSH invalide pour {machine_id} : {error}")
                return 2

            if not preflight.get("ok"):
                failure_kind = preflight.get("failure_kind", "unreachable")
                log(
                    f"Pré-vérification échouée sur {machine_id} ({failure_kind}) : {preflight.get('reason')} "
                    "— libération du bail et recherche d'une autre machine."
                )
                _release_lease_safely(lease_id)
                exclusions.append((machine_id, failure_kind))
                continue

            try:
                return execute_remote_build(
                    repo, cargo_args, requested_target, native_target, machine_id,
                    ssh_profile, preflight, git_source, reason, lease_id=lease_id,
                )
            except (OSError, ValueError, RuntimeError) as error:
                log(str(error))
                return 1

        if kind == "wait":
            max_wait = int(decision.get("max_wait_secs", 600))
            retry_after = max(1, int(decision.get("retry_after_secs", 5)))
            now = time.monotonic()
            if wait_started is None:
                wait_started = now
                log(
                    f"Attente d'une machine ouverte ({decision.get('reason')}, "
                    f"plafond {max_wait} s, nouvelle tentative toutes les {retry_after} s)…"
                )
            elapsed = now - wait_started
            if elapsed >= max_wait:
                fallback = decision.get("fallback_after_wait", "refuse")
                if fallback == "discrete":
                    ok_compat, compat_err = check_local_target_compatible(requested_target)
                    if not ok_compat:
                        log(f"Attente maximale ({max_wait} s) atteinte et {compat_err}")
                        return 2
                    log(f"Attente maximale ({max_wait} s) atteinte : repli en mode discret sur ce poste.")
                    return run_local_cargo(repo, cargo_args, discrete=True)
                log(
                    f"Attente maximale ({max_wait} s) atteinte ({decision.get('reason')}) : "
                    "la politique refuse le repli local. Relancez avec --local ou --host <machine> si souhaité."
                )
                return 2
            time.sleep(min(retry_after, max(0.1, max_wait - elapsed)))
            continue

        if kind == "fallback_discrete":
            ok_compat, compat_err = check_local_target_compatible(requested_target)
            if not ok_compat:
                log(f"Repli local refusé ({decision.get('reason')}) : {compat_err}")
                return 2
            log(f"Repli discret autorisé (cas {decision.get('case_kind')}) : {decision.get('reason')}")
            return run_local_cargo(repo, cargo_args, discrete=True)

        if kind == "refused":
            log(
                f"Compilation refusée [{decision.get('code')}] (cas {decision.get('case_kind')}) : "
                f"{decision.get('message')} Action : {decision.get('action')}"
            )
            return 2

        log(f"Réponse inattendue du daemon maître : {payload}")
        return 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    cargo = sub.add_parser("cargo", help="arguments Cargo (avec déport release Lot 1a et entretien avant/après)")
    cargo.add_argument("arguments", nargs=argparse.REMAINDER)
    clean = sub.add_parser("clean", help="purger les caches anciens ou au-delà du budget")
    clean.add_argument("--dry-run", action="store_true")
    done = sub.add_parser("installed", help="purger les intermédiaires d'un staging installé")
    done.add_argument("--target", required=True)
    done.add_argument("--binary", required=True)
    done.add_argument("--dry-run", action="store_true")
    target = sub.add_parser("target-dir", help="résoudre la cible pour les recettes, sans lancer Cargo")
    target.add_argument("--installer", action="store_true", help="staging historique si aucun volume configuré")
    inspect = sub.add_parser("worktrees", help="inventaire lecture seule des worktrees")
    inspect.add_argument("--repo", required=True)
    inspect.add_argument("--base-ref", default="main")
    inspect.add_argument("--json", action="store_true")
    # Contourner le bogue Python 3.9 d'argparse.REMAINDER sur une option en tête
    # (ex: `build.py cargo --local build --release`).
    if len(sys.argv) >= 2 and sys.argv[1] == "cargo":
        args = argparse.Namespace(action="cargo", arguments=sys.argv[2:])
    else:
        args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    if args.action == "target-dir":
        try:
            configure_build_target(repo, [])
            target = os.environ.get("CARGO_TARGET_DIR")
            path = Path(os.path.abspath(repo / target)) if target else (
                staging_parent() / "bridget-build-installer" if args.installer else repo / "target")
            print(path)
            return 0
        except (OSError, ValueError) as error:
            log(f"Cible de compilation indisponible : {error}")
            return 2
    if args.action == "worktrees":
        return report_worktrees(Path(args.repo), args.base_ref, args.json)
    if args.action == "cargo":
        code = run_cargo_action(repo, args.arguments, parser)
        if code:
            return code  # Aucun retrait de worktree après un échec.
    try:
        if args.action == "installed":
            installed(repo, args.target, args.binary, args.dry_run)
        else:
            if args.action != "cargo":
                prune(repo, *limits(), dry_run=getattr(args, "dry_run", False))
            prune_merged_worktrees(repo, dry_run=getattr(args, "dry_run", False))
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        log(str(error))
        # L'entretien ne transforme pas un build réussi en échec de compilation.
        return 0 if args.action == "cargo" else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
