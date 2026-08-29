# Tâches : Plan de contrôle Bridget et Maicie

**Entrées** : artefacts de conception dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/`
**Prérequis** : `plan.md`, `spec.md`, `reuse-audit.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`
**Principe** : les tests et preuves de frontière précèdent le comportement productif qu'ils valident. Chaque lot doit rester activable et réversible indépendamment.

**Gate de reprise avant T001** : la branche a été réalignée sur
`e72a79b28f51d56548ce01a30c6a103005ce169d` et l'audit de réutilisation a été
rejoué avec verdict PASS. Vérifier que `origin/main` n'a pas avancé de nouveau,
puis rejouer Analyze après tout delta de périmètre.

## Phase 1 - Préparation et preuves de référence

**But** : figer les scénarios, versions et conditions d'activation avant toute évolution runtime.

- [x] T001 Écrire les scénarios Gherkin des six stories et des cas limites SPEC-064 dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/tests/features/064-plan-controle-bridget-maicie.feature`, avec un identifiant de scénario relié à chaque SC-001 à SC-011
- [x] T002 Relever les chemins, versions, empreintes et opérations observables des binaires Codex, Claude et Cursor configurés, en distinguant `provider_kind=cursor` de `execution_path=acp`, dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/provider-baseline.md` sans enregistrer de secret ni de contenu utilisateur [FR-027] [SC-008]
- [x] T003 [P] Figer les schémas et transcriptions publiques minimales des versions Codex, Claude et ACP supportées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/tests/fixtures/provider-contracts/README.md` et des fixtures adjacentes [FR-040] [FR-041]
- [x] T004 [P] Documenter les flags, gates, rollback et suppressions de compatibilité par lot dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/rollout.md` [FR-043] [FR-044]
- [x] T005 Ajouter la matrice exigences vers scénarios et commandes de preuve dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/quickstart.md`, avec une ligne pour FR-001 à FR-044 et SC-001 à SC-011 [SC-010]

---

## Phase 2 - Fondations bloquantes

**But** : finaliser SPEC-063, stabiliser le vocabulaire partagé et introduire les machines durables sans changer encore l'expérience utilisateur.

**Gate** : cette phase bloque toutes les user stories.

- [x] T006 Ajouter dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs` les tests SPEC-063 où `item.id` et `userMessage.clientId` sont volontairement différents, y compris mauvais identifiant, événement tardif et absence de preuve [FR-005] [FR-009] [FR-041]
- [x] T007 Corriger dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs` l'acquittement de steering pour exiger `item/started.userMessage.clientId`, avec repli borné et acquittement unique [FR-005] [FR-006] [SC-002]
- [x] T008 Exécuter la preuve SPEC-063 sur le vrai binaire configuré et consigner version, événements corrélés et issue sans contenu utilisateur dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/063-interruption-pilotage-tour-humain/evidence/client-id-consumption.md` [FR-027] [SC-008]
- [x] T009 [P] Ajouter les tests de sérialisation rétrocompatible pour origine, intention et références dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-core/src/message.rs` [FR-002] [FR-007] [FR-040] [FR-044]
- [x] T010 [P] Ajouter les tests de négociation, refus de capacité et round-trip des nouveaux contrats dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` [FR-010] [FR-028] [FR-040]
- [x] T011 [P] Ajouter les tests de transitions pures, terminalité monotone et révision attendue dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-core/src/execution.rs` [FR-003] [FR-004] [FR-009]
- [x] T012 Étendre `BridgetMessage` avec origine, intention et références optionnelles rétrocompatibles dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-core/src/message.rs` [FR-002] [FR-007] [FR-042]
- [x] T013 Définir `WorkSubmission`, `Execution`, états et raisons structurées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-core/src/execution.rs`, puis les exporter depuis `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-core/src/lib.rs` [FR-001] [FR-003] [FR-004] [FR-010]
- [x] T014 Étendre les enums `ClientCapability`, commandes et projections versionnées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` sans créer de protocole parallèle [FR-013] [FR-040] [FR-044]
- [x] T015 Étendre `ManagedSession` et `ManagedEventKind` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/managed_session.rs` avec identité provider, thread, tour, item, état d'attente et capacités observées [FR-004] [FR-007] [FR-016] [FR-027] [FR-029]
- [x] T016 Étendre `AdapterCapabilities` et le preflight de lancement dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/lifecycle.rs` avec source, version et refus typé [FR-018] [FR-027] [FR-028]
- [x] T017 Ajouter les tests SQLite de migration additive, reprise après crash et anciennes lignes héritées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_store_test.rs` [FR-006] [FR-042] [FR-044] [NFR-001] [SC-003]
- [x] T018 Introduire les tables, index et migrations idempotentes des soumissions, exécutions, bindings et corrélations dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/execution_store.rs` [FR-001] [FR-007] [FR-008] [FR-016] [FR-042]
- [x] T019 Relier `SendDelivery` aux soumissions et exécutions sans modifier son rejeu exact dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/idempotency.rs` [FR-001] [FR-003] [FR-006] [FR-042]
- [x] T020 Exposer les écritures et lectures conditionnelles du magasin d'exécution depuis `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/lib.rs`, avec comparaison d'état, révision et génération [FR-009] [FR-031]
- [x] T021 Ajouter les bascules de double écriture et projections héritées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/registry.rs`, avec valeurs par défaut inactives et refus des combinaisons incompatibles [FR-043] [FR-044]
- [x] T022 Exécuter les suites ciblées `cargo test -p bridget-core`, `cargo test -p bridget-transport` et les tests du magasin d'exécution, puis consigner uniquement commandes et verdicts dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/foundations.md` [SC-001] [SC-002] [SC-003]

