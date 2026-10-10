#!/usr/bin/python3
"""Serveur app-server Codex FERMÉ de recette 149 : AUCUN modèle, AUCUN réseau.

Il parle le JSON-RPC ligne à ligne attendu par bridget-transport (initialize, thread/start,
turn/start, notifications item/agentMessage/delta et turn/completed). Chaque exécution réelle
est consignée dans le fichier d'évidence (PID, argv, NOMS d'environnement, paramètres reçus).
Aucune valeur d'environnement n'est consignée.

Marqueurs de mission : WAIT_149 (reste actif), SLOW_149:<ms>, REFUSE_149 (refus fournisseur),
NONCE_<id> (écho dans la réponse). Par défaut : « fixture149-answer:<nonce> ».
"""
import json
import os
import re
import subprocess
import sys
import time

evidence = sys.argv[1]
BRIDGET_BIN = sys.argv[2]
argv = sys.argv[3:]


def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps({"event": event, "pid": os.getpid(), "ppid": os.getppid(),
                                 "t": time.time(), **values}) + "\n")


def send(frame):
    print(json.dumps(frame), flush=True)


def nested_delegate(nonce, inner):
    """Comme un vrai Codex : lance le serveur MCP `bridget mcp` monté avec l'environnement
    d'identité de CET enfant (jamais celui du parent) et appelle bridget_delegate une fois.
    Le prompt de l'enfant n'est pas une preuve d'identité : seul l'environnement compte."""
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


record("started", provider="closed_codex_fixture_149_no_model", cwd=os.getcwd(), argv=argv,
       env_names=sorted(os.environ.keys()))
turns = 0
for line in sys.stdin:
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
    elif method == "turn/start":
        turns += 1
        text = " ".join(item.get("text", "") for item in params.get("input", []) if isinstance(item, dict))
        nonce = re.search(r"NONCE_([A-Za-z0-9]+)", text)
        nonce = nonce.group(1) if nonce else "none"
        # Les marqueurs entre NESTED_149[...] visent le petit-enfant, pas ce tour.
        own = re.sub(r"NESTED_149\[[^\]]*\]", "", text)
        wait = "WAIT_149" in own
        slow = re.search(r"SLOW_149:(\d+)", own)
        refuse = "REFUSE_149" in own
        record("prompt", turn=turns, nonce=nonce, wait=wait, slow=slow.group(1) if slow else None,
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
        if wait:
            continue
        if slow:
            time.sleep(int(slow.group(1)) / 1000.0)
        answer = f"fixture149-answer:{nonce}"
        send({"method": "item/agentMessage/delta", "params": {"threadId": "thread-fx149",
              "turnId": f"turn-fx149-{turns}", "itemId": "i1", "delta": answer}})
        send({"method": "turn/completed", "params": {"threadId": "thread-fx149",
              "turn": {"id": f"turn-fx149-{turns}", "status": "completed", "items": []}}})
        record("answered", nonce=nonce)
    elif rid is not None:
        send({"id": rid, "result": {}})
