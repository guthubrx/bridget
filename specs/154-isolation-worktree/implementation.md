# Journal d’implémentation — session154

Statut : Implemented. Implémentation terminée ; livraison encore à faire. Date du point : 2026-10-12.
Branche : session-154-isolation-worktree. Base : 89f7a4163c2fb27229587f50a46414295ba3674d,
incluant 153 livrée. Worktree : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree
28/28 tâches acquises après GO du principal. T025/T026 sont cochées.
Les commits, intégration, push et livraison restent la responsabilité du principal.
Aucune activation 154 ni livraison déduite de ce journal.

## Changement préparé

Champ workspace optionnel, worktree sous la racine parent, branche nouvelle,
base figée. Admission en deux passes avec rejeu/autorisation avant les lectures Git.
Git hors verrou, réservation/claim de rapport via WorkspaceWork séparé.
Rapport à chaque mission/ronde avant publication CAS, marqueur privé de mission,
faits inconnus si tentative bornée échoue, suffixe unique assaini et budget UTF-8.
Conservation sans suppression. Garde 153 et droits 149 restent les autorités.

## Responsabilités et fichiers

| Responsable | Périmètre |
|---|---|
| Owner Git | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/workspace_git.rs et ses tests ; raccords wrapper/lib |
| Owner protocole | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-transport/src/protocol.rs ; stockage/reset ; délégation MCP |
| Owner natif | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/daemon/native_delegation.rs ; coordinationdaemon, catalogue et tests natifs |
| Owner documentaire | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree ; guide, skill, référence et ADR154 |
| Principal | Git/tests/build/livraison ; revues et autorité de cochage |

## Preuves acquises, distinctes

| Passage | Résultat / portée | Reçu |
|---|---|---|
| BaselineT001 | Canonique193 PASS/0 FAIL/1ignoré ; R0 séparé, recetteGit initialement incomplète | validation/baseline.md |
| MesureT002 | Cinq créations privées RC0 ; médiane 0,588788 s, max 0,662820 s ; délai de 20 s | validation/mesure-worktree.md |
| REDT003/T022 | Finales1 PASS/6 FAIL,23,45s ; helpers imbriqués exclus | adversarial-review-claude.md |
| FondationsR1 | 15 PASS/7 FAIL,49,10s ; sous-ensembles acquis T004/T021/T022 | validation/fondations154-r1.md |
| CandidatR1 | CompilationRC101,15,97s ; zérotest, annotationResult corrigée | validation/candidat154-r2.md |
| CandidatR2 | Daemonpartiel9 PASS/2 FAIL puis SIGTERM ; transport3 PASS complet ; aucune finale daemon | validation/candidat154-r2.md |
| CandidatR3 | Finales35 PASS/9 FAIL,51,04s ; six helpers nonadditionnés | validation/candidat154-r3.md |
| CandidatR4 | Finale4 PASS/6 FAIL,11,59s ; purge, emboîté, quota40/mappingfermé etbranchechangée PASS ; six socketsfixtures | validation/candidat154-r4.md |

Les corrections de fixtures respectent les scénarios prévus. Aucun scénario de
confort ni suite supplémentaire demandé dans ce journal. Aucun GREENglobal.
R5 terminé sur les six fixtures de socket : 6 PASS, 0 FAIL, RC0 ; BRIDGET_HOME
privé conservé après env_clear. T025 doit ensuite effectuer la suite workspace finale.

## Revues et risques

Le principal a relu les nouveaux blocs natifs, cancel_tree, admission/helpers,
rapport/exécuteur, garde153, marqueur/CAS, wrapper, protocole/stockage etdocs.
La contre-revueR1 porte quinze hypothèses et choix motivés. Ces relectures
ne remplacent pas Converge ni audit final après les derniers changements.

Risque Git : environnement/configuration durcis et formes fermées ; le parent
reste source de confiance ADR011. SIGTERM vise le seul child.id attesté ; reap
borné sans SIGKILL. Programme hostile ignorant SIGTERM peut persister, retour
borné sans garantie d’arrêt absolu. État transitoire de commondir de création parallèle :
refus fermé, pas de réessai automatique. Branche/worktree toujours conservés.

Portée FR014 précisée sur décision du principal : exécuteur des parcours154
(avant-vol, création, rapport) et des deux lectures du wrapper T022. Les lectures
historiques de communication.rs et derive_domain_at restent hors périmètre,
conformément au plan/H04 et à l’ADR. Aucun code hors périmètre modifié.

