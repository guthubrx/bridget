# Analyze — Session155, prélecture du 2026-10-12

Périmètre : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/155-demandes-enfant/specs/155-demandes-enfant.
Analyse croisée de spec, plan, tasks, data-model, research, contrat, quickstart et
audit de réutilisation. Les contre-revues Codex du 11 octobre restent historiques.
Statut courant In Progress, 4/41 ; T000/T001/T002/T036 cochées par root. Les paragraphes0/41 et NON LU ci-dessous sont les checkpoints antérieurs au dernier passage T002 ; aucun test du parcours forward155 attesté ici.

## Corrections prouvées

| Écart | Correction documentaire | Preuve |
|---|---|---|
| T000 opposé au gate avant tout code | Seule exception indépendante ; correctif et livraison seuls préalables ; T001 puis T002 avant le reste | FR003 et T000 ; point unique approval_response, transport2195 |
| T003 ne nommait que trois trames | Les cinq sont nommées avec leur direction, y compris ChildRequestCancel et ChildRequestClosed | Contrat155 et tâches T016/T023 déjà requis |
| Chemins résolus contre absence de filesystem | FR019/T007/T018/data-model : citation et normalisation lexicale seulement | FR011/T009 : aucune résolution filesystem, écriture workspaceWrite refusée |
| Compteurs MCP hérités | Native24→25, delegation_mcp7→8 ; préserver wait/ack/continue | wrapper5031–5056 et delegation_mcp65–78 lus |
| T006/plan seulement20cas | Au moins24 cas conformément au critère existant | SC002 et data-model ; aucun scénario de confort ajouté |
| Ancien A2 :151 absente | Résolu ; T034 applicable et réutilise WaitEpoch, forme151 conservée | native_delegation692–764, résultat755 ; checkpoint151 livré |
| Recette manuelle mal numérotée | Quickstart T040 et validation explicite utilisateur maintenue | T040 existante |
| Commandes sur WT150 | Commandes actuelles sur WT155 dédié ; preuves historiques conservées | Branche session-155-demandes-enfant et dossier présent |
| Ancres de réutilisation anciennes | Recherches et lectures fraîches,20items, point unique T000 | reuse-audit du présent passage, gate 5/5 |

La relecture du point T000 voit déjà l’édition de l’owner transport. Elle ne prouve
ni test réussi ni livraison. Le statut documentaire reste0/41.

## Couverture —33 FR, 14 SC, 7 US

| Exigences fonctionnelles | Tâches prévues |
|---|---|
| FR001–004 | T000,T003,T010,T011 |
| FR005–008 | T004,T012–018 |
| FR009–013 | T000,T006–009,T012,T013,T017,T018,T024 |
| FR014–017 | T024,T025,T027 |
| FR018–022 | T019,T020,T026,T028,T029,T032 |
| FR023–027 et FR027b | T016,T018,T021–023,T029 |
| FR028 | T005,T017 |
| FR029 | T030,T031 |
| FR030–031 | T005,T018,T033 |
| FR032 | T036,T037 |

FR027b est l’identifiant additionnel ; il n’existe pas de FR027a.

| Critère de succès | Tâches prévues |
|---|---|
| SC001 aucun accord sans décision humaine et remise valide | T012,T017,T021,T028 |
| SC002 table d’au moins 24 cas | T006–009 |
| SC003 décisions fermées | T000,T012,T013 |
| SC004 issues et expiration au tick | T016,T021–023,T029 |
| SC005 compatibilité sauf T000 explicite | T003,T010,T039 |
| SC006 chaîne de trois niveaux | T030 |
| SC007 transport non bloquant | T012,T013,T016 |
| SC008 purge | T033 |
| SC009 remise, annulation, reconnexion et issue unknown | T016,T021–023 |
| SC010 décisions derrière250 anciens items | T028,T029 |
| SC011 bout en bout isolé | T035 |
| SC012 échéance à la remise | T021,T022,T029 |
| SC013 refus des décisions durables dans les deux modes | T000,T012,T013 |
| SC014 fichiers workspaceWrite refusés | T008,T009 |

| Story | Tâches prévues |
|---|---|
| US1 questions | T014,T015,T019,T024,T025 |
| US2 approbations et cycle de vie | T012,T013,T016–023,T028,T029 |
| US3 limite absolue | T006–009 |
| US4 délai et fermeture | T022,T023,T027,T033 |
| US5 chaîne et absence d’élévation | T030,T031,T032 |
| US6 état et attente | T026,T034 |
| US7 compatibilité | T010,T011 |

T000–T040 :41 identifiants uniques,41 cases ouvertes. T038 revue,T039 suite et
T040 recette ferment la validation ; la recette exige l’accord explicite utilisateur.

## Réserves ouvertes, sans incohérence documentaire bloquante

