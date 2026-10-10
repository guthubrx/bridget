"""Relais transparent de la CLI réelle ; archive seulement les noms de modèle."""
import json
import os
import signal
import subprocess
import sys
import threading

evidence, executable, *arguments = sys.argv[1:]
child = subprocess.Popen([executable, *arguments], stdin=sys.stdin.buffer,
                         stdout=subprocess.PIPE, stderr=sys.stderr.buffer)


def terminate(signum, _frame):
    # Ce PID provient de Popen et reste notre enfant ; aucun PID externe.
    if child.poll() is None:
        child.send_signal(signum)


signal.signal(signal.SIGTERM, terminate)
signal.signal(signal.SIGINT, terminate)


def forward():
    for line in child.stdout:
        try:
            event = json.loads(line)
            models = set()
            if isinstance(event.get("model"), str):
                models.add(event["model"])
            usage = event.get("modelUsage", event.get("model_usage", {}))
            if isinstance(usage, dict):
                models.update(str(key) for key in usage)
            message = event.get("message", {})
            if isinstance(message, dict) and isinstance(message.get("model"), str):
                models.add(message["model"])
            if models:
                with open(evidence, "a", encoding="utf-8") as target:
                    target.write(json.dumps({"event": "provider_model", "models": sorted(models)}) + "\n")
        except (ValueError, TypeError, OSError):
            pass
        sys.stdout.buffer.write(line)
        sys.stdout.buffer.flush()


reader = threading.Thread(target=forward)
reader.start()
status = child.wait()
reader.join()
sys.exit(status if status >= 0 else 128 - status)
