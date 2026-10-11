# Tâches - Session156 guidage, interruption et lecture de file

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R13), `data-model.md`,
`contracts/guidance156.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : session150 livrée (garde de pause, refus nommés, `native_delegation.rs`).

Règle : une tâche n'est cochée qu'après son test vert. Les tâches « Rouge
d'abord » sont écrites et vues en échec avant le code. Exécution séquentielle
des phases (fichiers communs : `daemon.rs`, `mcp.rs`).

## Phase 1 - Préparation

- [ ] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `spec_087`, `native148`, `native149`, `native150`, `t3code`, `mcp::`, `send_idempotent`, `control` de `bridget-daemon` et la suite de `bridget-transport` avec la recette ; consigner PASS/FAIL/ignorés dans `specs/156-guidage-file/validation/baseline.md`
- [ ] T002 [P] Lever les quatre vérifications ouvertes (lecture seule, aucun code), réponses avec `fichier:ligne` dans la section « Vérifications levées » de `research.md` : (a) ce que reçoit l'émetteur quand un transport rejette un `steer` de façon asynchrone (`DeliveryRejected` vers saga `indeterminate` et avis, `t3code.rs` `report_discard`, `wrapper.rs`) ; (b) inventaire des producteurs légitimes de `origin=Human` (connexion attach, relais UI, client nu humain, `daemon.rs:8604-8606`) avant de neutraliser l'origine ; (c) fournisseurs T3 à guidage natif prouvé (confirmer Claude et Cursor, `t3code.rs:3126`, et T3 `CommandPolicy.ts:238-262`) ; (d) noms des champs de la tâche native (propriétaire, `child`) dans `crates/bridget-daemon/src/delegation.rs` ; (e) si un `RoleHandshake{Attach}` est accepté sur une connexion qui a déjà une identité d'agent (`daemon.rs:14190-14213`), et ce que donne `env -i bridget send` (client nu `human`, `cli.rs:3479`, `:4322`) ; (f) points de construction de `DeliverIdempotent` (envoi et reprise) et acteurs qui reçoivent `CancelDelivery` (`daemon.rs:18155-18175`)

## Phase 2 - Fondations (bloquantes)

- [ ] T003 [P] Rouge d'abord puis code dans `crates/bridget-transport/src/protocol.rs` : `ClientCapability::QueueListV1`, `WrapperToDaemon::QueueList`, `DaemonToWrapper::QueueListResult`, `ExecutionControlRefusal::NotPermitted` ; tests : les trames existantes se sérialisent à l'octet près comme avant, les nouvelles font un aller-retour ; classer les trames dans la matrice exhaustive de `crates/bridget-daemon/src/daemon.rs` (~`:14541-14610`, sans `_`)
- [ ] T004 [P] Rouge d'abord puis code dans `crates/bridget-daemon/src/referent_control.rs` : `AutonomousEffect::AgentGuidance` et son entrée dans `admit_autonomous_effect` ; test `guidance156_pause_garde_table`
- [ ] T005 [P] Rouge d'abord : dans le nouveau `crates/bridget-daemon/src/guidance.rs`, tests `guidance156_matrix_*` (humain, parent par lien, parent natif, pair même projet, autre projet, soi-même, non attesté, preuve enfant, `InternalPause`) ; puis `authorize(&GuidanceFacts) -> Result<(), GuidanceRefusal>` pure ; déclarer le module dans `crates/bridget-daemon/src/lib.rs` ; annoter la complexité (O(1))
- [ ] T006 [P] Dans `crates/bridget-daemon/src/idempotency.rs` et `idempotency/send_delivery.rs` (tests dans `idempotency/tests.rs`) : table `guidance_events` et ses deux index, `record_guidance_event`, `count_interrupts_target(target, since)`, `count_interrupts_pair(actor, target, since)` (comptent `outcome='admitted'` des acteurs agents), purge des événements de plus de 24 h appelée par `run_maintenance` ; index `idx_send_deliveries_recipient_phase` ; `queued_deliveries_for_recipient(instance_id, now, limit)` (requête de `data-model.md`, tri `rowid`) ; tests : comptages exacts aux bornes 30 s et 10 min (horloge injectée), la table n'est jamais lue pour un rejeu, lecture ordonnée après une réaffectation, base 150 migrée sans perte

## Phase 3 - US1 Choisir l'intention à l'envoi (P1)

**But** : `bridget_send` et la CLI posent une intention demandée, sans changer l'existant.
**Test indépendant** : filtres `guidance156_mcp`, `guidance156_cli`, `guidance156_send`, `guidance156_t3`.

- [ ] T007 [US1] Rouge d'abord dans `crates/bridget-daemon/src/mcp.rs` (tests) : `guidance156_mcp_schema_intent`, `guidance156_mcp_intent_inconnue_refusee` (`invalid_intent`, aucune connexion ouverte), `guidance156_mcp_queue_egal_absent` (mêmes octets `SendIdempotent`), `guidance156_mcp_steer_pose_steer_current`, `guidance156_mcp_interrupt_pose_interrupt_and_start`
- [ ] T008 [US1] Dans `crates/bridget-daemon/src/mcp.rs` : `intent` dans la liste blanche de `reject_unknown_arguments` (`:685-694`), analyse, pose de l'intention sur le message AVANT l'envoi, schéma de `bridget_send` (`:2028-2046`), résultat `intent` + `effect: "unconfirmed"` (`send_issue_result`), clé `guidance` de `bridget_capabilities` ; mettre à jour les tests qui figent schémas et liste d'outils (~`:3934-3975`) ; T007 passe
- [ ] T009 [US1] Dans `crates/bridget-daemon/src/cli.rs` (`cmd_send`, ~`:3499-3560`) : option `--intent <valeur>` ; tests `guidance156_cli_intent_valide`, `guidance156_cli_intent_inconnue`, `guidance156_cli_sans_intent_inchange`
- [ ] T010 [US1] Rouge d'abord dans `crates/bridget-daemon/src/daemon.rs` (tests, faux wrapper) : `guidance156_send_canon_intention_demandee` (le canon contient `SteerCurrent`/`InterruptAndStart`, jamais une valeur réécrite), `guidance156_send_rejeu_actif_puis_inactif_identique` (même résultat, aucune lecture de l'état de la cible), `guidance156_send_autre_intent_envelope_mismatch`, `guidance156_send_queue_octets_inchanges`, `guidance156_send_steer_transmis_tel_quel` (le wrapper reçoit `SteerCurrent`), `guidance156_send_interrupt_admet_execution` (`admit_starting_message_for_project`)
- [ ] T011 [US1] Dans `crates/bridget-daemon/src/daemon.rs` (`handle_idempotent_send_with_result_authority`, avant `canonical_send`, ~`:8584-8654`) : aucune réécriture d'intention ; le résultat renvoie l'intention demandée ; T010 passe (la garde et la neutralisation d'origine arrivent en T015)
- [ ] T012 [US1] Rouge d'abord puis code dans `crates/bridget-daemon/src/t3code_contract_v2.rs`, `t3code_contract.rs`, `t3code.rs` : `dispatch_command`/`dispatch` prennent `Option<&str>` (`"steer"`, `"restart"`) ; `steer` envoyé seulement si le fournisseur est dans la liste confirmée par T002(c), sinon remise rejetée `steer_unavailable` et rapportée comme `report_discard` ; jamais de `steer` que T3 puisse transformer en interruption ; un message de guidage est choisi dans toute la file (pas seulement `queue.front()`, `:1962-1975`), part même si le fil est occupé et n'entre jamais dans un lot (`batch_family`, `:3150`) ; tests `guidance156_t3_steer_pose_delivery_intent`, `guidance156_t3_interrupt_pose_restart`, `guidance156_t3_steer_fournisseur_hors_liste_refuse`, `guidance156_t3_guidage_derriere_tete_bloquee`, `guidance156_t3_guidage_hors_lot`, `guidance156_t3_groupable_claude_inchange` (session 129) ; adapter les tests existants (`t3code.rs:5306`, `t3code_contract_v2.rs:~565`)
- [ ] T013 [US1] Écrire `crates/bridget-daemon/tests/guidance156_e2e.rs` (rouge d'abord, motif de `native_delegation_e2e.rs`) : transport Codex simulé en tour actif, un `steer` d'un parent donne l'événement `turn_steer` sur ce tour ; transport Claude simulé, `steer` rejeté sans interrompre le tour, puis `interrupt` démarre un nouveau tour ; transport Codex inactif, `steer` rejeté par le transport sans effet de bord

## Phase 4 - US2 Droits et garde-fous sur tous les chemins (P1)

**But** : seuls l'humain attaché et le parent guident, sur tous les chemins atteignables par les outils ; pas de boucle ; pause et mission respectées. Les trames brutes sont testées en défense en profondeur (spec, « Cadre de menace »).
**Test indépendant** : filtres `guidance156_guard`, `guidance156_human`, `guidance156_control`, `guidance156_rate`, `guidance156_pause`, `guidance156_mission`, `guidance156_midturn`, `guidance156_cancel`, `guidance156_combo`.
**Dépend de** : T002, T005, T006, T011.

- [ ] T014 [US2] Rouge d'abord dans `crates/bridget-daemon/src/daemon.rs` (tests) : `guidance156_guard_*` sur `SendIdempotent` et `Send` historique avec `intent` brut : pair même projet, autre projet, soi-même, non attesté, preuve enfant (ADR 044), `from` forgé refusés ; parent par lien ouvert et par tâche native admis ; `guidance156_origin_human_forgee_par_agent_retiree` (défense en profondeur : une trame avec `origin=Human` d'un agent attesté n'interrompt pas), `guidance156_origin_human_attach_conservee` ; un refus ne livre rien
- [ ] T015 [US2] Dans `daemon.rs` : fonction de neutralisation de l'origine appelée sur `Send` (`:17521+`) et sur `SendIdempotent` avant le canon (selon l'inventaire de T002(b)) ; construire les `GuidanceFacts` (lien par `agent_link_for_child`, tâche native, projet, attestation par `live_connection_identity`, rôle attach) ; appeler `authorize` avant `prepare_dispatch` sur `SendIdempotent` (refus par `reject_idempotent_send`) et sur `Send` (`Nack`) ; écrire `guidance_events` ; T014 passe
- [ ] T016 [US2] Rouge d'abord `guidance156_human_client_nu_refuse_pour_controle` (client nu `from=human` avec `intent` : `identity_unattested`, envoi en file toujours admis), `guidance156_human_cli_sans_preuve_refuse` (CLI sous `env -i`, preuve d'identité absente, `--intent steer` refusé), `guidance156_human_reconnexion_agent_sans_autorite` (un agent attesté qui se reconnecte, ouvre un client nu ou envoie `RoleHandshake{Attach}` sur sa connexion ne gagne aucune autorité humaine ; résultat de T002(e)), `guidance156_human_attach_admis` ; puis implémenter : humain attesté = rôle attach sans identité d'agent ; refuser le contrôle aux clients nus ; documenter en commentaire la limite hors modèle (pseudo-terminal, trame brute)
- [ ] T017 [US2] Rouge d'abord `guidance156_control_interrupt_client_non_autorise`, `guidance156_control_steer_pair_refuse` (SC-008), `guidance156_control_pause_interne_admise` (`conn_id None`, `daemon.rs:9667`), `guidance156_control_parent_mission_steer_origine_humaine_forgee` (SC-010 : parent autorisé, enfant en mission active, `SteerCurrent` avec `origin=Human` ; le message transmis au wrapper n'a plus d'origine, le tour n'est pas interrompu), puis : normaliser l'origine du message de commande avant le canon et avant la garde, et appeler la même garde dans `handle_execution_control` (`:8990-9250`) avant `reserve_control_command` ; l'interruption de pause reste admise et n'écrit pas dans `guidance_events` ; `spec_087` reste vert
- [ ] T018 [US2] Rouge d'abord `guidance156_rate_cooldown_30s`, `guidance156_rate_plafond_3_par_10min`, `guidance156_rate_survit_au_redemarrage`, `guidance156_rate_retry_after_exact`, `guidance156_rate_humain_exempt`, `guidance156_rate_refus_non_compte`, puis brancher les comptages de T006 dans la garde
- [ ] T019 [US2] Rouge d'abord `guidance156_pause_steer_agent_refuse`, `guidance156_pause_interrupt_agent_refuse`, `guidance156_pause_queue_admise`, `guidance156_pause_humain_admis`, `guidance156_pause_aucun_tour_cree`, puis appeler `admit_autonomous_effect(AgentGuidance, …)` pour les appelants non humains
- [ ] T020 [US2] Rouge d'abord `guidance156_mission_interrupt_enfant_refuse` (`mission_active`) et `guidance156_mission_steer_enfant_admis`, puis lire la tâche native active de la cible dans les faits (champs confirmés par T002(d)) ; commentaire renvoyant à la session152
- [ ] T021 [US2] Guidage implicite du pont T3 (FR-022). Rouge d'abord `guidance156_midturn_pair_attend_fin_de_tour`, `guidance156_midturn_enfant_vers_parent_immediat` (session 129), `guidance156_midturn_parent_immediat`, `guidance156_midturn_humain_attache_immediat`, `guidance156_midturn_trame_sans_valeur_attend`, `guidance156_midturn_reprise_recalcule` ; puis : le daemon calcule `mid_turn_ok` (action `MidTurnDelivery` de la garde : humain attaché, parent de la cible, enfant de la cible) à chaque construction de `DeliverIdempotent`, envoi et reprise (points trouvés en T002(f)) ; `crates/bridget-daemon/src/t3code.rs` n'appelle `deliverable_while_busy` qu'avec cette valeur (ou `origin=Human` fabriquée sur `Deliver`) ; `guidance156_t3_groupable_claude_inchange` de T012 est mis à jour en conséquence
- [ ] T022 [US2] `bridget_cancel` : retrait en attente contre interruption active (FR-023). Rouge d'abord `guidance156_cancel_pair_demande_active_acp_sans_interruption` (SC-012 : demande annulée, tour intact), `guidance156_cancel_pair_demande_en_file_retiree`, `guidance156_cancel_parent_interrompt`, `guidance156_cancel_parent_pause_sans_interruption`, `guidance156_cancel_parent_plafond_sans_interruption`, `guidance156_cancel_parent_mission_sans_interruption`, `guidance156_cancel_interruption_comptee`, `guidance156_acp_cancel_queued_atomique` (le retrait de file et le refus d'atteindre le tour actif se font sous le même verrou) ; puis : champ `interrupt_active` de `CancelDelivery` (T003), `ManagedSession::cancel_queued` (défaut faux) dans `crates/bridget-transport/src/managed_session.rs`, implémentation dans `crates/bridget-transport/src/acp.rs` (`:518-531`), traitement dans `crates/bridget-daemon/src/wrapper.rs`, décision dans le traitement de `CancelRequest` (`daemon.rs:18125-18200`) par la garde commune ; le pont T3 garde son retrait de file ; documenter qu'un ancien wrapper garde l'ancien comportement jusqu'à sa relance
- [ ] T023 [US2] Test combiné `guidance156_combo_*` : (a) trois interruptions d'utilisateur admises sur une cible, puis une pause : la pause interrompt quand même ; (b) pendant la pause, `steer`/`interrupt` refusés sur `SendIdempotent`, `Send`, `ControlExecution` et `bridget_cancel` actif, l'interruption de pause interne passe ; (c) les compteurs ignorent l'interruption de pause ; (d) une interruption comptée par `bridget_send` pèse sur `bridget_cancel`, `ControlExecution` et inversement

## Phase 5 - US3 Voir la file d'un agent (P2)

**But** : `bridget_queue` lit la file, ne modifie rien.
**Test indépendant** : filtre `guidance156_queue`.
**Dépend de** : T003, T006, Phase 4.

- [ ] T024 [US3] Rouge d'abord dans `crates/bridget-daemon/src/daemon.rs` (tests) : `guidance156_queue_list_ordre_rowid`, `guidance156_queue_list_apres_reaffectation_meme_ordre`, `guidance156_queue_list_filtre_par_droits` (destinataire, parent, humain attaché voient tout ; autre : ses messages), `guidance156_queue_list_borne_200`, `guidance156_queue_list_ne_modifie_rien` (base identique avant/après), `guidance156_queue_negociation_requise`, `guidance156_queue_preuve_enfant_refusee`, `guidance156_queue_bound_t3_64_sinon_null`
- [ ] T025 [US3] Dans `daemon.rs` : traitement de `QueueList` (capacité `QueueListV1`, résolution de la cible, lecture de T006, filtre par droits, `preview` de 120 caractères, annotation de complexité O(log n + k), k ≤ 200) ; T024 passe
- [ ] T026 [US3] Outil `bridget_queue` dans `crates/bridget-daemon/src/mcp.rs` (schéma à un argument `target`, identité, négociation `QueueListV1`, résolution de cible comme `bridget_send`, preuve enfant refusée) et `bridget queue --target <cible>` dans `crates/bridget-daemon/src/cli.rs` ; tests `guidance156_mcp_queue_schema`, `guidance156_cli_queue_parse` ; mettre à jour le test de la liste d'outils

## Phase 6 - US4 Compatibilité et finitions

- [ ] T027 [P] Documenter : `docs/reference-communication.md` (remplacer la phrase « Cette règle n'annonce aucune option CLI de contrôle nouvelle », ~`:175-180`, par l'intention, la matrice de droits, les refus, l'origine fabriquée, le cadre de menace de la spec, le guidage implicite réservé à l'humain, au parent et à ses enfants, `bridget_queue`, et le rôle de `bridget_cancel`), `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`
- [ ] T028 [P] Écrire l'ADR `docs/decisions/156-guidage-et-lecture-de-file.md` (contexte, cadre de menace et limites hors modèle avec leurs preuves, failles fermées, intention demandée, pas de promesse d'effet, refus de `auto`, report de `cancel`/`reorder`/`promote` avec prérequis : réservation durable avant émission, accusé durable, réconciliation avant relivraison, phase d'annulation, saga de promotion ; statut selon les réponses aux 3 questions)
- [ ] T029 [US4] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test ne passe de vert à rouge, sauf les changements visibles acceptés (question 3a, tests de la session 129 réécrits en T021) ; consigner dans `specs/156-guidage-file/validation/tests-156.md` ; test d'égalité d'octets d'un `bridget_send` sans `intent` avant/après
- [ ] T030 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests sensibles à la charge avant de conclure ; consigner les compteurs dans `validation/tests-156.md`
- [ ] T031 Relire le diff complet (Articles XIX/XX : lignes suppressibles, abstractions, chemins d'erreur), écrire `specs/156-guidage-file/implementation.md` et enregistrer `git diff` dans `specs/156-guidage-file/validation/diff-156.patch`

## Dépendances

- T001 avant tout. T002 avant T012, T015, T016, T020, T021, T022.
- T003, T004, T005, T006 en parallèle (fichiers distincts), avant les phases 3 à 5.
- Phase 3 avant Phase 4 (la garde s'appuie sur l'envoi avec intention).
- Phase 4 avant Phase 5 (la lecture filtre par la même garde de parenté).
- T015, T016, T017, T022 touchent `daemon.rs` : exécution séquentielle. T021 touche aussi `t3code.rs` (après T012).
- T027, T028 en parallèle ; T029 à T031 en dernier.
- Session 150 (acquis) avant 156 ; 152 pour lever `mission_active` plus tard ; 155 et 157 modifient les mêmes fichiers : exécution séquentielle.

## Exemples de parallélisme

- T003, T004, T005, T006 : protocole, pause, garde pure, stockage.
- T027, T028 : documentation seule.

## Stratégie

MVP = Phases 1 à 4 : l'intention existe, la garde couvre tous les chemins
atteignables par les outils (envoi, contrôle, guidage implicite, annulation),
l'origine est fabriquée par le daemon, l'anti-boucle et la pause tiennent. Puis
la lecture de file (Phase 5), puis finitions.

## Charge future (Article XX)

- Réduisent la charge : T015/T017/T021/T022 (une garde, quatre chemins, quatre
  contournements fermés), T012 (booléen remplacé par une valeur, plus de repli
  interruptif caché), plus de réécriture d'intention selon l'état de la cible.
- Ajoutent de la complexité justifiée : T005 (module pur testable par table),
  T006 (table d'audit et de rythme), T015 (neutralisation d'origine), T021
  (champ `mid_turn_ok`, car le pont ne connaît pas la parenté), T022 (méthode
  `cancel_queued`, car `cancel_delivery` mélange retrait et interruption).
- Écartés pour rester minimal : `cancel`/`reorder`/`promote` de file, mode
  `auto`, `edit`, trames daemon vers wrapper, preuve humaine indépendante contre
  un processus hostile (hors modèle, ADR 011).
