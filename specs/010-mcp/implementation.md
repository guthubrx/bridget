# Journal d’implémentation — MCP natif

## T1001 — Spike-gate des quatre voies de branchement

- **Date** : 2026-08-22
- **Issue** : **GO — 3/4, Gemini documenté indisponible conformément à T708**
- **Serveur** :
  `/Users/moi/Nextcloud/10.Scripts/bridget/specs/010-mcp/spike/fake-mcp-server.py`
- **Préconditions** : `OPENAI_API_KEY` et `CODEX_API_KEY` absents lors du
  lancement ACP ; aucune commande persistante (`claude mcp add`, édition de
  `config.toml` ou de `settings.json`) n’a été utilisée.

### Preuve commune du serveur

L’auto-test `initialize` retourne uniquement une frame JSON-RPC sur stdout :

```text
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18",...}}
```

Les diagnostics `démarrage`, `initialize` et `EOF stdin` sont exclusivement
écrits sur stderr. Pour chaque voie validée, stderr a été redirigé vers un
fichier temporaire distinct ; les trois journaux contiennent la séquence :

```text
[fake-mcp] reçu method=initialize id=0
[fake-mcp] reçu method=notifications/initialized id=None
[fake-mcp] reçu method=tools/list id=1
[fake-mcp] reçu method=tools/call id=2
```

### Mesure de non-mutation

Le hash récursif des répertoires complets proposé dans le protocole inclut des
transcriptions et caches d’exécution concurrents ; il n’est pas un observable
déterministe de la configuration. La gate mesure donc le contenu des quatre
fichiers de configuration persistante avant et après **chaque** voie. Les
quatre hashes sont restés identiques lors des trois exécutions réussies :

```text
f7a41cefee60d352899c1e7aa8b0bfbce6f307e8  /Users/moi/.claude/settings.json
8ebd97c4a46dadaa1f40d3483bbdf01a706cd272  /Users/moi/.claude/settings.local.json
806d2899116225584e1fdcaaa1060d0fe731f80f  /Users/moi/.codex/config.toml
0210305302e4f71b17a3b490382e92fa225f1e19  /Users/moi/.gemini/settings.json
```

Une recherche finale ne trouve ni chemin du faux serveur, ni serveur `probe`,
dans ces fichiers ou dans `/Users/moi/.claude.json`.

### Voie 1 — Claude Code

- **Version** : `2.1.234 (Claude Code)`
- **Commande** :

```text
claude -p 'Appelle exactement une fois l outil MCP probe puis réponds uniquement avec le texte retourné par cet outil.' --output-format json --dangerously-skip-permissions --strict-mcp-config --mcp-config /tmp/bridget-mcp-claude-cxbridget.json
```

Le fichier temporaire passé à `--mcp-config` contenait uniquement :

```json
{
  "mcpServers": {
    "probe": {
      "type": "stdio",
      "command": "/bin/sh",
      "args": [
        "-c",
        "exec python3 /Users/moi/Nextcloud/10.Scripts/bridget/specs/010-mcp/spike/fake-mcp-server.py 2>/tmp/bridget-mcp-claude-probe.log"
      ]
    }
  }
}
```

**Preuves** : `tools/list` puis `tools/call` dans stderr du serveur ; résultat
final du harness `PROBE_OK` ; diff des quatre hashes vide.

### Voie 2 — Codex CLI

- **Version** : `codex-cli 0.149.0`
- **Commande** :

```text
codex --dangerously-bypass-approvals-and-sandbox -c 'mcp_servers.probe={command="/bin/sh",args=["-c","exec python3 /Users/moi/Nextcloud/10.Scripts/bridget/specs/010-mcp/spike/fake-mcp-server.py 2>/tmp/bridget-mcp-codex-probe.log"]}' exec --ephemeral --ignore-user-config --skip-git-repo-check -C /Users/moi/Nextcloud/10.Scripts/bridget --json 'Appelle exactement une fois l outil MCP probe puis réponds uniquement avec le texte retourné par cet outil.'
```

**Preuves** : événement `mcp_tool_call` `probe/probe` terminé avec contenu
`PROBE_OK`, message final `PROBE_OK`, `tools/list` puis `tools/call` dans stderr
du serveur, diff des quatre hashes vide.

### Voie 3 — Gemini CLI

- **Versions constatées en T708** : `gemini 0.46.0`, puis
  `@google/gemini-cli 0.56.0` pinné.
- **Statut** : non testable pour un compte individuel, décision déjà actée par
  la révision de la session 007.
- **Refus du harness au `session/new`** :

```text
This client is no longer supported for Gemini Code Assist for individuals.
To continue using Gemini, please migrate to the Antigravity suite of products.
```

Aucun contournement, aucune écriture de configuration et aucune promesse de
support silencieuse ne sont introduits.

### Voie 4 — équipier ACP

- **Adaptateur pinné** : `@zed-industries/codex-acp 0.16.0`
- **Commande du harness** :

```text
npx @zed-industries/codex-acp@0.16.0 -c 'model="gpt-5.5"'
```

Le client JSON-RPC de spike a envoyé :

```json
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget","mcpServers":[{"name":"probe","command":"/bin/sh","args":["-c","exec python3 /Users/moi/Nextcloud/10.Scripts/bridget/specs/010-mcp/spike/fake-mcp-server.py 2>/tmp/bridget-mcp-acp-probe.log"],"env":[]}]}}
```

Puis `session/prompt` a demandé l’appel unique. L’adaptateur a émis un
`tool_call` intitulé `Tool: probe/probe`, demandé la permission ACP
`allow_once`, reçu l’option `approved`, puis retourné un `tool_call_update`
contenant `PROBE_OK`, trois chunks texte `PRO`/`BE`/`_OK` et
`stopReason=end_turn`. stderr du serveur prouve `tools/list` puis `tools/call` ;
le diff des quatre hashes est vide.

### Nettoyage et self-review

- Les trois journaux stderr, la configuration Claude temporaire et le home
  isolé d’une tentative diagnostique ont été supprimés.
- Aucun processus `fake-mcp-server.py` ou `codex-acp` du spike ne subsiste.
- Aucun fichier source ou configuration utilisateur n’a été modifié.
- La gate prouve les formes d’injection éphémère ; elle ne préjuge pas encore
  du serveur Bridget MCP, qui relève des tâches suivantes.
