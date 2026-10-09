# Tâches — SPEC142

Date : 2026-10-07. Statut : Implemented — publié, non committé. Gate réutilisation PASS. Huit tâches cochées après preuves et phase Converge lecture seule.

Racine Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/`.
Racine dotfiles : `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/`.

## Phase 1 — Setup

- [x] T001 — Préparation : relire spec/plan/audit ; fixer les six identités, sources explicites, cibles résolues et empreintes scripts/LaunchAgent. Vérifier la sauvegarde `/Users/moi/.cache/bridget-skills-142.eTqmgl/`. Couverture : FR08/09/14, SC03/06.
- [x] T002 — RED : écrire puis exécuter `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/tests/test_142_bridget_skill_sync.py` pour copie legacy plus récente, préservation scripts/cibles externes, sauvegarde, sources manquantes, liens et idempotence. Consigner les échecs de comportement, pas seulement un défaut de harness. Couverture : FR07/09/10/11/13, SC03/04/05.

## Phase 2 — Histoires utilisateur

- [x] T003 [US1] — Compétences : produire les trois canons, trois alias courts distincts et métadonnées françaises. Préserver le comportement Bridget, les règles loop/handoff et tous les scripts. Ajouter les deux liens relatifs `scripts`. Couverture : US01/02, FR01–08, SC01–03.
- [x] T004 [US3] — GREEN publisher : étendre `/Volumes/8TB2/01-workflow/git-worktrees/dotfiles/142-bridget-skills/claude/sync_codex_skills.sh` avec branche explicite six noms sans mtime ni suppression d'arbres legacy. Archiver pre-104 hors découverte sans perte. Rendre les tests ciblés verts ; vérifier `bash -n`. Couverture : US03, FR09–11/13, SC04/05.
- [x] T005 [US4] — Appelants : corriger les huit fichiers listés dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/142-skills-bridget/specs/142-skills-bridget/research.md`. Vérifier les quatre lignes CLI inchangées. Couverture : US04, FR12, SC05.

## Phase 3 — Validation et publication

- [x] T006 — Validation : vérifier frontmatter, prompts `$canon`, descriptions, alias invocables et parité des instructions. Rejouer les tests ciblés et la fixture native Codex sans modèle ; distinguer limites Claude. Revue locale de diff. Couverture : FR01–07/13/15, SC01/02/04/06.
- [x] T007 [US4] — Publication : sauvegarde et cibles exactes, publication coordonnée, seconde exécution, comparaison scripts/config et fichiers installés. Aucun restart ni mission. Documenter toute limite ou blocage sans revendiquer un état non prouvé. Couverture : FR08–11/14/15, SC03–06.
- [x] T008 — Clôture : Converge lecture seule, audit ciblé, journal factuel et statut sur preuves. Pas de commit/fusion/push sans autorisation distincte. Couverture finale : tous FR et SC.

## Dépendances

T001 → T002 → T004. T003 et T005 peuvent avancer dans leurs fichiers distincts après T001. T006 attend T003/T004/T005. T007 attend T006. T008 attend T007. Ownership source distribué par le principal ; ne jamais revenir sur les edits d'autrui.
