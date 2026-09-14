# Tâches 098 — Adaptateur t3code (option A, plan révision 3)

## Préparation
- [x] T001 Sonde de réalité sur l'application t3code installée (0.0.40) : démarrer t3code, lire `~/.t3/userdata/server-runtime.json`, vérifier `GET /api/orchestration/snapshot`, `GET …/threads/:id` (pagination `turnLimit`/`beforeCursor`, `streaming`, `latestTurn.assistantMessageId`, `threadSequence`) et `POST /api/orchestration/dispatch` avec un jeton émis par `t3 auth session issue --subject bridget --label bridget-sonde --json` (CLI `t3` à installer par npm si absent) ; observer un `dispatch` pendant un tour actif ; révoquer la session de sonde. Consigner dans specs/098-t3code-adapter/research.md « Sonde T001 » : formes exactes, écarts avec le dépôt, verdict. Si une route manque sur 0.0.40 : STOP et arbitrage utilisateur avant T004.
- [x] T002 Baseline verte du worktree en environnement privé (umask 077) : fmt, clippy, suite workspace ; ajouter `minreq = { version = "2", default-features = false }` et la fonctionnalité `v5` de `uuid` dans Cargo.toml ; `cargo build --offline --locked` vert ; consigner dans implementation.md.

## Fondations (bloquantes pour US2 à US4)
- [x] T003 Créer crates/bridget-daemon/src/t3code_contract.rs : lecture validée de `server-runtime.json` (port, pid ; adresse reconstruite `http://127.0.0.1:<port>` ; refus origine non locale, PID absent, forme inconnue), types validés champ par champ pour snapshot, détail paginé et `DispatchResult`, construction de `thread.turn.start` (`commandId`, `messageId`), commandes `t3 auth session issue/list/revoke` (JSON). Tests unitaires : fixtures JSON conformes et malformées → refus nommés par route et par champ.
- [x] T004 Écrire crates/bridget-daemon/tests/fixtures/t3code_server_098.py (faux t3code HTTP : fichier runtime, snapshot avec `updatedAt`/`latestTurn`/`archivedAt`, détail paginé avec `streaming`/`latestTurn`, `dispatch` avec reçus par `commandId`, modes 401/500/délai, tour actif simulé, archivage, message humain intercalé, fil de 2 000 messages, faux `t3 auth` JSON) et les oracles rouges de crates/bridget-daemon/tests/t3code_098_test.rs pour US1 à US4. Observable : tests rouges pour cause fonctionnelle.
- [x] T005 Rendre `deliver_idempotent_to_interactive` et `IdempotentDeliveryTracker` utilisables hors du wrapper (`pub(crate)`) dans crates/bridget-daemon/src/wrapper.rs, sans changer leur comportement ; tests existants inchangés et verts.

