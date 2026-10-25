# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/reuse-audit.md](../../../../../../specs/145-bridget-thread-viewer/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Source absolue | Présence et responsabilité | Réutilisation / adaptation |
| --- | --- | --- |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/threads.rs` | list458, show485, history754 inspectés directement. | Règles et bornes réutilisées ; projection humaine à ajouter. La sortie show brute contient ACK/wake, donc non exposable. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs` | list1179, show1216, history1385 inspectés directement. | Autorisation membre et pagination ASC réutilisées. Aucun appel thread_read ni ACK. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs` | Refus Client11410 et branche agent11638 inspectés directement. | Matrice exhaustive et négociation à étendre avec nouvelle capacité fermée ; ne pas rendre ThreadRequest agent admissible globalement. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/t3code_identity.rs` | refresh54 inspecté directement. | Discipline d'attestation et refus conservée ; preuve PID MCP non requise pour humain dormant. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/t3code.rs` | Enregistrement primaire1277 attesté par principal et lecture ciblée. | Ajouter fait T3ThreadBindingFact après Registered et contexte projet, sans nouveaux credentials. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/rightPanelStore.ts` | Unions22/42, singleton181 et migration349 inspectés. | Ajouter surface Bridget ; ne pas persister corps/messages. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/RightPanelTabs.tsx` | Props91, menus339, titres613 et intégration682/871 identifiés ; extraits titres et menus inspectés. | Ajouter icône b neutre, titre, action et fermeture native. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/components/ChatView.tsx` | Points4565/9633/10269/10326 reçus de l'exploration principale. | Branches de montage et contrôles natifs à étendre ; pas nouvelle application. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/web/src/routes/_chat.pull-requests.tsx` | Point2080 transmis par principal. | Couvrir les nouvelles props de RightPanelTabs sans créer une consultation hors contexte autorisé. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/contracts/src/rpc.ts` | Déclaration WS_METHODS289 inspectée. | Ajouter les seules lectures typées Bridget ; aucune API Bridget initiale. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/auth/RpcAuthorization.ts` | Map exhaustive24 inspectée. | Couvrir AuthOrchestrationReadScope et tests de refus. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/ws.ts` | Handlers natifs inspectés ; montage3389 transmis par principal. | Injecter service Effect et une RPC bridget.read avec trois actions fermées ; serveur résout le contexte. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/orchestration/Services/ProjectionSnapshotQuery.ts` | getThreadShellById et getProjectShellById confirmés directement. | Résoudre conversation et racine projet autoritative. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/apps/server/src/processRunner.ts` | Interface argv/délai/output et service142 inspectés. | Réutiliser exécution bornée ; pas shell ni dependency de processus. |

<a id="extrait-2"></a>

## Extrait 2

| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-transport/src/protocol.rs` | Chemin confirmé par rg ; matrice protocole transmise par principal. | CRÉER nouvelles variantes/capacité HumanThreadViewV1 dans le protocole existant. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/communication/client.rs` | Connexion Unix, rôle Client et négociation inspectés directement. | RÉUTILISER DaemonConnection et budget ; CRÉER négociation humaine et exchange fermé. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/cli.rs` | Chemin confirmé ; run/initialisation inspectés par worker et reçu principal. | CRÉER inspect et retour avant initialisation/identité ; pas autostart. |
| `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/ledger_requests.rs` | agent_display_name93 inspecté directement. | RÉUTILISER nom local Option ; aucun enrichissement de modèle. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime/src/state/orchestration.ts` | Requêtes environnement typées inspectées directement. | RÉUTILISER pattern de query native ; CRÉER lecture Bridget. |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/packages/client-runtime/src/state/projectCommands.ts` | Factory des queries58 inspectée directement. | RÉUTILISER routage d'environnement et contrôles de query, sans commandes mutation. |

<a id="extrait-3"></a>

## Extrait 3

CRÉER le protocole Client `HumanThreadViewV1`, son fait primaire `T3ThreadBindingFact`, les deux index en mémoire de liaison/source T3, la CLI inspect, une projection humaine, un service Effect T3, une seule RPC `bridget.read` et `BridgetPanel`. Aucune voie humaine équivalente n'existe au point de départ. Ces additions sont nécessaires pour séparer confiance humaine et rôle agent.

<a id="extrait-4"></a>

## Extrait 4

RÉUTILISER le transport Client Unix, la négociation, les lectures list/show/history, le contrôle membre, les limites, les noms locaux, les résolutions serveur T3, le ProcessRunner borné, les factories de requête d'environnement, le store et les onglets natifs. Aucune table, migration, framework ou dépendance nouvelle. Aucune alternative équivalente n'impose un arbitrage utilisateur.

<a id="extrait-5"></a>

## Extrait 5

- [x] Besoin utilisateur, exigences FR145 et checklist présents.
- [x] Recherche principale reçue et validations primaire frontend/tests/sécurité consignées.
- [x] Tous les chemins inventoriés sont exacts ; le client-runtime réel est sous packages, pas apps/web/src/lib.
- [x] Lectures existantes, sorties sensibles et rôle Client actuel vérifiés.
- [x] Aucune voie humaine ou API Bridget T3 équivalente n'est prétendue existante ; ui.rs supprimé exclu.
- [x] Contrat fermé, version, enum d'erreurs, noms, bornes et une seule RPC fixés.
- [x] Réutilisations et créations sont explicites ; zéro dépendance et zéro table nouvelles.
- [x] Identité dormant, source T3 canonisée, refus ambigu et confiance OS existante documentés.
- [x] Lecture avant maintenance mutative et CLI avant initialisation/autostart incluses au plan.
- [x] Plan, modèle, contrat et recette alignés ; aucune question fonctionnelle ou arbitrage équivalent bloquant.
- [x] Verdict conception PASS ; le GO principal reste nécessaire pour produire les tâches.

<a id="extrait-6"></a>

## Extrait 6

Préparation des essais T3 — RÉUTILISER les dépendances déjà installées au moyen d'overlays node_modules privés145. Cinq liens de workspace sont remplacés et trois dossiers shared/ssh/tailscale sont préparés selon le reçu principal. Inspection documentaire directe : les liens contracts/runtime et shared/ssh/tailscale des overlays pointent vers `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`. Les imports Node145 sont prouvés par le principal. Aucun arbre principal ni installation active n'est annoncé modifié. Le reçu détaillé et ses limites restent dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/validation.md` ; ces préparations ne prouvent pas les tests produit.
