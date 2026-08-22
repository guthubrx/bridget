# Journal d'Implémentation — Transport ACP

## T701 — Spike adaptateur Codex hors Bridget

- **Statut** : terminé
- **Date** : 2026-08-22
- **Précondition** : `OPENAI_API_KEY` et `CODEX_API_KEY` absents de
  l'environnement.
- **Hygiène** : le script jetable et les trois transcriptions, créées en mode
  `0600`, ont été supprimés après extraction.

Les réponses `initialize` complètes contiennent des métadonnées de modèles et
des instructions volumineuses, générées par le service (plus de 200 ko chacune).
Les lignes JSON-RPC déterminantes sont consignées ici afin d'éviter de
versionner ces données externes non nécessaires à Bridget.

### Tentative 1 — modèle surchargé (échec)

Commande :

```text
npx @zed-industries/codex-acp@0.16.0 -c 'model="gpt-5.6-sol"' -c 'model_reasoning_effort="high"'
```

```text
> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"bridget-spike","version":"0.1.0"}}}
< {"jsonrpc":"2.0","result":{"protocolVersion":1,...},"id":1}
> {"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/007-transport-acp","mcpServers":[]}}
< {"jsonrpc":"2.0","result":{"sessionId":"01a027b0-..."},"id":2}
> {"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"01a027b0-...","prompt":[{"type":"text","text":"Réponds exactement : SPIKE_ACP_OK"}]}}
< {"jsonrpc":"2.0","error":{"code":-32603,"message":"Internal error","data":{"message":"...The 'gpt-5.6-sol' model requires a newer version of Codex..."}},"id":3}
```

### Tentative 2 — configuration temporaire (succès de diagnostic)

Exécution de diagnostic avec une configuration temporaire, non retenue pour le
transport ; elle a uniquement permis d'identifier le modèle compatible du coeur
embarqué.

```text
> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"bridget-spike","version":"0.1.0"}}}
< {"jsonrpc":"2.0","result":{"protocolVersion":1,...},"id":1}
> {"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/007-transport-acp","mcpServers":[]}}
< {"jsonrpc":"2.0","result":{"sessionId":"01a027b4-..."},"id":2}
> {"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"01a027b4-...","prompt":[{"type":"text","text":"Réponds exactement : SPIKE_ACP_OK"}]}}
< {"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"SPIKE_ACP_OK"}}}}
< {"jsonrpc":"2.0","result":{"stopReason":"end_turn"},"id":3}
```

Aucun adaptateur ne demande de clé API pendant ces deux tentatives.

### Tentative 3 — pin gpt-5.5 avec le home Codex courant (succès)

Commande :

```text
npx @zed-industries/codex-acp@0.16.0 -c 'model="gpt-5.5"'
```

```text
> {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"bridget-spike","version":"0.1.0"}}}
< {"jsonrpc":"2.0","result":{"protocolVersion":1,...},"id":1}
> {"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/Users/moi/Nextcloud/10.Scripts/bridget/.worktrees/007-transport-acp","mcpServers":[]}}
< {"jsonrpc":"2.0","result":{"sessionId":"01a027bc-..."},"id":2}
> {"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"01a027bc-...","prompt":[{"type":"text","text":"Réponds exactement : SPIKE_ACP_OK"}]}}
< {"jsonrpc":"2.0","method":"session/update","params":{"update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"SPIKE_ACP_OK"}}}}
< {"jsonrpc":"2.0","result":{"stopReason":"end_turn"},"id":3}
```

La transcription est créée en mode `0600`. Aucun adaptateur ne demande de clé
API pendant cette tentative.

### Self-review Article XIX/XX

- **Nécessité** : valider la compatibilité ACP avant toute intégration évite de
  construire sur un adaptateur inutilisable.
- **Simplicité** : le spike utilise uniquement `node`, `npx` et JSON-RPC ; aucune
  dépendance ni source Bridget n'a été ajoutée.
- **Vérifications** : absence de `OPENAI_API_KEY` et `CODEX_API_KEY`, trois
  cycles `initialize`/`session/new`/`session/prompt`, retour `stopReason` sur
  la tentative gagnante.
- **Non vérifié** : Claude vérifié en T707 (voir section T707) ; Gemini reste
  T708.

## T702 — Décision et dépréciations

- **Statut** : terminé
- **Fichiers** : `docs/decisions/003-transport-acp.md`,
  `docs/DEPRECATIONS.md`

### Self-review Article XIX/XX

- **Nécessité** : la nouvelle frontière de protocole est une décision
  structurante ; les chemins hérités doivent rester traçables.
- **Simplicité** : un ADR et un tableau unique, sans nouvel outil ni format.
- **Vérifications** : l'ADR reprend le cycle R-001 et l'arbitrage R-004 ; le
  registre a les trois colonnes requises.
- **Non vérifié** : aucun chemin de livraison hérité n'est encore remplacé ;
  T703 et T705 alimenteront le registre au moment de leur suppression.

## T703 — Registre d'agents

- **Statut** : terminé
- **Fichiers** : `crates/bridget-daemon/src/registry.rs`, `cli.rs`,
  `wrapper.rs`, `docs/DEPRECATIONS.md`
- **Vérifications** : registre par défaut, surcharge utilisateur, validation,
  type inconnu et tests historiques du workspace.

### Self-review Article XIX/XX

