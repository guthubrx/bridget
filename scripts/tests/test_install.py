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
BINARY = b'''#!/bin/sh
case "$1" in
--version) printf 'bridget 9.8.7 (fixture)\\n';;
clients) [ ! -f "$FIXTURE/client-incompatible" ] || { echo 'Claude Code 2.1.251 < 2.1.280: user approval required'; exit 1; };;
agents) [ -f "$FIXTURE/service-active" ] && [ ! -f "$FIXTURE/health-failure" ] || exit 1; printf '[]\\n';;
*) exit 1;;
esac
'''


class InstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="bginstall-", dir="/tmp")
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.home = self.root / "home with spaces"
        self.home.mkdir()
        self.bin = self.root / "fixtures"
        self.bin.mkdir()
        self.env = dict(os.environ, HOME=str(self.home), SHELL="/bin/bash",
                        PATH=str(self.bin) + os.pathsep + os.environ["PATH"],
                        FIXTURE=str(self.root))
        for key in ("BRIDGET_HOME", "BRIDGET_SOCKET", "BRIDGET_CHANNEL", "XDG_CONFIG_HOME"):
            self.env.pop(key, None)
        self.stub("uname", '#!/bin/sh\ncase "$1" in -s) echo Linux;; -m) echo x86_64;; esac\n')
        self.stub("id", "#!/bin/sh\necho 1001\n")
        self.stub("loginctl", "#!/bin/sh\necho yes\n")
        self.stub("systemctl", '''#!/usr/bin/env python3
import os, pathlib, sys
r = pathlib.Path(os.environ['FIXTURE'])
with (r / 'service-commands').open('a') as f: f.write(' '.join(sys.argv[1:]) + '\\n')
if (r / 'manager-unavailable').exists(): sys.exit(1)
if 'show' in sys.argv:
    fragment = r / 'manager-fragment'
    print(('loaded' if fragment.exists() else 'not-found') if 'LoadState' in sys.argv else fragment.read_text())
if 'enable' in sys.argv:
    (r / 'service-enabled').touch()
    (r / 'service-active').touch()
if 'disable' in sys.argv:
    if (r / 'cleanup-failure').exists(): sys.exit(1)
    (r / 'service-enabled').unlink(missing_ok=True)
    (r / 'service-active').unlink(missing_ok=True)
if 'is-enabled' in sys.argv:
    enabled = (r / 'service-enabled').exists()
    if '--quiet' not in sys.argv: print('enabled' if enabled else 'disabled')
    sys.exit(0 if enabled else 1)
if 'is-active' in sys.argv:
    active = (r / 'service-active').exists()
    if '--quiet' not in sys.argv: print('active' if active else 'inactive')
    sys.exit(0 if active else 3)
''')
        self.stub("sleep", "#!/bin/sh\nexit 0\n")
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
if 'darwin' in url and not (r / 'allow-darwin').exists(): sys.exit(22)
out = pathlib.Path(args[args.index('-o') + 1])
providers = {'https://claude.ai/install.sh':'claude', 'https://chatgpt.com/codex/install.sh':'codex', 'https://cursor.com/install':'cursor-agent'}
if url in providers:
    shutil.copyfile(r / (providers[url] + '.installer'), out)
else:
    shutil.copyfile(r / ('checksum' if url.endswith('.sha256') else 'archive'), out)