**Checkpoint** : la consommation SPEC-063 est prouvée et les contrats communs sont testables, mais aucune nouvelle fonction n'est encore activée en production.

---

## Phase 3 - User Story 1 : Piloter et comprendre une exécution réelle (P1)

**Objectif** : séparer présence, livraison, visibilité, tour, attente et issue, puis permettre les quatre intentions de pilotage avec repli borné.

**Test indépendant** : pendant un tour actif, soumettre QueueOnly, SteerCurrent et InterruptAndStart, puis vérifier ordre, corrélation, état, preuve et borne de chaque demande.

- [x] T023 [P] [US1] Ajouter les tests d'admission, FIFO par priorité, redémarrage et non-duplication dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/work_submission_test.rs` [FR-001] [FR-006] [FR-008] [SC-003]
- [x] T024 [P] [US1] Ajouter les tests d'états running, waiting_approval, waiting_user_input, interrupting et unreachable dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_lifecycle_test.rs` [FR-004] [FR-029] [NFR-002]
- [x] T025 [P] [US1] Ajouter les scénarios provider adversariaux silence, saturation, mauvais identifiant et événement tardif dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/managed_wrapper_test.rs` [FR-005] [FR-006] [FR-009]
- [x] T026 [P] [US1] Ajouter les tests UI qui distinguent connexion, vitalité provider, état du tour, attente, âge du progrès et file dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/ui_relay_test.rs` [FR-033] [NFR-007] [SC-004]
- [x] T027 [US1] Implémenter l'admission et la sélection atomique du prochain travail dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/execution_store.rs` sans scan complet répété [FR-001] [FR-008]
- [x] T028 [US1] Relier admission, remise et création d'exécution dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/wrapper.rs`, avec rollback ou issue persistée à chaque frontière [FR-003] [FR-006]
- [x] T029 [US1] Mapper les événements `ManagedSession` vers les transitions durables et ignorer les sorties de mauvais tour ou génération dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/managed_supervisor.rs` [FR-004] [FR-007] [FR-009] [FR-032]
- [x] T030 [US1] Implémenter QueueOnly, TriggerTurn, SteerCurrent, InterruptAndStart et ControlOnly dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/wrapper.rs`, sans transformation silencieuse [FR-002] [FR-013] [FR-031]
- [x] T031 [US1] Détecter et borner les boucles d'autorisation dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/managed_supervisor.rs`, avec état corrélé et raison machine [FR-029] [FR-030]
- [x] T032 [US1] Publier commandes, lookup et événements d'exécution via le contrat existant dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` et leur traitement dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/daemon.rs` [FR-003] [FR-004] [FR-010]
- [x] T033 [US1] Remplacer la lecture booléenne `busy` par une projection détaillée avec compatibilité temporaire dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/assets/ui/app.js` [FR-033] [SC-004]
- [x] T034 [US1] Brancher les compteurs et jauges de livraison/exécution sur `Metrics` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/daemon.rs`, avec labels bornés et aucun corps de message [FR-034] [FR-035]
- [x] T035 [US1] Exécuter le parcours indépendant US1 et consigner chronologie, preuves et durée de diagnostic dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us1-control.md` [SC-001] [SC-002] [SC-004]

---

## Phase 4 - User Story 2 : Coordonner une flotte avec propriété durable (P1)

**Objectif** : rattacher chaque enfant à un parent, un mandat et un travail, appliquer les quotas avant création et préserver les résultats tardifs.

**Test indépendant** : créer deux enfants, attendre leurs résultats, faire disparaître le parent et vérifier propriété, réveil et politique d'orphelin après redémarrage.

- [x] T036 [P] [US2] Ajouter les tests de réservation, profondeur, cycle, quota et rollback de lien dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/agent_graph_test.rs` [FR-011] [FR-012] [FR-014]
- [x] T037 [US2] Ajouter les tests de parent disparu, résultat tardif et transfert explicite dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/agent_graph_test.rs` [FR-015] [SC-005]
- [x] T038 [US2] Étendre `SpawnOrder`, `SpawnLease` et les refus de création avec parent, rôle, mandat et exécution propriétaire dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/fleet.rs` [FR-011] [FR-014]
- [x] T039 [US2] Étendre le schéma atomique `DesiredEquipier` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/desired_state.rs` avec références optionnelles rétrocompatibles [FR-011] [FR-012] [FR-044]
- [x] T040 [US2] Persister la machine `AgentLink` et ses index parent/enfant dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/fleet.rs`, avec un seul propriétaire ouvert et terminalité monotone [FR-012] [FR-015]
- [x] T041 [US2] Appliquer profondeur, nombre d'enfants et capacité avant tout effet de création dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/lifecycle.rs` [FR-014] [FR-031]
- [x] T042 [US2] Ajouter l'attente événementielle et le réveil du parent sur événements pertinents dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/daemon.rs`, avec timeout et reprise par curseur [FR-013] [FR-015]
- [x] T043 [US2] Exposer ascendance, propriétaire, mandat et agrégats de descendants dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/assets/ui/app.js` [FR-011] [FR-037] [SC-005]
- [x] T044 [US2] Exécuter le parcours indépendant US2 avec redémarrage et consigner les propriétaires ou motifs d'absence dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us2-fleet.md` [SC-003] [SC-005]

