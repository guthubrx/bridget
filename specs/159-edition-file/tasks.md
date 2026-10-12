# Tâches - Session159 file retenue par le daemon et édition

**Entrées** : `spec.md`, `plan.md`, `research.md` (R0-R17), `data-model.md`,
`contracts/edition159.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine canonique après intégration** : `/Users/moi/Nextcloud/10.Scripts/64.bridget`.
**Worktree d’implémentation B** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/159B-edition-file` ; ce chemin devient historique après son nettoyage.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Prérequis** : livraison158B attestée par ROOT ; session156 déjà livrée (`guidance.rs`, origine fabriquée, `bridget_queue` en lecture).
**Base 159B confirmée** : A14 livrées, source 53863bd09a4ff364dc2b412519b343c77e163685, métadonnées b0a165edfc6fc5476b2330193c7822e0faecef33. La validation technique B14 est close après FULL/disposition, Converge et gates ROOT ; le cochage appartient exclusivement à ROOT. held_queue reste fermé. Les checkpoints e52a/b235/candidat2124 sont historiques.

Règle : une tâche n'est cochée qu'après son test vert. Les tâches « Rouge
d'abord » sont écrites et vues en échec avant le code. Exécution séquentielle
(fichiers communs : `daemon.rs`, `idempotency.rs`, `mcp.rs`, `wrapper.rs`).

**Découpage** : le lot A (T001 à T013 et T028, 159a) se livre et se mesure seul, réglage
`held_queue` fermé par défaut. Le lot B (T014 à T027, 159b) dépend du lot A.
**Garanties** : avant la libération, ni perte ni doublon. Après, garanties de toute
remise (R17). Aucun test ne doit affirmer plus.

# LOT A - 159a : file retenue par le daemon

## Phase 1 - Préparation

- [x] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `spec_087`, `native148`, `native149`, `native150`, `guidance156`, `t3code`, `mcp::`, `send_idempotent`, `idempotency` de `bridget-daemon` et la suite de `bridget-transport` avec la recette ; consigner PASS/FAIL/ignorés dans `specs/159-edition-file/validation/baseline.md` ; mesurer sur un porteur Codex simulé, réglage inexistant, l'intervalle fin de tour vers début du message suivant (référence de SC-004) et le consigner
- [x] T002 [P] Lever les dix vérifications ouvertes de `research.md` (lecture seule, réponses avec `fichier:ligne` dans « Vérifications levées ») : (1) modes de porteurs gérés qui envoient `TurnState`, événements, source de l'état réel de la session (`is_busy`), cadence du battement où attacher la réattestation, types d'agents pouvant déclarer `turn_attest` (`wrapper.rs:4757`, `:5403`, `:6103-6224`) ; (2) période du tick des relances (`daemon.rs:6826`) ; (3) causes d'écart du pont à reproduire (`t3code.rs:1855-1912`) et traitement de `DeliveryIndeterminate` pour `reply=true` (`daemon.rs:16395-16420`) ; (4) un envoi sans intention ne crée aucune exécution liée (`daemon.rs:8828-8850`) ; (5) `IdempotencyStore` peut lire et fermer `tracked_requests` dans sa transaction (repère candidat `send_delivery.rs:89–175`) ; (6) regroupement dans les porteurs gérés (`BatchedDelivery`) ; (7) forme finale de `guidance.rs` et de `bridget_queue list` livrés par 156 ; (8) colonnes de `tracked_requests` (`state`, `deadline_at`, repère candidat `store.rs:170–187`) et lien message vers demande ; (9) contenu de `control_pause_interruptions` et points de levée de la pause (`resume_executions_after_pause`) ; (10) résolution de l'identifiant d'agent à l'admission (`routing_agent_id`) contre `final_agent_id` à l'enregistrement

## Phase 2 - Fondations (bloquantes)

