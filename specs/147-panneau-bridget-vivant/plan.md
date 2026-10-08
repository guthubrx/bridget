# Plan technique — SPEC147

Date : 2026-10-08 ; extensions US6–US9 approuvées le 2026-10-09. Développement : Implemented, Validé en environnement isolé,54/54 tâches ; deux passes Converge couvrent31FR/20SC/9US. Tests, builds, audits frais et recette native validés dans leurs périmètres distincts. Livraison : EN COURS, autorisation explicite de commit/fusion/push/installation et redémarrage T3 reçue ; installation/activation pas encore attestées. Sans PR T3 ni cleanup142/145.

## Résumé

Ajouter un signal de changement humain, sans contenu, au daemon Bridget. Le faire traverser la CLI locale, puis le flux RPC T3 existant. Le panneau utilise ce signal pour relire les vues146 autorisées. Il ne reçoit pas les corps dans l'abonnement.

Mémoriser seulement le UUID du fil choisi dans le store natif du panneau droit. La clé contient l'environnement, le projet et la conversation T3. Au retour sur une conversation, restaurer ce UUID puis revalider son accès, même s'il n'apparaît pas dans la première page.

## Contexte technique et espaces isolés

- Bridget et documents : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant.
- T3 : /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant.
- Branche dans chaque dépôt : `session-147-panneau-bridget-vivant`.
- Socle : sources146 fusionnées, sans importer les changements non committés142/145.
- Bridget : Rust, protocole IPC local typé, SQLite et verrou de daemon existants.
- T3 : React, TypeScript, Zustand, Effect, flux RPC et Vitest existants.
- Dépendances, tables, migrations ou journaux durables nouveaux : aucun.
- Périmètre initial : consultation humaine et références de navigation, sans changement des missions ni relances. Extension approuvée : UUID des commandes CLI et de l'outil MCP de fils partagés et montage MCP Claude/GLM attesté. Les tests d'écriture interagents sont des fixtures privées distinctes du canal humain FR147-15.

Les helpers /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/.specify/scripts/bash/setup-plan.sh et check-prerequisites.sh sont absents. Les templates locaux attendus sont absents. La sélection provient de `.specify/feature.json` ; les artefacts suivent manuellement l'outline des compétences. Aucun script absent n'est annoncé comme exécuté.

## Architecture retenue

### 1. Signal humain dans Bridget

Ajouter la seule capacité `HumanThreadWatchV1`, publiée sous `human_thread_watch_v1`. Son contrat est distinct des observations et sollicitations qui réveillent les agents. Les types, noms wire, bornes et commandes sont fixés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md.

Extraire la garde d'autorité de `handle_human_thread_view`, actuellement /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/daemon.rs:11790. Vue et suivi utilisent cette même garde. Elle conserve Client négocié, liaison T3 primaire unique, identité vivante, instance, hôte et projet attestés. La CLI ne fait ni Register, ni autostart, ni découverte d'identité agent.

Le suivi porte sur tous les fils accessibles de la conversation attestée, pas seulement sur le fil choisi. Il est installé avant son premier `ready`, sous le même verrou qui ordonne les mutations. `ready` seq0 est obligatoire, premier et non coalescible : une rafale avant l'écriture socket ne peut pas l'écraser. Une mutation ne peut donc tomber entre la lecture initiale et l'inscription. Chaque connexion reçoit une génération nouvelle. Chaque reprise déclenche une revalidation complète par les lectures146 ; aucun journal de rejeu durable n'est requis.

Publier `changed` seulement après mutation committée : création, publication, remplacement via publication et clôture pour le premier lot. Un nom d'annuaire réellement modifié peut invalider les fils accessibles. Le premier lot n'ajoute aucun CRUD de titres/appartenances ; l'extension US8 ajoute uniquement les membres via CLI/MCP et une invalidation humaine postcommit, sans édition UI.

Conserver la distinction `ThreadTxOutcome::Done` / `Replayed` avant la projection JSON de `tx_result`, actuellement /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:717. Un rejeu idempotent, un refus, un ACK ou une lecture n'émettent pas de changement. La liste générique de commandes mutatives ne suffit pas : Create n'y figure pas au socle (`daemon.rs:12624`).

