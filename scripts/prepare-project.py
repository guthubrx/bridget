#!/usr/bin/env python3
"""Préparation des dépôts distants sur la fédération SSH existante.
Aucun lancement de modèle. Le worker conserve son admission et ses contrôles.
"""
import base64
import contextlib
import errno
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tempfile

TIMEOUT = 300
MAX_JSON = 1024 * 1024

class Refused(Exception):
    pass

def require(condition, code):
    if not condition:
        raise Refused(code)

def digest(data):
    return hashlib.sha256(data).hexdigest()

def private_read(path):
    path = Path(path)
    info = path.lstat()
    require(stat.S_ISREG(info.st_mode) and info.st_uid == os.geteuid()
            and stat.S_IMODE(info.st_mode) == 0o600, "project_preparation_private_file")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
    with os.fdopen(fd, "rb") as stream:
        require(os.fstat(stream.fileno()).st_ino == info.st_ino,
                "project_preparation_file_changed")
        data = stream.read(MAX_JSON + 1)
    require(len(data) <= MAX_JSON, "project_preparation_registry_size")
    return data

def absolute(path):
    value = Path(path)
    require(value.is_absolute() and ".." not in value.parts
            and not any(ord(c) < 32 for c in str(value)), "project_preparation_path")
    return value

def contained(path, root):
    return path == root or root in path.parents

def no_links(path):
    path = absolute(path)
    for item in reversed((path, *path.parents)):
        try:
            require(not item.is_symlink(), "project_preparation_symlink")
        except OSError:
            raise Refused("project_preparation_path")
    return path

def private_dir(path):
    path = no_links(path)
    path.mkdir(mode=0o700, parents=True, exist_ok=True)
    info = path.stat()
    require(info.st_uid == os.geteuid() and stat.S_IMODE(info.st_mode) == 0o700,
            "project_preparation_private_directory")
    return path

@contextlib.contextmanager
def locked(path):
    fd = os.open(no_links(path), os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "a+") as handle:
        # Non bloquant : une seconde admission reprend après la première.
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise Refused("project_preparation_busy")
        yield

