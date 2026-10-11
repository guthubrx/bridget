# Tâches - Session150 socle de la délégation native

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R11), `data-model.md`,
`contracts/delegation150.md`, `reuse-audit.md`, `adversarial-review-codex.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).

Règle : une tâche n'est cochée qu'après son test vert. Les tests marqués
« rouge d'abord » sont écrits et vus en échec avant le code.

## Phase 1 - Préparation

- [x] T001 Établir la référence de non-régression avant tout changement : lancer les filtres `native148`, `native149`, `delegation149` et `spec_087` de `crates/bridget-daemon` avec la recette, et consigner les compteurs PASS/FAIL/ignorés dans `specs/150-socle-delegation/validation/baseline.md`

## Phase 2 - Fondations (bloquantes)

- [x] T002 [P] Ajouter `reply_timeout: Option<u64>` avec `#[serde(default, skip_serializing_if = "Option::is_none")]` à la variante `Delegate` de `crates/bridget-transport/src/protocol.rs` ; test unitaire : une requête sans le champ produit exactement les mêmes octets `serde_json::to_vec` qu'avant
- [x] T003 [P] Ajouter à la struct `Task` de `crates/bridget-daemon/src/delegation.rs` les champs `reply_timeout: Option<i64>` et `delivery_acked: bool` avec `#[serde(default)]` ; test : un payload 149 sans ces champs se relit avec `None`/`false`, et `visible_changed` ignore `delivery_acked` (aucun signal Lineage)
- [x] T004 [P] Ajouter la variante `AutonomousEffect::NativeDelegation` dans `crates/bridget-daemon/src/referent_control.rs` et l'entrée correspondante dans la table de vérité de `admit_autonomous_effect` (pause → `Deferred { motif: "pause" }`, sinon `Admitted`)
- [x] T005 [P] Ajouter dans `crates/bridget-daemon/src/idempotency/send_delivery.rs` une lecture explicite de la phase d'une remise par `(issuer_scope, idempotency_key)` qui rend `acked`, `dispatching`, `indeterminate`, `orphaned` ou absente ; tests unitaires des cinq cas

## Phase 3 - US2 Purge sans perte utile (P1)

**But** : borner l'historique sans jamais perdre un résultat non accusé ni
supprimer un arbre dont un sous-agent peut tourner.
**Test indépendant** : filtre `native150_purge`.

- [x] T006 [US2] Rouge d'abord : écrire dans `crates/bridget-daemon/src/delegation_lineage_tests.rs` les tests `native150_purge_*` de la table de vérité R2 : arbre terminé et accusé purgé ; `delivery_acked=false` conservé ; `failed` avec `failure_sent=false` conservé ; `cancelled` sans remise purgé ; une tâche active dans l'arbre le conserve ; deux arbres du même propriétaire dont un actif → seul l'arbre terminé est purgé ; arbre de moins de 7 jours conservé hors pression de borne
- [x] T007 [US2] Implémenter dans `crates/bridget-daemon/src/delegation_lineage.rs` la table `native_delegation_tombstones`, la sélection bornée des arbres purgeables (racine sans `parent_task_id`, âge = max `completed_at` sinon `updated_at`, 128 arbres au plus) et la suppression atomique d'un arbre : pierres tombales, reçus d'annulation, lignes, nouvelle génération ; signaler chaque propriétaire racine touché par `committed()` après le commit ; annoter la complexité des fonctions publiques ; T006 passe
- [x] T008 [US2] Dans `crates/bridget-daemon/src/delegation.rs` et l'admission de `crates/bridget-daemon/src/daemon/native_delegation.rs`, rendre `request_expired` quand la clé `(owner, request_id)` a une pierre tombale, sans créer de tâche ; purger les pierres de plus de 30 jours (fonction appelée par l'entretien en T011) ; tests `native150_purge_rejeu_cle_purgee` et `native150_purge_pierre_expiree_libere_la_cle`
- [x] T009 [US2] Dans `crates/bridget-daemon/src/daemon/native_delegation.rs`, écrire le passage de purge sous verrou : pour chaque arbre candidat, refuser si un enfant a une présence connectée, un `managed_spawns` dont `lease.name == child`, une saga de lancement `requested/reserved/starting` sous `fleet.supervisor_scope()` (`crates/bridget-daemon/src/idempotency/spawn_commands.rs:313`), ou une entrée de flotte désirée `Running` ; sinon décommissionner chaque entrée par `fleet.decommission` puis supprimer l'arbre ; si une décommission échoue, conserver l'arbre ; tests `native150_purge_enfant_present_conserve`, `native150_purge_lancement_en_cours_conserve`, `native150_purge_entree_stopped_decommissionnee`
- [x] T010 [US2] Dans `crates/bridget-daemon/src/daemon/native_delegation.rs`, recopier l'accusé : pour toutes les tâches `result_available`/`failed` avec envoi fait et `delivery_acked=false` (pas de lot : le total est borné à 4096), lire la phase (T005) de `native-result-<task>` ou `native-failure-<task>` sous `issuer_scope(task_id)` et poser `delivery_acked=true` si `acked` ; tests `native150_purge_outcome_unknown_puis_remise_effacee_conserve` (remise indéterminée effacée → arbre conservé), `native150_purge_accuse_recopie` et `native150_purge_129_accuses_expires_tous_recopies`
- [x] T011 [US2] Brancher la recopie (T010), la purge (T009) et la purge des pierres tombales expirées (T008) dans `run_maintenance` de `crates/bridget-daemon/src/daemon.rs`, la recopie AVANT `purge_expired_sends`, jamais dans `DaemonState::new` ; test `native150_purge_entretien_signale_resync` : un lecteur Lineage du propriétaire touché reçoit `Resync`, et l'ouverture d'un daemon ne purge rien

