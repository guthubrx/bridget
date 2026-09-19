#!/usr/bin/env python3
"""Vrais daemon et wrapper `bridget claude`, faux fournisseur sous PTY.

Le faux `claude` n'imite pas l'interface : il journalise les octets qu'il
reçoit, sa taille de fenêtre et ses arguments. Les caractères PTY côté humain
sont émis exclusivement par ce harnais. Racines privées, aucun compte.
"""
import fcntl
import json
import os
import pathlib
import re
import resource
import select
import signal
import sqlite3
import struct
import subprocess
import sys
import tempfile
import termios
import time

AGENT_ID = "97000000-0000-4000-8000-000000000001"
PASTE_START = b"\x1b[200~"
PASTE_END = b"\x1b[201~"


def fake_claude():
    """Fournisseur factice : lit son terminal en mode brut, journalise tout."""
    log = pathlib.Path(os.environ["BRIDGET_FAKE_CLAUDE_LOG"])
    exit_code = int(os.environ.get("BRIDGET_FAKE_CLAUDE_EXIT", "0"))

    def record(kind, payload):
        with log.open("a", encoding="utf-8") as output:
            output.write(json.dumps({"kind": kind, "payload": payload}) + "\n")

    def window():
        rows, cols, _, _ = struct.unpack("HHHH", fcntl.ioctl(0, termios.TIOCGWINSZ, b"\0" * 8))
        return {"rows": rows, "cols": cols}

    record("args", sys.argv[1:])
    record("winsz", window())
    signal.signal(signal.SIGWINCH, lambda *_: record("winsz", window()))
    attributes = termios.tcgetattr(0)
    raw = termios.tcgetattr(0)
    raw[0] &= ~(termios.BRKINT | termios.ICRNL | termios.INPCK | termios.ISTRIP | termios.IXON)
    raw[1] &= ~termios.OPOST
    raw[3] &= ~(termios.ECHO | termios.ICANON | termios.IEXTEN | termios.ISIG)
    raw[6][termios.VMIN] = 1
    raw[6][termios.VTIME] = 0
    termios.tcsetattr(0, termios.TCSANOW, raw)
    os.write(1, b"FAKE-CLAUDE-READY\r\n")
    received = bytearray()
    transcript_written = [False]
    try:
        while True:
            try:
                ready, _, _ = select.select([0], [], [], 0.2)
            except InterruptedError:
                continue
            if not ready:
                continue
            data = os.read(0, 65536)
            if not data:
                break
            received.extend(data)
            record("rx", data.hex())
            os.write(1, b"ECHO:" + data.hex().encode() + b"\r\n")
            if PASTE_END in data and not transcript_written[0]:
                # Comme Claude Code : la session laisse un transcript sous
                # HOME/.claude/projects/<slug du cwd canonique>/.
                cwd = os.path.realpath(os.getcwd())
                slug = "".join(c if c.isalnum() or c == "-" else "-" for c in cwd)
                directory = pathlib.Path(os.environ["HOME"]) / ".claude" / "projects" / slug
                directory.mkdir(parents=True, exist_ok=True)
                # Comme Claude Code : le fichier porte l'identifiant de session imposé.
                session_id = next((sys.argv[i + 1] for i, a in enumerate(sys.argv) if a == "--session-id" and i + 1 < len(sys.argv)), "fake-session")
                (directory / f"{session_id}.jsonl").write_text(
                    json.dumps({"type": "user", "uuid": "fake-u1", "message": {"role": "user", "content": "QUESTION-FAKE-097"}}) + "\n"
                    + json.dumps({"type": "assistant", "uuid": "fake-a1", "message": {"role": "assistant", "model": "fake-model",
                        "stop_reason": "end_turn", "content": [{"type": "text", "text": "REPONSE-FAKE-097"}]}}) + "\n")
                transcript_written[0] = True
            if b"quit\r" in received:
                break
            if b"\x03" in received:
                exit_code = 130
                break
    finally:
        termios.tcsetattr(0, termios.TCSANOW, attributes)
        record("exit", exit_code)
    os._exit(exit_code)


def terminal_host():
    """Le shell d'une vraie fenêtre survit au programme qu'il lance (cf. 090)."""
    root = pathlib.Path(sys.argv[2])
    before = termios.tcgetattr(0)
    child = subprocess.Popen(sys.argv[3:])
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    (root / "wrapper.pid").write_text(str(child.pid))
    status = child.wait()
    after = termios.tcgetattr(0)
    for attributes in [before, after]:
        attributes[3] &= ~getattr(termios, "PENDIN", 0)
    (root / "terminal-status.json").write_text(json.dumps({"restored": before == after, "exit_code": status}))
    return status