def atomic_json(path, value, previous=None):
    path = Path(path)
    data = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
    fd, name = tempfile.mkstemp(prefix=".project-prepare-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as output:
            os.fchmod(output.fileno(), 0o600)
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
        if previous is not None:
            require(private_read(path) == previous, "project_preparation_registry_changed")
        os.replace(name, path)
    finally:
        if os.path.exists(name):
            os.unlink(name)

def git(root, *arguments, check=True, phase="git"):
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
               GIT_TERMINAL_PROMPT="0", GIT_NO_REPLACE_OBJECTS="1",
               GIT_LFS_SKIP_SMUDGE="1")
    timeout_code = {
        "git": "project_preparation_git_timeout",
        "bundle_base": "project_preparation_bundle_base_timeout",
        "bundle_export": "project_preparation_bundle_export_timeout",
        "bundle_verify": "project_preparation_bundle_verify_timeout",
        "bundle_import": "project_preparation_bundle_import_timeout",
    }[phase]
    try:
        result = subprocess.run(
            ["git", "-c", "gc.auto=0", "-c", "maintenance.auto=false", "-c", "core.hooksPath=" + os.devnull,
             "-c", "protocol.file.allow=always", "-C", str(root), *arguments],
            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        raise Refused(timeout_code) from None
    require(len(result.stdout) <= MAX_JSON, "project_preparation_git_output")
    if check:
        require(result.returncode == 0, "project_preparation_git_failed")
    return result

def machine(registry, request):
    selected = registry.get("machines", {}).get(request["machine_id"], {})
    require(selected.get("enabled") is True
            and all(selected.get(k) == request["authority"].get(k)
                    for k in ("worker_id", "provisioning_revision", "credential_fingerprint")),
            "project_preparation_machine_mismatch")
    return selected

def registry_read(path):
    raw = private_read(path)
    return raw, json.loads(raw)

def mapping_matches(mapping, host, cwd):
    return mapping["source_host"] == host and contained(cwd, absolute(mapping["source_root"]))

def select_mapping(selected, request):
    mappings = selected.get("mappings", {})
    requested = request.get("project_id")
    source = absolute(request["source_root"])
    cwd = absolute(request["cwd"])
    if requested:
        require(requested in mappings, "project_mapping_unknown")
        found = [(requested, mappings[requested])]
    else:
        found = [(key, value) for key, value in mappings.items()
                 if mapping_matches(value, request["origin_host"], cwd)]
    require(len(found) <= 1, "project_mapping_ambiguous")
    if found:
        key, value = found[0]
        require(mapping_matches(value, request["origin_host"], cwd),
                "project_mapping_mismatch")
        return key, value
    identity = digest((request["origin_host"] + "\0" + str(source)).encode())
    key = "auto-" + identity[:32]
    base = Path.home() / ".local/share/bridget-projects" / key
    mapping = dict(source_host=request["origin_host"], source_root=str(source),
                   remote_root=str(base / "repository"),
                   expected_git_identity=str(base / "mirror.git"), mapping_revision=1)
    extra = request.get("additional_root")
    if extra:
        mapping["additional_root"] = dict(extra, remote_root=str(base / "attachments"),
                                          remote_uid=os.geteuid(), remote_mode=0o700)
    return key, mapping

def commit_present(root, commit):
    return git(root, "cat-file", "-e", commit + "^{commit}", check=False).returncode == 0

def git_common_dir(root):
    value = git(root, "rev-parse", "--path-format=absolute", "--git-common-dir").stdout.decode().strip()
    return no_links(absolute(value)).resolve()

def linked_checkout_target(mapping, request):
    relative = request.get("linked_checkout")
    if relative is None:
        return None
    source = absolute(request.get("checkout_root", request["source_root"]))
    require(contained(source, absolute(request["source_root"]))
            and contained(absolute(request["cwd"]), source),
            "project_preparation_git_identity_mismatch")
    mapped = absolute(mapping["source_root"])
    require(contained(source, mapped) and source != mapped and request.get("commit"),
            "project_preparation_git_identity_mismatch")
    expected = source.relative_to(mapped)
    require(str(expected) == relative and ".git" not in expected.parts,
            "project_preparation_git_identity_mismatch")
    return no_links(absolute(mapping["remote_root"]) / expected)

def check_linked_checkout(root, target):
    top = git(target, "rev-parse", "--show-toplevel", check=False)
    require(top.returncode == 0 and Path(top.stdout.decode().strip()) == target
            and git_common_dir(target) == git_common_dir(root),
            "project_preparation_destination_occupied")

def prepare_linked_checkout(root, mapping, request, registry_path):
    target = linked_checkout_target(mapping, request)
    if target is None:
        return
    identity = dict(source_host=mapping["source_host"],
                    source_root=request.get("checkout_root", request["source_root"]),
                    target=str(target), common_dir=str(git_common_dir(root)))
    receipts = private_dir(Path(registry_path).parent / "linked-checkouts")
    receipt = receipts / (digest(json.dumps(identity, sort_keys=True).encode()) + ".json")
    previous = private_read(receipt) if receipt.exists() else None
    if previous is not None:
        saved = json.loads(previous)
        require(saved.get("identity") == identity, "project_preparation_mapping_changed")
        require(saved.get("ready") is True, "project_preparation_checkout_incomplete")
    if target.exists():
        check_linked_checkout(root, target)
        if previous is None:
            # Sans recu de creation, des fichiers suivis manquants peuvent
            # venir d'une extraction interrompue. Conserver puis refuser.
            status = git(target, "status", "--porcelain=v1", "--untracked-files=no")
            require(not status.stdout.strip(), "project_preparation_checkout_incomplete")
        # Le worker reste responsable du HEAD : sonde puis FF propre/inactif,
        # ou refus. Le helper ne deplace jamais un checkout existant.
    else:
        atomic_json(receipt, dict(identity=identity, ready=False), previous)
        previous = private_read(receipt)
        target.parent.mkdir(parents=True, exist_ok=True)
        no_links(target.parent)
        try:
            git(root, "worktree", "add", "--detach", str(target), request["commit"])
        except (Refused, OSError) as error:
            if str(error).endswith("_timeout"):
                raise
            try:
                if not os.path.lexists(target):
                    listed = git(root, "worktree", "list", "--porcelain", "-z", check=False)
                    entry = b"worktree " + os.fsencode(target)
                    if listed.returncode == 0 and entry not in listed.stdout.split(b"\0"):
                        require(private_read(receipt) == previous, "project_preparation_registry_changed")
                        receipt.unlink()
            except (Refused, OSError, subprocess.TimeoutExpired):
                pass
            raise
        check_linked_checkout(root, target)
    atomic_json(receipt, dict(identity=identity, ready=True), previous)

def remote_inspect(request, registry_path):
    commit = request.get("commit")
    if commit is not None:
        require(isinstance(commit, str) and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit),
                "expected_base_commit_invalid")
    _, registry = registry_read(registry_path)
    selected = machine(registry, request)
    key, mapping = select_mapping(selected, request)
    proposed = request.get("mapping")
    if proposed is not None:
        require(proposed == mapping and request.get("resolved_project_id") == key,
                "project_preparation_mapping_changed")
    root = no_links(mapping["remote_root"])
    target = linked_checkout_target(mapping, request)
    if target is not None and target.exists():
        check_linked_checkout(root, target)
    if root.exists():
        require(root.is_dir(), "project_preparation_destination_occupied")
        if request.get("commit"):
            origin = git(root, "config", "--local", "--no-includes",
                         "--get", "remote.origin.url", check=False)
            require(origin.returncode == 0
                    and origin.stdout.decode().strip() == mapping["expected_git_identity"],
                    "project_preparation_destination_occupied")
        missing = bool(request.get("commit") and not commit_present(root, request["commit"]))
        have = git(root, "rev-parse", "HEAD", check=False).stdout.decode().strip() if missing else None
        base = request.get("bundle_base")
        if base is not None:
            require(isinstance(base, str) and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", base),
                    "project_preparation_bundle_base_invalid")
            require(commit_present(root, base), "project_preparation_bundle_base_missing")
            have = base if missing else None
        return dict(project_id=key, mapping=mapping, needs_bundle=missing, have_commit=have)
    require(request.get("bundle_base") is None, "project_preparation_bundle_base_missing")
    require(request.get("commit") is not None, "project_preparation_git_required")
    return dict(project_id=key, mapping=mapping, needs_bundle=True, have_commit=None)

