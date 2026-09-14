#!/usr/bin/env python3
"""Faux t3code pour la session 098 : serveur HTTP local et faux CLI `t3`.

Le serveur imite le contrat relevé sur t3code 0.0.40 (sonde T001) : fichier
`userdata/server-runtime.json`, snapshot, détail paginé, dispatch dédupliqué
par `commandId`, un tour à la fois par fil (les suivants sont mis en file), fil
archivé qui reste dans le snapshot. Des routes `/__test/*` sans jeton pilotent
les scénarios (message humain, archivage, 401 forcés, lenteur).

`t3code_server_098.py auth session ...` joue le CLI officiel : émission,
liste et révocation de sessions, toutes consignées dans `fake-t3.log`.
"""
import json
import os
import pathlib
import secrets
import signal
import sys
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

HOME = pathlib.Path(os.environ["T3CODE_HOME"])
SESSIONS = HOME / "fake-sessions.json"
CLI_LOG = HOME / "fake-t3.log"


def now():
    return time.strftime("%Y-%m-%dT%H:%M:%S.000Z", time.gmtime())


# ---------------------------------------------------------------------------
# Faux CLI `t3 auth session ...`
# ---------------------------------------------------------------------------


def load_sessions():
    if SESSIONS.exists():
        return json.loads(SESSIONS.read_text())
    return []


def save_sessions(sessions):
    SESSIONS.write_text(json.dumps(sessions))


def fake_cli(argv):
    with CLI_LOG.open("a") as log:
        log.write(" ".join(argv) + "\n")
    if argv[:3] == ["auth", "session", "issue"]:
        options = dict(zip(argv[3::2], argv[4::2]))
        session = {
            "sessionId": "ses_" + secrets.token_hex(6),
            "token": "tok_" + secrets.token_hex(12),
            "expiresAt": "2099-01-01T00:00:00.000Z",
            "client": {"label": options.get("--label", ""), "subject": options.get("--subject", "")},
        }
        sessions = load_sessions()
        sessions.append(session)
        save_sessions(sessions)
        if "--json" in argv:
            print(json.dumps(session))
        return 0
    if argv[:3] == ["auth", "session", "list"]:
        print(json.dumps(load_sessions()))
        return 0
    if argv[:3] == ["auth", "session", "revoke"]:
        wanted = argv[3]
        sessions = [s for s in load_sessions() if s["sessionId"] != wanted]
        save_sessions(sessions)
        return 0
    sys.stderr.write("faux t3 : commande inconnue\n")
    return 2


# ---------------------------------------------------------------------------
# Faux serveur
# ---------------------------------------------------------------------------


