# Rapport d'analyse croisée - Session154

Date : 2026-10-11 (relecture sur base153 livrée). Entrées : `spec.md`,
`plan.md`, `tasks.md`, `research.md`, `data-model.md`,
`contracts/workspace154.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
Analyse en lecture seule, sans écriture de code.

## Verdict

PASS initial (historique avant contre-revueR1 ; statut actuel en cours ci-dessous). Arbitrage utilisateur reçu le 2026-10-11 : le daemon exécute Git pour tout
parent autorisé, comme T3, durci. 28 tâches. Constats : 0 critique, 0 élevé, 1
risque accepté (A0), 3 moyens, 4 faibles.

## Couverture des exigences

| Exigence | Tâches | Test |
|---|---|---|
| FR-001, FR-002 | T003, T011 | `native154_contrat_*` |
| FR-003 | T007, T009 | `native154_creation_*` (rejeu) |
| FR-004 | T010 | `native154_creation_catalogue_par_paire` |
| FR-005, FR-006 | T009, T010 | `native154_creation_*` |
| FR-007, FR-008, FR-009 | T006, T008 | `native154_git_*`, `native154_creation_*` |
| FR-010 | T009 | `native154_creation_*` (lien symbolique, recontrôle) |
| FR-011, FR-012 | T007, T008 | `native154_creation_*` |
| FR-013 | T007, T008 | filtre hostile, objet absent, inclusion |
| FR-014, FR-015 | T005, T006, T017 | `native154_git_*`, liste blanche |
| FR-016 | T009, T012 | `native154_isolation_*` |
| FR-017 | T007, T009, T028 | verrou libre (SC-009, SC-011) |
| FR-018 | T007, T009, T017 | restes nommés, aucune suppression |
| FR-019, FR-020, FR-021 | T014, T015, T016 | `native154_rapport_*` |
| FR-022, FR-023 | T017, T018, T019 | `native154_conservation_*` |
| FR-024 | T013 | `native154_isolation_*` |
| FR-025 | T020 | `native154_partage_*` |
| FR-026 | T021 | `native154_worktree_paths_do_not_collide` |
| FR-027 | T027 | réservations concurrentes (SC-010) |
| FR-028 | T022 | `native154_wrapper_*` |
| FR-029 | T023, T024 | revue |
| SC-001 | T012 | `native154_isolation_*` |
| SC-002 | T017 | `native154_conservation_*`, liste blanche |
| SC-003, SC-004 | T005, T007 | `native154_git_*`, `native154_creation_*` |
| SC-005 | T005, T017 | exécuteur enregistreur |
| SC-006 | T003, T016, T020, T025 | octets et non-régression |
| SC-007 | T014 | `native154_rapport_*` |
| SC-008 | T002 | `validation/mesure-worktree.md` |
| SC-009 | T007 | `native154_creation_verrou_libre` |
| SC-010 | T007, T027 | réservations concurrentes |
| SC-011 | T028 | `native154_rapport_hors_verrou` |

Toutes les exigences ont au moins une tâche et un test. T001, T002, T025 et T026
sont des tâches de méthode.

## Cohérence entre artefacts

- Les treize codes de refus sont identiques dans `spec.md`, le contrat et T007.
- La liste blanche Git est identique dans FR-015, le contrat, D5 du plan, T005,
  T006 et T017. Elle ne contient aucune suppression.
- L'état `Task.workspace` du `data-model.md` (`active`, `kept`) correspond à
  FR-020, au contrat et à T004. Aucun état `removed` ou `retained` ne subsiste
  (recherche `retained` et `removed` : seulement dans le texte de rejet).
- L’ordre des refus conserve153 : keep_alive/brief avant rejeu ; workspace après le rejeu.
- Chaque ronde reçoit un rapport Git frais avant publication ; CAS réutilise mission et round_no.
- `kept` conserve dossier/branche et permet les écritures des rondes suivantes.
- `cleanup_done` attend arrêt confirmé/fin de garde153 et rapport terminé.
- Les conditions de purge de la session150 ne sont pas modifiées : D7, T019.

## Constats

| # | Gravité | Constat | Sort |
|---|---|---|---|
| A0 | accepté | Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle et l'exécution de Git peut faire exécuter du code avec les droits du daemon. Ce risque est accepté explicitement, au même niveau que T3, dans le cadre de menace de l'ADR 011. Les durcissements réduisent la surface sans la fermer. | Accepté par l'utilisateur (2026-10-11). Écrit dans la spec et dans l'ADR 154 (T024). |
| A1 | moyen | Le flux le plus courant (parent Claude, enfant GLM) est refusé : la règle de droits exige le même dossier. | Défaut conservateur confirmé : refus explicite ; élargissement hors périmètre. |
| A2 | moyen | Un enfant sous bac à sable Codex `workspace-write` ne peut pas commiter dans un worktree lié. Le rapport dira « 0 commit, modifications non commitées ». | Défaut conservateur confirmé : ne rien élargir. Documenté en T023. |
| A3 | moyen | Fenêtre de course sur un lien `.worktrees` remplacé entre le contrôle et Git. Aucun descripteur ouvert ne peut être donné à Git sur macOS. | Recontrôle du chemin canonique après création ; attaquant = parent délégant ; effet = extrait propre. Résiduel accepté, noté dans l'ADR (T024). |
| A4 | faible | Les worktrees `kept` s'accumulent et n'ont pas de fiche après la purge de la tâche. | Plafond de 32 (`workspace_limit`), chemin dans le résultat, recette humaine. Accepté. |
| A5 | faible | Les dépôts avec Git LFS, filtres, inclusions ou clone partiel sont refusés. | Documenté ; `shared` reste disponible. Accepté. |
| A6 | faible | Le résultat structuré151 est déjà livré. | L'objet `workspace` est autonome ; la ligne de fin est retirable. Conserver le contrat151 existant et ajouter la ligne avant publication. |
| A7 | faible | Seule la version Git 2.55.0 est testée ; la version minimale est 2.44. | Contrôle de version au démarrage de chaque création ; noté en « non vérifié » dans le plan. |

## Constats corrigés pendant l'analyse et la contre-revue

- Le nom de branche par défaut dépend du `task_id`, absent de l'avant-vol :
  T008 ne contrôle l'existence que pour un nom explicite ; la réservation fixe
  le `task_id` à la première passe.
- Le motif `/.worktrees/` du `.gitignore` du dépôt est ancré à la racine et ne
  couvre pas un dossier du parent en sous-répertoire : le refus
  `workspace_dir_not_ignored` remplace l'écriture dans `info/exclude`.
- Le contexte de droits Claude d'un chemin inexistant serait celui du parent :
  D3, droits recalculés sur le worktree réel à la seconde passe.
- Ronde 2 : rapport Git hors verrou (T028), réservations concurrentes et plafond
  (T027), revalidation sans auto-contradiction (T009, contrat), lectures du
  wrapper bornées par l'exécuteur (T022), `GIT_ALLOW_PROTOCOL=` vide,
  `--no-pager`, signatures et programmes externes coupés (T005), dossier final
  jamais pré-créé par Bridget (T009).
- Contre-revue : verrou global (deux passes), filtres exécutables (refus),
  suppression (retirée), tête détachée (`head_on_branch`), wrapper (exécuteur borné complet, T022),
  version Git (2.44), accumulation (plafond de 32).

## Minimalisme et responsabilité (Articles XIX et XX)

- Un module nouveau. Pas de dépendance, table, état Lineage, outil MCP ni code
  de suppression. Environ 120 lignes prévues sont retirées par la révision.
- `native_permissions.rs` et le code de production `observation.rs` ne changent pas. `wrapper.rs` remplace les appels Git non bornés par l’exécuteur T022. Aucun refactor de `communication.rs` ni du projet commun.
- Non vérifié : comportement d'un vrai enfant dans un worktree lié, coût sur un
  très gros dépôt, Git 2.44 à 2.54. À éprouver hors de cette session.

## Gate avant implémentation

- [x] Chaque exigence a une tâche et un test
- [x] Aucun constat critique ou élevé ouvert (A0 : risque accepté)
- [x] Aucune duplication non arbitrée (`reuse-audit.md` PASS)
- [x] Contre-revue Codex : objections bloquantes fermées
- [x] Arbitrage sur l'exécution de Git reçu (2026-10-11)
- [x] Questions 1 à 3 : valeurs par défaut confirmées par l'utilisateur

## Relecture finale des phases2–4

Base : `89f7a4163c2fb27229587f50a46414295ba3674d`, incluant153 livrée et active.
Dossier explicite : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/154-isolation-worktree/specs/154-isolation-worktree

| Contrôle | Résultat |
|---|---|
| Inventaire | 29FR, 11SC, 5US ; couverture100% dans la matrice ci-dessus |
| Tâches | 28IDs uniques T001–T028 ; 0cochée ; aucun test exécuté par le documentaire |
| Règles153 | T003/T004/T009/T010/T015/T017/T018/T019/T028 conservées et explicitées |
| État kept | transition unique ; droits inchangés ; rondes suivantes possibles |
| Publication | rapport neuf par mission/ronde ; CAS existant ; historique publié immuable |
| Nettoyage | rapport terminé ET arrêt confirmé/fin de garde ; garde active interdit cleanup après ronde1 |
| Wrapper | T022 exécuteur borné complet, temps/sortie bornés, status omis si filtre |
| Réutilisation | gate5/5, zéro duplication non arbitrée, zéro arbitrage supplémentaire |
| Exécution | T001 sixfiltres ; T002 cinqmesures en clone privé ; T025 une suite finale |

Relecture sans mutation après correction : cohérence spec/plan/tasks/data-model/contrat
validée. Les constats moyens/ faibles historiques restent documentés avec leurs
choix conservateurs ; aucun constat critique ou élevé ouvert. Aucun ajout gratuit
de table, migration, dépendance, outil ou champ public de ronde.

Le script de prérequis absent a été remplacé par la sélection explicite du dossier.
`feature.json` étranger est préservé. Verdict final : **Analyze PASS documentaire**.
Cela ne vaut ni test154 vert, ni validation runtime, ni livraison154. La mesureT002 reçue du principal est contrôlée dans validation/mesure-worktree.md ; les28tâches restent noncochées dans ce retour documentaire.

## Contre-revueR1 du2026-10-12 — statut actuel

Le PASS documentaire précédent est historique. Quinzehypothèses examinées :
décisions du principal consignées dans adversarial-review-claude.md. Les choix
retenus sont en cours d’implémentation ; aucun GREEN ou livraison déduit. H11 a
reçu l’ancreregistreparent (à vérifier), H12 borne du lot reste en étude. Le
marqueur privé Task.workspace_report_mission est autorisé pour publication,
cleanup/reprise et reset ànouvelle ronde ; aucun champ public de ronde.
Baseline acquise ; T001/T002 seules cochées. T003/T022 restent noncochées
après RED1PASS/6FAIL. Prochain gate : GREEN prévu, puis relecture finale du
principal. Les docsT023/T024 sont préparées, sans preuve d’activation154.

## ComplémentH12 et limiteSIGTERM

WorkspaceWorkArcMutexséparé unique : creationsMap etreportsSet des
(task,mission,round) en cours. Claim empêche tickconcurrent de relancerGit lent ;
DropRAII retireseulementl’entrée du verrou séparé. Aucunthread/backgroundengine
ni quota arbitraire. Choixautoriséroot ; testnatifconcurrence àprouver.

SIGTERM sur seulchild.id Git puis reapborné, pasSIGKILL. Programmehostile
ignorantSIGTERM peutpersister ; retourborné, pas arrêtabsolu. Parent/ancêtre
principal etregistreGitcommun restent la sourcefiable du modèleADR011.
FondationsR1 :15PASS/7FAIL ; T004/T021/T022acquises ; cinqcasesdone,23open.
T011attendcontratferméglobal ; T003/T005nonacquises. Aucun GREENglobal.

## CandidatR2 interrompu — état de validation actuel

R1compilation :RC101,15,97s,zérotest (annotationResult corrigée). R2 :
RC101,256,30s,sourcesstables ; daemonpartiel9PASS/2FAIL, puis SIGTERM du
principal sur le seul PID20468 identifié comme binairetest et nonFirefox ;
transportterminé3PASS/0FAIL. Deuxoraclesfixtures corrigés en cours, lecteur
socket grandrésultat àcorriger par testsowner. Aucun GREENglobal, aucuncochage
supplémentaire. Reçu détaillé :validation/candidat154-r2.md.

## R3 — validation partielle, proposition seulement

Finales35PASS/9FAIL (daemon32/9,transport3/0),51,04s,RC101. Sixhelpers0/1
nonadditionnés. DixGitPASS ; grandrésultat etfixturesR2 désormaisPASS. Six
socketsfixtures, rétention, Stopemboîté et vrai défautmappingGit40appels en
correction. T005/T006/T010/T013/T014/T015/T016/T020 proposés surpreuves mais
noncochés jusqu’àGORoot aprèsR4 ; branchechangée obligatoire nonprouvéeR3.
FR014global vscommunication/derive_domain hérités horsscope reste un point de
revuefinale, sanspatchoffscope. Voirvalidation/candidat154-r3.md.

## R4 — état courant après GO ciblé

Finale4PASS/6FAIL,11,59s,RC101 ; six helpers0/1 nonadditionnés. Purge150,
emboîté,40appelsquota/mappingfermé et testdirectbranchechangée PASS. Sixfixtures
omettentBRIDGET_HOMEaprèsenv_clear ; correctionrecette en cours, R5sixseuls.
GOcochages : huitpropositionsR3, T012/T019 etT023/T024après espacement de prose.
19/28acquises,9ouvertes. implementation.md préparé ; T026resteouverte.
Aucun GREENglobal, Converge/audit final et livraison encoreàfaire.

## R5 — point courant avant suite finale

Six tests corrigés :6PASS/0FAIL,RC0,36,25s,sourcesstables. Cochage autorisé
T007/T008/T009/T017/T018/T027/T028 sur preuves R3/R4/R5 et revues root.
26/28acquises ; T025/T026 restent ouvertes. Groupes ciblésdistincts protocole3/
stockage2/MCP1/observation1/Git10/natif27/wrapper4 ; aucun totalglobal surhashunique.
Converge/audit/livraison restent àfaire, suitefinale et revue en premier.
