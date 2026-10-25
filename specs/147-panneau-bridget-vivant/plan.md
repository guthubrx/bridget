# Plan technique — SPEC147

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-1).

## Résumé

## Complément US10 — compatibilité adaptateur de bureau V2 (2026-10-09)

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-4).

Une connexion WebSocket courte par dispatch, loopback/auth/délai/taille bornés.
Dépendance directe tungstenite0.26.2, déjà utilisée par bridget-transport, sans TLS,
handshake seul : nécessaire car minreq ne gère
pas WebSocket. Préférer son implémentation RFC testée à des trames artisanales ou
à un sous-processus Node pour chaque message. Pas nouveau service ni pool.
Source primaire : https://docs.rs/tungstenite/0.26.2/tungstenite/client/fn.client_with_config.html.

La sélection de base d'identité suit le protocole authentifié, jamais l'existence
du fichier. Lire les sessions natives V2 en lecture seule ; garder les gardes OS
et le refus d'ambiguïté. Aucune identité depuis le titre, cwd ou texte du modèle.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-6).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-8).

## Contexte technique et espaces isolés

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-9).

Les helpers /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/.specify/scripts/bash/setup-plan.sh et check-prerequisites.sh sont absents. Les templates locaux attendus sont absents. La sélection provient de `.specify/feature.json` ; les artefacts suivent manuellement l'outline des compétences. Aucun script absent n'est annoncé comme exécuté.

## Architecture retenue

### 1. Signal humain dans Bridget

Ajouter la seule capacité `HumanThreadWatchV1`, publiée sous `human_thread_watch_v1`. Son contrat est distinct des observations et sollicitations qui réveillent les agents. Les types, noms wire, bornes et commandes sont fixés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/specs/147-panneau-bridget-vivant/contracts/watch.md.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-10).

Le suivi porte sur tous les fils accessibles de la conversation attestée, pas seulement sur le fil choisi. Il est installé avant son premier `ready`, sous le même verrou qui ordonne les mutations. `ready` seq0 est obligatoire, premier et non coalescible : une rafale avant l'écriture socket ne peut pas l'écraser. Une mutation ne peut donc tomber entre la lecture initiale et l'inscription. Chaque connexion reçoit une génération nouvelle. Chaque reprise déclenche une revalidation complète par les lectures146 ; aucun journal de rejeu durable n'est requis.

Publier `changed` seulement après mutation committée : création, publication, remplacement via publication et clôture pour le premier lot. Un nom d'annuaire réellement modifié peut invalider les fils accessibles. Le premier lot n'ajoute aucun CRUD de titres/appartenances ; l'extension US8 ajoute uniquement les membres via CLI/MCP et une invalidation humaine postcommit, sans édition UI.

Conserver la distinction `ThreadTxOutcome::Done` / `Replayed` avant la projection JSON de `tx_result`, actuellement /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/threads.rs:717. Un rejeu idempotent, un refus, un ACK ou une lecture n'émettent pas de changement. La liste générique de commandes mutatives ne suffit pas : Create n'y figure pas au socle (`daemon.rs:12624`).

Le registre de suivis et ses files sont en mémoire, bornés. Une saturation remplace les changements perdus par un `resync`, jamais par une fausse continuité. Les écritures socket sont hors verrou ; un lecteur lent ne bloque pas les mutations. Une déconnexion de liaison, une ambiguïté, une perte de projet ou un refus d'accès invalident le suivi. Aucun corps, titre, nom de membre, UUID de fil partagé, prompt ou erreur brute n'est transporté dans les signaux. Un changement de fil étranger à l'appartenance attestée ne produit aucun signal pour cette vue.

### 2. CLI de suivi et ressource adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-11).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-12).

Réutiliser la résolution d'exécutable de ce service. Pour le flux, utiliser le `ChildProcessSpawner` Effect existant dans une portée annulable. `ProcessRunner.run` collecte une sortie finale avec timeout : il reste correct pour `read`, mais ne convient pas à `watch`. Le décodeur JSONL borne chaque ligne et le tampon partiel avant son décodage Schema. Un stderr brut n'est jamais rendu ni journalisé comme diagnostic métier.

