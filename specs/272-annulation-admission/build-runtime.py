#!/usr/bin/env python3
"""Reconstruire 272 depuis 271, sans retirer les modes Loop ni le stockage externe."""
import argparse
from pathlib import Path
import subprocess
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
repo=Path(__file__).resolve().parents[2]
subprocess.run(['python3',str(repo/'specs/271-guidage-messages/build-runtime.py'),'--output',str(args.output)],check=True)
subprocess.run(['patch','-p1','-F','0','--batch','--forward','-i',str(Path(__file__).with_name('runtime.patch'))],cwd=args.output,check=True)
print(args.output)
