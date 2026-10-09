# Plan technique — SPEC145

Date : 2026-10-07. Statut : Implemented, 20/20 tâches vérifiées. Audit validé et Converge2 CONVERGED à 16:24:21 UTC. Clôture documentaire hors Converge. Non installé, non activé.

## Contexte et contraintes

Documents et Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`, base `3bb89e0d`.
T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`, base `a1a4f2ef12`.
Branches isolées : `session-145-bridget-thread-viewer`. Aucun autre worktree, processus actif, mission, LaunchAgent, modèle ni installation n'est modifié.

Le préflight et les skills Specify, Plan, AuditExisting, Tasks, Analyze et Implement ont été lus par le principal. Les templates et helpers locaux attendus sont absents. Appliquer leur protocole documentaire sans prétendre une exécution de helper ni installer un outillage.

T3 utilise React, TypeScript, Zustand, Effect et son contrat RPC existant. Bridget utilise Rust et SQLite. Les prescriptions génériques Next.js, Axios, Pytest ou Cartae ne remplacent pas ces piles. Zéro dépendance nouvelle prévue. La livraison reste soumise à une autorisation distincte.

## Architecture retenue

1. La surface native `BridgetPanel` reçoit la référence de conversation et d'environnement T3 actuelle. Le store du panneau droit conserve seulement les informations de navigation natives ; les corps et credentials ne sont pas persistés.
2. Une RPC `bridget.read`, constante `WS_METHODS.bridgetRead`, porte trois actions de lecture fermées avec `AuthOrchestrationReadScope`. La connexion authentifiée choisit l'environnement. Le navigateur n'envoie ni agent Bridget ni chemin racine d'autorité.
3. Le service Effect T3 résout la conversation par `ProjectionSnapshotQuery.getThreadShellById`, puis le projet par `getProjectShellById`. Il utilise `project.workspaceRoot`. Le `worktreePath` décrit le contexte, pas la racine d'autorisation.
4. Le service appelle la CLI par `ProcessRunner` avec argv séparés, délai de six secondes et sortie limitée à 256 Kio. Aucun `sh -c`, contenu interpolé ni recherche réseau distante n'est introduit.
5. La CLI `bridget thread inspect` négocie la capacité `HumanThreadViewV1` sur une connexion IPC Client Unix locale. Son enum fermé autorise seulement list, show et history. La voie agent `ThreadRequest` garde ses règles actuelles et son refus du rôle Client.
6. Le daemon résout l'agent par un fait `T3ThreadBindingFact` détenu par la connexion primaire T3. Il vérifie la route, l'identité vivante, les UUID stables de fil et d'instance et la racine du projet T3 actif. La liaison fonctionne quand le modèle est dormant ; aucun PID MCP n'est requis.
7. Les lectures `list`, `show`, `history` et leur stockage restent réutilisés. L'appartenance est vérifiée à chaque lecture. Une projection humaine retire curseurs, réveils, credentials et métadonnées de transport. Les noms proviennent de `Store::agent_display_name`, sans génération ni résolution distante.

Ce canal local repose sur la confiance existante du compte système. Le fait de liaison ne revendique pas une authentification cryptographique entre deux processus du même compte. Le navigateur ne peut pas choisir l'agent. L'index de liaison est en mémoire et nettoyé à la déconnexion. Aucune table ni migration n'est ajoutée.

Le handler humain retourne avant `collect_closed_attach_views`, maintenance mutative. Protéger aussi les refus. La preuve de source T3 utilise un `HashSet` en mémoire `t3_project_connections`, car le tuple `communication_projects` perd cette source. Retirer cette preuve sur fait autre/invalide et à déconnexion. L'index connexion → liaison conserve une ambiguïté comme refus.

La CLI intercepte inspect dans `run()` avant `initialize_process` et `resolve_current_identity`. Utiliser `Namespace::from_environment` sans prepare et `DaemonConnection::connect` direct. Un daemon absent produit un refus, sans autostart ni création de namespace.

Résoudre l'exécutable par `T3CODE_BRIDGET_EXECUTABLE` via `HostProcessEnvironment`, puis le binaire utilisateur `/Users/moi/.local/bin/bridget` sur cette machine, ou `resolveCommandPath("bridget")` existant. Le chemin utilisateur générique est construit depuis HOME. Refuser `.cmd` et `.bat` Windows. Décoder un JSON métier sur exit2 ; ne pas exposer stderr brut.

## Identité et refus

La liaison primaire publie le fait uniquement après `Registered` et `report_project_context`. Le daemon accepte le fait si la route appartient à cette connexion, si l'identité vivante correspond à `stable_uuid(thread)`, si l'instance correspond à `stable_uuid(instance:thread)` et si `communication_projects.0` atteste le projet T3 actif. Ne pas accepter `last_known` comme autorité.

