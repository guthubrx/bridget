# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/plan.md](../../../../../../specs/145-bridget-thread-viewer/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Documents et Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`, base `3bb89e0d`.
T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`, base `a1a4f2ef12`.
Branches isolées : `session-145-bridget-thread-viewer`. Aucun autre worktree, processus actif, mission, LaunchAgent, modèle ni installation n'est modifié.

<a id="extrait-2"></a>

## Extrait 2

T3 utilise React, TypeScript, Zustand, Effect et son contrat RPC existant. Bridget utilise Rust et SQLite. Les prescriptions génériques Next.js, Axios, Pytest ou Cartae ne remplacent pas ces piles. Zéro dépendance nouvelle prévue. La livraison reste soumise à une autorisation distincte.

<a id="extrait-3"></a>

## Extrait 3

1. La surface native `BridgetPanel` reçoit la référence de conversation et d'environnement T3 actuelle. Le store du panneau droit conserve seulement les informations de navigation natives ; les corps et credentials ne sont pas persistés.
2. Une RPC `bridget.read`, constante `WS_METHODS.bridgetRead`, porte trois actions de lecture fermées avec `AuthOrchestrationReadScope`. La connexion authentifiée choisit l'environnement. Le navigateur n'envoie ni agent Bridget ni chemin racine d'autorité.
3. Le service Effect T3 résout la conversation par `ProjectionSnapshotQuery.getThreadShellById`, puis le projet par `getProjectShellById`. Il utilise `project.workspaceRoot`. Le `worktreePath` décrit le contexte, pas la racine d'autorisation.
4. Le service appelle la CLI par `ProcessRunner` avec argv séparés, délai de six secondes et sortie limitée à 256 Kio. Aucun `sh -c`, contenu interpolé ni recherche réseau distante n'est introduit.
5. La CLI `bridget thread inspect` négocie la capacité `HumanThreadViewV1` sur une connexion IPC Client Unix locale. Son enum fermé autorise seulement list, show et history. La voie agent `ThreadRequest` garde ses règles actuelles et son refus du rôle Client.
6. Le daemon résout l'agent par un fait `T3ThreadBindingFact` détenu par la connexion primaire T3. Il vérifie la route, l'identité vivante, les UUID stables de fil et d'instance et la racine du projet T3 actif. La liaison fonctionne quand le modèle est dormant ; aucun PID MCP n'est requis.
7. Les lectures `list`, `show`, `history` et leur stockage restent réutilisés. L'appartenance est vérifiée à chaque lecture. Une projection humaine retire curseurs, réveils, credentials et métadonnées de transport. Les noms proviennent de `Store::agent_display_name`, sans génération ni résolution distante.

<a id="extrait-4"></a>

## Extrait 4

Le handler humain retourne avant `collect_closed_attach_views`, maintenance mutative. Protéger aussi les refus. La preuve de source T3 utilise un `HashSet` en mémoire `t3_project_connections`, car le tuple `communication_projects` perd cette source. Retirer cette preuve sur fait autre/invalide et à déconnexion. L'index connexion → liaison conserve une ambiguïté comme refus.

<a id="extrait-5"></a>

## Extrait 5

Résoudre l'exécutable par `T3CODE_BRIDGET_EXECUTABLE` via `HostProcessEnvironment`, puis le binaire utilisateur `/Users/moi/.local/bin/bridget` sur cette machine, ou `resolveCommandPath("bridget")` existant. Le chemin utilisateur générique est construit depuis HOME. Refuser `.cmd` et `.bat` Windows. Décoder un JSON métier sur exit2 ; ne pas exposer stderr brut.

<a id="extrait-6"></a>

## Extrait 6

La liaison primaire publie le fait uniquement après `Registered` et `report_project_context`. Le daemon accepte le fait si la route appartient à cette connexion, si l'identité vivante correspond à `stable_uuid(thread)`, si l'instance correspond à `stable_uuid(instance:thread)` et si `communication_projects.0` atteste le projet T3 actif. Ne pas accepter `last_known` comme autorité.

<a id="extrait-7"></a>

## Extrait 7

Comparer le projet après `resolve_communication_project(workspaceRoot,host,T3,None)`. Une racine Git réelle peut différer du chemin workspace. Ne pas comparer ces chaînes brutes comme s'il s'agissait de la même identité projet.

<a id="extrait-8"></a>

## Extrait 8

Les bornes historiques restent60 Kio avec réserve de métadonnées12 Kio. La projection humaine enrichie de noms est bornée à128 Kio. Le service T3 refuse une sortie tronquée, UTF-8 invalide, trop grande ou de version incompatible. Il ne montre pas une page partielle comme si elle était complète.

<a id="extrait-9"></a>

## Extrait 9

Étendre les unions et migrations de `rightPanelStore`, les titres, icônes, menus et commandes de `RightPanelTabs`, puis les branches de montage et boutons de `ChatView`. Les surfaces partagées de la page Pull requests doivent compiler et conserver leur comportement. Ajouter `BridgetPanel` dans le dossier composants natif T3.

<a id="extrait-10"></a>

## Extrait 10

Une seule surface Bridget par conversation T3 permet de sélectionner plusieurs fils Bridget. L'état de consultation est lié à environnement, conversation T3, fil Bridget et génération de requête. La recherche filtre seulement les données chargées. Un libellé indique cette portée. Aucun abonnement ni polling n'est requis.

<a id="extrait-11"></a>

## Extrait 11

Bridget : tests Rust ciblés sur protocole, fait de liaison, CLI, daemon et stockage. Comparer les données métier avant/après, sans exiger identité binaire SQLite/WAL ; compter zéro dispatch, read, ACK ou réveil. T3 : tests du service avec faux ProcessRunner, autorisation RPC et suites React du store, onglets et panneau. Tester le clavier, A → B, la fermeture pendant lecture et la pagination.

<a id="extrait-12"></a>

## Extrait 12

L'audit dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/reuse-audit.md` distingue les sources présentes des additions145. L'existence de lectures stockage ne prouve pas une API T3 existante. Les fichiers `ui.rs` supprimés ne sont pas des composants disponibles.