| Id | Nature | Suite prévue |
|---|---|---|
| A1 | Schéma 0.162.1 ciblé désormais lu ; contrat et revue finale T002 ouverts | Méthodes inconnues ou non sûres refusées ; T003 attend le GO root |
| A3 | Transition waiting_approval→running et validité concurrente de remise non prouvées par cette lecture | T016,T021–023 : tests prévus à exécuter |
| A4 | Sources150 livrées présentes ; baseline155 root 180 PASS/0 FAIL | T001 acquise et cochée par root |
| A5 | Écritures confinées refusées en v1, gain fonctionnel limité | Limite explicite FR011 ; aucune résolution filesystem ajoutée |
| A10 | T000 change intentionnellement le repli deny préexistant | Exception FR003, RED/GREEN et livraison seuls à consigner par root |
| A11 | Wrapper disparu après sent : annulation non prouvable | Issue unknown explicite, aucune fausse confirmation |
| A6 | Parent dangerFullAccess : effets d’une commande non confinés par nature | Limite héritée, documentée ; pas de nouveau droit |
| A7 | Fenêtres600/3600 s initiales | Constantes uniques, plafonnées par mission_deadline_at |

A2 est résolu par l’existant151. Le risque d’un WT150 partagé est retiré des
commandes actuelles ; le travail155 reste sur son WT dédié. Les choix utilisateur
déjà inscrits dans la spec sont conservés, sans nouvelle exigence.

## Relecture et verdict

Une seconde lecture a confronté le présent tableau à spec/plan/tasks/contrat et
quickstart : cinq trames, compteurs 24/25 et 7/8, minimum24 cas, T000 indépendant,
T034 applicable, chemins lexicaux, T040 humaine et 0/41 cohérents. Les listes
numérotées des contre-revues antérieures sont des checkpoints de conception,
pas des preuves d’implémentation. La neutralisation des objections est prévue par
les tâches ; elle sera validée par les tests, pas par ce rapport.

Analyze documentaire : PASS après corrections. Reuse gate :5/5,20/20 items.
Aucune conclusion GREEN, livraison ou convergence formelle. Le reste de155
commence après T001 puis T002 ; T000 garde son correctif et sa livraison séparés.

## Reçu T000 transmis par root après la prélecture

T000 ciblée : 3 PASS, 0 FAIL, RC0, 3,79 s ; 207 sources stables. SHA256 du
transport : 27a2a44928ff3ecd39c260902f0e3f7580276a8f7496bc36b6e760b722162055.
Claude T000 R1 : APPROVE_WITH_CHANGES, sans blocage. Limite LOW acceptée pour
le correctif isolé : permission_summary affiche « décision absente » pour refused
(attach.rs:3739), sans accord de permission ; erreur JSON-RPC -32000 correcte.
L’offre absente doit être refusée selon T000 ; la compatibilité fournisseur réelle
reste à établir en T002. Aucun succès live ni du reste de155 n’est déduit.
Tâches figées non cochées ; le reçu ciblé ne vaut pas livraison de T000.

## Analyze après contre-revue plan R1 — 2026-10-12

Les précisions demandées sont synchronisées dans spec/plan/tasks/data-model/
contrat, sans nouvel identifiant de tâche ni cochage. T002 reste ouvert : schéma
elicitation tiers/approval/_meta NON LU, refus fermé des formes non établies.
T006 garde au moins 24 cas ; T014 étend les scénarios déjà prévus.

La relecture distingue CAS de décision open+deadline et claim pending+FR027,
avec rowcount ; fermeture fournisseur non confirmée Cancel→Closed même
none/pending ; void ne vaut pas confirmation ; expiration/fermeture avant chaque
redelivery sent et replay terminal de réponse persistée. recent_requests expose
delivery_state. FR017/T027/T029 : humain pendant pause acquitté sans exécution,
reprise sur nouvelle demande. T037 : never ne crée aucune route humaine.

T034 est stateless : ready ne contient que les tâches terminales, pending garde
les nonterminales ; pending_requests est une clé racine du résultat wait avec
task_id, Parent/open/nonexpirée/actionnable seulement. Human reste visible dans
status mais ne déclenche pas le retour wait. timed_out est faux si ready ou
pending_requests est non vide, vrai seulement à l’échéance wait si les deux sont
vides. Question Parent déjà ouverte : retour immédiat cohérent. WaitEpoch après
commit et compteur avant lecture sont réutilisés. Aucun curseur de consommation.

Analyze documentaire :PASS après cette relecture ; gates T001/T002 du reste155
toujours ouverts. Reuse 5/5 ; 41 tâches non cochées. Le seul résultat ciblé attesté
reste le reçu T000 transmis par root, sans conclusion forward ou live.

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