## Progression et prochaine étape

Acquises : T001, T002, T003, T004, T005, T006, T007, T008, T009, T010, T011, T012, T013, T014, T015, T016, T017, T018, T019, T020, T021, T022, T023, T024, T025, T026, T027, T028.
Ouvertes : aucune tâche d’implémentation. Pipeline de convergence/livraison à poursuivre.
Le principal complète les preuves restantes. T026 attend relecture finale du
diff complet, contrôle des abstractions/erreurs/unwrap et export du patch cumulé.
Export prévu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/diff-154.patch
Cet export n’est pas déclaré créé. Aucun Git exécuté par le documentaire.

## Point après R5

R5 : 6 PASS/0 FAIL, 36,25 s, RC0. R3/R4/R5 couvrent distinctement les 27 oracles
natifs ; autres groupes : protocole 3, stockage 2, MCP 1, observation 1, Git 10 et
wrapper 4. Aucune empreinte globale ni GREEN de suite unique déduit.
Le principal autorise les cochages nécessaires :26/28 acquises. T025/T026 seules
ouvertes ; suite workspace finale puis revue/exportpatch attendus.
Voir validation/candidat154-r5.md pour chaque nom et sa preuve.

## Prélecture finale — suite terminée puis correctif CAS ciblé

Le principal a terminé la lecture de tous les hunks des 31 fichiers du diff,
puis du correctif natif de publication. Aucun point bloquant des Articles XIX/XX
n’est signalé par cette prélecture. La suite workspace unique (exécution 3313)
a terminé : 94 cibles, 2194 PASS, 9 FAIL, 72 ignorés, 499,95 s, RC101.
Les huit échecs historiques 152 restent présents. L’écart coreAttachGap persiste
après son seul rejeu ciblé ; le diagnostic reste en cours. Aucun GREEN global.

La suite complète a été exécutée AVANT le correctif CAS. La relecture a montré
qu’un descendant admis pendant le rapport pouvait laisser publier le résultat.
RED réel : 0 PASS/1 FAIL. Le propriétaire natif a ajouté le seul recontrôle de
descendants_busy sous verrou avant mutation/publication : activité ou erreur
→ différer, faits et marqueur inchangés. Le principal a relu ce bloc.

APRÈS ce correctif : cinq tests de rapport PASS, 0 FAIL, 13,68 s, RC0. Ce passage
ciblé ne transforme pas la suite complète précédente en preuve du nouveau code.
Les empreintes de ces deux états et les reçus sont dans validation/tests-154.md.
T025/T026 restent ouvertes ; aucune exportation du patch final ni livraison
n’est déduite de ce point. Aucun Git/Cargo/test/service exécuté par le documentaire.

## Deux derniers constats — preuves ciblées distinctes

Le diagnostic coreAttachGap établit une erreur de date dans la fixture locale
face au journal UTC. Après minuit CEST : veille locale 2026-10-11, veille UTC
2026-10-10, journal courant UTC 2026-10-11. La fixture supprimait le courant.
Le principal confirme que le code concerné est inchangé depuis 153. Le patch
existant est limité à date -u et commentaire UTC ; GREEN encore en attente.
La première frame n’a pas été observée après nettoyage des anciens fichiers
privés. Aucun succès de frame n’est inventé.

FR001 : null explicite workspace/base_ref/branch accepté à tort. RED réel
0 PASS/1 FAIL puis réutilisation du générique deserialize_present_optional
existant avec trois attributs. GREEN ciblé protocole 3 PASS/0 FAIL, 7,82 s, RC0.
L’état post-null est distinct du post-CAS et de la suite complète avant CAS.
Empreintes et reçus : validation/tests-154.md. Aucun helper neuf.

26/28 tâches restent acquises. T025/T026 attendent le résultat core corrigé et
l’export par le principal du diff final, nouveaux fichiers inclus. Aucune
Converge formelle, livraison ou activation proclamée. Aucun Git/Cargo/test
ou source modifié par le documentaire.

## Clôture après les incréments finaux

CoreUTC : 1 PASS/0 FAIL, 13,99 s, RC0. Le défaut de date de fixture est corrigé, sans
changement de la production concernée. Le principal a relu tout le diff de
31fichiers et les trois incréments finaux : CoreUTC, attributs null, CAS. Aucun
blocage XIX/XX signalé. Worktrees vérifiés à 22:49 : principal et 154 seulement.