---

## Phase 5 - User Story 3 : Reprendre ou bifurquer sans faux-semblant (P1)

**Objectif** : utiliser reprise ou bifurcation natives uniquement lorsqu'elles sont attestées, sinon annoncer une reconstruction textuelle.

**Test indépendant** : reprendre un fil Codex compatible, tenter une version incompatible, puis reconstruire chez un provider sans capacité native et vérifier l'ascendance.

- [x] T045 [P] [US3] Ajouter les tests de contrat `thread/resume` et `thread/fork` avec versions compatibles et incompatibles dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs` [FR-017] [FR-018] [FR-020]
- [x] T046 [P] [US3] Ajouter les tests de persistance session/thread/tour, changement de génération et ascendance dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_resume_test.rs` [FR-016] [FR-019]
- [x] T047 [US3] Ajouter les tests du fallback textuel qui interdit l'étiquette native dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_resume_test.rs` [FR-017] [FR-020] [SC-006]
- [x] T048 [US3] Implémenter les commandes neutres resume/fork et leurs issues structurées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/managed_session.rs` [FR-017] [FR-020]
- [x] T049 [US3] Implémenter `thread/resume` et `thread/fork` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs` à partir du contrat de la version réellement observée [FR-018] [FR-027] [FR-028]
- [x] T050 [US3] Persister et valider l'ascendance de reprise/bifurcation dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/execution_store.rs`, avec refus de génération ou binding incompatible [FR-016] [FR-019] [FR-020]
- [x] T051 [US3] Conserver `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/reprise.rs` comme fallback déclaré et publier `native`, `forked` ou `reconstructed` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` [FR-017] [SC-006]
- [x] T052 [US3] Exécuter le parcours indépendant US3 sur deux profils provider et consigner capacité, mode et références dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us3-resume.md` [SC-006] [SC-008]

