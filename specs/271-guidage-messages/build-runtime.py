#!/usr/bin/env python3
"""Reconstruire 271 depuis 269, avec l’entretien du stockage livré en 270."""
import argparse
from pathlib import Path
import shutil
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
repo = Path(__file__).resolve().parents[2]
subprocess.run(['python3', str(repo / 'specs/269-refus-demarrage-steer/build-runtime.py'),
                '--output', str(args.output)], check=True)
subprocess.run(['patch', '-p1', '-F', '0', '--batch', '--forward', '-i',
                str(Path(__file__).with_name('runtime.patch'))], cwd=args.output, check=True)
for name in ['Makefile', 'scripts/build.py', 'scripts/install.sh']:
    shutil.copyfile(repo / name, args.output / name)
print(args.output)