class State:
    def __init__(self, log_dir):
        self.lock = threading.RLock()
        self.sequence = 10
        self.turn_delay = 0.4
        self.unauthorized_left = 0
        self.dispatches = []
        self.commands = {}
        self.log = pathlib.Path(log_dir) / "dispatch.jsonl"
        self.projects = [{"id": "proj-1", "workspaceRoot": "/tmp/projet-alpha"}]
        self.threads = {}
        self.add_thread("thread-alpha", "Alpha", "claudeAgent")
        alpha = self.threads["thread-alpha"]
        alpha["messages"] = [
            self.message("m-hist-u", "user", "historique humain", None),
            self.message("m-hist-a", "assistant", "réponse historique", "turn-0"),
        ]
        alpha["latestTurn"] = {"turnId": "turn-0", "state": "completed", "assistantMessageId": "m-hist-a"}

    @staticmethod
    def message(message_id, role, text, turn_id, streaming=False):
        return {
            "id": message_id,
            "role": role,
            "text": text,
            "streaming": streaming,
            "turnId": turn_id,
            "attachments": [],
            "createdAt": now(),
            "updatedAt": now(),
        }

    def add_thread(self, thread_id, title, provider, with_session=True):
        """`with_session=False` reproduit un fil créé et jamais démarré : la
        session fournisseur n'existe qu'au premier tour (t3code 0.0.41)."""
        self.threads[thread_id] = {
            "id": thread_id,
            "projectId": "proj-1",
            "title": title,
            "modelSelection": {"instanceId": provider, "model": "modele-de-test"},
            "runtimeMode": "approval-required",
            "interactionMode": "default",
            "worktreePath": "/tmp/projet-alpha/.worktrees/" + thread_id,
            "archivedAt": None,
            "deletedAt": None,
            "settledOverride": None,
            "updatedAt": now(),
            "latestTurn": None,
            "session": {
                "providerName": provider,
                "providerInstanceId": provider,
                "status": "ready",
                "activeTurnId": None,
            }
            if with_session
            else None,
            "messages": [],
            "queue": [],
            "counter": 0,
        }

    def next_sequence(self):
        self.sequence += 1
        return self.sequence

    def snapshot(self):
        threads = []
        for thread in self.threads.values():
            public = {k: v for k, v in thread.items() if k not in ("messages", "queue", "counter")}
            threads.append(public)
        return {"snapshotSequence": self.sequence, "projects": self.projects, "threads": threads}

    def detail(self, thread_id, turn_limit):
        thread = self.threads[thread_id]
        messages = thread["messages"]
        turns = []
        for message in messages:
            if message["turnId"] and message["turnId"] not in turns:
                turns.append(message["turnId"])
        kept_turns = set(turns[-turn_limit:]) if turn_limit else set(turns)
        # Pagination par tours : on coupe avant le premier message du plus ancien tour gardé.
        start = 0
        if kept_turns and len(turns) > turn_limit:
            oldest = turns[-turn_limit]
            for index, message in enumerate(messages):
                if message["turnId"] == oldest:
                    start = index
                    break
            # Les messages utilisateur immédiatement avant ce tour lui appartiennent.
            while start > 0 and messages[start - 1]["turnId"] is None:
                start -= 1
        page = messages[start:]
        return {
            "snapshotSequence": self.sequence,
            "thread": {"messages": page, "latestTurn": thread["latestTurn"]},
            "page": {"hasMore": start > 0, "turnLimit": turn_limit},
        }

    def user_message(self, thread_id, message_id, text, source):
        """Un message utilisateur démarre un tour, ou attend son tour (FIFO)."""
        thread = self.threads[thread_id]
        thread["messages"].append(self.message(message_id, "user", text, None))
        thread["updatedAt"] = now()
        thread["queue"].append((message_id, text, source))
        self.pump(thread_id)

    def pump(self, thread_id):
        thread = self.threads[thread_id]
        if (thread["session"] or {}).get("activeTurnId") or not thread["queue"]:
            return
        message_id, text, source = thread["queue"].pop(0)
        if thread["session"] is None:
            thread["session"] = {
                "providerName": thread["modelSelection"]["instanceId"],
                "providerInstanceId": thread["modelSelection"]["instanceId"],
                "status": "ready",
                "activeTurnId": None,
            }
        thread["counter"] += 1
        turn_id = f"turn-{thread_id}-{thread['counter']}"
        thread["session"]["activeTurnId"] = turn_id
        thread["session"]["status"] = "running"
        thread["latestTurn"] = {"turnId": turn_id, "state": "running", "assistantMessageId": None}
        thread["updatedAt"] = now()
        delay = self.turn_delay

        def finish():
            time.sleep(delay)
            with self.lock:
                assistant_id = f"a-{turn_id}"
                thread["messages"].append(
                    self.message(assistant_id, "assistant", f"echo[{source}]: {text}", turn_id)
                )
                thread["latestTurn"] = {
                    "turnId": turn_id,
                    "state": "completed",
                    "assistantMessageId": assistant_id,
                }
                thread["session"]["activeTurnId"] = None
                thread["session"]["status"] = "ready"
                thread["updatedAt"] = now()
                self.next_sequence()
                self.pump(thread_id)

        threading.Thread(target=finish, daemon=True).start()

    def dispatch(self, command):
        command_id = command["commandId"]
        if command_id in self.commands:
            return self.commands[command_id]
        kind = command["type"]
        thread_id = command["threadId"]
        if kind == "thread.turn.start":
            # Le schéma client de t3code exige ces deux champs (pas de valeur
            # par défaut sur la route HTTP) : les omettre est un 400.
            if "runtimeMode" not in command or "interactionMode" not in command:
                raise ValueError("runtimeMode et interactionMode requis")
            message = command["message"]
            self.dispatches.append(
                {
                    "commandId": command_id,
                    "messageId": message["messageId"],
                    "text": message["text"],
                    "runtimeMode": command["runtimeMode"],
                    "interactionMode": command["interactionMode"],
                }
            )
            with self.log.open("a") as log:
                log.write(json.dumps(self.dispatches[-1]) + "\n")
            self.user_message(thread_id, message["messageId"], message["text"], "bridget")
        elif kind == "thread.archive":
            self.threads[thread_id]["archivedAt"] = now()
        else:
            raise ValueError(kind)
        result = {"sequence": self.next_sequence()}
        self.commands[command_id] = result
        return result


