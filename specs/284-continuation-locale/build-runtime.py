#!/usr/bin/env python3
"""Reconstruire les lots 283 et 284 depuis le runtime 282 qualifié."""
import argparse
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
r = Path(__file__).resolve().parents[2]
subprocess.run(['python3', str(r/'specs/282-trafic-inventaire/build-runtime.py'), '--output', str(a.output)], check=True)
subprocess.run(['patch', '-p1', '-F', '0', '--batch', '--forward', '-i', str(Path(__file__).with_name('runtime.patch'))], cwd=a.output, check=True)
print(a.output)
