# Contre-revue Claude — plan152, ronde1

Date : 2026-10-11. Verdict transmis par le principal : APPROVE_WITH_CHANGES.
Cette trace traite les objections communiquées par le principal et les preuves
ciblées du code. Ce n’est pas une copie intégrale de la sortie Claude.

Task de revue : node:delegated-task:command%3Amcp%3A36bf308a-6b4f-49a5-a2c2-541bffd8e74f%3Adelegate-task%3A152-plan-review-20261011-r1.

| Priorité | Objection | Preuve actuelle | Traitement dans les tâches existantes |
|---|---|---|---|
| P1 | Les contrôles de révocation et de sources sont séparés du bloc environnement | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:824 | T022/T023/T024 réutilisent ces contrôles avant l’effet ; cas T004 existants. |
| P1 | Le marqueur possède un second contrôle d’identité fatal | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233 | T043 vérifie le marqueur avant publication ; cas T048 sur les frontières. |
| P1 | Exclure un marqueur illisible de la map ne ferme pas self.load | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/managed_process.rs:1233 | Proposition rejetée comme preuve suffisante ; T048 couvre les incohérences natives lisibles. Corruption/illisibilité hors FR029, refus sûr existant conservé. |
| P2 | Reprise ne doit pas dépendre d’un marqueur déjà supprimé par réconciliation | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1211 | T023 utilise l’entrée de flotte et le lien attesté pour sa décision. |
| P2 | Un descendant doit utiliser le temps restant effectif du parent | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1287 | T017 utilise effective_deadline pour l’admission imbriquée ; date brute exposée documentée. |
| P2 | RequestWait151 calcule encore la date métier brute | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:289 | T018 gèle ce calcul pour les demandes natives, sans mutation ni changement de wait_timeout. |
| P2 | La suspension exige le contrôle courant, pas le seul payload | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:73 ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:126 | T021 place suspended dans current_task_view et laisse task_view pur. |
| P2 | Cleanup d’annulation peut écraser un terminal acquis | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs:1672 | T027 conserve erreur/résultat terminal ; cas intégrés à ses tests existants. |
| P2 | Le bail de lancement expiré ne doit pas être relancé | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs:4204 | T004/T023/T024 couvrent l’échec nommé compatible avec FR012. |
| P2 | La garde NativeDelegation existe déjà | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:107 | T009 conserve le nouvel effet nommé prévu et partage le bras existant ; duplication rejetée. |
| P2 | O(P) ignore le coût de sélection temporelle | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/referent_control.rs:165 | T050 ajoute l’index temporel au schéma existant ; coût O(log N + P). |

Toutes ces réponses sont des corrections de conception. Leurs tests152 restent
à écrire. Aucun code, service, modèle ou test nouveau n’a été exécuté par cette
passe. Les 50 tâches sont conservées ; seule T001 est cochée après sa baseline.
Les choix ne réduisent aucune des 29 FR, 9 SC ou 6 US.

## Revue de code152, ronde1 — REQUEST_CHANGES en contre-analyse

Date : 2026-10-11. Task : 152-code-review-20261011-r1.
Modèle de la capture : claude-haiku-5-5. Revue en lecture seule, sans
exécution. Aucun des cinq constats n’est reproduit par le reviewer.
Les priorités ci-dessous sont celles de la revue, pas un diagnostic validé.

Capture exacte, champs `summary` et `latestTerminalSummary` identiques :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r1.json.
SHA-256 : 7e53a22a2ab7d7bf1683350327950c104a3fe9979a415bd777e98be401f7799f.

Source citée par les cinq objections, numéros historiques de la revue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs.

| Objection et priorité annoncée | Hypothèse du reviewer | Contre-décision |
|---|---|---|
| H1, HIGH, lignes1445–1528 | Une exécution étrangère dans un état terminal peut faire remonter une erreur jusqu’au démarrage ; le reviewer l’assimile à FR-029. | Interprétation rejetée par le principal : FR-029 couvre commande/instance/lien natifs et marqueur, pas une ligne SQLite d’exécution étrangère. Les gardes/CAS149 restent stricts ; cas F2/F3 existants PASS R4. |
| M2, MEDIUM, lignes1295/1823 | `pause_covers(...).unwrap_or(false)` traite une erreur SQL comme absence de pause, puis fait échouer la mission. | Contre-analyse du chemin et des erreurs avant GO ; lecture, non reproduit. |
| M3, MEDIUM, lignes1476–1518 | Une erreur de stockage de la décision de reprise devient `failed` avec texte SQL brut. | Distinguer stockage et refus métier dans le contrat actuel ; contre-analyse, non reproduit. |
| M4, MEDIUM à confirmer, vers1780 | L’origine d’échéance utilise l’instantané d’exécution avec repli sur la tâche au lieu de la demande/mission durable. | Confrontation FR-009/FR-010 et T045 requise avant décision. |
| M5, LOW à MEDIUM, lignes2072–2215 | Certains retours d’annulation restent silencieux ; un arrêt non confirmé n’arrête pas la boucle. | Vérifier précontrôle, arrêt effectif et reprise de `cancelling` ; aucune promesse d’atomicité des PID. |

