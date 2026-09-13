# Tâches 093 — implémentées, vérifiées et installées

## Préparation

- [x] T001 Contrat, plan et réemploi PASS dans specs/093-attach-conversation/ ;
  critères tirés de la capture et scope compatible avec 091/092.
- [x] T002 Analyse croisée avant code dans specs/093-attach-conversation/analysis.md ;
  chaque FR couverte, aucun CRITICAL.

## US1 — Lecture

- [x] T003 [US1] Ajouter les oracles Markdown complet/fragmenté, Unicode et ANSI
  hostile dans crates/bridget-daemon/src/attach_renderer.rs (FR-001/002/006).
- [x] T004 [US1] Raccorder parsing/style et corps sous en-tête dans
  crates/bridget-daemon/src/attach.rs et attach_renderer.rs ; dépendance minimale
  dans crates/bridget-daemon/Cargo.toml/Cargo.lock ; jamais parser Markdown maison.
- [x] T005 [US1] Réutiliser les noms attestés de la présence pour les labels dans
  crates/bridget-daemon/src/attach.rs ; fallback UUID compact, test identité
  d'adressage inchangée et absence d'indentation par UUID (FR-003).

## US2 — Faits utiles

- [x] T006 [US2] Tester puis filtrer reasoning/terminaux ordinaires dans la vue
  TTY de crates/bridget-daemon/src/attach.rs, sans supprimer la clôture logique ;
  fixtures erreurs/refus/permissions/Gap/End et inconnu toujours visibles,
  non-TTY diagnostic intact (FR-004/005/008).

## US3 — Géométrie

- [x] T007 [US3] Test PTY et implémentation resize 100→45→100 sans frappe dans
  crates/bridget-daemon/src/attach.rs : bloc courant/dernier bloc géré, brouillon,
  footer et nouveaux messages ; conserver les bornes (FR-007/008).

## Consolidation et livraison

- [x] T008 Exécuter tests ciblés parser/attach/spec092 et examiner chaque hunk,
  consigner résultats dans specs/093-attach-conversation/implementation.md ;
  mutation ciblée de filtre/resize ou oracle discriminant explicitement démontré.
- [x] T009 Convergence puis audit de code/sécurité/complexité dans
  specs/093-attach-conversation/convergence.md et verification.md ; revue adverse
  indépendante ou indisponibilité documentée, aucun manque caché.
- [x] T010 Aligner README.md, README.en.md et skills/bridget/SKILL.md sur les
  comportements testés et limites du scrollback ; aucune promesse d'édition/MCP/SSH.
- [ ] T011 Gate final fmt, clippy, workspace, build release puis installation
  atomique sauvegardée ; recette du binaire installé et commande attach humaine
  dans specs/093-attach-conversation/implementation.md (FR-009).

- [ ] T012 Correctif recette humaine : reproduction écran interprété du resize
  100→45→160, état courant puis retenu puis historique ; aucune duplication du
  header et réélargissement du bloc encore géré. Test rouge avant fix, vert après,
  capture avant/après et nouvelle installation sauvegardée. Ne pas confondre
  sortie d'octets sur PTY avec position finale des cellules à l'écran.

## Ordre et ownership

T001→T002→T003/T004→T005→T006→T007→T008→T009→T010→T011.
Agent23 seul auteur de T003–T007. Pilote : artefacts, validation, installation.
Revue indépendante en parallèle des docs, pas d'édition concurrente attach.rs.
Pas de commit automatique. Les cases implémentation restent ouvertes jusqu'aux
tests effectifs du pilote si l'environnement de l'équipier bloque Cargo.
