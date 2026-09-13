#!/usr/bin/env bash
# Oracles SPEC 095 : gestionnaires natifs et SSH sont doublés. Aucun service,
# réseau, daemon Bridget ou fichier utilisateur réel n'est touché.
set -euo pipefail
umask 077
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
exec /usr/bin/python3 - "$SCRIPT_DIR" <<'PY'
import json
import os
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
import json, os, pathlib, signal, socket, stat, subprocess, sys, time
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
state=pathlib.Path(os.environ['TEST_SERVICE_STATE'])
if name == 'uname':
    print(os.environ['TEST_PLATFORM']); sys.exit(0)
if name == 'stat':
    item=os.stat(args[-1], follow_symlinks=False)
    print(f'{item.st_uid} {stat.S_IMODE(item.st_mode):o}')
    sys.exit(0)
log_path=pathlib.Path(os.environ['TEST_LOG'])
with log_path.open('a') as log:
    log.write(json.dumps([name,args])+'\n')
if name == 'launchctl':
    if args[0] == 'bootstrap':
        state.write_text('active\n')
        if os.environ.get('TEST_CREATE_REMOTE_STALE'):
            endpoint=socket.socket(socket.AF_UNIX)
            endpoint.bind(os.environ['TEST_CREATE_REMOTE_STALE'])
            endpoint.close()
        if os.environ.get('TEST_SIGNAL_ON_BOOTSTRAP'):
            os.kill(os.getppid(), signal.SIGTERM); time.sleep(0.2)
        sys.exit(int(os.environ.get('TEST_ACTIVATE_EXIT','0')))
    if args[0] == 'print':
        print_once=state.with_suffix('.print-once')
        if state.exists() and not print_once.exists() and os.environ.get('TEST_SIGNAL_ON_PRINT'):
            print_once.write_text('done\n')
            os.kill(os.getppid(), signal.SIGTERM); time.sleep(0.2)
        if state.exists() and not print_once.exists() and os.environ.get('TEST_PRINT_FAIL'):
            print_once.write_text('done\n')
            sys.exit(8)
        if state.exists(): print('state = running'); sys.exit(0)
        sys.exit(3)
    if args[0] == 'bootout':
        if os.environ.get('TEST_BOOTOUT_FAIL'): sys.exit(9)
        state.unlink(missing_ok=True); sys.exit(0)
    raise AssertionError(args)
if name == 'systemctl':
    assert args[0] == '--user', args
    rest=args[1:]
    if rest == ['daemon-reload']: sys.exit(0)
    if rest[:2] == ['show','--property=LoadState']:
        print('loaded' if state.exists() or os.environ.get('TEST_LABEL_LOADED') else 'not-found')
        sys.exit(0)
    if rest[:2] == ['enable','--now']:
        state.write_text('active\n'); sys.exit(0)
    if rest[:2] == ['disable','--now']:
        state.unlink(missing_ok=True); sys.exit(0)
    if rest[:2] in (['is-active','--quiet'], ['is-enabled','--quiet']):
        sys.exit(0 if state.exists() else 3)
    raise AssertionError(args)
if name == 'ssh':
    required=['BatchMode=yes','StrictHostKeyChecking=yes','ControlMaster=no',
      'ControlPath=none','ControlPersist=no','ExitOnForwardFailure=yes',
      'ServerAliveInterval=15','ServerAliveCountMax=3','ConnectTimeout=10',
      'ConnectionAttempts=1','StreamLocalBindUnlink=no','StreamLocalBindMask=0177',
      'ForwardAgent=no','PermitLocalCommand=no']
    assert all(item in args for item in required), args
    assert args[:2] == ['-F','/dev/null'], args
    if '-N' in args:
        assert args.count('-R') == 1, args
        sys.exit(int(os.environ.get('TEST_FORWARD_EXIT','0')))
    assert args[-1].startswith('bash -s -- '), args
    env=dict(os.environ)
    if env.get('TEST_REMOTE_PROBE_EXIT'):
        env['PATH']=env['TEST_REMOTE_TOOLS']+':/usr/bin:/bin'
    result=subprocess.run(['/bin/bash','-c',args[-1]],input=sys.stdin.buffer.read(),env=env)
    sys.exit(result.returncode)
