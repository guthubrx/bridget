"""Installer contract tests: real archives and isolated homes; network is local fixture."""
import hashlib
import importlib.util
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / "install.sh"
BINARY = b'#!/bin/sh\nprintf "bridget 9.8.7 (fixture)\\n"\n'


class InstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="bridget-install-test-")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.home = self.root / "home with spaces"
        self.home.mkdir()
        self.bin = self.root / "fixtures"
        self.bin.mkdir()
        self.env = dict(os.environ, HOME=str(self.home), SHELL="/bin/bash",
                        PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        FIXTURE=str(self.root))
        self.stub("uname", '#!/bin/sh\ncase "$1" in -s) echo Linux;; -m) echo x86_64;; esac\n')
        self.stub("id", "#!/bin/sh\necho 1001\n")
        self.stub("curl", '''#!/usr/bin/env python3
import os, pathlib, shutil, sys
r = pathlib.Path(os.environ['FIXTURE'])
args = sys.argv[1:]
url = args[-1]
with (r / 'downloads').open('a') as f: f.write(url + '\\n')
if url.endswith('/releases/latest'):
    print('https://github.com/guthubrx/bridget/releases/tag/v9.8.7', end='')
    sys.exit(0)
if (r / 'network-failure').exists(): sys.exit(22)
if 'darwin' in url: sys.exit(22)
out = pathlib.Path(args[args.index('-o') + 1])
shutil.copyfile(r / ('checksum' if url.endswith('.sha256') else 'archive'), out)
''')
        self.archive()

    def stub(self, name, content):
        path = self.bin / name
        path.write_text(content)
        path.chmod(0o755)

    def archive(self, binary=BINARY, member=None, link=False):
        stem = "bridget-9.8.7-linux-x86_64"
        with tarfile.open(self.root / "archive", "w:gz") as archive:
            info = tarfile.TarInfo(member or f"{stem}/bridget")
            if link:
                info.type = tarfile.SYMTYPE
                info.linkname = "/bin/sh"
                archive.addfile(info)
            else:
                info.size = len(binary)
                info.mode = 0o755
                archive.addfile(info, io.BytesIO(binary))
        digest = hashlib.sha256((self.root / "archive").read_bytes()).hexdigest()
        (self.root / "checksum").write_text(f"{digest}  {stem}.tar.gz\n")

    @property
    def installed(self):
        return self.home / ".local/bin/bridget"

    def run_install(self, *args, ok=True):
        result = subprocess.run(["sh", str(SCRIPT), *args], env=self.env,
                                capture_output=True, text=True, timeout=20)
        if ok:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stderr)
        return result

    def existing(self):
        self.installed.parent.mkdir(parents=True)
        self.installed.write_bytes(b"previous executable")

    def test_release_latest_and_update_preserve_data_and_profile(self):
        profile = self.home / ".bashrc"
        profile.write_text('return\n# existing settings\n')
        credentials = self.home / ".claude.json"
        credentials.write_text("private fixture")
        self.run_install()
        self.assertEqual(self.installed.read_bytes(), BINARY)
        self.assertTrue(os.access(self.installed, os.X_OK))
        self.assertTrue(profile.read_text().startswith('. "$HOME/.local/share/bridget/env"\n'))
        self.assertIn("existing settings", profile.read_text())
        self.run_install()
        self.assertEqual(profile.read_text().count('. "$HOME/.local/share/bridget/env"'), 1)
        self.assertFalse((self.home / ".local/state/bridget-install/previous-bridget").exists())
        self.assertEqual(credentials.read_text(), "private fixture")
        self.assertIn("source=v9.8.7", (self.home / ".local/state/bridget-install/installed.txt").read_text())
        self.assertFalse((self.home / ".config/systemd").exists())

    def test_corruption_keeps_old_binary_and_profiles(self):
        self.existing()
        (self.root / "archive").write_bytes(b"corrupt")
        result = self.run_install(ok=False)
        self.assertIn("SHA-256 mismatch", result.stderr)
        self.assertEqual(self.installed.read_bytes(), b"previous executable")
        self.assertFalse((self.home / ".bashrc").exists())

    def test_download_failure_keeps_old_binary(self):
        self.existing()
        (self.root / "network-failure").touch()
        self.run_install(ok=False)
        self.assertEqual(self.installed.read_bytes(), b"previous executable")

    def test_repeat_install_preserves_distinct_rollback(self):
        self.existing()
        self.run_install()
        self.run_install()
        self.assertEqual((self.home / ".local/state/bridget-install/previous-bridget").read_bytes(), b"previous executable")

    def test_bash_login_keeps_existing_profile_selection(self):
        (self.home / ".profile").write_text("export MY_SETTING=keep\n")
        self.run_install()
        self.assertFalse((self.home / ".bash_profile").exists())
        self.assertIn("MY_SETTING=keep", (self.home / ".profile").read_text())
        (self.home / ".bash_login").write_text("export LOGIN_SETTING=keep\n")
        self.run_install()
        self.assertFalse((self.home / ".bash_profile").exists())
        self.assertIn('LOGIN_SETTING=keep', (self.home / ".bash_login").read_text())
        self.assertIn('. "$HOME/.local/share/bridget/env"', (self.home / ".bash_login").read_text())

    def test_pin_skips_latest_lookup(self):
        self.run_install("--version", "v9.8.7")
        self.assertNotIn("/releases/latest", (self.root / "downloads").read_text())

    def test_checksum_accepts_uppercase_hex(self):
        checksum = self.root / "checksum"
        checksum.write_text(checksum.read_text().upper())
        self.run_install()

    def test_unsupported_package_does_not_build_sources(self):
        self.stub("uname", '#!/bin/sh\ncase "$1" in -s) echo Darwin;; -m) echo arm64;; esac\n')
        result = self.run_install(ok=False)
        self.assertIn("Use --source explicitly", result.stderr)
        self.assertFalse(self.installed.exists())

    def test_bad_executable_and_symlink_archive_are_rejected(self):
        for binary, link in ((b"not executable format", False), (BINARY, True)):
            with self.subTest(link=link):
                self.archive(binary=binary, link=link)
                self.run_install(ok=False)
                self.assertFalse(self.installed.exists())

    def test_archive_cannot_extract_outside_installation(self):
        self.archive(member="../../escaped")
        self.run_install(ok=False)
        self.assertFalse((self.root / "escaped").exists())
        self.assertFalse(self.installed.exists())

    def test_symlink_destination_is_replaced_without_touching_target(self):
        self.installed.parent.mkdir(parents=True)
        target = self.root / "old-release"
        target.write_bytes(b"old release")
        self.installed.symlink_to(target)
        self.run_install()
        self.assertFalse(self.installed.is_symlink())
        self.assertEqual(target.read_bytes(), b"old release")

    def test_profile_symlink_is_refused(self):
        target = self.root / "external-profile"
        target.write_text("keep me")
        (self.home / ".bashrc").symlink_to(target)
        self.run_install(ok=False)
        self.assertEqual(target.read_text(), "keep me")
        self.assertFalse(self.installed.exists())
        self.assertFalse((self.home / ".profile").exists())

    def test_concurrent_install_is_refused(self):
        lock = self.home / ".local/state/bridget-install/lock"
        lock.mkdir(parents=True)
        result = self.run_install(ok=False)
        self.assertIn("Another installation", result.stderr)
        self.assertTrue(lock.exists())

    def test_options_are_validated(self):
        for args in (("--source", "--version", "v9.8.7"), ("--version", "../evil"), ("--version", "v9.8.7\n../evil"), ("--unknown",)):
            with self.subTest(args=args):
                self.run_install(*args, ok=False)
                self.assertFalse(self.installed.exists())

    def test_source_uses_locked_build_wrapper_and_shared_cache(self):
        source = self.root / "source-fixture"
        (source / "scripts").mkdir(parents=True)
        (source / "scripts/build.py").write_text('''import os, pathlib, sys
r = pathlib.Path(os.environ['FIXTURE'])
with (r / 'build-args').open('a') as f: f.write(' '.join(sys.argv[1:]) + '\\n')
if sys.argv[1] == 'cargo':
    target = pathlib.Path(os.environ['CARGO_TARGET_DIR']) / 'release/bridget'
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes((r / 'candidate').read_bytes())
''')
        (self.root / "candidate").write_bytes(BINARY)
        self.stub("git", '''#!/usr/bin/env python3
import os, pathlib, shutil, sys
mode = pathlib.Path(os.environ['FIXTURE']) / 'git-mode'
mode = mode.read_text() if mode.exists() else ''
if 'clone' in sys.argv:
    shutil.copytree(pathlib.Path(os.environ['FIXTURE']) / 'source-fixture', sys.argv[-1])
    (pathlib.Path(sys.argv[-1]) / '.git').mkdir()
elif 'status' in sys.argv:
    if mode == 'error': sys.exit(128)
    if mode == 'dirty': print(' M edited-file')
elif 'checkout' in sys.argv or 'fetch' in sys.argv: pass
elif 'remote' in sys.argv: print('https://example.com/other.git' if mode == 'origin' else 'https://github.com/guthubrx/bridget.git')
else: print('1234567890abcdef1234567890abcdef12345678')
''')
        self.stub("cc", "#!/bin/sh\nexit 0\n")
        self.stub("rustup", "#!/bin/sh\nexit 0\n")
        self.stub("cargo", "#!/bin/sh\nexit 0\n")
        self.run_install("--source")
        self.assertEqual(self.installed.read_bytes(), BINARY)
        args = (self.root / "build-args").read_text()
        self.assertIn("cargo build --locked --release -p bridget-daemon", args)
        self.assertIn("installed --target", args)
        self.assertIn("bridget-build-installer", args)
        self.run_install("--source")
        self.assertEqual(self.installed.read_bytes(), BINARY)
        for mode in ('dirty', 'error', 'origin'):
            (self.root / 'git-mode').write_text(mode)
            result = self.run_install('--source', ok=False)
            self.assertEqual(self.installed.read_bytes(), BINARY)
            if mode == 'error':
                self.assertIn('Cannot inspect', result.stderr)
        (self.root / 'git-mode').unlink()
        checkout = self.home / ".local/state/bridget-install/source"
        (checkout / "scripts/build.py").write_text("raise SystemExit(42)\n")
        self.run_install("--source", ok=False)
        self.assertEqual(self.installed.read_bytes(), BINARY)

    def test_real_build_wrapper_needs_stable_checkout_for_cache_ownership(self):
        spec = importlib.util.spec_from_file_location("bridget_build", SCRIPT.with_name("build.py"))
        build = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(build)
        first, second = self.root / "clone-one", self.root / "clone-two"
        for repo in (first, second):
            subprocess.run(["git", "init", "--quiet", str(repo)], env=self.env, check=True)
        target = self.home.resolve() / ".cache/bridget-build-installer"
        target.mkdir(parents=True)
        with patch.dict(os.environ, dict(self.env, CARGO_TARGET_DIR=str(target))):
            build.register_staging(first, [])
            self.assertIn(target, list(build.owned_staging(first)))
            self.assertNotIn(target, list(build.owned_staging(second)))
            with self.assertRaisesRegex(ValueError, "staging enregistré"):
                build.installed(second, target, self.root / "unused", True)
            # The installer now uses first's stable path on every invocation.
            build.register_staging(first, [])
            self.assertIn(target, list(build.owned_staging(first)))


if __name__ == "__main__":
    unittest.main()
