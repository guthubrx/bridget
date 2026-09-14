# Journal d'implémentation — 098 Adaptateur t3code

## Métadonnées
- Spec : 098-t3code-adapter · Branche : session-098-t3code-adapter · Démarré : 2026-09-14 · Statut : In Progress

## T001 — Sonde de réalité (09:28–09:31)
- Prérequis posés : CLI `t3` 0.0.40 installé (`npm i -g t3`), application « T3 Code (Alpha) » 0.0.40 démarrée (`open -a`), `server-runtime.json` présent sur 127.0.0.1:3773.
- Résultats complets dans research.md « Sonde T001 ». Verdict : contrat conforme ; trois ajustements du plan (corrélation par rang FIFO, fin de vie par `archivedAt`, changement par `updatedAt`/`latestTurn`).
- Coût : 3 tours Claude haiku sur le compte de l'humain ; fil de sonde « Sonde Bridget 098 » archivé dans le projet 63.studio-horizon ; session t3code de sonde révoquée (0 restante).

## T002 — Dépendances et baseline
- `minreq = { version = "2", default-features = false }` (crate daemon), `uuid` avec `v5` (workspace) ; `cargo fetch` puis build vert ; fmt et clippy verts. La suite complète est celle de `main` au départ du worktree (1328/0/56) ; elle sera rejouée en T014.

## T003 — Contrat t3code
- crates/bridget-daemon/src/t3code_contract.rs : lecture validée de `server-runtime.json` (boucle locale seule, PID vivant, adresse reconstruite), snapshot, détail paginé, `thread.turn.start`, `DispatchResult`, sessions `t3 auth` (JSON, libellé) ; client HTTP `minreq` avec délai borné ; 6 tests unitaires sur les formes réelles capturées en T001 et sur des formes malformées (refus nommés par champ).

## T005 — Primitives du wrapper rendues visibles
- `deliver_idempotent_to_interactive`, `IdempotentDeliveryTracker::{open, open_at}`, `connect_and_register_with_domain_at`, `send_wrapper_message`, `record_interactive_turn`, `AttachRelayWorker::{start, subscribe, unsubscribe, shutdown, reset_generation}` en `pub(crate)` ; comportement inchangé.

## T004 — Faux t3code et oracles
- crates/bridget-daemon/tests/fixtures/t3code_server_098.py : serveur HTTP local (fichier runtime, snapshot, détail paginé par tours, dispatch dédupliqué par `commandId`, un tour à la fois avec file FIFO, archivage) et faux CLI `t3 auth session issue|list|revoke` consigné dans `fake-t3.log` ; routes `/__test/*` (message humain, archivage, nouveau fil, 401 forcés, lenteur).
- crates/bridget-daemon/tests/t3code_098_test.rs : trois oracles (install sans CLI ; scénario complet US1→US4 : idempotence, agent `claude|t3code|cli` nommé « Alpha », réponse liée, humain intercalé, journal sans rejeu, 401 renouvelé une fois avec révocation, fil `codex` ajouté, archivage, uninstall ; 401 répété → `auth_failed`). Verts en 9 s.
- Écarts corrigés par les oracles : la connexion du wrapper porte un délai de lecture d'1 s (le lien le retire), la localisation n'est retenue par le daemon que sous tmux (assertion retirée, `status` porte le titre), les agents archivés restent listés `stopped` (filtre `connected`).

## T006–T011 — Module t3code.rs (commande, pont, corrélation, journal, ré-authentification)
- crates/bridget-daemon/src/t3code.rs (`bridget t3 install|status|uninstall|serve`, enregistré dans cli.rs/lib.rs) : état 0600 sous `<BRIDGET_HOME>/t3code` (token.json, manifest.json, status.json, threads/<id>.json), service LaunchAgent/systemd écrit par le module, rollback (session révoquée, fichiers retirés) si le service échoue, récupération par libellé si le reçu JSON est illisible.
- Pont : sondage du snapshot (3 s, `BRIDGET_T3_POLL_MS`), un lien par fil vivant avec session fournisseur (connexion daemon dédiée, `stable_uuid`, `PresenceMode::Cli`, `JournalReady`, `DisplayNameSet` = titre), lien fermé à l'archivage ou à la disparition, recréé après 5 s s'il meurt ; `Heartbeat` à chaque tour.
- Remise : attente bornée d'un fil sans `activeTurnId` (`BRIDGET_T3_TURN_WAIT_SECS`), `thread.turn.start` avec `commandId` = identifiant du message, `pending` écrit avant retour ; `DeliverIdempotent` accusé seulement après dispatch (tracker réutilisé).
- Corrélation `correlate()` par rang FIFO après l'ancre (dernier tour connu au dispatch), réponse liée `in_reply_to`, page élargie jusqu'à 320 tours si l'ancre manque, abandon journalisé après trois lectures sans le message.
- Journal : premier regard = repère (historique mémorisé, non rejoué), puis `turn_start {from:"human"}`, `update {kind:text}` (streaming terminé), `turn_end` (tour clos) ; état borné à 1 000 identifiants.
- Session partagée `Session::call` : un renouvellement par génération sur 401, rejoué une fois ; second 401 → `auth_failed` publié et pause 60 s ; serveur injoignable → liens fermés, attente du runtime.
- Tests unitaires `spec098_*` (6) : rang FIFO avec humain intercalé, attente de fin de tour, message absent, identité stable v4, type d'agent, état borné 0600.

