#!/usr/bin/env python3
"""Installer local : reprend le plist canonique, remplace seulement son programme."""
import argparse
import importlib.util
import os
from pathlib import Path
import plistlib
import subprocess
import sys

DOSSIER=Path(__file__).resolve().parent
CANON=Path('/Users/moi/.codex/skills/agent-loop/scripts/install_launch_agent.py')

def construire(args):
    spec=importlib.util.spec_from_file_location('installer_agent_loop',CANON)
    canon=importlib.util.module_from_spec(spec);spec.loader.exec_module(canon)
    # Le canon crée uniquement le dossier des logs ; la commande print reste sans plist installé.
    plist=canon.build_plist(args)
    plist['ProgramArguments']=[args.python,str(DOSSIER/'heartbeat.py'),'--root',str(Path(args.root).expanduser()),'--run-id',args.run_id]
    return plist

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root',default='/Users/moi/Documents/opus2D/orchestration/runs')
    p.add_argument('--run-id',default='psychologie-120-20261004')
    p.add_argument('--interval',type=int,default=60)
    p.add_argument('--label',default='local.opus2d.psychologie-heartbeat')
    p.add_argument('--python',default=sys.executable)
    p.add_argument('--run-at-load',action='store_true')
    p.add_argument('--load',action='store_true',help='Seulement avec install : activer le LaunchAgent')
    p.add_argument('commande',choices=['print','install'])
    args=p.parse_args()
    if args.interval<1:p.error('interval doit être positif')
    if Path(args.run_id).name!=args.run_id or args.run_id in {'','.','..'}:p.error('run-id invalide')
    if Path(args.label).name!=args.label:p.error('label invalide')
    if args.load and args.commande!='install':p.error('--load exige install')
    plist=construire(args)
    if args.commande=='print':
        sys.stdout.buffer.write(plistlib.dumps(plist));return 0
    if not (Path(args.root).expanduser()/args.run_id/'run.json').is_file():p.error('Initialiser le run canonique avant installation')
    path=Path.home()/'Library'/'LaunchAgents'/f'{args.label}.plist'
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_bytes(plistlib.dumps(plist))
    print(path)
    if args.load:
        domaine=f'gui/{os.getuid()}'
        subprocess.run(['launchctl','bootstrap',domaine,str(path)],check=True)
        subprocess.run(['launchctl','print',f'{domaine}/{args.label}'],check=True)
    return 0

if __name__=='__main__':sys.exit(main())
