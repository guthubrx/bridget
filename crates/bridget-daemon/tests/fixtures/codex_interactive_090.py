#!/usr/bin/env python3
"""Vrais daemon, wrapper, app-server et TUI ; HTTP local déterministe.

Les caractères PTY sont émis exclusivement par ce harnais (humain simulé),
jamais par le produit. Délai global, racines privées, aucun compte utilisateur.
"""
import fcntl
import http.server
import importlib.util
import json
import os
import pathlib
import re
import resource
import select
import signal
import shutil
import socket
import sqlite3
import struct
import subprocess
import sys
import tempfile
import termios
import threading
import time

REPO = pathlib.Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location("shared_probe", REPO / "fixtures/probe_shared_session.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


def terminal_host():
    """Le shell d'une vraie fenêtre survit au programme qu'il lance.

    Darwin révoque le PTY quand son leader sort : mesurer termios DANS ce
    shell de test après sortie du wrapper, pas sur un fd déjà révoqué.
    """
    root = pathlib.Path(sys.argv[2])
    before = termios.tcgetattr(0)
    child = subprocess.Popen(sys.argv[3:])
    # Comme un shell attendant son job de premier plan : Ctrl-C appartient
    # au programme, pas au parent chargé de constater la restauration termios.
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    (root / "wrapper.pid").write_text(str(child.pid))
    status = child.wait()
    after = termios.tcgetattr(0)
    for attributes in [before, after]:
        attributes[3] &= ~getattr(termios, "PENDIN", 0)
    (root / "terminal-status.json").write_text(json.dumps({"restored": before == after, "exit_code": status}))
    return status


def main():
    bridget, codex = sys.argv[1:3]
    live = "--subscription" in sys.argv
    selection = next((mode for mode in ["--named-resume", "--menu-resume", "--menu-cancel", "--name-missing", "--name-ambiguous"] if mode in sys.argv), None)
    initial_resume = "--initial-resume" in sys.argv or selection in ["--named-resume", "--menu-resume"]
    copied_resume = "--copied-resume" in sys.argv
    paginated_resume = "--paginated-resume" in sys.argv
    human_resume = "--human-resume" in sys.argv
    root = pathlib.Path(tempfile.mkdtemp(prefix="b90-", dir="/tmp"))
    state, home = root / "state", root / "home"
    state.mkdir(mode=0o700)
    home.mkdir(mode=0o700)
    environment = {"HOME": str(home), "CODEX_HOME": str(home), "BRIDGET_HOME": str(state),
        "BRIDGET_SOCKET": str(state / "bridget.sock"), "TERM": "xterm-256color",
        "PATH": str(pathlib.Path(bridget).parent) + ":/opt/homebrew/bin:/usr/bin:/bin", "LANG": "en_US.UTF-8"}
    provider = http.server.ThreadingHTTPServer(("127.0.0.1", 0), probe.Fixture)
    if "--human-busy" in sys.argv:
        probe.Fixture.first_release = threading.Event()
    threading.Thread(target=provider.serve_forever, daemon=True).start()
    (home / "config.toml").write_text('model="fixture"\nmodel_provider="fixture"\ncheck_for_update_on_startup=false\n'
        'approval_policy="on-request"\nsandbox_mode="read-only"\n'
        '[model_providers.fixture]\nname="fixture"\nwire_api="responses"\nrequires_openai_auth=false\n'
        f'base_url="http://127.0.0.1:{provider.server_port}/v1"\n'
        f'[projects."{root}"]\ntrust_level="trusted"\n')
    os.chmod(home / "config.toml", 0o600)
    # Le registre décide du chemin du vrai binaire, pas un PATH caché du test.
    (state / "agents.json").write_text(json.dumps({"agents": {"codex": {
        "command": codex, "protocol": "codex_app_server", "args": ["app-server"],
        "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
        "mcp": {"interactive": "codex", "acp_session": False}}}}))
    os.chmod(state / "agents.json", 0o600)
    daemon_log = open(root / "daemon.log", "wb")
    daemon = subprocess.Popen([bridget, "daemon"], env=environment, cwd=root,
        stdin=subprocess.DEVNULL, stdout=daemon_log, stderr=daemon_log)
    master, slave = os.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
    wrapper = None
    attach = None
    attached = bytearray()
    observer = None
    peer = None
    peer_reader = None
    receipts = []
    transcript = bytearray()
    query_counts = {}
    deadline = time.monotonic() + (180 if live or human_resume else 80)

    def tick(enforce_deadline=True):
        if enforce_deadline and time.monotonic() >= deadline:
            raise TimeoutError("budget global 090 dépassé")
        descriptors = [master] + ([attach.stdout.fileno()] if attach and attach.poll() is None else [])
        for fd in select.select(descriptors, [], [], 0.025)[0]:
            data = os.read(fd, 65536)
            if fd == master:
                transcript.extend(data)
                (root / "terminal.log").write_bytes(transcript)
                # Réponses de terminal, pas décisions fournisseur. Les
                # requêtes peuvent être coupées entre deux read().
                for query, reply in {
                    b"\x1b[6n": b"\x1b[1;1R",
                    b"\x1b[?u": b"\x1b[?0u",
                    b"\x1b[c": b"\x1b[?1;2c",
                    b"\x1b]10;?\x1b\\": b"\x1b]10;rgb:ffff/ffff/ffff\x1b\\",
                    b"\x1b]11;?\x1b\\": b"\x1b]11;rgb:0000/0000/0000\x1b\\",
                }.items():
                    count = transcript.count(query)
                    for _ in range(count - query_counts.get(query, 0)):
                        os.write(master, reply)
                    query_counts[query] = count
            else:
                attached.extend(data)

    def until(predicate, detail):
        while not predicate():
            if wrapper and wrapper.poll() is not None:
                raise AssertionError(f"wrapper quitté {wrapper.returncode}: {detail}")
            tick()

    def terminal_text(start=0):
        # La TUI peut déplacer le curseur entre deux mots. L'oracle porte
        # sur le texte rendu, pas sur le découpage des séquences ANSI.
        text = transcript[start:].decode(errors="replace")
        text = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", " ", text)
        text = re.sub(r"\x1b\](?:[^\x1b\x07]|\x1b(?!\\))*(?:\x07|\x1b\\)", " ", text)
        return " ".join(text.split())

    def cli(*args):
        result = subprocess.run([bridget, *args], env=environment, cwd=root,
            stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        assert result.returncode == 0, (args, result.stderr.decode())
        return result.stdout.decode()

    def terminal_restored():
        return json.loads((root / "terminal-status.json").read_text())["restored"]

    def signal_wrapper(sig):
        pid = int((root / "wrapper.pid").read_text())
        subprocess.run(["/bin/ps", "-p", str(pid), "-o", "pid=,comm="], check=True)
        os.kill(pid, sig)

    def own_terminal():
        # Une vraie fenêtre de terminal fournit aussi /dev/tty. Sans terminal
        # contrôlant, crossterm peut lire ailleurs que le stdin PTY du harnais.
        os.setsid()
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        fcntl.ioctl(slave, termios.TIOCSCTTY, 0)

    try:
        if live:
            # Opt-in abonnement existant, sans fallback API. Toute opération
            # après la copie est désormais couverte par le finally d'effacement.
            auth_bytes = pathlib.Path(os.environ["BRIDGET_CODEX_090_AUTH"]).read_bytes()
            auth = json.loads(auth_bytes)
            assert auth.get("auth_mode") == "chatgpt" and not auth.get("OPENAI_API_KEY"), "recette abonnement uniquement"
            fd = os.open(home / "auth.json", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            with os.fdopen(fd, "wb") as output:
                output.write(auth_bytes)
            (home / "config.toml").write_text('check_for_update_on_startup=false\napproval_policy="on-request"\nsandbox_mode="read-only"\n'
                f'[projects."{root}"]\ntrust_level="trusted"\n')
        until(lambda: (state / "bridget.sock").exists(), "daemon prêt")
        resume_id = None
        if copied_resume:
            # Opt-in diagnostic : copie privée, aucun auth.json ni écriture
            # dans le fil source. Aucun tour n'est envoyé depuis cette copie.
            source = pathlib.Path(os.environ["BRIDGET_CODEX_090_ROLLOUT"])
            destination = home / "sessions" / "2026" / "01" / "01" / source.name
            destination.parent.mkdir(parents=True)
            shutil.copyfile(source, destination)
            with destination.open() as rollout:
                resume_id = json.loads(rollout.readline())["payload"]["id"]
        if "--resume-thread" in sys.argv or initial_resume or paginated_resume or selection:
            seed_path = root / "seed.sock"
            seed = subprocess.Popen([codex, "app-server", "--listen", "unix://" + str(seed_path)], env=environment,
                cwd=root, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                until(seed_path.exists, "serveur de création historique réel prêt")
                client = probe.Client(str(seed_path))
                started = client.rpc("thread/start", {"cwd": str(root),
                    "historyMode": "paginated" if paginated_resume else "legacy"})
                resume_id = started["result"]["thread"]["id"]
                assert "result" in client.rpc("thread/name/set", {"threadId": resume_id, "name": "HISTORIQUE-090"})
                if initial_resume or paginated_resume or selection:
                    assert "result" in client.rpc("turn/start", {"threadId": resume_id,
                        "input": [{"type": "text", "text": "HISTORIQUE-A-CONSERVER-090"}]})
                    seed_deadline = time.monotonic() + 10
                    while not any(event.get("method") == "turn/completed" for event in client.events):
                        client.socket.settimeout(max(0.001, seed_deadline - time.monotonic()))
                        client.events.append(client.receive())
                        assert time.monotonic() < seed_deadline
                    probe.Fixture.count = 0
                if selection == "--name-ambiguous":
                    duplicate = client.rpc("thread/start", {"cwd": str(root), "historyMode": "legacy"})["result"]["thread"]["id"]
                    assert "result" in client.rpc("thread/name/set", {"threadId": duplicate, "name": "HISTORIQUE-090"})
                    client.events.clear()
                    assert "result" in client.rpc("turn/start", {"threadId": duplicate,
                        "input": [{"type": "text", "text": "DEUXIEME-CONVERSATION-090"}]})
                    seed_deadline = time.monotonic() + 10
                    while not any(event.get("method") == "turn/completed" for event in client.events):
                        client.socket.settimeout(max(0.001, seed_deadline - time.monotonic()))
                        client.events.append(client.receive())
                        assert time.monotonic() < seed_deadline
                    probe.Fixture.count = 0
                client.socket.close()
            finally:
                subprocess.run(["/bin/ps", "-p", str(seed.pid), "-o", "pid=,comm="], check=False)
                seed.terminate()
                seed.wait(timeout=5)
                seed_path.unlink(missing_ok=True)
        wrapper_args = [bridget, "codex", "--agent-id", "90000000-0000-4000-8000-000000000001", "--no-alt-screen"]
        if human_resume:
            wrapper_args = [bridget, "codex", "--name", "gui-coder", "--yolo", "--no-alt-screen"]
        if initial_resume or copied_resume or paginated_resume:
            wrapper_args += ["--name", "coder-recette-090", "--yolo", "resume", resume_id]
        if copied_resume:
            wrapper_args += ["-m", "fixture"]
        if "--missing-resume" in sys.argv:
            wrapper_args += ["resume", "90000000-0000-4000-8000-000000000099"]
        if live:
            wrapper_args += ["-m", "gpt-5.6-luna"]
        if selection:
            wrapper_args = [bridget, "codex", "--agent-id", "90000000-0000-4000-8000-000000000001", "--name", "coder-recette-090", "--yolo", "--no-alt-screen", "resume"]
            if selection in ["--named-resume", "--name-ambiguous"]:
                wrapper_args += ["HISTORIQUE-090"]
            elif selection == "--name-missing":
                wrapper_args += ["NOM-ABSENT-090"]
        wrapper = subprocess.Popen([sys.executable, __file__, "--terminal-host", str(root), *wrapper_args],
            env=environment, cwd=root, stdin=slave, stdout=slave, stderr=slave,
            preexec_fn=own_terminal)
        if selection in ["--menu-resume", "--menu-cancel"]:
            until(lambda: b"Choix :" in transcript, "menu réel depuis thread/list")
            menu_children = subprocess.check_output(["/usr/bin/pgrep", "-P", (root / "wrapper.pid").read_text()]).decode().split()
            assert menu_children, "aucun app-server réel au menu"
            assert not json.loads(cli("agents", "--json")), "présence avant choix humain"
            menu_socket = next(state.glob("c-*.sock"))
            menu_observer = probe.Client(str(menu_socket))
            assert menu_observer.rpc("thread/loaded/list", {})["result"]["data"] == [], "fil provisoire créé pour le menu"
            menu_observer.socket.close()
            if selection == "--menu-cancel":
                os.write(master, b"\x03")
            else:
                os.write(master, b"9999\r")
                until(lambda: b"Choix invalide" in transcript, "choix invalide sans ouverture de fil")
                os.write(master, b"1\r")
        if selection in ["--menu-cancel", "--name-missing", "--name-ambiguous"]:
            until(lambda: wrapper.poll() is not None, "sélection refusée/annulée avant présence")
            tick()
            assert wrapper.returncode != 0
            assert not json.loads(cli("agents", "--json"))
            assert not list(state.glob("c-*.sock"))
            if selection == "--menu-cancel":
                for pid in menu_children:
                    assert subprocess.run(["/bin/ps", "-p", pid, "-o", "pid="], capture_output=True).returncode != 0, f"serveur survivant {pid}"
            assert terminal_restored() and probe.Fixture.count == 0
            expected = {"--menu-cancel": "reprise annulée", "--name-missing": "introuvable", "--name-ambiguous": "plusieurs"}[selection]
            assert expected in terminal_text(), terminal_text()
            print("selection_refused_or_cancelled_without_thread_presence_prompt", selection, flush=True)
            return
        if "--missing-resume" in sys.argv:
            until(lambda: wrapper.poll() is not None, "fil absent refusé avant présence")
            assert wrapper.returncode != 0
            assert not json.loads(cli("agents", "--json")), "présence fabriquée après échec de reprise"
            assert not list(state.glob("c-*.sock")), "socket privée survivante"
            assert terminal_restored()
            assert b"thread/resume" in transcript, transcript.decode(errors="replace")
            print("missing_resume_refused_without_presence_or_socket", flush=True)
            return
        until(lambda: bool(list(state.glob("c-*.sock"))), "socket Codex")
        until(lambda: b"Bridget :" in transcript, "inscription et journal prêts")
        socket_path = next(state.glob("c-*.sock"))
        observer = probe.Client(str(socket_path))
        threads = observer.rpc("thread/loaded/list", {})["result"]["data"]
        assert len(threads) == 1, threads
        thread_id = threads[0]
        if initial_resume or copied_resume or paginated_resume:
            assert thread_id == resume_id, "un nouveau fil a remplacé la reprise demandée"
        print("same_thread", thread_id, flush=True)
        until(lambda: (b"gpt-5.6-luna" if live else b"fixture") in transcript
            and (b"context" in transcript or b"shortcuts" in transcript), "TUI configurée (pas seulement l'écran Resuming) prête à saisir")
        if human_resume:
            original = next(a for a in json.loads(cli("agents", "--json")) if a["display_name"] == "gui-coder")
            original_id = original["agent_id"]

            def resolve_wire(frame):
                with socket.socket(socket.AF_UNIX) as connection:
                    connection.settimeout(3)
                    connection.connect(str(state / "bridget.sock"))
                    connection.sendall((json.dumps(frame, separators=(",", ":")) + "\n").encode())
                    with connection.makefile("rb") as reader:
                        return json.loads(reader.readline())

            resolution_request = {"type": "display_name_resolve", "request": {"version": 1, "display_name": "gui-coder"}}
            assert resolve_wire(resolution_request) == {"type": "display_name_resolution",
                "outcome": {"status": "found", "agent_id": original_id, "active": True}}
            # Mutation : supprimer le contrôle canonique accepterait ce champ
            # inconnu via les defaults Serde et ferait échouer cet oracle.
            rejected = resolve_wire({**resolution_request, "future": True})
            assert rejected["outcome"]["status"] == "rejected", rejected
            assert resolve_wire({"type": "display_name_resolve", "request": {"version": 2, "display_name": "gui-coder"}})["outcome"]["status"] == "rejected"

            def refused_launch(arguments, expected):
                # Vrai second terminal, mais refus exigé AVANT tout app-server.
                sockets_before = set(state.glob("c-*.sock"))
                with sqlite3.connect(state / "bridget.db") as database:
                    profiles_before = database.execute("SELECT agent_id, display_name FROM agent_profiles ORDER BY agent_id").fetchall()
                reject_master, reject_slave = os.openpty()
                child = subprocess.Popen([bridget, "codex", *arguments], env=environment, cwd=root,
                    stdin=reject_slave, stdout=reject_slave, stderr=subprocess.PIPE)
                try:
                    _, stderr = child.communicate(timeout=8)
                    assert child.returncode != 0 and expected in stderr.decode(), stderr.decode()
                    assert set(state.glob("c-*.sock")) == sockets_before, "second fournisseur démarré malgré refus"
                    with sqlite3.connect(state / "bridget.db") as database:
                        assert database.execute("SELECT agent_id, display_name FROM agent_profiles ORDER BY agent_id").fetchall() == profiles_before
                finally:
                    if child.poll() is None:
                        subprocess.run(["/bin/ps", "-p", str(child.pid), "-o", "pid=,comm="], check=False)
                        child.terminate()
                        child.wait(timeout=5)
                    os.close(reject_master)
                    os.close(reject_slave)

            refused_launch(["--name", "gui-coder", "--yolo", "resume", thread_id], "déjà actif")
            assert wrapper.poll() is None
            os.write(master, b"\x1b[200~HUMAN-RESUME-090\x1b[201~")
            until(lambda: b"HUMAN-RESUME-090" in transcript, "saisie historique")
            os.write(master, b"\r")
            until(lambda: b"OK-090" in transcript, "réponse historique")
            for restart_mode in ["name-and-thread", "thread-only", "name-new-thread"]:
                observer.socket.close()
                observer = None
                os.write(master, b"\x03\x03")
                until(lambda: wrapper.poll() is not None, "sortie avant reprise humaine")
                tick()
                assert terminal_restored() and not socket_path.exists()
                assert "Reprendre : bridget codex" in terminal_text() and "resume " + thread_id in terminal_text()
                wrapper = None
                # Le daemon ne conserve plus aucune présence en mémoire : la
                # résolution doit réellement lire le profil durable.
                subprocess.run(["/bin/ps", "-p", str(daemon.pid), "-o", "pid=,comm="], check=True)
                daemon.terminate()
                daemon.wait(timeout=5)
                daemon = subprocess.Popen([bridget, "daemon"], env=environment, cwd=root,
                    stdin=subprocess.DEVNULL, stdout=daemon_log, stderr=daemon_log)
                until(lambda: (state / "bridget.sock").exists(), "daemon repris")
                if restart_mode == "name-and-thread":
                    refused_launch(["--name", "autre-nom", "--agent-id", "90000000-0000-4000-8000-000000000099", "resume", thread_id], "identités différentes")
                os.close(master)
                os.close(slave)
                master, slave = os.openpty()
                fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 120, 0, 0))
                transcript.clear()
                query_counts.clear()
                arguments = [bridget, "codex", "--yolo", "--no-alt-screen"]
                if restart_mode != "thread-only":
                    arguments += ["--name", "gui-coder"]
                if restart_mode != "name-new-thread":
                    arguments += ["resume", thread_id]
                wrapper = subprocess.Popen([sys.executable, __file__, "--terminal-host", str(root), *arguments],
                    env=environment, cwd=root, stdin=slave, stdout=slave, stderr=slave, preexec_fn=own_terminal)
                until(lambda: b"Bridget :" in transcript and b"fixture" in transcript
                    and (b"context" in transcript or b"shortcuts" in transcript), "reprise réelle sans UUID Bridget : " + restart_mode)
                socket_path = next(state.glob("c-*.sock"))
                observer = probe.Client(str(socket_path))
                loaded = observer.rpc("thread/loaded/list", {})["result"]["data"]
                agent = next(a for a in json.loads(cli("agents", "--json")) if a["state"] == "connected")
                assert agent["agent_id"] == original_id and agent["display_name"] == "gui-coder", agent
                if restart_mode == "name-new-thread":
                    assert loaded != [thread_id]
                    thread_id = loaded[0]
                else:
                    assert loaded == [thread_id], loaded
                    history = observer.rpc("thread/read", {"threadId": thread_id, "includeTurns": True})["result"]
                    assert "HUMAN-RESUME-090" in json.dumps(history), "historique perdu"
                assert probe.Fixture.count == 1, "prompt rejoué implicitement"
            os.write(master, b"\x03\x03")
            until(lambda: wrapper.poll() is not None, "sortie finale")
            assert terminal_restored() and not socket_path.exists()
            print("human_resume_same_identity_and_history_active_refused_restart_durable", flush=True)
            return
        if copied_resume or paginated_resume:
            agent = next(a for a in json.loads(cli("agents", "--json"))
                if a.get("agent_id") == "90000000-0000-4000-8000-000000000001")
            assert agent["display_name"] == "coder-recette-090"
            assert probe.Fixture.count == 0, "tour fournisseur indésirable à la reprise"
            if paginated_resume:
                until(lambda: b"HISTORIQUE-A-CONSERVER-090" in transcript, "historique paginé rendu par la TUI native")
            assert not termios.tcgetattr(slave)[3] & termios.ICANON
            mark = len(transcript)
            os.write(master, b"\x1b[200~SAISIE-REPRISE-090\x1b[201~")
            until(lambda: b"SAISIE-REPRISE-090" in transcript[mark:], "saisie rendue par la TUI reprise, sans validation du prompt")
            os.write(master, b"\x03\x03")
            until(lambda: wrapper.poll() is not None, "sortie après reprise de copie")
            assert terminal_restored() and not socket_path.exists()
            assert probe.Fixture.count == 0
            print("paginated_or_copied_resume_real_tui_ready_without_prompt", flush=True)
            return
        prompt = "HUMAN-090 : pour ce premier tour seulement, reponds OK-090. Les prochains tours pourront demander des outils MCP." if live else "HUMAN-090"
        os.write(master, b"\x1b[200~" + prompt.encode() + b"\x1b[201~")
        until(lambda: b"HUMAN-090" in transcript, "texte humain rendu avant validation")
        os.write(master, b"\r")
        if not live:
            until(lambda: probe.Fixture.count >= 1, "tour humain transmis par TUI")
            if "--human-busy" in sys.argv:
                queued_issue = cli("send", "--to", "90000000-0000-4000-8000-000000000001", "--from", "90000000-0000-4000-8000-000000000002",
                    "--id", "queued090", "--issued-at", str(int(time.time())), "--issuer-scope", "fixture_090_queue_scope", "QUEUED-090")
                assert "in_flight" in queued_issue, queued_issue
                assert probe.Fixture.count == 1, "tour Bridget concurrent avec le tour humain bloqué"
                assert not any(b'"message_id":"queued090"' in p.read_bytes() for p in (state / "sessions").rglob("*.jsonl")), "ACK/journal de tour fabriqué avant consommation"
                probe.Fixture.first_release.set()
                until(lambda: probe.Fixture.count == 2, "FIFO libérée par le terminal humain réel")
                until(lambda: any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "queued090" for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "tour Bridget en attente terminé")
                print("human_busy_fifo_no_early_ack", flush=True)
        until(lambda: any(json.loads(line).get("event") == "turn_end" for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "terminal du tour humain attesté")
        until(lambda: b"OK-090" in transcript, "réponse visible dans la TUI")
        agents = json.loads(cli("agents", "--json"))
        agent = next(a for a in agents if a.get("agent_id") == "90000000-0000-4000-8000-000000000001")
        assert agent.get("mode") != "tmux", agent
        print("human_turn_native", probe.Fixture.count, "presence", agent, flush=True)
        if initial_resume:
            assert agent["display_name"] == "coder-recette-090", agent
            assert agent["agent_id"] == "90000000-0000-4000-8000-000000000001"
            history = observer.rpc("thread/read", {"threadId": thread_id, "includeTurns": True})["result"]["thread"]
            assert history["name"] == "HISTORIQUE-090", "titre historique écrasé par Bridget"
            assert "HISTORIQUE-A-CONSERVER-090" in json.dumps(history)
            assert "HUMAN-090" in json.dumps(history)
            contexts = [json.loads(line)["payload"] for path in home.rglob("rollout*.jsonl")
                if resume_id in path.name for line in path.read_bytes().splitlines()
                if json.loads(line).get("type") == "turn_context"]
            assert contexts, "aucun contexte fournisseur attesté"
            assert contexts[-1]["approval_policy"] == "never", contexts[-1]
            assert contexts[-1]["sandbox_policy"]["type"] == "danger-full-access", contexts[-1]
            issued_at = str(int(time.time()))
            send = ("send", "--to", agent["agent_id"], "--id", "after-resume-090",
                "--issued-at", issued_at, "--issuer-scope", "fixture_090_resume_scope", "APRES-REPRISE-090")
            cli(*send)
            until(lambda: any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "after-resume-090"
                for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "tour Bridget consommé après reprise")
            def accepted_after_resume():
                with sqlite3.connect(state / "bridget.db") as database:
                    return database.execute("SELECT public_result_kind FROM idempotency_records WHERE idempotency_key='after-resume-090'").fetchall() == [("accepted",)]
            until(accepted_after_resume, "ACK durable après reprise (distinct du journal de tour)")
            assert "accepted" in cli(*send).lower()
            history = observer.rpc("thread/read", {"threadId": thread_id, "includeTurns": True})["result"]["thread"]
            assert "APRES-REPRISE-090" in json.dumps(history), "message remis à un autre fil"
            assert probe.Fixture.count == 2, "retry injecté deux fois après reprise"
            print("initial_resume_preserves_history_name_and_uuid_yolo_attested", flush=True)
            os.write(master, b"\x03\x03")
            until(lambda: wrapper.poll() is not None, "sortie après reprise explicite")
            assert terminal_restored() and not socket_path.exists()
            return
        if "--reconnect" in sys.argv:
            count_before = probe.Fixture.count
            subprocess.run(["/bin/ps", "-p", str(daemon.pid), "-o", "pid=,comm="], check=True)
            daemon.terminate()
            daemon.wait(timeout=5)
            assert wrapper.poll() is None, "redémarrage daemon a arrêté la TUI"
            daemon = subprocess.Popen([bridget, "daemon"], env=environment, cwd=root,
                stdin=subprocess.DEVNULL, stdout=daemon_log, stderr=daemon_log)
            until(lambda: (state / "bridget.sock").exists(), "socket Bridget recréée")
            def reconnected():
                return any(a.get("agent_id") == agent["agent_id"] and a.get("state") == "connected" for a in json.loads(cli("agents", "--json")))
            until(reconnected, "même identité après redémarrage réel du daemon")
            assert observer.rpc("thread/loaded/list", {})["result"]["data"] == [thread_id]
            assert probe.Fixture.count == count_before, "prompt de reconstruction caché"
            print("daemon_restarted_identity_and_thread_unchanged", flush=True)
        if "--new-thread" in sys.argv or "--resume-thread" in sys.argv:
            secondary_body = None
            if "--new-thread" in sys.argv:
                started = observer.rpc("thread/start", {"cwd": str(root), "historyMode": "legacy"})
                assert "result" in started, started
                navigated_thread = started["result"]["thread"]["id"]
                secondary_body = "TOUR-DU-SECOND-FIL-090"
                secondary_turn = observer.rpc("turn/start", {"threadId": navigated_thread,
                    "input": [{"type": "text", "text": secondary_body}]})
                assert "result" in secondary_turn, secondary_turn
                secondary_deadline = time.monotonic() + 10
                while not any(event.get("method") == "turn/completed"
                    and event.get("params", {}).get("threadId") == navigated_thread
                    for event in observer.events):
                    observer.socket.settimeout(max(0.001, secondary_deadline - time.monotonic()))
                    observer.events.append(observer.receive())
                    assert time.monotonic() < secondary_deadline
            else:
                mark = len(transcript)
                navigation = ("/resume " + resume_id).encode()
                os.write(master, navigation)
                until(lambda: navigation.decode() in terminal_text(mark), "commande de navigation affichée")
                os.write(master, b"\r")
                navigated_thread = resume_id

            def navigation_loaded():
                loaded = observer.rpc("thread/loaded/list", {})["result"]["data"]
                return thread_id in loaded and len(loaded) >= 2

            until(navigation_loaded, "second fil réellement chargé par l'app-server")
            loaded = observer.rpc("thread/loaded/list", {})["result"]["data"]
            assert navigated_thread != thread_id and navigated_thread in loaded, loaded
            assert wrapper.poll() is None and socket_path.exists(), "notification globale devenue destructive"
            connected = [entry for entry in json.loads(cli("agents", "--json"))
                if entry.get("agent_id") == agent["agent_id"] and entry.get("state") == "connected"]
            assert len(connected) == 1 and connected[0]["connection_id"] == agent["connection_id"], connected

            message_id = "after-foreign-thread-090"
            message_body = "BRIDGET-RESTE-SUR-LE-PARENT-090"
            issued_at = str(int(time.time()))
            cli("send", "--to", agent["agent_id"], "--id", message_id,
                "--issued-at", issued_at, "--issuer-scope", "fixture_090_foreign_thread", message_body)
            until(lambda: any(json.loads(line).get("event") == "turn_end"
                and json.loads(line).get("message_id") == message_id
                for path in (state / "sessions").rglob("*.jsonl")
                for line in path.read_bytes().splitlines()), "message Bridget terminé après chargement du fil tiers")
            parent_read = observer.rpc("thread/read", {"threadId": thread_id, "includeTurns": True})
            assert "result" in parent_read, parent_read
            navigated_read = observer.rpc("thread/read", {"threadId": navigated_thread, "includeTurns": True})
            assert "result" in navigated_read, navigated_read
            parent_history = parent_read["result"]["thread"]
            navigated_history = navigated_read["result"]["thread"]
            assert message_body in json.dumps(parent_history), "remise Bridget détournée du fil principal"
            assert message_body not in json.dumps(navigated_history), "remise Bridget attribuée au fil navigué"
            if secondary_body:
                assert secondary_body in json.dumps(navigated_history), "tour secondaire non matérialisé"
                assert secondary_body not in json.dumps(parent_history), "tour secondaire attribué au parent"
                journal = b"".join(path.read_bytes() for path in (state / "sessions").rglob("*.jsonl"))
                assert secondary_body.encode() not in journal, "tour secondaire journalisé par Bridget sur le parent"
            assert wrapper.poll() is None, "fin du tour parent a fermé la TUI"
            print("foreign_thread_non_destructive_parent_thread_still_targeted",
                thread_id, navigated_thread, flush=True)
            mark = len(transcript)
            os.write(master, b"/quit")
            until(lambda: b"/quit" in transcript[mark:], "commande de sortie native rendue")
            os.write(master, b"\r")
            until(lambda: wrapper.poll() is not None, "sortie explicite après chargement du fil tiers")
            assert not socket_path.exists() and terminal_restored()
            return
        sender_id = "90000000-0000-4000-8000-000000000002"
        peer = socket.socket(socket.AF_UNIX)
        peer.connect(str(state / "bridget.sock"))
        peer.sendall((json.dumps({"type": "Register", "agent_type": "fixture", "identity_version": 2,
            "agent_id": sender_id, "instance_id": "90000000-0000-4000-8000-000000000003"}) + "\n").encode())
        peer_file = peer.makefile("rb")
        assert json.loads(peer_file.readline())["type"] == "Registered"

        def receive_replies():
            try:
                for line in peer_file:
                    frame = json.loads(line)
                    receipts.append(frame)
                    if frame["type"] == "DeliverIdempotent":
                        peer.sendall((json.dumps({"type": "DeliverAcked", "delivery_id": frame["delivery_id"],
                            "delivery_generation": frame["delivery_generation"]}) + "\n").encode())
            except (OSError, ValueError):
                pass
        peer_reader = threading.Thread(target=receive_replies)
        peer_reader.start()
        issued = str(int(time.time()))
        attach = subprocess.Popen([bridget, "attach", agent["agent_id"]], env=environment, cwd=root,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        # Les tours humains n'ont pas d'id de message Bridget : le renderer
        # existant affiche leurs deltas séparément (le vrai fournisseur peut
        # couper « OK », « - », « 090 »). Ne pas imposer le framing du mock.
        until(lambda: b"090" in attached, "attach réel rejoue les deltas du tour humain")
        attached_before = len(attached)
        provider_before_reply = probe.Fixture.count
        probe.Fixture.reply_arguments = {"to": sender_id, "body": "REPLY-MCP-090", "in_reply_to": "request-090",
            "id": "reply-mcp-090", "issued_at": int(issued)}
        send_args = ("send", "--to", "90000000-0000-4000-8000-000000000001", "--from", sender_id,
            "--reply", "--timeout", "60", "--id", "request-090", "--issued-at", issued,
            "--issuer-scope", "fixture_090_sender_scope", ("Appelle uniquement l'outil bridget_send avec EXACTEMENT ces arguments JSON : "
                + json.dumps(probe.Fixture.reply_arguments) + ". Puis affiche OK-090." if live else "INTERAGENT-090"))
        print("send_issue", cli(*send_args), flush=True)
        if not live:
            until(lambda: probe.Fixture.count >= provider_before_reply + 1, "tour Bridget reçu par le même fournisseur")
        def permission_visible():
            if live:
                ended = any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "request-090"
                    for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines())
                assert not ended, "le fournisseur a terminé sans l'appel MCP demandé ; échec de recette, ne pas attendre ses rappels"
            return "Allow the bridget MCP server" in terminal_text() and "enter to submit" in terminal_text()
        until(permission_visible,
            "permission native affichée")
        print("permission_displayed", "wrapper", wrapper.pid, flush=True)
        attributes = termios.tcgetattr(slave)
        print("permission_terminal_flags", attributes[:4], "canonical", bool(attributes[3] & termios.ICANON), flush=True)
        assert not any(f.get("message", {}).get("body") == "REPLY-MCP-090" for f in receipts), "outil exécuté avant décision humaine"
        assert live or probe.Fixture.count == provider_before_reply + 1, "tour poursuivi sans permission humaine"
        if any(mode in sys.argv for mode in ["--abort-permission", "--hangup-permission", "--provider-eof"]):
            # Arrêt réel PENDANT la requête pendante : le serveur et la TUI
            # doivent tous deux disparaître, pas seulement leur socket.
            owned_pid = (root / "wrapper.pid").read_text()
            children = subprocess.check_output(["/usr/bin/pgrep", "-P", owned_pid]).decode().split()
            started = time.monotonic()
            if "--provider-eof" in sys.argv:
                server_pids = [pid for pid in children if "app-server" in subprocess.check_output(["/bin/ps", "-p", pid, "-o", "command="]).decode()]
                assert len(server_pids) == 1, server_pids
                subprocess.run(["/bin/ps", "-p", server_pids[0], "-o", "pid=,comm="], check=True)
                # TERM/INT drainent une élicitation : ils ne produisent PAS
                # l'EOF testé. Crash réel du seul enfant identifié, sans core
                # dump ni SIGKILL ; le wrapper doit survivre pour nettoyer.
                os.kill(int(server_pids[0]), signal.SIGABRT)
            else:
                signal_wrapper(signal.SIGHUP if "--hangup-permission" in sys.argv else signal.SIGTERM)
            while wrapper.poll() is None:
                if time.monotonic() - started >= 9:
                    subprocess.run(["/usr/bin/sample", str(wrapper.pid), "2", "-file", str(root / "blocked.sample")], timeout=5)
                    raise AssertionError("sortie au signal dépasse 9 s")
                # Un vrai terminal continue de lire les écritures de sortie.
                # wait() seul bloquerait le fournisseur sur son PTY plein.
                tick()
            for child_pid in children:
                assert subprocess.run(["/bin/ps", "-p", child_pid, "-o", "pid="], capture_output=True).returncode != 0, f"enfant survivant {child_pid}"
            assert not socket_path.exists()
            assert terminal_restored(), ("restore", wrapper.returncode)
            print("pending_permission_abort_seconds", time.monotonic() - started, flush=True)
            return
        if "--deny-permission" in sys.argv:
            os.write(master, b"\x1b")
            until(lambda: probe.Fixture.count >= provider_before_reply + 2, "refus humain consommé par Codex")
            until(lambda: any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "request-090"
                for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "tour refusé terminé")
            assert not any(f.get("message", {}).get("body") == "REPLY-MCP-090" for f in receipts)
            with sqlite3.connect(state / "bridget.db") as database:
                assert database.execute("SELECT state FROM tracked_requests WHERE id='request-090'").fetchall() != [("answered",)]
            before = probe.Fixture.count
            cli("send", "--to", agent["agent_id"], "--id", "after-denial-090", "--issued-at", issued,
                "--issuer-scope", "fixture_090_after_denial", "AFTER-DENIAL-090")
            until(lambda: probe.Fixture.count > before, "session encore joignable après refus humain")
            until(lambda: any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "after-denial-090"
                for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "nouveau tour terminé après refus")
            mark = len(transcript)
            os.write(master, b"/quit")
            until(lambda: b"/quit" in transcript[mark:], "sortie explicitement saisie")
            os.write(master, b"\r")
            until(lambda: wrapper.poll() is not None, "sortie après refus")
            assert terminal_restored() and not socket_path.exists()
            print("permission_denied_no_reply_session_still_usable", flush=True)
            return
        # Le harnais joue l'humain sur la VRAIE TUI ; aucun client RPC ne
        # fabrique son accord. Seul Allow ponctuel, jamais Always allow.
        # Une navigation puis son rendu prouvent que la modale traite déjà
        # les touches ; l'apparition d'un début de peinture ne le prouve pas.
        mark = len(transcript)
        os.write(master, b"\x1b[B")
        until(lambda: "› 2. Allow for this session" in terminal_text(mark), "choix natif déplacé, sans validation")
        mark = len(transcript)
        os.write(master, b"\x1b[A")
        until(lambda: "› 1. Allow" in terminal_text(mark), "autorisation ponctuelle sélectionnée par l'humain")
        print("permission_native_ready_for_human", flush=True)
        os.write(master, b"\r")
        until(lambda: any(f.get("message", {}).get("body") == "REPLY-MCP-090" for f in receipts), "réponse du vrai outil MCP")
        if not live:
            until(lambda: probe.Fixture.count >= provider_before_reply + 2, "résultat de l'outil consommé par Codex")
        def database_rows(query):
            with sqlite3.connect(state / "bridget.db") as database:
                return database.execute(query).fetchall()

        until(lambda: database_rows("SELECT state FROM tracked_requests WHERE id='request-090'") == [("answered",)], "demande durablement answered")
        until(lambda: database_rows("SELECT public_result_kind FROM idempotency_records WHERE idempotency_key='request-090'") == [("accepted",)], "ACK durable du tour interagent")
        until(lambda: any(json.loads(line).get("event") == "turn_end" and json.loads(line).get("message_id") == "request-090" for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()), "terminal interagent observé")
        print("tracked_request_answered", flush=True)
        retry = cli(*send_args)
        assert "accepted" in retry.lower(), retry
        print("retry_issue", retry, flush=True)
        until(lambda: any(b"OK-090" in p.read_bytes() and b"human" in p.read_bytes() for p in (state / "sessions").rglob("*.jsonl")), "journal des deux origines")
        assert observer.rpc("thread/loaded/list", {})["result"]["data"] == [thread_id]
        print("interagent_same_thread", thread_id, "requests", probe.Fixture.count, flush=True)
        assert len([f for f in receipts if f.get("message", {}).get("body") == "REPLY-MCP-090"]) == 1, receipts
        assert database_rows("SELECT id, body FROM ledger WHERE sender='90000000-0000-4000-8000-000000000001'") == [("reply-mcp-090", "REPLY-MCP-090")], "réponse automatique ajoutée après la réponse MCP"
        assert live or probe.Fixture.count == provider_before_reply + 2, "retry injecté comme un nouveau tour"
        print("mcp_reply_unique", flush=True)
        # L'oracle suit les deltas réellement journalisés, pas la segmentation
        # HTTP du faux fournisseur. Exiger chaque fragment après l'abonnement
        # et le terminal prouve aussi le live du fournisseur sous abonnement.
        text_deltas = [json.loads(line)["payload"]["content"]
            for p in (state / "sessions").rglob("*.jsonl") for line in p.read_bytes().splitlines()
            if json.loads(line).get("message_id") == "request-090"
            and json.loads(line).get("event") == "update"
            and json.loads(line).get("payload", {}).get("kind") == "text"]
        # Un vrai modèle peut commenter avant l'appel MCP : ce texte doit aussi
        # être visible dans attach. Seul le fournisseur synthétique est figé.
        rendered_answer = "".join(text_deltas).strip()
        assert (rendered_answer.endswith("OK-090") if live else rendered_answer == "OK-090"), text_deltas
        until(lambda: all(delta.encode() in attached[attached_before:] for delta in text_deltas)
            and b"[fin]" in attached[attached_before:], "attach réel suit les deltas et le terminal interagent")
        print("attach_replay_then_live", flush=True)
        os.write(master, b"\x03\x03")
        until(lambda: wrapper.poll() is not None, "sortie native")
        assert not socket_path.exists(), "socket privée orpheline"
        assert terminal_restored(), "termios non restauré"
        print("terminal_restored_and_socket_removed", flush=True)
    finally:
        if probe.Fixture.first_release is not None:
            probe.Fixture.first_release.set()
        (home / "auth.json").unlink(missing_ok=True)
        (root / "provider-requests.json").write_text(json.dumps(probe.Fixture.requests, indent=2))
        (root / "terminal.log").write_bytes(transcript)
        (root / "attach.log").write_bytes(attached)
        if observer:
            observer.socket.close()
        if peer:
            peer.shutdown(socket.SHUT_RDWR)
            peer.close()
        if peer_reader:
            peer_reader.join(timeout=3)
        cleanup_errors = []
        for process in [attach, wrapper, daemon]:
            if process and process.poll() is None:
                subprocess.run(["/bin/ps", "-p", str(process.pid), "-o", "pid=,comm="], check=False)
                if process is wrapper and (root / "wrapper.pid").exists():
                    try:
                        signal_wrapper(signal.SIGTERM)
                    except (ProcessLookupError, subprocess.CalledProcessError):
                        pass
                else:
                    process.terminate()
                try:
                    if process is wrapper:
                        cleanup_deadline = time.monotonic() + 12
                        while process.poll() is None and time.monotonic() < cleanup_deadline:
                            tick(enforce_deadline=False)
                        if process.poll() is None:
                            raise subprocess.TimeoutExpired(process.args, 12)
                    else:
                        process.wait(timeout=12)
                except subprocess.TimeoutExpired:
                    cleanup_errors.append(process.pid)
        os.close(master)
        os.close(slave)
        daemon_log.close()
        provider.shutdown()
        print("fixture_root", root, flush=True)
        assert not cleanup_errors, f"nettoyage incomplet : {cleanup_errors}"


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--terminal-host":
        sys.exit(terminal_host())
    main()