def publish_mapping(registry_path, request, key, mapping):
    raw, registry = registry_read(registry_path)
    selected = machine(registry, request)
    mappings = selected.setdefault("mappings", {})
    previous = mappings.get(key)
    require(previous is None or previous == mapping, "project_preparation_mapping_changed")
    if previous is None:
        require(len(mappings) < 64, "machine_mapping_limit")
        mappings[key] = mapping
        atomic_json(registry_path, registry, raw)

def remote_receive(request, registry_path, bundle_stream):
    with locked(Path(registry_path).parent / "project-preparation.lock"):
        result = remote_inspect(request, registry_path)
        mapping = result["mapping"]
        root = no_links(mapping["remote_root"])
        commit = request.get("commit")
        if result["needs_bundle"]:
            with tempfile.TemporaryDirectory(prefix="bridget-project-") as temp:
                bundle = Path(temp) / "source.bundle"
                with bundle.open("wb") as target:
                    shutil.copyfileobj(bundle_stream, target, length=1024 * 1024)
                require(bundle.stat().st_size > 0, "project_preparation_bundle_missing")
                if root.exists():
                    git(root, "bundle", "verify", str(bundle), phase="bundle_verify")
                    git(root, "fetch", "--no-tags", "--no-write-fetch-head",
                        "--no-recurse-submodules", str(bundle), "refs/heads/source", phase="bundle_import")
                else:
                    base = no_links(root.parent)
                    # Les nouvelles destinations sont dans le seul espace géré.
                    managed = Path.home() / ".local/share/bridget-projects" / result["project_id"]
                    require(base == managed, "project_preparation_destination_missing")
                    private_dir(base)
                    marker = base / "source.json"
                    expected = {k: mapping[k] for k in
                                ("source_host", "source_root", "remote_root", "expected_git_identity")}
                    if marker.exists():
                        require(json.loads(private_read(marker)) == expected,
                                "project_preparation_destination_occupied")
                    else:
                        require(not any(base.iterdir()), "project_preparation_destination_occupied")
                        atomic_json(marker, expected)
                    mirror = no_links(mapping["expected_git_identity"])
                    if not mirror.exists():
                        git(base, "clone", "--bare", str(bundle), str(mirror), phase="bundle_import")
                    else:
                        git(mirror, "fetch", "--no-tags", str(bundle), "refs/heads/source", phase="bundle_import")
                    require(commit_present(mirror, commit), "project_preparation_commit_missing")
                    # Clone dans un dossier temporaire puis publication. Aucune suppression
                    # d'un checkout étranger ou réinitialisation d'un checkout existant.
                    with tempfile.TemporaryDirectory(prefix=".checkout-", dir=base) as stage:
                        checkout = Path(stage) / "repository"
                        git(base, "clone", "--no-checkout", str(mirror), str(checkout), phase="bundle_import")
                        git(checkout, "checkout", "--detach", commit, phase="bundle_import")
                        require(not root.exists(), "project_preparation_destination_occupied")
                        os.rename(checkout, root)
                require(commit_present(root, commit), "project_preparation_commit_missing")
        prepare_linked_checkout(root, mapping, request, registry_path)
        if mapping.get("additional_root"):
            extra = mapping["additional_root"]
            private_dir(extra["remote_root"])
        publish_mapping(registry_path, request, result["project_id"], mapping)
        result["needs_bundle"] = False
        result["have_commit"] = None
        return result

