# Audit de réutilisation de l’existant — 155-demandes-enfant

Date : 2026-10-12. Gate documentaire : PASS,5/5. Aucun test du parcours forward155 attesté ici.
Dossier audité : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant.
Les ancres de code ci-dessous partent de /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant.

Le présent passage remplace les ancres150 et les compteurs anciens. Les20items
ci-dessous ont été confrontés à l’existant par recherche `rg` et lectures ciblées.
Il ne conclut pas à une validation de l’implémentation155.

| # | Item du plan | Preuve fraîche | Décision |
|---|---|---|---|
|1|T000 et réponses d’approbation|`crates/bridget-transport/src/codex_app_server.rs:2195`, appel de production autour de3344, test existant `approval_response_choisit_accept_ou_decline_selon_permissions`|Réutiliser le point unique `approval_response`. La prélecture a trouvé le repli première offre ; la relecture voit déjà le correctif de l’autre owner. Aucun second sélecteur ; preuve RED/GREEN et livraison restent à consigner par root.|
|2|Boîte humaine|`crates/bridget-daemon/src/daemon.rs:9916`, `human_inbox.rs:88–99`|Réutiliser `Deposited` et `deposit_and_notify`, faire rendre le résultat au déposant daemon.|
|3|Lecture et décision humaines|`human_inbox.rs:189,237,308,333`|Réutiliser get par identifiant exact, resolve, ack et close_self.|
|4|Reliaison d’un dépôt après panne|`human_inbox.rs:115,185,275–291`|Ajouter une lecture par clé exacte : la déduplication existante vise les items ouverts ; le balayage plafonné à 200 ne retrouve pas un item ancien résolu.|
|5|Pause|`crates/bridget-daemon/src/referent_control.rs:115,129–137`|Réutiliser `AutonomousEffect::NativeDelegation` et `admit_autonomous_effect`, pas de seconde garde.|
|6|Propriétaire|`crates/bridget-daemon/src/daemon/native_delegation.rs:319,355`|Réutiliser can_own_task/recover_owner pour Respond.|
|7|Avis système idempotent|`daemon/native_delegation.rs:1639,2268–2270`|Réutiliser send_as et l’identifiant stable, séparer les faits du texte enfant.|
|8|Droits figés|`crates/bridget-transport/src/native_permissions.rs:23–31`, `crates/bridget-daemon/src/native_permissions.rs:328`|Réutiliser le snapshot parent, sans relire les droits du parent vivant.|
|9|Politique et délai de mission|`daemon/native_delegation.rs:130,1468,2223`|Réutiliser mission_deadline_at et CHILD_POLICY_ENV, pas de nouvelle politique de droits.|
|10|Routage vers le wrapper|`daemon.rs:9668`, `wrapper.rs:4481`|Réutiliser le patron ControlExecutionDispatch, sa connexion et son instance attestée.|
|11|États et transitions d’exécution|`execution_store.rs:952–954,1260`, `wrapper.rs:5506`|Réutiliser waiting_approval/waiting_user_input et publish_execution_transition. Le retour running reste à prouver en T016.|
|12|Tick|`daemon.rs:5110`, `daemon/native_delegation.rs:2133`|Réutiliser le tick existant pour expiration et renvoi.|
|13|Attente151 livrée|`daemon/native_delegation.rs:692–764`, WaitEpoch712 et résultat755 ; checkpoint AGENTS151 livré|T034 applicable : réutiliser bridget_task_wait et WaitEpoch. Conserver version/timed_out/waited_secs/ready/pending.|
|14|Purge d’arbre|`crates/bridget-daemon/src/delegation_lineage.rs:146,200`|Réutiliser for_child et purge_trees ; supprimer les demandes avec l’arbre en T033.|
|15|Outils MCP|`wrapper.rs:5031–5056,5061,5265`, `delegation_mcp.rs:65–78`|Ajouter Respond :24→25 outils natifs,7→8 outils delegation_mcp ; préserver wait,ack,continue et request_wait.|
|16|Interface de session|`crates/bridget-transport/src/managed_session.rs:109,148,277` ; recherche ProviderRequest/answer_provider_request|Étendre l’interface existante ; aucun événement corrélé équivalent trouvé.|
|17|Observation et attente ancienne|`wrapper.rs:5700,5705,5741`|Ne pas réutiliser publish_approval_wait (limite 3) ni permission_required après coup pour une attente fournisseur corrélée.|
|18|Table native_child_requests et module daemon|Recherche des noms dans crates ; les données corrélées n’existent pas dans human_inbox|Créations prévues justifiées par réception, arbitrage, livraison, expiration et reliaison ; une table et un module, pas de moteur parallèle.|
|19|Cinq trames ChildRequest* et outil Respond|Contrat155, ManagedSession existante, protocol existant|Pas d’équivalent qui transporte une demande, sa réponse, son accusé et sa fermeture corrélés. Cinq trames nommées en T003 ; un outil en T025.|
|20|Classification de droits|native_permissions snapshot et child_policy existants ; FR011/T009|Fonction pure distincte : verdict d’une demande, pas calcul de politique. Chemins cités et normalisés lexicalement seulement ; aucune résolution filesystem.|

