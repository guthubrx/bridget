#!/usr/bin/env python3
"""Bridget updates: official stable assets, one host lock, bounded activation.
Bundled in the binary. No URL, service command or path is accepted from mobile.
"""
import fcntl
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import selectors
import stat
import sys
import subprocess
import tarfile
import tempfile
import time
import urllib.error
import urllib.parse
import urllib.request
import uuid

API = "https://api.github.com/repos/guthubrx/bridget/releases/latest"
BASE = "https://github.com/guthubrx/bridget/releases/download/"
MAX_ARCHIVE = 256 * 1024 * 1024
MAX_BINARY = 512 * 1024 * 1024
CHECK_INTERVAL = 6 * 3600
SERVICES = {"linux": {"core": "bridget-core.service", "mobile": "bridget-mobile.service", "worker": "bridget-worker-cartae.service"},
            "darwin": {"t3": "com.bridget.t3"}}


class Refused(Exception):
    pass


def version(value):
    if not isinstance(value, str) or not re.fullmatch(r"(0|[1-9][0-9]{0,2})\.(0|[1-9][0-9]{0,2})\.(0|[1-9][0-9]{0,2})", value):
        raise Refused("update_version_invalid")
    return tuple(map(int, value.split(".")))


def private_json(path, default=None):
    if not path.exists() and not path.is_symlink():
        return default
    st = path.lstat()
    if not stat.S_ISREG(st.st_mode) or st.st_uid != os.getuid() or st.st_mode & 0o077 or st.st_size > 65536:
        raise Refused("update_state_unavailable")
    return json.loads(path.read_bytes())