''')
        self.archive()

    def stub(self, name, content):
        path = self.bin / name
        path.write_text(content)
        path.chmod(0o755)

    def archive(self, binary=BINARY, member=None, link=False, platform="linux-x86_64", release="9.8.7"):
        stem = "bridget-" + release + "-" + platform
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

    def provider_fixtures214(self):
        import shutil
        # Only fixture programs are visible; a host Gemini must not hide this case.
        for name in ("sh", "bash", "python3", "tar", "gzip", "sha256sum", "cut", "awk", "grep",
                     "tr", "sed", "mkdir", "chmod", "cat", "mktemp", "touch", "cp",
                     "mv", "rm", "rmdir", "wc"):
            if not (self.bin / name).exists():
                (self.bin / name).symlink_to(shutil.which(name))
        self.env["PATH"] = str(self.bin)
        client = self.root / "provider-client"
        client.write_text("#!/bin/sh\nexit 97\n")
        client.chmod(0o755)
        for program in ("claude", "codex", "cursor-agent"):
            (self.root / (program + ".installer")).write_text(
                '#!/bin/sh\nmkdir -p "$HOME/.local/bin"\n'
                f'cp "$FIXTURE/provider-client" "$HOME/.local/bin/{program}"\n'
                f'printf "{program}\\n" >> "$FIXTURE/provider-installs"\n')
        self.stub("node", "#!/bin/sh\nexit 0\n")
        self.stub("npm", '#!/bin/sh\n'
                  '[ "$1" = install ] && [ "$2" = --global ] && [ "$3" = --prefix ] && '
                  '[ "$4" = "$HOME/.local" ] && [ "$5" = @google/gemini-cli ] || exit 99\n'
                  'mkdir -p "$HOME/.local/bin"\n'
                  'cp "$FIXTURE/provider-client" "$HOME/.local/bin/gemini"\n'
                  'printf "gemini\\n" >> "$FIXTURE/provider-installs"\n')

    def test214_explicit_selection_installs_four_without_login_or_model(self):
        self.provider_fixtures214()
        result = self.run_install("--providers", "all")
        self.assertEqual(set((self.root / "provider-installs").read_text().splitlines()),
                         {"claude", "codex", "gemini", "cursor-agent"})
        self.assertIn("Authentication and model access are NOT checked", result.stderr)
        self.assertIn("Personal sign-in", result.stderr)
        self.assertTrue((self.root / "service-active").exists())

    def test214_terminal_offers_each_missing_program_and_respects_no(self):
        import errno
        import pty
        import select
        import time
        self.provider_fixtures214()
        child, fd = pty.fork()
        if child == 0:
            os.execve("/bin/sh", ["sh", str(SCRIPT)], self.env)
        output = bytearray()
        os.write(fd, b"y\nn\nn\nn\n")
        deadline = time.monotonic() + 15
        try:
            while time.monotonic() < deadline:
                ready, _, _ = select.select([fd], [], [], 0.2)
                if not ready:
                    continue
                try:
                    chunk = os.read(fd, 8192)
                except OSError as exc:
                    if exc.errno != errno.EIO:
                        raise
                    break
                if not chunk:
                    break
                output.extend(chunk)
        finally:
            os.close(fd)
        _, status = os.waitpid(child, 0)
        text = output.decode(errors="replace")
        self.assertEqual(os.waitstatus_to_exitcode(status), 0, text)
        for name in ("claude", "codex", "gemini", "cursor"):
            self.assertIn(f"Install {name} from its official publisher", text)
        self.assertEqual((self.root / "provider-installs").read_text().splitlines(), ["claude"])
        proof = os.environ.get("BRIDGET214_INSTALL_PROOF")
        if proof:
            Path(proof).write_text(text)

    def test214_existing_program_and_credentials_are_preserved(self):
        self.provider_fixtures214()
        self.stub("claude", "#!/bin/sh\nexit 98\n")
        credentials = self.home / ".claude.json"
        credentials.write_text("unchanged account")
        result = self.run_install("--providers", "claude")
        self.assertIn("no update or account change", result.stderr)
        self.assertFalse((self.root / "provider-installs").exists())
        self.assertEqual(credentials.read_text(), "unchanged account")
        self.assertNotIn("claude.ai", (self.root / "downloads").read_text())

    def test214_none_and_noninteractive_default_do_not_install_provider(self):
        self.provider_fixtures214()
        for args in [(), ("--providers", "none")]:
            self.run_install(*args)
        self.assertFalse((self.root / "provider-installs").exists())

    def test214_refused_federation_does_not_install_selected_programs(self):
        self.provider_fixtures214()
        self.existing()
        state = self.home / ".cache/bridget-core"
        state.mkdir(parents=True)
        (state / "federation.env").write_text("fixture unavailable federation")
        result = self.run_install("--providers", "all", ok=False)
        self.assertIn("existing federation", result.stderr)
        self.assertFalse((self.root / "provider-installs").exists())
        self.assertNotIn("claude.ai", (self.root / "downloads").read_text())
        self.assertEqual(self.installed.read_bytes(), b"previous executable")

    def test214_incompatible_existing_client_precedes_any_provider_install(self):
        self.provider_fixtures214()
        self.existing()
        (self.root / "client-incompatible").touch()
        self.run_install("--providers", "codex", ok=False)
        self.assertFalse((self.root / "provider-installs").exists())
        self.assertFalse((self.root / "service-commands").exists())

    def test214_new_program_is_checked_again_before_link_and_service_start(self):
        self.provider_fixtures214()
        self.existing()
        installer = self.root / "codex.installer"
        installer.write_text(installer.read_text() + 'touch "$FIXTURE/client-incompatible"\n')
        result = self.run_install("--providers", "codex", ok=False)
        self.assertIn("compatibility check refused", result.stderr)
        self.assertEqual((self.root / "provider-installs").read_text().splitlines(), ["codex"])
        self.assertEqual(self.installed.read_bytes(), b"previous executable")
        self.assertFalse((self.root / "service-active").exists())
        self.assertFalse((self.home / ".profile").exists())

    def test214_invalid_selection_refuses_before_download(self):
        for value in ["claude,,codex", "claude,", ",codex", "codex,codex",
                      "claude;echo PWN", "other", ""]:
            self.run_install("--providers", value, ok=False)
        self.run_install("--providers", "none", "--providers", "codex", ok=False)
        self.assertFalse((self.root / "downloads").exists())
        self.assertFalse(self.installed.exists())

    def test214_gemini_old_node_keeps_existing_bridget_and_services(self):
        self.provider_fixtures214()
        self.stub("node", "#!/bin/sh\nexit 1\n")
        self.existing()
        result = self.run_install("--providers", "gemini", ok=False)
        self.assertIn("Node.js 20", result.stderr)
        self.assertEqual(self.installed.read_bytes(), b"previous executable")
        self.assertFalse((self.root / "provider-installs").exists())
        self.assertFalse((self.root / "service-active").exists())
        self.assertNotIn(" enable ", (self.root / "service-commands").read_text())
        self.assertFalse((self.home / ".profile").exists())

    def test214_failed_provider_install_keeps_existing_bridget(self):
        self.provider_fixtures214()
        (self.root / "codex.installer").write_text("#!/bin/sh\nexit 1\n")
        self.existing()
        self.run_install("--providers", "codex", ok=False)
        self.assertEqual(self.installed.read_bytes(), b"previous executable")
        self.assertFalse((self.root / "service-active").exists())
        self.assertNotIn(" enable ", (self.root / "service-commands").read_text())
        self.assertFalse((self.home / ".profile").exists())

    def test209_client_incompatibility_precedes_link_profiles_and_services(self):
        self.existing()
        (self.root / "client-incompatible").touch()
        result = self.run_install(ok=False)
        self.assertIn("2.1.251", result.stdout)
        self.assertIn("Ask the user", result.stderr)
        self.assertEqual(self.installed.read_bytes(), b"previous executable")
        self.assertFalse((self.root / "service-commands").exists())
        self.assertFalse((self.home / ".profile").exists())

    def test209_published_legacy_package_warns_without_claiming_compatibility(self):
        legacy=BINARY.replace(b"bridget 9.8.7 (fixture)",b"bridget 0.6.3 (fixture)")
        start=legacy.index(b"clients)");end=legacy.index(b"agents)",start)
        legacy=legacy[:start]+b"clients) exit 2;;\n"+legacy[end:]
        self.archive(binary=legacy, release="0.6.3")
        result=self.run_install("--version","v0.6.3")
        self.assertIn("compatibility is NOT verified",result.stderr)
        self.assertEqual(self.installed.read_bytes(),legacy)

    def test209_future_package_without_check_preserves_existing_installation(self):
        self.existing()
        modern=BINARY
        start=modern.index(b"clients)");end=modern.index(b"agents)",start)
        modern=modern[:start]+b"clients) exit 2;;\n"+modern[end:]
        self.archive(binary=modern)
        self.run_install(ok=False)
        self.assertEqual(self.installed.read_bytes(),b"previous executable")

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
        unit = self.home / '.config/systemd/user/app.cartae.bridget.service'
        self.assertTrue(unit.exists())
        self.assertIn('WantedBy=default.target', unit.read_text())
        self.assertTrue((self.root / 'service-active').exists())

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

    def test_missing_user_manager_does_not_report_installation_success(self):
        self.existing()
        (self.root / 'manager-unavailable').touch()
        result = self.run_install(ok=False)
        self.assertIn('user manager is unavailable', result.stderr)
        self.assertEqual(self.installed.read_bytes(), b'previous executable')

    def test_existing_daemon_is_not_started_or_restarted(self):
        (self.root / 'service-active').touch()
        self.run_install()
        self.assertFalse((self.root / 'service-commands').exists())
        self.assertFalse((self.home / '.config/systemd').exists())

    def test_unavailable_federation_never_starts_a_local_daemon(self):
        root = self.home / '.cache/bridget-core'
        root.mkdir(parents=True)
        (root / 'federation.env').write_text('channel=ssh-unix\n')
        result = self.run_install(ok=False)
        self.assertIn('existing federation is unavailable', result.stderr)
        self.assertFalse((self.root / 'service-commands').exists())

    def test_failed_health_check_is_not_installation_success(self):
        (self.root / 'health-failure').touch()
        result = self.run_install(ok=False)
        self.assertIn('installation is not complete', result.stderr.lower())
        self.assertNotIn('Installed bridget', result.stderr)
        self.assertIn('disable --now', (self.root / 'service-commands').read_text())

    def test_service_owned_by_another_installation_is_preserved(self):
        (self.root / 'manager-fragment').write_text('/other/config/app.cartae.bridget.service')
        result = self.run_install(ok=False)
        self.assertIn('another Bridget installation', result.stderr)
        self.assertFalse(self.installed.exists())
        self.assertNotIn('disable --now', (self.root / 'service-commands').read_text())

    def test_existing_inactive_service_is_not_disabled_on_failure(self):
        self.run_install()
        (self.root / 'service-active').unlink()
        (self.root / 'health-failure').touch()
        (self.root / 'service-commands').write_text('')
        result = self.run_install(ok=False)
        self.assertIn('existing service was not disabled', result.stderr)
        self.assertNotIn('disable --now', (self.root / 'service-commands').read_text())

    def test_cleanup_failure_is_reported_as_unconfirmed_shutdown(self):
        (self.root / 'health-failure').touch()
        (self.root / 'cleanup-failure').touch()
        result = self.run_install(ok=False)
        self.assertIn('shutdown could not be confirmed', result.stderr)
        self.assertIn('may still be active', result.stderr)

    def test_shell_namespace_and_service_values_preserve_special_characters(self):
        self.home = self.root / "home's & files"
        self.home.mkdir()
        self.env['HOME'] = str(self.home)
        self.run_install()
        result = subprocess.run(['sh', '-c', '. "$HOME/.local/share/bridget/env"; printf "%s" "$BRIDGET_HOME"'],
                                env=self.env, capture_output=True, text=True, check=True)
        self.assertEqual(result.stdout, str(self.home.resolve() / '.cache/bridget-core'))

    def test_macos_loads_a_private_launchagent_and_verifies_readiness(self):
        import plistlib
        (self.root / 'allow-darwin').touch()
        self.archive(platform='darwin-arm64')
        self.stub('uname', '#!/bin/sh\ncase "$1" in -s) echo Darwin;; -m) echo arm64;; esac\n')
        self.stub('plutil', '''#!/usr/bin/env python3