## Recherche de ce passage

`rg` a recherché approval_response et ses appelants, BRIDGET_SAFE_MCP_TOOLS,
les noms de delegation_mcp::tools, wait_tasks/WaitEpoch, can_own_task/recover_owner,
CHILD_POLICY_ENV, mission_deadline_at, ControlExecutionDispatch, tick,
purge_trees/for_child, les dépôts et lectures human_inbox, les transitions,
ManagedWaitState/ManagedEventKind/ManagedSession et les noms proposés.
Les blocs indiqués ont été lus. La relecture distingue le correctifT000 en cours
chez l’autre owner d’un résultat de test ou d’une livraison : aucun n’est déduit.
Les contrats151 et les checkpoints150/151 livrés ont été consultés.
Les formes du fournisseur, ses champs d’effet et ses durées restent à lire en T002.

## Règles et arbitrages conservés

La constitution globale XVIII/XIX impose des créations justifiées et des boucles
bornées. Les décisions027/149 imposent la garde unique et les droits hérités.
Les specs148–154 sont livrées ;151 fournit déjà l’attente,150 le délai et la purge.
T000 est la seule exception indépendante aux gates T001/T002 : livraison seule
préalable. Les autres parcours commencent après la référence T001 et le contratT002.
Le type humain intervention_required reste réutilisé. Claude et ACP restent hors
périmètre. Aucun nouveau sélecteur de réponse, outil d’attente ou service.

## Gate avant tasks — revalidation fraîche

- [x] Aucune duplication évidente non arbitrée : point unique approval_response et attente151 réutilisés.
- [x] Chaque item extrait du plan a une ligne d’audit :20/20 ci-dessus.
- [x] Règles applicables relues : constitution XVIII/XIX, garde027 et droits149.
- [x] Specs proches vérifiées : acquis150 et attente151 dans les sources livrées.
- [x] Divergences du plan corrigées : cinq trames, 24→25 et7→8outils, T000 indépendant, chemins lexicaux.

PASS 5/5 documentaire ; gates d’exécution T001/T002 ouverts. Aucune tâche cochée.

## Reçu T000 transmis par root après la prélecture

T000 ciblée : 3 PASS, 0 FAIL, RC0, 3,79 s ; 207 sources stables. SHA256 du
transport : 27a2a44928ff3ecd39c260902f0e3f7580276a8f7496bc36b6e760b722162055.
Claude T000 R1 : APPROVE_WITH_CHANGES, sans blocage. Limite LOW acceptée pour
le correctif isolé : permission_summary affiche « décision absente » pour refused
(attach.rs:3739), sans accord de permission ; erreur JSON-RPC -32000 correcte.
L’offre absente doit être refusée selon T000 ; la compatibilité fournisseur réelle
reste à établir en T002. Aucun succès live ni du reste de155 n’est déduit.
Tâches figées non cochées ; le reçu ciblé ne vaut pas livraison de T000.

## Revalidation ciblée après contre-revue plan R1

Lectures supplémentaires : mcp_elicitation_response dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-transport/src/codex_app_server.rs:2317,
wait_tasks dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs:692,
et transition_if_current dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/execution_store.rs:1260.
Le schéma généré fournisseur elicitation/approval/_meta reste NON LU (T002 ouvert).

Items 11/13/16/18 précisés : CAS SQL avec rowcount, anneau et verrou existants,
fermeture Cancel→Closed même none/pending, replay de réponse persistée et
revalidation avant redelivery. WaitEpoch et attente d’état151 restent réutilisés :
ready terminal inchangé, pending_requests racine Parent seulement. Aucun moteur
ni mémoire persistante de waiter, aucun sélecteur ni outil nouveau supplémentaire.
Les questions déjà ouvertes restent actionnables ; aucune consommation artificielle.