### 3. RPC et cycle de vie du client

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-13).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-14).

Ready protégé sur le wire ne garantit pas un rendu ready distinct avant changed. L'état sticky conserve sa preuve avant coalescence de l'atome. La cible native ajoute un visitId client local unique ; une visite n'a qu'une subscription et une réponse d'une visite ancienne reste rejetée. visitId/subscriptionId ne traversent pas le wire : RPC garde threadId/projectId seulement.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-15).

Le panneau ne monte la consommation que lorsqu'il est visible et actif. Fermeture de surface, démontage, changement d'environnement/conversation ou page masquée libèrent abonnement, processus enfant et connexion. Au retour visible, la nouvelle génération impose une revalidation. Il ne faut pas simplement ignorer visuellement un flux resté monté dans une surface cachée.

### 4. Sélection dans le store existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-16).

Ne pas ajouter cette valeur à `ThreadRightPanelState`. `upsertSurface` le reconstruit (:274) et `updateThread` peut le supprimer (:293). Ces actions natives ne doivent pas effacer le choix Bridget. Réutiliser la persistance et sa validation existantes. Un snapshot ancien sans la nouvelle table équivaut à une table vide.

Remplacer la sélection locale `useState(null)` du panneau (:73) par la référence de contexte du store. La liste récente conserve ses pages de20 entrées. Restaurer le choix via `show` puis `history_recent`, même hors de la page courante. Ne pas parcourir toutes les pages pour chercher le UUID ni sélectionner le premier fil à sa place.

### 5. Actualisation coalescée et lecture stable

Le signal fermé porte `version: 1`, `generation: UUID`, `seq: entier sûr JavaScript` et `status: ready | changed | resync`. Les trois statuts invalident liste et détail sélectionné, avec une seule revalidation groupée. Il n'y a aucun identifiant de fil dans le signal. `seq` croît dans la génération ; un dépassement impose une nouvelle connexion et un nouveau `ready`, jamais un retour à zéro dans la même génération. Chaque contexte possède au plus une revalidation en cours et un bit `dirty` complémentaire. Une rafale ne crée pas une promesse de lecture par signal. Une réponse dont la version locale a été dépassée ne publie pas un état présenté comme courant.

Les pages146 restent la source du contenu. La borne `snapshot_seq` d'une ancienne page masque un `superseded_by_seq` apparu après cet instantané, comme le prouvent /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/147-panneau-bridget-vivant/crates/bridget-daemon/src/store/threads.rs:718 et son test :2034. Une simple fusion de tête récente avec anciennes pages n'est donc pas correcte.

À revalidation, obtenir le nouveau snapshot S par la tête `history_recent`. Si plus de50 nouveautés séparent cette tête de l'ancien segment, parcourir les pages intermédiaires jusqu'à l'ancre ancienne. Reconstruire ensuite le segment déjà consulté avec `to_seq=S` commun, dans un staging volatile. Toutes ses pages et relations de remplacement sont relues avant publication atomique du segment cohérent. Si une page ancienne était en chargement, invalider sa génération locale, puis refaire ce parcours commun ; sa réponse tardive ne s'ajoute pas. Les nouveautés s'insèrent par séquence/UUID, sans trou ni doublon. Le coût est O(nouveautés + pages consultées), pas un inventaire de toute la base. Un échec de réconciliation n'est pas publié comme état courant.

Les lignes des messages inchangés gardent leurs clés, leur corps et leur sous-arbre DOM. Les dépliages, détails, focus et sélection de texte restent en place. La mise à jour ne force ni remontage global, ni scroll en tête, ni focus automatique. Les copies restent les corps complets originaux. Le retrait d'un contenu refusé prime sur ces garanties de confort.