## US1 — Installer et retirer en une commande (P1)
- [x] T006 [US1] Créer crates/bridget-daemon/src/t3code.rs : `install` (jeton par CLI officielle avec libellé unique, reçu JSON, récupération par `client.label` si reçu illisible, état 0600, manifeste d'installation, service via le moteur 095, rollback par étape), `status`, `uninstall` (révocation par identifiant, retrait du service, effacement) ; enregistrer `t3` dans crates/bridget-daemon/src/cli.rs et crates/bridget-daemon/src/lib.rs. Observable : oracles US1 verts (idempotence, refus nommés, rollback, zéro écriture dans `~/.t3`).

## US2 — Chaque fil est un agent visible et joignable (P1)
- [x] T007 [US2] Dans t3code.rs, le pont `serve` : sondage borné du snapshot, une connexion daemon par fil (`protocol = t3code`, `PresenceMode::Cli`, UUID v5 stable, domaine = dossier), `JournalReady`, retrait à l'archivage, restauration après redémarrage. Observable : oracles US2 verts (`who` : deux agents `t3code`, retrait < 10 s, mêmes UUID après redémarrage).

## US3 — Remise dans le fil et réponse liée (P1)
- [x] T008 [US3] File et worker par fil : attente bornée de l'absence de `activeTurnId`, `dispatch` avec `commandId` = identifiant de remise et `messageId` du pont, accusé après `DispatchResult`, 401 → un renouvellement puis un essai, indéterminé sur refus/délai/panne/déconnexion, boucle de connexion jamais bloquée. Observable : un seul `dispatch` après rejeu ; deux demandes concurrentes → deux tours successifs ; déconnexion pendant l'attente → indéterminé.
- [x] T009 [US3] Corrélation durable et réponse liée : section `pending` du fichier d'état du fil `t3code-state.json` écrite avant le `dispatch`, reprise au démarrage ; réponse liée envoyée seulement pour le tour assistant de même rang que le `messageId` du pont (appariement FIFO message utilisateur ↔ tour, T001), `streaming = false` et tour `completed` ; rang non établi → demande laissée ouverte, ambiguïté journalisée. Observable : `answered` dans le cas non ambigu, demande ouverte dans le cas ambigu, redémarrage entre accusé et réponse sans perte.

## US4 — Journal, attach, ré-authentification (P2)
- [x] T010 [US4] Projection du journal : repère d'installation, séquence durable par fil, lecture paginée seulement si `updatedAt` ou `latestTurn` a changé, événements `turn_start {from:"human"}` / `update {kind:text}` / `turn_end` dans l'ordre du fil, lacune annoncée ; `bridget attach` admis. Observable : fil achevé avant le premier passage et fil créé pendant l'arrêt projetés une fois ; fil de 2 000 messages lu par pages ; fil inchangé non relu.
- [x] T011 [US4] Ré-authentification et panne : 401 répété → un renouvellement puis état d'échec explicite dans `status` et le journal ; coupure puis reprise du faux serveur → identités conservées, zéro doublon. Observable : oracles US4 verts.

## Consolidation
- [x] T012 Documentation : README.md, README.en.md, skills/bridget/SKILL.md, skills/bridget/references/commandes.md (intégration, prérequis CLI `t3`, portées administratives du jeton, absence d'émission depuis l'agent en v1, course résiduelle, retrait) ; docs/decisions/034-adaptateur-t3code.md ; tests/features/098-t3code-adapter.feature aligné sur les scénarios livrés.
- [x] T013 Recette réelle (SC-09805) : t3code du poste démarré, `bridget t3 install`, fil Claude ouvert dans l'application, mission envoyée depuis `codex-h3`, message visible dans l'application, réponse liée reçue ; preuves expurgées dans implementation.md ; validation manuelle de l'humain demandée.
- [x] T014 Gates finaux en environnement privé (umask 077) : fmt, clippy `-D warnings`, suite complète workspace ; aucun test supprimé ; compteurs dans implementation.md et statut de la spec.

Dépendances : T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014. T001 est un gate : une route absente sur l'application réelle arrête tout avant T004.

Stratégie : MVP = T001-T007 (fils visibles et joignables) ; puis remise et réponse liée ; puis journal et robustesse ; documentation ; recette réelle ; gates. Article XX : T005 rend visible sans dupliquer ; T003 concentre toute dépendance externe en un module ; aucune écriture dans t3code.

## Convergence (après contre-revue #2)
- [x] C001 Corrélation prouvée : n'apparier que si les messages utilisateur et les tours assistants se correspondent un pour un après l'ancre ; sinon `Waiting` (fil au travail) ou `Ambiguous` (fil au repos) avec journalisation. Tests : tour interrompu sans texte, tour de sous-agent.
- [x] C002 Attente durable écrite avant le dispatch, identifiants de message et de réponse déterministes, état sauvegardé après chaque décision.
- [x] C003 Verrou `auth_failed` : aucune nouvelle émission après le refus d'une session neuve. Oracle : compte d'émissions stable dans la durée.
- [x] C004 Aucune session administrative orpheline : révocation immédiate si la persistance échoue, consignation des échecs, `uninstall` refusant d'effacer sans révocation acquise, rollback du manifeste.
- [x] C005 Politique du fil lue au moment du dispatch, jamais mise en cache. Oracle : politique changée entre deux sondages.
