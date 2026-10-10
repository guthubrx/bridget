#!/usr/bin/env python3
"""r6_daemon.py - démarre / arrête le daemon FIXTURE r6 (session 149, smoke réel r8).

Différences avec run_fixture_daemon.sh :
  - environnement MINIMAL explicite (le shell du testeur porte BRIDGET_HOME de la prod et des
    jetons CLAUDE_CODE_* qui ne doivent JAMAIS atteindre un fixture) ;
  - session propre (start_new_session) : le daemon survit à la commande appelante ;
  - arrêt : SIGTERM à UN PID, vérifié (commande = « daemon », BRIDGET_HOME = fixture, propriétaire,
    pas Firefox), attente bornée 3 s par défaut, jamais -9, jamais groupe.
Sous-commandes : start | stop. Aucun port TCP n'est demandé : socket Unix du fixture seulement.
"""
import argparse
import os
import signal
import subprocess
import sys
import time

PROD = "/Users/moi/.cache/bridget-core"


def ps_env_command(pid: int) -> str:
    return subprocess.run(["ps", "-Eww", "-p", str(pid), "-o", "command="],
                          capture_output=True, text=True).stdout


def alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
        return True
    except ProcessLookupError:
        return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["start", "stop"])
    parser.add_argument("--fixture-root", required=True)
    parser.add_argument("--bin149", required=True)
    parser.add_argument("--wait", type=float, default=3.0, help="attente après SIGTERM (s)")
    options = parser.parse_args()

    fixture = os.path.realpath(options.fixture_root)
    home = os.path.join(fixture, "home")
    if fixture.startswith(PROD) or home.startswith(PROD) or "/.config/bridget" in fixture:
        print("REFUS : fixture = production")
        return 2
    pidfile = os.path.join(fixture, "state", "daemon.pid")
    log_path = os.path.join(fixture, "logs", "daemon-stderr.log")

    if options.action == "start":
        os.umask(0o077)
        sock = os.path.join(home, "bridget.sock")
        if os.path.exists(sock):
            print(f"REFUS : socket déjà présent {sock}")
            return 2
        tmpdir = os.path.join(fixture, "tmp")
        os.makedirs(tmpdir, mode=0o700, exist_ok=True)
        env = {
            "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
            "HOME": os.environ.get("HOME", os.path.expanduser("~")),
            "TERM": "xterm-256color",
            "LANG": os.environ.get("LANG", "fr_FR.UTF-8"),
            "TMPDIR": tmpdir,
            "BRIDGET_HOME": home,
            "BRIDGET_SOCKET": sock,
        }
        log_fd = os.open(log_path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
        proc = subprocess.Popen([options.bin149, "daemon"], env=env, stdin=subprocess.DEVNULL,
                                stdout=log_fd, stderr=log_fd, start_new_session=True, cwd=fixture)
        with open(os.open(pidfile, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as handle:
            handle.write(f"{proc.pid}\n")
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if os.path.exists(sock):
                break
            if proc.poll() is not None:
                print(f"ECHEC : daemon mort (code {proc.returncode}) ; voir {log_path}")
                return 1
            time.sleep(0.25)
        else:
            print("ECHEC : socket absent après 15 s")
            return 1
        print(f"OK daemon fixture pid={proc.pid} socket={sock} (env minimal, session propre)")
        return 0

    # stop
    with open(pidfile, encoding="utf-8") as handle:
        pid = int(handle.read().strip())
    if not alive(pid):
        print(f"daemon pid={pid} déjà arrêté")
        return 0
    text = ps_env_command(pid)
    if "firefox" in text.lower():
        print("REFUS : Firefox")
        return 2
    if f"BRIDGET_HOME={home}" not in text or " daemon" not in text:
        print(f"REFUS : pid={pid} n'est pas le daemon de ce fixture")
        return 2
    t0 = time.monotonic()
    os.kill(pid, signal.SIGTERM)
    while time.monotonic() - t0 < options.wait:
        if not alive(pid):
            print(f"OK daemon pid={pid} terminé {time.monotonic() - t0:.2f}s après SIGTERM")
            return 0
        time.sleep(0.1)
    print(f"ATTENTION pid={pid} encore vivant après {options.wait}s (pas de -9)")
    return 1


if __name__ == "__main__":
    sys.exit(main())
