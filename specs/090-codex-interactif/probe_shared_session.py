#!/usr/bin/env python3
"""Sonde de conception : vrai Codex, fournisseur HTTP synthétique local, zéro secret.

Le cadrage WebSocket réduit est réservé à cette sonde, jamais au produit.
Tout échange est borné ; le processus fournisseur de test est explicitement arrêté.
"""
import http.server
import json
import os
import pathlib
import select
import socket
import struct
import subprocess
import sys
import tempfile
import threading
import time


class Fixture(http.server.BaseHTTPRequestHandler):
    count = 0
    requests = []
    reply_arguments = None
    reply_sent = False
    first_release = None
    def log_message(self, *_args):
        pass

    def do_POST(self):
        request = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        Fixture.count += 1
        Fixture.requests.append(request)
        if Fixture.count == 1 and Fixture.first_release is not None:
            assert Fixture.first_release.wait(20), "barrière du tour humain non libérée"
        if Fixture.count == 1:
            print("tools", [v.get("name", v.get("type")) for v in request.get("tools", [])], flush=True)
        item = {"id": "msg_fixture", "type": "message", "role": "assistant",
                "content": [{"type": "output_text", "text": "OK-090", "annotations": []}]}
        if Fixture.reply_arguments and not Fixture.reply_sent:
            Fixture.reply_sent = True
            item = {"id": "fc_mcp090", "type": "function_call", "call_id": "call_mcp090",
                    "namespace": "mcp__bridget", "name": "bridget_send",
                    "arguments": json.dumps(Fixture.reply_arguments)}
        if "--approval" in sys.argv and Fixture.count == 1:
            item = {"id": "fc_fixture", "type": "function_call", "call_id": "call_fixture", "name": "exec_command",
                    "arguments": json.dumps({"cmd": "printf APPROVAL_SENTINEL", "sandbox_permissions": "require_escalated", "justification": "Sonde locale : refuser cette commande."})}
        events = [
            {"type": "response.created", "response": {"id": "resp_fixture", "status": "in_progress", "output": []}},
            {"type": "response.output_item.added", "output_index": 0, "item": item},
            {"type": "response.output_text.delta", "item_id": "msg_fixture", "output_index": 0, "content_index": 0, "delta": "OK-090"},
            {"type": "response.output_item.done", "output_index": 0, "item": item},
            {"type": "response.completed", "response": {"id": "resp_fixture", "status": "completed", "output": [item], "usage": {"input_tokens": 1, "output_tokens": 1, "total_tokens": 2}}},
        ]
        body = "".join(f"event: {v['type']}\ndata: {json.dumps(v)}\n\n" for v in events).encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def exact(stream, count):
    result = b""
    while len(result) < count:
        chunk = stream.recv(count - len(result))
        if not chunk:
            raise EOFError("socket Codex fermée")
        result += chunk
    return result


class Client:
    def __init__(self, path):
        self.socket = socket.socket(socket.AF_UNIX)
        self.socket.settimeout(10)
        self.socket.connect(path)
        self.socket.sendall(b"GET / HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n")
        header = b""
        while not header.endswith(b"\r\n\r\n"):
            header += exact(self.socket, 1)
        assert header.startswith(b"HTTP/1.1 101"), header
        self.events = []
        self.id = 0
        self.rpc("initialize", {"clientInfo": {"name": "bridget_probe", "version": "090"}, "capabilities": {"experimentalApi": "--no-experimental" not in sys.argv}})
        self.send({"method": "initialized", "params": {}})

    def send(self, value):
        data = json.dumps(value).encode()
        mask = os.urandom(4)
        size = len(data)
        header = bytes([129, 128 | size]) if size < 126 else bytes([129, 254]) + struct.pack("!H", size)
        self.socket.sendall(header + mask + bytes(c ^ mask[i % 4] for i, c in enumerate(data)))

    def receive(self):
        header = exact(self.socket, 2)
        assert header[0] == 129 and not header[1] & 128, header
        count = header[1] & 127
        if count == 126:
            count = struct.unpack("!H", exact(self.socket, 2))[0]
        if count == 127:
            count = struct.unpack("!Q", exact(self.socket, 8))[0]
        assert count < 4_000_000
        return json.loads(exact(self.socket, count))

    def rpc(self, method, params):
        self.id += 1
        self.send({"id": self.id, "method": method, "params": params})
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            self.socket.settimeout(max(0.001, deadline - time.monotonic()))
            value = self.receive()
            if value.get("id") == self.id:
                return value
            self.events.append(value)
        raise TimeoutError(method)


