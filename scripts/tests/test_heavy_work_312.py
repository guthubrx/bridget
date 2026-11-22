"""Tests Python 3.9+ pour la spécification 312 (Lot 1a — travaux lourds hors poste)."""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "build.py"
spec = importlib.util.spec_from_file_location("bridget_build_312", SCRIPT)
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class HeavyWork312Tests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="bridget-hw312-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()

    def _init_git_repo_with_remote(self):
        bare = self.root / "remote.git"
        subprocess.run(["git", "init", "--bare", "-b", "main", str(bare)], check=True, capture_output=True)
        repo = self.root / "repo"
        repo.mkdir()
        env = {
            **os.environ,
            "GIT_AUTHOR_NAME": "Test",
            "GIT_AUTHOR_EMAIL": "test@example.com",
            "GIT_COMMITTER_NAME": "Test",
            "GIT_COMMITTER_EMAIL": "test@example.com",
        }
        subprocess.run(["git", "-C", str(repo), "init", "-b", "main"], check=True, capture_output=True, env=env)
        subprocess.run(["git", "-C", str(repo), "remote", "add", "origin", str(bare)], check=True, capture_output=True, env=env)
        (repo / "README.md").write_text("hello\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(repo), "add", "README.md"], check=True, capture_output=True, env=env)
        subprocess.run(["git", "-C", str(repo), "commit", "-m", "root"], check=True, capture_output=True, env=env)
        subprocess.run(["git", "-C", str(repo), "push", "-u", "origin", "main"], check=True, capture_output=True, env=env)
        return repo, bare, env

    def test_rmtree_no_follow_removes_tree_and_refuses_symlink(self):
        parent = self.root / "parent"
        parent.mkdir()
        victim = self.root / "outside"
        victim.mkdir()
        (victim / "keep.txt").write_text("precious", encoding="utf-8")

        target = parent / "deps"
        (target / "nested").mkdir(parents=True)
        (target / "nested" / "file.o").write_bytes(b"abc")
        (target / "link_to_victim").symlink_to(victim, target_is_directory=True)

        pfd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            build.rmtree_no_follow(pfd, "deps")
        finally:
            os.close(pfd)

        self.assertFalse(target.exists())
        self.assertEqual((victim / "keep.txt").read_text(encoding="utf-8"), "precious")

        # Un dossier racine qui est un lien symbolique est refusé par O_NOFOLLOW.
        (parent / "sym_dir").symlink_to(victim, target_is_directory=True)
        pfd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            with self.assertRaises(OSError):
                build.rmtree_no_follow(pfd, "sym_dir")
        finally:
            os.close(pfd)
        self.assertEqual((victim / "keep.txt").read_text(encoding="utf-8"), "precious")

    def test_block_digest_matches_sha256(self):
        sample = self.root / "blob.bin"
        payload = b"0123456789abcdef" * 10000
        sample.write_bytes(payload)
        self.assertEqual(build.digest(sample), hashlib.sha256(payload).digest())

    def test_canonical_owner_and_legacy_dual_recognition_in_owned_staging(self):
        repo, _bare, _env = self._init_git_repo_with_remote()
        cache = self.root / "cache"
        cache.mkdir(mode=0o700)
        root_sha = build.canonical_owner_id(repo)
        self.assertIsNotNone(root_sha)
        self.assertEqual(len(root_sha), 40)
        legacy = build.owner_id(repo)
        self.assertNotEqual(root_sha, legacy)

        stage_canon = cache / "bridget-build-canon"
        stage_canon.mkdir(mode=0o700)
        (stage_canon / build.OWNER).write_text(json.dumps({"version": 1, "repo": root_sha}), encoding="utf-8")

        stage_legacy = cache / "bridget-build-legacy"
        stage_legacy.mkdir(mode=0o700)
        (stage_legacy / build.OWNER).write_text(json.dumps({"version": 1, "repo": legacy}), encoding="utf-8")

        stage_other = cache / "bridget-build-other"
        stage_other.mkdir(mode=0o700)
        (stage_other / build.OWNER).write_text(json.dumps({"version": 1, "repo": "f" * 40}), encoding="utf-8")

        with patch.object(build, "BUILD_SETTINGS", self.root / "absent.json"), \
             patch.object(build, "staging_parent", return_value=cache):
            found = set(build.owned_staging(repo))
            self.assertEqual(found, {stage_canon, stage_legacy})
            # remote_cache_root sans build.json -> staging_parent()
            self.assertEqual(build.remote_cache_root(), cache)

        # Avec BRIDGET_BUILD_OWNER sur un dossier sans .git
        no_git = self.root / "extracted"
        no_git.mkdir()
        with patch.dict(os.environ, {"BRIDGET_BUILD_OWNER": root_sha}):
            self.assertEqual(build.canonical_owner_id(no_git), root_sha)

        # Avec build.json invalide -> refus sans repli
        bad_settings = self.root / "bad-build.json"
        bad_settings.write_text("{invalid", encoding="utf-8")
        bad_settings.chmod(0o600)
        with patch.object(build, "BUILD_SETTINGS", bad_settings):
            with self.assertRaises(ValueError):
                build.remote_cache_root()

    def test_local_target_guard_and_foreign_output_path(self):
        ok_mac, _ = build.check_local_target_compatible("macos-arm64", host_platform="darwin", host_machine="arm64")
        self.assertTrue(ok_mac)

        ok_linux_on_mac, msg = build.check_local_target_compatible(
            "linux-x86_64-glibc2.36", host_platform="darwin", host_machine="arm64"
        )
        self.assertFalse(ok_linux_on_mac)
        self.assertIn("--host", msg)

        ok_old_glibc, _ = build.check_local_target_compatible(
            "linux-x86_64-glibc2.36", host_platform="linux", host_machine="x86_64", host_glibc=(2, 36)
        )
        self.assertTrue(ok_old_glibc)

        ok_new_glibc, msg_glibc = build.check_local_target_compatible(
            "linux-x86_64-glibc2.36", host_platform="linux", host_machine="x86_64", host_glibc=(2, 39)
        )
        self.assertFalse(ok_new_glibc)
        self.assertIn("2.39 > 2.36", msg_glibc)

        # Chemin binaire natif vs cible étrangère (§7.12)
        target_dir = self.root / "target"
        with patch.dict(os.environ, {"CARGO_TARGET_DIR": str(target_dir)}):
            native_path, is_native = build.resolve_local_output_binary_path(
                self.root, ["build", "--release"], "macos-arm64", "macos-arm64"
            )
            foreign_path, is_foreign_native = build.resolve_local_output_binary_path(
                self.root, ["build", "--release"], "linux-x86_64-glibc2.36", "macos-arm64"
            )
        self.assertTrue(is_native)
        self.assertEqual(native_path, target_dir / "release/bridget")
        self.assertFalse(is_foreign_native)
        self.assertEqual(foreign_path, target_dir / "linux-x86_64-glibc2.36/release/bridget")

    def test_git_source_validation_clean_pushed_dirty_unpushed_and_untracked(self):
        repo, _bare, env = self._init_git_repo_with_remote()
        info = build.validate_git_source_for_offload(repo)
        self.assertEqual(len(info["sha12"]), 12)
        self.assertEqual(info["untracked"], [])

        # Fichier non suivi : autorisé mais signalé (et absent de l'archive).
        (repo / "scratch.txt").write_text("temp\n", encoding="utf-8")
        info_u = build.validate_git_source_for_offload(repo)
        self.assertEqual(info_u["untracked"], ["scratch.txt"])
        tar_path = self.root / "out.tar"
        build.create_git_archive(repo, info_u["head_sha"], tar_path)
        tar_list = subprocess.run(["tar", "-tf", str(tar_path)], capture_output=True, text=True, check=True).stdout
        self.assertIn("README.md", tar_list)
        self.assertNotIn("scratch.txt", tar_list)
        (repo / "scratch.txt").unlink()

        # Fichier suivi modifié : refusé.
        (repo / "README.md").write_text("dirty\n", encoding="utf-8")
        with self.assertRaises(ValueError) as ctx_dirty:
            build.validate_git_source_for_offload(repo)
        self.assertIn("--local", str(ctx_dirty.exception))

        # Commit non poussé : refusé.
        subprocess.run(["git", "-C", str(repo), "commit", "-am", "local only"], check=True, capture_output=True, env=env)
        with self.assertRaises(ValueError) as ctx_unpushed:
            build.validate_git_source_for_offload(repo)
        self.assertIn("ancêtre", str(ctx_unpushed.exception))

    def test_ssh_relative_home_file_checks_0600_and_no_symlink(self):
        fake_home = self.root / "home"
        ssh_dir = fake_home / ".ssh"
        ssh_dir.mkdir(parents=True, mode=0o700)
        key = ssh_dir / "id_ed25519"
        key.write_text("dummy", encoding="utf-8")
        key.chmod(0o600)
        kh = ssh_dir / "known_hosts"
        kh.write_text("host ssh-ed25519 AAAA\n", encoding="utf-8")
        kh.chmod(0o600)

        cmd = build.ssh_command_prefix(
            {
                "host": "macbook-m1.local",
                "user": "moi",
                "port": 22,
                "identity_rel": ".ssh/id_ed25519",
                "known_hosts_rel": ".ssh/known_hosts",
            },
            home=fake_home,
        )
        self.assertIn("ForwardAgent=no", cmd)
        self.assertIn("BatchMode=yes", cmd)
        self.assertIn("/dev/null", cmd)

        # Droits trop larges refusés.
        key.chmod(0o644)
        with self.assertRaises(ValueError):
            build.validate_relative_home_file(".ssh/id_ed25519", home=fake_home)
        key.chmod(0o600)

        # Traversée .. refusée.
        with self.assertRaises(ValueError):
            build.validate_relative_home_file("../outside", home=fake_home)

        # Lien symbolique refusé.
        sym = ssh_dir / "sym_key"
        sym.symlink_to(key)
        with self.assertRaises(ValueError):
            build.validate_relative_home_file(".ssh/sym_key", home=fake_home)

    def test_guardian_kills_child_group_on_stdin_eof_and_serializes_with_flock(self):
        guardian = self.root / "guardian.py"
        guardian.write_text(build.REMOTE_GUARDIAN_SCRIPT, encoding="utf-8")
        guardian.chmod(0o700)
        cache_dir = self.root / "cache-guard"
        cache_dir.mkdir(mode=0o700)

        # 1. Coupure stdin -> arrêt du groupe enfant en < 3 s
        work1 = self.root / "w1"
        work1.mkdir(mode=0o700)
        pid_file = work1 / "child.pid"
        cfg1 = work1 / "cfg.json"
        cfg1.write_text(json.dumps({
            "work_dir": str(work1),
            "archive_path": str(work1 / "unused.tar"),
            "cache_dir": str(cache_dir),
            "archive_sha256": "0" * 64,
            "owner_sha": "a" * 40,
            "build_id": "1234567890ab",
            "target": "macos-arm64",
            "cargo_args": ["build", "--release"],
            "heartbeat_timeout_secs": 2.0,
            "kill_grace_secs": 1.0,
            "custom_command": [
                sys.executable, "-c",
                f"import os, time; open({str(pid_file)!r}, 'w').write(str(os.getpid())); time.sleep(30)",
            ],
        }), encoding="utf-8")

        proc = subprocess.Popen(
            [sys.executable, str(guardian), str(cfg1)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        proc.stdin.write(b"ping\n")
        proc.stdin.flush()
        deadline = time.monotonic() + 5.0
        while not pid_file.exists() and time.monotonic() < deadline:
            time.sleep(0.05)
        self.assertTrue(pid_file.exists(), "enfant du gardien non démarré")
        child_pid = int(pid_file.read_text().strip())

        t_close = time.monotonic()
        proc.stdin.close()
        stdout = proc.stdout.read()
        proc.wait(timeout=10)
        proc.stdout.close()
        proc.stderr.close()
        elapsed = time.monotonic() - t_close
        self.assertLess(elapsed, 4.0)
        result_lines = [
            line[len(build.HEAVY_WORK_RESULT_PREFIX):]
            for line in stdout.decode().splitlines()
            if line.startswith(build.HEAVY_WORK_RESULT_PREFIX)
        ]
        self.assertEqual(len(result_lines), 1)
        parsed = json.loads(result_lines[0])
        self.assertFalse(parsed["ok"])
        self.assertIn("stdin_eof", parsed["error"])
        with self.assertRaises(ProcessLookupError):
            os.kill(child_pid, 0)

        # 2. Sérialisation fcntl.flock de deux gardiens concurrents sur le même cache
        log_file = self.root / "flock-order.log"
        procs = []
        for idx in (1, 2):
            w = self.root / f"flock-w{idx}"
            w.mkdir(mode=0o700)
            c = w / "cfg.json"
            c.write_text(json.dumps({
                "work_dir": str(w),
                "archive_path": str(w / "unused.tar"),
                "cache_dir": str(cache_dir),
                "archive_sha256": "0" * 64,
                "owner_sha": "a" * 40,
                "build_id": "1234567890ab",
                "target": "macos-arm64",
                "cargo_args": ["build", "--release"],
                "heartbeat_timeout_secs": 10.0,
                "kill_grace_secs": 1.0,
                "custom_command": [
                    sys.executable, "-c",
                    f"import time; f=open({str(log_file)!r}, 'a'); f.write('start-{idx}\\n'); f.flush(); time.sleep(0.35); f.write('end-{idx}\\n'); f.flush()",
                ],
            }), encoding="utf-8")
            p = subprocess.Popen(
                [sys.executable, str(guardian), str(c)],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            p.stdin.write(b"ping\n")
            p.stdin.flush()
            procs.append(p)
        for p in procs:
            out = p.stdout.read()
            p.wait(timeout=15)
            p.stdin.close()
            p.stdout.close()
            p.stderr.close()
            lines = [
                line[len(build.HEAVY_WORK_RESULT_PREFIX):]
                for line in out.decode().splitlines()
                if line.startswith(build.HEAVY_WORK_RESULT_PREFIX)
            ]
            self.assertTrue(json.loads(lines[0])["ok"], out.decode())
        events = log_file.read_text(encoding="utf-8").splitlines()
        self.assertEqual(len(events), 4)
        # Pas d'entrelacement : start-X immédiatement suivi de end-X.
        first_id = events[0].split("-")[1]
        second_id = events[2].split("-")[1]
        self.assertEqual(events, [f"start-{first_id}", f"end-{first_id}", f"start-{second_id}", f"end-{second_id}"])

    def test_host_without_daemon_requires_explicit_ssh_and_foreign_local_refused(self):
        stderr = io.StringIO()
        parser = build.argparse.ArgumentParser()
        with contextlib.redirect_stderr(stderr), \
             patch.object(build, "call_bridget_heavy_work", return_value=(False, "daemon_unreachable")):
            code = build.run_cargo_action(self.root, ["--host", "sysle2", "build", "--release"], parser)
        self.assertEqual(code, 2)
        self.assertIn("--ssh-host", stderr.getvalue())

        # Cible étrangère --build-target linux-x86_64-glibc2.36 --local refusée sur macOS (y compris via main() Python 3.9)
        stderr2 = io.StringIO()
        with contextlib.redirect_stderr(stderr2), \
             patch.object(sys, "argv", ["build.py", "cargo", "--local", "--build-target", "linux-x86_64-glibc2.36", "build", "--release"]), \
             patch.object(build, "check_local_target_compatible", return_value=(False, "impossible en local --host")):
            code2 = build.main()
        self.assertEqual(code2, 2)
        self.assertIn("--host", stderr2.getvalue())


if __name__ == "__main__":
    unittest.main()