Le patch complet incluant les nouveaux fichiers est exporté dans
validation/diff-154.patch. Empreinte avant ce dernier cochage :
`7e7d8136fe2ef7bff519c30c2aa851af9ce24f49d1b97e89438c9d846d3877fd`.
Le principal le réexportera après gel documentaire. Aucun Git exécuté ici.

T025/T026 acquises ; 28/28, état Implemented seulement. La suite complète est
historiquement RC101 avec 2194 PASS/9 FAIL/72 ignorés ; les corrections CAS/null/UTC
sont éprouvées par les seuls sous-ensembles ciblés nécessaires. Aucun GREEN
global, aucune Converge formelle ni livraison/activation proclamée. Les notes
plus haut décrivent les états historiques avant leurs corrections.

## Après Converge P1 — revue R1 et correctifs en cours

Converge P1 : 28/28 tâches, 29 FR, 11 SC, 5 US ; checkpoint du principal avant
les deux correctifs retenus F1/F2. La contre-revue R1 donne APPROVE_WITH_CHANGES.
F1 est corrigé avec preuve ciblée : quota après retrait humain, seulement sur NotFound. F2 est corrigé avec preuve ciblée : ignorer une
entrée voisine non pertinente du registre sans accepter une cible illisible.
Les tests existants union32 et ancre sont étendus. RED0/2 puis GREEN2/0 acquis ;
les reçus finaux ci-dessous portent ces preuves. Le principal reconvergera.

Audit C1 archivé : 0 critique, 0 élevé, 2 moyens, 2 faibles, couverture 64,75 %.
Ce checkpoint ne représente pas une couverture entière. Aucun nouveau hash de
source final ou de patch final n’est déclaré avant la capture fraîche.
Correction QUALLOW : après arrêt confirmé, sauvegarder le rapport puis poser
cleanup_done dans une seconde sauvegarde ordonnée. Aucune sauvegarde commune
n’est revendiquée pour ce chemin de nettoyage.

F3/F4 restent des limites de diagnostic en concurrence : tentative nommée,
exclusivité Git sans écrasement, refus tardif et code au rejeu parfois distincts.
F5 : UTF-8 strict, sujet invalide → rapport indisponible. F6 : contrôle de version
par commande et coût hors verrou accepté. Aucun helper/cache/outil nouveau.
Cases de tâches inchangées ; captures et reconvergence finales restent à faire.

## F1/F2 corrigés — preuves ciblées finales avant P2

RED : 0 PASS, 2 FAIL, 13,03 s, RC101. GREEN : 2 PASS, 0 FAIL, 0 ignoré,
10,68 s, RC0. Les extensions des tests union32 et ancre existants ont été
exécutées sur 207 sources stables pendant chaque passage. F1/F2 sont corrigés,
avec preuve ciblée. La cible illisible reste en erreur ; seule une absence
NotFound de chemin Task libère la place du quota. Aucun helper nouveau.

RED reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-red-r1.json
RED journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-red-r1.log
RED SHA256 : `8696fe79a32405b3b29830029f0c8d1cd53b43045c9668c477bbe3af66e1f4bd`.
GREEN reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-green-r1.json
GREEN journal : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree/validation/review154-final-green-r1.log
GREEN SHA256 : `c8caa4f6a9bd63a4024830c02e4f22254ce081a82d872658924b1811306c770e`.

Empreintes vérifiées contre les sources courantes et le reçu GREEN :
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/daemon/native_delegation.rs : `75e3713d50a0ee60492e5cdaf2cf7f6f5960fb814550ee95d100d2d36d4979a4`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/workspace_git.rs : `e0e4f6dcf385d8644b777572dcf23aae4f8d54743fb95ee31d82165770f79ca4`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/daemon/native_workspace154_tests.rs : `150bfbbdac8ece378bf45a3c6883d23abd9aa14a6a2b58df4fafeb9fcca8b0e8`.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/crates/bridget-daemon/src/workspace_git_tests.rs : `2208dc67f2d6fb6aa6dfb9bfc28c50be8b945ce1566d75f7a0e7c3b509bd8472`.

Les copies JSON et logs de validation sont identiques aux archives. Ce passage
ne remplace pas la suite workspace historique avant les correctifs. Aucun
GREEN global, verdict P2, capture d’audit fraîche ou nouveau patch final déclaré.
Docs gelées pour permettre la reconvergence P2 du principal. Tasks inchangé.
