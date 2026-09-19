#!/usr/bin/env python3
"""Recette réelle, sans modèle ni daemon : un pair Unix local et un pair SSH-Unix."""

import argparse
import json
import os
import re
import select
import shlex
import socket
import stat
import subprocess
import time
import uuid


REMOTE = r'''import json,socket,sys,uuid
sock_path,agent_id=sys.argv[1:]
s=socket.socket(socket.AF_UNIX); s.settimeout(15); s.connect(sock_path); f=s.makefile("rb")
def send(v): s.sendall((json.dumps(v,separators=(",",":"))+"\n").encode())
def recv(*kinds):
    v=json.loads(f.readline())
    if v.get("type") not in kinds: raise RuntimeError("expected "+str(kinds)+", got "+str(v.get("type")))
    return v
def out(v): print(json.dumps(v,separators=(",",":")),flush=True)
send({"type":"Register","agent_type":"fixture-095-roundtrip","identity_version":2,
      "agent_id":agent_id,"host":"federation-095-remote-probe","transport":"ssh",
      "channel":"ssh-unix","mode":"acp","os":"linux","instance_id":str(uuid.uuid4()),
      "journal_available":False,"turn_in_progress":False})
r=recv("Registered"); out({"event":"registered","agent_id":r["agent_id"]})
for line in sys.stdin:
    c=json.loads(line)
    if c["op"] == "list":
        send({"type":"ListAgents"}); a=recv("AgentList")["agents"]
        out({"event":"list","ids":[x["agent_id"] for x in a]})
    elif c["op"] == "send":
        send(c["message"]); a=recv("Ack"); out({"event":"sent","id":a["id"]})
    elif c["op"] == "receive":
        d=recv("Deliver","DeliverIdempotent"); m=d.get("message",d)
        if d["type"] == "DeliverIdempotent": send({"type":"DeliverAcked","delivery_id":d["delivery_id"],"delivery_generation":d["delivery_generation"]})
        out({"event":"received","id":m["id"],"from":m["from"],"to":m["to"],"wire_type":d["type"]})
    elif c["op"] == "close":
        send({"type":"Unregister"}); send({"type":"ListAgents"}); recv("AgentList")
        out({"event":"closed"}); break
f.close(); s.close()
'''


class Peer:
    def __init__(self, path, timeout):
        self.sock = socket.socket(socket.AF_UNIX)
        self.sock.settimeout(timeout)
        self.sock.connect(path)
        self.file = self.sock.makefile("rb")

    def send(self, value):
        self.sock.sendall((json.dumps(value, separators=(",", ":")) + "\n").encode())

    def recv(self, *kinds):
        value = json.loads(self.file.readline())
        if value.get("type") not in kinds:
            raise RuntimeError(f"{kinds} attendu, reçu {value.get('type')}")
        return value

    def close(self):
        self.file.close()
        self.sock.close()