def atomic_json(path, value):
    raw = json.dumps(value, separators=(",", ":")).encode()
    if len(raw) > 65536:
        raise Refused("update_state_unavailable")
    fd, temp = tempfile.mkstemp(prefix=".update-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as f:
            f.write(raw)
            f.flush()
            os.fsync(f.fileno())
        os.replace(temp, path)
        sync_dir(path.parent)
    finally:
        Path(temp).unlink(missing_ok=True)


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def allowed_url(url):
    p = urllib.parse.urlsplit(url)
    return p.scheme == "https" and not p.username and not p.password and not p.fragment and p.port in (None, 443) and p.hostname in ("github.com", "release-assets.githubusercontent.com")


class Redirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        if not allowed_url(newurl):
            raise Refused("update_download_origin_invalid")
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def download(url, output, maximum, budget=180):
    if url != API and not allowed_url(url):
        raise Refused("update_download_origin_invalid")
    opener = urllib.request.build_opener(Redirect())
    deadline = time.monotonic() + budget
    req = urllib.request.Request(url, headers={"Accept": "application/vnd.github+json" if url == API else "application/octet-stream",
                                             "User-Agent": "Bridget-Updater"})
    digest = hashlib.sha256()
    count = 0
    with opener.open(req, timeout=10) as response:
        if response.status != 200:
            raise Refused("update_download_unavailable")
        while True:
            if time.monotonic() >= deadline:
                raise Refused("update_download_timeout")
            chunk = response.read(32768)
            if not chunk:
                break
            count += len(chunk)
            if count > maximum:
                raise Refused("update_download_too_large")
            digest.update(chunk)
            output.write(chunk)
    return count, digest.hexdigest()


def candidate(metadata, installed, target):
    tag = metadata.get("tag_name", "")
    if metadata.get("draft") is not False or metadata.get("prerelease") is not False or not tag.startswith("v"):
        raise Refused("update_release_invalid")
    release = tag[1:]
    if version(release) <= version(installed):
        return None
    stem = "bridget-" + release + "-" + target
    name = stem + ".tar.gz"
    assets = metadata.get("assets")
    if not isinstance(assets, list) or len(assets) > 64:
        raise Refused("update_release_invalid")
    matches = [a for a in assets if a.get("name") == name]
    if len(matches) != 1:
        raise Refused("update_platform_unavailable")
    asset = matches[0]
    digest = asset.get("digest") or ""
    if not re.fullmatch("sha256:[0-9a-f]{64}", digest) or type(asset.get("size")) is not int or not 0 < asset["size"] <= MAX_ARCHIVE:
        raise Refused("update_integrity_unavailable")
    if asset.get("browser_download_url") != BASE + tag + "/" + name:
        raise Refused("update_download_origin_invalid")
    return {"version": release, "url": asset["browser_download_url"], "size": asset["size"],
            "sha256": digest[7:], "member": stem + "/bridget"}


def file_digest(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(32768), b""):
            digest.update(block)
    return digest.hexdigest()


def binary_version(path):
    response = subprocess.run([str(path), "--version"], check=True, capture_output=True, timeout=10)
    if len(response.stdout) > 4096:
        raise Refused("update_binary_invalid")
    match = re.search(rb"\bbridget\s+([0-9]+\.[0-9]+\.[0-9]+)\b", response.stdout)
    if not match:
        raise Refused("update_binary_invalid")
    result = match.group(1).decode()
    version(result)
    return result


def extract_binary(archive, member, destination):
    # Equivalent to install.sh: extract exactly the known regular member, no extractall.
    found = False
    with tarfile.open(archive, "r:gz") as package:
        for index, entry in enumerate(package):
            if index >= 64 or entry.size > MAX_BINARY or entry.size < 0:
                raise Refused("update_archive_invalid")
            if entry.name != member:
                continue
            if found or not entry.isfile() or not 0 < entry.size <= MAX_BINARY:
                raise Refused("update_archive_invalid")
            found = True
            with package.extractfile(entry) as source, open(destination, "xb") as output:
                shutil.copyfileobj(source, output, 32768)
                output.flush()
                os.fsync(output.fileno())
            if destination.stat().st_size != entry.size:
                raise Refused("update_archive_invalid")
    if not found:
        raise Refused("update_archive_invalid")
    destination.chmod(0o700)


def rpc(binary, *args):
    # The daemon negotiates human administration and checks the peer UID.
    result = subprocess.run([str(binary), "updates", *args], capture_output=True, timeout=12)
    raw = result.stdout if result.returncode == 0 else result.stderr
    if len(raw) > 65536:
        raise Refused("update_response_invalid")
    try:
        value = json.loads(raw)
    except (ValueError, UnicodeError):
        raise Refused("update_daemon_unavailable")
    if result.returncode or value.get("status") != "ok":
        raise Refused(value.get("code", "update_daemon_unavailable"))
    return value


def switch_link(link, target):
    temp = link.with_name(".update-link-" + uuid.uuid4().hex)
    try:
        temp.symlink_to(target)
        os.replace(temp, link)
        sync_dir(link.parent)
    finally:
        temp.unlink(missing_ok=True)


def service_command(system, action, service):
    name = SERVICES[system][service]
    if system == "linux":
        return ["systemctl", "--user", action, name]
    if action == "restart":
        return ["launchctl", "kickstart", "-k", "gui/" + str(os.getuid()) + "/" + name]
    return ["launchctl", "print", "gui/" + str(os.getuid()) + "/" + name]


def restart(system, services, deadline):
    for service in services:
        remaining = min(15, deadline - time.monotonic())
        if remaining <= 0:
            raise Refused("update_activation_timeout")
        subprocess.run(service_command(system, "restart", service), check=True, capture_output=True, timeout=remaining)


def healthy(system, services, binary, expected, deadline):
    while time.monotonic() < deadline:
        try:
            for service in services:
                command = service_command(system, "is-active" if system == "linux" else "print", service)
                result = subprocess.run(command, check=True, capture_output=True, timeout=5)
                if system == "linux" and result.stdout.strip() != b"active":
                    raise Refused("update_service_unavailable")
                if system == "darwin" and b"state = running" not in result.stdout:
                    raise Refused("update_service_unavailable")
            for service in services:
                name = SERVICES[system][service]
                if system == "linux":
                    pid = subprocess.run(["systemctl", "--user", "show", "-p", "MainPID", "--value", name], check=True, capture_output=True, timeout=5).stdout.decode().strip()
                    if not pid.isdigit() or Path("/proc", pid, "exe").resolve() != Path(binary).resolve():
                        raise Refused("update_loaded_version_mismatch")
                else:
                    output = subprocess.run(service_command(system, "print", service), check=True, capture_output=True, timeout=5).stdout
                    match = re.search(rb"\bpid = ([0-9]+)", output)
                    if not match:
                        raise Refused("update_loaded_version_mismatch")
                    files = subprocess.run(["lsof", "-n", "-P", "-a", "-p", match.group(1).decode(), "-d", "txt", "-F", "n"], capture_output=True, timeout=5).stdout.decode()
                    if "n" + str(Path(binary).resolve()) not in files.splitlines():
                        raise Refused("update_loaded_version_mismatch")
            info = rpc(binary, "status")
            if "core" in services and info.get("core_version") != expected:
                raise Refused("update_loaded_version_mismatch")
            return True
        except (Refused, OSError, subprocess.SubprocessError):
            time.sleep(1)
    return False


def process_inventory():
    # Bounded pipes; TERM concerns only the lsof child created here, never another job.
    child = subprocess.Popen(["lsof", "-n", "-P", "-F", "n"], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    buffers = {child.stdout: bytearray(), child.stderr: bytearray()}
    deadline = time.monotonic() + 30
    try:
        with selectors.DefaultSelector() as selector:
            for stream in buffers:
                selector.register(stream, selectors.EVENT_READ)
            while selector.get_map():
                if time.monotonic() >= deadline:
                    return None
                for key, _ in selector.select(0.2):
                    data = os.read(key.fileobj.fileno(), 32768)
                    if not data:
                        selector.unregister(key.fileobj)
                    else:
                        buffers[key.fileobj].extend(data)
                        if len(buffers[child.stderr]) or sum(map(len, buffers.values())) > 8 * 1024 * 1024:
                            return None
        if child.wait(timeout=2) != 0 or not buffers[child.stdout]:
            return None
        return bytes(buffers[child.stdout])
    finally:
        if child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=2)
            except subprocess.TimeoutExpired:
                pass
        child.stdout.close()
        child.stderr.close()


class Updater:
    def __init__(self, home, root):
        self.home, self.root = Path(home), Path(root)
        self.system = platform.system().lower()
        config = private_json(self.root / "update-host.json")
        if not config or set(config) != {"host", "services"} or self.system not in SERVICES:
            raise Refused("update_host_not_configured")
        if not re.fullmatch(r"[a-zA-Z0-9_.-]{1,128}", config["host"]):
            raise Refused("update_host_not_configured")
        services = config["services"]
        if not isinstance(services, list) or not services or len(set(services)) != len(services) or any(s not in SERVICES[self.system] for s in services):
            raise Refused("update_host_not_configured")
        self.host, self.services = config["host"], services
        architecture = {"arm64": "arm64", "aarch64": "arm64", "x86_64": "x86_64"}.get(platform.machine())
        if not architecture:
            raise Refused("update_platform_unavailable")
        self.target = self.system + "-" + architecture
        self.link = self.home / ".local/bin/bridget"
        if not self.link.is_symlink() or not self.link.resolve().is_file():
            raise Refused("update_stable_link_required")
        self.binary = self.link
        self.original = self.link.resolve()
        self.state_path = self.root / "update-host-state.json"
        self.state = private_json(self.state_path, {})
        self.install_root = self.home / ".local/lib/bridget-communication"

    def save(self, **values):
        self.state.update(values)
        atomic_json(self.state_path, self.state)

    def report(self, stage, available=None):
        installed = binary_version(self.link)
        self.save(stage=stage, installed=installed, available=available, observed_at=int(time.time()))
        fd, name = tempfile.mkstemp(prefix=".update-report-", dir=self.root)
        os.close(fd)
        try:
            path = Path(name)
            atomic_json(path, {"action": "report", "host": self.host, "installed": installed, "available": available, "stage": stage})
            rpc(self.binary, "report", str(path))
        finally:
            Path(name).unlink(missing_ok=True)

    def step(self, force=False):
        policy = rpc(self.binary, "status")
        target = policy.get("install_target")
        if target is not None:
            version(target)
        manual = target is not None and version(target) > version(binary_version(self.link))
        requested = policy.get("check_generation", 0) != self.state.get("check_generation", 0)
        automatic = policy.get("automatic", False)
        if not automatic and not manual and not requested and not force:
            self.report("disabled", self.state.get("available"))
            return
        installed = binary_version(self.link)
        pending = self.state.get("candidate")
        if force or requested or time.time() - self.state.get("checked_at", 0) >= CHECK_INTERVAL:
            # Record attempt before I/O, so an outage does not cause a 60-second download loop.
            self.save(checked_at=int(time.time()), check_generation=policy.get("check_generation", 0))
            buffer = io.BytesIO()
            download(API, buffer, 262144, 30)
            pending = candidate(json.loads(buffer.getvalue()), installed, self.target)
            self.save(candidate=pending)
        if not pending or version(pending["version"]) <= version(installed):
            if healthy(self.system, self.services, self.link, installed, time.monotonic() + 2):
                self.report("up_to_date")
            elif automatic or manual:
                # A newer executable on disk does not prove that every service loaded it.
                self.report("ready", installed)
                self.activate(self.link.resolve(), installed, manual)
            else:
                self.report("waiting_idle", installed)
            return
        # Revalidate persisted release metadata, never trust an arbitrary persisted URL.
        v = pending["version"]
        expected_member = "bridget-" + v + "-" + self.target + "/bridget"
        if pending["member"] != expected_member or pending["url"] != BASE + "v" + v + "/bridget-" + v + "-" + self.target + ".tar.gz":
            raise Refused("update_release_invalid")
        self.report("available", v)
        manual = manual and target == v
        if not automatic and not manual:
            return
        directory = self.install_root / ("v" + v + "-auto")
        executable = directory / "bin/bridget"
        if not executable.exists():
            self.report("downloading", v)
            directory.mkdir(parents=True, mode=0o700, exist_ok=True)
            if directory.is_symlink():
                raise Refused("update_install_path_invalid")
            with tempfile.TemporaryDirectory(prefix=".prepare-", dir=directory) as staging:
                staging = Path(staging)
                archive = staging / "package.tar.gz"
                with archive.open("xb") as f:
                    size, digest = download(pending["url"], f, MAX_ARCHIVE)
                    f.flush()
                    os.fsync(f.fileno())
                if size != pending["size"] or digest != pending["sha256"]:
                    raise Refused("update_integrity_failed")
                staged = staging / "bridget"
                extract_binary(archive, pending["member"], staged)
                if binary_version(staged) != v:
                    raise Refused("update_binary_invalid")
                if self.system == "darwin":
                    subprocess.run(["codesign", "--verify", "--strict", str(staged)], check=True, capture_output=True, timeout=15)
                executable.parent.mkdir(mode=0o700, exist_ok=True)
                os.replace(staged, executable)
                sync_dir(executable.parent)
                atomic_json(directory / "receipt.json", {"version": v, "sha256": file_digest(executable), "archive": digest})
        receipt = private_json(directory / "receipt.json")
        if not receipt or file_digest(executable) != receipt["sha256"] or binary_version(executable) != v:
            raise Refused("update_integrity_failed")
        if self.state.get("failed_version") == v:
            self.report("rolled_back", v)
            return
        self.report("ready", v)
        self.activate(executable, v, manual)

    def activate(self, executable, v, manual=False):
        current = rpc(self.binary, "status")
        if not current.get("automatic", False) and not (manual and current.get("install_target") == v):
            self.report("disabled", v)
            return
        old = self.link.resolve()
        if old == executable and self.state.get("rollback"):
            previous = Path(self.state["rollback"])
            if not previous.is_relative_to(self.install_root) or not previous.is_file():
                raise Refused("update_rollback_unavailable")
            old = previous
        token = str(uuid.uuid4())
        try:
            lease = rpc(self.binary, "prepare", token)
        except Refused as error:
            if str(error) in ("update_missions_active_or_unconfirmed", "update_operations_active", "update_activation_active", "update_disabled"):
                self.report("waiting_idle", v)
                return
            raise
        # All service actions + verification + rollback finish before the 180-second lease.
        deadline = min(time.monotonic() + 110, time.monotonic() + lease["expires_at"] - time.time() - 60)
        switched = False
        try:
            current = rpc(self.binary, "status")
            if not current.get("automatic", False) and not (manual and current.get("install_target") == v):
                return
            self.report("installing", v)
            self.save(rollback=str(old), activation={"token": token, "old": str(old), "new": str(executable), "version": v})
            switch_link(self.link, executable)
            switched = True
            restart(self.system, self.services, deadline)
            if not healthy(self.system, self.services, self.link, v, min(deadline, time.monotonic() + 25)):
                raise Refused("update_health_failed")
            self.report("active", v)
            self.save(activation=None)
            # Releasing the barrier is the final activation action; no rollback after this.
            switched = False
            rpc(self.binary, "finish", token)
        except Exception:
            if switched:
                switch_link(self.link, old)
                restart(self.system, self.services, time.monotonic() + 35)
                if not healthy(self.system, self.services, self.link, binary_version(old), time.monotonic() + 15):
                    raise Refused("update_rollback_unconfirmed")
                self.save(activation=None, failed_version=v)
                self.report("rolled_back", v)
            raise
        finally:
            try:
                rpc(self.binary, "finish", token)
            except (Refused, OSError, subprocess.SubprocessError):
                pass
        try:
            self.cleanup()
        except Exception:
            self.save(cleanup="inventory_unavailable")

    def cleanup(self):
        # Only directories created by this updater; active binaries and one rollback survive.
        keep = {self.link.resolve(), Path(self.state.get("rollback", str(self.link.resolve())))}
        output = process_inventory()
        if output is None:
            self.save(cleanup="process_inventory_unknown")
            return
        opened = [Path(line[1:].removesuffix(" (deleted)")) for line in output.decode(errors="strict").splitlines() if line.startswith("n/")]
        for directory in self.install_root.glob("v*-auto"):
            executable = directory / "bin/bridget"
            if directory.is_symlink() or executable in keep or any(p == directory or directory in p.parents for p in opened):
                continue
            receipt = private_json(directory / "receipt.json")
            expected = {directory / "bin", executable, directory / "receipt.json"}
            actual = set(directory.rglob("*"))
            if not receipt or actual != expected or any(p.is_symlink() for p in actual):
                continue
            if file_digest(executable) != receipt.get("sha256"):
                continue
            executable.unlink()
            (directory / "receipt.json").unlink()
            (directory / "bin").rmdir()
            directory.rmdir()
        self.save(cleanup="checked")

    def recover(self):
        action = self.state.get("activation")
        if not action:
            return
        old = Path(action["old"])
        if not old.is_relative_to(self.install_root) or not old.is_file():
            raise Refused("update_rollback_unavailable")
        # Recovery needs the same durable lease; no restart after it expired.
        rpc(self.binary, "prepare", action["token"])
        switch_link(self.link, old)
        restart(self.system, self.services, time.monotonic() + 35)
        if not healthy(self.system, self.services, self.link, binary_version(old), time.monotonic() + 20):
            raise Refused("update_rollback_unconfirmed")
        rpc(self.binary, "finish", action["token"])
        self.save(activation=None)
        self.report("rolled_back", action["version"])


def main():
    os.umask(0o077)
    home = Path.home()
    root = Path(os.environ.get("BRIDGET_HOME", str(home / ".cache/bridget-core")))
    if not root.is_dir() or root.is_symlink() or root.stat().st_uid != os.getuid() or root.stat().st_mode & 0o077:
        raise Refused("update_root_invalid")
    lock_path = root / "update-host.lock"
    fd = os.open(lock_path, os.O_CREAT | os.O_RDWR | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, "w") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return
        updater = Updater(home, root)
        updater.recover()
        watch = os.environ.get("BRIDGET_UPDATE_WATCH") == "1"
        while True:
            try:
                updater.step(force=not watch)
            except Exception as error:
                code = str(error) if isinstance(error, Refused) else "update_unavailable"
                updater.save(error=code, stage="error", observed_at=int(time.time()))
                try:
                    updater.report("error", updater.state.get("available"))
                except Exception:
                    pass
                print(json.dumps({"status": "deferred", "code": code}), flush=True)
            if not watch:
                return
            if updater.link.resolve() != updater.original:
                # Reload the updater implementation from the newly installed binary.
                fcntl.flock(lock, fcntl.LOCK_UN)
                os.execv(str(updater.link), [str(updater.link), "updates", "watch"])
            time.sleep(60)


if __name__ == "__main__":
    main()