## Phase 4 - US1 Délégation disponible dans la durée (P1)

**But** : plus de blocage définitif après 4096 délégations.
**Test indépendant** : filtres `native150_admission`, `native150_ouverture`.
**Dépend de** : Phase 3 (purge de borne).

- [x] T012 [US1] Rouge d'abord : test `native150_admission_apres_5000_terminees` dans `crates/bridget-daemon/src/daemon/native_delegation.rs` (5000 tâches terminées, accusées, sans enfant vivant → nouvelle délégation refusée `task_limit` sur le code actuel)
- [x] T013 [US1] Remplacer le chargement complet `tasks()` de l'admission (`native_delegation.rs:441-451`) par des comptages SQL indexés dans `crates/bridget-daemon/src/delegation.rs` (actives totales ≥ 128, actives de l'instance ≥ 16 → `task_limit`) ; si le total atteint 4096, lancer un passage de purge borné (T009) puis refuser `retention_saturated` si le total reste à 4096 ; annoter la complexité ; tests T012 vert, `native150_admission_saturation_sans_purgeable`, `native150_admission_limites_actives_inchangees`, `native150_admission_purge_de_borne_plus_ancien_d_abord` (sous pression, un arbre de moins de 7 jours est purgé, le plus ancien en premier, et le total ne dépasse jamais 4096)
- [x] T014 [US1] Retirer le refus `records.len() > 4096` de `initialize_lineage` dans `crates/bridget-daemon/src/delegation_lineage.rs` en gardant cycle et profondeur ; adapter `delegation149_migration_fermee_cycle_profondeur_et_volume` (volume désormais accepté) ; test `native150_ouverture_10000_lignes`

## Phase 5 - US3 La pause refuse les nouvelles délégations (P1)

**Test indépendant** : filtre `native150_pause`.

- [x] T015 [US3] Rouge d'abord : test `native150_pause_refuse_nouvelle_cle` (délégation admise pendant une pause sur le code actuel), puis appeler `admit_autonomous_effect(AutonomousEffect::NativeDelegation, …)` dans l'admission de `crates/bridget-daemon/src/daemon/native_delegation.rs`, après le contrôle de rejeu et avant `parent_fact` ; refus `referent_paused` sans écriture ; tests `native150_pause_rejeu_cle_admise_rend_la_tache`, `native150_pause_cle_libre_apres_levee`, `native150_pause_statut_et_annulation_disponibles` ; `native148_replay_precedes_grant_revocation_and_definition_change` reste vert

## Phase 6 - US4 Délai de réponse par tâche (P2)

**Test indépendant** : filtre `native150_delai`.

