# Journal d'implémentation — 097 Claude natif et interactif sans tmux

## Métadonnées
- Spec : 097-claude-sans-tmux · Branche : session-097-claude-sans-tmux · Démarré : 2026-09-13 · Statut : In Progress

## T001 — Baseline verte
- Le code de ce worktree est byte-identique à l'arbre vérifié avant les commits `d9d790a5`/`e39ed91d` (main), seul un fichier de test avait été corrigé et cette correction est incluse.
- Commande (racine privée `/private/tmp/b097.DL9BqV`, umask 077) :
  `env -i HOME=$R/provider BRIDGET_HOME=$R/state BRIDGET_SOCKET=$R/state/bridget.sock TMPDIR=$R/tmp CARGO_HOME=… RUSTUP_HOME=… PATH=… perl -e 'alarm 900; exec @ARGV' cargo test --offline --locked --workspace --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4`
- Résultat 2026-09-13 19:56 : 1302 réussites, 1 échec (test de pied de page attach comparant des octets colorés) ; après correction du test : module attach 112/112, total 1303/0/55 ignorés. `cargo fmt --all --check` : 0 ; `cargo clippy --workspace --all-targets --features bridget-daemon/test-support -- -D warnings` : 0.
- Piège consigné : sans umask 077 et racine courte sous /private/tmp, 46 à 162 faux échecs (sockets 755 refusées, chemins de socket trop longs).

## T002 — Garde d'injection partagée
- `validate_injection_content` (taille, ESC) extraite de `validate_tmux_content` dans crates/bridget-transport/src/tmux.rs ; tmux conserve ses motifs interdits par-dessus. `attach::terminal_geometry` exposé en `pub(crate)`.
- `cargo test -p bridget-transport --lib tmux::` : 11/11 (nouveau test : la garde commune accepte `send-keys`, refuse ESC et le dépassement).

## T003 — PtyTransport
- crates/bridget-transport/src/pty.rs : écriture dans une copie du maître PTY, collage encadré puis CR après 150 ms, écriture bornée par `poll` (5 s), `is_alive` par `kill(pid, 0)`. Aucune lecture d'écran.
- Tests unitaires 5/5 sur un `openpty` local : octets exacts, CR différé, corps ESC sans écriture, `AgentDead` sans écriture, copie indépendante du maître.

## T004 — Harnais et test rouge
- crates/bridget-daemon/tests/fixtures/claude_interactive_097.py : vrai daemon, vrai `bridget claude`, faux `claude` dans un PATH privé (journalise args, octets reçus, taille de fenêtre, code de sortie). Test Rust crates/bridget-daemon/tests/claude_interactive_097_test.rs.
- Rouge fonctionnel constaté avant implémentation : présence `tmux/tmux`, pas de refus sans terminal.

