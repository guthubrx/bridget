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

## T1003 — Dispatcher MCP stdio

- `bridget mcp` exécute un unique lecteur stdin et un writer stdout sérialisé.
  Celui-ci ne produit que des lignes JSON-RPC 2.0 ; les diagnostics restent
  sur stderr.
- La négociation expose exclusivement la capacité `tools`, avec la version
  MCP `2025-06-18`. Les réponses conservent l'identifiant JSON-RPC numérique
  ou chaîne reçu.
- La fixture versionnée
  `crates/bridget-daemon/tests/fixtures/mcp/fr009.jsonl` contient les quinze
  cas FR-009. Les tests vérifient notamment les rappels idempotents de
  `tools/list`, `ping`, les notifications, les identifiants mixtes et les
  erreurs de protocole. Le golden de pureté décode chaque octet de stdout
  comme réponse JSON-RPC.

## T1006 — Branchement MCP éphémère par session

- **Commande de preuve** :

```text
BRIDGET_MCP_REAL_SMOKE=1 cargo test -p bridget-daemon --features test-support \
  --test mcp_injection_smoke_test -- --include-ignored --test-threads=1
```

- **Résultat** : **3/3 vert**, durée 93,18 s. Le banc lance le wrapper de
  production (`CARGO_BIN_EXE_bridget`) pour Codex et Claude ; le scénario ACP
  lance le vrai wrapper équipier et `AcpTransport` avec une session
  `session/new`. Dans les trois cas, le serveur épinglé
  `specs/010-mcp/spike/fake-mcp-server.py` observe `tools/call name=probe` et
  le harness reçoit `PROBE_OK`.
- **Versions observées** : Codex CLI `0.149.0`, Claude Code `2.1.234`.
  Gemini demeure `unsupported`, conformément au constat T708/T1001.
- **Trace Codex** : `initialize → notifications/initialized → tools/call
  name=probe → PROBE_OK`. Cette version appelle directement l’outil sans
  `tools/list` (différent du spike T1001) ; l’exécution réussie de `probe`
  établit néanmoins que l’injection éphémère est effective.
- **Traces Claude et ACP** : `initialize → notifications/initialized →
  tools/list → tools/call name=probe → PROBE_OK`.
- **Non-mutation** : avant/après chaque voie interactive, le banc compare les
  octets des fichiers réels `~/.claude/settings.json`,
  `~/.claude/settings.local.json`, `~/.codex/config.toml` et
  `~/.gemini/settings.json` ; le diff est vide. Le scénario ACP utilise un
  HOME isolé avec les mêmes sentinelles et vérifie le même invariant. La
  configuration Claude temporaire est protégée par un garde RAII, y compris
  en cas de refus avant `spawn`.

## T1007 — Prompt allégé

### Phase A — Fixtures et mesure statique

Les blocs v1 avant/après sont versionnés sous
`crates/bridget-daemon/tests/fixtures/prompts/`. La fixture avant reprend le
bloc interactif historique avec un nom d'agent stable ; la fixture après ne
conserve que l'identité de session, la reconnaissance du préfixe `💬` et la
sémantique `reply=yes`/`reply=no`. Elle ne contient plus de commande shell, de
syntaxe d'envoi, de règle `Bridget ready` ni d'interdiction de consulter le
binaire.

Méthode de comptage SC-005 : nombre de valeurs scalaires Unicode retourné par
`str::chars().count()` sur le contenu brut UTF-8 de chaque fixture, saut LF
final inclus, sans normalisation Unicode. Résultat : **793 caractères avant**,
**234 après**, soit une réduction de **70,49 %**. Le test
`prompt_reduction_test` verrouille le seuil de 60 % et l'absence des règles de
syntaxe supprimées.

La phase B branchera exactement la fixture réduite dans le wrapper puis
rejouera la matrice quickstart 007 §1 à §4 dès que T1005 et T1006 seront
intégrées.
