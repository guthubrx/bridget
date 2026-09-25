#!/usr/bin/env bash
# Oracles SPEC 096 : frontières SSH/launchd doublées, aucun service réel.
set -euo pipefail
umask 077
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
exec /usr/bin/python3 - "$SCRIPT_DIR" <<'PY'
import hashlib
import json
import os
import pty
from pathlib import Path
import shutil
import socket
import stat
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(sys.argv.pop())
PYTHON = sys.executable

FAKE_TOOL = r'''
import json, os, pathlib, stat, subprocess, sys
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
if name == 'uname': print('Darwin'); sys.exit(0)
if name == 'stat':
    item=os.stat(args[-1], follow_symlinks=False)
    print(f'{item.st_uid} {stat.S_IMODE(item.st_mode):o}')
    sys.exit(0)
log=pathlib.Path(os.environ['TEST_LOG'])
with log.open('a') as stream: stream.write(json.dumps([name,args])+'\n')
states=pathlib.Path(os.environ['TEST_SERVICE_STATES']); states.mkdir(exist_ok=True)
if name == 'launchctl':
    if args[0] == 'bootstrap':
        label=pathlib.Path(args[-1]).stem
        (states/label).write_text('running\n'); sys.exit(0)
    label=args[-1].rsplit('/',1)[-1]
    marker=states/label
    if args[0] == 'print':
        if marker.exists(): print('state = running'); sys.exit(0)
        sys.exit(3)
    if args[0] == 'bootout': marker.unlink(missing_ok=True); sys.exit(0)
if name == 'ssh':
    if '-N' in args: sys.exit(0)
    result=subprocess.run(['/bin/bash','-c',args[-1]], input=sys.stdin.buffer.read(), env=os.environ)
    sys.exit(result.returncode)
raise AssertionError((name,args))
'''


