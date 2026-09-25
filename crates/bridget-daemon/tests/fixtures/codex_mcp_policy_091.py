"""Le vrai parseur/config/read Codex lit la projection de production, sans LLM.

Mutation : retirer tools ou remplacer approve par auto fait échouer l'oracle.
Le serveur MCP n'est jamais lancé : aucun thread/start ni appel métier.
"""
import json
import os
import queue
import subprocess
import sys
import tempfile
import threading
import time


with tempfile.TemporaryDirectory(prefix="b91-policy-", dir="/tmp") as root:
    # Configuration antagoniste : l'injection ne doit ni autoriser le reste
    # de Bridget, ni toucher un serveur extérieur, ni élargir le sandbox.
    with open(os.path.join(root, "config.toml"), "w", encoding="utf-8") as config:
        config.write('approval_policy="never"\nsandbox_mode="workspace-write"\n'
                     '[sandbox_workspace_write]\nnetwork_access=false\n'
                     '[mcp_servers.other]\ncommand="/nonexistent/other"\n'
                     'default_tools_approval_mode="prompt"\n')
    process = subprocess.Popen(
        [os.environ.get("BRIDGET_TEST_CODEX_BIN", "/opt/homebrew/bin/codex"),
         *json.loads(sys.argv[1])],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
        text=True, cwd=root, env={**os.environ, "CODEX_HOME": root},
    )
    replies = queue.Queue()

    def read():
        for line in process.stdout:
            replies.put(json.loads(line))
        replies.put(None)

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    try:
        for message in [
            {"id": 1, "method": "initialize", "params": {
                "clientInfo": {"name": "bridget-policy-test", "version": "1"}}},
            {"method": "initialized"},
            {"id": 2, "method": "config/read", "params": {"includeLayers": False}},
        ]:
            process.stdin.write(json.dumps(message) + "\n")
            process.stdin.flush()
        deadline = time.monotonic() + 10
        while True:
            reply = replies.get(timeout=max(0, deadline - time.monotonic()))
            assert reply is not None, "EOF avant config/read"
            if reply.get("id") == 2:
                config = reply["result"]["config"]
                break
        bridget = config["mcp_servers"]["bridget"]
        assert bridget["default_tools_approval_mode"] == "prompt", bridget
        assert bridget["tools"] == {
            name: {"approval_mode": "approve"} for name in
            ("bridget_who", "bridget_send", "bridget_ledger", "bridget_cancel",
             "bridget_read_artifact", "bridget_publish_artifact",
             "bridget_rename", "bridget_dnd", "bridget_domain", "bridget_runtime",
             "bridget_status", "bridget_control_status")
        }, bridget
        assert config["mcp_servers"]["other"]["default_tools_approval_mode"] == "prompt"
        assert config["approval_policy"] == "never", config["approval_policy"]
        assert config["model"] == "gpt-5.6-terra", config["model"]
        assert config["sandbox_mode"] == "workspace-write", config["sandbox_mode"]
        assert config["sandbox_workspace_write"]["network_access"] is False
    finally:
        # Seulement le processus de ce test, jamais un daemon/agent de production.
        process.terminate()
        process.wait(timeout=5)
        reader.join(timeout=2)