def main():
    p = argparse.ArgumentParser()
    for name in ("host", "user", "identity", "known-hosts", "local-socket", "remote-socket", "journal"):
        p.add_argument("--" + name, required=True)
    p.add_argument("--port", required=True, type=int)
    p.add_argument("--timeout", type=float, default=15)
    a = p.parse_args()
    if not 1 <= a.port <= 65535:
        p.error("--port hors plage")
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9.-]*", a.host) or ".." in a.host:
        p.error("--host invalide")
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_-]*", a.user):
        p.error("--user invalide")
    for path in (a.identity, a.known_hosts):
        mode = stat.S_IMODE(os.stat(path).st_mode)
        if not stat.S_ISREG(os.stat(path).st_mode) or mode != 0o600:
            p.error(f"fichier SSH régulier 0600 requis: {path}")
    if not stat.S_ISSOCK(os.stat(a.local_socket).st_mode):
        p.error("--local-socket ne désigne pas une socket")
    journal_fd = os.open(a.journal, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    journal = os.fdopen(journal_fd, "w")
    count = 0
    def record(event, **facts):
        nonlocal count
        count += 1
        if count > 24:
            raise RuntimeError("journal borné dépassé")
        print(json.dumps({"seq": count, "at": int(time.time()), "event": event, **facts},
                         separators=(",", ":"), sort_keys=True), file=journal, flush=True)
    local_id, remote_id = uuid.uuid4(), uuid.uuid4()
    local = Peer(a.local_socket, a.timeout)
    ssh = ["ssh", "-F", "/dev/null", "-p", str(a.port), "-i", a.identity,
           "-o", "IdentitiesOnly=yes", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
           "-o", "ConnectionAttempts=1", "-o", "ServerAliveInterval=5",
           "-o", "ServerAliveCountMax=2", "-o", "RequestTTY=no",
           "-o", "StrictHostKeyChecking=yes", "-o", f"UserKnownHostsFile={a.known_hosts}",
           "-o", "GlobalKnownHostsFile=/dev/null", "-o", "ForwardAgent=no", "-o", "ForwardX11=no"]
    remote_command = " ".join(map(shlex.quote, ("python3", "-u", "-c", REMOTE, a.remote_socket, str(remote_id))))
    ssh.extend((f"{a.user}@{a.host}", remote_command))
    remote = subprocess.Popen(ssh, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, bufsize=1)
    pending = b""
    def stderr_tail():
        chunks, deadline = [], time.monotonic() + min(a.timeout, 1)
        while sum(map(len, chunks)) < 2048:
            remaining = deadline - time.monotonic()
            if remaining <= 0 or not select.select((remote.stderr,), (), (), remaining)[0]:
                break
            chunk = os.read(remote.stderr.fileno(), 2048 - sum(map(len, chunks)))
            if not chunk:
                break
            chunks.append(chunk)
        return b"".join(chunks).decode(errors="replace").strip()
    def call(command=None):
        nonlocal pending
        if command:
            remote.stdin.write(json.dumps(command, separators=(",", ":")) + "\n"); remote.stdin.flush()
        deadline = time.monotonic() + a.timeout
        while b"\n" not in pending:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(f"sonde SSH muette depuis {a.timeout:g}s")
            ready, _, _ = select.select((remote.stdout,), (), (), remaining)
            if not ready:
                raise TimeoutError(f"sonde SSH muette depuis {a.timeout:g}s")
            chunk = os.read(remote.stdout.fileno(), 4096)
            if not chunk:
                raise RuntimeError(f"sonde SSH fermée (code {remote.poll()}): {stderr_tail()}")
            pending += chunk
        line, pending = pending.split(b"\n", 1)
        return json.loads(line)
    local_registered = remote_registered = False
    try:
        registered = call()
        remote_registered = True
        local.send({"type":"Register","agent_type":"fixture-095-roundtrip","identity_version":2,
          "agent_id":str(local_id),"host":"federation-095-local-probe","transport":"unix",
          "channel":"unix","mode":"acp","os":"macos","instance_id":str(uuid.uuid4()),
          "journal_available":False,"turn_in_progress":False})
        assert local.recv("Registered")["agent_id"] == str(local_id)
        local_registered = True
        assert registered == {"event":"registered","agent_id":str(remote_id)}
        record("registered", local=str(local_id), remote=str(remote_id))
        local.send({"type":"ListAgents"}); local_ids=[x["agent_id"] for x in local.recv("AgentList")["agents"]]
        remote_ids=call({"op":"list"})["ids"]
        assert {str(local_id),str(remote_id)} <= set(local_ids) == set(remote_ids)
        record("same_inventory", probes=sorted((str(local_id), str(remote_id))))
        for sender, recipient, peer in ((local_id,remote_id,local),(remote_id,local_id,None)):
            mid=str(uuid.uuid4()); message={"type":"Send","id":mid,"from":str(sender),"to":str(recipient),
                                           "body":"federation-095-roundtrip","reply":False}
            if peer: peer.send(message); assert peer.recv("Ack")["id"] == mid; event=call({"op":"receive"})
            else: assert call({"op":"send","message":message})["id"] == mid; d=local.recv("Deliver","DeliverIdempotent"); m=d.get("message",d); d["type"] == "DeliverIdempotent" and local.send({"type":"DeliverAcked","delivery_id":d["delivery_id"],"delivery_generation":d["delivery_generation"]}); event={"id":m["id"],"from":m["from"],"to":m["to"],"wire_type":d["type"]}
            assert (event["id"],event["from"],event["to"]) == (mid,str(sender),str(recipient)); record("delivered", id=mid, sender=str(sender), recipient=str(recipient), wire_type=event["wire_type"])
        assert call({"op":"close"})["event"] == "closed"
        remote_registered = False
        local.send({"type":"Unregister"}); local.send({"type":"ListAgents"}); final=local.recv("AgentList")
        local_registered = False
        final_states={x["agent_id"]:x["state"] for x in final["agents"] if x["agent_id"] in {str(local_id),str(remote_id)}}
        assert set(final_states) == {str(local_id),str(remote_id)} and not {"connected","busy"} & set(final_states.values())
        record("disconnected", states=final_states)
        remote.stdin.close()
        assert remote.wait(timeout=a.timeout) == 0
    finally:
        if remote_registered and remote.poll() is None:
            try: call({"op":"close"})
            except Exception: pass
        if local_registered:
            try: local.send({"type":"Unregister"})
            except Exception: pass
        local.close(); journal.close()
        try: remote.wait(timeout=0.2)
        except subprocess.TimeoutExpired: pass
        if remote.poll() is None:
            observed = subprocess.run(("ps", "-p", str(remote.pid), "-o", "command="),
                                      capture_output=True, text=True, check=False).stdout.strip()
            words = shlex.split(observed) if observed else []
            if not words and remote.poll() is None:
                raise RuntimeError(f"refus de terminer le PID enfant non observable {remote.pid}")
            if words and (os.path.basename(words[0]) != "ssh" or "firefox" in observed.lower()):
                raise RuntimeError(f"refus de terminer le PID enfant non SSH {remote.pid}")
            if remote.poll() is None:
                remote.terminate(); remote.wait(timeout=a.timeout)


if __name__ == "__main__":
    main()
