#!/usr/bin/env python3
"""pty_probe.py — sonde PTY : lance argv dans un PTY, envoie des touches planifiées,
affiche l'écran nettoyé. Sert à franchir l'onboarding interactif sans modèle.
Usage : pty_probe.py --cwd D --duration S [--key T:HEX ...] -- argv...
Arrêt : SIGTERM au seul chef (jamais de groupe)."""
import argparse, os, pty, re, select, signal, sys, time
ap = argparse.ArgumentParser()
ap.add_argument("--cwd", required=True)
ap.add_argument("--duration", type=float, default=20)
ap.add_argument("--key", action="append", default=[], help="secondes:texte (\\r = Entrée)")
ap.add_argument("argv", nargs="+")
o = ap.parse_args()
keys = sorted((float(k.split(":", 1)[0]), k.split(":", 1)[1].encode().decode("unicode_escape").encode()) for k in o.key)
os.chdir(o.cwd)
pid, fd = pty.fork()
if pid == 0:
    os.execv(o.argv[0], o.argv)
t0 = time.monotonic(); buf = b""
while time.monotonic() - t0 < o.duration:
    while keys and time.monotonic() - t0 >= keys[0][0]:
        try: os.write(fd, keys.pop(0)[1])
        except OSError: keys.clear(); break
    r, _, _ = select.select([fd], [], [], 0.3)
    if r:
        try: buf += os.read(fd, 65536)
        except OSError: break
txt = buf.decode("utf-8", "replace")
txt = re.sub(r"\x1b\[[0-9;?]*[A-Za-z]|\x1b[78=>]|\x1b\][^\x07]*\x07", " ", txt).replace("\r", "")
print(re.sub(r"[ \t]+", " ", txt)[-2500:])
try: os.kill(pid, signal.SIGTERM)
except ProcessLookupError: pass
time.sleep(3)
try:
    os.kill(pid, 0); print("ATTENTION chef toujours vivant", pid)
except ProcessLookupError: print("chef terminé")
