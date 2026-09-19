#!/usr/bin/env bash
# Tests de scripts : SSH, Git et Cargo sont des doublures ; un oracle exerce
# aussi le parseur du vrai rsync avec un faux shell SSH, sans transfert réseau.
# Aucun serveur SSH, fournisseur, daemon, socket ou fichier utilisateur réel.
set -euo pipefail
umask 077
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
exec python3 - "$SCRIPT_DIR" <<'PY'
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
REAL_RSYNC = shutil.which('rsync')

# Les doubles SSH exécutent le programme distant dans un répertoire privé
# local : on teste vraiment le quoting, les gardes et pipefail, pas un succès
# programmé en lieu et place de la logique du déploiement.
FAKE = r'''
import json, os, pathlib, shutil, subprocess, sys
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
with open(os.environ['TEST_LOG'], 'a') as log:
    log.write(json.dumps([name,args])+'\n')
if name == 'ssh':
    required=['BatchMode=yes','StrictHostKeyChecking=yes','ControlMaster=no',
      'ControlPath=none','ControlPersist=no','ExitOnForwardFailure=yes',
      'ServerAliveInterval=15','ServerAliveCountMax=3','ConnectTimeout=10',
      'ConnectionAttempts=1','StreamLocalBindUnlink=no','StreamLocalBindMask=0177',
      'ForwardAgent=no','PermitLocalCommand=no']
    assert all(item in args for item in required), args
    assert args[:2] == ['-F','/dev/null'], args
    assert 'GlobalKnownHostsFile=/dev/null' in args and 'UpdateHostKeys=no' in args
    known=[arg for arg in args if arg.startswith('UserKnownHostsFile=')]
    assert len(known)==1 and known[0].startswith('UserKnownHostsFile="') and known[0].endswith('"'), known
    if os.environ.get('TEST_RSYNC_ARGV_ONLY'):
        # Le vrai rsync a exercé son parseur -e ; aucun serveur ni transfert
        # n'est lancé. La fermeture sans handshake est attendue par la sonde.
        sys.exit(0)
    if '-N' in args:
        assert args.count('-R') == 1
        sys.exit(int(os.environ.get('TEST_FORWARD_EXIT','0')))
    assert args[-1].startswith('bash -s -- '), args
    result=subprocess.run(['/bin/bash','-c',args[-1]],input=sys.stdin.buffer.read())
    sys.exit(result.returncode)
if name == 'git':
    assert args[:2] == ['-C',os.environ['TEST_SOURCE']]
    if args[2:] == ['rev-parse','--show-toplevel']: print(os.environ['TEST_SOURCE'])
    elif args[2:] == ['rev-parse','HEAD']: print('a'*40)
    elif args[2] == 'status': pass
    elif args[2:] == ['ls-files','-z']: sys.stdout.buffer.write(b'Cargo.toml\0Cargo.lock\0')
    else: raise AssertionError(args)
elif name == 'rsync':
    assert '--delete' not in args
    assert '--no-links' in args and '--chmod=Du=rwx,Dgo=,Fu=rw,Fgo=' in args
    ssh_args=args[args.index('-e')+1]
    for flag in ['StrictHostKeyChecking=yes','BatchMode=yes','ControlPath=none']:
        assert flag in ssh_args, ssh_args
    file_list=sys.stdin.buffer.read()
    if os.environ.get('TEST_RSYNC_ARGV_PROBE'):
        # Conserver les arguments EXACTS du déploiement, dont -e et les espaces.
        # PATH commence par tools/ : ssh reste impérativement notre doublure.
        observed=subprocess.run([os.environ['TEST_REAL_RSYNC'],'--dry-run']+args,
          input=file_list,capture_output=True,timeout=5,
          env=dict(os.environ,TEST_RSYNC_ARGV_ONLY='1'))
        assert observed.returncode != 0, 'le faux shell ne parle pas le protocole rsync'
    destination=args[-1].split(':',1)[1]
    assert destination.startswith("'") and destination.endswith("'"), destination
    destination=pathlib.Path(destination[1:-1])
    source=pathlib.Path(args[-2])
    for raw in file_list.split(b'\0'):
        if raw:
            rel=os.fsdecode(raw)
            shutil.copyfile(source/rel,destination/rel)
            os.chmod(destination/rel,0o600)
elif name == 'cargo':
    if args == ['--version']:
        print('cargo fixture'); sys.exit(0)
    assert args == ['build','--locked','--offline','--release','-p','bridget-daemon','--bin','bridget'],args
    assert os.environ['RUSTUP_AUTO_INSTALL']=='0'
    assert os.environ['CARGO_NET_OFFLINE']=='true'
    assert os.environ['BRIDGET_BUILD_ID']=='a'*12
    if os.environ.get('TEST_BUILD_FAIL'):
        print('ERREUR_BUILD_FIXTURE'); sys.exit(23)
    binary=pathlib.Path('target/release/bridget')
    binary.parent.mkdir(parents=True)
    binary.write_text('#!/bin/sh\nexit 0\n')
    binary.chmod(0o700)
'''