Gate documentaire maintenu 5/5 ; les mêmes 41 tâches, toutes non cochées. Les
scénarios existants T014/T023/T027 et la relecture T038 portent les précisions ;
aucune nouvelle série de tests. Aucun runtime forward155 validé ici.

Checkpoint après le gel documentaire des tâches : root a livré et coché T000
seule. 41 identifiants conservés, 1/41 cochée, 40 ouvertes. Aucun cochage par
l’owner documentaire ; aucune réécriture tasks après son gel. Les mentions 0/41
ci-dessus décrivent la prélecture avant ce cochage. Le reste forward155 n’est
pas validé par cette livraison ciblée.

## Checkpoint T001 et schéma 0.162.1 — remplace les réserves NON LU antérieures

Root a acquis T001 : 180 PASS/0 FAIL, 207 sources stables, et coché T000/T001
seules. État courant : 2/41, 39 ouvertes. Le schéma réel 0.162.1 est désormais lu
sur les méthodes ciblées ; 0.161.0 reste une hypothèse historique. Source primaire :
/Users/moi/.cache/bridget-delivery155.um4xccv4/codex-schema155-0.162.1.
Le contrat détaillé et son reçu sont écrits par l’autre owner. T002 reste ouverte
jusqu’à la revue finale root ; T003 attend le contrat figé. Aucun test forward réel.

FileChange Params n’a ni paths ni availableDecisions. grantRoot vise une portée
session à l’effet incertain : v1 ferme entièrement fileChange avec
approval_method_unsupported, duration_unsupported si grantRoot non nul.
Les notifications changes/path/diff/kind existent et ont été lues : faits cités
et piste future seulement, aucun cache/corrélation supplémentaire ou route
Human réelle inventée. Classifier synthétique facts complets/once/DF→Human
conservé ; aucune limitation définitive du fournisseur affirmée.

permissions a une réponse permissions/scope(turn ou session)/strictAutoReview,
pas decision ; refus JSON-RPC -32000 child_request_refused, motif
approval_method_unsupported. user_input a exactement
{answers:{questionId:{answers:[string]}}}, adaptation du mapping parent requise ;
aucun decline/cancel natif, refus et expiration par erreur JSON-RPC fermée.
Elicitation définit cinq modes : form,openai/form,openaiForm,url,openai/userVerification.
_meta est JSON arbitraire, sans clé approval/durée définie. Aucun form MCP tiers n’est routable Parent : absence de preuve structurée non-secret/non-approval, refus v1 approval_method_unsupported ; enrichis/URL/
_meta non vide/tiers vide fermés. Exception vide Bridget distincte.
resolved contient threadId/requestId, pas turnId ; table existante, aucune clé inventée.

Relecture des mêmes tâches : T012/T013 n’exigent aucun événement forward
fileChange, T014/T015 adaptent les réponses par méthode, T022 refuse user_input
par JSON-RPC, T006/T008 restent le classifier pur. Aucun nouvel identifiant de
tâche, outil ou série de tests. Analyze documentaire cohérent sous gate T002
encore ouvert ; reuse gate 5/5 maintenu. T000/T001 seuls cochés par root.

Relecture finale T002 : contrat de l’autre owner lu intégralement, SHA256
58a748ed031f4fca0f401e1ed44452230ecc124abeb47c3e1d87b626b90f4e9f ;
reçu schema155.md lu, SHA256
baed253c69e49027f355154254dea6c68e07a91dee90e9968c033ebb2a5aceb9.
Les réponses fileChange/permissions/user_input refusées sont JSON-RPC -32000
child_request_refused, sans result. MCP form n’a pas d’isSecret standard ; son
absence ne suffit pas à prouver une question non secrète. L’erreur seule ne
prouve pas une fermeture runtime fournisseur. A1 formes est résolue par lecture
primaire et refus conservateurs ; T002 attend le GO final root, non cochée ici.
Aucune action sur le code ou les deux fichiers de l’autre owner.

## Arbitrage primaire final T002 — formulaire MCP tiers fermé en v1

Aucune forme MCP ne fournit une preuve structurée de non-secret et de
non-approbation. Même bool/enum peut accepter un outil. FR013 et le gate T002
imposent donc approval_method_unsupported pour TOUT formulaire MCP tiers en v1,
même form typé, complet, non vide et sans _meta. Aucun heuristique textuel,
marqueur inventé ou route Parent. L’exception historique vide Bridget reste
distincte. user_input seul remonte au parent, avec isSecret normatif défaut false ;
isSecret=true reste secret_not_forwardable. Les modes URL restent refusés.

