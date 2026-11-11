#!/usr/bin/env python3
"""Reconstruire les seules sources exécutables autorisées du correctif 263."""
import argparse
from pathlib import Path
import subprocess

BASE = '75259217a72b3177cc0f50ba58c1fc3f97e5e22e'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
repo = Path(__file__).resolve().parents[2]
out = args.output
if not out.is_absolute() or out.exists() or out.is_symlink():
    parser.error('--output doit être un nouveau chemin absolu')
out.mkdir(parents=True, mode=0o700)
archive = subprocess.Popen(['git', '-C', str(repo), 'archive', BASE], stdout=subprocess.PIPE)
try:
    subprocess.run(['tar', '-x', '-C', str(out)], stdin=archive.stdout, check=True)
finally:
    archive.stdout.close()
if archive.wait():
    raise SystemExit('git archive a échoué ; conserver la sortie pour diagnostic')
subprocess.run(['patch', '-p1', '-F', '0', '--batch', '--forward', '-i',
                str(Path(__file__).with_name('runtime.patch'))], cwd=out, check=True)
print(out)