---

## Phase 6 - User Story 4 : Préserver les vérités Bridget et Maicie (P1)

**Objectif** : relier mission et exécution par contrats publics, puis supprimer la dépendance du daemon envers l'état privé Maicie.

**Test indépendant** : faire évoluer, interrompre et rendre indisponible une exécution liée à une délégation, puis vérifier que la projection change sans transition métier implicite et que Bridget fonctionne sans Maicie.

- [x] T053 [P] [US4] Ajouter les tests de domaine Maicie pour `ExecutionReference` et `ExecutionProjection`, y compris Fresh, Gap, Ended et Unavailable, dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/domain.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/runtime.rs` [FR-023] [FR-024] [FR-025]
- [x] T054 [P] [US4] Ajouter un test d'architecture qui interdit toute dépendance productive `bridget-daemon -> maicie` dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/mission_boundary_test.rs` [FR-021] [FR-022] [FR-026]
- [x] T055 [P] [US4] Ajouter les tests qui prouvent qu'un événement completed, unreachable ou stale ne clôt ni ne rouvre un objectif dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/app.rs` [FR-021] [FR-024] [SC-007]
- [x] T056 [US4] Ajouter les références et projections d'exécution opaques au domaine et magasin Maicie dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/domain.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/store.rs` [FR-021] [FR-023]
- [x] T057 [US4] Étendre le flux Bridget consommé par Maicie dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/runtime.rs` en réutilisant curseur, gap et fraîcheur [FR-022] [FR-025]
- [x] T058 [US4] Faire produire une projection de mission publique, versionnée et atomique dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/ui_projection.rs`, sans boucle résidente cachée [FR-021] [FR-025]
- [x] T059 [US4] Faire lire la projection publique depuis `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` sans importer les types privés Maicie [FR-025] [FR-026]
- [x] T060 [US4] Retirer la dépendance `maicie` de `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/Cargo.toml` après migration de tous ses consommateurs, puis vérifier que `cargo tree -p bridget-daemon` ne contient plus Maicie [FR-026]
- [x] T061 [US4] Exécuter le parcours indépendant US4 avec Maicie indisponible et consigner l'absence de transition métier implicite dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us4-boundary.md` [SC-007]

---

## Phase 7 - User Story 5 : Exploiter plusieurs fournisseurs par contrat vérifiable (P2)

**Objectif** : rendre chaque stratégie dépendante d'une capacité observée et conserver les preuves brutes des divergences.

**Test indépendant** : démarrer Codex app-server, Claude stream-json, Cursor via ACP, un provider sans steering et une version inconnue, puis vérifier capacités, refus, replis, attentes d'autorisation et événements inconnus.

