#!/usr/bin/env python3
"""Faux serveur MCP stdio jetable — spike-gate D-403 (session 010).

Rôle : prouver l'injection ÉPHÉMÈRE d'un serveur MCP par chaque harness
(Claude Code, Codex, Gemini, équipier ACP via mcpServers), sans daemon
Bridget et sans écrire aucune configuration utilisateur.

Protocole : JSON-RPC 2.0, un message par ligne (transport stdio MCP).
Outil unique : probe. Logs sur stderr exclusivement (pureté stdout).
"""
import json
import os
import sys

PROTOCOL_VERSION = "2025-06-18"


def log(msg):
    print(f"[fake-mcp] {msg}", file=sys.stderr, flush=True)
    # Le smoke T1006 peut demander une copie du journal sans mélanger ce
    # diagnostic au transport JSON-RPC stdout.
    path = os.environ.get("BRIDGET_MCP_SMOKE_LOG")
    if path:
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(f"[fake-mcp] {msg}\n")


def send(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
    sys.stdout.flush()


def result(id_, payload):
    send({"jsonrpc": "2.0", "id": id_, "result": payload})


def error(id_, code, message):
    send({"jsonrpc": "2.0", "id": id_, "error": {"code": code, "message": message}})


def main():
    log("démarrage")
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            log(f"ligne non-JSON ignorée: {line[:80]!r}")
            continue
        method = msg.get("method")
        id_ = msg.get("id")
        if method == "tools/call":
            log(f"reçu method={method} id={id_} name={(msg.get('params') or {}).get('name')}")
        else:
            log(f"reçu method={method} id={id_}")
        if method == "initialize":
            result(id_, {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "bridget-spike-probe", "version": "0.0.1"},
            })
        elif method == "notifications/initialized":
            pass  # notification, pas de réponse
        elif method == "tools/list":
            result(id_, {"tools": [{
                "name": "probe",
                "description": "Sonde du spike-gate Bridget 010 : répond PROBE_OK.",
                "inputSchema": {"type": "object", "properties": {}, "additionalProperties": False},
            }]})
        elif method == "tools/call":
            name = (msg.get("params") or {}).get("name")
            if name == "probe":
                result(id_, {"content": [{"type": "text", "text": "PROBE_OK"}], "isError": False})
            else:
                error(id_, -32602, f"outil inconnu: {name}")
        elif method == "ping":
            result(id_, {})
        elif id_ is not None:
            error(id_, -32601, f"méthode inconnue: {method}")
        else:
            log(f"notification inconnue ignorée: {method}")
    log("EOF stdin — arrêt propre")


if __name__ == "__main__":
    main()