La contre-vérification H1 est faite par le principal, pas exécutée par le
propriétaire documentaire. Repères des tests stricts indiqués par celui-ci :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation_permissions149_tests.rs:1287
et lignes2385/2402 de ce même fichier absolu. Les 78 tests149 sont PASS
dans la capture R4 ; aucun oracle de cible étrangère n’est affaibli.

Le principal confie les quatre autres contre-analyses à la façade.
Ce tableau archive les objections et les décisions ; il n’autorise aucun
scénario ou waiver nouveau. La capture brute et la ronde de plan restent
conservées. T032/T034/T037 ne sont pas clôturées ; aucun APPROVE final.

## Contre-décisions terminales et corrections avant R5

Le principal autorise les seuls correctifs nécessaires après contre-analyse.
La capture de revue reste REQUEST_CHANGES ; aucune nouvelle décision
APPROVE n’est inventée et aucun constat n’est présenté comme reproduit
par le reviewer.

| Objection | Décision retenue et preuve du correctif |
|---|---|
| H1 | Rejet de l’interprétation FR-029 : commande/instance/lien natifs et marqueur, pas corruption étrangère du magasin d’exécution. Les gardes/CAS F2/F3 stricts restent conservés et PASS R4. |
| M2 | Retenue. N1285 rend `Result<bool, String>` et propage les erreurs de lecture/persistance ; D18654 revient sur `Err` avant les mutations générales. N1857 diffère le tick si la pause est illisible. |
| M3 | Retenue. N1525 ne convertit en échec que les neuf refus métier nommés ; les autres erreurs sont propagées. Aucun texte SQL opaque ne devient un refus métier. |
| M4 | Retenue. N1623/N1808 prennent uniquement `get_request(mission).created_at`. Une demande absente/illisible diffère ; aucun repli sur date de tâche ou d’exécution. |
| M5 | Réserve LOW, sans patch : précontrôle sans effet, puis échec partiel d’arrêt admis par FR-020 et déjà testé dans les cas prévus. Aucune atomicité de PID ni nouveau scénario. |

Repères N actuels :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon/native_delegation.rs.
SHA-256 : c46904d3623faa2c1799893db257b027aa94b460ed32cd5e3f3f72b53da99b6a.

Repères D actuels :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/crates/bridget-daemon/src/daemon.rs.
SHA-256 : 2f401b64b0fc6a5b8f85be0a56516cef315b6c32f46a9ed216f00e7273df7685.

La course E2E Connected est aussi traitée dans le chemin existant D3096 :
après identité Fleet attestée, mêmes gardes d’instance/commande/connexion
primaire/état libre avant la reprise. La lecture est faite contre les sources
gelées. R5 est exécuté par le principal ; son résultat final n’est pas encore
acquis ici. Aucun code, test ou service n’est exécuté par le propriétaire
documentaire. T032/T034/T036/T037 restent ouvertes.

## Ronde2 capturée et contre-lecture finale des objections

Capture exacte :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r2.json.
SHA-256 :1f9c502b2ac7658523af3ab097207c4c17d3d0dcbbbd0f03520fa3a321fb89c4.
Tâche152-code-review-20261011-r2, Claude Haiku5.5, completed.
Summary/latestTerminalSummary identiques. Verdict REQUEST_CHANGES.
Lecture seule du reviewer ; aucun test ou modèle réel exécuté par lui.

| Point | Décision et état présent |
|---|---|
| H1 | Limite explicitée dans spec, plan, contrat, guide et ADR. FR-029 couvre commande/instance/lien natifs et marqueur lisible. Les incohérences de cible étrangère et les erreurs CAS/stockage peuvent bloquer le démarrage. Les contrôles149 restent stricts. |
| M2 | Reconnue résolue par R2 : erreur de pause propagée/différée ; le caller revient avant les mutations générales. |
| M3 | Reconnue résolue par R2 : seuls neuf refus nommés deviennent failed ; les erreurs opaques remontent. |
| M4 | Origine durable reconnue résolue ; Ok(None) après admission/working sans échéance confirmé séparément par façade/qualité. Patch posé : close_lost_execution avec gardes, puis sauvegarde failed/request_unavailable ; SQL Err ou clôture refusée reportent sans mutation de tâche. Chemin avant admission intact. R6 reste en diagnostic, pas GREEN. |
| L5 | Causalité REFUTÉE par qualité et lecture : close_lost_execution vient après tous les arrêts et success=true ; son continue ne commande pas un arrêt de parent. Limite réelle : clôtures CAS partiellement abouties après les arrêts confirmés. Aucun patch L5 ni atomicité de PID promise. |