class Federation096(unittest.TestCase):
    def setUp(self):
        base = '/private/tmp' if Path('/private/tmp').is_dir() else '/tmp'
        self.root = Path(tempfile.mkdtemp(prefix='bf096-', dir=base)).resolve()
        self.root.chmod(0o700)
        self.home = self.root / 'home'; self.home.mkdir(mode=0o700)
        self.tools = self.root / 'tools'; self.tools.mkdir(mode=0o700)
        self.local = self.root / 'local'; self.local.mkdir(mode=0o700)
        self.identity = self.root / 'identity'; self.identity.write_text('KEY\n'); self.identity.chmod(0o600)
        self.known = self.root / 'known_hosts'; self.known.write_text('HOST\n'); self.known.chmod(0o600)
        for name in ['uname', 'stat', 'ssh', 'launchctl']:
            tool = self.tools / name
            tool.write_text('#!' + PYTHON + '\n' + FAKE_TOOL)
            tool.chmod(0o700)
        self.master = socket.socket(socket.AF_UNIX)
        self.master.bind(str(self.local / 'master.sock'))
        (self.local / 'master.sock').chmod(0o600)
        self.env = {
            'PATH': str(self.tools) + ':/usr/bin:/bin',
            'HOME': str(self.home),
            'TEST_LOG': str(self.root / 'calls.jsonl'),
            'TEST_SERVICE_STATES': str(self.root / 'states'),
        }

    def tearDown(self):
        self.master.close()
        shutil.rmtree(self.root)

    def run_script(self, args, success=True, env=None):
        result = subprocess.run(
            ['/bin/bash', str(SCRIPTS / 'federate-ssh.sh')] + args,
            env=env or self.env, text=True, capture_output=True, timeout=15,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def run_pty(self, args, answers):
        master, slave = pty.openpty()
        process = subprocess.Popen(
            ['/bin/bash', str(SCRIPTS / 'federate-ssh.sh')] + args,
            env=self.env, stdin=slave, stdout=slave, stderr=subprocess.PIPE, text=False,
        )
        os.close(slave)
        os.write(master, answers.encode())
        returncode = process.wait(timeout=15)
        stderr = process.stderr.read().decode()
        process.stderr.close()
        os.close(master)
        self.assertEqual(returncode, 0, stderr)
        return stderr

    def calls(self):
        path = self.root / 'calls.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def clear_calls(self):
        (self.root / 'calls.jsonl').unlink(missing_ok=True)

    def options(self, label, remote_name):
        remote = self.root / remote_name
        return [
            '--label', label, '--user', 'fixture', '--identity', str(self.identity),
            '--known-hosts', str(self.known), '--root', str(self.local),
            '--socket', str(self.local / 'master.sock'), '--remote-root', str(remote),
            '--remote-socket', str(remote / 'peer.sock'),
        ]

    def direct_install(self, label='poste-beta-core', remote_name='remote-one'):
        self.run_script([
            'install', '--label', label, '--host', '127.0.0.1', '--user', 'fixture',
            '--port', '2222', '--identity', str(self.identity), '--known-hosts', str(self.known),
            '--root', str(self.local), '--socket', str(self.local / 'master.sock'),
            '--remote-root', str(self.root / remote_name),
            '--remote-socket', str(self.root / remote_name / 'peer.sock'),
        ])

    def test_help_and_closed_url_grammar(self):
        self.assertIn('bridget federate', self.run_script(['cli', '--help']).stdout)
        for destination in ['http://localhost', 'ssh://u:p@localhost', 'ssh://localhost/path', 'ssh://localhost?q=1', 'ssh://localhost:0', '\tssh://localhost']:
            self.run_script(['cli', destination], success=False)
        self.run_script(['cli', 'ssh://localhost:22', '-p', '2222'], success=False)
        self.assertFalse(any(name == 'ssh' for name, _ in self.calls()))

    def test_dns_name_reuses_attested_ip_install_without_mutation(self):
        self.direct_install()
        config = self.home / '.local/share/bridget-federation/poste-beta-core/config'
        before = hashlib.sha256(config.read_bytes()).hexdigest()
        self.clear_calls()
        result = self.run_script(['cli', 'ssh://fixture@localhost', '-p', '2222'])
        self.assertIn('réutilisée', result.stdout + result.stderr)
        self.assertEqual(hashlib.sha256(config.read_bytes()).hexdigest(), before)
        self.assertFalse(any(name == 'ssh' for name, _ in self.calls()))
        other_identity = self.root / 'other-identity'
        other_identity.write_text('OTHER\n'); other_identity.chmod(0o600)
        self.run_script([
            'cli', 'ssh://fixture@localhost', '-p', '2222',
            '--identity', str(other_identity),
        ], success=False)
        self.assertEqual(hashlib.sha256(config.read_bytes()).hexdigest(), before)

    def test_ambiguous_destination_requires_label(self):
        self.direct_install('first', 'remote-first')
        self.direct_install('second', 'remote-second')
        self.clear_calls()
        result = self.run_script(['cli', 'ssh://fixture@localhost', '-p', '2222'], success=False)
        self.assertIn('--label', result.stderr)
        selected = self.run_script(['cli', 'ssh://fixture@localhost', '-p', '2222', '--label', 'second'])
        self.assertIn('[second]', selected.stdout + selected.stderr)

    def test_known_destination_cannot_be_cloned_under_a_new_label(self):
        for destination in ['ssh://fixture@127.0.0.1:2222', 'ssh://fixture@localhost:2222']:
            with self.subTest(destination=destination):
                if not (self.home / '.local/share/bridget-federation/poste-beta-core').exists():
                    self.direct_install()
                self.clear_calls()
                self.run_script(
                    ['cli', destination] + self.options('clone', 'remote-clone'),
                    success=False,
                )
                self.assertFalse((self.home / '.local/share/bridget-federation/clone').exists())
                self.assertFalse(any(name == 'launchctl' and args[0] == 'bootstrap' for name, args in self.calls()))

    def test_new_destination_non_tty_lists_missing_flags_without_side_effect(self):
        result = self.run_script(['cli', 'ssh://localhost', '-p', '2222'], success=False)
        for option in ['--label', '--user', '--identity', '--known-hosts', '--root', '--socket', '--remote-root', '--remote-socket']:
            self.assertIn(option, result.stderr)
        self.assertEqual(self.calls(), [])

    def test_complete_new_destination_delegates_to_install(self):
        result = self.run_script(['cli', 'ssh://localhost', '-p', '2222'] + self.options('new-link', 'remote-new'))
        self.assertIn('[new-link] installée', result.stderr)
        runner = self.home / '.local/share/bridget-federation/new-link/runner.sh'
        self.assertTrue(runner.exists())
        self.assertNotIn(str(SCRIPTS.parent), runner.read_text())

    def test_double_tty_prompts_only_for_missing_new_install_values(self):
        remote = self.root / 'remote-tty'
        answers = '\n'.join([
            'tty-link', 'fixture', str(self.identity), str(self.known), str(self.local),
            str(self.local / 'master.sock'), str(remote), str(remote / 'peer.sock'), '',
        ])
        self.run_pty(['cli', 'ssh://localhost', '-p', '2222'], answers)
        self.assertTrue((self.home / '.local/share/bridget-federation/tty-link/runner.sh').exists())

    def test_global_status_never_uses_dns_or_ssh(self):
        self.direct_install()
        self.clear_calls()
        result = self.run_script(['cli', 'status'])
        self.assertIn('poste-beta-core', result.stdout)
        self.assertFalse(any(name == 'ssh' for name, _ in self.calls()))

    def test_remove_requires_and_matches_destination(self):
        self.direct_install()
        self.clear_calls()
        self.run_script(['cli', 'remove'], success=False)
        self.assertTrue((self.home / '.local/share/bridget-federation/poste-beta-core').exists())
        self.run_script(['cli', 'remove', '--label', 'poste-beta-core', '-p', '2222'], success=False)
        self.assertTrue((self.home / '.local/share/bridget-federation/poste-beta-core').exists())
        self.assertFalse(any(name == 'launchctl' and args[0] == 'bootout' for name, args in self.calls()))
        result = self.run_script(['cli', 'remove', 'ssh://fixture@localhost', '-p', '2222'], success=False)
        self.assertIn('--label poste-beta-core', result.stderr)
        self.assertTrue((self.home / '.local/share/bridget-federation/poste-beta-core').exists())
        self.run_script(['cli', 'remove', 'ssh://fixture@localhost', '-p', '2222', '--label', 'poste-beta-core'])
        self.assertFalse((self.home / '.local/share/bridget-federation/poste-beta-core').exists())

    def test_dns_remove_can_be_confirmed_only_on_double_tty(self):
        self.direct_install()
        self.run_pty(['cli', 'remove', 'ssh://fixture@localhost', '-p', '2222'], 'oui\n')
        self.assertFalse((self.home / '.local/share/bridget-federation/poste-beta-core').exists())

    def test_exact_registered_host_remove_needs_no_dns_confirmation(self):
        self.direct_install()
        self.run_script(['cli', 'remove', 'ssh://fixture@127.0.0.1:2222'])
        self.assertFalse((self.home / '.local/share/bridget-federation/poste-beta-core').exists())

    def test_inventory_and_dns_bounds_refuse_instead_of_truncating(self):
        base = self.home / '.local/share/bridget-federation'
        base.mkdir(parents=True, mode=0o700)
        for index in range(129):
            (base / f'candidate-{index:03}').mkdir(mode=0o700)
        result = self.run_script(['cli', 'status'], success=False)
        self.assertIn('maximum 128', result.stderr)
        shutil.rmtree(base)
        self.direct_install()
        hooks = self.root / 'python-hooks'; hooks.mkdir(mode=0o700)
        (hooks / 'sitecustomize.py').write_text(
            "import os, socket, time\n"
            "_mode=os.environ.get('TEST_DNS_MODE')\n"
            "if _mode == 'many':\n"
            "    socket.getaddrinfo=lambda *a, **k: [(socket.AF_INET, socket.SOCK_STREAM, 6, '', (f'10.0.0.{i}', 0)) for i in range(1, 66)]\n"
            "elif _mode == 'slow':\n"
            "    socket.getaddrinfo=lambda *a, **k: (time.sleep(10), [])[1]\n"
        )
        many = dict(self.env, PYTHONPATH=str(hooks), TEST_DNS_MODE='many')
        result = self.run_script(['cli', 'ssh://fixture@alias.invalid', '-p', '2222'], success=False, env=many)
        self.assertIn('maximum 64', result.stderr)
        slow = dict(self.env, PYTHONPATH=str(hooks), TEST_DNS_MODE='slow')
        started = __import__('time').monotonic()
        self.run_script(['cli', 'ssh://fixture@alias.invalid', '-p', '2222'], success=False, env=slow)
        self.assertLess(__import__('time').monotonic() - started, 7.0)


unittest.main(verbosity=2)
PY