def main():
    root = tempfile.mkdtemp(prefix="bg090-", dir="/tmp")
    os.chmod(root, 0o700)
    provider = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Fixture)
    threading.Thread(target=provider.serve_forever, daemon=True).start()
    path = root + "/s.sock"
    environment = {"HOME": root, "CODEX_HOME": root, "PATH": "/opt/homebrew/bin:/usr/bin:/bin"}
    args = ["/opt/homebrew/bin/codex", "-c", 'model_provider="fixture"', "-c", 'model="fixture"',
            "-c", 'model_providers.fixture.name="fixture"', "-c", 'model_providers.fixture.wire_api="responses"',
            "-c", f'model_providers.fixture.base_url="http://127.0.0.1:{provider.server_port}/v1"',
            "-c", 'model_providers.fixture.requires_openai_auth=false', "app-server", "--listen", "unix://" + path]
    server = subprocess.Popen(args, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    clients = []
    try:
        deadline = time.monotonic() + 5
        while not os.path.exists(path):
            assert server.poll() is None, "app-server quitté"
            assert time.monotonic() < deadline, "socket absente"
            select.select([], [], [], 0.01)
        a, b = Client(path), Client(path)
        clients = [a, b]
        params = {"cwd": root, "approvalPolicy": "on-request", "sandbox": "read-only"}
        if "--legacy-history" in sys.argv:
            params["historyMode"] = "legacy"
        started = a.rpc("thread/start", params)
        assert "result" in started, started
        tid = started["result"]["thread"]["id"]
        print("start", tid, flush=True)
        for label, client in [("A", a), ("B", b)]:
            print("loaded", label, client.rpc("thread/loaded/list", {}), flush=True)
        print("resume_before_turn", b.rpc("thread/resume", {"threadId": tid}), flush=True)
        print("name", a.rpc("thread/name/set", {"threadId": tid, "name": "bridget-probe"}), flush=True)
        print("resume_after_name", b.rpc("thread/resume", {"threadId": tid}), flush=True)
        if "--inspect-empty" in sys.argv:
            print("read_empty", b.rpc("thread/read", {"threadId": tid, "includeTurns": True}), flush=True)
            print("history_files", list(pathlib.Path(root).rglob("rollout*")), flush=True)
            return
        turn = a.rpc("turn/start", {"threadId": tid, "input": [{"type": "text", "text": "Test synthétique local."}]})
        print("turn", turn, flush=True)
        deadline = time.monotonic() + 10
        approvals = set()
        while time.monotonic() < deadline:
            sockets = select.select([a.socket, b.socket], [], [], max(0, deadline - time.monotonic()))[0]
            for stream in sockets:
                client = a if stream is a.socket else b
                event = client.receive()
                if "requestApproval" in event.get("method", ""):
                    label = "A" if client is a else "B"
                    approvals.add(label)
                    print("approval_recipient", label, event["method"], event["id"], flush=True)
                    if client is b:
                        b.send({"id": event["id"], "result": {"decision": "decline"}})
                if event.get("method") == "turn/completed":
                    print("terminal", "A" if client is a else "B", event["params"]["turn"]["status"], flush=True)
                    deadline = 0
                    break
            if deadline == 0:
                break
        else:
            if "--approval" in sys.argv:
                print("approval_recipients", sorted(approvals), flush=True)
                a.rpc("turn/interrupt", {"threadId": tid, "turnId": turn["result"]["turn"]["id"]})
            raise TimeoutError("turn/completed")
        print("resume_after_turn", b.rpc("thread/resume", {"threadId": tid}).get("error", "OK"), flush=True)
        print("B_methods", [v.get("method") for v in b.events], flush=True)
    finally:
        for client in clients:
            client.socket.close()
        subprocess.run(["/bin/ps", "-p", str(server.pid), "-o", "pid=,comm="], check=False)
        server.terminate()
        server.wait(timeout=10)
        provider.shutdown()
        print("probe_root", root, flush=True)


if __name__ == "__main__":
    main()