raise AssertionError((name,args))
'''


class Federation095(unittest.TestCase):
    def setUp(self):
        base = '/private/tmp' if Path('/private/tmp').is_dir() else '/tmp'
        self.root = Path(tempfile.mkdtemp(prefix='bf095-', dir=base)).resolve()
        self.root.chmod(0o700)
        for name in ['home', 'local', 'remote', 'tools', 'remote-tools']:
            (self.root / name).mkdir(mode=0o700)
        self.home = self.root / 'home'
        self.local = self.root / 'local'
        self.remote = self.root / 'remote'
        self.identity = self.root / 'identity'
        self.known = self.root / 'known_hosts'
        for path in [self.identity, self.known]:
            path.write_text('SENTINELLE\n')
            path.chmod(0o600)
        for name in ['uname', 'stat', 'ssh', 'launchctl', 'systemctl']:
            path = self.root / 'tools' / name
            path.write_text('#!' + PYTHON + '\n' + FAKE_TOOL)
            path.chmod(0o700)
        remote_python = self.root / 'remote-tools' / 'python3'
        remote_python.write_text('#!/bin/sh\nexit "${TEST_REMOTE_PROBE_EXIT:-22}"\n')
        remote_python.chmod(0o700)
        self.master = socket.socket(socket.AF_UNIX)
        self.master.bind(str(self.local / 'master.sock'))
        (self.local / 'master.sock').chmod(0o600)
        self.env = {
            'PATH': str(self.root / 'tools') + ':/usr/bin:/bin',
            'HOME': str(self.home),
            'XDG_CONFIG_HOME': str(self.home / '.config'),
            'XDG_DATA_HOME': str(self.home / '.local/share'),
            'TEST_LOG': str(self.root / 'calls.jsonl'),
            'TEST_SERVICE_STATE': str(self.root / 'service.active'),
            'TEST_PLATFORM': 'Darwin',
            'TEST_REMOTE_TOOLS': str(self.root / 'remote-tools'),
        }
        self.common = [
            '--label', 'fixture', '--host', 'test.invalid', '--user', 'fixture',
            '--port', '2222', '--identity', str(self.identity),
            '--known-hosts', str(self.known), '--root', str(self.local),
            '--socket', str(self.local / 'master.sock'),
            '--remote-root', str(self.remote),
            '--remote-socket', str(self.remote / 'peer.sock'),
        ]

    def tearDown(self):
        self.master.close()
        shutil.rmtree(self.root)

    def run_script(self, args, success=True, env=None, installed=False):
        script = self.runner if installed else SCRIPTS / 'federate-ssh.sh'
        result = subprocess.run(
            ['/bin/bash', str(script)] + args,
            env=env or self.env,
            text=True,
            capture_output=True,
            timeout=15,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def calls(self):
        path = self.root / 'calls.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def clear_calls(self):
        (self.root / 'calls.jsonl').unlink(missing_ok=True)

    @property
    def install_dir(self):
        return self.home / '.local/share/bridget-federation/fixture'

    @property
    def runner(self):
        return self.install_dir / 'runner.sh'

    @property
    def plist(self):
        return self.home / 'Library/LaunchAgents/com.bridget.federation.fixture.plist'

    @property
    def unit(self):
        return self.home / '.config/systemd/user/bridget-federation-fixture.service'

    def install(self, platform='Darwin'):
        env = dict(self.env, TEST_PLATFORM=platform)
        return self.run_script(['install'] + self.common, env=env)

    def test_launchd_cycle_uses_autonomous_owned_runner(self):
        self.install()
        self.assertEqual(stat.S_IMODE(self.install_dir.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE(self.runner.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE((self.install_dir / 'config').stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE((self.install_dir / 'receipt').stat().st_mode), 0o600)
        plist = self.plist.read_text()
        self.assertIn(str(self.runner), plist)
        self.assertIn('<key>ThrottleInterval</key><integer>10</integer>', plist)
        self.assertIn('<key>Umask</key><integer>63</integer>', plist)
        self.assertNotIn(str(SCRIPTS.parent), plist)
        self.assertNotIn('daemon', plist)
        result = self.run_script(['status', '--label', 'fixture'])
        self.assertIn('processus actif', result.stdout)
        self.run_script(['remove', '--label', 'fixture'])
        self.assertFalse(self.plist.exists())
        self.assertFalse(self.install_dir.exists())
        self.assertEqual(self.identity.read_text(), 'SENTINELLE\n')
        self.assertEqual(self.known.read_text(), 'SENTINELLE\n')

    def test_systemd_user_cycle_has_bounded_restart_and_linger_notice(self):
        result = self.install('Linux')
        unit = self.unit.read_text()
        self.assertIn('Restart=always', unit)
        self.assertIn('RestartSec=5', unit)
        self.assertIn('UMask=0077', unit)
        self.assertIn('StartLimitIntervalSec=0', unit)
        self.assertIn(str(self.runner), unit)
        self.assertNotIn(str(SCRIPTS.parent), unit)
        self.assertIn('linger', result.stderr.lower())
        env = dict(self.env, TEST_PLATFORM='Linux')
        self.assertIn('processus actif', self.run_script(['status', '--label', 'fixture'], env=env).stdout)
        self.run_script(['remove', '--label', 'fixture'], env=env)
        systemctl = [args for name, args in self.calls() if name == 'systemctl']
        self.assertTrue(any(args[:3] == ['--user', 'enable', '--now'] for args in systemctl))
        self.assertTrue(any(args[:3] == ['--user', 'disable', '--now'] for args in systemctl))

    def test_install_compatible_is_idempotent(self):
        self.install()
        before = {p.name: p.read_bytes() for p in self.install_dir.iterdir() if p.is_file()}
        result = self.install()
        after = {p.name: p.read_bytes() for p in self.install_dir.iterdir() if p.is_file()}
        self.assertEqual(before, after)
        self.assertIn('déjà installée', result.stderr)

    def test_install_existing_with_different_target_is_refused_without_mutation(self):
        self.install()
        before = {p.name: p.read_bytes() for p in self.install_dir.iterdir() if p.is_file()}
        self.clear_calls()
        divergent = list(self.common)
        divergent[divergent.index('--host') + 1] = 'other.invalid'
        self.run_script(['install'] + divergent, success=False)
        after = {p.name: p.read_bytes() for p in self.install_dir.iterdir() if p.is_file()}
        self.assertEqual(before, after)
        self.assertEqual(self.calls(), [])

    def test_collision_service_and_symlink_refused_without_activation(self):
        self.plist.parent.mkdir(parents=True)
        self.plist.write_text('ETRANGER\n')
        self.run_script(['install'] + self.common, success=False)
        self.assertEqual(self.plist.read_text(), 'ETRANGER\n')
        self.assertFalse(self.install_dir.exists())
        self.assertFalse((self.root / 'service.active').exists())
        self.plist.unlink()
        self.install_dir.parent.mkdir(parents=True)
        self.install_dir.symlink_to(self.remote, target_is_directory=True)
        self.run_script(['install'] + self.common, success=False)
        self.assertEqual(list(self.remote.iterdir()), [])

    def test_native_label_loaded_elsewhere_is_refused_without_bootout_or_publication(self):
        (self.root / 'service.active').write_text('foreign\n')
        self.run_script(['install'] + self.common, success=False)
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())
        calls = self.calls()
        self.assertTrue(any(name == 'launchctl' and args[0] == 'print' for name, args in calls))
        self.assertFalse(any(name == 'launchctl' and args[0] == 'bootout' for name, args in calls))
        self.assertFalse(any(name == 'ssh' for name, _ in calls))

    def test_local_preparation_failure_rolls_back_and_retry_succeeds(self):
        service_dir = self.plist.parent
        service_dir.mkdir(parents=True)
        service_dir.chmod(0o777)
        self.run_script(['install'] + self.common, success=False)
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())
        service_dir.chmod(0o700)
        self.install()
        self.assertTrue(self.runner.exists())

    def test_partial_activation_rolls_back_only_after_stop_and_remote_cleanup(self):
        env = dict(self.env, TEST_PRINT_FAIL='1')
        self.run_script(['install'] + self.common, success=False, env=env)
        calls = self.calls()
        stop_index = next(i for i, item in enumerate(calls) if item[0] == 'launchctl' and item[1][0] == 'bootout')
        cleanup_index = next(i for i, item in enumerate(calls) if item[0] == 'ssh' and i > stop_index)
        self.assertLess(stop_index, cleanup_index)
        self.assertFalse(any(name == 'ssh' and '-N' in args for name, args in calls))
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())

    def test_failed_stop_during_rollback_preserves_receipt_config_and_unit(self):
        env = dict(self.env, TEST_PRINT_FAIL='1', TEST_BOOTOUT_FAIL='1')
        result = self.run_script(['install'] + self.common, success=False, env=env)
        self.assertIn('installation conservée', result.stderr)
        self.assertTrue(self.runner.exists())
        self.assertTrue((self.install_dir / 'config').exists())
        self.assertTrue((self.install_dir / 'receipt').exists())
        self.assertTrue(self.plist.exists())
        self.assertTrue((self.root / 'service.active').exists())

    def test_ambiguous_remote_cleanup_during_rollback_preserves_install(self):
        remote_socket = str(self.remote / 'peer.sock')
        env = dict(
            self.env,
            TEST_PRINT_FAIL='1',
            TEST_CREATE_REMOTE_STALE=remote_socket,
            TEST_REMOTE_PROBE_EXIT='22',
        )
        result = self.run_script(['install'] + self.common, success=False, env=env)
        self.assertIn('installation conservée', result.stderr)
        self.assertTrue(self.runner.exists())
        self.assertTrue((self.install_dir / 'config').exists())
        self.assertTrue((self.install_dir / 'receipt').exists())
        self.assertTrue(self.plist.exists())
        self.assertTrue((self.remote / 'peer.sock').exists())
        self.assertFalse((self.root / 'service.active').exists())
        self.assertFalse(any(name == 'ssh' and '-N' in args for name, args in self.calls()))

    def test_term_exits_143_then_exit_trap_rolls_back_once(self):
        env = dict(self.env, TEST_SIGNAL_ON_BOOTSTRAP='1')
        result = self.run_script(['install'] + self.common, success=False, env=env)
        self.assertEqual(result.returncode, 143, result.stdout + result.stderr)
        bootouts = [item for item in self.calls() if item[0] == 'launchctl' and item[1][0] == 'bootout']
        self.assertEqual(len(bootouts), 1)
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())

    def test_tampered_install_refuses_status_and_remove_before_mutation(self):
        self.install()
        config = self.install_dir / 'config'
        config.write_text(config.read_text() + 'host=$(touch /tmp/injected)\n')
        inode = self.plist.stat().st_ino
        self.clear_calls()
        self.run_script(['status', '--label', 'fixture'], success=False)
        self.run_script(['remove', '--label', 'fixture'], success=False)
        self.assertEqual(self.plist.stat().st_ino, inode)
        self.assertTrue((self.root / 'service.active').exists())
        self.assertEqual(self.calls(), [])
        self.assertNotIn('eval ', self.runner.read_text())

    def test_service_run_recovers_only_private_econnrefused_socket(self):
        self.install()
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        old_inode = (self.remote / 'peer.sock').stat().st_ino
        self.clear_calls()
        self.run_script(['service-run', '--config', str(self.install_dir / 'config')], installed=True)
        self.assertFalse((self.remote / 'peer.sock').exists())
        calls = self.calls()
        self.assertEqual([name for name, _ in calls], ['ssh', 'ssh'])
        self.assertIn('StreamLocalBindUnlink=no', calls[-1][1])
        self.assertGreater(old_inode, 0)

    def test_service_run_refuses_socket_replaced_between_probe_and_unlink(self):
        self.install()
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        old_inode = (self.remote / 'peer.sock').stat().st_ino
        hooks = self.root / 'python-hooks'
        hooks.mkdir(mode=0o700)
        (hooks / 'sitecustomize.py').write_text(
            "import os, socket\n"
            "_target=os.environ.get('TEST_TOCTOU_PATH')\n"
            "_real=os.lstat\n"
            "_count=0\n"
            "def _checked(path, *args, **kwargs):\n"
            "    global _count\n"
            "    if _target and os.fspath(path) == _target:\n"
            "        _count += 1\n"
            "        if _count == 2:\n"
            "            os.rename(_target, _target + '.inode-held')\n"
            "            endpoint=socket.socket(socket.AF_UNIX)\n"
            "            endpoint.bind(_target)\n"
            "            endpoint.close()\n"
            "    return _real(path, *args, **kwargs)\n"
            "os.lstat=_checked\n"
        )
        env = dict(
            self.env,
            PYTHONPATH=str(hooks),
            TEST_TOCTOU_PATH=str(self.remote / 'peer.sock'),
        )
        self.run_script(
            ['service-run', '--config', str(self.install_dir / 'config')],
            success=False,
            env=env,
            installed=True,
        )
        self.assertTrue((self.remote / 'peer.sock').exists())
        self.assertNotEqual((self.remote / 'peer.sock').stat().st_ino, old_inode)
        self.assertFalse(any(name == 'ssh' and '-N' in args for name, args in self.calls()))

    def test_install_refuses_initial_stale_before_local_publication(self):
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        inode = (self.remote / 'peer.sock').stat().st_ino
        self.run_script(['install'] + self.common, success=False)
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())
        self.assertFalse((self.root / 'service.active').exists())

    def test_run_089_still_refuses_stale_socket_without_unlink(self):
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        inode = (self.remote / 'peer.sock').stat().st_ino
        self.run_script(['run'] + self.common, success=False)
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)

    def test_service_run_refuses_live_socket_and_ambiguous_probe(self):
        self.install()
        live = socket.socket(socket.AF_UNIX)
        live.bind(str(self.remote / 'peer.sock'))
        live.listen(1)
        inode = (self.remote / 'peer.sock').stat().st_ino
        self.run_script(['service-run', '--config', str(self.install_dir / 'config')], success=False, installed=True)
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)
        live.close()
        env = dict(self.env, TEST_REMOTE_PROBE_EXIT='22')
        self.run_script(['service-run', '--config', str(self.install_dir / 'config')], success=False, env=env, installed=True)
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)

    def test_status_never_touches_remote_socket(self):
        self.install()
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        inode = (self.remote / 'peer.sock').stat().st_ino
        self.clear_calls()
        self.run_script(['status', '--label', 'fixture'])
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)
        self.assertFalse(any(name == 'ssh' for name, _ in self.calls()))

    def test_remove_stops_then_cleans_stale_without_second_forward(self):
        self.install()
        stale = socket.socket(socket.AF_UNIX)
        stale.bind(str(self.remote / 'peer.sock'))
        stale.close()
        self.clear_calls()
        self.run_script(['remove', '--label', 'fixture'])
        calls = self.calls()
        stop_index = next(i for i, item in enumerate(calls) if item[0] == 'launchctl' and item[1][0] == 'bootout')
        probe_index = next(i for i, item in enumerate(calls) if item[0] == 'ssh')
        self.assertLess(stop_index, probe_index)
        self.assertFalse(any(name == 'ssh' and '-N' in args for name, args in calls))
        self.assertFalse((self.remote / 'peer.sock').exists())
        self.assertFalse(self.install_dir.exists())

    def test_remove_succeeds_when_remote_root_and_socket_are_already_absent(self):
        self.install()
        self.remote.rmdir()
        self.clear_calls()
        self.run_script(['remove', '--label', 'fixture'])
        self.assertFalse(self.install_dir.exists())
        self.assertFalse(self.plist.exists())
        self.assertFalse(self.remote.exists())
        self.assertFalse(any(name == 'ssh' and '-N' in args for name, args in self.calls()))

    def test_remove_live_socket_refuses_and_keeps_attested_files(self):
        self.install()
        live = socket.socket(socket.AF_UNIX)
        live.bind(str(self.remote / 'peer.sock'))
        live.listen(1)
        inode = (self.remote / 'peer.sock').stat().st_ino
        self.run_script(['remove', '--label', 'fixture'], success=False)
        self.assertEqual((self.remote / 'peer.sock').stat().st_ino, inode)
        self.assertTrue(self.runner.exists())
        self.assertTrue(self.plist.exists())
        self.assertFalse((self.root / 'service.active').exists())
        live.close()

    def test_old_positional_admin_surface_remains_rejected(self):
        for action in ['install', 'status', 'remove']:
            self.run_script([action, 'historique'], success=False)


unittest.main(verbosity=2)
PY