class Handler(BaseHTTPRequestHandler):
    state = None
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        pass

    def reply(self, code, payload):
        body = json.dumps(payload).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def body(self):
        length = int(self.headers.get("Content-Length") or 0)
        return json.loads(self.rfile.read(length) or b"{}")

    def authorized(self):
        with self.state.lock:
            if self.state.unauthorized_left > 0:
                self.state.unauthorized_left -= 1
                return False
        header = self.headers.get("Authorization", "")
        token = header[7:] if header.startswith("Bearer ") else ""
        return any(s["token"] == token for s in load_sessions())

    def do_GET(self):
        url = urlparse(self.path)
        if url.path == "/__test/dispatches":
            with self.state.lock:
                return self.reply(200, self.state.dispatches)
        if not self.authorized():
            return self.reply(401, {"error": "unauthorized"})
        with self.state.lock:
            if url.path == "/api/orchestration/snapshot":
                return self.reply(200, self.state.snapshot())
            if url.path.startswith("/api/orchestration/threads/"):
                thread_id = url.path.rsplit("/", 1)[1]
                if thread_id not in self.state.threads:
                    return self.reply(404, {"error": "unknown thread"})
                limit = int(parse_qs(url.query).get("turnLimit", ["20"])[0])
                return self.reply(200, self.state.detail(thread_id, limit))
        return self.reply(404, {"error": "no route"})

    def do_POST(self):
        url = urlparse(self.path)
        payload = self.body()
        with self.state.lock:
            if url.path == "/__test/human":
                self.state.user_message(
                    payload["threadId"], "h-" + secrets.token_hex(4), payload["text"], "human"
                )
                return self.reply(200, {"ok": True})
            if url.path == "/__test/settle":
                self.state.threads[payload["threadId"]]["settledOverride"] = "settled"
                self.state.next_sequence()
                return self.reply(200, {"ok": True})
            if url.path == "/__test/archive":
                self.state.threads[payload["threadId"]]["archivedAt"] = now()
                self.state.next_sequence()
                return self.reply(200, {"ok": True})
            if url.path == "/__test/thread":
                self.state.add_thread(
                    payload["id"],
                    payload["title"],
                    payload["provider"],
                    payload.get("withSession", True),
                )
                self.state.next_sequence()
                return self.reply(200, {"ok": True})
            if url.path == "/__test/title":
                thread = self.state.threads[payload["threadId"]]
                thread["title"] = payload["title"]
                thread["updatedAt"] = now()
                self.state.next_sequence()
                return self.reply(200, {"ok": True})
            if url.path == "/__test/policy":
                thread = self.state.threads[payload["threadId"]]
                thread["runtimeMode"] = payload["runtimeMode"]
                thread["interactionMode"] = payload["interactionMode"]
                thread["updatedAt"] = now()
                self.state.next_sequence()
                return self.reply(200, {"ok": True})
            if url.path == "/__test/unauthorized":
                self.state.unauthorized_left = int(payload["count"])
                return self.reply(200, {"ok": True})
            if url.path == "/__test/slow":
                self.state.turn_delay = float(payload["seconds"])
                return self.reply(200, {"ok": True})
        if not self.authorized():
            return self.reply(401, {"error": "unauthorized"})
        if url.path == "/api/orchestration/dispatch":
            with self.state.lock:
                try:
                    return self.reply(200, self.state.dispatch(payload))
                except (KeyError, ValueError) as error:
                    return self.reply(400, {"error": repr(error)})
        return self.reply(404, {"error": "no route"})


def serve(log_dir):
    # Devenir chef de groupe : le harnais nettoie ses enfants par `kill(-pid)`,
    # qui n'atteint un processus que s'il dirige son propre groupe. Sans cela
    # le faux serveur survit à son test et s'accumule.
    try:
        os.setsid()
    except OSError:
        pass

    def watch_parent(initial):
        while os.getppid() == initial:
            time.sleep(0.5)
        os._exit(0)

    threading.Thread(target=watch_parent, args=(os.getppid(),), daemon=True).start()
    for received in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(received, lambda *_: os._exit(0))
    Handler.state = State(log_dir)
    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    port = server.server_address[1]
    runtime = HOME / "userdata"
    runtime.mkdir(parents=True, exist_ok=True)
    (runtime / "server-runtime.json").write_text(
        json.dumps(
            {
                "version": 1,
                "pid": os.getpid(),
                "host": "127.0.0.1",
                "port": port,
                "origin": f"http://127.0.0.1:{port}",
                "startedAt": now(),
            }
        )
    )
    sys.stdout.write(f"READY {port}\n")
    sys.stdout.flush()
    server.serve_forever()


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "auth":
        sys.exit(fake_cli(sys.argv[1:]))
    if len(sys.argv) == 3 and sys.argv[1] == "serve":
        serve(sys.argv[2])
    else:
        sys.stderr.write("usage : t3code_server_098.py serve <dossier de journal> | auth session ...\n")
        sys.exit(2)
