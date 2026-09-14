# Recherche 098 — faits et décisions

## Environnement constaté le 2026-09-14

t3code à jour (commit `e3792a53f`, 1 198 commits tirés le matin même), dépôt local `/Users/moi/11.Repositories/t3code`. Bridget `main` après 097. Codex 0.154.0, Claude Code 2.1.270.

## Faits t3code (lecture du code, fichier:ligne dans le dépôt t3code)

- **Découverte** : le serveur écrit `~/.t3/userdata/server-runtime.json` (`version, pid, host, port, origin`) ; port par défaut 3773 (`apps/server/src/config.ts:21`, `server.ts:619-634`). `GET /.well-known/t3/environment` décrit l'authentification.
- **Authentification headless** : `t3 auth session issue --ttl <durée> --subject bridget --token-only` écrit une session dans la base SQLite d'auth, serveur démarré ou non (`apps/server/src/cli/auth.ts:162-194`) ; durée par défaut 30 jours, révocation par `t3 auth session revoke`. Portées : `orchestration:read` pour lire, `orchestration:operate` pour `dispatchCommand`, donc pour démarrer un tour (`apps/server/src/auth/RpcAuthorization.ts:24`). `Authorization: Bearer` fonctionne sur HTTP et sur l'upgrade WebSocket. `T3CODE_DEV_AUTH_TOKEN` est réservé au mode dev : écarté.
- **Commandes sans framing RPC** : `POST /api/orchestration/dispatch` accepte un `ClientOrchestrationCommand` JSON, dont `{type:"thread.turn.start", commandId, threadId, message:{messageId, role:"user", text, attachments:[]}, runtimeMode, interactionMode, createdAt}` (`packages/contracts/src/orchestration.ts:1213-1233`, route `environmentHttp.ts:530-538`), réponse `DispatchResult{sequence}`. Lecture : `GET /api/orchestration/snapshot` (projets, fils, sessions avec `providerName`, `providerInstanceId`, `status`, `activeTurnId`, `worktreePath`) et `GET /api/orchestration/threads/:threadId`.
- **Événements** : abonnement par fil en RPC WebSocket (`orchestration.subscribeThread`), framing Effect RPC non vérifié octet à octet ; événements `thread.created`, `thread.archived`, `thread.settled`, `thread.message-sent` (rôle, texte, `turnId`), `thread.turn-start-requested`. `thread.message.assistant.*` sont internes, pas exposés.
- **Tour déjà en cours** : `thread.turn.start` n'est pas refusé ; hors compaction, le message est transmis immédiatement au fournisseur (injection pendant le tour, `ProviderCommandReactor.ts:1567-1573`) ; pendant une compaction, il est mis en file. `commandId` est l'identifiant de commande côté t3code.
- **Réglages** (lus pour information, jamais modifiés en v1) : fichier `~/.t3/userdata/settings.json`, écriture atomique par le serveur, rechargement à chaud par surveillance du dossier (`serverSettings.ts:890-931`) ; RPC `serverUpdateSettings` (patch fusionné) sans équivalent HTTP. Champs : `providers.claudeAgent.launchArgs` (attention : clé `claudeAgent`, pas `claude`), `providers.codex.launchArgs`, `binaryPath`, `homePath`. Variables d'environnement par instance dans `providerInstances`, remplacées en bloc et déportées vers un magasin de secrets : à éviter.
- **Claude** : `launchArgs` devient `extraArgs` du SDK, `--mcp-config <fichier>` y est transmis tel quel (`ClaudeAdapter.ts:4650-4654, 4751`) ; le serveur MCP `t3-code` est ajouté séparément par programme. `homePath` → `CLAUDE_CONFIG_DIR`. **Codex** : `launchArgs` tokenisés puis placés devant les arguments t3code dans `codex app-server` ; `-c mcp_servers.bridget.<clé>=…` s'ajoute sans collision (`CodexAdapter.ts:2281-2286`). Piège : la variable serveur `T3CODE_CODEX_LAUNCH_ARGS` écrase silencieusement `launchArgs`.
- **Identifiant de session fournisseur** : non exposé dans les contrats publics (`ProviderRuntimeBinding` interne).
- **Serveur MCP `t3-code`** : outils d'appareil, d'aperçu et de pull requests, identifiants liés à un fil, non distribuables : sans utilité pour Bridget.
- **Versionnage** : pas de version globale du contrat orchestration ; évolution additive champ par champ.

## Faits du poste (2026-09-14)

- Application installée : « T3 Code (Alpha) » 0.0.40, Electron (`app.asar`), sans binaire CLI `t3` dans le PATH ; serveur non lancé lors de la recherche ; `~/.t3/userdata/settings.json` presque vide. Le CLI `t3` (npm) est donc un prérequis pour `bridget t3 install` ; sans lui, refus nommé. Le contrat lu dans le dépôt peut être en avance sur l'application : la sonde T001 vérifie les routes sur l'application réelle.
- Aucun client HTTP dans le workspace Bridget ; `uuid` sans `v5`.

