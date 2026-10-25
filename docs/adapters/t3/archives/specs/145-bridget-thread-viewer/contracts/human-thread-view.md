# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/contracts/human-thread-view.md](../../../../../../../specs/145-bridget-thread-viewer/contracts/human-thread-view.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Décision principale figée pour les tâches. RPC T3 : `bridget.read`, constante `WS_METHODS.bridgetRead`.

<a id="extrait-2"></a>

## Extrait 2

Le daemon ne fait pas confiance à un `agent_id` fourni par le client. Il n'existe aucun tel champ dans ce contrat. Il résout `T3ThreadBindingFact{version:1,t3_thread_id}` détenu par la connexion primaire. Vérifications : route.conn égale cette connexion ; identité vivante ; agent égale `stable_uuid(thread)` ; instance égale `stable_uuid(instance:thread)` ; projet actif `communication_projects.0` de source T3. Canoniser le projet avec `resolve_communication_project(workspaceRoot,host,T3,None)` ; racine Git et workspace ne sont pas toujours identiques. `last_known` n'est pas une autorité. Deux liaisons admissibles produisent un refus ambigu. À déconnexion, retirer le fait et sa preuve source T3 en mémoire.

<a id="extrait-3"></a>

## Extrait 3

Le handler humain retourne avant `collect_closed_attach_views`. Aucune maintenance mutative annexe ne précède lecture ou refus. Erreurs métier fermées : `unsupported_version`, `binding_unavailable`, `project_mismatch`, `thread_unavailable`, `invalid_request`, `storage_unavailable`, `response_too_large`. Une erreur sans sujet attesté peut avoir subject null. Les erreurs opérationnelles CLI/T3, dont daemon absent et délai, restent distinctes.

<a id="extrait-4"></a>

## Extrait 4

Commande : `bridget thread inspect --t3-thread UUID --project-root chemin --action list|show|history ... --json`. Les options de pagination correspondent strictement à l'action. Elle utilise le rôle Client et négocie la capacité avant la demande.

<a id="extrait-5"></a>

## Extrait 5

Le service T3 lance des arguments séparés. Il n'utilise aucun shell. Il fixe un délai de six secondes et un plafond de sortie256 Kio. Échec de commande, daemon absent, délai, dépassement, UTF-8 invalide, JSON invalide et version incompatible deviennent des erreurs distinctes utiles. Les diagnostics ne journalisent ni corps ni credentials.

<a id="extrait-6"></a>

## Extrait 6

Résolution d'exécutable : `T3CODE_BRIDGET_EXECUTABLE` via `HostProcessEnvironment`, puis binaire utilisateur `/Users/moi/.local/bin/bridget` sur cette machine ou `resolveCommandPath("bridget")`. Construire le chemin générique depuis HOME. Refuser `.cmd` et `.bat` Windows. Décoder un JSON métier valide sur exit2 ; ne pas exposer stderr brut.

<a id="extrait-7"></a>

## Extrait 7

Accord opérationnel : exit3 signifie daemon absent ou namespace indisponible. Le service T3 le projette en état `unavailable`, sans parser stderr. Exit2 conserve le JSON métier existant. Aucun nouveau code d'erreur métier ni repli d'identité n'est ajouté.

<a id="extrait-8"></a>

## Extrait 8

Une méthode native `bridget.read` porte les trois actions fermées. Elle transmet la conversation T3 sélectionnée et la pagination/désignation du fil Bridget. Le groupe RPC applique `AuthOrchestrationReadScope`. La connexion authentifiée route l'environnement. Le serveur résout conversation et projet via `ProjectionSnapshotQuery`, puis choisit `project.workspaceRoot`.

<a id="extrait-9"></a>

## Extrait 9

Toute réponse est liée à environnement, conversation T3, fil Bridget et génération. Changer ou fermer invalide la génération. Un refus vide les données non autorisées. La recherche agit localement sur les pages chargées. Rafraîchir relit seulement liste/détail/historique de la sélection courante et repart avec un nouveau snapshot. Il n'y a ni abonnement ni polling.
