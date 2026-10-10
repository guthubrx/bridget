#!/usr/bin/python3
"""Fournisseur fermé de recette : aucun réseau, aucun modèle réel."""
import json
import os
import sys

args = sys.argv[2:]
model = args[args.index("--model") + 1]
effort = args[args.index("--effort") + 1]
assert model == "glm5.3" and effort == "high"
assert "--restricted" in args
assert args[args.index("--permission-mode") + 1] == "plan"
assert not any(key.startswith("T3CODE_") or key.startswith("BRIDGET_T3_") for key in os.environ)
evidence = sys.argv[1]


def record(event, **values):
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps({"event": event, "pid": os.getpid(), **values}) + "\n")


def emit(frame):
    print(json.dumps(frame), flush=True)


record("started", provider="closed_fixture", model=model, effort=effort, t3_present=False)
initialized = False
for line in sys.stdin:
    frame = json.loads(line)
    if frame.get("type") != "user":
        continue
    content = frame.get("message", {}).get("content", "")
    if not isinstance(content, str):
        content = json.dumps(content)
    blocked = "WAIT_CANCEL_148" in content
    record("prompt", blocked=blocked, resume_card="Carte de reprise Bridget" in content, mission="Inspecte les faits locaux de la fixture148." in content)
    if not initialized:
        emit({"type": "system", "subtype": "init", "model": model, "session_id": "native148-fixture"})
        initialized = True
    if blocked:
        continue
    text = "fixture-glm5.3-answer-148"
    emit({"type": "stream_event", "event": {"type": "content_block_delta", "delta": {"type": "text_delta", "text": text}}})
    emit({"type": "result", "is_error": False, "terminal_reason": "completed", "result": text})