Comparer le projet après `resolve_communication_project(workspaceRoot,host,T3,None)`. Une racine Git réelle peut différer du chemin workspace. Ne pas comparer ces chaînes brutes comme s'il s'agissait de la même identité projet.

Les liaisons absentes, ambiguës, révoquées ou d'un autre projet produisent un refus explicite. Aucun inventaire global des agents ne sert de repli. La présence de deux fournisseurs dans un même projet ne donne pas accès aux fils de l'autre agent. Un ancien daemon sans capacité négociée produit un état incompatible, jamais un essai de `read` ou d'identité agent.

## Pagination et projection

Liste : ordre UUID croissant, `after_thread_id` exclusif, limite20 par défaut et100 maximum. Historique : séquence croissante, `from_seq` positif, limite50 par défaut et200 maximum. Réutiliser la borne `snapshot_seq` sous `to_seq` pour les pages suivantes. Un rafraîchissement démarre une nouvelle lecture cohérente. Ces positions de lecture humaines sont distinctes des curseurs agents.

Les bornes historiques restent60 Kio avec réserve de métadonnées12 Kio. La projection humaine enrichie de noms est bornée à128 Kio. Le service T3 refuse une sortie tronquée, UTF-8 invalide, trop grande ou de version incompatible. Il ne montre pas une page partielle comme si elle était complète.

Le panneau montre l'ordre attesté. Un type legacy absent apparaît sous « Message ». Les types attestés sont History, Action, Blocker et Decision. Le corps source reste exact ; sa copie utilise la chaîne source. Les membres et noms sont disponibles, jamais déduits des corps. La liste les enrichit par SQL groupé, avec au maximum16 membres par fil.

## Intégration native

Étendre les unions et migrations de `rightPanelStore`, les titres, icônes, menus et commandes de `RightPanelTabs`, puis les branches de montage et boutons de `ChatView`. Les surfaces partagées de la page Pull requests doivent compiler et conserver leur comportement. Ajouter `BridgetPanel` dans le dossier composants natif T3.

Une seule surface Bridget par conversation T3 permet de sélectionner plusieurs fils Bridget. L'état de consultation est lié à environnement, conversation T3, fil Bridget et génération de requête. La recherche filtre seulement les données chargées. Un libellé indique cette portée. Aucun abonnement ni polling n'est requis.

À fermeture, changement de contexte ou refus, annuler les travaux en cours quand possible et invalider leur génération. Toute réponse tardive est ignorée. Vider les données d'un contexte quitté ou devenu non autorisé. Le bouton Rafraîchir relit la liste et le fil courant sans émission, ACK ni tour modèle.

## Validation et gates

Avant les tâches : spec et checklist lisibles ; contrat fermé ; inventaire des sources exactes ; audit de réutilisation avec verdict de conception PASS. Le principal doit accorder le gate avant création de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/tasks.md`.

Avant implémentation : Analyze compare spec, plan, contrat et tâches. Les tests RED couvrent autorisation, non-mutation, anciens daemons, bornes, texte exact et contexte tardif. Ils n'utilisent pas le daemon actif.

Bridget : tests Rust ciblés sur protocole, fait de liaison, CLI, daemon et stockage. Comparer les données métier avant/après, sans exiger identité binaire SQLite/WAL ; compter zéro dispatch, read, ACK ou réveil. T3 : tests du service avec faux ProcessRunner, autorisation RPC et suites React du store, onglets et panneau. Tester le clavier, A → B, la fermeture pendant lecture et la pagination.

Exécuter ensuite format, lint, typecheck et build selon les scripts existants vérifiés. Conserver commandes, compteurs et échecs réels dans la recette. L'aperçu isolé est désormais autorisé par l'utilisateur ; son ouverture about:blank ne prouve pas la recette. Aucune recette sur application active. Revendiquer une recette native seulement après observation de la surface isolée.

## Réutilisation et risques

L'audit dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/reuse-audit.md` distingue les sources présentes des additions145. L'existence de lectures stockage ne prouve pas une API T3 existante. Les fichiers `ui.rs` supprimés ne sont pas des composants disponibles.

Risque principal : le rôle Client pourrait recevoir trop de capacités si `ThreadRequest` est réouvert globalement. L'enum humain séparé et la matrice exhaustive du daemon empêchent cette dérive. Deuxième risque : le changement de contexte UI pourrait publier une réponse ancienne. Les clés et générations explicites doivent le prévenir et les tests le reproduire. Troisième risque : une page enrichie pourrait dépasser sa borne. La sérialisation et le décodage doivent appliquer leurs plafonds avant publication.
