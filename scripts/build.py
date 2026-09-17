#!/usr/bin/env python3
"""Cargo puis entretien borné de ses caches ; jamais de cargo clean global."""

import argparse
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import time


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


def opened_paths():
    """Échec de l'observation = aucune suppression, jamais une supposition d'inactivité."""
    result = subprocess.run(["lsof", "-nP", "-Fpn"], capture_output=True, timeout=30)
    if result.returncode != 0 or result.stderr:
        raise RuntimeError("lsof indisponible ou incomplet ; nettoyage ignoré")
    pid, paths = None, []
    for line in result.stdout.splitlines():
        if line.startswith(b"p"):
            pid = int(line[1:])
        elif line.startswith(b"n/") and pid != os.getpid():
            paths.append(os.fsdecode(line[1:]))
    return paths


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
                if any(p == name or p.startswith(name + "/") for p in opened_paths()):
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    cargo = sub.add_parser("cargo", help="arguments Cargo inchangés, nettoyage après succès")
    cargo.add_argument("arguments", nargs=argparse.REMAINDER)
    clean = sub.add_parser("clean", help="purger les caches anciens ou au-delà du budget")
    clean.add_argument("--dry-run", action="store_true")
    done = sub.add_parser("installed", help="purger les intermédiaires d'un staging installé")
    done.add_argument("--target", required=True)
    done.add_argument("--binary", required=True)
    done.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    if args.action == "cargo":
        if not args.arguments or args.arguments[0] not in ("build", "test", "check", "clippy"):
            parser.error("commande Cargo attendue : build, test, check ou clippy")
        cargo_bin = os.environ.get("BRIDGET_CARGO") or shutil.which("cargo")
        if not cargo_bin:
            fallback = Path.home() / ".cargo/bin/cargo"
            cargo_bin = str(fallback) if fallback.is_file() else "cargo"
        result = subprocess.run([cargo_bin, *args.arguments], cwd=repo, check=False)
        if result.returncode:
            return result.returncode  # Conserver les traces d'un build échoué.
        try:
            register_staging(repo, args.arguments)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            log(f"staging non enregistré : {error}")
    try:
        if args.action == "installed":
            installed(repo, args.target, args.binary, args.dry_run)
        else:
            prune(repo, *limits(), dry_run=getattr(args, "dry_run", False))
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        log(str(error))
        # L'entretien ne transforme pas un build réussi en échec de compilation.
        return 0 if args.action == "cargo" else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
