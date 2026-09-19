#!/usr/bin/env python3
"""Couture vrai daemon/wrapper/Codex, HTTP déterministe privé, zéro abonnement.

Le HTTP enregistre le modèle et l'effort EFFECTIVEMENT demandés. Le fil est
relu dans les événements bruts, pas reconstruit par le test. Tous les processus
appartiennent au harnais et la racine HOME/socket est privée.
"""
import http.server
import importlib.util
import json
import os
import pathlib
import queue
import socket
import subprocess
import sys
import tempfile
import threading
import time

REPO = pathlib.Path(__file__).resolve().parents[4]
spec = importlib.util.spec_from_file_location("shared_probe", REPO / "fixtures/probe_shared_session.py")
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class Peer:
    def __init__(self, path):
        self.socket = socket.socket(socket.AF_UNIX)
        self.socket.settimeout(6)
        self.socket.connect(str(path))
        self.reader = self.socket.makefile("rb")

    def send(self, value):
        self.socket.sendall(json.dumps(value).encode() + b"\n")

    def until(self, kind):
        deadline = time.monotonic() + 6
        while time.monotonic() < deadline:
            self.socket.settimeout(max(.001, deadline - time.monotonic()))
            line = self.reader.readline(4_000_000)
            assert line, "socket fermée"
            value = json.loads(line)
            if kind == "SnapshotCaughtUp" and value.get("reason") == "journal_unavailable":
                return value
            if value["type"] in ["AttachRejected", "Nack"]:
                raise AssertionError(value)
            if value["type"] == kind:
                return value
        raise TimeoutError(kind)

    def close(self):
        self.reader.close()
        self.socket.close()


