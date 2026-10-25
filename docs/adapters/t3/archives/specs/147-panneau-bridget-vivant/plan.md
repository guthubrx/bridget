# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/plan.md](../../../../../../specs/147-panneau-bridget-vivant/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-08 ; extensions US6–US9 approuvées le 2026-10-09. Développement : Implemented, Validé,54/54 tâches ; deux passes Converge couvrent31FR/20SC/9US. Sources committées/fusionnées/poussées. Livraison EN COURS : Bridget installé/actif, T3 package signé prêt et activation pending. Sans PR T3 ; dirty142/145/146 conservés. Reçus runtime et cleanup détaillés dans validation/results.json.

<a id="extrait-2"></a>

## Extrait 2

Autorisation utilisateur : adapter Bridget et ajouter à147, pas créer148.
Travail dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/session-147-t3-v2.
Les US1–US9 restent livrées ; les preuves ci-dessous ne valent pas preuve US10.

<a id="extrait-3"></a>

## Extrait 3

Contrat vérifié sur T3 upstream563645cf6e10, intégré localement1535fc0844 :
shell HTTP avec x-t3-orchestration-protocol:2 ; détail projection V2 ; mutations
par RPC WebSocket orchestration.dispatchCommand ; base userdata/statev2.sqlite.

<a id="extrait-4"></a>

## Extrait 4

Réutiliser Client, Snapshot/ThreadSummary/ThreadDetail, la file et la corrélation.
Lire orchestrationProtocolVersion dans /.well-known/t3/environment au premier
snapshot. L'absence de ce champ annonce V1 ; une autre version est refusée.
Confirmer par la lecture authentifiée du snapshot correspondant, puis mémoriser
seulement dans ce Client
(réinitialisé lors du changement de runtime). Aucun essai V1 après envoi V2.
Le chemin V1 reste inchangé ; un refus401/403 ou une panne ne rétrograde pas.
Le mapping V2 reste dans t3code_contract_v2.rs : runs/messages natifs vers les
vues internes existantes, statuts validés, pas de résumé ni d'appel de modèle.
Les messages de sous-agents ne doivent pas devenir la réponse du tour racine.
Les observations d'outils utilisent uniquement les faits V2 confirmés.
Ne pas sonder une route absente pour choisir la version : le vrai serveur peut
renvoyer index.html avec HTTP200. Le test natif a révélé ce cas.

<a id="extrait-5"></a>

## Extrait 5

Tests : parsing fermé V2 et V1, fin/cancel/queue/steer, RPC idempotent et ambigu,
identité de version et sessions étrangères, puis vrai serveur T3 privé sans modèle.
Complexité O(fils + messages + items), lookup indexé ; I/O bornées, pas une requête
par message. Aucun changement du panneau ou du protocole humain147.
Le helper de prérequis SpecKit est absent : vérification manuelle des artefacts.
L'outil Sequential Thinking n'est pas disponible ; choix et invariants explicités ici.

<a id="extrait-6"></a>

## Extrait 6

La fusion T3 est construite mais n'est pas activée tant que ce complément n'est
pas validé. Aucun push, nouveau PR, restart ou modification de base réelle.

<a id="extrait-7"></a>

## Extrait 7

Ajouter un signal de changement humain, sans contenu, au daemon Bridget. Le faire traverser la CLI locale, puis le flux RPC T3 existant. Le panneau utilise ce signal pour relire les vues146 autorisées. Il ne reçoit pas les corps dans l'abonnement.

<a id="extrait-8"></a>

## Extrait 8

Mémoriser seulement le UUID du fil choisi dans le store natif du panneau droit. La clé contient l'environnement, le projet et la conversation T3. Au retour sur une conversation, restaurer ce UUID puis revalider son accès, même s'il n'apparaît pas dans la première page.

<a id="extrait-9"></a>

## Extrait 9

- Bridget et documents : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant.
- T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant.
- Branche dans chaque dépôt : `session-147-panneau-bridget-vivant`.
- Socle : sources146 fusionnées, sans importer les changements non committés142/145.
- Bridget : Rust, protocole IPC local typé, SQLite et verrou de daemon existants.
- T3 : React, TypeScript, Zustand, Effect, flux RPC et Vitest existants.
- Dépendances, tables, migrations ou journaux durables nouveaux : aucun.
- Périmètre initial : consultation humaine et références de navigation, sans changement des missions ni relances. Extension approuvée : UUID des commandes CLI et de l'outil MCP de fils partagés et montage MCP Claude/GLM attesté. Les tests d'écriture interagents sont des fixtures privées distinctes du canal humain FR147-15.

<a id="extrait-10"></a>

## Extrait 10

