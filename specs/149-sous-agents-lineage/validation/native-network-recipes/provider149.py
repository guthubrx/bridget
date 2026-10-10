#!/usr/bin/python3
"""Fournisseur fermé de recette 149 : AUCUN modèle, AUCUN réseau.

Adapté de crates/bridget-daemon/tests/fixtures/native_delegation_148.py (copie d'outil,
sans nouveau framework). Il parle le protocole claude_stream_json et consigne chaque
exécution réelle dans un fichier d'évidence (un lancement = une ligne `started` avec le PID).
Il ne consigne jamais de valeur d'environnement : seulement les NOMS des variables.

Comportements pilotés par des marqueurs dans la mission :
  WAIT_149      bloque jusqu'à l'arrêt du processus (mission active)
  SLOW_149:<ms> attend <ms> puis répond (mission active un temps borné)
  REFUSE_149    le fournisseur refuse (erreur explicite, sans réponse de succès)
  par défaut    répond « fixture149-answer:<nonce> » (nonce lu dans la mission)
"""
import json
import os
import re
import sys
import time

evidence = sys.argv[1]
args = sys.argv[2:]
MODEL = args[args.index("--model") + 1] if "--model" in args else None


def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps({"event": event, "pid": os.getpid(), "ppid": os.getppid(),
                                 "t": time.time(), **values}) + "\n")


def emit(frame):
    print(json.dumps(frame), flush=True)


forbidden = sorted(k for k in os.environ if k.startswith("T3CODE_") or k.startswith("BRIDGET_T3_"))
record(
    "started",
    provider="closed_fixture_149_no_model",
    model=MODEL,
    cwd=os.getcwd(),
    argv=args,
    env_names=sorted(os.environ.keys()),
    t3_env_names=forbidden,
)
initialized = False
for line in sys.stdin:
    try:
        frame = json.loads(line)
    except ValueError:
        continue
    if frame.get("type") != "user":
        continue
    content = frame.get("message", {}).get("content", "")
    if not isinstance(content, str):
        content = json.dumps(content)
    nonce = re.search(r"NONCE_([A-Za-z0-9]+)", content)
    nonce = nonce.group(1) if nonce else "none"
    wait = "WAIT_149" in content
    slow = re.search(r"SLOW_149:(\d+)", content)
    refuse = "REFUSE_149" in content
    record("prompt", nonce=nonce, wait=wait, slow=slow.group(1) if slow else None,
           refuse=refuse, chars=len(content))
    if not initialized:
        emit({"type": "system", "subtype": "init", "model": MODEL, "session_id": "fixture149"})
        initialized = True
    if wait:
        continue
    if refuse:
        record("refused")
        emit({"type": "result", "is_error": True, "terminal_reason": "model_unavailable",
              "result": "fixture149-refused"})
        continue
    if slow:
        time.sleep(int(slow.group(1)) / 1000.0)
    text = f"fixture149-answer:{nonce}"
    emit({"type": "stream_event", "event": {"type": "content_block_delta",
                                            "delta": {"type": "text_delta", "text": text}}})
    emit({"type": "result", "is_error": False, "terminal_reason": "completed", "result": text})
    record("answered", nonce=nonce)