def main():
    bridget, codex = sys.argv[1:3]
    root = pathlib.Path(tempfile.mkdtemp(prefix="b91-native-", dir="/tmp"))
    state, home = root / "s", root / "h"
    state.mkdir(mode=0o700)
    home.mkdir(mode=0o700)
    env = {"HOME": str(home), "CODEX_HOME": str(home), "BRIDGET_HOME": str(state),
           "BRIDGET_SOCKET": str(state / "bridget.sock"), "PATH": "/opt/homebrew/bin:/usr/bin:/bin",
           "TERM": "xterm-256color", "LANG": "en_US.UTF-8"}
    provider = http.server.ThreadingHTTPServer(("127.0.0.1", 0), probe.Fixture)
    probe.Fixture.first_release = threading.Event()
    threading.Thread(target=provider.serve_forever, daemon=True).start()
    (home / "config.toml").write_text('model_provider="fixture"\ncheck_for_update_on_startup=false\n'
        'approval_policy="never"\nsandbox_mode="read-only"\n'
        '[model_providers.fixture]\nname="fixture"\nwire_api="responses"\nrequires_openai_auth=false\n'
        f'base_url="http://127.0.0.1:{provider.server_port}/v1"\n')
    os.chmod(home / "config.toml", 0o600)
    children, peers = [], []
    try:
        # Catalogue du binaire réellement testé, pas liste codée dans l'oracle.
        server = subprocess.Popen([codex, "app-server"], env=env, cwd=root, stdin=subprocess.PIPE,
                                  stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        children.append(server)
        incoming = queue.Queue()
        def read_catalogue():
            for line in server.stdout:
                incoming.put(json.loads(line))
            incoming.put(None)
        threading.Thread(target=read_catalogue, daemon=True).start()
        for message in [{"id":1,"method":"initialize","params":{"clientInfo":{"name":"test091","version":"1"}}},
                        {"method":"initialized"}, {"id":2,"method":"model/list","params":{"limit":100}}]:
            server.stdin.write(json.dumps(message) + "\n")
            server.stdin.flush()
        end = time.monotonic() + 10
        while True:
            answer = incoming.get(timeout=max(.001, end - time.monotonic()))
            assert answer is not None
            if answer.get("id") == 2:
                models = [m for m in answer["result"]["data"] if m["supportedReasoningEfforts"]]
                break
        assert len(models) >= 2
        choices = [{"model": m["model"], "effort":m["supportedReasoningEfforts"][0]["reasoningEffort"]}
                   for m in models[:2]]
        server.terminate()
        server.wait(timeout=5)
        definition = {"command":codex, "protocol":"codex_app_server",
                      "args":["app-server", "-c", 'model="' + choices[0]["model"] + '"'],
                      "permissions":"deny", "notify_timeout_secs":20,
                      "mcp":{"interactive":"none","acp_session":False}}
        (state / "agents.json").write_text(json.dumps({"agents":{"codex":definition}}))
        os.chmod(state / "agents.json", 0o600)
        log = open(root / "daemon.log", "wb")
        daemon = subprocess.Popen([bridget, "daemon"], env=env, cwd=root, stdin=subprocess.DEVNULL, stdout=log, stderr=log)
        children.append(daemon)
        deadline = time.monotonic() + 10
        while not (state / "bridget.sock").exists():
            assert daemon.poll() is None and time.monotonic() < deadline
            time.sleep(.01)
        agent = "91000000-0000-4000-8000-000000000091"
        log_wrapper = open(root / "wrapper.log", "wb")
        wrapper = subprocess.Popen([bridget, "codex", "--equipier", "--agent-id", agent],
                                   env=env, cwd=root, stdin=subprocess.DEVNULL, stdout=log_wrapper, stderr=log_wrapper)
        children.append(wrapper)
        observer = Peer(state / "bridget.sock")
        peers.append(observer)
        observer.send({"type":"RoleHandshake","role":"attach"})
        observer.until("RoleAccepted")
        deadline = time.monotonic() + 15
        while True:
            assert wrapper.poll() is None, (root / "wrapper.log").read_text()
            observer.send({"type":"ListAgents"})
            agents = observer.until("AgentList")["agents"]
            if any(a.get("agent_id") == agent for a in agents):
                break
            assert time.monotonic() < deadline, agents
            time.sleep(.02)
        deadline = time.monotonic() + 10
        while True:
            observer.send({"type":"Subscribe","agent":agent,"window":{"kind":"Tail","value":20}})
            ready = observer.until("SnapshotCaughtUp")
            if ready["type"] == "SnapshotCaughtUp":
                break
            assert time.monotonic() < deadline, ready
            time.sleep(.02)
        # Aucun appel HTTP au modèle lors de la sélection ou d'un refus.
        for index, choice in enumerate(choices):
            if index == 0:
                observer.send({"type":"SelectRuntime","agent":agent,"selection":choice})
                selected = observer.until("RuntimeSelectionResult")["outcome"]
                assert selected == {"status":"selected","selection":choice}, selected
            assert len(probe.Fixture.requests) == index, "prompt caché pendant la sélection"
            for invalid, reason in [({"model":"absent091","effort":"low"}, "model_unavailable"),
                                    ({"model":choice["model"],"effort":"absent091"}, "effort_unavailable")]:
                observer.send({"type":"SelectRuntime","agent":agent,"selection":invalid})
                assert observer.until("RuntimeSelectionResult")["outcome"] == {"status":"refused","reason":reason}
            message_id = "tour091-" + str(index)
            observer.send({"type":"Send","id":message_id,"from":"humain","to":agent,
                           "body":"SENTINELLE-HISTORIQUE-091" if index == 0 else "Lis le tour précédent.",
                           "reply":False})
            observer.until("Ack")
            deadline = time.monotonic() + 15
            while len(probe.Fixture.requests) <= index:
                assert time.monotonic() < deadline
                time.sleep(.01)
            actual = probe.Fixture.requests[index]
            assert actual["model"] == choice["model"], (actual["model"], choice)
            assert actual["reasoning"]["effort"] == choice["effort"], actual.get("reasoning")
            if index == 0:
                # Le HTTP du tour 1 est encore bloqué à une vraie barrière.
                # Changer maintenant doit préserver ce tour et ne lancer que
                # le prochain avec les nouveaux réglages, sans redémarrage.
                observer.send({"type":"SelectRuntime","agent":agent,"selection":choices[1]})
                assert observer.until("RuntimeSelectionResult")["outcome"] == {
                    "status":"selected","selection":choices[1]}
                assert len(probe.Fixture.requests) == 1
                probe.Fixture.first_release.set()
            if index:
                assert "SENTINELLE-HISTORIQUE-091" in json.dumps(actual["input"]), "historique perdu"
            # L'annuaire est une observation du flux, pas le reçu de sélection.
            deadline = time.monotonic() + 10
            while True:
                observer.send({"type":"ListAgents"})
                info = next(a for a in observer.until("AgentList")["agents"] if a["agent_id"] == agent)
                if (index == 0 or (info.get("model") == choice["model"] and info.get("effort") == choice["effort"])) and info["state"] == "connected":
                    break
                assert time.monotonic() < deadline, info
                time.sleep(.02)
        rolls = list(home.rglob("rollout-*.jsonl"))
        assert len(rolls) == 1, "le changement ne doit pas créer un deuxième fil"
        print(json.dumps({"result":"PASS","choices":choices,"same_thread":True,"history_preserved":True,
                          "changed_during_turn":True,"http_turns":len(probe.Fixture.requests),"root":str(root)}), flush=True)
    finally:
        probe.Fixture.first_release.set()
        for peer in peers:
            peer.close()
        # Le daemon ferme les wrappers --equipier non persistants par Disconnect.
        if 'daemon' in locals() and daemon.poll() is None:
            subprocess.run(["/bin/ps","-p",str(daemon.pid),"-o","pid=,comm="], check=True)
            daemon.terminate()
            daemon.wait(timeout=10)
        if 'wrapper' in locals():
            wrapper.wait(timeout=10)
        for child in reversed(children):
            if child.poll() is None:
                subprocess.run(["/bin/ps","-p",str(child.pid),"-o","pid=,comm="], check=True)
                child.terminate()
                child.wait(timeout=10)
        provider.shutdown()
        provider.server_close()


if __name__ == "__main__":
    main()
