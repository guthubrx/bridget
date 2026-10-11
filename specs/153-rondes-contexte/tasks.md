# Tâches - Session153 rondes multiples et contexte transmis

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R13), `data-model.md`,
`contracts/rounds153.md`, `reuse-audit.md`.
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).

Règle : une tâche n'est cochée qu'après son test vert. Les tâches « Rouge
d'abord » sont écrites et vues en échec avant le code. Préalable : SPEC-150 est
livrée dans le worktree. SPEC-151 et SPEC-152 sont relues en T001.

## Phase 1 - Préparation

- [ ] T001 Établir la référence et les points de couplage : (a) lancer les filtres `native148`, `native149`, `native150`, `delegation149`, `spec_103` de `crates/bridget-daemon` avec la recette et consigner PASS/FAIL/ignorés dans `specs/153-rondes-contexte/validation/baseline.md` ; (b) relire le code livré par 150, 151 et 152 et lister dans `validation/couplage.md` : le prédicat de comptage des tâches actives et le prédicat de purge (150), les champs par ronde de `Task` et les lieux qui nomment `native-result-<task_id>` (150, 151), la boîte de fin et l'attente de tâche (151 : `work_state`, regroupement des messages `native-result-*` par préfixe), `suspended_at`, `latest_in_chain` et la réécriture de `prepare_restart` (152) ; décider pour chaque point « branché par T0xx » ou « sans effet » ; si le code attendu diffère de `plan.md`, corriger `plan.md` avant T002

## Phase 2 - Fondations (bloquantes)

- [ ] T002 [P] Dans `crates/bridget-transport/src/protocol.rs`, ajouter à `NativeDelegationRequest::Delegate` les champs `keep_alive_secs: Option<u64>` et `brief: Option<serde_json::Value>` avec `#[serde(default, skip_serializing_if = "Option::is_none")]`, et la variante `Continue { task_id: String, request_id: String, task: String, #[serde(default, skip_serializing_if = "Option::is_none")] brief: Option<serde_json::Value> }` ; tests unitaires : une requête 150 sans les champs produit les mêmes octets `serde_json::to_vec` qu'avant ; `Continue` fait un aller-retour ; deux sérialisations du même brief sont identiques
- [ ] T003 [P] Dans `crates/bridget-daemon/src/delegation.rs`, ajouter à `Task` les champs `round: u32`, `idle_deadline_at: Option<i64>`, `round_body: Option<String>`, `rounds: Vec<RoundRecord>`, `guarded: bool`, `guard_released: bool`, `unacked_rounds: Vec<u32>` (tous `#[serde(default)]`), le type `RoundRecord { round, request_id, digest }`, la table d'index `native_delegation_past_missions` (création dans `open`, lecture `past_mission_task(mission)`) et les méthodes `round_no()`, `result_message_id()`, `holds_slot()` (`!terminal() || (guarded && !cleanup_done)`), `continuable()` (inclut `!guard_released`) selon `data-model.md` ; tests : un payload 149 et un payload 150 se relisent avec les valeurs par défaut ; `result_message_id` vaut `native-result-<id>` en ronde 1 et `native-result-<id>-r2` en ronde 2 ; `visible_changed` (`delegation_lineage.rs:51`) ignore les sept champs ; l'ouverture d'une base 149 crée la table sans erreur
- [ ] T004 [P] Rouge d'abord puis vert, dans `crates/bridget-daemon/src/delegation_lineage.rs` et `delegation_lineage_tests.rs` : méthode `begin_round(task_id, expected_round, clock, ...)` (l'heure est relue par `begin_round` via `clock` juste avant l'écriture ; les tests injectent l'horloge) (transaction unique de `data-model.md`, CAS sur `round` et sur l'état, refus si `now >= idle_deadline_at`, insertion de l'ancienne mission dans `native_delegation_past_missions`, ajout de la ronde à `unacked_rounds` si `delivery_acked` est faux, `completed_at` remis à vide malgré `stamp`, incrément de projection, signal après commit) ; tests `native153_ronde_begin_round_*` : écriture complète des champs par ronde, mission passée écrite, ronde non accusée conservée, refus exactement à `now == idle_deadline_at`, refus si `guard_released`, échéance franchie entre la lecture d'admission et l'écriture (horloge injectée), refus si garde levée, refus si `expected_round` périmé, `completed_at` vide puis reposé à la fin de la ronde 2, un lecteur de projection reçoit une mutation visible

