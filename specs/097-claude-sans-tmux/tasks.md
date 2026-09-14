# Tâches 097 — Claude natif et interactif sans tmux

## Préparation
- [x] T001 Vérifier la baseline verte du worktree en environnement privé (umask 077, racine courte) : fmt, clippy, suite workspace ; consigner la commande et les compteurs dans specs/097-claude-sans-tmux/implementation.md.

## Fondations (bloquantes pour US2 à US5)
- [x] T002 Extraire `validate_tmux_content` en `validate_injection_content` partagée dans crates/bridget-transport/src/tmux.rs (alias conservé, tests existants inchangés) ; exposer `terminal_geometry` en `pub(crate)` dans crates/bridget-daemon/src/attach.rs. Observable : `cargo test -p bridget-transport --lib tmux::` vert.
- [x] T003 Créer `PtyTransport` dans crates/bridget-transport/src/pty.rs (trait `Transport` : validation partagée, `wrap_envelope`, collage encadré, digestion bornée, `\r`, `is_alive` par PID) + export dans src/lib.rs ; tests unitaires sur un `openpty` local : octets exacts écrits, refus d'un corps avec ESC, `AgentDead` si le PID est mort.

## US2 — Session Claude interactive joignable dans n'importe quel terminal (P1)
- [x] T004 [US2] Écrire le harnais crates/bridget-daemon/tests/fixtures/claude_interactive_097.py (faux `claude` sous PTY : écho hexadécimal des octets reçus, impression de `TIOCGWINSZ`, sortie sur commande `quit`, code de sortie configurable) et le test rouge crates/bridget-daemon/tests/claude_interactive_097_test.rs avec les oracles : relais frappe→enfant, enfant→écran, SIGWINCH, remise collée + `\r`, présence `claude_pty/cli`, termios restauré, code de sortie relayé. Observable : test rouge pour cause fonctionnelle, pas de compilation.
- [x] T005 [US2] Créer crates/bridget-daemon/src/claude_interactive.rs : `Launch::check_terminal` (message Claude), `PtySession` (openpty, `setsid` + `TIOCSCTTY` en `pre_exec`, garde termios `cfmakeraw` restauré en `Drop`, threads de relais stdin→maître et maître→stdout, `SIGWINCH` via signal-hook → `TIOCSWINSZ`, `wait` + code de sortie) et déclarer le module dans crates/bridget-daemon/src/lib.rs. Complexité documentée : relais O(octets).
- [x] T006 [US2] Raccorder `wrapper::launch` dans crates/bridget-daemon/src/wrapper.rs : pour `claude` hors `--equipier`, lancer par `PtySession`, enregistrer `protocol = "claude_pty"`, `PresenceMode::Cli`, `location = None` (initial et reconnexion), `transport: Option<Box<dyn Transport>>` alimenté par `PtyTransport` ; retirer le contexte tmux du chemin Claude ; ne plus ajouter automatiquement `--dangerously-skip-permissions`/`--permission-mode bypassPermissions` pour Claude interactif (FR-09710 : la décision reste à l'humain ; un bypass explicite passé par l'utilisateur est relayé tel quel). Observable : T004 vert hors oracles US3/US4 ; `ps` de la session ne montre plus le bypass implicite.

## US3 — Remise honnête pendant un tour et saisie préservée (P1)
- [x] T007 [US3] Étendre le harnais et le test 097 : saisie humaine partielle puis remise Bridget → composer contient saisie + collage encadré, un seul `\r` de remise ; corps au-delà du plafond ou avec séquence de contrôle → aucune écriture, `DeliveryIndeterminate` observé côté daemon ; rappel palier → même voie. Observable : oracles verts sans modification de `deliver_idempotent_to_interactive`.

## US4 — Présence et refus explicites (P1)
- [x] T008 [US4] Dans crates/bridget-daemon/src/wrapper.rs, refuser au lancement tout type resté tmux (`gemini`, `gclaude`, `--`) sans pane attesté, avec diagnostic nommant `bridget spawn` ; refuser `bridget claude` sans terminal ou PTY. Test unitaire voisin dans wrapper.rs ; mise à jour des tests `who` de crates/bridget-daemon/src/cli.rs qui supposaient `tmux` pour Claude. Observable : `bridget who` sur une session 097 affiche `claude | claude_pty | cli | —`.