def installed_links(namespace):
    home = Path.home()
    base = home / ".local/share/bridget-federation"
    if not base.is_dir():
        return []
    links = []
    for directory in sorted(base.iterdir()):
        try:
            private_dir(directory)
            raw = private_read(directory / "config")
            values = dict(line.split("=", 1) for line in raw.decode().splitlines())
            receipt = dict(line.split("=", 1) for line in
                           private_read(directory / "receipt").decode().splitlines())
            require(values["version"] == "1" and values["root"] == str(namespace),
                    "project_preparation_link_namespace")
            require(receipt["config"] == str(directory / "config")
                    and receipt["config_sha256"] == digest(raw),
                    "project_preparation_link_changed")
            for name in ("runner", "service"):
                target = no_links(receipt[name])
                require(target.is_file() and target.stat().st_uid == os.geteuid()
                        and digest(target.read_bytes()) == receipt[name + "_sha256"],
                        "project_preparation_link_changed")
            require(re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9.-]*", values["host"])
                    and re.fullmatch(r"[a-zA-Z_][a-zA-Z0-9_-]*", values["user"])
                    and 1 <= int(values["port"]) <= 65535,
                    "project_preparation_link_invalid")
            for name in ("identity", "known_hosts", "remote_root"):
                absolute(values[name])
            links.append(values)
        except (OSError, ValueError, KeyError, Refused):
            continue
    return links

