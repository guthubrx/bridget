# Rapport d'analyse croisée - Session154

Date : 2026-10-11 (révisé après contre-revue Codex). Entrées : `spec.md`,
`plan.md`, `tasks.md`, `research.md`, `data-model.md`,
`contracts/workspace154.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
Analyse en lecture seule, sans écriture de code.

## Verdict

PASS. Arbitrage utilisateur reçu le 2026-10-11 : le daemon exécute Git pour tout
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
- L'ordre des refus prolonge celui de la session150 et le rejeu précède tout
  contrôle `workspace`, à chaque passe.
- Les conditions de purge de la session150 ne sont pas modifiées : D7, T019.

## Constats

| # | Gravité | Constat | Sort |
|---|---|---|---|
| A0 | accepté | Un parent confiné qui peut modifier la configuration du dépôt entre le contrôle et l'exécution de Git peut faire exécuter du code avec les droits du daemon. Ce risque est accepté explicitement, au même niveau que T3, dans le cadre de menace de l'ADR 011. Les durcissements réduisent la surface sans la fermer. | Accepté par l'utilisateur (2026-10-11). Écrit dans la spec et dans l'ADR 154 (T024). |
| A1 | moyen | Le flux le plus courant (parent Claude, enfant GLM) est refusé : la règle de droits exige le même dossier. | Question 1. Défaut : refus explicite, session 154b. Ouvert. |
| A2 | moyen | Un enfant sous bac à sable Codex `workspace-write` ne peut pas commiter dans un worktree lié. Le rapport dira « 0 commit, modifications non commitées ». | Question 2. Défaut : ne rien élargir. Documenté en T023. Ouvert. |
| A3 | moyen | Fenêtre de course sur un lien `.worktrees` remplacé entre le contrôle et Git. Aucun descripteur ouvert ne peut être donné à Git sur macOS. | Recontrôle du chemin canonique après création ; attaquant = parent délégant ; effet = extrait propre. Résiduel accepté, noté dans l'ADR (T024). |
| A4 | faible | Les worktrees `kept` s'accumulent et n'ont pas de fiche après la purge de la tâche. | Plafond de 32 (`workspace_limit`), chemin dans le résultat, recette humaine. Accepté. |
| A5 | faible | Les dépôts avec Git LFS, filtres, inclusions ou clone partiel sont refusés. | Documenté ; `shared` reste disponible. Accepté. |
| A6 | faible | La session151 (résultat structuré) peut préférer un objet à la ligne de fin du résultat. | L'objet `workspace` est autonome ; la ligne de fin est retirable. Coordination à la planification 151. |
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
  suppression (retirée), tête détachée (`head_on_branch`), wrapper (2 lignes),
  version Git (2.44), accumulation (plafond de 32).

## Minimalisme et responsabilité (Articles XIX et XX)

- Un module nouveau. Pas de dépendance, table, état Lineage, outil MCP ni code
  de suppression. Environ 120 lignes prévues sont retirées par la révision.
- Aucune ligne existante n'est supprimée. `native_permissions.rs` et
  `observation.rs` ne changent pas. `wrapper.rs` reçoit 2 lignes de durcissement.
- Non vérifié : comportement d'un vrai enfant dans un worktree lié, coût sur un
  très gros dépôt, Git 2.44 à 2.54. À éprouver hors de cette session.

## Gate avant implémentation

- [x] Chaque exigence a une tâche et un test
- [x] Aucun constat critique ou élevé ouvert (A0 : risque accepté)
- [x] Aucune duplication non arbitrée (`reuse-audit.md` PASS)
- [x] Contre-revue Codex : objections bloquantes fermées
- [x] Arbitrage sur l'exécution de Git reçu (2026-10-11)
- [x] Questions 1 à 3 : valeurs par défaut confirmées par l'utilisateur