Le registre de suivis et ses files sont en mémoire, bornés. Une saturation remplace les changements perdus par un `resync`, jamais par une fausse continuité. Les écritures socket sont hors verrou ; un lecteur lent ne bloque pas les mutations. Une déconnexion de liaison, une ambiguïté, une perte de projet ou un refus d'accès invalident le suivi. Aucun corps, titre, nom de membre, UUID de fil partagé, prompt ou erreur brute n'est transporté dans les signaux. Un changement de fil étranger à l'appartenance attestée ne produit aucun signal pour cette vue.

### 2. CLI de suivi et ressource T3

Étendre le client de lecture humain de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/communication/client.rs:147. La commande `bridget thread watch --t3-thread UUID --project-root ROOT --json` fournit un flux JSONL. Le handshake a un délai borné ; le flux accepté n'a pas le timeout unary de dix secondes. L'annulation ou la fermeture du pipe ferme la connexion locale.

Étendre le service existant /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts:74 avec `watch`. Il résout conversation et projet par `ProjectionSnapshotQuery`, puis transmet `workspaceRoot`. Le navigateur ne fournit aucun chemin racine, agent, instance ou droit.

Réutiliser la résolution d'exécutable de ce service. Pour le flux, utiliser le `ChildProcessSpawner` Effect existant dans une portée annulable. `ProcessRunner.run` collecte une sortie finale avec timeout : il reste correct pour `read`, mais ne convient pas à `watch`. Le décodeur JSONL borne chaque ligne et le tampon partiel avant son décodage Schema. Un stderr brut n'est jamais rendu ni journalisé comme diagnostic métier.

### 3. RPC et cycle de vie du client

Étendre /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/bridget.ts avec entrée et événement fermés. Ajouter `WS_METHODS.bridgetWatch = "bridget.watch"` au contrat RPC dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/contracts/src/rpc.ts, à côté de `bridget.read` (:998). Conserver `AuthOrchestrationReadScope` dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/auth/RpcAuthorization.ts.

Utiliser l'observation de flux existante dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/ws.ts:709, puis la factory de subscription native dans /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/runtime.ts. L'adaptation147 de /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/packages/client-runtime/src/state/orchestration.ts réutilise `createEnvironmentSubscriptionAtomFamily` pour conserver un `BridgetWatchState` volatil : dernier événement, readyGeneration et subscriptionId local. Fixer `idleTtlMs: 0` : le défaut de cinq minutes laisserait le suivi vivant après fermeture.

Ready protégé sur le wire ne garantit pas un rendu ready distinct avant changed. L'état sticky conserve sa preuve avant coalescence de l'atome. La cible native ajoute un visitId client local unique ; une visite n'a qu'une subscription et une réponse d'une visite ancienne reste rejetée. visitId/subscriptionId ne traversent pas le wire : RPC garde threadId/projectId seulement.

Le supervisor existant porte les reprises de transport WebSocket T3. Si ce transport reste sain alors que le CLI/daemon de suivi tombe, réutiliser Schedule Effect pour reprendre le seul stream CLI : au plus trois tentatives supplémentaires après l'initiale, seulement pour unavailable/command_failed/timeout. Chaque tentative revalide autorité/handshake ; ready neuf rattrape par lectures146. Durant l'attente, masquer corps et garder UUID. Au plafond, arrêt explicite avec refresh manuel ou nouvelle visite. Invalid_output/version/projet/binding/demande invalide ne sont jamais retryables. Aucun timer de lecture de corps ni retry infini ajouté.

Le panneau ne monte la consommation que lorsqu'il est visible et actif. Fermeture de surface, démontage, changement d'environnement/conversation ou page masquée libèrent abonnement, processus enfant et connexion. Au retour visible, la nouvelle génération impose une revalidation. Il ne faut pas simplement ignorer visuellement un flux resté monté dans une surface cachée.

### 4. Sélection dans le store existant

Étendre /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/rightPanelStore.ts par une table distincte `bridgetSelectionByContextKey`. La clé est `JSON.stringify([environmentId, projectId, threadId])`. La valeur est seulement un UUID de fil partagé validé. Aucun corps, titre, membre, credential ou message n'est persisté.

Ne pas ajouter cette valeur à `ThreadRightPanelState`. `upsertSurface` le reconstruit (:274) et `updateThread` peut le supprimer (:293). Ces actions natives ne doivent pas effacer le choix Bridget. Réutiliser la persistance et sa validation existantes. Un snapshot ancien sans la nouvelle table équivaut à une table vide.