## T012 — Documentation
- README.md / README.en.md (section 098), skills/bridget/SKILL.md (tableau + conduite), skills/bridget/references/commandes.md (ligne `t3` + section), docs/decisions/034-adaptateur-t3code.md, tests/features/098-t3code-adapter.feature (nom, rang, humain intercalé, 401), quickstart révision 3, reuse-audit (4 arbitrages ajoutés).

## T013 — Recette réelle (10:39–10:46)
- Conditions : t3code 0.0.41-nightly démarré par `t3 --mode web --no-browser` (l'application n'était pas lancée), CLI `t3` 0.0.40, daemon Bridget réel du poste, binaire debug du worktree, `bridget t3 install --no-service` puis `bridget t3 serve`.
- Résultat : les quatre fils du poste sont apparus dans `bridget who` sous leur titre (« Présentation du projet », « Comprendre T3 Code », « New thread », « Recette Bridget 098 »), type `codex`/`claude`, transport `t3code`, mode `cli`, état `connected`. Un fil dédié « Recette Bridget 098 » (claude-sonnet-5, `approval-required`) a été créé pour ne toucher aucune conversation existante.
- Mission envoyée par `bridget_send` depuis l'agent Claude de cette session : remise `[reçu]` dans le registre, tour exécuté dans t3code (9,8 s), réponse renvoyée par le pont et demande close. L'agent du fil a répondu en refusant la consigne à réponse fixe qu'il jugeait non vérifiable : comportement sain de sa part, sans effet sur le transport, qui est prouvé de bout en bout.
- Trois écarts réels trouvés et corrigés (détail dans research.md « Sonde T013 ») : `host` absent en mode web, fil neuf sans session fournisseur, `runtimeMode`/`interactionMode` exigés par la route HTTP. Chacun est couvert par un test (contrat, unitaire, oracle du fil neuf dans le faux serveur).
- Nettoyage : fil de recette archivé, `bridget t3 uninstall` (session révoquée, état effacé), trois sessions t3 de travail révoquées (seule « T3 Code Desktop » de l'humain subsiste), serveur t3code arrêté. Aucun fichier de t3code modifié.
- Écart avec la tâche : la mission part de l'agent Claude de la session, pas de `codex-h3` ; l'oracle visé (un agent Bridget tiers joint un fil t3code et reçoit la réponse liée) est identique.

## T014 — Gates
- Faux échec d'environnement identifié : `daemon_guard_nettoie_apres_une_panique_injectee` appelle `lsof`, qui vit dans `/usr/sbin` et manquait au `PATH` de la recette de test ; avec `/usr/sbin`, le test est vert. La recette de gate ajoute donc `/usr/sbin`.

## Contre-revue #2 et corrections (bridget-revue, BLOCKED → corrigé)
- Six objections sur l'implémentation, cinq retenues et corrigées, une close par la revue (détail et preuves dans adversarial-review-bridget-revue.md, tour 4).
- Corrections : corrélation prouvée au lieu d'ordinale (nouveau verdict `Ambiguous`), attente écrite avant le dispatch avec identifiants déterministes, identifiant de réponse stable et sauvegarde après chaque décision, verrou `auth_failed` après refus d'une session neuve, aucune session laissée orpheline (révocation immédiate, `orphan-sessions.json`, `uninstall` qui refuse d'effacer sans révocation acquise, rollback du manifeste).
- Défaut trouvé en écrivant le test réclamé par l'objection 6 : la politique du fil était figée à l'ouverture du lien. Elle est désormais lue au moment du dispatch, ce qui supprime aussi un champ d'état.
- Tests ajoutés : 4 unitaires (tour sans réponse, tour de sous-agent, fil au repos sans tour, identifiant de réponse stable) et 2 renforcements d'oracle (verrou d'émission dans la durée, politique changée entre deux sondages). 17 tests unitaires et 3 oracles verts.
