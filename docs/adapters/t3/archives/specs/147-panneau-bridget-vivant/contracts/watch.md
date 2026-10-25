# Archives techniques de l’adaptateur T3

Document d’origine : [specs/147-panneau-bridget-vivant/contracts/watch.md](../../../../../../../specs/147-panneau-bridget-vivant/contracts/watch.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Capacité Rust : `ClientCapability::HumanThreadWatchV1`.
- Nom wire : `human_thread_watch_v1`.
- Requête IPC : `WrapperToDaemon::HumanThreadWatchV1 { request: HumanThreadWatchV1 }`.
- Réponse IPC de flux : `DaemonToWrapper::HumanThreadWatchEvent { event: HumanThreadWatchEvent }`.
- CLI : `bridget thread watch --t3-thread UUID --project-root ROOT --json`.
- Constante T3 : `WS_METHODS.bridgetWatch = "bridget.watch"`.
- RPC : `bridget.watch`, stream de `BridgetWatchEvent`.
- Service T3 étendu : `BridgetReader.watch`.
- Mémoire de navigation : `bridgetSelectionByContextKey` dans `useRightPanelStore`.

<a id="extrait-2"></a>

## Extrait 2

IPC, fermé : `version: 1`, `t3_thread_id: UUID canonique`, `project_root: chemin workspace résolu par le serveur T3`.

<a id="extrait-3"></a>

## Extrait 3

RPC, fermé : `threadId` de conversation T3 et `projectId` uniquement. L'environnement est celui de la connexion T3 authentifiée. Les champs `projectRoot`, `agentId`, `instanceId`, `sharedThreadId`, credentials et toute propriété supplémentaire sont refusés.

<a id="extrait-4"></a>

## Extrait 4

Codes autorisés : `unsupported_version`, `binding_unavailable`, `project_mismatch`, `thread_unavailable`, `invalid_request`, `storage_unavailable`, `response_too_large`. Ce sont les codes humains existants ; aucune erreur brute, stderr, argv ou message libre n'est exposé. T3 les traduit en `BridgetReadError` et termine son stream typé. Les diagnostics de transport utilisent les catégories fermées du service, sans body.

<a id="extrait-5"></a>

## Extrait 5

- `thread_unavailable`, confirmé par une lecture du fil : refus d'accès ou absence du fil → supprimer le choix et purger ses données.
- `binding_unavailable`, transport absent, timeout ou stockage indisponible : arrêter le suivi, garder le choix, masquer le contenu non vérifié et proposer reprise.
- `project_mismatch` ou refus du scope T3 : arrêter et purger le contenu du contexte refusé. Ne pas transformer ce refus de contexte en preuve de suppression du fil ; son UUID n'accorde aucun accès.
- Version incompatible ou demande invalide : état explicite, aucun repli silencieux, aucun flux laissé actif.

<a id="extrait-6"></a>

## Extrait 6

Deux ruptures distinctes existent. Le supervisor existant reprend le transport WebSocket T3. Si ce WebSocket reste sain mais que le processus CLI/daemon de suivi tombe, le service réessaie son seul stream technique avec Schedule existant, au plus trois tentatives supplémentaires après l'initiale, espacées d'une seconde. Seules les catégories techniques fermées `unavailable`, `command_failed` et `timeout` sont éligibles. Chaque nouvelle tentative refait handshake et autorité ; son nouveau ready impose une revalidation. Pendant l'attente, l'état local ne garde aucun événement ou preuve d'accès actif.

<a id="extrait-7"></a>

## Extrait 7

1. Vérifier version, rôle Client nu et capacité exacte.
2. Résoudre le projet hors verrou à partir de la racine fournie par le serveur T3.
3. Sous le verrou du daemon, revalider liaison T3 primaire unique, route, agent/instance stable, hôte, présence et projet attestés.
4. Installer le suivi, sa génération et sa file avant d'émettre `ready`, sous le verrou qui ordonne les mutations. Sérialiser/écrire les événements hors verrou.
5. Après commit d'une mutation réelle, déterminer les suivis dont l'agent est membre du fil concerné. Revalider l'autorité. Une mutation d'un fil étranger ne produit aucun signal.
6. À perte de binding/projet/autorité, terminer avec code fermé et libérer le suivi. Ne pas publier un identifiant de fil désormais inaccessible.

<a id="extrait-8"></a>

## Extrait 8

| Cas | Résultat exigé |
| --- | --- |
| Watch autorisé | ready seq0, génération nouvelle, aucune identité agent créée |
| Create/Post/supersedes/Close réellement committé | changed pour les seules vues concernées |
| Rejeu, lecture, ACK, mutation refusée | aucun changed |
| Mutation d'un fil étranger | aucun signal et aucune fuite d'identifiant |
| Mutation entre inscription et ready | changement pris en compte ; pas de fenêtre perdue |
| Plus de16 mutations avant écriture de ready | ready seq0 reste premier ; resync suit sans croissance de file |
| ready et changed reçus avant premier rendu de l'atome | readyGeneration reste attestée ; dernier événement accepté pour la seule visite/subscription courante |
| Rafale/file pleine | mémoire bornée, resync, état relu exact |
| Client lent | mutations non bloquées ; suivi fermé proprement |
| Restart/reconnexion | nouvelle génération, ready, rattrapage des lectures |
| seq overflow | arrêt/reconnexion ; pas de wrap dans la même génération |
| Ligne invalide, UTF-8 invalide, excès de champs ou taille | refus fermé ; pas de données partielles affichées |
| Binding ambigu/déconnecté/projet refusé | arrêt et contenu purgé/masqué ; choix conservé sauf refus du fil |
| Refus métier fermé vs coupure transport | refus ne se reconnecte pas en boucle ; transport transitoire reprend avec génération nouvelle |
| CLI/daemon rompu avec WebSocket T3 sain | au plus initiale + trois reprises techniques ; corps masqués, UUID gardé ; ready neuf revalide |
| invalid_output/version/projet/binding/demande invalide | aucune reprise technique automatique ; refresh manuel explicite |
| Close UI/page masquée/dix cycles | zéro suivi restant, aucune rétention idle cinq minutes |
| A → B → A, choix hors première page | choix correct restauré, réponse ancienne ignorée |
| Message déplié/focus/copietexte pendant nouveauté | état des éléments inchangés conservé |
| Remplacement d'une consigne page2 pendant chargement page3 | nouveau snapshot commun S ; segment publié atomiquement ; réponse page3 ancienne ignorée ; remplacement présent en page2 |
| Plus de50 nouveautés entre deux invalidations | tête, intervalle intermédiaire et segment consulté cohérents sous S ; aucun trou ni doublon |