Remplacer la sélection locale `useState(null)` du panneau (:73) par la référence de contexte du store. La liste récente conserve ses pages de20 entrées. Restaurer le choix via `show` puis `history_recent`, même hors de la page courante. Ne pas parcourir toutes les pages pour chercher le UUID ni sélectionner le premier fil à sa place.

### 5. Actualisation coalescée et lecture stable

Le signal fermé porte `version: 1`, `generation: UUID`, `seq: entier sûr JavaScript` et `status: ready | changed | resync`. Les trois statuts invalident liste et détail sélectionné, avec une seule revalidation groupée. Il n'y a aucun identifiant de fil dans le signal. `seq` croît dans la génération ; un dépassement impose une nouvelle connexion et un nouveau `ready`, jamais un retour à zéro dans la même génération. Chaque contexte possède au plus une revalidation en cours et un bit `dirty` complémentaire. Une rafale ne crée pas une promesse de lecture par signal. Une réponse dont la version locale a été dépassée ne publie pas un état présenté comme courant.

Les pages146 restent la source du contenu. La borne `snapshot_seq` d'une ancienne page masque un `superseded_by_seq` apparu après cet instantané, comme le prouvent /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs:718 et son test :2034. Une simple fusion de tête récente avec anciennes pages n'est donc pas correcte.

À revalidation, obtenir le nouveau snapshot S par la tête `history_recent`. Si plus de50 nouveautés séparent cette tête de l'ancien segment, parcourir les pages intermédiaires jusqu'à l'ancre ancienne. Reconstruire ensuite le segment déjà consulté avec `to_seq=S` commun, dans un staging volatile. Toutes ses pages et relations de remplacement sont relues avant publication atomique du segment cohérent. Si une page ancienne était en chargement, invalider sa génération locale, puis refaire ce parcours commun ; sa réponse tardive ne s'ajoute pas. Les nouveautés s'insèrent par séquence/UUID, sans trou ni doublon. Le coût est O(nouveautés + pages consultées), pas un inventaire de toute la base. Un échec de réconciliation n'est pas publié comme état courant.

Les lignes des messages inchangés gardent leurs clés, leur corps et leur sous-arbre DOM. Les dépliages, détails, focus et sélection de texte restent en place. La mise à jour ne force ni remontage global, ni scroll en tête, ni focus automatique. Les copies restent les corps complets originaux. Le retrait d'un contenu refusé prime sur ces garanties de confort.

Un refus autoritaire du fil (`thread_unavailable`) retire la sélection et purge le contenu. Une coupure, un timeout, une liaison momentanément indisponible ou un stockage indisponible conservent le UUID mais ne donnent pas une autorisation fraîche. Une perte d'autorité du contexte arrête le flux et masque son contenu ; elle n'est pas convertie en preuve que le fil mémorisé a été supprimé. À la reprise, aucun contenu non revalidé n'est réaffiché. Une génération locale de consultation distincte du UUID de contexte empêche A → B → A de valider une réponse de l'ancien A.

### 6. US5 — Destinataires dans la ligne auteur/date

Ajout approuvé le 2026-10-09. La lane Web étend seulement la ligne existante de /Users/moi/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx. Elle affiche Auteur → Noms pour notify targets/all, et Auteur · Sans sollicitation pour none. La date reste dans cette ligne ; aucune hauteur dédiée supplémentaire.

Utiliser les targets effectifs déjà projetés avec le message, y compris all, auteur exclu selon GO du principal. Résoudre leurs noms avec detail.members autorisé déjà chargé. Ne pas recalculer all depuis les membres actuels. Si un nom n'est pas disponible : libellé honnête Nom indisponible + UUID court, identifiant exact accessible dans les détails existants. Aucun nom inventé, appel d'annuaire, champ wire, API, mutation, notification ou réveil. L'exclusion de l'auteur ne prouve pas une livraison.

L'indication décrit la sollicitation demandée, pas une livraison ou lecture confirmée. La copie du corps reste strictement originale. Tests RED puis GREEN pour none/targets/all/nom absent/multidestinations et copie ; QA isolée compacte. T027–T030 ont été ajoutées après validation de l’US5 ; leurs preuves RED/GREEN/QA/convergence sont validées. Les IDs T001–T026 restent inchangés, les30 tâches sont cochées après le gate final.