Extraire la garde d'autorité de `handle_human_thread_view`, actuellement /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11790. Vue et suivi utilisent cette même garde. Elle conserve Client négocié, liaison T3 primaire unique, identité vivante, instance, hôte et projet attestés. La CLI ne fait ni Register, ni autostart, ni découverte d'identité agent.

<a id="extrait-11"></a>

## Extrait 11

Étendre le client de lecture humain de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/communication/client.rs:147. La commande `bridget thread watch --t3-thread UUID --project-root ROOT --json` fournit un flux JSONL. Le handshake a un délai borné ; le flux accepté n'a pas le timeout unary de dix secondes. L'annulation ou la fermeture du pipe ferme la connexion locale.

<a id="extrait-12"></a>

## Extrait 12

Étendre le service existant /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:74 avec `watch`. Il résout conversation et projet par `ProjectionSnapshotQuery`, puis transmet `workspaceRoot`. Le navigateur ne fournit aucun chemin racine, agent, instance ou droit.

<a id="extrait-13"></a>

## Extrait 13

Étendre /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts avec entrée et événement fermés. Ajouter `WS_METHODS.bridgetWatch = "bridget.watch"` au contrat RPC dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/rpc.ts, à côté de `bridget.read` (:998). Conserver `AuthOrchestrationReadScope` dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts.

<a id="extrait-14"></a>

## Extrait 14

Utiliser l'observation de flux existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts:709, puis la factory de subscription native dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts. L'adaptation147 de /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts réutilise `createEnvironmentSubscriptionAtomFamily` pour conserver un `BridgetWatchState` volatil : dernier événement, readyGeneration et subscriptionId local. Fixer `idleTtlMs: 0` : le défaut de cinq minutes laisserait le suivi vivant après fermeture.

<a id="extrait-15"></a>

## Extrait 15

Le supervisor existant porte les reprises de transport WebSocket T3. Si ce transport reste sain alors que le CLI/daemon de suivi tombe, réutiliser Schedule Effect pour reprendre le seul stream CLI : au plus trois tentatives supplémentaires après l'initiale, seulement pour unavailable/command_failed/timeout. Chaque tentative revalide autorité/handshake ; ready neuf rattrape par lectures146. Durant l'attente, masquer corps et garder UUID. Au plafond, arrêt explicite avec refresh manuel ou nouvelle visite. Invalid_output/version/projet/binding/demande invalide ne sont jamais retryables. Aucun timer de lecture de corps ni retry infini ajouté.

<a id="extrait-16"></a>

## Extrait 16

Étendre /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts par une table distincte `bridgetSelectionByContextKey`. La clé est `JSON.stringify([environmentId, projectId, threadId])`. La valeur est seulement un UUID de fil partagé validé. Aucun corps, titre, membre, credential ou message n'est persisté.

<a id="extrait-17"></a>

## Extrait 17

Ajout approuvé le 2026-10-09. La lane Web étend seulement la ligne existante de /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx. Elle affiche Auteur → Noms pour notify targets/all, et Auteur · Sans sollicitation pour none. La date reste dans cette ligne ; aucune hauteur dédiée supplémentaire.

<a id="extrait-18"></a>

## Extrait 18

Le tableau exhaustif figure dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/reuse-audit.md. Les principales réutilisations sont la garde humaine145, les lectures et curseurs146, les mutations transactionnelles, l'annuaire, la résolution de contexte T3, les flux RPC, le superviseur, les subscriptions client-runtime et le store du panneau droit.

<a id="extrait-19"></a>

## Extrait 19

Les standards génériques Next.js/Axios, les dossiers Cartae et Pytest ne décrivent pas T3/Bridget. Réutiliser les piles React/Effect/Vitest et Rust existantes est une divergence justifiée, pas une omission de tests ou de typage. Les scénarios restent formulés en langage métier, indépendamment de leur runner.

<a id="extrait-20"></a>

## Extrait 20

