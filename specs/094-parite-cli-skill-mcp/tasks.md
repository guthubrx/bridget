# Tâches 094

Racine absolue : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/091-communication-agent-ux.
Statut : Implémenté et fichiers installés ; activation des sessions vivantes en attente d'accord humain. Pas de commit automatique. Les chemins ci-dessous sont
relatifs à cette unique racine pour éviter de noyer la checklist.

## Préparation

- [x] T001 Cadrer spec/plan/contrat et audit de réutilisation dans specs/094-parite-cli-skill-mcp ; neuf items audités sans duplication.
- [x] T002 Analyser la cohérence spec/plan/tasks et consigner les objections indépendantes dans specs/094-parite-cli-skill-mcp/analysis.md avant code.

## US1 — Actions propres

- [x] T003 [US1] Poser oracles rouges spec094 dans crates/bridget-daemon/src/mcp.rs et tests/core_089_identity_test.rs : outils absents, champs libres, conflits, DND/domaine/runtime, instance et erreurs.
- [x] T004 [US1] Corriger les gardes Domain/Availability/Runtime Declared dans crates/bridget-daemon/src/daemon.rs ; prouver raw refusé, A→B refusé, vieille instance refusée, A→A accepté et hooks observés inchangés.
- [x] T005 [US1] Étendre crates/bridget-daemon/src/communication/client.rs et refactoriser domain/dnd/runtime dans crates/bridget-daemon/src/cli.rs : même client, durée bornée, domaine atomique et erreurs de persistance attestées.
- [x] T006 [US1] Exposer rename/dnd/domain/runtime dans crates/bridget-daemon/src/mcp.rs : schémas fermés, reçus corrélés, aucun paramètre cible/source, tests ciblés verts.

## US2 — Observations

- [x] T007 [US2] Poser tests spec094 dans crates/bridget-daemon/src/mcp.rs et src/daemon.rs : statut assaini, indisponibilité, contrôle lecture bornée, mutation interdite et panne compteur inbox sans faux zéro.
- [x] T008 [US2] Exposer status/control_status dans crates/bridget-daemon/src/mcp.rs et corriger le faux zéro de handle_control_state_read dans src/daemon.rs ; réutiliser projections existantes sans fichiers libres.

## US3 — Documentation et inventaire

- [x] T009 [P] [US3] Compléter skills/bridget/SKILL.md, sa référence references/commandes.md, README.md et README.en.md : chaque commande, rôle, décision, exemple, limites de session et fournisseur. Pas d'affirmation d'installation anticipée.
- [x] T010 [US3] Ajouter oracle d'inventaire/catalogue dans crates/bridget-daemon/src/mcp.rs (ou test094 existant approprié) : commande du répartiteur sans décision détectée, équivalences non dupliquées, outils réservés absents ; corriger uniquement libellés aide trompeurs concernés dans src/cli.rs.

## US4 — Permissions et recette

- [x] T011 [US4] Étendre listes fermées dans crates/bridget-daemon/src/wrapper.rs ; adapter tests/fixtures/codex_mcp_policy_091.py et tests/claude_native_permissions_test.rs ; tester autorisation positive et retrait négatif sans bypass global.
- [x] T012 [US4] Recette vrai daemon privé via tests/core_089_identity_test.rs et tests/core_089_crash_test.rs : rename/DND/domaine, reconnexion, erreurs et identité ; recette fournisseur existante, résultats dans specs/094-parite-cli-skill-mcp/implementation.md.

## Consolidation

- [x] T013 Revue indépendante code/skill, analyse finale et convergence exigences→code→oracles dans specs/094-parite-cli-skill-mcp ; aucune assertion de sécurité globale des hooks historiques.
- [x] T014 Exécuter fmt/clippy/workspace puis release, installer atomiquement le binaire vérifié et la skill/référence ; consigner empreinte et sessions non rechargées dans specs/094-parite-cli-skill-mcp/implementation.md.

## Dépendances et parallélisme

T001→T002→T003→T004→T005→T006 ; T007/T008 ensuite dans les mêmes fichiers.
T009 est le seul couloir parallèle d'écriture (documentation exclusivement).
T010/T011→T012→T013→T014. Le pilote exécute tests/build ; l'équipier n'écrit
pas dans un fichier pendant sa contre-validation. Tous les résultats nomment
la recette réelle ou le blocage, pas seulement le nombre de tests présents.