## Faits Bridget (sondes locales du 2026-09-14)

- Claude Code transmet `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_ENTRYPOINT` et `CLAUDECODE` à ses serveurs MCP (observé sur quatre processus `bridget mcp` fils de `claude`). Codex app-server ne transmet aucune variable de fil à ses serveurs MCP (observé sur trois processus fils de `codex`).
- Le serveur MCP Bridget résout son identité par `BRIDGET_AGENT_ID_FILE` + `BRIDGET_AGENT_INSTANCE_ID`, ou par un marqueur `agent-pids/<pid>` d'un ancêtre (`crates/bridget-daemon/src/mcp_identity.rs:116-131, 202-270`). Sans wrapper, `IdentityNotFound`.
- La remise idempotente interactive (`deliver_idempotent_to_interactive`, tracker de reçus par instance) et la commande d'administration embarquée de la 096 (`crates/bridget-daemon/src/federate.rs`, 96 lignes) sont réutilisables telles quelles.

## Sonde T001 sur l'application réelle (2026-09-14 09:28–09:31, T3 Code 0.0.40, CLI `t3` 0.0.40 installé par npm)

- `server-runtime.json` : `{"version":1,"pid","host":"127.0.0.1","port":3773,"origin","startedAt"}` ; `GET /.well-known/t3/environment` : `serverVersion 0.0.40`.
- `t3 auth session issue --subject bridget --label bridget-sonde --ttl 2h --json` → `{sessionId, token, method:"bearer-access-token", scopes, subject, client:{label, deviceType:"bot"}, expiresAt}` ; `t3 auth session list --json` liste `client.label` ; `revoke <sessionId>` fonctionne (0 session restante).
- `GET /api/orchestration/snapshot` : `{snapshotSequence, projects[{id, workspaceRoot}], threads[], updatedAt}` ; chaque fil porte `id, projectId, title, worktreePath, archivedAt, deletedAt, updatedAt, latestTurn{turnId,state,assistantMessageId,…}, session{providerName, providerInstanceId, status, activeTurnId}, modelSelection{instanceId, model, options}, runtimeMode, interactionMode` ; **pas de `threadSequence` ni de messages inline** en 0.0.40 : la détection de changement s'appuie sur `updatedAt` et `latestTurn`.
- `GET /api/orchestration/threads/:id?turnLimit=N` : `{snapshotSequence, thread{messages[{id, role, text, streaming, turnId, attachments, createdAt, updatedAt}], latestTurn}, page}` ; les messages utilisateur ont `turnId = null`, les messages assistant portent le `turnId` de leur tour.
- `POST /api/orchestration/dispatch` : `thread.create` (payload du tag v0.0.40 : `commandId, threadId, projectId, title, modelSelection{instanceId:"claudeAgent", model}, runtimeMode, interactionMode, branch, worktreePath, createdAt`) → 200 `{sequence}` ; `thread.turn.start` → 200 ; **rejeu du même `commandId` → même `sequence`, aucun second tour** ; `thread.archive` → 200 et le fil **reste dans le snapshot** avec `archivedAt` renseigné.
- Tour réel Claude (haiku) : 4 s, `session.status` passe par `running` avec `activeTurnId`, puis `ready` ; `latestTurn.state = completed` avec `assistantMessageId`.
- **Dispatch pendant un tour actif** : accepté (200), le message est mis en file et traité comme le tour suivant, jamais en parallèle ; l'ordre des messages est alors `user A, user B, assistant A (tour A), assistant B (tour B)` : l'adjacence « premier assistant après mon message » est fausse ; l'appariement exact est **par ordre : le k-ième message utilisateur du fil reçoit le k-ième tour assistant** (FIFO observé), chaque tour étant identifié par son `turnId` et clos par `latestTurn.state = completed` ou `streaming = false`.
- Verdict T001 : contrat conforme, plan ajusté sur trois points (corrélation par rang, archivage par `archivedAt`, changement par `updatedAt`). Fil de sonde archivé, session de sonde révoquée.

## Sonde T013 sur l'application réelle (recette, 0.0.41-nightly, 2026-09-14)

Trois écarts avec la sonde T001, tous corrigés dans le code et couverts par un test :