## Phase 3 - US1 Brief structuré (P1)

**But** : transmettre un contexte structuré et borné, avec le validateur existant.
**Test indépendant** : filtre `native153_brief`.

- [ ] T005 [US1] Rouge d'abord : dans `crates/bridget-daemon/src/daemon/native_delegation.rs` (module de tests), écrire `native153_brief_*` : brief valide → le corps de mission envoyé à l'enfant égale `<corps de bridget_handoff preview>\n\n--- Consigne ---\n<consigne>` ; brief invalide (champ inconnu, `null`, chaîne blanche, objectif absent) → `brief_invalid` sans tâche ni clé consommée ; brief rendu de 16 385 octets → `brief_invalid`, rendu de 16 384 → admis ; brief + consigne de 65 536 octets → corps inférieur à `INJECTED_BODY_MAX_CHARS` ; même brief rejoué → même tâche ; brief différent avec la même clé → `envelope_mismatch` ; appel sans brief → mêmes octets canoniques qu'en 150 (ces tests échouent sur le code actuel)
- [ ] T006 [US1] Dans l'admission de `bridget_delegate` (`native_delegation.rs`, branche `Delegate`), après le contrôle de borne du délai (150) et avant le contrôle de rejeu : si `brief` est présent, appeler `crate::handoff::validate_and_render` ; erreur → refus `brief_invalid` portant le champ et la raison ; stocker `round_body = <corps rendu>\n\n--- Consigne ---\n<task>` dans la tâche ; ajouter `brief_max_bytes` au catalogue (`native_delegation.rs:392`) ; mettre à jour le test qui fige les clés du catalogue (`native_delegation_permissions149_tests.rs`) ; T005 passe
- [ ] T007 [US1] Dans `tick` (`native_delegation.rs:1134`), envoyer `task.round_body` quand il est présent, sinon `request.task` ; le reste de l'envoi (idempotence par `task.mission`, délai de la tâche de 150) ne change pas ; test `native153_brief_corps_envoye_a_l_enfant` avec la remise simulée

## Phase 4 - US2 Garder l'enfant vivant et le terminer (P1)

**But** : l'enfant reste vivant jusqu'à fin explicite ou délai d'inactivité.
**Test indépendant** : filtre `native153_garde`.

- [ ] T008 [US2] Rouge d'abord : écrire `native153_garde_*` dans le module de tests de `native_delegation.rs` : avec `keep_alive_secs`, après la remise du résultat l'enfant est encore connecté, `cleanup_done` est faux et `idle_deadline_at` vaut remise + durée ; sans `keep_alive_secs`, nettoyage identique à 149 ; annulation agent et humaine reçue pendant `result_available` avec `result_sent` faux → après l'envoi par `tick`, aucune garde posée (`guard_released`), enfant arrêté, aucune ronde ; expiration → garde levée, enfant arrêté au tick suivant, `status` reste `result_available` ; `bridget_task_cancel` sur tâche inactive → garde levée, statut inchangé ; `human_cancel` sur tâche inactive → garde levée ; nettoyage de la tâche parente → enfants inactifs arrêtés et gardes levées sous verrou ; `prepare_restart` → gardes levées ; hors bornes (59, 7 201) → `keep_alive_out_of_range` ; même clé avec autre durée → `envelope_mismatch`
- [ ] T009 [US2] Dans l'admission : valider `keep_alive_secs` de 60 à 7 200 (`keep_alive_out_of_range`) à la même place que le brief ; ajouter `keep_alive_min_secs`, `keep_alive_max_secs`, `max_rounds` au catalogue ; mettre à jour le test des clés du catalogue
- [ ] T010 [US2] Dans `tick` : (a) dans la sauvegarde qui pose `result_sent = true` (`native_delegation.rs:1228-1232`), poser aussi `idle_deadline_at = now + keep_alive_secs` quand la tâche est `guarded` et pas `guard_released` (code local, `tick` est le seul écrivain de la remise depuis 151 révisée) ; (b) ajouter `&& task.idle_deadline_at.is_none()` aux deux conditions de nettoyage (`:1079` et `:1283`) ; (c) avant la branche `cancelling`, si `result_available` et `now >= idle_deadline_at`, lever la garde et sauvegarder (journaliser `keep_alive_expired`) ; annoter la complexité ; T008 (garde, expiration) passe
- [ ] T011 [US2] Fin explicite et arrêts : (a) branche `Cancel` de `native_delegation.rs:518` : sur une tâche `guarded` en `result_available`, que `result_sent` soit vrai ou non, poser `guard_released`, lever la garde et sauvegarder sans changer l'état ; (b) `human_cancel` (`delegation_lineage.rs:235`) : une tâche `guarded` en `result_available` « pilote » aussi (`drives`), poser `guard_released` et lever la garde dans la même transaction sans mutation visible ; (c) `cancel_tree`, phase sous verrou (`:1380-1400`) : lever la garde de chaque descendant et de la tâche nettoyée avant l'arrêt ; (d) `prepare_restart` (`:922`) : lever toute garde ; T008 passe en entier
- [ ] T012 [US2] Dans le comptage des places (admission de 150, T013 de 150) et dans le prédicat de purge (T007 et T009 de 150) : remplacer « non terminale » par `holds_slot()` (garde non confirmée arrêtée = place tenue) pour les 128 actives et les 16 par instance ; ajouter `idle_deadline_at.is_none()` au prédicat de purge ; tests `native153_limites_enfant_inactif_compte`, `native153_limites_17e_delegation_refusee_avec_16_inactifs`, `native153_limites_purge_refuse_arbre_avec_garde`, `native153_limites_expiration_arret_refuse_place_tenue` (arrêt refusé : la 17e admission reste refusée ; arrêt confirmé : elle passe), `native153_limites_annulation_arret_refuse_place_tenue`

