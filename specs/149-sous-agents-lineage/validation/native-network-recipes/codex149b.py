#!/usr/bin/python3
"""Serveur app-server Codex FERMÉ de recette 149 (ronde r3) : AUCUN modèle, AUCUN réseau.

Même protocole que codex149.py (r2), avec en plus :
- lecture de stdin NON bloquante (select + os.read) : le fournisseur voit un `turn/interrupt` pendant un tour ;
- HB_149:<ms>  : tour « vivant » qui écrit un battement d'horloge (fichier <evidence>.hb, réécrit toutes les 200 ms)
                 pendant <ms> au plus, répond à `turn/interrupt` du tour exact et s'arrête à l'EOF de stdin ;
- STREAM_149:<n>:<ms> : n fragments `item/agentMessage/delta` espacés de <ms> avant la réponse ;
- LATE_COMPLETE_149:<ms> : envoie la réponse tout de suite puis `turn/completed` <ms> plus tard ;
- IGNORE_TERM_149:<ms> : ignore SIGTERM pendant <ms> (borné), puis répond normalement (annulation lente à obtenir) ;
- journal « sigterm », « interrupt », « stdin_eof » dans le fichier d'évidence (PID, PPID, PGID, horloge).
Marqueurs hérités : WAIT_149, SLOW_149:<ms> (sommeil bloquant, non interruptible), REFUSE_149, NONCE_<id>, NESTED_149[...].
Chaque exécution consigne PID/argv/NOMS d'environnement (jamais de valeur).
"""
import json
import os
import re
import select
import signal
import subprocess
import sys
import time

evidence = sys.argv[1]
BRIDGET_BIN = sys.argv[2]
argv = sys.argv[3:]
HB_PATH = evidence + ".hb"
ignore_until = 0.0


def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps({"event": event, "pid": os.getpid(), "ppid": os.getppid(), "pgid": os.getpgrp(),
                                 "t": time.time(), **values}) + "\n")


def send(frame):
    print(json.dumps(frame), flush=True)


def on_term(_signum, _frame):
    ignored = time.time() < ignore_until
    record("sigterm", ignored=ignored)
    if not ignored:
        os._exit(143)


signal.signal(signal.SIGTERM, on_term)

_buf = b""


def read_line(timeout):
    """Une ligne de stdin, None si délai, '' si EOF."""
    global _buf
    deadline = time.time() + timeout
    while True:
        if b"\n" in _buf:
            line, _buf = _buf.split(b"\n", 1)
            return line.decode("utf-8", "replace")
        left = deadline - time.time()
        if left <= 0:
            return None
        ready, _, _ = select.select([0], [], [], left)
        if ready:
            chunk = os.read(0, 65536)
            if not chunk:
                return ""
            _buf += chunk


def nested_delegate(nonce, inner):
    child = subprocess.Popen([BRIDGET_BIN, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             stderr=subprocess.DEVNULL, text=True, env=dict(os.environ))
    def rpc(frame):
        child.stdin.write(json.dumps(frame) + "\n")
        child.stdin.flush()
    def read(expected):
        for out in child.stdout:
            value = json.loads(out)
            if value.get("id") == expected:
                return value
        return None
    rpc({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "codex149", "version": "1"}}})
    read(1)
    rpc({"jsonrpc": "2.0", "method": "notifications/initialized"})
    rpc({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "bridget_delegate", "arguments": {
        "request_id": f"nested-{nonce}", "agent_type": "fixture-codex-149", "model": "fixture-model-149",
        "effort": "high", "task": f"NONCE_{nonce}g {inner}", "cwd": os.getcwd()}}})
    response = read(2) or {}
    result = response.get("result", {})
    payload = result.get("structuredContent") or {}
    record("nested_delegate", is_error=result.get("isError", False), code=result.get("code"),
           status=payload.get("status"), task_id=payload.get("task_id"),
           child_agent_id=payload.get("child_agent_id"),
           refusal=(result.get("content") or [{}])[0].get("text") if result.get("isError") else None)
    child.stdin.close()
    try:
        child.wait(timeout=10)
    except Exception:
        child.terminate()
    return payload


def finish_turn(turns, nonce, status="completed", text=None):
    if text is not None:
        send({"method": "item/agentMessage/delta", "params": {"threadId": "thread-fx149",
              "turnId": f"turn-fx149-{turns}", "itemId": "i1", "delta": text}})
    send({"method": "turn/completed", "params": {"threadId": "thread-fx149",
          "turn": {"id": f"turn-fx149-{turns}", "status": status, "items": []}}})


def heartbeat_turn(turns, nonce, total_ms):
    """Tour vivant : battement toutes les 200 ms ; répond à turn/interrupt ; quitte à l'EOF."""
    end = time.time() + total_ms / 1000.0
    beats = 0
    record("hb_start", nonce=nonce, total_ms=total_ms, hb=HB_PATH)
    while time.time() < end:
        beats += 1
        with open(HB_PATH, "w", encoding="utf-8") as output:
            output.write(json.dumps({"pid": os.getpid(), "t": time.time(), "n": beats}) + "\n")
        line = read_line(0.2)
        if line is None:
            continue
        if line == "":
            record("stdin_eof", during="hb", beats=beats)
            os._exit(0)
        try:
            frame = json.loads(line)
        except ValueError:
            continue
        if frame.get("method") == "turn/interrupt":
            params = frame.get("params") or {}
            record("interrupt", turn_id=params.get("turnId"), thread_id=params.get("threadId"),
                   expected_turn=f"turn-fx149-{turns}", exact=params.get("turnId") == f"turn-fx149-{turns}", beats=beats)
            if frame.get("id") is not None:
                send({"id": frame["id"], "result": {}})
            finish_turn(turns, nonce, status="interrupted")
            record("hb_stop", reason="interrupt", beats=beats)
            return "interrupted"
        if frame.get("id") is not None:
            send({"id": frame["id"], "result": {}})
    record("hb_stop", reason="elapsed", beats=beats)
    return "elapsed"