- [x] T016 [US4] Dans l'admission de `crates/bridget-daemon/src/daemon/native_delegation.rs` : refuser `reply_timeout_out_of_range` hors 60 à 86 400 ; pour un délai explicite d'enfant, refuser `reply_timeout_exceeds_parent` s'il dépasse `parent.mission_deadline_at - now` ; sans délai explicite, aucun contrôle ; stocker la valeur dans la tâche ; respecter l'ordre des refus de `contracts/delegation150.md` ; tests `native150_delai_bornes`, `native150_delai_enfant_explicite_trop_long`, `native150_delai_enfant_implicite_inchange`, `native150_delai_rejeu_autre_valeur_envelope_mismatch`
- [x] T017 [US4] Utiliser `task.reply_timeout.unwrap_or(3600)` pour l'échéance de mission (`native_delegation.rs:1007`, `:1170`) et pour `message.reply_timeout` du suivi de demande (`:1143`) ; test `native150_delai_applique_echeance_et_suivi`
- [x] T018 [US4] Exposer `reply_timeout` (entier, 60 à 86 400) dans le schéma `bridget_delegate` de `crates/bridget-daemon/src/delegation_mcp.rs` et sa liste blanche ; ajouter `mission_reply_min_secs` et `mission_reply_max_secs` au catalogue (`native_delegation.rs:392`) ; mettre à jour le test qui fige les clés (`native_delegation_permissions149_tests.rs:195`)

## Phase 7 - US5 Compatibilité et finitions