## Réutilisation de l'existant

Le tableau exhaustif figure dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/reuse-audit.md. Les principales réutilisations sont la garde humaine145, les lectures et curseurs146, les mutations transactionnelles, l'annuaire, la résolution de contexte T3, les flux RPC, le superviseur, les subscriptions client-runtime et le store du panneau droit.

Seul le contrat de suivi humain est nouveau. Réutiliser les observations d'agents serait incorrect : elles peuvent solliciter un agent et ne constituent pas une invalidation de consultation autorisée. Aucun deuxième service métier, store générique, framework de transport, table SQLite ou modèle n'est créé.

## Vérification constitutionnelle

| Gate | Résultat de conception | Preuve ou limite |
| --- | --- | --- |
| III — workflow | PASS final isolé | Spec, plan, recherche, contrat, audits et30 tâches prouvées ; gate final du principal validé. |
| XVI — isolation | PASS | Deux worktrees147 ; bases142/145 intactes. |
| XV — tests avant suite | Planifié | RED puis GREEN ciblés ; aucune tâche ni recette déjà déclarée PASS. Commit/livraison demandent une autorisation distincte. |
| XVIII — complexité | PASS conception | Invalidation bornée O(1) par suivi ; dispatch O(S), S borné ; staging O(nouveautés + pages consultées), sans recherche linéaire répétée ni requête par message. |
| XIX — minimalisme | PASS conception | Contrat nouveau nécessaire ; transport, autorité, lectures, store et frameworks existants réutilisés ; zéro dépendance/table/journal nouveaux. |
| XX — paresse vertueuse | PASS conception | Une garde commune et une règle de reprise évitent deux politiques d'accès ou deux journaux. |
| XX — impatience maîtrisée | PASS conception | Tests ciblés et CLI de suivi reproductible ; actualisation prouvée par délai visible, pas par volume de code. |
| XX — responsabilité | PASS avec limite | Noms, refus, bornes, cycle de vie et tests explicites. Rien n'est encore implémenté ni activé. |
| XXI — contexte partagé | PASS | Propriétaires distincts ; les racines et les changements d'autres sessions ne sont pas édités. |

Les standards génériques Next.js/Axios, les dossiers Cartae et Pytest ne décrivent pas T3/Bridget. Réutiliser les piles React/Effect/Vitest et Rust existantes est une divergence justifiée, pas une omission de tests ou de typage. Les scénarios restent formulés en langage métier, indépendamment de leur runner.

Réévaluation après conception : mêmes gates ; aucun problème de duplication non arbitré. Le principal a revu le gate avant les tâches, puis autorisé l'implémentation. L'ADR147 consigne le canal humain et la convergence, sans inventer une nouvelle plateforme. Les tableaux de conception ne remplacent pas les preuves de validation finales.

## Plan de validation

1. Tests Rust protocol/daemon/CLI147 RED sur le socle : capacité inconnue, aucun signal pour Create/Post réel, garde de liaison, inscription initiale, no-replay, annulation et sortie bornée.
2. Tests Rust GREEN : mutation committée une fois, replay/ACK/read/refus silencieux, rename pertinent, files saturées → resync, ready seq0 préservé en premier même après une rafale supérieure à16 avant écriture, client lent sans blocage, perte d'autorité, daemon redémarré, aucune action agent.
3. Tests service/RPC/runtime T3 RED puis GREEN : contexte serveur, payload fermé, scope lecture, JSONL fractionné/invalide/trop grand, handshake, EOF, cancellation, idleTtl0 et reprise. Distinguer WebSocket repris par supervisor et CLI rompu sous WebSocket sain, avec initiale + trois reprises maximum uniquement techniques. Tester ready+changed avant premier rendu : état sticky garde readyGeneration, une subscription par visitId, aucun champ wire ajouté. Un refus métier fermé reste arrêté, sans boucle de retries, jusqu'à refresh manuel ou nouvelle visite.
4. Tests store/panneau RED puis GREEN : A → B → A, projet/environnement, choix hors première page, ancien snapshot, code refus ≠ panne, late response, coalescence, 60s sans relecture, 100 changements et pages déjà chargées. Cas obligatoires : remplacement d'une consigne en page2 pendant le chargement de page3 ; nouveau snapshot commun S, publication atomique, réponse ancienne ignorée et remplacement visible en page2. Plus de50 nouveautés : intervalle entre tête et ancre ancienne complètement rattrapé sous S, pas de trou ni doublon.
5. Recette intégrée isolée : vrai daemon et CLI147 avec données synthétiques puis surface T3 de test. Observer le délai de deux secondes, focus, texte sélectionné, dépliage, copie, supersession, page masquée et fermeture. Aucun agent ou base T3 actifs.
6. Contrôles adaptés : format Rust, lint/clippy, tests ciblés puis suites touchées ; typecheck/lint/tests/build T3 selon scripts existants. Journaliser sorties, nombres réels, RED/GREEN et limites.
7. Contre-revue, audit, Analyze et convergence documentaire. Aucun fournisseur adverse joignable dans le même projet à ce stade ; ne pas contacter un autre projet pour simuler cette preuve. Une revue locale ne sera pas qualifiée d'inter-fournisseurs.

