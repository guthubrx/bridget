# Audit de reutilisation de l'existant — 175
## Decision
Statut: PASS
Date: 2026-10-23
Feature dir: specs/175-cycle-worktrees
Conclusion courte: Les changements étendent deux mécanismes existants. Aucun second nettoyeur ni système de propriété n'est créé.
## Synthese
| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 5 |
| Items audites | 5 |
| Reutilisations deja prevues | 5 |
| Existants potentiellement pertinents | 0 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 5 |
| Specs existantes applicables | 3 |
## Reutilisations correctement identifiees
| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Conservation | task_view / workspace_report_ready | crates/bridget-daemon/src/daemon/native_delegation.rs:3624,9125 | projection sans écriture |
| Restitution | append_workspace_result | crates/bridget-daemon/src/daemon/native_delegation.rs | rappel de conservation existant |
| Inventaire | worktree_entries / git_ok | scripts/build.py:246,287 | étendre le parseur existant |
| Gardes | prune_merged_worktrees / busy_paths | scripts/build.py:269,292 | réutiliser observations, ne pas copier un moteur |
| Guides/tests | docs/delegation-native.*, test_build_cleanup.py, native_workspace154_tests.rs | chemins existants | extension ciblée |
## Existant potentiellement pertinent non mentionne
Aucun.
## Duplications evidentes
Aucune.
## Memoires et regles applicables
| Source | Regle | Impact sur le plan |
|---|---|---|
| AGENTS.md | serveur Cartae | code et tests distants |
| Constitution XIX/XX | réutiliser / charge future | pas de nouveau service |
| Constitution XXI.3 | coordination limitée | un responsable par livrable |
| AGENTS.md T3 | source avant choix | trois chemins T3 lus |
| Docs publiques | anglais + français | deux éditions synchronisées |
## Specs livrees applicables
| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 154 | worktree conservé | même propriétaire et mêmes rapports |
| 172 | préparation distante | aucune migration de chemin |
| 174 | capacité et gardes | garder 128 et lsof fail-closed |
## Journal de recherche
| Requete | Portee | Resultat |
|---|---|---|
| rg workspace_report/retained/task_view | crates/bridget-daemon | projection et rapport présents |
| rg worktree/cleanup/prune | scripts/build.py, tests, cli.rs | nettoyeur externe et refus cleanup noyau |
| rg MAX_WORKTREES | workspace_git/native_delegation | plafond partagé 128 |
| rg removeWorktree/createWorktree | dépôt T3 | création, partage et retrait explicite |
## Arbitrages
| Sujet | Decision | Justification | Date |
|---|---|---|---|
| Inventaire | REUTILISER build.py | mêmes observations, pas de nouveau nettoyeur | 2026-10-23 |
| Projection | REUTILISER task_view | pas de format persisté nouveau | 2026-10-23 |
| Artefacts 175 | CREER | aucune session cycle de vie présente ; 174 audit distinct | 2026-10-23 |
| Rangement et partage | documentation seulement | éviter migration/élargissement implicite | 2026-10-23 |
## Gate avant tasks
- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees

## Précisions après contre-revue du plan
Un seul évaluateur de motifs doit alimenter inventaire et retrait existant. La commande worktrees retourne avant toute purge. Git sans verrou optionnel ; erreurs et délais deviennent inconnu. Le rappel de restitution reste une consigne constante de relecture du statut, jamais un motif dynamique figé. Décisions détaillées : adversarial-review.md.

## Items réutilisés au clavier
- REUTILISER le tuple worktree_entries comme Worktree nommé : ajouter locked/prunable observés, sans second registre.
- EXTRAIRE integration_reason, age_reason, dirty_reason depuis prune_merged_worktrees : règles partagées, contrainte explicite d’un seul évaluateur pour retrait et observation.
- CREER inventory/report_worktrees et table des consignes : aucun rendu d’inventaire existant trouvé ; branche dans la commande d’entretien existante. Pas de dépendance.
- REUTILISER git_ok/busy_paths : erreurs explicites, status sans verrou ni fsmonitor, index des fichiers ouverts par passage.