## US5 — Fermeture et pannes (P1)
- [x] T009 [US5] Oracles et implémentation dans le test 097 et claude_interactive.rs : sortie normale, `SIGTERM`/`SIGHUP` du wrapper, enfant tué → terminal restauré, présence déconnectée, code relayé ; daemon arrêté puis relancé → même identité, notification « reconnecté » remise par le PTY. Observable : zéro processus survivant, `stty` du harnais identique avant/après.

## US2 bis — Journal et attach (FR-09711)
- [x] T010 [US2] Dans crates/bridget-daemon/src/wrapper.rs, alimenter le journal interactif Claude depuis le transcript déjà localisé (`ClaudeTranscriptLocator`) : `turn_start {from:"human"}` pour un message utilisateur non issu de Bridget, `update {kind:"text"}` pour le texte assistant, `turn_end` au résultat, corps bornés au plafond d'injection ; test unitaire sur fixture JSONL. Observable : `bridget attach <UUID>` rend les deux origines sur une session 097.

## US1 — Claude géré recetté (P1)
- [x] T011 [US1] Corriger le gate `gate_reel_claude_stream_json_reponse_liee_et_attach` dans crates/bridget-daemon/tests/core_089_native_test.rs : plus de lecture du trousseau ; `start_wrapper` reçoit `HOME=<home réel>` et `USER` dans `extra_environment` par-dessus l'isolation (BRIDGET_HOME, socket, TMPDIR, XDG restent privés) ; registre privé avec `--setting-sources ""`, `--strict-mcp-config`, `--tools ""`, modèle et effort explicites ; garde `BRIDGET_CLAUDE_NATIVE_GATE=1` et `BRIDGET_TEST_CLAUDE_BIN`. Observable : sans la garde, le test reste ignoré ; avec un HOME isolé, le refus « Not logged in » est nommé, pas masqué. Observable : compilation et exécution `--include-ignored` documentées.
- [x] T012 [US1] Exécuter la recette réelle gérée (un tour, compte local) : demande suivie → `answered`, journal attachable, arrêt < 10 s, zéro survivant ; consigner preuves expurgées, durée et modèle dans specs/097-claude-sans-tmux/implementation.md. Si le compte est indisponible : consigner le refus réel, statut `Blocked` pour cette tâche.

## Consolidation
- [x] T013 Aligner README.md, README.en.md, skills/bridget/SKILL.md et skills/bridget/references/commandes.md : `bridget claude` sans tmux, présence `claude_pty`, refus sans terminal, différence interactif/géré/attach ; rédiger docs/decisions/033-claude-interactif-pty.md (contexte, décision, conséquences, statut) ; vérifier que tests/features/097-claude-sans-tmux.feature reflète les scénarios livrés.
- [x] T014 Recette réelle interactive : `bridget claude` dans iTerm sans tmux, envoi depuis un Codex connecté, réponse liée au ledger, rejeu de l'incident du 2026-09-13 (SC-09702, SC-09706) ; preuves dans implementation.md. Demander la validation manuelle de l'humain.
- [x] T015 Gates finaux en environnement privé (umask 077) : fmt, clippy `-D warnings`, suite complète workspace, tests 097/090/089 ciblés ; aucun test supprimé ; compteurs dans implementation.md et statut de la spec.

Dépendances : T001 → T002 → T003 → T004 → T005 → T006 → (T007, T008, T009, T010 en parallèle sur fichiers disjoints après T006) → T011 → T012 → T013 → T014 → T015. T011 est indépendant de T004-T010 et peut démarrer après T001.

Stratégie : MVP = T001-T006 (session interactive joignable) ; incréments US3/US4/US5/journal ; recette gérée ; documentation ; gates. Article XX : T002 et T006 retirent du code (validation dupliquée, contexte tmux Claude) ; T003 et T005 ajoutent deux unités justifiées par la spec ; T010 réutilise une source existante plutôt qu'un nouveau flux.