- **Nécessité** : les types et paramètres d'agents doivent être configurables
  sans modifier le binaire.
- **Simplicité** : le registre réutilise `serde_json`; il ne crée ni dépendance
  ni couche de lancement supplémentaire.
- **Non vérifié** : l'exécution ACP réelle relève de T704 et T705.

## T704 — Client JSON-RPC minimal et transport ACP

- **Statut** : terminé
- **Fichiers** : `crates/bridget-transport/src/acp.rs`, `lib.rs`, fixtures
  `tests/fixtures/acp/generic.jsonl` et `tests/fixtures/acp/codex-spike.jsonl`.
- **Vérifications** : un lecteur stdout unique, writer sérialisé, corrélation
  `id → waiter`, worker FIFO et file bornée. Les fixtures générique et Codex
  couvrent la matrice R-004 ; le mapping de prompt applique le contrat de
  livraison sans transformer le corps.
- **Limite assumée** : le branchement du wrapper, le relais des événements et
  le `CancelDelivery` typé sont traités par T705 ; le transport expose déjà la
  purge par id et l'arrêt ACP propre.

## T705 — Branchement équipier et cycle de livraison

- **Statut** : terminé
- **Fichiers** : `crates/bridget-daemon/src/wrapper.rs`, `daemon.rs`,
  `crates/bridget-transport/src/protocol.rs`, `crates/bridget-core/src/message.rs`.
- **Vérifications** : `--equipier` sélectionne ACP et ne transmet aucun prompt
  historique ; les refus asynchrones et annulations sont typés ; le daemon ne
  clôt une demande suivie qu'après livraison réussie et ne contourne DND qu'une
  fois les participants de la demande validés sans mutation.

## T706 — Journal de session JSONL

- **Choix de file** : le canal reste borné pour borner la mémoire sous flux
  soutenu ; saturation et erreur d'écriture produisent `JournalFailed`, ce qui
  arrête immédiatement le transport puis draine ses livraisons terminales.

## T707 — Équipier Claude

- **Statut** : terminé
- **Précondition** : `ANTHROPIC_API_KEY` absent de l'environnement.
- **Commande** : `npx @zed-industries/claude-code-acp@0.16.2` ; aucune
  surcharge de modèle n'a été nécessaire.
- **Transcription extraite** :

```text
> initialize (protocolVersion: 1)
< agentInfo: @zed-industries/claude-code-acp 0.16.2 ; authMethods: claude-login
> session/new
< sessionId: claude-spike-session
> session/prompt: « Réponds exactement : CLAUDE_ACP_OK »
< session/update available_commands_update
< session/update agent_message_chunk: "" ; "C" ; "LAUDE_ACP_OK"
< result stopReason: "end_turn"
```

- **Quickstart §1–§3** : équipier `claude-t707` lancé par le registre par
  défaut, demande suivie livrée puis réponse et clôture reçues ; le corps avec
  apostrophe, guillemets, `$VAR` et backticks est revenu entre balises
  `<echo>` sans transformation.
- **Hygiène** : le script et la transcription bruts, créés en `0600`, sont
  supprimés après extraction ; la fixture versionnée est minimisée et ne porte
  pas la liste locale des commandes annoncées par l'adaptateur.

## T708 — Ouverture Gemini (bloquée)

- **Précondition** : `GEMINI_API_KEY` et `GOOGLE_API_KEY` absents.
- **Tentatives** : `gemini 0.46.0 --acp`, puis
  `npx @google/gemini-cli@0.56.0 --acp` via une fixture de registre temporaire
  Codex+Claude enrichie dynamiquement de l'entrée Gemini.
- **Refus exact au `session/new`** :

```text
This client is no longer supported for Gemini Code Assist for individuals.
To continue using Gemini, please migrate to the Antigravity suite of products:
https://antigravity.google
```

- **Issue** : aucun équipier enregistré, aucun prompt, aucune réponse ni
  `stopReason` ; les fixtures temporaires de tentative sont supprimées. La
  tâche reste décochée en attente de la révision de la voie de support Gemini.

## T709 — Relances informées

- **Choix de persistance** : une relance différée est enregistrée comme
  événement typé `reminder_deferred` dans le store des demandes. Elle relève
  du cycle de vie d'une demande et est exposée par `ListRequests` et
  `bridget requests` (niveau et horodatage du dernier report), sans détourner
  le journal JSONL de session ACP.

## T708 — Ouverture déclarative validée

- **Fixture** : `crates/bridget-daemon/tests/fixtures/registry/codex-claude.json`
  ne contient initialement que Codex et Claude. Le test ajoute ensuite le type
  `stdio-ouvert`, absent du code de production, puis charge le registre écrit
  dans un fichier temporaire.
- **Échange** : l'adaptateur stdio reçoit `initialize`, `session/new` et
  `session/prompt`, retourne un chunk `fixture-response` et `end_turn`. Le test
  traverse le daemon : livraison suivie, tour ACP, réponse avec `in_reply_to`,
  puis demande en état `answered`.
- **Contrôle mécanique SC-003** : avant le commit, la commande
  `git diff --name-only d68ed09 -- crates | rg -v '^crates/[^/]+/tests/'`
  ne produit aucune sortie. La modification Rust est donc limitée au test
  d'intégration ; aucun fichier de code de production n'est modifié.