def main():
    bridget = sys.argv[1]
    mode = sys.argv[2]
    root = pathlib.Path(tempfile.mkdtemp(prefix="b97-", dir="/tmp"))
    state, home, binaries = root / "state", root / "home", root / "bin"
    for directory in (state, home, binaries):
        directory.mkdir(mode=0o700)
    fake_log = root / "fake-claude.jsonl"
    launcher = binaries / "claude"
    launcher.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{pathlib.Path(__file__).resolve()}" --fake-claude "$@"\n')
    launcher.chmod(0o700)
    live = mode == "--live"
    environment = {"HOME": str(home), "USER": os.environ.get("USER", "moi"), "BRIDGET_HOME": str(state),
        "BRIDGET_SOCKET": str(state / "bridget.sock"), "TERM": "xterm-256color", "LANG": "en_US.UTF-8",
        "PATH": f"{binaries}:{pathlib.Path(bridget).parent}:/usr/bin:/bin",
        "BRIDGET_FAKE_CLAUDE_LOG": str(fake_log)}
    workdir = root
    if live:
        # Vrai Claude Code, compte de l'humain : HOME réel et USER portent la
        # session d'abonnement ; l'état Bridget reste privé. Répertoire de
        # travail explicite (déjà approuvé par l'humain), modèle économique.
        real_claude = pathlib.Path(os.environ["BRIDGET_CLAUDE_097_BIN"])
        workdir = pathlib.Path(os.environ["BRIDGET_CLAUDE_097_CWD"])
        environment["HOME"] = os.environ["HOME"]
        environment["PATH"] = f"{real_claude.parent}:{pathlib.Path(bridget).parent}:/usr/bin:/bin"
        environment.pop("BRIDGET_FAKE_CLAUDE_LOG")
        launcher = real_claude
    if mode == "--exit-code":
        environment["BRIDGET_FAKE_CLAUDE_EXIT"] = "7"
    (state / "agents.json").write_text(json.dumps({"agents": {"claude": {
        "command": str(launcher), "protocol": "claude_stream_json", "forbidden_env": ["ANTHROPIC_API_KEY"],
        "mcp": {"interactive": "claude", "acp_session": False}}}}))
    os.chmod(state / "agents.json", 0o600)
    daemon_log = open(root / "daemon.log", "wb")
    def start_daemon():
        return subprocess.Popen([bridget, "daemon"], env=environment, cwd=root,
            stdin=subprocess.DEVNULL, stdout=daemon_log, stderr=daemon_log)

    daemons = [start_daemon()]
    if live:
        deadline_seconds = 150
    master, slave = os.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
    wrapper = None
    transcript = bytearray()
    deadline = time.monotonic() + (150 if live else 60)

    pending = {"detail": "démarrage"}

    def tick():
        if time.monotonic() >= deadline:
            raise TimeoutError(f"budget global 097 dépassé pendant : {pending['detail']}")
        for fd in select.select([master], [], [], 0.025)[0]:
            try:
                data = os.read(fd, 65536)
            except OSError:
                return
            transcript.extend(data)
            (root / "terminal.log").write_bytes(transcript)

    def until(predicate, detail, allow_exit=False):
        pending["detail"] = detail
        while not predicate():
            if wrapper and wrapper.poll() is not None and not allow_exit:
                raise AssertionError(f"wrapper quitté {wrapper.returncode}: {detail}\n{terminal_text()}")
            tick()

    def terminal_text(start=0):
        text = transcript[start:].decode(errors="replace")
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", " ", text)
        return " ".join(text.split())

    def cli(*args, check=True):
        result = subprocess.run([bridget, *args], env=environment, cwd=root,
            stdin=subprocess.DEVNULL, capture_output=True, timeout=10)
        if check:
            assert result.returncode == 0, (args, result.stderr.decode())
        return result

    def fake_events():
        if not fake_log.exists():
            return []
        return [json.loads(line) for line in fake_log.read_text().splitlines() if line.strip()]

    def received_bytes():
        return bytes.fromhex("".join(e["payload"] for e in fake_events() if e["kind"] == "rx"))

    def own_terminal():
        os.setsid()
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

    def signal_wrapper(sig):
        pid = int((root / "wrapper.pid").read_text())
        os.kill(pid, sig)

    def presence():
        return [a for a in json.loads(cli("agents", "--json").stdout) if a.get("agent_id") == AGENT_ID]

    def query(sql, *params):
        with sqlite3.connect(state / "bridget.db") as database:
            return database.execute(sql, params).fetchall()

    def send(message_id, body, *extra):
        issued = str(int(time.time()))
        cli("send", "--to", AGENT_ID, "--id", message_id, "--issued-at", issued,
            "--issuer-scope", "fixture_097_pty_interactive_scope", *extra, "--", body)

    def wait_phase(message_id, expected):
        until(lambda: query("SELECT phase FROM send_deliveries WHERE idempotency_key=?", message_id) == [(expected,)],
            f"phase {expected} pour {message_id} : {query('SELECT phase FROM send_deliveries WHERE idempotency_key=?', message_id)}")

    def journal_events():
        return [json.loads(line) for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()]

    report = {}
    failed = True
    try:
        until(lambda: (state / "bridget.sock").exists(), "daemon prêt")
        wrapper_args = [bridget, "claude", "--agent-id", AGENT_ID]
        if mode == "--user-bypass":
            wrapper_args += ["--dangerously-skip-permissions"]
        if live:
            wrapper_args += ["--model", os.environ.get("BRIDGET_CLAUDE_097_MODEL", "claude-haiku-4-5-20251001")]
        wrapper = subprocess.Popen([sys.executable, __file__, "--terminal-host", str(root), *wrapper_args],
            env=environment, cwd=workdir, stdin=slave, stdout=slave, stderr=slave, preexec_fn=own_terminal)
        if live:
            # Recette réelle : présence, remise dans la vraie TUI, réponse lue au
            # journal (transcript), sortie propre, terminal restauré.
            until(lambda: any(a.get("agent_id") == AGENT_ID for a in json.loads(cli("agents", "--json").stdout)), "présence enregistrée")
            agent = next(a for a in json.loads(cli("agents", "--json").stdout) if a.get("agent_id") == AGENT_ID)
            report["presence"] = {k: agent.get(k) for k in ("agent_type", "transport", "mode", "location", "state")}
            assert agent["transport"] == "claude_pty" and agent["mode"] == "cli", agent
            # Le transcript Claude Code naît au premier message : l'invite doit
            # être visible avant l'envoi, le journal est vérifié après.
            until(lambda: "❯" in terminal_text(), "invite de la vraie TUI affichée")
            time.sleep(2)
            send("live-097", "Réponds uniquement par le mot PONG-097, sans utiliser d'outil.")
            wait_phase("live-097", "acked")
            until(lambda: any(e.get("event") == "update" and "PONG-097" in str(e.get("payload", {}).get("text", "")) for e in journal_events()),
                f"réponse PONG-097 de la vraie TUI au journal : {[e.get('event') for e in journal_events()][-8:]}")
            report["live"] = "message collé dans la vraie TUI, réponse au journal"
            # Sortie native : Ctrl-D sur invite vide (un `/exit` peut être une
            # commande utilisateur redéfinie, comme sur ce poste).
            os.write(master, b"\x04")
            exit_deadline = time.monotonic() + 8
            while wrapper.poll() is None and time.monotonic() < exit_deadline:
                tick()
            if wrapper.poll() is None:
                os.write(master, b"\x03")
                time.sleep(0.3)
                os.write(master, b"\x03")
            until(lambda: wrapper.poll() is not None, "Claude Code quitté (Ctrl-D puis Ctrl-C)", allow_exit=True)
            until(lambda: (root / "terminal-status.json").exists(), "statut terminal écrit", allow_exit=True)
            status = json.loads((root / "terminal-status.json").read_text())
            report["terminal"] = status
            assert status["restored"], status
            until(lambda: not any(a.get("agent_id") == AGENT_ID and a.get("state") == "connected"
                for a in json.loads(cli("agents", "--json").stdout)), "présence retirée", allow_exit=True)
            print(json.dumps(report, ensure_ascii=False))
            failed = False
            return
        # 1. Sortie de l'enfant relayée jusqu'au terminal de l'humain.
        until(lambda: b"FAKE-CLAUDE-READY" in transcript, "sortie du fournisseur relayée au terminal")
        # 2. Présence honnête : canal réel, jamais tmux.
        until(lambda: any(a.get("agent_id") == AGENT_ID for a in json.loads(cli("agents", "--json").stdout)), "présence enregistrée")
        agent = next(a for a in json.loads(cli("agents", "--json").stdout) if a.get("agent_id") == AGENT_ID)
        report["presence"] = {k: agent.get(k) for k in ("agent_type", "transport", "mode", "location", "state")}
        assert agent["agent_type"] == "claude", agent
        assert agent["transport"] == "claude_pty", f"transport réel attendu claude_pty : {agent}"
        assert agent["mode"] == "cli", agent
        assert agent.get("location") in (None, ""), agent
        # 3. Arguments : aucun bypass implicite ; le bypass explicite est relayé.
        args = next(e["payload"] for e in fake_events() if e["kind"] == "args")
        report["args"] = args
        if mode == "--user-bypass":
            assert "--dangerously-skip-permissions" in args, args
        else:
            assert not any("dangerously-skip-permissions" in a or "bypassPermissions" in a for a in args), args
        assert "--strict-mcp-config" in args and "--mcp-config" in args, args
        # 4. Frappe humaine relayée octet pour octet.
        os.write(master, b"hello")
        until(lambda: received_bytes().endswith(b"hello"), "frappe relayée à l'enfant")
        # 5. Redimensionnement : SIGWINCH suivi jusqu'au PTY de l'enfant.
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 100, 0, 0))
        until(lambda: {"rows": 40, "cols": 100} in [e["payload"] for e in fake_events() if e["kind"] == "winsz"],
            f"taille de fenêtre propagée : {[e['payload'] for e in fake_events() if e['kind'] == 'winsz']}")
        if mode == "--busy":
            # Saisie humaine partielle avant la remise : elle n'est ni effacée ni réécrite.
            os.write(master, b" saisie en cours")
            until(lambda: received_bytes().endswith(b" saisie en cours"), "saisie partielle relayée")
        # 6. Remise Bridget : collage encadré puis CR, accusé durable.
        send("request-097", "MISSION-097")
        wait_phase("request-097", "acked")
        until(lambda: received_bytes().endswith(b"\r") and PASTE_END in received_bytes(), "collage puis CR reçus")
        received = received_bytes()
        start = received.index(PASTE_START)
        end = received.index(PASTE_END, start)
        pasted = received[start + len(PASTE_START):end]
        assert b"MISSION-097" in pasted and b"request-097" in pasted, pasted
        assert received[end + len(PASTE_END):] == b"\r", received[end:]
        if mode == "--busy":
            assert received[:start].endswith(b" saisie en cours"), "la saisie précède le collage sans effacement"
        assert received.count(PASTE_START) == 1 and received.count(b"\r") == 1, received
        assert any(e.get("event") == "turn_start" and e.get("message_id") == "request-097" for e in journal_events()), "journal turn_start"
        if mode == "--basic":
            # Journal relayé : tours humain et assistant lus depuis le transcript.
            until(lambda: any(e.get("event") == "update" and e.get("payload", {}).get("text") == "REPONSE-FAKE-097" for e in journal_events()),
                f"texte assistant au journal : {[e.get('event') for e in journal_events()]}")
            assert any(e.get("event") == "turn_start" and e.get("payload", {}).get("from") == "human"
                and e.get("payload", {}).get("body") == "QUESTION-FAKE-097" for e in journal_events()), "tour humain au journal"
            assert any(e.get("event") == "turn_end" and e.get("message_id") == "fake-a1" for e in journal_events()), "fin de tour au journal"
            assert sum(1 for e in journal_events() if e.get("event") == "turn_start"
                and "MISSION-097" in e.get("payload", {}).get("body", "")) == 1, "remise Bridget journalisée une seule fois"
            report["journal"] = "tours humain et assistant relayés"
        if mode == "--busy":
            # Corps hostile : refusé par le CLI avant le daemon (H-001) ; le refus
            # au niveau du transport est prouvé par les tests unitaires de pty.rs.
            before = len(received_bytes())
            refused = cli("send", "--to", AGENT_ID, "--", "corps \x1b]52;c;x\x07 hostile", check=False)
            assert refused.returncode != 0 and b"contr" in refused.stderr, refused.stderr
            assert len(received_bytes()) == before, "aucun octet ne doit partir pour un corps refusé"
            # Notification : même voie que les messages.
            send("notice-097", "NOTICE-097")
            wait_phase("notice-097", "acked")
            until(lambda: received_bytes().count(PASTE_START) == 2, "notification remise par le PTY")
        if mode == "--daemon-restart":
            # Perte du daemon : le fournisseur continue ; la reconnexion garde
            # l'identité et la notification passe par la même voie PTY.
            daemons[0].terminate()
            daemons[0].wait(timeout=10)
            until(lambda: not (state / "bridget.sock").exists() or daemons[0].poll() is not None, "ancien daemon arrêté")
            time.sleep(0.5)
            assert wrapper.poll() is None, "le wrapper ne doit pas mourir avec le daemon"
            daemons.append(start_daemon())
            until(lambda: (state / "bridget.sock").exists(), "nouveau daemon prêt")
            until(lambda: any(a.get("state") == "connected" and a.get("transport") == "claude_pty" for a in presence()),
                f"reconnexion avec la même identité : {presence()}")
            until(lambda: received_bytes().count(PASTE_START) == 2 and b"reconnect" in received_bytes(),
                "notification de reconnexion remise par le PTY")
            send("request-097-apres-reprise", "APRES-REPRISE-097")
            wait_phase("request-097-apres-reprise", "acked")
            until(lambda: received_bytes().count(PASTE_START) == 3, "remise après reprise")
            report["daemon_restart"] = "identité conservée, notification et remise par le PTY"
        if mode == "--sigterm":
            # Signal reçu par le wrapper : relayé au fournisseur, terminal restauré.
            signal_wrapper(signal.SIGTERM)
            until(lambda: wrapper.poll() is not None, "wrapper terminé après SIGTERM", allow_exit=True)
            until(lambda: (root / "terminal-status.json").exists(), "statut terminal écrit", allow_exit=True)
            status = json.loads((root / "terminal-status.json").read_text())
            report["terminal"] = status
            assert status["restored"], status
            assert status["exit_code"] != 0, status
            until(lambda: not any(a.get("state") == "connected" for a in presence()), "présence retirée", allow_exit=True)
            assert not subprocess.run(["/usr/bin/pgrep", "-f", f"BRIDGET_FAKE_CLAUDE_LOG={fake_log}"], capture_output=True).stdout.strip(), "faux fournisseur survivant"
            print(json.dumps(report, ensure_ascii=False))
            failed = False
            return
        # 7. Fin : sortie du fournisseur → terminal restauré, présence retirée, code relayé.
        os.write(master, b"quit\r")
        until(lambda: wrapper.poll() is not None, "wrapper terminé après sortie du fournisseur", allow_exit=True)
        until(lambda: (root / "terminal-status.json").exists(), "statut terminal écrit", allow_exit=True)
        status = json.loads((root / "terminal-status.json").read_text())
        report["terminal"] = status
        assert status["restored"], status
        expected_exit = 7 if mode == "--exit-code" else 0
        assert status["exit_code"] == expected_exit, status
        until(lambda: not any(a.get("agent_id") == AGENT_ID and a.get("state") == "connected"
            for a in json.loads(cli("agents", "--json").stdout)), "présence retirée", allow_exit=True)
        assert not subprocess.run(["/usr/bin/pgrep", "-f", f"BRIDGET_FAKE_CLAUDE_LOG={fake_log}"], capture_output=True).stdout.strip(), "faux fournisseur survivant"
        print(json.dumps(report, ensure_ascii=False))
        failed = False
    finally:
        if failed:
            print("--- diagnostic 097 ---", file=sys.stderr)
            print("fake events:", fake_events()[-12:], file=sys.stderr)
            print("journal:", [(e.get("event"), str(e.get("payload"))[:80]) for e in journal_events()][-10:], file=sys.stderr)
            try:
                print("send_deliveries:", query("SELECT idempotency_key, phase FROM send_deliveries"), file=sys.stderr)
                print("ledger:", query("SELECT id, sender, target FROM ledger ORDER BY ts DESC LIMIT 3"), file=sys.stderr)
            except Exception as error:
                print("base illisible:", error, file=sys.stderr)
            print("terminal:", terminal_text()[-600:], file=sys.stderr)
            print("daemon.log:", (root / "daemon.log").read_text(errors="replace")[-1500:], file=sys.stderr)
        if wrapper and wrapper.poll() is None:
            wrapper.terminate()
            try:
                wrapper.wait(timeout=5)
            except subprocess.TimeoutExpired:
                wrapper.kill()
        for daemon in daemons:
            if daemon.poll() is None:
                daemon.terminate()
                try:
                    daemon.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    daemon.kill()
        os.close(master)
        os.close(slave)
        subprocess.run(["/usr/bin/pkill", "-f", f"BRIDGET_FAKE_CLAUDE_LOG={fake_log}"], capture_output=True)


if __name__ == "__main__":
    if sys.argv[1] == "--fake-claude":
        fake_claude()
    elif sys.argv[1] == "--terminal-host":
        sys.exit(terminal_host())
    else:
        main()