1. **`server-runtime.json` sans `host`.** L'application publie `host`; `t3 --mode web --no-browser` ne publie que `origin`. Le lecteur accepte désormais les deux et refuse toute écoute non locale dans les deux cas (`spec098_runtime_du_mode_web_sans_host_est_lu_par_son_origine`).
2. **Un fil neuf n'a pas de session fournisseur.** `session` est `null` tant qu'aucun tour n'a eu lieu ; exiger une session rendait un fil neuf injoignable pour son premier message. Le fournisseur vient de `modelSelection.instanceId` (`claudeAgent`, `codex`, `antigravity`), la session primant quand elle existe. Une session peut aussi être `stopped` sans que le fil cesse d'être joignable.
3. **`thread.turn.start` exige `runtimeMode` et `interactionMode`.** Le schéma interne leur donne une valeur par défaut, mais le schéma *client* de la route HTTP (`ClientThreadTurnStartCommand`, packages/contracts/src/orchestration.ts:1235-1253) ne l'a pas : les omettre donne un 400 sans corps. Le pont répète les valeurs déjà enregistrées sur le fil : imposer `full-access` à un fil réglé `approval-required` serait une élévation de privilège silencieuse.

Autres faits confirmés en conditions réelles : `thread.create` refuse `runtimeMode: "local"` et `interactionMode: "chat"` (littéraux `approval-required|auto-accept-edits|auto|full-access` et `default|plan`) ; les quatre fils du poste sont apparus dans `bridget who` sous leur titre avec le bon type ; un message remis a produit un tour puis une réponse liée qui a clos la demande.

## Décisions (révision 2 après contre-revue adverse)

1. **HTTP d'abord, RPC WebSocket jamais en v1.** Remise par `POST /api/orchestration/dispatch`, état des fils par `GET /api/orchestration/snapshot`, détail par `GET …/threads/:id`. Le framing Effect RPC reste une inconnue : on ne s'y expose pas.
2. **Jeton par la commande officielle**, sortie structurée (jeton, identifiant de session, expiration), conservé 0600, renouvelé au plus une fois par incident 401, révoqué par identifiant au retrait. Ses portées sont administratives (`AuthAdministrativeScopes`, `apps/server/src/cli/auth.ts:162-193`) : limite acceptée et documentée, faute d'émission à portées réduites prouvée.
3. **Aucune écriture dans t3code** : v1 sans émission depuis l'agent, donc sans injection MCP ni modification de `settings.json`. L'objection « deux écrivains sur settings.json » disparaît avec le besoin.
4. **Une identité Bridget par fil**, possédée par le pont, présence `t3code | cli`, UUID v5 stable ; retrait à l'archivage.
5. **Remise** : attente bornée de l'absence de tour actif, puis `dispatch` avec `commandId` = identifiant de remise (t3code déduplique par `commandId`, `OrchestrationEngine.ts:144-171`) ; accusé après `DispatchResult` ; course résiduelle documentée et testée.
6. **Réponse liée renvoyée par le pont** : la fin du tour de remise renvoie le texte assistant à l'émetteur avec `in_reply_to`.
7. **Journal par différence de snapshots avec curseur durable**, sans rejeu d'historique ni doublon ; `thread.message-sent` n'est pas accessible en HTTP.
8. **Validation route par route** ; `server-runtime.json.version` n'est que la version du fichier.
9. **Contrat t3code en un seul module.** Client HTTP : `minreq` sans TLS ; UUID v5 via la dépendance existante ; prérequis CLI `t3`.
10. **Émission depuis l'agent t3code** : reportée à une v2 conditionnée à une attestation publique fil ↔ processus MCP ; aucune identité déduite d'une variable d'environnement.

## Alternatives écartées

- Abonnement RPC WebSocket : framing non vérifié, dépendance à `effect/unstable/rpc` ; le sondage HTTP local coûte moins et se teste avec un faux serveur.
- Modifier t3code (fork) : inutile pour la remise et l'annuaire ; seule l'identité fine par fil pour Codex l'exigerait, hors périmètre.
- Variables d'environnement par instance t3code : remplacées en bloc et déportées vers un magasin de secrets, trop intrusif pour un réglage réversible.
- Jeton dev `T3CODE_DEV_AUTH_TOKEN` : mode dev uniquement.

## Risques nommés (révision 2)

- Corrélation de la réponse : exacte par `messageId` du pont dans l'ordre du fil ; un message humain intercalé rend la réponse non prouvée → demande laissée ouverte, jamais une réponse inventée.
- Course résiduelle entre « pas de tour actif » et `dispatch` : t3code injecte alors le message pendant le tour ; détectée après coup par la projection, journalisée, testée.
- Portées administratives du jeton : limite acceptée, révocable, libellé unique pour la retrouver.
- Évolution additive de t3code : validation route par route ; un champ manquant produit un refus nommé.
- Origine du serveur : seule la boucle locale est acceptée ; le jeton ne circule jamais hors de 127.0.0.1.

## Sources
Rapport de recherche du sous-agent sur t3code (81 lectures, fichiers cités ci-dessus) ; sondes locales `ps -Eww` sur les serveurs MCP Bridget vivants ; code Bridget cité.