## Risques et décisions de sortie

- Fenêtre inscription/mutation : inscription avant `ready`, verrou commun et test d'interleaving.
- Mutation committée versus rejeu : conserver l'issue transactionnelle, pas un simple JSON `ok`.
- Fuite d'autorité : résolution côté serveur et garde commune avant inscription et publication.
- Ressources laissées ouvertes : portée Effect, idleTtl0, visibilité et tests des dix cycles.
- Rafale ou client lent : coalescence, bornes, resync et socket hors verrou.
- Pagination : fusion par clés et réconciliation contrôlée ; pas de reset visuel de la lecture.
- Réponse obsolète A → B → A : génération de consultation, pas seulement comparaison d'identifiant.
- Données anciennes après panne : sélection conservée, affichage soumis à revalidation ; refus certain purge.

Le plan et le gate ont été revus par le principal. Implémentation et preuves sont en cours. La convergence autorisée intègre les trois cas observés : ready coalescé côté rendu, CLI rompu sous WebSocket sain et intervalle de plus de50 nouveautés. Aucun nouveau task ni achèvement T026 n'est déduit de cette correction de conception. Aucun commit, fusion, push, installation, redémarrage ni cleanup ne sont réalisés par ce plan.

## Extension approuvée — US6 UUID / US7 Claude-GLM

### US6 — Plan concret validé avant implémentation

Réutiliser require_uuid dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:493. Le rendre pub(crate) afin que le CLI le réutilise, sans nouveau helper. Parser via la dépendance uuid existante, exiger la forme hyphénée36 caractères équivalente au résultat canonique avec eq_ignore_ascii_case, puis retourner la forme minuscule. Refuser formes compactes, accolades, URN, espaces ou caractères non hexadécimaux dans les champs UUID déclarés. Normaliser avant tout hash d'idempotence et requête SQL concernée.

canonical_uuid dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:350 reste strict et inchangé pour le contrat humain145/146/147 et les curseurs fermés. La compatibilité agent ne remplace pas une garde d'identité/projet. Aucun élargissement du protocole humain.

thread_members dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/cli.rs:1519 réutilise require_uuid pour reconnaître puis normaliser le UUID hyphéné majuscule/mixte. Les noms et préfixes gardent leur résolution existante. Ne pas normaliser un corps ni un nom.

Normaliser aussi le reçu métier ACK, actuellement validé séparément dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:1027, avant lookup/idempotence cohérents avec Post. Conserver le refus métier ReceiptInvalid au lieu de le transformer en erreur générique. Create/Post/Close/ACK, membres et targets suivent la même représentation avant hash/SQL. Zéro dépendance, migration, table, nouvelle abstraction ou changement de protocole.

Tests avant code : sept scénarios comportementaux minimaux couvrent Create canonique, Post canonique/copie, Close replay, ACK/reçu, membres/targets, formes invalides et invariants humains. Fixtures MCP/ThreadRequest agent et formes CLI réelles ; UUID avec a–f pour rendre la casse observable. Un replay en casse différente ne produit qu'un effet. Tester les noms/préfixes CLI pour éviter une régression du résolveur.

Gate de réutilisation : REUTILISER require_uuid + thread_members + tests colocalisés existants. Le principal a arbitré ce plan avant implémentation ; aucune duplication proposée. T031–T034 sont cochées après reçus RED/GREEN/régressions et GO du principal.

### US7 — Plan MCP concret validé avant tâches