Les mêmes scénarios T014 sont des refus MCP et des réponses user_input ; T015
applique cette frontière, sans nouvel identifiant ni nombre de tests/scénarios.
41 tâches conservées, T000/T001 seules cochées par root ; T002 attend le GO final.
Les mentions antérieures de formulaire candidat Parent sont supersédées.
Aucun code, helper/API, test exécuté ou commande Cargo/Git/service.

État actuel après clôture du gate : T002 est final PASS et cochée par root, attestée par /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant/validation/gate-schema155.json. Les mentions de T002 ouverte ou en attente ci-dessus sont les checkpoints antérieurs. Protocole/store en validation ; aucun GREEN global déclaré. T037 attend l’implémentation réelle.

## Réutilisation ciblée T023 — preuve physique et demandes fournisseur

Recherche par nom `settle_after_stop` et responsabilité : aucun règlement de callback155 après arrêt OS existant ; `close_lost_execution` traite la feuille d'exécution, pas ce callback. Création interne ciblée, justifiée par FR026/FR027b, sans framework, table, état ni outil neuf.
Réutiliser les deux preuves existantes : arrêt149 (Stopped/StoppedForced/NotFound puis cancel_stopped durable) et coldrestart152 (groupes fermés avant prepare_restart), dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_delegation.rs.
Créer `settle_after_stop` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs. Réutiliser for_task borné32, invalidate, settle_sent_child_request, confirm_child_closed, get/ack/close_self exacts. Delivered/unknown restent inchangés ; sent non confirmé devient unknown ; before_apply exige sent_at=None et arrêt physique prouvé, sans fabriquer ChildRequestClosed.
Le règlement négatif d'inbox précède cleanup ; il n'interprète aucun accord humain. Les tests T022/T028 et la relecture T038 déjà prévus portent cette correction, sans nouveau scénario. Les exécutions restent réservées au principal ; aucun GREEN global annoncé ici.

## Justification de réutilisation T038 — helpers et présentation

Relecture ciblée des sources gelées : due_child_requests dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/delegation.rs:490
reste l’API T005 requise et testée pour l’échéance (199 vide, 200 dû). Le tick utilise
child_request_followups (:423), car il doit aussi visiter sent et none/void
non confirmés ; remplacer cette requête par les seules open échues perdrait
les renvois et clôtures. Le prédicat OR parcourt l’index existant : coût
O(n + B), n entrées parcourues et B données projetées. La limite 512 borne les
lignes retournées, pas le travail de scan potentiel complet. Aucun doublon de
responsabilité à retirer ici.

format_cited_paths dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/native_permissions.rs:212
est le helper pur T009, testé sans filesystem : 64 valeurs, 4096 octets UTF-8,
contrôles filtrés, phrase d’affichage et signal de coupe. Le core155, dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs:422,
construit un tableau JSON de chemins normalisés lexicalement : 64 valeurs,
budget local 1024 octets, puis contrôle global 4096 octets de facts_json. quoted_text
(:13) remplace les contrôles et borne sur frontière UTF-8 ; paths_verified=false.
Aucune résolution filesystem ni preuve de confinement n’est produite.

Les deux présentations ont des formes et budgets distincts. Un raccord expansif
du formatter au core ne supprime ni normalisation lexicale ni borne du JSON.
La duplication légère de présentation est conservée et justifiée par ces
contraintes ; aucun helper/API supplémentaire. Les deux warnings dead_code
restent visibles et non bloquants, sans revendiquer un lint vert.
La revue complète T038 et la capture du patch par root restent ouvertes.

## Limite de réutilisation SendAs — FR018

SendAs est réutilisé avec ses gardes149, notamment le disjoncteur. L’avis Parent
est tenté, pas garanti. Dans la fixture de budget, root a observé le refus du
neuvième avis (limite 8 échanges / 180 s). Le libellé initial FR018 « Une question ouvre
un avis » était plus fort que ce comportement ; la spec décrit désormais la
tentative et la limite, sans supprimer la garde ni ajouter un nouvel outil.

La lecture de publish_parent (:278/301) et receive (:500) dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/crates/bridget-daemon/src/daemon/native_child_requests.rs
confirme l’insertion durable avant l’appel SendAs. Le refus n’efface pas la
demande ; Status reste sa lecture propriétaire, Wait la rend si Parent ouverte
non expirée/actionnable. Aucun succès d’avis n’est déduit de la seule présence
de la demande. Le coût O(n+B) du suivi reste inchangé.