R5 ciblé :295 PASS/0 FAIL/1 ignored, comparatif239/0/1, native15256PASS.
Suite R1 distincte :94 cibles,2103 PASS/8 FAIL/72 ignored, mêmes cinq cibles
et huit noms que151 ; pas succès global. T032/T034 clôturées,48/50.
Après patch None, R6 :48 PASS/3 FAIL/0 ignored, RC101, E2E non exécutés.
Trois cas d'annulation attendent working et observent failed. Diagnostic
fixture sans suivi durable/produit en cours, aucune adaptation d'oracle.

T036/T037 restent ouvertes. Aucun APPROVE de ronde3, Converge, audit,
modèle réel, installation ou activation n'est déduit.
Preuves et limites :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.

### Contre-réponse M4 après les trois FAIL R6

Le principal/qualité confirment par lecture des trois bases privées : zéro
tracked_request, zéro exécution, échéances NULL. Les fixtures US4 forçaient
Working sans admission ; aucun défaut FR-018/FR-019 n'est démontré.
Les arrêts restent conformes [false,false]/[false,true]. R6 exerce le refus
request_unavailable réellement prévu pour ces absences.

GO test-only : create_request existant avec mission/owner/child/délai réels,
sans échéance artificielle ni changement d'oracle/préfixe produit. Ce GO ne
vaut pas GREEN R7. La revue R2 reste REQUEST_CHANGES jusqu'à une nouvelle
revue terminale. T036 attend R7/098 et T037 son diff final ;48/50 inchangé.

## Ronde3 — un défaut retenu, correctif posé, validation en cours

Capture : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r3.json.
SHA-256 : bf28aae1c7c55c4e3d32708ab1982182d95956c1747c7e611a7e3b1bb76922c0.
Tâche152-code-review-20261011-r3, Claude Haiku5.5, completed, REQUEST_CHANGES.
H1/M2/M3/M4/L5 sont reconnus résolus. Le reviewer ne lit pas R5/R6/R7 et
n'exécute aucun test ; ses réserves de course restent non confirmées.

Unique défaut retenu et confirmé indépendamment : request_unavailable ne
consommait pas l'interruption de pause. Une exécution interrupted restait
candidate à une reprise. Le correctif N1464–1469 ajoute cette raison au
booléen existant, après les gardes cible/CAS ; même abandon_pause_interruption,
sans nouvelle continuation ni assouplissement SQL/foreign/CAS.

R7 conserve deux FAIL de fixtures sans demande durable, confirmé par lecture
des DB :148TurnCompleted et149Working. create_request existant complète ces
données, sans modifier les oracles. Native15256PASS et les autres filtres
positifs restent distincts. R8/098 attendus ; aucun APPROVE final ni Converge.

## Validation finale native après R3 — T036 exécutée

R8 final :295 PASS/0 FAIL/1 ignored, sept RC0 et sources stables. Les
fixtures148/149 corrigées et l'ensemble native152 sont GREEN. Le correctif
request_unavailable consomme la pause liée après gardes existantes.
Le rapport R3 historique reste REQUEST_CHANGES ; aucune nouvelle décision
APPROVE n'est inventée. La revue terminale adverse aura lieu après Converge.

098 seul :0 PASS/1 FAIL, RC101, HTTP404 répétés comme151/R1 ; aucun effet
de charge prouvé. La suite R1 reste2103/8/72 avant patch final, pas GREEN
sur hashesR8. T036 cochée pour les exécutions requises :49/50. T037 attend
son verdict final ; Converge et audit non déclarés.
Preuves : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/tests-152.md.

## Ronde4 — APPROVE_WITH_CHANGES, clarification documentaire

Capture : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/152-reprise-missions/specs/152-reprise-missions/validation/code-review-152-r4.json.
SHA-256 :3c24241024963489c9b94f280e6da33752a9c732a910dccb1d7f938b6060c228.
Aucun blocage code retenu. R4 reconnaît le correctif de pause par lecture.
M2, Connected et les bornes ne sont pas relus dans cette ronde ; les preuves
R3, Converge et R8 restent distinctes. Aucun test exécuté par le reviewer.

Objection partiellement réfutée : les lignes142 et183 du compte rendu
décrivent le code et le diagnostic empirique R6, pas un oracle automatisé.
Clarification retenue : aucun test ciblé n'affirme ensemble request_unavailable
et abandon de pause dans ces deux branches. Lecture causale indépendante
pour la pause, observation réelle R6 pour le refus, régressions R8 pour les
cas existants. Limite ajoutée au compte rendu ; aucun scénario ou code ajouté.