- [x] T019 [US5] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148/149/087 ne passe de vert à rouge ; consigner dans `specs/150-socle-delegation/validation/tests-150.md`
- [x] T020 [P] Documenter le délai, les cinq refus, la rétention (dont la lecture de statut d'une tâche purgée, qui rend le refus existant de tâche inconnue) et la pause dans `docs/delegation-native.md` (nouvelle section « Refus »), `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`
- [x] T021 [P] Écrire l'ADR `docs/decisions/150-retention-et-delai-delegation-native.md` (contexte, décisions R1-R9, conséquences, statut Accepté)
- [x] T022 [P] Écrire l'ADR `docs/decisions/150-feuille-de-route-delegation-cinq-etoiles.md` : sessions 150-158, critères visés, transfert de la suspension par pause vers 152, incohérence du catalogue `discovery` notée pour 157
- [x] T023 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests connus sensibles à la charge avant de conclure ; consigner les compteurs dans `validation/tests-150.md`
- [x] T024 Relire le diff complet (Articles XIX/XX : lignes suppressibles, abstractions, chemins d'erreur), écrire `specs/150-socle-delegation/implementation.md` et enregistrer `git diff` dans `specs/150-socle-delegation/validation/diff-150.patch`

## Dépendances

- T001 avant tout.
- T002-T005 en parallèle, avant les phases 3 à 6.
- Phase 3 (US2) avant Phase 4 (US1) : l'admission utilise la purge de borne.
- Phases 5 (US3) et 6 (US4) indépendantes de 3-4, mais touchent le même fichier
  `native_delegation.rs` : exécution séquentielle.
- T019-T024 en dernier ; T020-T022 en parallèle.

## Exemples de parallélisme

- T002, T003, T004, T005 : fichiers distincts.
- T020, T021, T022 : documentation seule.

## Stratégie

MVP = Phases 1 à 4 : la délégation ne se bloque plus, sans perte de résultat.
Puis pause (US3), délai (US4), finitions.

## Charge future (Article XX)

- Réduisent la charge : T013 (admission O(log n), retrait d'un mur), T014
  (retrait d'une garde), T011 (purge automatique, plus d'historique sans fin).
- Ajoutent de la complexité justifiée : T009 (contrôle de vivacité, exigé par
  la contre-revue), T010 (preuve d'accusé durable, exigée par la contre-revue),
  T007 (pierre tombale, faute d'`issued_at` dans le contrat).

## Convergence

Passage 1 (2026-10-11) : 4 exigences partielles (FR-009, FR-012, FR-013, FR-021).

- [x] T025 Amender FR-009 dans `specs/150-socle-delegation/spec.md`, la ligne `retention_saturated` de `contracts/delegation150.md` et la section rétention de `docs/delegation-native.md` : à l'admission, la purge de borne retire seulement les arbres nécessaires pour repasser sous 4096, sans limite de 128 (exception assumée, la base ne dépasse pas 4096 par le chemin normal) ; préciser que la clé redevient libre au passage d'entretien suivant les 30 jours (≤ 1 h) et lister tous les cas qui bloquent la purge (agent routé, décommission en cours)
- [x] T026 [P] Test `native150_pause_precede_refus_de_droits` dans `crates/bridget-daemon/src/daemon/native_delegation.rs` (mod tests) : parent sans droit de délégation ET pause active → `referent_paused`, aucune écriture
- [x] T027 [P] Test `native150_pause_annulation_humaine_disponible` : pendant la pause, `human_cancel` (`crates/bridget-daemon/src/daemon/native_lineage.rs`) annule une tâche active et rend son reçu
- [x] T028 [P] Ajouter `control_state_unavailable` à `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md` (remplacer « cinq refus » par la liste exacte) et à FR-021 de `spec.md`
- [x] T029 [P] Test `native150_statut_tache_purgee_rend_task_unavailable` : `bridget_task_status` sur une tâche purgée rend le refus existant documenté
- [x] T030 [P] Test `native150_delai_enfant_explicite_parent_sans_echeance` : fige le comportement quand `parent.mission_deadline_at` vaut `None` (aucun contrôle, conforme à R8)

Contre-revue de l'implémentation (Codex, 2026-10-11, BLOCKED, 5 défauts vérifiables) :

- [x] T031 `copy_delivery_acks` rend un résultat complet ou une erreur ; dans `maintain_stores` (`crates/bridget-daemon/src/daemon.rs`), si une preuve n'a pas pu être enregistrée, ne PAS lancer `purge_expired_sends` dans ce passage ; test `native150_purge_echec_recopie_suspend_purge_des_envois`
- [x] T032 Décommissionner les enfants d'un arbre par lot (une lecture, une écriture de flotte) dans `purge_retained` (`native_delegation.rs`), en réutilisant `desired_state.rs` ; test `native150_purge_arbre_volumineux_une_ecriture_de_flotte` avec entrées réellement présentes
- [x] T033 Ne lancer `tick` qu'après une délégation admise (pas après un refus) dans le gestionnaire (`native_delegation.rs` ~319) ; test `native150_pause_refus_par_le_gestionnaire_complet_ne_fait_rien_avancer` passant par le gestionnaire complet avec une tâche préexistante en attente
- [x] T034 Appliquer aussi la purge de borne à l'entretien (lot de 128 arbres par passage) dans `maintain_retention` ; test `native150_entretien_purge_de_borne_sans_admission` (5000 arbres récents purgeables, plusieurs passages, total ≤ 4096 à la fin)
- [x] T035 Pour un `reply_timeout` EXPLICITE seulement, aligner l'échéance durable du suivi de demande sur l'admission de la mission sans changer l'enveloppe entre ticks (ex. `message.reply_timeout = reply_timeout + (started_at - created_at)`, stable dès le lancement) ; sans délai explicite, octets et comportement 149 inchangés ; test `native150_delai_suivi_aligne_sur_admission_apres_lancement_retarde` lisant l'échéance réellement enregistrée dans `tracked_requests`

Passage 2 (2026-10-11) : 1 manque (FR-009, introduit par T034) et écarts mineurs.

- [x] T036 Dans `maintain_retention` (`crates/bridget-daemon/src/daemon/native_delegation.rs`), passer `max_trees.saturating_sub(trees_deja_purges)` au second appel de `purge_retained` et le sauter si 0 ; test `native150_entretien_borne_et_age_cumulent_au_plus_128` (4300 arbres soldés dont 200 de plus de 7 jours : un passage de `maintain_stores` retire au plus 128 arbres)
- [x] T037 [P] Cohérence doc/code : `control_state_unavailable` placé au rang de la pause dans l'ordre des contrôles de `contracts/delegation150.md` et `docs/delegation-native.md` ; commentaire de `purge_retained` corrigé (128 par passage d'entretien, sans limite à l'admission) ; `specs/150-socle-delegation/implementation.md` mis à jour (47 tests native150, lignes T025-T036)
- [x] T038 [P] Tests des deux branches de vivacité non couvertes : `native150_purge_agent_route_conserve` (`router.get_agent`) et `native150_purge_decommission_en_cours_conserve` (`decommissioning_names`)
- [x] T039 Si `set_request_deadline` échoue (T035), réessayer au tick suivant tant que l'échéance du suivi diffère de `mission_deadline_at` pour un délai explicite ; test `native150_delai_suivi_realigne_apres_echec_d_ecriture`
