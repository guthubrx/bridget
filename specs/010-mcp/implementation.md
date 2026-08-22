# Journal d’implémentation — MCP natif

## Checklist finale SC-001 à SC-006

- [x] **SC-001** — le dispatcher MCP conserve un unique lecteur et un writer
  sérialisé ; la matrice FR-009 de 15 cas est couverte par
  `crates/bridget-daemon/tests/fixtures/mcp/fr009.jsonl` et ses tests.
- [x] **SC-002** — l'identité est résolue à chaque `tools/call` : tests de
  renommage, filiation et marqueur historique dans `mcp_identity.rs`.
- [x] **SC-003** — `bridget_send` respecte l'idempotence 012 : corpus MCP et
  tests de retry, refus et `outcome_unknown` dans `mcp.rs`.
- [x] **SC-004** — les trois voies disponibles ont passé le spike T1001 sans
  mutation de configuration ; Gemini reste documenté indisponible.
- [x] **SC-005** — le prompt MCP allégé est versionné et mesuré par
  `crates/bridget-daemon/tests/prompt_reduction_test.rs`.
- [x] **SC-006** — la projection ledger partagée est utilisée par CLI et MCP,
  avec golden du renderer CLI et DTO MCP testés dans `cli.rs` et `mcp.rs`.

### Non-régression finale

La suite cumulée a exécuté 241 tests unitaires daemon, 29 tests core, 8 tests
d'intégration et 2 tests idempotence ; 6 tests sont explicitement ignorés.

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

### Phase B — Branchement et matrice comportementale

Le wrapper choisit désormais le bloc réduit uniquement lorsque le registre
active MCP pour la session interactive. Le repli `none`/`unsupported` conserve
le bloc historique octet pour octet. Deux tests unitaires comparent directement
les chaînes produites aux fixtures versionnées : une reformulation non mesurée
ne peut donc pas entrer par un second chemin.

Le banc lance ensuite le vrai chemin `bridget codex` avec le MCP interactif du
registre par défaut. Un exécutable Codex de fixture capture les arguments
effectivement reçus après `wrapper::launch`; le test exige que l'un d'eux soit
exactement la fixture réduite, avec le nom de session substitué. Ce passage
réel a détecté puis fait retirer un point-virgule que la garde d'arguments
refusait avant le spawn.

Cet exécutable reste vivant après la capture : dans la **même racine, la même
instance et la même session**, il lance le serveur `bridget mcp` réellement
injecté, appelle `bridget_who`, reçoit les quatre enveloppes du corpus par le
transport tmux de production puis les clôt avec `bridget reply`. Les oracles
vérifient la demande suivie, le corps riche octet pour octet, l'ordre du tour
lent et de sa file, ainsi qu'une coupure et reconnexion réelles du socket du
même wrapper. Le ledger final contient exactement quatre demandes `answered`.

L'état `busy` et la relance différée ne s'appliquent pas à cette voie
interactive : le wrapper tmux 007 livre les enveloppes sans signal de fin de
tour (`wrapper.rs`, boucle de livraison historique autour des lignes
1030–1070) et se reconnecte donc en état `connected`. La session 010 ne change
pas cette baseline. Le test verrouille explicitement `connected` avant et après
la coupure ainsi que l'absence de `deferred_reminder_level`, sans fabriquer de
signal `TurnState`. Les deux oracles restent portés par le wrapper ACP, qui
réenregistre son tour actif et émet `TurnState` (autour des lignes 2439–2488).
Le scénario interactif et la matrice ACP utilisent des racines, daemons, bases
SQLite et proxies distincts : leurs compteurs de tours et journaux ne peuvent
plus s'intercaler lors d'une exécution chargée.

La matrice `managed_parity_test::matrice_fr008_compare_le_meme_corpus_et_les_frames_attach`
rejoue en complément le quickstart 007 §1 à §4 trois fois dans chacun des deux
modes ACP de lancement. Les oracles restent indépendants de la simple parité :
présence ACP et état `connected`, demande suivie répondue et close, corps riche
exact au `turn_start`, FIFO pendant le tour lent, relance différée persistée,
coupure réelle du socket wrapper puis reconnexion conservant `busy`. La fixture
réduite est donc reliée causalement au premier corpus, tandis que le second
verrouille les garanties ACP impossibles à observer sur le transport tmux.

Validation ciblée : **2/2** tests d'identité de prompt, **2/2** tests statiques
de réduction, corpus dans la session MCP capturée puis matrice comportementale
**1/1** au vert en **36,17 s** (une session interactive capturée, puis trois
campagnes par mode ACP, quatre tours par campagne).