- [x] T062 [P] [US5] Ajouter un test de contrat commun qui exécute les mêmes oracles sur Codex app-server, Claude stream-json, Cursor via `AcpTransport` et le faux provider dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/tests/provider_contract_test.rs` [FR-027] [FR-040] [FR-041]
- [x] T063 [US5] Ajouter les cas version inconnue, capacité annoncée puis refusée, EOF et événement inconnu dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/tests/provider_contract_test.rs` [FR-028] [FR-032]
- [x] T064 [P] [US5] Ajouter les tests d'autorité de contrôle, identité, génération, thread et tour incohérents dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/capabilities_integration_test.rs` [FR-029] [FR-030] [FR-031] [NFR-005]
- [x] T065 [US5] Centraliser l'observation de chemin, version, empreinte, contrat et capacités dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/managed_session.rs` et les définitions existantes de `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` [FR-027]
- [x] T066 [US5] Mapper les capacités et événements Codex depuis `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/codex_app_server.rs`, en conservant JSON brut et provenance pour tout événement inconnu [FR-028] [FR-032]
- [x] T067 [US5] Mapper les capacités, autorisations et événements Claude dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/claude_provider_session.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/claude_stream_json.rs`, puis Cursor et les autres fournisseurs ACP dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/acp.rs`, sans branche spéciale Cursor hors ACP [FR-028] [FR-029] [FR-032]
- [x] T068 [US5] Appliquer la matrice capacité vers opération, refus ou fallback dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/managed_supervisor.rs` sans heuristique par nom de provider [FR-018] [FR-028]
- [x] T069 [US5] Afficher version observée, capacités, fallback et attente d'autorisation dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/assets/ui/app.js` [FR-027] [FR-029]
- [ ] T070 [US5] Exécuter le parcours indépendant US5 sur Codex app-server, Claude stream-json et Cursor via ACP, puis consigner la matrice réellement prouvée dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us5-providers.md` [SC-008]

---

## Phase 8 - User Story 6 : Gouverner l'autonomie par des faits mesurés (P2)

**Objectif** : agréger temps, usage et descendants, puis autoriser une continuation uniquement sur preuve d'inactivité et politique explicite.

**Test indépendant** : atteindre successivement les limites de temps, d'usage et d'enfants et vérifier pause, blocage ou arrêt sans nouvelle livraison concurrente ni clôture métier.

- [x] T071 [P] [US6] Ajouter les tests d'agrégation temps, usage et descendants dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_budget_test.rs` [FR-037] [FR-038]
- [x] T072 [US6] Ajouter les tests de course entre continuation, nouvelle soumission, interruption et limite dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_budget_test.rs` [FR-038] [FR-039]
- [x] T073 [P] [US6] Ajouter les tests Maicie qui empêchent toute clôture métier issue de la seule limite runtime dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/app.rs` [FR-021] [FR-024] [SC-007]
- [x] T074 [US6] Étendre les politiques de quota existantes dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/fleet.rs` avec temps, usage, profondeur et descendants, sans moteur de règles générique [FR-037] [FR-038] [NFR-004]
- [x] T075 [US6] Agréger usage et durée depuis les événements provider dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/execution_store.rs`, avec cardinalité bornée et attribution à l'exécution [FR-035] [FR-037]
- [x] T076 [US6] Publier pause, blocage, limite d'usage, limite de budget et terminaison comme issues distinctes dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-transport/src/protocol.rs` [FR-038]
- [x] T077 [US6] Autoriser la continuation dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/managed_supervisor.rs` seulement après preuve d'inactivité et réservation atomique sans concurrent [FR-039]
- [x] T078 [US6] Ajouter les limites métier aux délégations sans leur donner d'effet technique direct dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/domain.rs` et `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/plugins/maicie/src/store.rs` [FR-021] [FR-023] [FR-038]
- [x] T079 [US6] Exécuter le parcours indépendant US6 et consigner limites, consommations et absence de clôture implicite dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/us6-autonomy.md` [SC-007]

---

## Phase 9 - Observabilité, compatibilité et clôture technique

**But** : vérifier le programme comme un ensemble observable, réversible et maintenable avant toute activation générale.

- [x] T080 [P] Ajouter les tests de métriques à cardinalité bornée et de redaction du contenu dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_observability_test.rs` [FR-034] [FR-035] [NFR-006]
- [x] T081 Ajouter les tests d'alertes pour message vieillissant, tour sans progrès, boucle d'autorisation et file saturée dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_observability_test.rs` [FR-036] [SC-009]
- [x] T082 Instrumenter les transitions, latences, files, replis et événements tardifs dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/daemon.rs` en réutilisant `Metrics` et les journaux structurés existants [FR-034] [FR-035]
- [x] T083 Définir et publier les seuils d'alerte consommables depuis `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/src/ui.rs` sans imposer de backend externe non validé [FR-036] [SC-009]
- [x] T084 Vérifier les coûts algorithmiques des files, lookups, descendants et projections avec un benchmark reproductible dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/crates/bridget-daemon/tests/execution_scale_test.rs` et consigner les seuils observés [NFR-003] [NFR-007]
- [x] T085 Retirer les bascules et projections héritées dont les consommateurs sont nuls, ou documenter leur date de retrait, dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/rollout.md` [FR-043] [FR-044] [SC-011]
- [x] T086 Exécuter `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`, puis consigner commandes, versions et verdicts dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/final-validation.md` [SC-001] [SC-010]
- [x] T087 Exécuter les tests Rust référencés pour tous les scénarios de `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/tests/features/064-plan-controle-bridget-maicie.feature` et compléter leurs commandes et verdicts dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/quickstart.md` [SC-001] [SC-010]
- [x] T088 Faire une revue de sécurité des identités, commandes de contrôle, permissions, secrets et télémétrie, puis consigner les findings vérifiés dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/evidence/security-review.md` [FR-031] [FR-035]
- [x] T089 Mettre à jour le statut, les compteurs de tâches/tests et les dépendances réellement livrées dans `/home/moi/bridget-referent/.worktrees/session-064-plan-controle-bridget-maicie/specs/064-plan-controle-bridget-maicie/spec.md` uniquement après preuves et activation admise [SC-010] [SC-011]

## Dépendances entre stories

```text
Phase 1 préparation
        |