record("started", provider="closed_codex_fixture_149_no_model_b", cwd=os.getcwd(), argv=argv,
       env_names=sorted(os.environ.keys()))
turns = 0
while True:
    line = read_line(3600)
    if line is None:
        continue
    if line == "":
        record("stdin_eof", during="idle")
        break
    try:
        frame = json.loads(line)
    except ValueError:
        continue
    method = frame.get("method")
    rid = frame.get("id")
    params = frame.get("params") or {}
    if method == "initialize":
        send({"id": rid, "result": {"userAgent": "fixture149", "codexHome": "/tmp",
                                    "platformFamily": "unix", "platformOs": "macos"}})
    elif method == "initialized":
        continue
    elif method in ("thread/start", "thread/resume"):
        record(method.replace("/", "_"), keys=sorted(params.keys()),
               sandbox=params.get("sandbox"), sandbox_policy=params.get("sandboxPolicy"),
               approval=params.get("approvalPolicy"), cwd=params.get("cwd"),
               model=params.get("model"))
        send({"id": rid, "result": {"thread": {"id": "thread-fx149"}, "model": params.get("model") or "fixture-model-149",
                                    "reasoningEffort": "high"}})
    elif method == "account/rateLimits/read":
        send({"id": rid, "result": {"rateLimits": {"primary": {"usedPercent": 1, "windowDurationMins": 300,
                                                                "resetsAt": 1900000000}, "rateLimitReachedType": None}}})
    elif method == "turn/interrupt":
        record("interrupt", turn_id=params.get("turnId"), thread_id=params.get("threadId"),
               expected_turn=f"turn-fx149-{turns}", exact=params.get("turnId") == f"turn-fx149-{turns}", during="idle_or_wait")
        if rid is not None:
            send({"id": rid, "result": {}})
    elif method == "turn/start":
        turns += 1
        text = " ".join(item.get("text", "") for item in params.get("input", []) if isinstance(item, dict))
        nonce = re.search(r"NONCE_([A-Za-z0-9]+)", text)
        nonce = nonce.group(1) if nonce else "none"
        own = re.sub(r"NESTED_149\[[^\]]*\]", "", text)
        wait = "WAIT_149" in own
        slow = re.search(r"SLOW_149:(\d+)", own)
        hb = re.search(r"HB_149:(\d+)", own)
        ign = re.search(r"IGNORE_TERM_149:(\d+)", own)
        late = re.search(r"LATE_COMPLETE_149:(\d+)", own)
        stream = re.search(r"STREAM_149:(\d+):(\d+)", own)
        refuse = "REFUSE_149" in own
        record("prompt", turn=turns, nonce=nonce, wait=wait, slow=slow.group(1) if slow else None,
               hb=hb.group(1) if hb else None, ignore_term=ign.group(1) if ign else None,
               refuse=refuse, chars=len(text), sandbox_policy=params.get("sandboxPolicy"),
               approval=params.get("approvalPolicy"), cwd=params.get("cwd"), model=params.get("model"),
               param_keys=sorted(params.keys()))
        if refuse:
            record("refused")
            send({"id": rid, "error": {"code": -32000, "message": "fixture149 model unavailable"}})
            continue
        send({"id": rid, "result": {"turn": {"id": f"turn-fx149-{turns}"}}})
        inner = re.search(r"NESTED_149\[([^\]]*)\]", text)
        if inner:
            nested_delegate(nonce, inner.group(1))
        if ign:
            ignore_until = time.time() + int(ign.group(1)) / 1000.0
            record("ignore_term_window", until=ignore_until)
        if stream:
            for i in range(int(stream.group(1))):
                send({"method": "item/agentMessage/delta", "params": {"threadId": "thread-fx149",
                      "turnId": f"turn-fx149-{turns}", "itemId": f"s{i}", "delta": f"chunk{i};"}})
                record("stream_chunk", i=i)
                time.sleep(int(stream.group(2)) / 1000.0)
        if hb:
            if heartbeat_turn(turns, nonce, int(hb.group(1))) == "interrupted":
                continue
        if wait:
            continue
        if slow:
            time.sleep(int(slow.group(1)) / 1000.0)
        if ign:
            while time.time() < ignore_until:
                time.sleep(0.1)
        answer = f"fixture149-answer:{nonce}"
        if late:
            # la réponse part tout de suite ; turn/completed arrive <ms> plus tard (fenêtre « résultat capturé / exécution encore active »)
            send({"method": "item/agentMessage/delta", "params": {"threadId": "thread-fx149",
                  "turnId": f"turn-fx149-{turns}", "itemId": "i1", "delta": answer}})
            record("answer_sent_late_complete", nonce=nonce, delay_ms=int(late.group(1)))
            time.sleep(int(late.group(1)) / 1000.0)
            finish_turn(turns, nonce)
        else:
            finish_turn(turns, nonce, text=answer)
        record("answered", nonce=nonce)
    elif rid is not None:
        send({"id": rid, "result": {}})