## T005 — Session PTY
- crates/bridget-daemon/src/claude_interactive.rs : `check_terminal`, `PtySession` (`openpty`, `setsid` + `TIOCSCTTY`, terminal hôte `cfmakeraw` restauré en `Drop`, relais stdin→maître et maître→stdout, `SIGWINCH`→`TIOCSWINSZ`, TERM/INT/HUP relayés à l'enfant, EOF humain → `SIGHUP`). 3 tests unitaires.

## T006 — Raccord du wrapper
- crates/bridget-daemon/src/wrapper.rs : Claude hors `--equipier` passe par `PtySession` ; présence `claude_pty` + `PresenceMode::Cli` (initial et deux reconnexions) ; `transport: Option<Box<dyn Transport>>` (PTY ou tmux) ; types tmux sans pane refusés avant présence ; bypass de permissions implicite retiré pour Claude interactif (FR-09710).
- Recette harnais : `cargo test -p bridget-daemon --features test-support --test claude_interactive_097_test` : 4/4 (relais sortie, présence `claude | claude_pty | cli | —`, args sans bypass / bypass explicite relayé, frappe relayée, SIGWINCH 40×100 propagé, remise collée + CR avec phase `acked` et `turn_start`, terminal restauré, code de sortie 0 et 7 relayés, présence retirée, zéro survivant). Tests lib `wrapper:: cli:: claude_interactive::` : 147/147.
- Les fixtures `who` de cli.rs qui décrivent une présence Claude historique en mode tmux restent valides : elles rendent une présence passée, pas un lancement.

## T007 — Saisie préservée, refus, notification
- Harnais `--busy` : saisie partielle « saisie en cours » précède le collage sans effacement ; un seul CR de remise ; corps avec séquence de contrôle refusé par le CLI (H-001) avant le daemon, zéro octet écrit ; le refus au niveau du transport (ESC, dépassement) est prouvé par les tests unitaires de pty.rs. Une notification idempotente suit la même voie (deuxième collage).
- Écart assumé : le scénario « DeliveryIndeterminate observé côté daemon pour un corps hostile » n'est pas atteignable par le CLI, qui refuse en amont ; l'état indéterminé est couvert par `AgentDead`/échec d'écriture au niveau transport.

## T008 — Présence et refus explicites
- `bridget gemini` sans serveur tmux (répertoire de sockets vide) : refus « aucun pane tmux … bridget spawn gemini » avant toute présence. `who` d'une session 097 : `claude | claude_pty | cli | —` ; libellé « Claude Code » pour le canal `claude_pty` dans attach.

## T009 — Fermeture et pannes
- `--sigterm` : SIGTERM au wrapper → relayé au fournisseur, terminal restauré, code de sortie non nul (128 + signal, correction d'un `exit(0)` historique), présence retirée, zéro survivant.
- `--daemon-restart` : arrêt puis relance du daemon → wrapper vivant, même identité, transport `claude_pty`, notification « reconnecté » remise par le PTY (chemin `Disconnect` unifié avec le chemin EOF via `notify_reconnected`), remise suivante `acked`.

## T010 — Journal relayé
- `RuntimeProbe::drain_transcript_journal` relit les lignes complètes du transcript Claude (offset persistant, remise à zéro sur changement de fichier, résolution du chemin par battement tant qu'aucun transcript n'existe) et `claude_transcript_journal_events` les traduit : `turn_start {from:"human"}`, `update {kind:"text"}`, `turn_end` ; remises Bridget (💬) et résultats d'outils exclus ; sidechains ignorés.
- Tests unitaires 2/2 ; harnais `--basic` : `QUESTION-FAKE-097` (tour humain), `REPONSE-FAKE-097` (texte assistant) et `turn_end` présents au journal en moins de 5 s, remise Bridget journalisée une seule fois.

## T011 — Gate Claude géré corrigé
- crates/bridget-daemon/tests/core_089_native_test.rs : plus de lecture du trousseau ; opt-in `BRIDGET_CLAUDE_NATIVE_GATE=1`, `BRIDGET_TEST_CLAUDE_BIN/HOME/USER/MODEL` ; HOME réel et `USER` transmis au wrapper par-dessus l'isolation ; registre privé neutralisant la configuration utilisateur ; `CLAUDE_CODE_OAUTH_TOKEN` désormais interdit. Verdict de modèle aligné sur la politique 091 : un équipier lancé sans ordre de lancement n'a pas de modèle déclaré (colonne « — ») ; l'absence de `model_mismatch` (servi = épinglé) est l'observable.

## T012 — Recette réelle du Claude géré (2026-09-13, 21:07–21:12 CEST)
- Commande : racine privée `mktemp -d /private/tmp/b097g.XXXXXX`, umask 077, `env -i HOME=$R/provider BRIDGET_HOME=$R/state BRIDGET_SOCKET=$R/state/bridget.sock TMPDIR=$R/tmp USER=moi … BRIDGET_CLAUDE_NATIVE_GATE=1 BRIDGET_TEST_CLAUDE_BIN=/Users/moi/.local/bin/claude BRIDGET_TEST_CLAUDE_HOME=/Users/moi BRIDGET_TEST_CLAUDE_USER=moi perl -e 'alarm 240; exec @ARGV' cargo test --offline --locked -p bridget-daemon --features test-support --test core_089_native_test claude -- --include-ignored --test-threads=1 --nocapture`.
- Modèle `claude-haiku-4-5-20251001`, un tour, compte claude.ai de l'humain. Résultat : demande suivie acquittée, réponse liée « BRIDGET-089-CLAUDE-OK » corrélée à l'identifiant exact, journal attaché (fragment reçu après `Subscribed`), présence `claude_stream_json` sans écart de modèle, `who` cohérent, arrêt par `stop_session` ; 3 exécutions consécutives réussies en 6,55 s, 4,51 s et 4,49 s.
- Échecs préalables, expliqués : deux exécutions ont buté sur l'ancienne assertion « modèle déclaré » (politique 091, voir T011) ; une exécution intermédiaire a échoué sans message capturé juste après la correction de l'assertion, puis trois réussites consécutives. Un processus Claude géré a survécu à la première exécution en échec (test paniqué avant nettoyage) et s'est terminé seul ; aucun survivant après les exécutions réussies.
- Aucune clé d'API, aucun secret lu ; transcripts de recette écrits sous le HOME réel (`~/.claude/projects/-private-tmp-bid-…`).

## T013 — Documentation et ADR
- README.md / README.en.md : section « Claude Code interactif, sans tmux (session 097) » avant la section 090 ; skills/bridget/SKILL.md : lignes de table pour `bridget claude` et la reprise ; skills/bridget/references/commandes.md : lignes `claude`, `gemini`, `gclaude` ; docs/decisions/033-claude-interactif-pty.md ; tests/features/097-claude-sans-tmux.feature relu contre les scénarios livrés.

## T014 — Recette réelle interactive (vraie TUI Claude Code, 2026-09-13 21:33 CEST)
- Harnais `--live` (test `vraie_tui_claude_code_recoit_un_message_et_repond`, ignoré sans opt-in) : vrai `bridget claude` sous le PTY du harnais, vrai Claude Code 2.1.270, compte claude.ai de l'humain (HOME réel, `USER`), modèle `claude-haiku-4-5-20251001`, répertoire de travail `/Users/moi/Nextcloud/10.Scripts/64.bridget`, état Bridget privé. Résultat 14,15 s : présence `claude | claude_pty | cli`, message Bridget collé dans la vraie TUI, réponse « PONG-097 » lue dans le transcript et relayée au journal (`update {kind:text}`), sortie par Ctrl-D, terminal restauré, code 0, présence retirée.
- Trois défauts révélés et corrigés par cette recette :
  1. le prompt initial positionnel était avalé par l'option variadique `--allowedTools` (TUI muette, aucun transcript) : il est désormais placé en tête des arguments ;
  2. le localisateur de transcript épinglait un transcript préexistant modifié (une autre session Claude du même projet) quand le nôtre n'existait pas encore : hors reprise (`--resume`/`--continue`), seuls les fichiers nés après le lancement sont admis ; en reprise, l'historique n'est pas rejoué (offset au lancement). Test unitaire de régression ajouté ;
  3. `/exit` est une commande utilisateur redéfinie sur ce poste : la sortie native passe par Ctrl-D puis Ctrl-C.
- Observation : le transcript de la TUI ne contient pas de tour pour le prompt initial « Tu es l'agent … » ; la remise et la réponse ne dépendent pas de lui.
- Reste à valider par l'humain (SC-09706) : rejeu de l'incident réel Codex → Claude depuis iTerm avec le binaire de cette branche.

## Convergence (Phase 6, passage 1) — 2026-09-13 21:42
Lecture du code produit contre spec.md, plan.md, tasks.md et checklists. Issue : **CONVERGED**, `tasks.md` inchangé.

| Exigence | Réalisation | Preuve |
|---|---|---|
| FR-09701 héritage du compte, aucune clé | gate réel HOME réel + `USER`, `CLAUDE_CODE_OAUTH_TOKEN` interdit ; refus nommé `api_error` du pilote | tests/core_089_native_test.rs:91-116 ; transport claude_stream_json.rs:2132-2157 ; research.md (sondes) |
| FR-09702 recette gérée complète | T012 : 3 exécutions vertes, réponse liée, journal, arrêt | implementation.md §T012 |
| FR-09703 interface native dans un PTY | `check_terminal`, `PtySession::spawn` | src/claude_interactive.rs:30, :176 ; src/wrapper.rs:1880 |
| FR-09704 relais frappe/affichage/taille/signaux | `relay`, `SIGWINCH`, `RawTerminal::drop` | src/claude_interactive.rs:114, :238, :80 ; harnais `--basic` |
| FR-09705 remise sans détruire la saisie, accusé après écriture | collage encadré + CR, tracker idempotent inchangé | transport pty.rs:109-122 ; wrapper.rs:751 ; harnais `--busy` |
| FR-09706 rappels et notifications même voie | branche `Deliver` et `notify_reconnected` sur le même transport | src/wrapper.rs:811 ; harnais `--daemon-restart` |
| FR-09707 refus sans voie de remise | refus sans terminal, refus tmux sans pane | src/wrapper.rs:1719 ; claude_interactive.rs:30 ; tests 097 |
| FR-09708 canal réel dans l'annuaire | `claude_pty` + `cli`, libellé « Claude Code » | src/wrapper.rs:1726 ; src/attach.rs:1700 ; harnais présence |
| FR-09709 aucun protocole nouveau | `protocol.rs` absent du diff | `git diff --stat` |
| FR-09710 permissions humaines | bypass implicite retiré, explicite relayé | src/wrapper.rs:1843 ; harnais `--user-bypass` |
| FR-09711 journal des deux origines, attach | drainage du transcript, vocabulaire attach | src/wrapper.rs:1351, :1216 ; recette réelle §T014 |
| FR-09712 arrêt propre, identité conservée | `wait`, 128+signal, reconnexion | claude_interactive.rs:289 ; wrapper.rs:2367 ; harnais `--sigterm`, `--daemon-restart` |
| FR-09713 documentation | README FR/EN, skill, références, ADR 033 | §T013 |

Non vérifié et assumé : deux sessions Claude interactives simultanées dans le même projet (identités distinctes par construction, transcript « né après » par session) ; rejeu de l'incident réel Codex → Claude depuis iTerm (validation humaine, SC-09706).

## T015 — Gates finaux (2026-09-13 21:41–21:48 CEST)
- `cargo fmt --all --check` : 0. `cargo clippy --offline --locked --workspace --all-targets --features bridget-daemon/test-support -- -D warnings` : 0.
- Suite complète : environnement privé `/private/tmp/b097f.*`, umask 077, `env -i … perl -e 'alarm 900' cargo test --offline --locked --workspace --features bridget-daemon/test-support --no-fail-fast -- --test-threads=4` : **1323 réussites, 0 échec, 56 ignorés** (baseline 1303/0/55 : +20 tests dont 8 intégration 097, 5 pty, 3 claude_interactive, 3 journal/localisateur, 1 garde tmux).
- Premier passage rouge (4 échecs) corrigé sans retirer de test : le test d'attache interactif utilise désormais un tmux factice qui atteste un pane (le refus sans pane est la nouvelle règle) ; les scénarios 097 sont sérialisés dans leur binaire et le contrôle de survivant cible la racine du scénario.
- Gates réels hors suite : recette Claude géré (3/3, §T012) et recette vraie TUI (§T014).

## Audit v14 (Phase 7, 2026-09-13 21:49–22:28 CEST) — session audits/2026-09-13/session-2026-09-13-097-claude-sans-tmux-01
- Mode fix, grille all sans 08, six auditeurs thématiques (sécurité, complexité, qualité, tests, duplication, minimalisme). Cycle 1 : 54 findings (0 CRITICAL, 7 HIGH, 19 MEDIUM, 28 LOW). Les 7 HIGH contre-audités (CONFIRMED) et corrigés sans commit :
  - SEC : descripteurs PTY sans `FD_CLOEXEC` → `FD_CLOEXEC` sur maître/esclave, copie du transport par `F_DUPFD_CLOEXEC` ; garde d'injection qui laissait passer ESC sans crochet → refus de tout caractère de contrôle hors LF/TAB et DEL ; course entre deux `bridget claude` simultanés (transcript du voisin) → `--session-id` imposé aux sessions neuves et transcript attendu `<dir>/<uuid>.jsonl`.
  - CPLX : relecture intégrale du transcript à chaque battement → lecture depuis l'offset.
  - TESTS : trois tests manquants ajoutés (maître non accueillant / délai d'écriture, échec d'injection → `DeliveryIndeterminate`, deux sessions simultanées). Le test du délai a révélé un blocage réel : `poll` annonçait un maître accueillant alors que l'écriture bloquait ; correctif induit : maître non bloquant côté remise et relais tolérant à `WouldBlock`.
- Cycle de scoring (lecture seule) : 47 findings ouverts (0 HIGH, 19 MEDIUM, 28 LOW), note globale A- (sécurité A, complexité A, qualité A-, tests C, duplication B+, minimalisme A). Potentiel minimalisme ~12 lignes. `validate_session.py` : 0 erreur. `audits/latest` pointé.
- Recette réelle vraie TUI rejouée après ces correctifs : verte (14,0 s) avec `--session-id` imposé ; harnais 097 8/8 ; transport 18/18 ; wrapper+claude_interactive 87/87.
- Convergence, passage 2 après les correctifs d'audit : aucune exigence de la spec n'est affectée (les correctifs renforcent FR-09705, FR-09711 et FR-09712) ; `tasks.md` inchangé, CONVERGED.
- MEDIUM à traiter en priorité (court terme) : nettoyage de l'enfant si `PtyTransport::from_master` échoue après le spawn (QUAL-003/SEC-005), fenêtre de recyclage de PID entre `wait` et l'arrêt de la boucle de signaux (SEC-004), duplication des deux boucles de reconnexion (DUP-002), code de sortie 128+signal étendu aux autres types interactifs (MIN-001), assertion exacte 128+signal (TEST-007).
- Gates finaux après audit (22:26–22:30 CEST) : fmt 0, clippy 0, suite complète workspace en environnement privé **1328 réussites, 0 échec, 56 ignorés** (+5 tests d'audit). Aucun commit effectué : diff à revoir par l'humain.
