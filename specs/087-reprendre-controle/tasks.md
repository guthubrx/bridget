# Tasks: Reprendre le contrôle

**Input**: Design documents from `/specs/087-reprendre-controle/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, reuse-audit.md (PASS)

**Tests**: inclus. La spec exige des preuves d'effet par levier (SC-001 à SC-006) et la constitution XV impose les gates ; chaque garde a un test d'effet et, pour les gardes de pause, un mutant qui doit rendre le test rouge.

**Organization**: par user story. Chemins relatifs à la racine du dépôt ; l'implémentation se fait dans le worktree serveur `/home/moi/bridget-referent/.worktrees/087-reprendre-controle`.

## Format: `[ID] [P?] [Story] Description`

- **[P]** : parallélisable (fichiers différents, aucune dépendance ouverte)
- **[Story]** : US1 pause, US2 focus, US3 boîte de réception, US4 budget, US5 visibilité

## Path Conventions

- Daemon : `crates/bridget-daemon/src/`, protocole : `crates/bridget-transport/src/protocol.rs`, cœur : `crates/bridget-core/src/`
- Maicie : `plugins/maicie/src/`, tests Maicie : `plugins/maicie/tests/`
- Interface : `crates/bridget-daemon/assets/ui/app.js` et `theme.css`
- Scénarios : `tests/features/`, décisions : `docs/decisions/`

---

## Phase 1: Setup

**Purpose**: réserver les ressources globales et poser les artefacts de traçabilité avant tout code.

- [x] T001 Réserver le numéro de migration Maicie v24 : `git fetch --all` puis `git grep -n "SCHEMA_VERSION: i64 = 24" $(git branch -r | tr -d ' ')` doit ne rien rendre ; consigner le résultat brut dans `specs/087-reprendre-controle/implementation.md` (section Métadonnées) avant de toucher `plugins/maicie/src/store.rs:58`
- [x] T002 [P] Écrire les scénarios Gherkin en français dans `tests/features/087-reprendre-controle.feature` : un scénario par acceptance scenario de US1 à US5 de `spec.md` plus l'essai adverse du quickstart §5 ; résultat observable : le fichier suit la forme `Fonctionnalité` / `Scénario` de `tests/features/086-projet-systeme-bridget-dogfooding.feature`
- [x] T003 [P] Rédiger `docs/decisions/027-etat-de-controle-et-voie-humaine-attestee.md` : contexte, décision (état de contrôle et boîte dans le daemon, focus et budget appliqués par Maicie, attestation humaine fabriquée par le daemon, borne de la garde identique à l'ADR-011), conséquences, statut Accepté ; vérifier que 027 est libre et ne pas reproduire les doublons 012 et 022

---

## Phase 2: Foundational

**Purpose**: contrat de protocole, tables et modules que toutes les stories utilisent.

**⚠️ CRITICAL**: aucune story ne commence avant T005 vert.

- [x] T004 Étendre `crates/bridget-transport/src/protocol.rs` selon `contracts/control-state-v1.md`, `contracts/human-inbox-v1.md` et `contracts/delegate-origin-v1.md` : constantes `CONTROL_STATE_CONTRACT_VERSION`, `HUMAN_INBOX_CONTRACT_VERSION` ; `ClientCapability::ControlStateV1`, `ServiceCapability::HumanInboxV1` ; enums fermés `ControlStateRefusal`, `HumanInboxRefusal`, `HumanInboxKind` avec `as_sql()` ; variantes `WrapperToDaemon::{ControlStateRead, ControlStateSet, HumanInboxDeposit, HumanInboxList, HumanInboxResolve, HumanInboxDecisions, HumanInboxAck}` et `DaemonToWrapper::{ControlState, ControlStateRejected, HumanInboxDeposited, HumanInbox, HumanInboxRejected, HumanInboxDecisionsBatch, HumanInboxAcked}` (dont le `kind` `human_route_replaced`) ; `ProjectRoundRefusal::ControlPaused` ; `DelegateOrigin` ; champs `origin` et `focus` sur `ServiceRequestPayload::Delegate` avec `required_contract_version()` mis à jour ; fixtures producteur/consommateur dans les tests de `protocol.rs` (aller-retour JSON de chaque trame neuve) ; résultat : `cargo test -p bridget-transport` vert avec le compte de tests annoncé
- [x] T005 Mettre à jour tous les initialiseurs `ServiceRequestPayload::Delegate { … }` du dépôt (`protocol.rs` tests, `plugins/maicie/src/guichet.rs:378`, `crates/bridget-daemon/src/mcp.rs`, `crates/bridget-daemon/src/cli.rs`, tests d'intégration) avec `origin: None, focus: None` ; gate : `cargo test --workspace --features test-support --no-run` sur le worktree, sortie brute conservée dans `implementation.md` ; résultat : compilation de tous les binaires de test, à l'exception documentée des deux tests d'intégration déjà cassés sur main (`mcp_injection_smoke_test`, `idempotency_crash_test`, cf. mémoire projet)
- [x] T006 Créer `crates/bridget-daemon/src/referent_control.rs` : DDL `control_state` (ligne unique) et `control_events` ajouté au batch d'ouverture de `crates/bridget-daemon/src/store.rs`, `read()`, `set(command_id, expected_generation, paused?, auto_objectives_cap?, reason, actor)` avec journalisation, bornes 1..=100, refus `GenerationMismatch`, `NothingToChange`, `BudgetOutOfRange` ; garde unique `admit_autonomous_effect(effect: AutonomousEffect, state) -> Admitted | Deferred { motif }` appelée par tous les puits daemon (ronde, relances, continuations) ; tests unitaires : valeurs par défaut, mutation et génération, refus, relecture après réouverture du store, table de vérité de la garde
- [x] T007 Créer `crates/bridget-daemon/src/human_inbox.rs` : DDL `human_inbox` avec `CHECK (kind IN (…))` produit par `HumanInboxKind::sql_in_clause()` (jamais recopié), `deposit(dedup_key, …)` idempotent parmi les items ouverts avec incrément d'`occurrences`, `list(state, limit)`, `resolve(item_id, choice, actor)` refusant un choix hors `options` et posant un `decision_id` UUID v4, `pending_decisions(producer, limit)` qui rend les décisions non acquittées **sans rien écrire**, `ack(decision_id)` idempotent posant `acked_at`, `close_self(item_id, reason)` ; tests unitaires : dédoublonnage, refus `ChoiceNotOffered`, `AlreadyResolved`, relève sans effet, acquittement idempotent, décision toujours relue tant qu'elle n'est pas acquittée
- [x] T008 Brancher les trames dans `crates/bridget-daemon/src/daemon.rs` (`handle_wrapper_message`) par des bras d'une ligne vers `referent_control` et `human_inbox` : `ControlStateRead` (Client, `ControlStateV1` ou `Lookup`), `ControlStateSet` et `HumanInboxList`/`HumanInboxResolve` (Client, `ControlStateV1`, principal humain vérifié par `resolve_sender_attribution`, sinon `HumanPrincipalRequired`), `HumanInboxDeposit`/`HumanInboxDecisions`/`HumanInboxAck` (Service, `HumanInboxV1`) ; tests dans `mod matrice_roles_tests` : chaque trame refusée hors de son rôle et de sa capacité, la façade MCP (`mcp.rs`, capacités `[SendIdempotent]`) ne peut pas muter l'état
- [x] T009 [P] Étendre `plugins/maicie/src/bridget_client.rs` : négociation des capacités `ControlStateV1` (rôle client de lecture) et `HumanInboxV1` (service), méthodes `read_control_state()`, `deposit_human_inbox(…)`, `fetch_human_decisions(limit)`, `ack_human_decision(decision_id)` ; tests avec le faux daemon des tests existants du module
- [x] T010 Créer `plugins/maicie/src/control.rs` : `ControlSnapshot { paused, auto_objectives_cap, read_at }` lu une fois par relève dans `open_store_with_reconciliation` (`plugins/maicie/src/main.rs:514`) et garde unique `admit_autonomous_effect(effect, snapshot) -> Admitted | Deferred { motif }` appelée par tous les puits Maicie (routines, réassignation, rejeu d'outbox, déblocage de dépendance, dispatch différé) ; en cas de daemon injoignable, snapshot `Unknown` qui **différe** toute ouverture automatique avec le motif `controle_inconnu` (jamais « pas de pause par défaut ») ; bloc `control` dans `maicie status --json` ; tests : table de vérité de la garde par effet et par état

**Checkpoint**: contrat, tables et lecture partagée prêts.

---

## Phase 3: User Story 1 - Mettre l'autonomie en pause (Priority: P1) 🎯 MVP

**Goal**: un geste suspend toute production autonome, persistant, visible, réservé au référent.

**Independent Test**: pause activée, une occurrence de routine et une fenêtre de ronde passent, zéro objectif et zéro réveil créés, les deux différés sont consignés ; redémarrage ; pause toujours affichée ; levée ; reprise.

- [x] T011 [US1] Ajouter `bridget control pause [--reason]`, `bridget control resume`, `bridget control status` dans `crates/bridget-daemon/src/cli.rs` : entrée et sortie doivent être un terminal (même refus que `plugins/maicie/src/main.rs:1313`), envoi de `ControlStateSet` avec `expected_generation` relu juste avant ; test du refus hors terminal et du rendu de `status`
- [x] T012 [US1] Garde de la ronde : dans `crates/bridget-daemon/src/daemon.rs:9887` rendre `ProjectRoundRefusal::ControlPaused` avant `PolicyDisabled` quand `control_state.paused`, enregistré par `record_project_round_dispatch` en `Refused` ; test d'effet : dispatch en pause → aucune remise, ligne `Refused { ControlPaused }` ; mutant : retirer la garde doit rendre ce test rouge, résultat consigné dans `implementation.md`
- [x] T013 [US1] Relances en pause : dans `collect_reminder_actions` (`daemon.rs:6914`), ne pas émettre les paliers 1 et 2 quand la pause est active, conserver le palier 3 (notification d'échec à l'émetteur) et consigner un événement `deferred { reason: pause }` ; test d'effet sur les trois paliers
- [x] T014 [US1] Continuations : chaque appelant de `reserve_governed_continuation` (`crates/bridget-daemon/src/managed_supervisor.rs:36`) passe `AutonomyRuntimeState::Paused` quand la pause est active ; test : en pause, la garde rend `GovernedContinuation::Budget(ExecutionBudgetOutcome::Paused)` et aucune continuation n'est réservée
- [x] T015 [US1] Routines : dans `plugins/maicie/src/routines.rs:440-500`, avant `delegate`, insérer une `RoutineOccurrence { state: Differee, reason: "pause" }` quand le snapshot est en pause, sans consommer le bucket ; test d'effet ; mutant
- [x] T016 [US1] Réassignation : ajouter la variante `ReductionReassignation::Differee { motif }` dans `plugins/maicie/src/domain.rs` (réducteur pur, entrée `ControlSnapshot`) et la consommer en `plugins/maicie/src/store.rs:6492` sans créer de génération ; test d'effet : chaîne pré-autorisée disponible mais pause active → zéro génération, réduction consignée ; mutant
- [x] T017 [US1] Outboxes en pause : dans `plugins/maicie/src/reconcile.rs` (`reconcile_pending`, `reconcile.rs:353-368`), passer chaque outbox par `admit_autonomous_effect` : celles d'objectifs d'origine `AutoGenerated` ne sont pas poussées tant que la pause est active, celles d'origine `HumanRequest` le sont toujours ; test d'effet : deux outboxes, une de chaque origine, pause → une seule remise ; mutant
- [x] T047 [US1] Déblocage de dépendance en pause : la matérialisation d'une outbox depuis `deferred_delegation_dispatch` à la clôture d'un prérequis (`plugins/maicie/src/store.rs:7724-7886`) passe par `admit_autonomous_effect` ; en pause la ligne différée est conservée avec le motif `pause` et rejouée à la levée ; test d'effet : prérequis clos pendant la pause → aucune outbox, ligne conservée ; après levée → outbox créée ; mutant
- [x] T018 [US1] Interface : routes `GET/POST /v1/control/state` dans `crates/bridget-daemon/src/ui.rs` via `ControlStateRead`/`ControlStateSet` sur la socket (aucune ouverture de base), bandeau en tête de l'écran principal dans `assets/ui/app.js` avec « Autonomie active » + bouton Pause ou « Pause depuis <durée> » + bouton Reprendre, style dans `theme.css` ; tests Node : projection de la durée, présence des libellés, absence de tout accès `Store::open` dans les nouvelles routes
- [x] T019 [US1] Persistance et pied d'annuaire : test d'intégration daemon (fixture `bridget daemon` de test) : pause, arrêt, redémarrage, `ControlStateRead` rend `paused: true` ; ligne `Contrôle : pause depuis … · budget …` ajoutée après `Daemon build-id` dans `crates/bridget-daemon/src/cli.rs:4193` ; test de rendu

**Checkpoint**: US1 livrable seule : le frein d'urgence existe et se voit.

---

## Phase 4: User Story 2 - Imposer un objectif prioritaire (Priority: P2 dans l'ordre d'exécution, P1 dans la spec)

**Goal**: une phrase et un projet ouvrent un objectif d'origine humaine attestée, marqué focus, délégué avec base gelée et identifiants calculés.

**Independent Test**: depuis l'interface, saisir une phrase, un objectif focus d'origine humaine existe, une délégation part avec `ReviewTarget` et le bloc `IDENTIFIANTS DE DÉPÔT`, une routine due est différée avec le motif `focus`.

- [x] T020 [US2] Daemon : dans le bras `ServiceRequest` (`daemon.rs:11167`), pour une charge `Delegate` : si l'émetteur attribué n'est pas le principal humain et que `origin` ou `focus` est présent → `ServiceRefusal::HumanOriginForbidden` ; si c'est le principal humain → ignorer tout `origin` reçu, enregistrer le message humain au ledger (`record_message_in_transaction`, émetteur `humain`, cible `maicie`, corps `goal`), calculer `canonical_request_sha256` sur les octets canoniques **sans** `origin`, `signature = human_message_content_seal(observed)`, puis déposer avec `origin.human` ; tests : dépôt humain porte une origine cohérente avec les fixtures de `plugins/maicie/tests/human_origin_attestation.rs` ; essai adverse : un client non humain fournissant `origin` est refusé et rien n'est déposé
- [x] T048 [US2] Alerte de reprise de route humaine : dans `replace_stale_ui_human_route` (`crates/bridget-daemon/src/daemon.rs:3169-3200`), si la route remplacée a un écrivain encore joignable et un `link_seen` plus récent que la fenêtre de heartbeat existante du daemon (constante de rétention de présence de `daemon.rs`, réutilisée, pas de nouveau seuil), déposer un item `human_route_replaced` (dedup `human-route:<conn_id>`) avec instance et canal de la nouvelle connexion, poussé sur le canal externe ; une route réellement morte ne produit rien ; tests d'effet dans les deux cas ; consigné dans `control_events`
- [x] T021 [US2] Migration Maicie v24 dans `plugins/maicie/src/store.rs` : tables `focus_queue`, `human_origin_consumptions` et `human_decisions_applied` (data-model.md), `SCHEMA_VERSION = 24`, consentement existant ; témoin de migration qui part d'une base fabriquée par le code v23 (`user_version = 23`), pas d'un DDL recopié, cf. règle de chantier ; test : une base v23 peuplée migre, une base v24 vierge s'ouvre
- [x] T022 [US2] Maicie guichet : `plugins/maicie/src/guichet.rs` parse `origin` et `focus` ; `plugins/maicie/src/greffe_service.rs` construit le permit par `ObjectiveOpeningPermit::human_request(…)` avec `AttestationConsumption` lu dans `human_origin_consumptions`, consomme le `message_id` dans la même transaction, refuse par `GuichetRefusalReason::HumanOriginInvalid` (motif interne journalisé) ; `focus` : inscription en `focus_queue` position 0 (`replace` recule l'ancien en 1) ou en fin de file (`queue`), refus si `focus` sans origine humaine valide ; tests : les sept `HumanOriginRefusal` produisent le même code public, rejeu du même `message_id` refusé, remplacement et mise en file
- [x] T023 [US2] Maicie `plugins/maicie/src/app.rs` `delegate` : pour un focus sans `review_target`, mesurer `origin/<branche par défaut>` du projet avec `plugins/maicie/src/review_git.rs` (`git ls-remote`) et construire `ReviewTarget` ; ajouter au corps produit par `delegation_instruction` (`app.rs:1729`) le bloc `IDENTIFIANTS DE DÉPÔT` avec `objective_id`, `delegation_id`, `message_id` ; tests : l'instruction contient les trois identifiants réels de la délégation créée, et la cible gelée mesurée
- [x] T024 [US2] Focus et travail automatique : routines → `Differee { reason: "focus" }` quand un focus est actif (`routines.rs`) ; `reconcile_pending` pousse les outboxes du focus avant toute autre ; les soumissions d'exécution issues d'une délégation focus portent `priority_class = "focus"` (`crates/bridget-core/src/execution.rs:99`), lu à l'admission dans `crates/bridget-daemon/src/daemon.rs` (`admit_starting_message_for_project`, `daemon.rs:7950-8000`) et ordonné dans `crates/bridget-daemon/src/execution_store.rs` (`execution_queue`) ; tests d'effet sur chaque point ; mutant sur la routine
- [x] T025 [US2] CLI `maicie focus status|close|queue` dans `plugins/maicie/src/main.rs` (`close` ferme le focus courant et promeut la file, sans clore l'objectif) et bloc `focus` dans `status --json` ; tests d'analyse d'arguments et de projection
- [ ] T026 [US2] Interface : route `POST /v1/control/focus` dans `ui.rs` (ServiceRequest `Delegate { goal, focus }` comme principal humain, `409 focus_conflict` avec le focus courant si `on_conflict` absent), champ « Travaille sur… » avec projet présélectionné dans la colonne Projets de `app.js`, confirmation à deux choix Remplacer / Mettre en file, focus distingué et en tête de la liste des objectifs ; tests Node, dont un cas où la pause est active : la route accepte le focus (FR-004)
- [ ] T027 [US2] Focus en attente d'agent : dans `reconcile.rs`, si le focus n'a aucune délégation remise après `durations.normal_secs`, déposer un item `focus_waiting_agent` (dedup `focus-waiting:<objective_id>`) ; test d'effet

**Checkpoint**: US2 livrable : la voie humaine attestée de l'ADR-014 est fermée.

---

## Phase 5: User Story 3 - Recevoir ce qui attend une décision (Priority: P1)

**Goal**: tout événement qui exige le référent arrive dans une boîte durable, visible, poussée vers son canal, tranchée par lui seul.

**Independent Test**: chaîne de réassignation épuisée → item en moins de deux minutes avec contexte et options ; décision → délégation mise à jour à la relève suivante.

- [ ] T028 [US3] Producteurs Maicie : quand `EtatGenerationDelegation::InterventionHumaineRequise` est posé (`plugins/maicie/src/domain.rs:1580`) avec le kind `chain_exhausted` pour le motif `chaine_preautorisee_epuisee` et `intervention_required` pour tout autre motif, quand un verdict de revue est à arbitrer (`review_verdict_pending`), quand une approbation d'activation de profil est en attente dans `activation_approvals` (`activation_approval`, dedup `activation:<approval_id>`), et à la saturation du budget, écrire une ligne `notification_outbox` avec destinataire réservé `human` et charge `{dedup_key, kind, subject_ref, context, options}` ; `reconcile_notification_one` (`plugins/maicie/src/reconcile.rs:456`) route ce destinataire vers `deposit_human_inbox` ; vérifier que `migrate_agent_participants` (`store.rs:541`) laisse `human` intact ; tests : un item par événement, options fermées (`reassign:<agent_id>`, `cancel`, `ack`)
- [x] T029 [US3] Producteurs daemon : palier 3 des relances (`collect_reminder_actions`) dépose `reply_debt` (dedup `reply-debt:<request_id>`) ; `focus_waiting_agent` reçu de Maicie ; tests d'effet
- [x] T030 [US3] Canal externe : chargement de `~/.config/bridget/human-channel.json` (0600, chemin absolu obligatoire, même garde que `install-bridget-ronde.sh:43`) dans `human_inbox.rs`, exécution de la commande avec le résumé JSON borné (≤ 400 caractères de `summary`) sur stdin à chaque dépôt créé, échec consigné dans `notify_attempts_json`, jamais bloquant, fichier absent = aucun appel ; tests avec une commande factice qui écrit son stdin dans un fichier temporaire, et avec une commande qui échoue
- [x] T031 [US3] Rappel : dans le thread horaire de `daemon.rs:4537`, repousser vers le canal les items ouverts depuis plus de `reminder_after_secs` (défaut 3600) ; test avec horloge injectée
- [ ] T032 [US3] Relève des décisions par Maicie : à chaque relève, `fetch_human_decisions` puis, pour chaque décision absente de `human_decisions_applied`, application dans `plugins/maicie/src/reconcile.rs` **dans une transaction** qui écrit aussi `human_decisions_applied` : `reassign:<agent_id>` crée la génération vers cet agent, `cancel` annule la délégation, `ack` ferme sans effet, `raise_budget` est ignoré côté Maicie (le plafond vit dans le daemon) ; puis `ack_human_decision(decision_id)` ; une décision déjà appliquée renvoie seulement l'acquittement ; tests d'effet par décision, plus un test de plantage simulé entre relève et application qui prouve que la décision est relue et appliquée une seule fois
- [ ] T033 [US3] Fermeture automatique : quand l'objet d'un item ouvert a disparu (délégation annulée ou objectif clos), le producteur appelle `close_self` avec `object_vanished` ; les dépôts répétés sur une clé ouverte incrémentent `occurrences` ; tests
- [x] T034 [US3] CLI `bridget inbox list [--all]` et `bridget inbox resolve <id> <choice>` (terminal requis) dans `cli.rs` ; tests de rendu et de refus
- [x] T035 [US3] Interface : routes `GET /v1/inbox`, `POST /v1/inbox/<id>/resolve` dans `ui.rs` via socket ; panneau boîte dans `app.js` (liste, contexte déplié, boutons issus de `options`, historique repliable), compteur dans le bandeau, signalement par la notification native ADR-016 existante ; tests Node

**Checkpoint**: US3 livrable : le système sait joindre le référent.

---

## Phase 6: User Story 4 - Borner le travail auto-généré (Priority: P2)

**Goal**: un plafond d'objectifs automatiques ouverts, appliqué à l'ouverture, visible, avec demande au référent quand il est atteint.

**Independent Test**: plafond 1, deux routines → un objectif, une occurrence différée `budget`, un item `budget_reached` unique ; clôture → la suivante repart.

- [x] T036 [US4] Plafond `auto_objectives_cap` : déjà porté par `control_state` (T006) ; ajouter `bridget control budget <n>` dans `cli.rs` et l'exposer comme descripteur de réglage serveur dans `crates/bridget-daemon/src/control_settings.rs:137` (`server_setting_descriptors`) avec prévisualisation puis application ; tests
- [x] T037 [US4] Application dans `plugins/maicie/src/app.rs` `delegate` : si le permit est `AutoGenerated`, compter les objectifs ouverts d'origine `AutoGenerated` (O(n), n borné, commentaire de complexité) et refuser par `DelegateError::BudgetReached` au plafond ; routines → `Differee { reason: "budget" }` ; dépôt `budget_reached` (dedup `budget-reached`) ; les permits `HumanRequest` ne comptent pas ; tests d'effet, mutant sur le comptage
- [x] T038 [US4] Interface : réglage du plafond dans Paramètres du serveur (`app.js`, même flux qu'emplacements : un bouton Enregistrer qui enchaîne prévisualisation puis application), affichage « n objectifs automatiques sur N » ; tests Node

**Checkpoint**: US4 livrable.

---

## Phase 7: User Story 5 - Voir l'état de contrôle d'un coup d'œil (Priority: P2)

**Goal**: les quatre faits toujours visibles, dans l'interface et en ligne de commande.

**Independent Test**: pause + focus + un item → les quatre faits affichés sur l'écran principal et rendus par `bridget who` ; ils disparaissent quand chaque levier est levé.

- [x] T039 [US5] `GET /v1/control/state` agrège l'état de contrôle, `open_count` de la boîte et le bloc `control` de `maicie status --json` (même appel que `crates/bridget-daemon/src/reprise.rs:436`) ; bandeau complet dans `app.js` : pause avec durée, focus (but tronqué, projet), compteur d'items, « n objectifs automatiques sur N » ; tests Node de la projection avec chaque levier absent ou présent
- [x] T040 [US5] Historique relisible : `bridget control status --history` lit `control_events` (auteur, horodatage, motif) ; test de rendu
- [ ] T041 [US5] Focus en tête et distingué dans la liste des objectifs de l'interface, pause visible dans le pied de `bridget who` et dans `maicie status` texte ; tests de rendu

---

## Phase 8: Polish & Cross-Cutting Concerns

- [x] T042 Essai adverse automatisé dans `crates/bridget-daemon/tests/control_adversarial_test.rs` : un client de type MCP (capacités `[SendIdempotent]`) tente `ControlStateSet`, `HumanInboxResolve` et un `Delegate` avec `origin` humain ; les trois sont refusés avec leur motif fermé, rien n'est écrit ; SC-005
- [ ] T043 Gates sur le worktree serveur, comptes de tests annoncés dans `implementation.md` : `cargo fmt --check` ; `cargo clippy -p bridget-transport -p bridget-daemon -p maicie --all-targets --features test-support -- -D warnings` limité aux fichiers modifiés (les trois erreurs préexistantes de `bridget-transport` sont listées, pas masquées) ; `cargo test -p bridget-transport` ; `cargo test -p bridget-daemon --lib --features test-support referent_control:: human_inbox:: ui:: project_ matrice_roles` ; `cargo test -p maicie --lib control:: routines:: guichet::` et les tests d'intégration touchés ; `node --test crates/bridget-daemon/assets/ui/app.js` ; comparaison des noms de tests rouges avec la base nue avant tout verdict
- [x] T044 Documentation : `docs/regles-chantier.md` section « Transmission des trois identifiants » remplacée par la voie focus, mention de la pause et de la boîte ; `specs/087-reprendre-controle/quickstart.md` relu contre les libellés réels ; `implementation.md` tenu à jour par tâche
- [ ] T045 Validation opérateur sur le serveur : dérouler `quickstart.md` §1 à §5, consigner chaque identifiant observé (`control_events`, `human_inbox`, `routine_occurrences`) dans `implementation.md` ; ne pas cocher sans la validation explicite du référent
- [ ] T046 Mise en service : compilation release depuis le checkout principal après fusion, binaire versionné dans `~/.local/lib/bridget/`, bascule du lien, redémarrage du relais UI puis du daemon, relance des agents gérés par UUID (`bridget agents --json` avant, `bridget relaunch <uuid>` après), vérification `bridget who` (build-id, ligne Contrôle) et absence d'avertissement `identity_version` ; consigner dans `implementation.md`

---

## Dependencies & Execution Order

- Phase 1 → Phase 2 → US1 → US2 → US3 → US4 → US5 → Polish.
- T047 et T048 ont été ajoutés après la contre-revue adverse (voir `adversarial-review-jim.md`) ; ils s'exécutent à leur place de phase, T047 avec US1 après T017, T048 avec US2 après T020.
- US2 dépend de T020 (daemon) et T021 (migration) ; US3 dépend de T007 et T009 ; US4 dépend de T006 et T010 ; US5 agrège les trois premières.
- T005 est le gate de sortie de la phase 2 : rien ne se coche avant lui.

## Parallel Execution Examples

- Phase 1 : T002 et T003 en parallèle avec T001.
- Phase 2 : T009 en parallèle de T006 à T008 (fichiers distincts) ; T010 après T009.
- US1 : T012, T013, T014 (daemon) en parallèle de T015, T016, T017 (Maicie) ; T018 et T019 ensuite.
- US3 : T028 (Maicie) en parallèle de T029 à T031 (daemon).

## Implementation Strategy

- MVP = Phase 1 + Phase 2 + US1 : le frein d'urgence, livrable et démontrable seul.
- Puis US2, qui rend au référent la position de client, puis US3 qui rend le système joignable.
- US4 et US5 sont courtes et s'appuient sur les tables déjà posées.
- Toute tâche cochée a son test d'effet joué et, pour les gardes de pause, son mutant consigné.

## Article XX

- Réduit la charge future : T005 (composition compilée), T007 (`kind` dérivé de l'enum, pas recopié), T014 (réutilise une garde existante au lieu d'en écrire une), T023 (supprime la transmission manuelle de trois identifiants), T043 (comptes de tests comparés à la base nue).
- Ajoute de la complexité à justifier : T030 (exécution d'une commande externe configurée ; justifiée par l'absence de canal sortant et bornée par un fichier 0600 à chemin absolu), T021 (une migration de plus ; justifiée par deux tables nécessaires à l'usage unique de l'attestation).

## Convergence

Passage 1, 2026-09-02 après intégration des deux versants. Lecture de `spec.md` FR par FR contre le code ; manques ci-dessous, ajoutés sans rien décocher.

- [ ] T049 [US2] FR-012, FR-015 : un focus sans agent disponible est refusé au lieu d'attendre ; `plugins/maicie/src/greffe_service.rs` `apply_delegate` doit laisser l'objectif ouvert en file et déposer `focus_waiting_agent` après `durations.normal_secs` (reprend T027) ; test d'effet
- [ ] T050 [US2] FR-012 : mesure de la base gelée sur `origin/<branche par défaut>` par `plugins/maicie/src/review_git.rs` quand le focus n'a pas de `review_target` (reprend T023 pour cette moitié) ; test
- [ ] T051 [US3] FR-020 : producteurs `review_verdict_pending` et `activation_approval` dans Maicie (reprend T028)
- [ ] T052 [US3] FR-024 : décision `reassign:<agent_id>` appliquée par Maicie (aujourd'hui `Unsupported`, non acquittée) ; exige soit un fait de réassignation construit par Maicie, soit une trame daemon (reprend T032)
- [ ] T053 [US3] FR-021 : trame `HumanInboxClose { item_id, reason }` (rôle service) dans `crates/bridget-transport/src/protocol.rs` et son bras daemon vers `human_inbox::close_self`, pour que Maicie ferme un item dont l'objet a disparu (reprend T033)
- [ ] T054 [US4] FR-031 : variante `GuichetRefusalReason::BudgetReached { cap, open }` au protocole ; aujourd'hui mappée sur `mutation_invalid`
- [ ] T055 [US5] FR-040 : focus courant dans le bandeau et en tête de liste, sans appeler `maicie status` en boucle ; concevoir une lecture Maicie sans effet de bord (reprend T041)
- [ ] T056 [US1] FR-002 : brancher un producteur réel de continuations gouvernées sur `reserve_governed_continuation` ; la garde existe, aucun appelant de production (dette antérieure, à décider avec le référent)