def ssh(link, request, operation, bundle=None):
    encoded = base64.b64encode(json.dumps(dict(request, operation=operation,
        registry=str(Path(link["remote_root"]) / "agents.json"))).encode()).decode()
    code = base64.b64encode(Path(__file__).read_bytes()).decode()
    remote = "python3 -c 'import base64;exec(base64.b64decode(\"" + code + "\"))' --receive " + encoded
    command = ["ssh", "-T", "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
               "-o", "ConnectTimeout=10", "-o", "ServerAliveInterval=10",
               "-o", "ServerAliveCountMax=3", "-o", "IdentitiesOnly=yes",
               "-o", "UserKnownHostsFile=" + link["known_hosts"],
               "-i", link["identity"], "-p", link["port"],
               link["user"] + "@" + link["host"], remote]
    with (open(bundle, "rb") if bundle else open(os.devnull, "rb")) as stream:
        try:
            answer = subprocess.run(command, stdin=stream, stdout=subprocess.PIPE,
                                    stderr=subprocess.PIPE, timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            code = {"inspect": "project_preparation_ssh_inspect_timeout",
                    "prepare": "project_preparation_ssh_prepare_timeout"}[operation]
            raise Refused(code) from None
    require(answer.returncode == 0 and len(answer.stdout) <= MAX_JSON,
            "project_preparation_transport_failed")
    result = json.loads(answer.stdout)
    require(result.get("ok") is True, result.get("error", "project_preparation_remote_failed"))
    return result["result"]

def export_bundle(source, commit, destination, have_commit=None):
    # Référence dans un dépôt temporaire : le dépôt parent reste en lecture seule.
    object_path = git(source, "rev-parse", "--path-format=absolute", "--git-path", "objects").stdout.decode().strip()
    absolute(object_path)
    scratch = destination.parent / "export.git"
    git(destination.parent, "init", "--bare", str(scratch))
    (scratch / "objects/info/alternates").write_text(object_path + "\n")
    git(scratch, "update-ref", "refs/heads/source", commit)
    exclude = []
    if have_commit and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", have_commit) and commit_present(source, have_commit):
        if git(source, "merge-base", "--is-ancestor", have_commit, commit, check=False).returncode == 0:
            exclude = ["^" + have_commit]
    git(scratch, "bundle", "create", str(destination), "refs/heads/source", *exclude, phase="bundle_export")

def prepare(request):
    registry_path = absolute(request["registry"])
    source = absolute(request["source_root"])
    cwd = absolute(request["cwd"])
    require(source.resolve() == source and cwd.resolve() == cwd and contained(cwd, source),
            "cwd_outside_parent_project")
    commit = request.get("commit")
    if commit:
        require(re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", commit) is not None,
                "expected_base_commit_invalid")
        top = Path(git(cwd, "rev-parse", "--show-toplevel").stdout.decode().strip())
        require(top == source or (contained(top, source)
                and no_links(top / ".git").is_file()
                and git_common_dir(top) == git_common_dir(source)),
                "project_preparation_git_root_outside_parent")
        source = top
        require(commit_present(source, commit), "expected_base_commit_mismatch")
    with locked(registry_path.parent / "project-preparation.lock"):
        _, registry = registry_read(registry_path)
        selected = registry.get("machines", {}).get(request["machine_id"], {})
        require(selected.get("enabled") is True, "machine_disabled")
        request["authority"] = {key: selected[key] for key in
                               ("worker_id", "provisioning_revision", "credential_fingerprint")}
        # La source additionnelle provient uniquement des correspondances déjà
        # approuvées pour cet hôte. Ses contenus ne sont pas transférés.
        extras = {json.dumps(m["additional_root"], sort_keys=True) for m in
                  selected.get("mappings", {}).values()
                  if m["source_host"] == request["origin_host"] and m.get("additional_root")}
        if extras:
            roots = {json.loads(e)["source_root"] for e in extras}
            if len(roots) == 1:
                extra = json.loads(next(iter(extras)))
                info = absolute(extra["source_root"]).stat()
                require(info.st_uid == extra["source_uid"]
                        and stat.S_IMODE(info.st_mode) == extra["source_mode"],
                        "project_preparation_additional_root_changed")
                request["additional_root"] = {k: extra[k] for k in
                                              ("source_root", "source_uid", "source_mode", "access")}
        # Résoudre côté source avant de contacter la destination. Une ancienne
        # correspondance divergente ne peut pas créer un second projet.
        local_key, local_mapping = select_mapping(selected, request)
        request.pop("linked_checkout", None)
        request.pop("bundle_base", None)
        request.pop("checkout_root", None)
        if commit:
            request["checkout_root"] = str(source)
        mapped_source = absolute(local_mapping["source_root"])
        if commit and source != mapped_source:
            # Comme T3, l'identite du depot est son repertoire Git commun.
            # Le chemin lexical seul confond un worktree lie et un autre depot.
            require(contained(source, mapped_source)
                    and no_links(source / ".git").is_file()
                    and git_common_dir(source) == git_common_dir(mapped_source),
                    "project_preparation_git_identity_mismatch")
            request["linked_checkout"] = str(source.relative_to(mapped_source))
        if local_key in selected.get("mappings", {}):
            request["project_id"] = local_key
        matches = []
        for link in installed_links(registry_path.parent):
            try:
                result = ssh(link, request, "inspect")
                matches.append((link, result))
            except Refused as error:
                if str(error) != "project_preparation_machine_mismatch":
                    raise
        require(len(matches) == 1, "project_preparation_link_unavailable")
        link, result = matches[0]
        # Le maître et le worker doivent déjà partager toute correspondance explicite.
        local = selected.get("mappings", {}).get(result["project_id"])
        require(local is None or local == result["mapping"], "project_preparation_mapping_changed")
        request.update(mapping=result["mapping"], resolved_project_id=result["project_id"])
        have = result.get("have_commit")
        if result["needs_bundle"] and have and re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", have) and commit_present(source, have):
            common = git(source, "merge-base", have, commit, check=False, phase="bundle_base")
            require(common.returncode in (0, 1), "project_preparation_git_failed")
            base = common.stdout.decode().strip()
            if common.returncode == 0 and base != have:
                require(re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", base),
                        "project_preparation_bundle_base_invalid")
                request["bundle_base"] = base
                confirmed = ssh(link, request, "inspect")
                require(confirmed["mapping"] == result["mapping"]
                        and confirmed["project_id"] == result["project_id"],
                        "project_preparation_mapping_changed")
                require(not confirmed["needs_bundle"] or confirmed.get("have_commit") == base,
                        "project_preparation_bundle_base_missing")
                result = confirmed
        if result["needs_bundle"]:
            require(commit is not None, "project_preparation_git_required")
            with tempfile.TemporaryDirectory(prefix="bridget-export-") as directory:
                bundle = Path(directory) / "source.bundle"
                export_bundle(source, commit, bundle, result.get("have_commit"))
                result = ssh(link, request, "prepare", bundle)
        else:
            result = ssh(link, request, "prepare")
        publish_mapping(registry_path, request, result["project_id"], result["mapping"])
        return result

def remote_registry(request):
    expected = absolute(request["registry"])
    # Le namespace du tunnel et le registre du worker peuvent être distincts.
    # Lire le lancement du worker local, sans identité fournie par un agent.
    found = set()
    proc = Path("/proc")
    if proc.is_dir():
        for process in proc.iterdir():
            if not process.name.isdigit():
                continue
            try:
                if process.stat().st_uid != os.geteuid():
                    continue
                argv = (process / "cmdline").read_bytes().decode().split("\0")
                if len(argv) < 3 or Path(argv[0]).name != "bridget" or argv[1] != "worker":
                    continue
                options = dict(zip(argv[2::2], argv[3::2]))
                if options.get("--machine-id") != request["machine_id"] or options.get("--worker-id") != request["authority"]["worker_id"]:
                    continue
                candidate = absolute(options["--registry-file"])
                _, config = registry_read(candidate)
                machine(config, request)
                found.add(candidate)
            except (OSError, ValueError, KeyError, Refused):
                continue
    require(len(found) <= 1, "project_preparation_registry_ambiguous")
    if found:
        return found.pop()
    # Le registre du maître peut décrire toute la flotte : sa présence dans le
    # tunnel ne prouve pas que cette machine est locale. Le registre du worker
    # local prime et un refus ne doit jamais retomber sur le registre global.
    candidate = Path.home() / ".config/bridget/agents.json"
    if candidate.exists():
        _, config = registry_read(candidate)
        machine(config, request)
        return candidate
    require(expected.exists(), "project_preparation_registry_missing")
    _, config = registry_read(expected)
    machine(config, request)
    return expected

def main():
    try:
        if len(sys.argv) == 3 and sys.argv[1] == "--receive":
            request = json.loads(base64.b64decode(sys.argv[2]))
            registry_path = remote_registry(request)
            if request["operation"] == "inspect":
                result = remote_inspect(request, registry_path)
            else:
                require(request["operation"] == "prepare", "project_preparation_operation")
                result = remote_receive(request, registry_path, sys.stdin.buffer)
        else:
            request = json.loads(sys.stdin.buffer.read(MAX_JSON))
            result = prepare(request)
        print(json.dumps(dict(ok=True, result=result)))
    except Refused as error:
        print(json.dumps(dict(ok=False, error=str(error))))
    except subprocess.TimeoutExpired:
        print(json.dumps(dict(ok=False, error="project_preparation_timeout")))
    except OSError as error:
        # Seulement l'errno constaté, jamais le texte libre ni un chemin.
        code = {errno.ENOSPC: "project_preparation_disk_full",
                errno.EDQUOT: "project_preparation_disk_quota"}.get(
                    error.errno, "project_preparation_failed")
        print(json.dumps(dict(ok=False, error=code)))
    except (KeyError, ValueError, TypeError):
        print(json.dumps(dict(ok=False, error="project_preparation_failed")))

if __name__ == "__main__":
    main()