- [x] T003 Rouge d'abord puis code dans `crates/bridget-daemon/src/idempotency.rs`, ses tests existants `idempotency/tests.rs`, et `store_schema.rs` : migration v7. Tests `held159_migration_*` : `sans_perte` (copie de la base158B effectivement livrée avec remises dans chaque phase, comptages identiques), `ordre_preserve` (`queue_rank` = ancien `rowid`), `recipient_agent_rempli` (`json_extract(message_bytes,'$.to')`, lignes illisibles signalées), `liens_execution_conserves` (lignes de `send_delivery_execution_links` présentes après, `foreign_key_check` à zéro), `phases_held_et_cancelled_admises`, `turn_gate_defaut_zero`, `rejeu_sans_effet`, `rollback_si_echec`, `fantomes_ecartes_avec_warn`, `precontrole_accepte_7_refuse_8` ; puis : reconstruction de `send_deliveries` (phases `held` et `cancelled`, `queue_rank INTEGER PRIMARY KEY AUTOINCREMENT`, `delivery_id UNIQUE NOT NULL` conservant les cibles FK, `recipient_agent`, `turn_gate`), copie des liens avant `DROP TABLE` et réinsertion après le renommage, index (`idx_send_deliveries_operation`, `idx_send_deliveries_kind_key`, `idx_send_deliveries_recipient_phase` (non unique, étendu à `(recipient_instance_id, phase, queue_rank)`), `idx_send_deliveries_agent_phase`), version 7, `MAX_IDEMPOTENCY_SCHEMA_VERSION = 7` ; test documenté : un binaire antérieur (constante 6) refuse la base v7 ; ne pas désactiver `foreign_keys`
- [x] T004 Rouge d'abord puis code dans `crates/bridget-daemon/src/idempotency/send_delivery.rs` et `daemon.rs` : rang, rattachement par agent, phase `held` dans les opérations existantes. Tests `held159_rank_*` : `monotone_sous_concurrence`, `survit_a_la_reaffectation`, `reprise_dans_l_ordre`, `sans_collision_entre_instances`, `orphelinage_couvre_les_retenus` (notice d'auteur écrite), `reprise_ne_rend_pas_les_retenus` ; `held159_adopt_*` : `adoption_a_l_enregistrement`, `adoption_sans_presence_apres_redemarrage_du_daemon`, `double_redemarrage_nouvelle_instance` (SC-015), `autre_agent_non_touche` ; puis : INSERT omettant `queue_rank`, attribué par SQLite AUTOINCREMENT dans la transaction Immediate de begin_send_delivery_with_guidance avec le mode Held ; séquence durable après purge sans nouveau compteur ni table métier (pas de lien d'exécution) et `recipient_agent` écrit, `dispatching_deliveries_for_instance` triée par rang (remplace `ORDER BY delivery_id`), `reassign_dispatching_deliveries` et `orphan_dispatching_for_instance` étendues à `held`, `adopt_held_for_agent` appelé à l'enregistrement d'un agent (`daemon.rs:6311+`) ; adapter les tests existants qui supposaient l'ordre par `delivery_id`
- [x] T005 Rouge d'abord puis code : réglage et décision de rétention. Tests `held159_hold_*` en table de vérité (`research.md` R4) : réglage fermé, instance sans `turn_attest`, intention présente, origine `Human`, origine `System`, alerte de fil, observation, incident délégué, mission native, fil T3, présence non `busy` sans retenus, présence non `busy` avec retenus, présence `busy`, file pleine (`queue_full` mémorisé) ; puis : champ `held_queue` (`#[serde(default)]`) dans `AgentDefinition` (`crates/bridget-daemon/src/registry.rs`), fonction pure `should_hold(facts)`, appel dans `handle_idempotent_send_with_result_authority` après `prepare_dispatch` et avant la poussée (`daemon.rs:8686+`) : en cas de rétention, insertion `Held`, aucun push, ajout dans `held_ids`, résultat d'admission inchangé ; réglage fermé : aucune différence d'octets ni de phase (`held159_hold_ferme_identique`)

## Phase 3 - Signal de tour véridique (bloquant pour la libération)

- [x] T006 Rouge d'abord puis code dans `crates/bridget-daemon/src/wrapper.rs`, `crates/bridget-transport/src/protocol.rs` et `daemon.rs` : `TurnState` véridique et réattesté. Tests `held159_turnstate_*` : `refus_de_guidage_en_plein_tour_n_envoie_pas_false` (le test rouge échoue sur `wrapper.rs:6666`), `refus_sans_tour_en_cours_envoie_false`, `reattestation_periodique` (≤ 30 s, horloge injectée), `reattestation_d_un_repos_perdu`, `busy_period_start_non_remis_a_zero_par_true_repete`, `sortie_forcee_depuis_busy_period_start`, `wrapper_muet_trois_periodes`, `busy_since_inchange` (comportement existant), `register_sans_turn_attest_octets_inchanges` ; puis : le wrapper envoie l’état réel de la session après un refus ; la minuterie Instant du heartbeat existant est vérifiée à chaque boucle même sous trafic continu, avec réattestation vraie périodique ≤ 30 s, sans scheduler supplémentaire ; champ optionnel `turn_attest` de `Register` ; présence : `busy_period_start`, `last_attested_at`, `turn_attest` ; `set_turn_state` met à jour ces champs sans changer le comportement legacy de `busy_since` ; `release_stale_busy_turns` utilise `busy_period_start` pour toute instance `turn_attest`, même réglage `held_queue` fermé ; instances anciennes sans attestation : `busy_since` legacy conservé ; deux conditions conjointes pour une instance attestée : silence de plus90s depuis last_attested_at ET période busy d'au moins notify_timeout_secs+30s ; true frais maintient busy ; défaut2730s si silence depuis le début, configuration courte ne contourne pas90s ; les mêmes oracles existants couvrent31/120/121s, aucun nouveau scénario

## Phase 4 - US1/US2 Retenir, libérer, balayer (P1)

**Test indépendant** : filtres `held159_release`, `held159_sweep`, `held159_crash`.

- [x] T007 [US1] Rouge d'abord puis code : libération. Tests `held159_release_*` : `repos_atteste_libere_le_plus_ancien`, `un_seul_par_cycle_de_tour`, `pas_de_liberation_si_busy`, `refus_de_guidage_ne_libere_pas` (SC-012), `ack_puis_faux_signal_terminal_ne_libere_pas`, `declencheurs_concurrents_une_seule_liberation`, `reservation_levee_par_true`, `reservation_levee_par_remise_indeterminee`, `reservation_levee_apres_ack_et_30s_sans_busy`, `reservation_survit_au_redemarrage_du_daemon`, `reconnexion_libere`, `fin_d_execution_libere`, `balayage_libere_si_repos_reattesté`, `wrapper_muet_attend_la_sortie_forcee`, `echeance_de_tour_posee_a_la_liberation`, `demande_close_juste_avant_ne_libere_pas_et_retire` (SC-014), `echeance_juste_avant_ne_libere_pas`, `pause_ne_libere_pas`, `pause_interruption_fin_de_tour_aucune_liberation`, `levee_de_pause_reprise_d_abord_puis_liberation` (SC-013), `panne_entre_transaction_et_push_relivree_par_la_reprise` (point d'arrêt `test_sync::checkpoint`) ; puis : `release_next_held(agent, now)` (transaction `Immediate`, conditions de `research.md` R5.3, `turn_gate = 1`) ; garde State du repos jusqu’au commit, pause/control_pause_interruptions relues SQL dans cette même transaction sur config.db_path, ordre de verrous préservé, aucun writer socket, push hors verrou après commit, `AutonomousEffect::HeldRelease` dans `crates/bridget-daemon/src/referent_control.rs` (différé en pause), levée de la réservation, appels depuis le gestionnaire de `TurnState`, `schedule_idempotent_delivery_recovery`, `handle_execution_transition`, la levée de la pause et le tick des relances ; construction de la trame par `defer_idempotent_delivery` ; jamais sous un verrou d'écriture de socket
- [x] T008 [US2] Rouge d'abord puis code : balayage des retenus et suivi mémoire. Tests `held159_sweep_*` : `enregistrement_expire_retire`, `demande_close_retire`, `demande_expiree_retire`, `auteur_prevenu`, `retrait_resultat_envoi_rejected_avec_raison`, `parite_avec_les_causes_du_pont`, `retenu_non_expire_conserve`, `pending_replies_retire_apres_commit`, `aucun_rappel_palier_1_2_pour_une_demande_retenue`, `palier_3_ferme_la_demande_et_le_balayage_retire_le_message`, `held_ids_reconstruit_au_demarrage` ; puis : `settle_held(delivery_id, reason)` (`cancelled`, `Rejected(<raison>)`, notice d'auteur, suivi clos, une transaction), retrait de `st.pending_replies` après commit, `held_ids`, filtre des relances (`daemon.rs:6896+`)
- [x] T009 [US1] Rouge d'abord puis code : `bridget_queue list` lit la base. Tests `held159_list_*` : `held_dans_l_ordre`, `released_pour_dispatching`, `filtre_par_droits_de_156`, `borne_200`, `queue_bound_capacite_du_type_sinon_null`, `ne_modifie_rien`, `fil_t3_et_type_ferme_released_seulement` ; puis : extension de `QueueList` (156) dans `daemon.rs` et de la lecture list par `queue_rank`, comme la reprise `dispatching_deliveries_for_instance` déjà prévue en T004
- [x] T010 [US2] Tableau de pannes et d'équivalence du lot A `crates/bridget-daemon/tests/held159_e2e.rs` (rouge d'abord, `test_sync::checkpoint`) : arrêt du daemon après rétention, après la transaction de libération, avant et après le push, après l'accusé ; arrêt du porteur avec et sans message en cours ; **arrêt du porteur entre l'écriture du reçu `Seen` et l'injection : la remise devient `indeterminate` (attendu, déclaré), le message suivant reste retenu** ; daemon puis porteur redémarrés avec une nouvelle instance (SC-015) ; ordre sur trois messages et une réaffectation (SC-003) ; réglage fermé identique à l'ancien comportement (SC-001) ; mesure de la latence repos attesté vers début du message suivant, comparée à T001 et consignée dans `validation/latence-159a.md` (SC-004)
- [x] T011 Répétition de la restauration (porte de livraison du lot A, SC-006) : en isolation (répertoire privé, `BRIDGET_HOME` temporaire), copier la base réelle par `.backup`, l'ouvrir avec le binaire neuf (migration v7), contrôler les données (comptages par phase, `foreign_key_check`, échantillon de remises et de liens), restaurer la sauvegarde, l'ouvrir avec le vrai ancien binaire158B installé, sauvegardé et empreinté AVANT la reconstruction159 (commit livré à relever par ROOT ; `1f2c661d` reste une ancienne référence, pas le choix courant), comparer comptages et échantillon avec l'original ; consigner commandes et résultats dans `validation/restauration-v7.md` ; sans cette pièce, le lot A n'est pas livré
- [x] T012 [P] Documenter le lot A : `docs/reference-communication.md` (réglage `held_queue`, règle de rétention, `turn_attest`, messages jamais retenus, `queue_full`, états `held` / `released`, pause et `HeldRelease`, garanties réduites après libération, signal perdu en deux cas, cadre de menace), `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md` ; procédure de sauvegarde et de restauration issue de T011 ; ADR159 initiale A dans /Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/159-file-retenue-et-edition.md avant livraison (architecture de rétention, migration v7 et restauration), étendue par T025 au lot B
- [x] T013 Non-régression et clôture du lot A : relancer les filtres de T001 et comparer à `validation/baseline.md` (seuls changements admis : tests réécrits en T004) ; suite complète `cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette ; relire le diff (Art. XIX/XX), écrire `specs/159-edition-file/implementation-a.md` et enregistrer `git diff` dans `validation/diff-159a.patch`

# LOT B - 159b : édition de la file retenue (dépend du lot A)

## Phase 5 - Fondations de l'édition

- [x] T014 Rouge d'abord puis code dans `crates/bridget-daemon/src/idempotency.rs` et `idempotency/send_delivery.rs` : table `queue_edits` et API. Tests `edit159_receipt_store_*` : `reserver_puis_rejouer_meme_resultat`, `autres_parametres_envelope_mismatch`, `refus_memorise`, `effet_et_recu_atomiques` (échec simulé entre les deux : ni effet ni reçu), `purge_apres_horizon_seulement` ; puis : table et index de `data-model.md` créés par `CREATE TABLE IF NOT EXISTS` (même méthode que `orphan_emitter_notices`, `idempotency.rs:835-841`) : pas de nouvelle version de schéma
- [x] T015 [P] Rouge d'abord puis code dans `crates/bridget-daemon/src/guidance.rs` (156) : actions `QueueCancel`, `QueueReorder`, `QueuePromote`, faits `actor_is_author`, `message_editable`, `recipient_turn_active`, `carrier_queue_fallback`, refus `NotEditable` et `NoActiveTurn` ; table de vérité `edit159_rights_*` de `research.md` R12 (humain attaché, parent, auteur, destinataire, pair, client nu, preuve enfant, autre projet, pause) ; `QueuePromote` passe toujours par la garde de `Steer`
- [x] T016 [P] Rouge d'abord puis code dans `crates/bridget-transport/src/protocol.rs` : `ClientCapability::QueueEditV1`, `WrapperToDaemon::QueueEditRequest`, `DaemonToWrapper::QueueEditResult`, champ optionnel `queue_fallback` de `Register` (absent des octets quand faux) ; tests : les trames existantes (dont `Register` sans les champs) se sérialisent à l'octet près, les nouvelles font un aller-retour ; classer les trames dans la matrice exhaustive de `crates/bridget-daemon/src/daemon.rs` (sans `_`)

## Phase 6 - US4 Retirer (P1)

**Test indépendant** : filtre `edit159_cancel`.

- [x] T017 [US4] Rouge d'abord puis code : `apply_cancel` et traitement dans `daemon.rs`. Tests `edit159_cancel_*` : `retient_puis_retire_jamais_livre`, `phase_cancelled_et_envoi_rejete_cancelled_in_queue`, `prevenance_auteur`, `suivi_de_reponse_clos_en_base_et_en_memoire`, `aucun_rappel_apres_retrait`, `une_seule_transaction` (échec simulé après chaque écriture : tout ou rien), `not_held_pour_message_libere`, `not_held_pour_message_jamais_retenu`, `rejeu_meme_resultat`, `autres_parametres_envelope_mismatch`, `droits_auteur_parent_humain`, `destinataire_et_pair_refuses`, `pause_admis`, `aucune_interruption_de_tour`, `crash_apres_commit_message_non_livre` ; puis : transaction unique selon `research.md` R7, retrait du suivi mémoire après commit
- [x] T018 [US4] Tableau de pannes du retrait dans `crates/bridget-daemon/tests/edit159_e2e.rs` : arrêt du daemon avant, pendant (échec interne) et après la transaction de retrait ; reconnexion du porteur ; le message retiré n'est jamais livré (SC-007), le message voisin est libéré normalement

## Phase 7 - US5 Réordonner (P2, isolable)

**Test indépendant** : filtre `edit159_reorder`. Retirer cette phase ne change ni la Phase 6 ni la Phase 8.

- [x] T019 [US5] Rouge d'abord puis code : `apply_reorder` et traitement. Tests `edit159_reorder_*` : `avant_un_message`, `a_la_fin`, `message_libere_not_held`, `before_inconnu_ou_autre_destinataire_not_held`, `ordre_de_liberation_suit`, `arrivee_concurrente_deplacement_valide_d_abord_donne_B_A_C`, `arrivee_concurrente_arrivee_valide_d_abord_donne_B_C_A` (SC-008, deux écrivains, les deux ordres de validation), `ordre_persistant_egal_ordre_de_validation`, `survit_a_la_reaffectation`, `rangs_hors_groupe_inchanges`, `droits_parent_humain`, `destinataire_refuse`, `pause_admis`, `rejeu_meme_resultat` ; permuter les valeurs de rang existantes dans la transaction unique via rangs provisoires négatifs pour éviter collision de clé primaire, jamais publiés ; aucun CHECK positif incompatible, aucun nouveau compteur

## Phase 8 - US6 Promouvoir (P1)

**Test indépendant** : filtre `edit159_promote`.

- [x] T020 [US6] Rouge d'abord puis code dans `crates/bridget-transport/src/codex_app_server.rs`, `claude_stream_json.rs`, `acp.rs` et `crates/bridget-daemon/src/wrapper.rs` : marqueur `steer-or-queue`. Tests `edit159_promote_transport_*` : Codex avec tour actif et guidage ouvert (guide), Codex sans tour actif (enfile en file ordinaire, pas de rejet), `steering_open` faux (enfile), Claude (enfile), ACP (enfile), **file ordinaire pleine : rejet explicite `DeliveryRejected`, remise indéterminée, pas de perte silencieuse** (garantie réduite), **Codex : erreur ambiguë après guidage appliqué : le message est remis en file et un doublon est possible, test qui l'établit et le documente** (`codex_app_server.rs:1724-1740`), message sans marqueur (comportement actuel inchangé), refus du fournisseur après acceptation (remis en file) ; le porteur met `queue_fallback` à vrai dans son `Register`
- [x] T021 [US6] Rouge d'abord puis code : `apply_promote` et traitement dans `daemon.rs`. Tests `edit159_promote_*` : `libere_et_pousse_en_guidage` (trame avec `SteerCurrent` et `steer-or-queue`, enveloppe stockée et canon inchangés), `garde_156_obligatoire_pair_refuse`, `destinataire_non_occupe_no_active_turn`, `porteur_sans_queue_fallback_steer_unavailable_message_reste_retenu`, `pause_agent_refuse`, `pause_humain_admis`, `sans_droit_refuse`, `not_held_pour_message_libere`, `demande_close_juste_avant_ne_promeut_pas` (SC-014), `echeance_juste_avant_ne_promeut_pas`, `pose_la_reservation_de_tour`, `rejeu_meme_resultat`, `panne_apres_transaction_relivre_en_file_ordinaire`, `aucun_doublon_cote_daemon`

## Phase 9 - Outil et clôture

- [x] T022 [US4] Dans `crates/bridget-daemon/src/mcp.rs` : `bridget_queue` gagne `action` (`cancel`, `reorder`, `promote`), `message_id`, `before_message_id`, `request_id` (obligatoire pour les écritures), négociation `QueueEditV1`, preuve enfant refusée, résultat selon `contracts/edition159.md`, clés `queue` de `bridget_capabilities` ; dans `crates/bridget-daemon/src/cli.rs` : `bridget queue cancel|reorder|promote` ; tests `edit159_mcp_schema`, `edit159_mcp_request_id_obligatoire`, `edit159_mcp_preuve_enfant_refusee`, `edit159_cli_parse` ; mettre à jour les tests qui figent les schémas et la liste d'outils
- [x] T023 Tests de bout en bout du lot B dans `crates/bridget-daemon/tests/edit159_e2e.rs` : scénario complet (retenir, lister, réordonner, retirer, promouvoir, libérer) sur un porteur Codex simulé réglage ouvert ; aucun doublon côté daemon, aucune perte avant libération ; réglage fermé : toutes les actions rendent `not_held`
- [x] T024 [P] Documenter le lot B : `docs/reference-communication.md` (actions, droits, refus, reçu, `queue_fallback`, garanties réduites de la promotion), `skills/bridget/SKILL.md`, `skills/bridget/references/commandes.md`
- [x] T025 [P] Étendre pour le lot B l’ADR `docs/decisions/159-file-retenue-et-edition.md` déjà écrite par T012 pour le lot A (contexte, racine commune des 11 défauts, file tenue par le daemon, `execution_queue` écartée avec preuves, découpage, coûts mesurés, signal de tour véridique, réservation de tour, pause et `HeldRelease`, migration v7 à sens unique avec sauvegarde et restauration répétée, promotion et ses garanties réduites, report d'`edit`, option 159c pour T3, statut selon les réponses aux 3 questions)
- [x] T026 Non-régression du lot B : relancer les filtres de T001 et `held159`, comparer à `validation/baseline.md` ; suite complète `cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` ; rejouer seuls les tests sensibles à la charge avant de conclure ; test d'égalité d'octets d'un envoi sans édition avant et après ; consigner dans `validation/tests-159.md`
- [x] T027 Relire le diff complet du lot B (Art. XIX/XX : lignes suppressibles, abstractions, chemins d'erreur, atomicité de chaque transaction, aucune promesse plus forte que les tests), écrire `specs/159-edition-file/implementation-b.md` et enregistrer `git diff` dans `validation/diff-159b.patch`

## Dépendances

- T001 avant tout. T002 avant T004, T005, T006, T007, T008.
- T003 avant T004 (même base, même fichier). T004, T005 et T006 avant T007.
- T006 (signal véridique) avant toute ouverture du réglage : sans lui, la rétention est refusée (`turn_attest`).
- T011 (répétition de la restauration) avant la livraison du lot A : porte de T013.
- Lot A complet (T013) avant le lot B. Le lot A peut être livré sans le lot B.
- T014, T015, T016 en parallèle (fichiers distincts), après le lot A.
- Phase 6 avant Phases 7 et 8 (traitement, reçu et droits communs). Phase 7 isolable.
- T020 avant T021 (le marqueur existe avant que le daemon l'envoie).
- T004, T005, T007, T008 (daemon.rs), T017, T021, T022 (daemon.rs, mcp.rs) : exécution séquentielle.
- T024, T025 en parallèle ; T026, T027 en dernier.
- Livraison158B attestée avant159A. 157 et158 restent des dépendances déjà intégrées dans cette base ; exécution séquentielle.

## Exemples de parallélisme

- T002 avec T003 (lecture contre écriture).
- T014 à T016 : table de reçu, garde, protocole.
- T012 avec T011 (documentation contre répétition), T024 avec T025.

## Stratégie

MVP = Lot A : signal de tour véridique, file retenue, réglage fermé par défaut,
mesurée, avec tableau de pannes et restauration répétée. Ensuite Lot B : `cancel`
(P1), `promote` (P1, garanties réduites), `reorder` (P2). Les fils T3 restent hors
lot (159c).

## Charge future (Article XX)

- Réduisent la charge : une seule copie de l'ordre ; édition en une transaction ;
  plus de tombe, de saga d'édition, de réconciliation ni de trame vers le porteur ;
  ordre de reprise déterministe ; refus `queue_full` net à l'admission ; signal de
  tour véridique (profite aussi aux relances et à la présence).
- Ajoutent de la complexité justifiée : T003 (migration à sens unique, liens
  protégés, quatre colonnes), T006 (signal véridique, exigé par la libération),
  T007 (réservation de tour et conditions revérifiées), T008 (parité des causes
  d'écart et suivi mémoire), T020 (marqueur de promotion).
- Écartés pour rester minimal : `edit`, `read`, tombe durable chez le porteur,
  saga d'édition, snapshot du porteur, trames daemon vers porteur, `execution_queue`,
  extension aux fils T3, reprise durable du porteur après `Seen`.

## Convergence — passage 2, lot A

- [x] T028 [US3] Lot A — réparer le manque FR-009/T009 révélé par Converge P2 : dans `held159_list_filtre_par_droits_de_156` existant, couvrir la branche client nu réellement négociée (sans RegisterAuxiliary), ses messages propres `held` et `released`, et le filtrage des reçus étrangers ; mesurer RED puis remplacer le lookup de propriété par `stored_send_delivery` existant, sans changer issuer_scope/clé/comparaison delivery_id ; mesurer GREEN et régression du même groupe. Aucun nouveau nom de test, API ou helper.