Un refus autoritaire du fil (`thread_unavailable`) retire la sélection et purge le contenu. Une coupure, un timeout, une liaison momentanément indisponible ou un stockage indisponible conservent le UUID mais ne donnent pas une autorisation fraîche. Une perte d'autorité du contexte arrête le flux et masque son contenu ; elle n'est pas convertie en preuve que le fil mémorisé a été supprimé. À la reprise, aucun contenu non revalidé n'est réaffiché. Une génération locale de consultation distincte du UUID de contexte empêche A → B → A de valider une réponse de l'ancien A.

### 6. US5 — Destinataires dans la ligne auteur/date

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-17).

Utiliser les targets effectifs déjà projetés avec le message, y compris all, auteur exclu selon GO du principal. Résoudre leurs noms avec detail.members autorisé déjà chargé. Ne pas recalculer all depuis les membres actuels. Si un nom n'est pas disponible : libellé honnête Nom indisponible + UUID court, identifiant exact accessible dans les détails existants. Aucun nom inventé, appel d'annuaire, champ wire, API, mutation, notification ou réveil. L'exclusion de l'auteur ne prouve pas une livraison.

L'indication décrit la sollicitation demandée, pas une livraison ou lecture confirmée. La copie du corps reste strictement originale. Tests RED puis GREEN pour none/targets/all/nom absent/multidestinations et copie ; QA isolée compacte. T027–T030 ont été ajoutées après validation de l’US5 ; leurs preuves RED/GREEN/QA/convergence sont validées. Les IDs T001–T026 restent inchangés, les30 tâches sont cochées après le gate final.

## Réutilisation de l'existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-18).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-19).

Réévaluation après conception : mêmes gates ; aucun problème de duplication non arbitré. Le principal a revu le gate avant les tâches, puis autorisé l'implémentation. L'ADR147 consigne le canal humain et la convergence, sans inventer une nouvelle plateforme. Les tableaux de conception ne remplacent pas les preuves de validation finales.

## Plan de validation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-20).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-21).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-22).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-23).

Respecter tout serveur homonyme Bridget déjà effectif dans tous les scopes/statuts, même failed/disabled/plugin/user/managed ; ne pas l'écraser. Opt-out flags/settings et métadonnées invalides/I/O refus conduisent à un skip codé fermé ; distinguer NotFound normal. Identité héritée non vide, y compris espaces : skip codé, jamais modification d'environnement/profil pour contourner ce signal. Aucun champ d'identité dans la configuration stdio MCP ; permissions normales et canUseTool inchangés, aucun alwaysLoad.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-24).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-25).

Six groupes RED dans ClaudeMcp.test.ts, ClaudeSettingsPaths.test.ts, BridgetExecutable.test.ts et ClaudeAdapter.test.ts : new propre, resume, homonymes/scopes/statuts, opt-out/config/I/O, identité héritée, rejet/pending/deadline/tardy. Fixtures SDK sans modèle distinctes d'une vraie façade MCP OS/credentials/daemon privé pour tests d'autorité no-forgery/birthbad. Le stub de handshake UUID102v33 ne prouve pas cette autorité. T035–T041 suivent le gate ; propriétaire Rust AUTH séparé après libération du writer UUID.

Portée UUID US6 : seulement les actions et l'outil de fils partagés create/post/read/ack/history/show/close et leurs références UUID thread/operation/membres/targets/reçus. Aucun changement des IDs opaques send ou d'autres outils, de l'acteur d'autorité, des noms/préfixes UUID partiels, de canonical_uuid humain ni des curseurs fermés.

## US9 — plan visuel natif accepté avant tâches/code

US147-09 P2/FR29–31/SC19–20 : choix natif en haut, retirer header h2/membres en bas et garder détails accessibles. Main a relu les wrappers Sidebar1449/indexcss1081/PreviewPanelShell133 ; findings Web réutilisés, pas exploration globale répétée. Gate conception avant code, pas nouveau composant/store/API/dépendance/helper.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/147-panneau-bridget-vivant/plan.md#extrait-26).

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