class Federation089(unittest.TestCase):
    def setUp(self):
        self.root=Path(tempfile.mkdtemp(prefix='bf-',dir='/private/tmp' if Path('/private/tmp').is_dir() else '/tmp')).resolve()
        self.root.chmod(0o700)
        for name in ['home','local','remote','tools','source']:
            (self.root/name).mkdir(mode=0o700)
        self.local=self.root/'local'
        self.remote=self.root/'remote'
        self.identity=self.root/'identity file'
        self.known=self.root/'known hosts'
        for path in [self.identity,self.known]:
            path.write_text('SENTINELLE_FIXTURE\n'); path.chmod(0o600)
        for name in ['ssh','rsync','git','cargo']:
            path=self.root/'tools'/name
            path.write_text('#!'+PYTHON+'\n'+FAKE); path.chmod(0o700)
        for name in ['Cargo.toml','Cargo.lock','secret-untracked']:
            (self.root/'source'/name).write_text('fixture\n')
        self.sock=socket.socket(socket.AF_UNIX)
        self.sock.bind(str(self.local/'master.sock'))
        (self.local/'master.sock').chmod(0o600)
        self.env={'PATH':str(self.root/'tools')+':/usr/bin:/bin','HOME':str(self.root/'home'),
          'TEST_LOG':str(self.root/'calls.jsonl'),'TEST_SOURCE':str(self.root/'source')}
        self.common=['--label','fixture','--host','test.invalid','--user','fixture',
          '--port','2222','--identity',str(self.identity),'--known-hosts',str(self.known)]

    def tearDown(self):
        self.sock.close()
        # Seulement la racine créée par ce cas, jamais de glob ni racine HOME.
        shutil.rmtree(self.root)

    def run_script(self, name, extra, success=True, env=None):
        command=['/bin/bash',str(SCRIPTS/name)]+extra
        result=subprocess.run(command,env=env or self.env,text=True,capture_output=True,timeout=15)
        if success: self.assertEqual(result.returncode,0,result.stdout+result.stderr)
        else: self.assertNotEqual(result.returncode,0,result.stdout+result.stderr)
        return result

    def tunnel(self, extra=None, success=True, env=None):
        return self.run_script('federate-ssh.sh',['run']+self.common+[
          '--root',str(self.local),'--socket',str(self.local/'master.sock'),
          '--remote-root',str(self.remote),'--remote-socket',str(self.remote/'peer.sock')]+(extra or []),success,env)

    def deploy(self, extra=None, success=True, env=None):
        return self.run_script('deploy-remote.sh',self.common+[
          '--source',str(self.root/'source'),'--remote-prefix',str(self.root/'release space'),
          '--remote-cargo',str(self.root/'tools/cargo')]+(extra or []),success,env)

    def calls(self):
        path=self.root/'calls.jsonl'
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_dry_run_sans_ecriture_ni_ssh_rsync_git(self):
        before={str(p.relative_to(self.root)):(p.lstat().st_mode,p.read_bytes() if p.is_file() else None) for p in self.root.rglob('*')}
        self.tunnel(['--dry-run']); self.deploy(['--dry-run'])
        after={str(p.relative_to(self.root)):(p.lstat().st_mode,p.read_bytes() if p.is_file() else None) for p in self.root.rglob('*')}
        self.assertEqual(before,after)
        self.assertEqual(self.calls(),[])

    def test_helpers_sourceables_sans_effet(self):
        result=subprocess.run(['/bin/bash','-c','source "$1"', 'fixture',str(SCRIPTS/'federate-ssh.sh')],env=self.env,text=True,capture_output=True,timeout=5)
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(result.stdout+result.stderr,'')
        self.assertEqual(self.calls(),[])

    def test_tunnel_unique_et_options_ssh_strictes(self):
        result=self.tunnel()
        self.assertIn('BRIDGET_CHANNEL=ssh-unix',result.stderr)
        calls=self.calls()
        self.assertEqual([name for name,_ in calls],['ssh','ssh'])
        self.assertIn(str(self.remote/'peer.sock')+':'+str(self.local/'master.sock'),calls[1][1])

    def test_forwarding_echec_est_echec_pas_succes(self):
        self.tunnel(success=False,env=dict(self.env,TEST_FORWARD_EXIT='255'))

    def test_socket_bind_umask_077_privee_acceptee(self):
        (self.local/'master.sock').chmod(0o700)
        self.tunnel()
        self.assertEqual(stat.S_IMODE((self.local/'master.sock').stat().st_mode),0o700)

    def test_socket_occupee_intacte(self):
        path=self.remote/'peer.sock'
        path.write_bytes(b'SENTINELLE')
        self.tunnel(success=False)
        self.assertEqual(path.read_bytes(),b'SENTINELLE')
        self.assertEqual(len(self.calls()),1)

    def test_socket_stale_refusee_sans_effacement(self):
        path=self.remote/'peer.sock'
        stale=socket.socket(socket.AF_UNIX); stale.bind(str(path)); stale.close()
        inode=path.stat().st_ino
        self.tunnel(success=False)
        self.assertTrue(stat.S_ISSOCK(path.stat().st_mode))
        self.assertEqual(path.stat().st_ino,inode)

    def test_reprise_meme_chemin_seulement_si_socket_absente(self):
        # Modèle explicite du post-nettoyage : ce double ne prouve PAS que
        # sshd nettoie une vraie socket lors d'une coupure (gate P3 séparée).
        self.tunnel(); self.tunnel()
        forwards=[args for name,args in self.calls() if name=='ssh' and '-N' in args]
        self.assertEqual(forwards[0],forwards[1])

    def test_creation_namespace_distant_prive(self):
        root=self.root/'new state'
        self.tunnel(['--remote-root',str(root),'--remote-socket',str(root/'peer.sock')])
        self.assertEqual(stat.S_IMODE(root.stat().st_mode),0o700)

    def test_repertoire_distant_non_prive_refuse_sans_chmod(self):
        self.remote.chmod(0o755)
        self.tunnel(success=False)
        self.assertEqual(stat.S_IMODE(self.remote.stat().st_mode),0o755)

    def test_symlink_distant_et_parent_refuses(self):
        (self.remote/'peer.sock').symlink_to(self.identity)
        self.tunnel(success=False)
        (self.remote/'peer.sock').unlink()
        link=self.root/'linked'; link.symlink_to(self.remote,target_is_directory=True)
        self.tunnel(['--remote-root',str(link),'--remote-socket',str(link/'peer.sock')],success=False)
        self.assertEqual(self.identity.read_text(),'SENTINELLE_FIXTURE\n')

    def test_cibles_historiques_et_globales_refusees_avant_ssh(self):
        for root in ['/home/fixture/.cache/bridget','/home/fixture/.config/bridget','/home/fixture/.local/bin','/tmp/bridget','/private/tmp/bridget','/home/fixture/.ssh','/Users/user/projets/bridget']:
            self.tunnel(['--remote-root',root,'--remote-socket',root+'/s.sock','--dry-run'],success=False)
            self.deploy(['--remote-prefix',root,'--dry-run'],success=False)
        self.assertEqual(self.calls(),[])

    def test_injection_et_arguments_malformes_refuses(self):
        for flag,value in [('--host','-oProxyCommand=evil'),('--user','x;touch /tmp/injected'),('--port','0'),('--port','65536'),('--label','../prod'),('--remote-root','relative'),('--remote-root','/tmp/../etc'),('--remote-root','/tmp/$(touch /tmp/injected)'),('--remote-root',"/tmp/quote'"),('--remote-root','/tmp/a\nb')]:
            self.tunnel([flag,value,'--dry-run'],success=False)
        self.run_script('federate-ssh.sh',['run','--root'],success=False)
        self.assertEqual(self.calls(),[])

    def test_socket_hors_namespace_refusee(self):
        self.tunnel(['--remote-socket',str(self.root/'other.sock'),'--dry-run'],success=False)
        self.assertEqual(self.calls(),[])

    def test_cle_non_privee_ou_liee_refusee(self):
        self.identity.chmod(0o644); self.tunnel(['--dry-run'],success=False)
        self.identity.chmod(0o600)
        linked=self.root/'identity-link'; linked.symlink_to(self.identity)
        self.tunnel(['--identity',str(linked),'--dry-run'],success=False)
        self.assertEqual(self.calls(),[])

    def test_socket_maitre_non_privee_refusee_avant_ssh(self):
        (self.local/'master.sock').chmod(0o666)
        self.tunnel(success=False)
        self.assertEqual(stat.S_IMODE((self.local/'master.sock').stat().st_mode),0o666)
        self.assertEqual(self.calls(),[])

    def test_prefixe_lie_refuse_sans_ecrasement(self):
        (self.root/'release space').symlink_to(self.remote,target_is_directory=True)
        self.deploy(success=False)
        self.assertEqual(list(self.remote.iterdir()),[])
        self.assertFalse(any(name=='rsync' for name,_ in self.calls()))

    def test_ancienne_surface_refusee(self):
        for action in ['install','status','remove','invalid']:
            self.run_script('federate-ssh.sh',[action,'historique'],success=False)
        self.run_script('deploy-remote.sh',['fixture@test.invalid','22','daemon'],success=False)
        self.assertEqual(self.calls(),[])

    def test_deploiement_client_only_prive_sans_non_suivis(self):
        self.deploy()
        prefix=self.root/'release space'
        self.assertEqual(stat.S_IMODE((prefix/'bin/bridget').stat().st_mode),0o700)
        self.assertEqual(stat.S_IMODE((prefix/'source/Cargo.toml').stat().st_mode),0o600)
        self.assertFalse((prefix/'source/secret-untracked').exists())
        self.assertEqual(list((self.root/'home').iterdir()),[])

    def test_vrai_parseur_rsync_preserve_les_arguments_ssh_avec_espaces(self):
        self.assertIsNotNone(REAL_RSYNC, 'rsync réel requis pour cet oracle de quoting')
        self.deploy(env=dict(self.env,TEST_RSYNC_ARGV_PROBE='1',TEST_REAL_RSYNC=REAL_RSYNC))
        # Deux appels bash distants encadrent rsync ; le troisième est celui
        # lancé par le VRAI parseur -e et se ferme avant tout échange réseau.
        probes=[args for name,args in self.calls()
          if name=='ssh' and not args[-1].startswith('bash -s -- ')]
        self.assertEqual(len(probes),1,self.calls())
        args=probes[0]
        self.assertEqual(args[args.index('-i')+1],str(self.identity))
        self.assertIn('UserKnownHostsFile="'+str(self.known)+'"',args)
        self.assertIn('StrictHostKeyChecking=yes',args)
        # Mutation : revenir à printf %q coupe identity\\ file en deux argv
        # et/ou déforme les quotes de known_hosts ; ces assertions échouent.

    def test_prefixe_occupe_refuse_sans_ecrasement(self):
        prefix=self.root/'release space'; prefix.mkdir(mode=0o700)
        (prefix/'sentinel').write_bytes(b'intact')
        self.deploy(success=False)
        self.assertEqual((prefix/'sentinel').read_bytes(),b'intact')
        self.assertFalse(any(name=='rsync' for name,_ in self.calls()))

    def test_toolchain_absente_refusee_avant_creation(self):
        self.deploy(['--remote-cargo',str(self.root/'missing')],success=False)
        self.assertFalse((self.root/'release space').exists())
        self.assertFalse(any(name=='rsync' for name,_ in self.calls()))

    def test_echec_build_ne_devient_pas_succes_du_tail(self):
        result=self.deploy(success=False,env=dict(self.env,TEST_BUILD_FAIL='1'))
        self.assertIn('ERREUR_BUILD_FIXTURE',result.stdout)
        self.assertNotIn('Client installé',result.stdout)
        self.assertFalse((self.root/'release space/bin/bridget').exists())

unittest.main(verbosity=2)
PY