Gate du principal reçu le2026-10-09 après les amendments de revue. La configuration effective du cas regional-wrkr-1/ClaudeAgent/claude_glm manque Bridget ; T3 injecte t3-code seulement. Aucune exclusion volontaire GLM.

Préparer le montage avec le SDK natif : mcpServerStatus puis setMcpServers, après création de la query et avant sessions.set/ready ou tout premier prompt. Même garantie pour new et resume ; aucun restart d'une query active. Conserver options.mcpServers.t3-code dans le payload, car setMcpServers remplace les serveurs dynamiques.

Fichiers production retenus :
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeMcp.ts : fonction Effect de préparation et politique de compatibilité, pas une classe enveloppe neutre.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSettingsPaths.ts : extraire calcul actuel des sources/settings et findRepositoryRoot depuis ClaudeSkills.ts ; consommateurs réels Skills et MCP. Exception architecture<=3 usages autorisée : même calcul de scope critique pour la sécurité, sémantiques MCP stricte et Skills fail-open préservées.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetExecutable.ts : extraire résolution Reader existante, priorité T3CODE_BRIDGET_EXECUTABLE puis HOME/.local/bin/bridget puis PATH, rejet cmd/bat ; trois usages read/watch/MCP, ordre inchangé.
- /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Layers/ClaudeAdapter.ts : interface ClaudeQueryRuntime expose les deux méthodes SDK ; préparation avant premier prompt, mcpServers t3-code conservé.
- Consommateurs existants : /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/provider/Drivers/ClaudeSkills.ts et /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/server/src/bridget/BridgetReader.ts.

Respecter tout serveur homonyme Bridget déjà effectif dans tous les scopes/statuts, même failed/disabled/plugin/user/managed ; ne pas l'écraser. Opt-out flags/settings et métadonnées invalides/I/O refus conduisent à un skip codé fermé ; distinguer NotFound normal. Identité héritée non vide, y compris espaces : skip codé, jamais modification d'environnement/profil pour contourner ce signal. Aucun champ d'identité dans la configuration stdio MCP ; permissions normales et canUseTool inchangés, aucun alwaysLoad.

Garde conservative MCP validée par le principal : si cwd est un enfant du dépôt, inspecter aussi les métadonnées root/.claude/settings.json et projects[root] de .claude.json pour respecter un refus d'auto-montage T3. Vérifier metadata avant status SDK ; flags/identity refusent sans I/O. Le calcul partagé et comportement Skills restent inchangés. Ne pas prétendre que la CLI charge ces plain settings : le getter MCP privé établit seulement root/.mcp.json scope project ; RC0 sous disabled ne prouve pas que le serveur soit activé.

Initialisation rejetée/pending/deadline : fermer query candidate et annuler son consumer avant ready/prompt. Une promesse SDK résolue tardivement ne monte pas le serveur après timeout. Candidate sous propriétaire/finalizer local unique jusqu'à sessions.set : si prepare réussit mais close ancienne query échoue, fermer candidate/consumer sans ready/prompt ni orphelin. Amendement T050 validé dans FR21/22 existants, pas nouveau framework/helper. Diagnostics codés sans secrets. Aucun endpoint, table, dépendance, profil global ou nouvelle autorité t3-code.

Six groupes RED dans ClaudeMcp.test.ts, ClaudeSettingsPaths.test.ts, BridgetExecutable.test.ts et ClaudeAdapter.test.ts : new propre, resume, homonymes/scopes/statuts, opt-out/config/I/O, identité héritée, rejet/pending/deadline/tardy. Fixtures SDK sans modèle distinctes d'une vraie façade MCP OS/credentials/daemon privé pour tests d'autorité no-forgery/birthbad. Le stub de handshake UUID102v33 ne prouve pas cette autorité. T035–T041 suivent le gate ; propriétaire Rust AUTH séparé après libération du writer UUID.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.

## US9 — plan visuel natif accepté avant tâches/code

US147-09 P2/FR29–31/SC19–20 : choix natif en haut, retirer header h2/membres en bas et garder détails accessibles. Main a relu les wrappers Sidebar1449/indexcss1081/PreviewPanelShell133 ; findings Web réutilisés, pas exploration globale répétée. Gate conception avant code, pas nouveau composant/store/API/dépendance/helper.