Phase 2 fondations et SPEC-063
        |
US1 contrôle d'exécution
   |               |
   +-----> US2 flotte
   |
   +-----> US4 frontière Maicie
   |
   +-----> US5 providers -----> US3 reprise
              |                     |
              +----------+----------+
                         |
          US2 + US4 + US5 -> US6 autonomie
                         |
              observabilité et clôture
```

- US1 est le MVP obligatoire après les fondations.
- US2, US4 et US5 peuvent avancer en parallèle après US1 si les fichiers partagés sont sérialisés par ownership.
- US3 dépend de la matrice de capacités US5 pour distinguer reprise, bifurcation et reconstruction.
- US4 dépend seulement de la projection d'exécution stable issue de US1, pas des opérations natives de reprise.
- US6 dépend de l'activité fiable US1, du graphe US2, de la frontière US4 et de l'usage provider US5.
- La phase 9 dépend de toutes les stories activées.

## Exemples de parallélisation sûre

- US1 : T023, T024, T025 et T026 touchent des fichiers de test distincts avant T027 à T034.
- US2 : T036 puis T037 construisent le même fichier de scénarios avant la saga T038-T042 ; l'UI T043 commence après stabilisation du contrat.
- US3 : T045 et T046 peuvent être écrits en parallèle, T047 complète ensuite le fichier de T046, puis T048-T051 restent séquentiels sur les contrats partagés.
- US4 : T053, T054 et T055 couvrent domaine, architecture et décision métier dans des fichiers distincts.
- US5 : T062 et T064 couvrent transport et daemon en parallèle, puis T063 complète le contrat transport avant les mappings provider.
- US6 : T071 et T073 couvrent runtime et invariant métier en parallèle, puis T072 complète les courses runtime avant la politique productive.

## Stratégie d'implémentation

1. Livrer d'abord la preuve SPEC-063 et les fondations, sans activer le nouveau modèle.
2. Livrer US1 comme MVP opérable avec double écriture et projection de compatibilité.
3. Ajouter flotte, reprise et providers comme incréments réversibles, chacun avec son propre gate.
4. Découpler Maicie seulement après stabilité du contrat public.
5. Activer budgets et continuation en dernier, lorsque l'inactivité et les coûts sont prouvés.
6. Ne créer aucun nouveau crate ou backend d'observabilité sans refaire l'audit de réutilisation et documenter trois consommateurs réels.

## Article XIX et XX

- T019, T038-T040, T056-T060, T065 et T082 imposent la réutilisation des magasins, sagas, contrats, projections et métriques existants.
- T051 conserve le fallback utile au lieu de le supprimer prématurément.
- T060 retire le couplage privé daemon vers Maicie et réduit la charge future.
- T085 et T089 donnent une condition de suppression aux couches de compatibilité.
- Les nouveaux modules `execution.rs` et `execution_store.rs` sont justifiés par deux machines d'état durables utilisées par core, daemon, transport public, UI, Maicie et tests.
- Toute extraction supplémentaire, nouveau crate ou dépendance externe reste interdite sans audit et arbitrage explicites.