1. Tests Rust protocol/daemon/CLI147 RED sur le socle : capacité inconnue, aucun signal pour Create/Post réel, garde de liaison, inscription initiale, no-replay, annulation et sortie bornée.
2. Tests Rust GREEN : mutation committée une fois, replay/ACK/read/refus silencieux, rename pertinent, files saturées → resync, ready seq0 préservé en premier même après une rafale supérieure à16 avant écriture, client lent sans blocage, perte d'autorité, daemon redémarré, aucune action agent.
3. Tests service/RPC/runtime T3 RED puis GREEN : contexte serveur, payload fermé, scope lecture, JSONL fractionné/invalide/trop grand, handshake, EOF, cancellation, idleTtl0 et reprise. Distinguer WebSocket repris par supervisor et CLI rompu sous WebSocket sain, avec initiale + trois reprises maximum uniquement techniques. Tester ready+changed avant premier rendu : état sticky garde readyGeneration, une subscription par visitId, aucun champ wire ajouté. Un refus métier fermé reste arrêté, sans boucle de retries, jusqu'à refresh manuel ou nouvelle visite.
4. Tests store/panneau RED puis GREEN : A → B → A, projet/environnement, choix hors première page, ancien snapshot, code refus ≠ panne, late response, coalescence, 60s sans relecture, 100 changements et pages déjà chargées. Cas obligatoires : remplacement d'une consigne en page2 pendant le chargement de page3 ; nouveau snapshot commun S, publication atomique, réponse ancienne ignorée et remplacement visible en page2. Plus de50 nouveautés : intervalle entre tête et ancre ancienne complètement rattrapé sous S, pas de trou ni doublon.
5. Recette intégrée isolée : vrai daemon et CLI147 avec données synthétiques puis surface T3 de test. Observer le délai de deux secondes, focus, texte sélectionné, dépliage, copie, supersession, page masquée et fermeture. Aucun agent ou base T3 actifs.
6. Contrôles adaptés : format Rust, lint/clippy, tests ciblés puis suites touchées ; typecheck/lint/tests/build T3 selon scripts existants. Journaliser sorties, nombres réels, RED/GREEN et limites.
7. Contre-revue, audit, Analyze et convergence documentaire. Aucun fournisseur adverse joignable dans le même projet à ce stade ; ne pas contacter un autre projet pour simuler cette preuve. Une revue locale ne sera pas qualifiée d'inter-fournisseurs.

<a id="extrait-21"></a>

## Extrait 21

Gate du principal reçu le2026-10-09 après les amendments de revue. La configuration effective du cas regional-wrkr-1/ClaudeAgent/claude_glm manque Bridget ; T3 injecte t3-code seulement. Aucune exclusion volontaire GLM.

<a id="extrait-22"></a>

## Extrait 22

Préparer le montage avec le SDK natif : mcpServerStatus puis setMcpServers, après création de la query et avant sessions.set/ready ou tout premier prompt. Même garantie pour new et resume ; aucun restart d'une query active. Conserver options.mcpServers.t3-code dans le payload, car setMcpServers remplace les serveurs dynamiques.

<a id="extrait-23"></a>

## Extrait 23

Fichiers production retenus :
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts : fonction Effect de préparation et politique de compatibilité, pas une classe enveloppe neutre.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSettingsPaths.ts : extraire calcul actuel des sources/settings et findRepositoryRoot depuis ClaudeSkills.ts ; consommateurs réels Skills et MCP. Exception architecture<=3 usages autorisée : même calcul de scope critique pour la sécurité, sémantiques MCP stricte et Skills fail-open préservées.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetExecutable.ts : extraire résolution Reader existante, priorité T3CODE_BRIDGET_EXECUTABLE puis HOME/.local/bin/bridget puis PATH, rejet cmd/bat ; trois usages read/watch/MCP, ordre inchangé.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts : interface ClaudeQueryRuntime expose les deux méthodes SDK ; préparation avant premier prompt, mcpServers t3-code conservé.
- Consommateurs existants : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSkills.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts.

<a id="extrait-24"></a>

## Extrait 24

Garde conservative MCP validée par le principal : si cwd est un enfant du dépôt, inspecter aussi les métadonnées root/.claude/settings.json et projects[root] de .claude.json pour respecter un refus d'auto-montage T3. Vérifier metadata avant status SDK ; flags/identity refusent sans I/O. Le calcul partagé et comportement Skills restent inchangés. Ne pas prétendre que la CLI charge ces plain settings : le getter MCP privé établit seulement root/.mcp.json scope project ; RC0 sous disabled ne prouve pas que le serveur soit activé.

<a id="extrait-25"></a>

## Extrait 25

Initialisation rejetée/pending/deadline : fermer query candidate et annuler son consumer avant ready/prompt. Une promesse SDK résolue tardivement ne monte pas le serveur après timeout. Candidate sous propriétaire/finalizer local unique jusqu'à sessions.set : si prepare réussit mais close ancienne query échoue, fermer candidate/consumer sans ready/prompt ni orphelin. Amendement T050 validé dans FR21/22 existants, pas nouveau framework/helper. Diagnostics codés sans secrets. Aucun endpoint, table, dépendance, profil global ou nouvelle autorité t3-code.

<a id="extrait-26"></a>

## Extrait 26

Dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx, surface de liste bg-sidebar pour contraste clair et ligne choisie row-active/foreground. Hover seulement sur les autres lignes, focus-visible anneau inset, aria-current/pressed sur contrôle du choix. Icône et palette natives, pas rebrand.