## Phase 5 - US3 Nouvelle ronde sur le même enfant (P1)

**But** : ouvrir la ronde suivante sans nouveau lancement, avec rejeu sûr.
**Test indépendant** : filtre `native153_ronde`.
**Dépend de** : Phases 3 et 4.

- [ ] T013 [US3] Rouge d'abord : écrire `native153_ronde_*` : ronde 2 sur tâche inactive → même `child`, même `child_instance`, aucun `spawn`, nouvelle `mission`, `status = mission_pending`, résultat effacé ; rejeu ×10 de la même clé → une ronde, une mission ; même clé, autre consigne ou autre brief → `envelope_mismatch` ; rejeu de la clé de la ronde 2 après l'ouverture de la ronde 3 → vue courante sans effet ; ronde en cours → `round_in_progress` ; tâche sans garde → `keep_alive_not_enabled` ; garde levée, `failed`, `cancelled`, enfant absent → `keep_alive_ended` ; 9e ronde → `round_limit` ; autre parent → `task_unavailable` ; pause → `referent_paused` (rejeu admis pendant la pause) ; grant retiré → refus existant ; source de politique modifiée → `settings_revision_changed` ; échec de la ronde 2 (échéance) → tâche `failed`, enfant arrêté ; politique du parent réduite sans révocation (attestation actuelle plus faible) → `permission_not_inherited`, politique inchangée → admise ; attestation absente → `permission_attestation_unavailable` ; `bridget_task_continue` à l'instant exact `idle_deadline_at` avant le tick → `keep_alive_ended`, enfant non prolongé ; résultat prêt mais pas encore envoyé par `tick` (`result_available`, `result_sent` faux) → `result_delivery_pending` sans écriture ni clé consommée, puis admission après l'envoi (`native153_ronde_continue_avant_envoi_reessayable`)
- [ ] T014 [US3] Extraire de `spawn_task` (`native_delegation.rs:535-553`) la fonction `recheck_task_authority(st, task)` (révocations, `recheck_permission_context_sources`, `recheck_context`) ; `spawn_task` l'appelle ; ajouter dans cette fonction, pour `Continue` seulement, le recontrôle de la politique actuelle : relire le fait du parent par `parent_fact` (attestation actuelle ou politique figée du parent natif), recalculer `child_policy` pour l'enfant avec les entrées de la délégation : même cible, même `cwd` figé et la posture **demandée** de `request` (pas la posture effective ; `None` reste `None`), sans retirer aucun champ de sécurité de la comparaison et refuser si le résultat diffère de `permission_snapshot.child_policy` (`permission_not_inherited`) ; pour une tâche sans snapshot (voie 148), exiger le grant applicable par `root_permission` et `authorize_cwd` (`delegation_grant_required`) ; tests : ronde immédiate sans changement admise pour posture absente, `discovery` et `development`, parent Codex vers Codex, Codex vers Claude, Claude vers Claude, parent natif (preuve que l'égalité est stable ; si elle ne l'est pas, documenter l'écart dans `couplage.md` et demander arbitrage : ne pas affaiblir la comparaison seul), réduction de droits refusée ; test de non-régression `native148` identique avant et après
- [ ] T015 [US3] Implémenter la branche `NativeDelegationRequest::Continue` dans `handle_locked_with_project` selon l'ordre de `plan.md` : identité, `invalid_request`, `brief_invalid`, lecture de la tâche par `task_id` et propriété (`can_own_task`, `recover_owner`), rejeu par `rounds` (SHA-256 des octets canoniques, `sha2` déjà présent), `AutonomousEffect::NativeDelegation` (150), révocations, `keep_alive_not_enabled`, `result_delivery_pending`, `keep_alive_ended` ou `round_in_progress`, `round_limit` (8), échéance (`now >= idle_deadline_at` : lever la garde et `keep_alive_ended`), présence de l'enfant (sinon lever la garde et `keep_alive_ended`), `recheck_task_authority` (révocations, sources, politique actuelle), puis `begin_round` qui relit l'heure lui-même (I7) ; ajouter `Continue` au calcul de `drives` (`:297-300`) pour que `tick` envoie la mission aussitôt ; annoter la complexité O(log n + r) ; T013 passe
- [ ] T016 [US3] Utiliser `task.result_message_id()` aux trois lieux : envoi du résultat (`native_delegation.rs:1216`), `authorize_retained_result` (`:680`) et la lecture de la phase `acked` de 150 (T010 de 150) ; vérifier que le résultat de la ronde 1 garde `native-result-<task_id>` ; tests `native153_ronde_resultat_id_distinct_par_ronde` (deux résultats, deux identifiants, deux remises) et `native153_ronde_accuse_recopie_par_ronde` (la preuve `delivery_acked` suit l'identifiant de la ronde courante)
- [ ] T028 [US3] Réponses tardives : dans `capture_reply` (`native_delegation.rs:771`), quand `for_mission` ne trouve rien, chercher `past_mission_task(correlation)` ; si l'expéditeur est authentifié comme l'enfant de la tâche et le destinataire son parent (mêmes contrôles qu'aujourd'hui), rendre `Ok(true)` sans écrire ni dispatcher ; rouge d'abord : `native153_ronde_reponse_tardive_chemin_idempotent` (`daemon.rs:8724`), `native153_ronde_reponse_tardive_chemin_ordinaire` (`daemon.rs:17562`), `native153_ronde_reponse_tardive_rejeu` (aucun réveil du parent, tâche inchangée, même résultat au rejeu), `native153_ronde_reponse_tardive_expediteur_inconnu` (un tiers n'est pas absorbé)
- [ ] T017 [US3] Exposer l'outil : `delegation_mcp.rs` (outil `bridget_task_continue`, liste blanche des champs, correspondance vers `Continue`, champs `keep_alive_secs` et `brief` de `bridget_delegate` avec le schéma du brouillon de `bridget_handoff`, description des avertissements du dossier) ; `mcp.rs` (dispatch `:473-480`, liste `:2608` et son test) ; `wrapper.rs` (`BRIDGET_SAFE_MCP_TOOLS` de 20 à 21, listes d'approbation Codex autour de `:8118`, `:8150`, `:8172`) ; `crates/bridget-daemon/tests/claude_native_permissions_test.rs:400` ; test : le catalogue MCP expose l'outil, un champ inconnu est refusé
- [ ] T018 [US3] Dans `task_view` (`native_delegation.rs:68`), ajouter `round`, `idle_deadline_at`, `continuable` (et `round` aux entrées d'attente de 151) ; test `native153_ronde_vue_statut` ; vérifier que la projection Lineage (`task_entry`) ne change pas de forme

- [ ] T029 [US4] Preuve d'accusé par ronde : étendre la recopie de l'entretien de 150 (T010 de 150) pour lire la phase de `result_message_id(task, r)` pour chaque `r` de `unacked_rounds` et retirer les rondes accusées (avant `purge_expired_sends`), étendre le prédicat de purge (`delivery_acked && unacked_rounds.is_empty()`), supprimer les lignes `native_delegation_past_missions` dans la suppression atomique d'un arbre (T007 de 150) ; rouge d'abord : `native153_limites_ronde_indeterminee_puis_n1_accusee_purge_refusee` (N indéterminée, N+1 accusée, arbre conservé), `native153_limites_accuse_tardif_de_n_autorise_la_purge`, `native153_limites_purge_supprime_missions_passees`

## Phase 6 - US4 et US5 Bout en bout, compatibilité

**Test indépendant** : filtres `native153_e2e`, `native148`, `native149`, `native150`.

- [ ] T019 [US4] Test d'intégration `native153_e2e_trois_rondes` dans `crates/bridget-daemon/tests/native_delegation_e2e.rs` : revue (ronde 1), corrections (ronde 2 avec brief delta), nouvelle revue (ronde 3), puis fin explicite ; vérifier une seule instance d'enfant, trois missions, trois résultats avec trois identifiants, `child_instance` constant, trois mutations visibles de lignée, 16 enfants inactifs puis une 17e délégation refusée
- [ ] T020 [US4] Test `native153_limites_lecteur_lignee` : après `result_available` puis `bridget_task_continue`, un lecteur Lineage reçoit une mutation et sa relecture montre `mission_pending` puis `working`, sans tâche fantôme ; la fin explicite ne produit pas de mutation visible
- [ ] T021 [US5] Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148/149/150/087/103 ne passe de vert à rouge ; consigner dans `specs/153-rondes-contexte/validation/tests-153.md`

## Phase 7 - Documentation et clôture

- [ ] T022 [P] Documenter dans `docs/delegation-native.md` une section « Rondes et brief » (garde, `bridget_task_continue`, fin explicite, clé distincte par ronde, brief, plafond, sept refus, limites : redémarrage, mémoire et biais, contre-revue indépendante = nouvelle délégation) ; mettre à jour `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md` (nouvel outil, champs, refus)
- [ ] T023 [P] Écrire l'ADR `docs/decisions/153-rondes-et-contexte-delegation-native.md` (contexte, décisions D1 à D12 du plan, alternatives rejetées, conséquences, statut Accepté)
- [ ] T024 Recette réelle de mémoire (`quickstart.md`, environnement isolé, sans relance de service) ; consigner le résultat ou « non vérifié » dans `validation/tests-153.md`
- [ ] T025 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests connus sensibles à la charge avant de conclure ; consigner les compteurs dans `validation/tests-153.md`
- [ ] T026 Relire le diff complet (Articles XIX et XX : lignes suppressibles, abstractions, chemins d'erreur, champs par ronde oubliés) ; contrôler la liste de `couplage.md` ; écrire `specs/153-rondes-contexte/implementation.md` et enregistrer `git diff` dans `specs/153-rondes-contexte/validation/diff-153.patch`
- [ ] T027 Mettre à jour `Tâches: N/29` et le statut dans la fiche de `spec.md`, puis cocher `checklists/requirements.md` pour les points vérifiés par les tests

## Dépendances

- T001 avant tout.
- T002, T003, T004 en parallèle (fichiers distincts), avant les phases 3 à 5.
- Phase 3 (US1) et Phase 4 (US2) touchent le même fichier `native_delegation.rs` :
  exécution séquentielle. Phase 5 (US3) dépend des deux.
- T012 et T029 dépendent de la livraison de 150 (prédicats de comptage et de purge, recopie des accusés, suppression d'arbre).
- L'ordre d'exécution suit les phases ; T028 et T029 ont été ajoutés après la contre-revue et gardent leur numéro.
- T016 dépend de T010 de 150 (lecture de la phase `acked`). T028 dépend de T003 et T004.
- T022 et T023 en parallèle, après T018.
- T024 à T027 en dernier.

## Exemples de parallélisme

- T002, T003, T004 : trois fichiers.
- T022, T023 : documentation seule.

## Stratégie

MVP = Phases 1 à 3 : brief structuré, utilisable seul et sans risque d'état.
Puis garde (US2), rondes (US3), bout en bout et documentation.

## Charge future (Article XX)

- Réduisent la charge : T006 et T007 (le parent ne recolle plus le contexte à la
  main), T015 (plus de nouvelle délégation pour une ronde de plus).
- Ajoutent de la complexité justifiée : T028 (réponse tardive absorbée, évite un réveil parasite du parent), T029 (aucun résultat indéterminé n'est oublié), T004 (écriture atomique d'une ronde,
  exigée par `stamp` et la purge 150), T014 (un contrôle de sécurité, deux
  usages), T016 (un identifiant par ronde, exigé par l'idempotence du résultat).
