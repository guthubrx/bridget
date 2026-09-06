# Tâches 090 — Implemented

## Phase 1 — Cadrage et sonde

- [x] T001 Vérifier le contrat réel de fil partagé et d'approbation dans specs/090-codex-interactif/probe_shared_session.py et research.md ; même threadId, aucun prompt caché, décision du second client respectée.
- [x] T002 Auditer la réutilisation, les bornes et les permissions dans specs/090-codex-interactif/reuse-audit.md ; gate PASS, aucune duplication non arbitrée.

## Phase 2 — Frontière native commune

- [x] T003 Déplacer connect_nonblocking de crates/bridget-daemon/src/communication/client.rs vers crates/bridget-transport/src/jsonl.rs sans changement de contrat ; rejouer tests de connexion/délais existants.
- [x] T004 Ajouter crates/bridget-transport/src/codex_socket.rs et la dépendance framing minimale ; tester bytes atypiques, limite, EOF, handshake et écriture pendant lecture inactive avant branchement du pilote.

## Phase 3 — US1, session interactive joignable

- [x] T005 [US1] Étendre crates/bridget-transport/src/codex_app_server.rs pour posséder un app-server Unix, nommer le fil et réutiliser reader/worker ; stdio géré inchangé, identité = PID réel du serveur.
- [x] T006 [US1] Ajouter crates/bridget-daemon/src/codex_interactive.rs et raccorder wrapper.rs : options natives explicites, terminal requis, présence non-tmux, TUI sur le même fil après journal/identité prêts ; tests parsing et refus sans TTY.
- [x] T007 [US1] Prouver demande/réponse liée et retry avec daemon/CLI/MCP réels dans crates/bridget-daemon/tests/codex_interactive_090_test.rs ; une remise, ledger et identité cohérents, namespace isolé.

## Phase 4 — US2, tour humain et permissions

- [x] T008 [US2] Observer les tours TUI dans crates/bridget-transport/src/codex_app_server.rs : journal complet, état attesté, aucune collision avec worker interagent ; tests tour externe puis message Bridget, raw exact et navigation explicitement traitée.
- [x] T009 [US2] Tester puis garantir dans le reader interactif que permission/élicitation n'obtiennent aucune réponse automatique ; vraie requête native décidée par la TUI dans codex_interactive_090_test.rs.

## Phase 5 — US3, fin et pannes

- [x] T010 [US3] Borner arrêt TUI/serveur, EOF et fermeture terminal dans codex_interactive.rs et codex_app_server.rs ; zéro enfant de test survivant et socket possédée nettoyée, sous 10 s.
- [x] T011 [US3] Rejouer coupure/reconnexion du daemon depuis codex_interactive_090_test.rs ; identité et threadId inchangés, même tracker, aucun thread/start supplémentaire ; parcours SSH configurable conservé.

## Phase 6 — Recette, convergence et audit

- [x] T012 Exécuter la vraie TUI sous PTY et un échange fournisseur sous abonnement existant si disponible dans codex_interactive_090_test.rs ; aucun fallback API payant. Consigner tout blocage exact dans implementation.md.
- [x] T013 Aligner README.md, README.en.md, skills/bridget/SKILL.md et docs/communication-installation.md sur le comportement vérifié, distinguer interactif/spawn persistant/attach.
- [x] T014 Exécuter cargo fmt --all --check, cargo test --workspace et cargo clippy --workspace --all-targets -- -D warnings, puis convergence/audit dans specs/090-codex-interactif/implementation.md ; aucune case fermée sans preuve.

## Dépendances et exécution

T001→T002→T003→T004→T005→T006→T007 ; T008/T009 prolongent le même pilote
séquentiellement ; T010/T011 après assemblage ; T012→T013→T014 en clôture.
Pas de couloir d'écriture parallèle sur wrapper/pilote. Les lectures/revues peuvent
être indépendantes. Les tâches de test rédigent d'abord l'oracle puis le fix utile.
US1 seule n'est PAS la livraison : toutes les phases sont requises.
