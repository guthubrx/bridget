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
  lance le **wrapper réel** avec registre, socket et home injectés : livraison
  suivie, tour ACP, réponse avec `in_reply_to`, puis demande en état
  `answered`.
- **Contrôle mécanique SC-003** : avant le commit, la commande
  `git diff --name-only d9e725f -- crates | rg -v '^crates/[^/]+/tests/'`
  ne produit aucune sortie. La modification Rust est donc limitée au test
  d'intégration ; aucun fichier de code de production n'est modifié.

## T710 — Garde de facturation

- Le lancement ACP refuse la première variable présente de `forbidden_env` et
  nomme la variable dans le message français, avec le contournement explicite
  `BRIDGET_ALLOW_API_KEY=1`.
- Les tests couvrent la priorité de `forbidden_env` (première clé si deux sont
  présentes, seconde seule), et le parsing strict : seul exactement `"1"`
  active le contournement.

## T711 — Checklist de finition

- [x] **SC-001** — échange Codex suivi sans prompt Bridget : transport ACP et
  clôture couverts par `acp::tests::false_adapter_exercises_stdio_reader_writer_and_prompt`.
- [x] **SC-002** — corps spéciaux conservé :
  `acp::tests::prompt_preserves_the_body_byte_for_byte`.
- [x] **SC-003** — type inconnu configuré et cycle complet via wrapper réel :
  `integration_test::test_unknown_registry_type_completes_a_tracked_exchange`
  (commit `fdf5835`).
- [x] **SC-004** — boucle de relance sous tour `busy` :
  `daemon::presence_tests::boucle_busy_persiste_les_reports_et_expire_une_seule_fois`
  vérifie zéro action de livraison douce/ferme à T/3 et 2T/3, les événements
  `reminder_deferred` persistés aux paliers 1 puis 2, puis un unique timeout à
  T malgré l'état `busy`.
- [x] **SC-005** — clés API refusées et contournement strict :
  `wrapper::reconnect_tests::api_key_forbidden_*` et
  `seul_le_contournement_egal_a_un_est_accepte`.
- [x] **SC-006** — validation tmux réelle : `t711-tmux` lancé sans
  `--equipier` apparaît avec le transport `tmux`; le pane a capturé le bloc
  historique « Règles ABSOLUES » puis le message
  `💬 cli-send-51888 → t711-tmux (reply=no, id=1e464491)`.
- [x] **SC-007** — `docs/DEPRECATIONS.md` relu : tous les chemins hérités
  retirés par la migration ACP sont étiquetés.

FR-014 reste hors checklist : T712 exige une validation fédérée SSH distincte.

**Annuaire ACP** : `t711-acp` lancé avec `bridget codex --name t711-acp
--equipier` est apparu dans `bridget who` avec `TRANSPORT=acp`. Les variables
`OPENAI_API_KEY` et `CODEX_API_KEY` étaient absentes avant le lancement ; le
wrapper a été terminé extérieurement, produisant un EOF sans `Unregister` et
l'état `unreachable`. Ce contrôle ne teste **pas** l'arrêt propre vers
`stopped`.

## T712 — Gate fédération SSH

- **Date** : 2026-08-22.
- **Isolement** : daemon de test local lancé avec `HOME=/tmp/bg-gate` et socket
  `/tmp/bg-gate/.cache/bridget/bridget.sock`; aucun daemon ou agent de
  production n'a été arrêté. Le client distant a été déployé depuis le commit
  `c0964e7` (client-only) sur `cartae.app:2222`.
- **Tunnel** : commande manuelle, équivalente à `federate-ssh.sh` mais avec le
  socket de gate isolé :

```text
ssh -N -p 2222 -o BatchMode=yes -o ControlMaster=no -o ControlPath=none -o ExitOnForwardFailure=yes -o ServerAliveInterval=20 -o ServerAliveCountMax=3 -R /home/moi/.cache/bridget/bridget.sock:/tmp/bg-gate/.cache/bridget/bridget.sock moi@cartae.app
```

- **Échange observé** : `bridget who` exécuté à distance a affiché
  `t712-acp ... transport acp ... connected`. Le `bridget send --reply` distant
  (id `ffdb7b5fcb844`) a produit dans le journal local la réponse Codex sur
  `router.rs` puis `turn_end` avec `stop_reason=end_turn` (seq 90). Les
  variables `OPENAI_API_KEY` et `CODEX_API_KEY` étaient absentes avant le
  lancement du wrapper de test.
- **Résultat strict** : le client éphémère `bridget send --reply` distant n'a
  créé aucune entrée visible par `bridget ledger` ni `bridget requests` (sortie
  `Ledger vide.` / `Aucune demande suivie.`). La clôture au ledger exigée par
  FR-014 n'est donc pas prouvée : T712 reste décochée et la session 007 reste
  `In Progress` sur ce gate.
- **Nettoyage** : wrapper, tunnel SSH, daemon de test et socket distant de gate
  ont été arrêtés/supprimés; la configuration persistante `federate-ssh` n'a
  pas été retirée faute d'autorisation explicite.

### Diagnostic du ledger distant

Le diagnostic a été rejoué sur le même socket de gate. `bridget who` local et
distant ont tous deux répondu `Aucun agent connecté.`, ce qui confirme que le
client distant emploie bien le socket SSH tunnelé. En revanche, pour le même
daemon, le client local a affiché le message `cli-send-2780730 → t712-acp`
dans son ledger alors que le client distant a affiché `Ledger vide.`. Le
ledger est donc lu depuis une base locale distante et non projeté par le
daemon fédéré : c'est un défaut de câblage du client fédéré, non une absence de
réponse ACP.
