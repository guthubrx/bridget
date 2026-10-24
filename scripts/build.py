#!/usr/bin/env python3
"""Cargo avec entretien avant et après exécution ; jamais de cargo clean global."""

import argparse
import fcntl
import hashlib
import json
import math
import os
import re
import selectors
import signal
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time
from collections import namedtuple


CACHE_DIRS = ("deps", "build", "incremental", ".fingerprint")
OWNER = ".bridget-build-owner.json"
GIB = 1024 ** 3


def log(message):
    print(f"[build Bridget] {message}", file=sys.stderr, flush=True)


def plain_dir(path):
    return path.is_dir() and not path.is_symlink() and path.absolute() == path.resolve()


def worktrees(repo):
    result = subprocess.run(
        ["git", "-C", str(repo), "worktree", "list", "--porcelain", "-z"],
        capture_output=True, check=False, timeout=10,
    )
    if result.returncode:
        return [repo]  # Paquet source sans Git : aucun autre projet à examiner.
    return [Path(os.fsdecode(line[9:])) for line in result.stdout.split(b"\0")
            if line.startswith(b"worktree ")]


def owner_id(repo):
    result = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "--path-format=absolute", "--git-common-dir"],
        text=True, capture_output=True, check=False, timeout=10,
    )
    return str(Path(result.stdout.strip()).resolve()) if result.returncode == 0 else str(repo)


def staging_parent():
    # Ne pas étendre ce périmètre à XDG_CACHE_HOME ou à un chemin fourni au nettoyeur.
    return Path.home().resolve() / ".cache"


def is_staging(path):
    return (plain_dir(path) and path.parent == staging_parent()
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
    if not is_staging(path):
        return
    marker = path / OWNER
    if marker.exists() or marker.is_symlink():
        return  # Ne jamais s'approprier un cache déjà marqué, même par un autre dépôt.
    with marker.open("x", encoding="utf-8") as stream:
        json.dump({"version": 1, "repo": owner_id(repo)}, stream)


def owned_staging(repo):
    identity = owner_id(repo)
    for path in staging_parent().glob("bridget-build-*"):
        marker = path / OWNER
        if not is_staging(path) or marker.is_symlink() or not marker.is_file():
            continue
        try:
            data = json.loads(marker.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            continue
        if data == {"version": 1, "repo": identity}:
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


def prune(repo, max_bytes, max_age, dry_run=False, only=None):
    if sys.version_info < (3, 11):
        raise RuntimeError("nettoyage sûr : Python 3.11 minimum requis")
    if not shutil.rmtree.avoids_symlink_attacks:
        raise RuntimeError("suppression sûre par descripteur indisponible ; nettoyage ignoré")
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
                                shutil.rmtree(child, dir_fd=directory)
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
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").digest()


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

    def remaining(self):
        return max(0, self.deadline - time.monotonic())


def observed_run(arguments, budget=None, timeout=15, limit=MAX_OBSERVATION_BYTES):
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
                                 start_new_session=True, env=dict(os.environ, LC_ALL="C"))
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


def process_output_complete(stdout, exempt=()):
    # Les types explicitement non fichiers n'ont pas de chemin à protéger.
    nonfiles = {"a_inode", "netlink", "unix", "IPv4", "IPv6", "sock", "pack", "PIPE",
                "KQUEUE", "PSXSEM", "PSXSHM", "systm", "NPOLICY", "FSEVENTS", "NEXUS", "CHAN"}
    pid, record = None, {}
    seen = False

    def valid():
        kind, name = record.get("t"), record.get("n", "")
        if "f" not in record:
            return False
        if kind in nonfiles:
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
                    return False
                record = {}
            if field == "p":
                if not value.isdigit():
                    return False
                pid = value
            elif field in {"f", "t", "D", "n"}:
                if pid is None or field in record:
                    return False
                record[field] = value
                seen = True
            else:
                return False
        except (ValueError, UnicodeError):
            return False
    return seen and (not record or valid())


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


def validate_mount_scopes(scopes, mounts):
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
                    raise ValueError("alias mount may hide usage")
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


def busy_paths(scopes=(), budget=None, ignore_own=()):
    budget = budget or ObservationBudget()
    command = LSOF_OPEN_FILES
    before, fuse = None, ()
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
        if result.returncode:
            budget.failures.add("process_access_unavailable")
            return None
        if result.stderr.strip():
            if before is None:
                raise ValueError("diagnostic without mount identity")
            exempt = foreign_mount_exemptions(result.stderr, scopes, mounts)
            result = observed_run([*command, *(arg for name in exempt for arg in ("-e", name))],
                                  budget, timeout=30)
            if result.returncode or result.stderr.strip() or not process_output_complete(result.stdout, (*fuse, *exempt)):
                raise ValueError("incomplete scoped observation")
            budget.exempt_mounts = len(exempt)
        elif not process_output_complete(result.stdout, fuse):
            raise ValueError("incomplete process records")
        if before is not None and scopes:
            validate_mount_scopes(scopes, mounts)
        if before is not None and mount_snapshot()[0] != before:
            raise ValueError("mount inventory changed")
        if not budget.remaining():
            raise ValueError("observation expired")
        paths = parse_open_paths(result.stdout, ignore_own)
        budget.process_reader = "system_lsof" if before is None else "linux_system_lsof"
        if scopes:
            roots = {path.resolve() for path in scopes}
            paths = [path for path in paths if path in roots or not roots.isdisjoint(path.parents)]
        return paths
    except (OSError, ValueError, UnicodeError):
        budget.failures.add("process_observation_incomplete")
        return None


def git_ok(repo, *arguments, budget=None):
    budget = budget or ObservationBudget()
    budget.git_calls += 1
    return observed_run(["git", "-c", "core.fsmonitor=false", "-C", str(repo), *arguments],
                        budget, timeout=15, limit=1024 * 1024)


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
    try:
        register_staging(repo, arguments)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        log(f"staging non enregistré : {error}")
    try:
        prune(repo, *limits())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        log(f"entretien des caches différé : {error}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    cargo = sub.add_parser("cargo", help="arguments Cargo inchangés, entretien avant et après exécution")
    cargo.add_argument("arguments", nargs=argparse.REMAINDER)
    clean = sub.add_parser("clean", help="purger les caches anciens ou au-delà du budget")
    clean.add_argument("--dry-run", action="store_true")
    done = sub.add_parser("installed", help="purger les intermédiaires d'un staging installé")
    done.add_argument("--target", required=True)
    done.add_argument("--binary", required=True)
    done.add_argument("--dry-run", action="store_true")
    inspect = sub.add_parser("worktrees", help="inventaire lecture seule des worktrees")
    inspect.add_argument("--repo", required=True)
    inspect.add_argument("--base-ref", default="main")
    inspect.add_argument("--json", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    if args.action == "worktrees":
        return report_worktrees(Path(args.repo), args.base_ref, args.json)
    if args.action == "cargo":
        if not args.arguments or args.arguments[0] not in ("build", "test", "check", "clippy"):
            parser.error("commande Cargo attendue : build, test, check ou clippy")
        cargo_bin = os.environ.get("BRIDGET_CARGO") or shutil.which("cargo")
        if not cargo_bin:
            fallback = Path.home() / ".cargo/bin/cargo"
            cargo_bin = str(fallback) if fallback.is_file() else "cargo"
        maintain_build_caches(repo, args.arguments)
        result = subprocess.run([cargo_bin, *args.arguments], cwd=repo, check=False)
        maintain_build_caches(repo, args.arguments)
        if result.returncode:
            return result.returncode  # Aucun retrait de worktree après un échec.
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