Dans /Volumes/8TB2/50-repos-archives/11.Repositories/t3code-local/.worktrees/147-panneau-bridget-vivant/apps/web/src/components/BridgetPanel.tsx, surface de liste bg-sidebar pour contraste clair et ligne choisie row-active/foreground. Hover seulement sur les autres lignes, focus-visible anneau inset, aria-current/pressed sur contrôle du choix. Icône et palette natives, pas rebrand.

Déplacer details sous la ligne courante comme frère du bouton, jamais contrôle interactif imbriqué dans bouton. Si choix absent de page/liste filtrée ET détail autorisé disponible, épingler une seule ligne choisie, même parent/key ; aucune ligne/titre/membres exposés en cas de refus ou masque. Pas nouvel appel ni sélection automatique. En bas, conserver ScrollArea, nœuds de messages par séquence et copie exacte ; retirer seulement le header répété. Détails accessibles et états loading/refused/empty conservés.

Cinq groupes RED avant code : styles/état accessible/nativefocus ; détails frère et header inférieur absent ; choix horspage1/filtre unique et restauration A→B→A ; masque/refus sans ligne-fuite ; refresh/identitéDOM/ScrollArea/copie et lisibilité clair/sombre/360px. Recette dans aperçu isolé déjà autorisé, aucune conversation/agent actif modifié.

## US8 — Ajout de membres et historique complet, plan accepté

Décision du principal : AddMembers / add_members dans les contrats de fils partagés existants, pas un nouveau canal ou composeur. CLI `bridget thread add-members FIL --member UUID --member UUID --id UUID [--cross-project-reason MOTIF]` réutilise --member répétable. Créateur initial seul, fil ouvert, maximum16 UUID canoniques dédoublonnés. Historique complet autorisé, curseur nouveau membre0. Aucun retrait ou transfert.

Réutiliser ThreadAction (protocol.rs2647), thread_request (communication/client.rs122–147), ThreadRequest/allowlist (daemon.rs13943/15176), handle_with_change/digest/replay (threads.rs573), require_uuid493, ensure_schema (store/threads.rs303), thread_members/parser (cli.rs1541), tool schema (mcp.rs484). Tous ces chemins sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates ; les responsabilités et emplacements ont été vérifiés par l'explorateur compact_t3 et le principal.

Négocier ClientCapability::ThreadMembersV1, wire thread_members_v1, une seule fois dans le même ClientHello que CommunicationProjectsV1. Élargir explicitement l'allowlist auxiliary et vérifier la capacité attestée côté daemon ; un ancien peer ne reçoit pas l'action incompatible. Nom figé par le principal, sérialisation à tester avant GREEN.

Valider l'union complète anciens+candidats avec les contrôles projet existants, puis transmettre cette union validée à la transaction. La transaction relit créateur/état/membres et refuse une divergence concurrente au lieu d'autoriser une audience non validée. Limite de stockage borne entries+16. NoChange (tous présents) ne crée aucun reçu et n'engage pas la clé d'opération. Une vraie addition écrit son résultat idempotent ; replay ne modifie rien.

Migration ciblée du schéma fil v2→v3 : reconstruire uniquement le CHECK de thread_operations pour accepter add_members. Tester aussi v1→v3 via fixtures136, seconde ouverture et rollback atomique ; opérations, reçus exacts, clés, index et relations conservés. Pas de remise à zéro ni nouvelle base. La borne members et les kinds hérités restent inchangés.

AddMembers reste exclu du chemin mutates déclenchant dispatch_thread_wakes, même si un wakepending ancien existe. Aucun nouvel entry, alerte agent, activité updated_at ou poste historique. Publier uniquement l'invalidation humaine autorisée après commit réel. Futur notify=all prend l'audience augmentée ; anciens targets effectifs et reçus restent identiques. Les lectures read/history gardent leur contrat : corps historiques exacts via history, aucune exécution de consigne ancienne.

RED avant code : négociation auxiliaire CLI/MCP réelle combinée, migrationsv1/v2→v3, créateur/fermé/max16/invalides/projets/concurrence, NoChange/replay, curseur0, ancien wakepending non dispatché, all futur et targets passés stables, historique multipage et watch postcommit. Gate et deux passes Analyze principales acceptés. T042–T048 suivent ce gate ; fichiers disjoints de daemon autorisés, seul daemon attend libération T040 AUTH. Aucun writer daemon concurrent. Docs utilisateur locales mises à jour sans publication globale.