import plistlib, sys
with open(sys.argv[-1], 'rb') as f: plistlib.load(f)
''')
        self.stub('launchctl', '''#!/usr/bin/env python3
import os, pathlib, sys
r = pathlib.Path(os.environ['FIXTURE'])
with (r / 'service-commands').open('a') as f: f.write(' '.join(sys.argv[1:]) + '\\n')
if sys.argv[1] == 'print' and 'app.cartae.bridget' in sys.argv[-1]: sys.exit(1)
if sys.argv[1] == 'bootstrap': (r / 'service-active').touch()
''')
        self.run_install()
        unit = self.home / 'Library/LaunchAgents/app.cartae.bridget.plist'
        data = plistlib.loads(unit.read_bytes())
        self.assertTrue(data['RunAtLoad'])
        self.assertTrue(data['KeepAlive'])
        self.assertEqual(data['Umask'], 63)
        self.assertEqual(data['ProgramArguments'][-1], 'daemon')
        self.assertNotIn('.cache/bridget-core', data['StandardOutPath'])
        commands = (self.root / 'service-commands').read_text()
        self.assertIn('bootstrap gui/1001', commands)
        self.assertNotIn('kickstart', commands)

    def test_lingering_permission_failure_is_explicit_before_binary_replacement(self):
        self.existing()
        self.stub('loginctl', '#!/bin/sh\ncase "$1" in show-user) echo no;; *) exit 1;; esac\n')
        result = self.run_install(ok=False)
        self.assertIn('Automatic startup needs permission', result.stderr)
        self.assertEqual(self.installed.read_bytes(), b'previous executable')

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
