# Tâches - Session152 reprise des missions natives

**Entrées** : `spec.md`, `plan.md`, `research.md` (R1-R12), `data-model.md`,
`contracts/reprise152.md`, `reuse-audit.md`, `analyze-report.md`, et
`specs/150-socle-delegation/adversarial-review-codex.md` (ronde 1, objections 1
à 4 et 7).
**Racine** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/150-socle-delegation`.
**Recette de test** : `quickstart.md` (`umask 077`, racine `/tmp/b.XXXX`,
`BRIDGET_HOME` privé, `--features test-support`).
**Fichiers** : `N` = `crates/bridget-daemon/src/daemon/native_delegation.rs`,
`D` = `crates/bridget-daemon/src/daemon.rs`,
`E` = `crates/bridget-daemon/src/execution_store.rs`,
`T` = `crates/bridget-daemon/src/delegation.rs`.

Règle : une tâche n'est cochée qu'après son test vert. Les tests « rouge d'abord »
sont écrits et vus en échec sur le code livré par 150 et 151 avant le code.

## Phase 1 - Préparation et tests rouges (reproduire d'abord)

- [ ] T001 Établir la référence de non-régression : lancer les filtres `native148`, `native149`, `native150`, `delegation149` et `spec_087` avec la recette, consigner les compteurs PASS/FAIL/ignorés dans `specs/152-reprise-missions/validation/baseline.md`
- [ ] T002 Écrire dans `N` (module de tests) une fixture de mission `working` sous pause : tâche native `working`, exécution `execution-{mission}` en `running`, présence de l'enfant, ligne `control_pause_interruptions`, et des fonctions qui rejouent les événements du wrapper (`TurnState(false)`, `ExecutionStateChanged(interrupted)`, `DeliveryRejected "tour annulé par le pilote"`) dans l'ordre voulu, en passant par le gestionnaire complet `WrapperToDaemon::DeliveryRejected` (pas seulement `rejected()`) avec une demande suivie dans `pending_replies` et un parent connecté qui capte ses messages
- [ ] T003 [US1] Rouge d'abord : tests `native152_rouge_pause_*` qui reproduisent l'échec d'une mission native sous pause : chemin A (`rejected()` après l'interruption → `failed`, `N:810-828`), chemin B (`tick` lit `interrupted` → `failed`, `N:1193-1203`), ordre inverse (`TurnState(false)` puis reprise avant `interrupted`, exécution encore `interrupting`). Ajouter au chemin A la réception complète : sur le code actuel, la demande suivie disparaît de `pending_replies` et le parent reçoit `Échec de livraison de la demande` (`daemon.rs:18095-18125`). Les trois tests échouent sur le code actuel
- [ ] T004 [US3] Rouge d'abord : test `native152_rouge_panne_mission_working` (`prepare_restart` passe la tâche `failed/unreachable` et clôt l'exécution, `N:922-952`) ; ajouter les tests qui prouvent H4 et R13 : (a) enfant en cours de lancement, relancé par `prepare_recovery` : bootstrap natif avec le bon `instance_id` (`wrapper.rs:470-490`) ; (b) enfant déjà connecté, relancé par la boucle `desired.equipiers` (`daemon.rs:3995-4020`) : sur le code actuel la nouvelle génération n'a ni lien natif ni `delegation_id`, et `parent_fact`, l'admission d'un descendant et `restart_identities` échouent ; (c) deux redémarrages successifs du magasin ; (d) une mission imbriquée (petit-enfant). Consigner les résultats dans `research.md` R6 et R13
- [ ] T005 [US2] Rouge d'abord : tests `native152_rouge_horloges` : l'échéance de mission expire pendant la pause (`N:1013-1023`) ; le palier 3 du suivi de demande est émis pendant la pause (`D:6900-6916`). Ajouter une recherche qui prouve qu'aucun balayage ne lit `tracked_requests.deadline_at` (R4)
- [ ] T006 [US2] Rouge d'abord : test `native152_rouge_lancement_apres_pause` : tâche `queued` retenue 600 s, `submit_spawn` refuse l'ordre expiré ou l'enveloppe change (`fleet.rs:961`, `:1662-1696`). Ajouter le cas d'une saga déjà soumise : son enveloppe ne doit pas bouger
- [ ] T007 [US4] Rouge d'abord : test `native152_rouge_annulation_partielle` : arbre de trois membres dont un `NotManaged` ; l'annulation actuelle arrête deux membres puis échoue (`N:1380-1486`)
- [ ] T038 [US3] Rouge d'abord : tests `native152_rouge_panne_pendant_pause` : (a) panne avec pause encore active au démarrage ; (b) panne avec pause levée pendant l'arrêt. Sur le code actuel (et sur un T023 naïf), `prepare_restart` teste l'échéance brute et peut donner `mission_reply_timeout` ; une exécution `interrupting` dont le processus a disparu n'obtient jamais `interrupted`
- [ ] T039 [US2] Rouge d'abord : test `native152_rouge_gel_mixte` : pause de 600 s en `queued`, lancement, envoi de la mission, demande suivie de 120 s, reconnexion de l'agent (`restore_pending_for_agent`). Sur un gel cumulé, le temps écoulé est faussé (`D:3860-3866`) ; la date d'envoi doit porter l'échéance du suivi (`D:8816-8824`)
- [ ] T040 [US4] Rouge d'abord : test `native152_rouge_annulation_multi_ticks` : racine et deux descendants, un membre non arrêtable ; sur plusieurs ticks, les descendants marqués `cancelling` (`N:1390-1394`) deviennent des racines d'annulation (`N:994-996`) et sont arrêtés malgré le refus global
- [ ] T047 [US3] Rouge d'abord : test `native152_rouge_trois_crashes_avant_envoi` : tâche `starting` puis `mission_pending`, aucune exécution de mission ; trois redémarrages successifs. Sur un compte dérivé des continuations, le compte reste à 0 (`N:608`, les relances de l'enfant ne créent aucune continuation) et la borne n'opère jamais
- [ ] T048 [US3] Rouge d'abord : test `native152_rouge_identite_frontieres` : crash simulé à chaque frontière de sauvegarde d'une reprise (entrée de flotte neuve mais `child_instance` ancien ; instance neuve sans entrée à jour ; lien absent) ; sur le code actuel, `restart_identities` rend `Err` et le démarrage échoue (`N:845-868`, `daemon.rs:4553-4555`)
- [ ] T049 [US2] Rouge d'abord : tests `native152_rouge_pause_sans_tick` : (a) pause enregistrée, crash avant le premier tick, pause levée pendant l'arrêt : aucun décalage ne peut être retrouvé sans état intermédiaire ; (b) admission de `send_as` (ligne `tracked_requests` écrite), crash avant la sauvegarde `working`, pause, reconnexion : la pause n'est pas retirée du suivi (`N:1154`, `:1188`)

## Phase 2 - Fondations (bloquantes)

- [ ] T008 [P] Ajouter à la struct `Task` de `T` les champs `launch_issued_at`, `mission_issued_at`, `restart_resumes` (u8), `cancel_stopped` avec `#[serde(default)]` ; test : un payload 151 sans ces champs se relit avec les défauts, et `visible_changed` les ignore (aucun signal Lineage). Ronde 2 : pas de `suspended_at`, `request_frozen_secs` ni `fleet_command_id` (valeurs dérivées, R22 à R24)
- [ ] T009 [P] Ajouter `AutonomousEffect::NativeMissionDispatch` dans `crates/bridget-daemon/src/referent_control.rs` et la table de vérité de `admit_autonomous_effect` (pause → `Deferred { motif: "pause" }`) ; test par table de vérité
- [ ] T010 [P] Ajouter dans `E` `latest_in_chain(root_execution_id)` (CTE récursif de `E:818`, feuille de plus grande `generation`) et `abandon_pause_interruption(execution_id)` ; tests : chaîne de 1, 2 et plus de 3 exécutions (pauses incluses) ; deux exécutions créées à la même seconde avec des identifiants dans l'ordre inverse de la filiation ; `resume_execution_id` posé par `reconstruct_active_for_agent` désigne la même exécution que la continuation ; annoter la complexité O(c), c = pauses et reprises
- [ ] T011 [P] Ajouter `frozen_secs: u64` à `PendingReply` dans `D` (valeur 0 partout où la structure est construite) ; test : une demande sans suspension se comporte comme avant
- [ ] T050 [P] Ajouter dans `crates/bridget-daemon/src/referent_control.rs` `pause_overlap(conn, from, to) -> i64` : somme des secondes de pause comprises entre `from` et `to`, reconstruite depuis `control_events` (`pause_on`, `pause_off`) et `control_state` (intervalle encore ouvert) ; l'état au début de la fenêtre vient du dernier événement avant `from` ; tests : aucune pause, une pause fermée, une pause ouverte, plusieurs pauses, fenêtre qui commence en pause, événements à la même seconde ; complexité O(P), P = événements dans la fenêtre

## Phase 3 - US1 La pause ne fait plus échouer les missions (P1)

**But** : suspendre au lieu d'échouer, reprendre avec l'identifiant d'origine.
**Test indépendant** : filtre `native152_pause`.

- [ ] T012 [US1] Dans `tick` de `N` : lire l'état de contrôle une fois par tick ; en pause, avant la vérification d'échéance, sauter les tâches `queued`, `starting`, `mission_pending`, `working` sans écrire ni changer d'état ; laisser passer `cancelling`, `waiting_for_children`, `result_available`, `failed` ; la suspension est un fait dérivé (pause active et état gelé), pas un champ ; test `native152_pause_tick_ne_perime_pas_l_echeance`
- [ ] T013 [US1] Dans `rejected()` de `N` : ne pas passer `failed` quand l'exécution de la chaîne de la mission a une ligne de pause non reprise (ou reprise par cette pause) et que la raison est le refus du pilote ; garder le comportement actuel sinon ; faire rendre à `rejected()` le booléen `pause_interruption` (voir T044) ; test `native152_pause_chemin_a_refus_de_remise_ne_fait_pas_echouer` (T003 chemin A vert) et `native152_pause_refus_hors_pause_echoue_toujours`
- [ ] T014 [US1] Remplacer `execution-{mission}` par `latest_in_chain` dans `tick` (`N:1186-1203`), `close_lost_execution` (`N:900-905`) et `descendants_busy` (`N:1349-1368`) ; un état `interrupted` couvert par une ligne de pause non reprise ne fait plus échouer la tâche ; tests `native152_pause_chemin_b_interrupted_ne_fait_pas_echouer` et `native152_pause_chaine_suivie_apres_reprise` (T003 chemin B vert)
- [ ] T015 [US1] Remplacer la garde de `D:4251-4258` par `native_delegation::may_resume(st, task, parent_execution_id)` : vrai si la tâche est `working`, non `cancelling`, l'exécution appartient à la chaîne de la mission, et (cause pause) l'état est `interrupted` ; ajouter dans `tick` l'appel à `schedule_execution_recovery` pour les tâches suspendues quand la pause est levée, l'enfant est connecté et libre ; la reprise réutilise le message d'origine ; tests `native152_pause_reprise_attend_interrupted` (ordre inverse de T003 vert), `native152_pause_reprise_meme_identifiant_de_mission`, `native152_pause_resultat_rendu_une_fois`
- [ ] T044 [US1] Dans le gestionnaire `WrapperToDaemon::DeliveryRejected` de `D:18078-18125` : si `rejected()` qualifie une interruption de pause, retourner `None` avant `pending_replies.retain`, avant `get_request` et avant `deliver_to_agent` ; tests par l'événement complet `native152_pause_chemin_a_suivi_conserve_sans_avis` (la demande reste dans `pending_replies`, le parent ne reçoit aucun avis) et `native152_pause_refus_hors_pause_notifie_toujours` (T003 chemin A complet vert)
- [ ] T016 [US1] Annulation pendant la pause : `may_resume` refuse `cancelling` et `cancelled` ; `cancel_tree` appelle `abandon_pause_interruption` pour l'exécution de la chaîne ; test `native152_pause_annulee_pendant_la_pause_n_est_pas_ressuscitee`

## Phase 4 - US2 Gel des horloges, aucun lancement pendant la pause (P1)

**Test indépendant** : filtres `native152_horloges`, `native152_lancement`.

- [ ] T017 [US2] Échéance effective : remplacer le test d'échéance de `tick` (`N:1013-1023`) par `effective_deadline(task, now) = mission_deadline_at + pause_overlap(mission_deadline_at - reply_timeout, now)` (T050) ; `mission_deadline_at` n'est jamais réécrite ; aucun état de suspension à régler ; tests `native152_horloges_echeance_gelee_et_decalee` (SC-003), `native152_horloges_echeance_sans_tick_pendant_la_pause` (crash après l'enregistrement de la pause, avant le premier tick, pause levée pendant l'arrêt), `native152_horloges_plusieurs_pauses_cumulees`
- [ ] T018 [US2] Suivi de demande : dans `collect_reminder_actions` de `D`, ignorer les demandes de mission native tant que la pause est active (ni palier, ni expiration) ; calculer le temps écoulé avec `frozen_secs = pause_overlap(request.created_at, now)`, où `request.created_at` vient de la ligne durable `tracked_requests` de la mission (donc de l'admission réelle de l'envoi, pas de l'état `working`) ; affecter `frozen_secs` à chaque tick et dans `restore_pending_for_agent` ; tests `native152_horloges_palier3_non_emis_en_pause`, `native152_horloges_suivi_survit_a_la_reconnexion` (T005 vert), `native152_horloges_pause_queued_ne_retire_rien_a_la_demande` (T039 vert), `native152_horloges_admission_crash_pause_reconnexion` (T049 vert : admission de l'envoi, crash avant la sauvegarde `working`, pause, reconnexion)
- [ ] T045 [US2] Date d'émission de la mission : poser `mission_issued_at = now` et sauvegarder avant la première tentative d'envoi de `mission_pending`, puis l'utiliser comme `issued_at` de `send_as` (au lieu de `created_at`, `N:1148-1156`) ; les tâches sans le champ gardent `created_at` ; tests `native152_horloges_suivi_part_de_la_date_d_envoi` et `native152_horloges_rejeu_d_envoi_meme_enveloppe`
- [ ] T019 [US2] Garde unique : appeler `admit_autonomous_effect(NativeMissionDispatch, …)` avant `spawn_task` et avant la remise de mission de `tick` (`N:1130-1190`) ; différé en pause sans changer l'état ; l'envoi du résultat et de l'avis d'échec reste admis ; tests `native152_pause_aucun_lancement_ni_remise` (compteur 0 sur 300 ticks simulés, SC-002), `native152_pause_resultat_acquis_reste_remis`
- [ ] T020 [US2] Enveloppe de lancement : dans `spawn_task` de `N`, poser `launch_issued_at = now` et sauvegarder AVANT le premier `submit_spawn` d'une tâche `queued` sans saga pour `command_id = task_id` (`spawn_commands.rs:313`) ; utiliser `launch_issued_at.unwrap_or(created_at)` pour `issued_at` et `+300` pour `deadline_at` ; une saga existante n'est jamais modifiée ; tests `native152_lancement_retenu_600s_reussit` (T006 vert, SC-004), `native152_lancement_rejeu_meme_enveloppe`, `native152_lancement_saga_existante_inchangee`
- [ ] T021 [US2] Dans `task_view` de `N`, ajouter `suspended` (dérivé : pause active et état gelé) et `resume_attempts` (valeur de `restart_resumes`), additifs ; vérifier que la projection Lineage ne change pas ; tests `native152_vue_suspended_additive` et un test qui compare les octets de la projection Lineage avant et après une suspension

## Phase 5 - US3 Reprise après redémarrage (P1)

**Test indépendant** : filtre `native152_panne`.

- [ ] T022 [US3] Extraire de `spawn_task` (`N:585-603`) la fonction `native_child_env(task, prepared)` (politique, sources, bootstrap de mission) ; `spawn_task` l'utilise ; test de non-régression `native148_*` vert ; deux usages après T024
- [ ] T023 [US3] Dans `prepare_restart` de `N` : pour une tâche `starting`, `mission_pending` ou `working`, décider en un seul passage : identité vérifiée (T043), échéance effective non passée (`effective_deadline`, T017, donc gel de pause compris), `restart_resumes < 2` ; si oui, incrémenter `restart_resumes` et sauvegarder AVANT toute reprise, et garder la tâche et son exécution ; sinon `failed` avec `unreachable`, `mission_reply_timeout` ou `native_resume_exhausted` ; ne plus arrêter l'entrée de flotte d'un enfant reprenable ; ne rien changer pour `waiting_for_children`, `result_available` et les états terminaux ; tests `native152_panne_mission_working_n_echoue_pas` (T004 vert), `native152_panne_echeance_passee_echoue`, `native152_panne_identite_ko_echoue_unreachable`, `native152_panne_queued_proprietaire_perdu_inchange`, `native152_panne_conserve_waiting_for_children`, `native152_panne_conserve_result_available`, `native152_panne_conserve_etats_terminaux`
- [ ] T024 [US3] Reprise de l'enfant, deux chemins dans `reserve_managed_recoveries` de `D:3886-4020` : (a) candidat en cours de lancement : ne plus l'exclure, passer par `prepare_recovery` et compléter l'environnement par `native_child_env` ; (b) enfant déjà connecté (entrée `Running` de la boucle `desired.equipiers`) : ordre de reprise avec `ownership: ownership_from_desired(&equipier)` (`fleet.rs:1649`) au lieu de `None` et environnement natif par `native_child_env` ; adapter selon T004 ; tests `native152_panne_enfant_en_lancement_garde_la_filiation`, `native152_panne_enfant_connecte_garde_la_filiation`, `native152_panne_enfant_relance_avec_environnement_natif`, `native152_panne_deux_redemarrages_du_magasin`, `native152_panne_mission_imbriquee`
- [ ] T025 [US3] Borne de reprises : `restart_resumes` est incrémenté seulement par la décision de T023 (un seul endroit, avant toute reprise, donc sans sous-comptage possible ; un crash entre l'incrément et la reprise ne fait que sur-compter) ; une reprise de pause ne l'incrémente pas ; tests `native152_panne_borne_de_deux_reprises` (SC-005), `native152_panne_trois_crashes_avant_l_envoi` (T047 vert : états `starting` et `mission_pending`, aucune exécution de mission), `native152_panne_resultat_rendu_une_fois`, `native152_panne_pause_ne_compte_pas`, `native152_panne_crash_entre_increment_et_reprise_sur_compte_seulement`
- [ ] T041 [US3] Panne pendant la pause : `prepare_restart` utilise `effective_deadline` (journal des pauses, T050), qui couvre sans état intermédiaire : pause encore active au démarrage (aucune expiration), pause levée pendant l'arrêt, et crash après l'enregistrement de la pause mais avant le premier tick ; tests `native152_panne_pause_active_au_demarrage`, `native152_panne_pause_levee_pendant_l_arret`, `native152_panne_pause_enregistree_avant_le_premier_tick` (T038 et T049 verts, SC-009)
- [ ] T042 [US3] Réconcilier `interrupting` : dans `prepare_restart` (réconciliation des groupes achevée), passer par `transition_if_current` une exécution `interrupting` qui a une ligne de pause non reprise vers `interrupted` (raison `pause_restart_reconciled`) ; test `native152_panne_interrupting_reconcilie_puis_reprise` (T015 n'attend plus un terminal absent)
- [ ] T043 [US3] Identité au redémarrage, sans bloquer le daemon : (1) dériver la commande courante depuis le lien attesté (`current_command(st, task)` : commande de l'entrée de flotte du nom de l'enfant dont `agent_link.delegation_id == task_id` et `role == native_delegate`, sinon `task_id`) et l'utiliser dans `parent_fact` (`N:43-44`), l'admission d'un descendant (`N:462-463`), `restart_identities` (`N:850`, `:868`) et `queued_owner_lost` ; (2) dans `restart_identities`, réconcilier l'instance enfant depuis le lien de flotte attesté avant le contrôle ; (3) si une tâche ne peut pas être réconciliée, la passer `failed` avec `native_restart_identity_mismatch`, l'exclure de la table d'identités et continuer : ne plus rendre `Err` pour tout le daemon (`daemon.rs:4553-4555`). Justification Art. XIX : un échec nommé d'une tâche vaut mieux qu'un daemon qui ne démarre pas, et la réconciliation se limite à ce que le lien atteste ; tests `native152_filiation_controles_vrais_apres_reprise`, `native152_filiation_restart_identities_au_deuxieme_redemarrage`, `native152_filiation_crash_a_chaque_frontiere_de_sauvegarde` (T048 vert : entrée de flotte écrite sans instance de tâche, instance sans entrée, entrée sans lien), `native152_filiation_incoherence_n_empeche_pas_le_demarrage`

## Phase 6 - US4 Annulation en cascade vérifiée d'abord (P2)

**Test indépendant** : filtre `native152_annulation`.

- [ ] T026 [US4] Dans `D:9255-9318`, extraire `classify_stop(st, name) -> StopKind` sans effet de bord ; `prepare_managed_stop` l'utilise ; test de non-régression des arrêts existants (`native148_cancel_stops_native_descendants_before_parent`)
- [ ] T027 [US4] Dans `cancel_tree` de `N`, classer tous les membres AVANT toute mutation (marquage `cancelling` des descendants compris, `N:1390-1394`) et avant le premier arrêt ; si un membre est `Unmanaged`, ne rien muter ni arrêter, garder la racine `cancelling` et écrire `native_cancel_unverifiable:<enfant>` ; tests `native152_annulation_membre_non_arretable_n_arrete_rien` (T007 vert, SC-006), `native152_annulation_reessai_apres_correction`, `native152_annulation_aucun_descendant_marque_si_refus` (T040 vert sur plusieurs ticks)
- [ ] T046 [US4] Dans `tick` de `N` : ne lancer `cancel_tree` que pour les racines d'annulation ; une tâche `cancelling` dont un ancêtre est `cancelling` est ignorée tant que sa racine est en cours ; tests `native152_annulation_descendant_non_racine_independante` et `native152_annulation_reprise_coordonnee_depuis_la_racine`
- [ ] T028 [US4] Enregistrer les arrêts confirmés dans `cancel_stopped` de la tâche racine, sauvegardé après chaque arrêt confirmé ; un nouvel essai ou une reprise après crash saute ces noms ; tests `native152_annulation_reessai_ne_rejoue_que_les_echecs` (compteur d'arrêts), `native152_annulation_ordre_profond_d_abord`, `native152_annulation_crash_entre_deux_arrets_confirmes`

## Phase 7 - US5 Reçus de refus (P2)

**Test indépendant** : filtre `native152_refus`.

- [ ] T029 [US5] Appliquer la liste exacte de cinq codes (`cwd_outside_parent_project`, `cwd_unavailable`, `model_unavailable`, `effort_unavailable`, `agent_type_unknown`) ; ramener le texte dynamique de `registry.get` (`registry.rs:157`) au code `agent_type_unknown` à l'admission ; test rouge `native152_refus_rejeu_change_de_reponse` (le `cwd` disparaît entre deux rejeux) ; test `native152_refus_texte_dynamique_jamais_stocke` ; inventorier les autres `Err` de l'admission et vérifier qu'aucun n'est enregistré
- [ ] T030 [US5] Créer la table `native_delegation_refusals` dans `T` (ouverture du magasin), écrire le reçu pour les seuls codes de la liste fermée, rejouer le même refus avec le même canonique, `envelope_mismatch` sinon ; une tâche existante prime ; tests `native152_refus_rejeu_identique`, `native152_refus_transitoire_jamais_garde` (un test par code transitoire, SC-007), `native152_refus_autre_enveloppe_mismatch`
- [ ] T031 [US5] Purger les reçus de plus de 30 jours dans le passage d'entretien de la 150 (`run_maintenance` de `D`) ; test `native152_refus_purge_30_jours`

## Phase 8 - Compatibilité et finitions

- [ ] T032 Relancer les filtres de T001 et comparer à `validation/baseline.md` : aucun test 148, 149, 150 ou `spec_087` ne passe de vert à rouge ; consigner dans `specs/152-reprise-missions/validation/tests-152.md`
- [ ] T033 [P] Documenter la pause (suspension, gel des horloges, limite des lancements en cours), la reprise après panne (borne 2), l'annulation vérifiée d'abord et les reçus de refus dans `docs/delegation-native.md`, `skills/bridget/SKILL.md` et `skills/bridget/references/commandes.md`
- [ ] T034 [P] Annoter la complexité des fonctions publiques ajoutées (`latest_in_chain`, `classify_stop`, `may_resume`, gel des demandes) et vérifier l'axe Minimalisme (lignes supprimables, extractions à deux usages)
- [ ] T035 [P] Écrire l'ADR `docs/decisions/152-reprise-missions-natives.md` (contexte, décisions D1 à D20, conséquences, statut Accepté)
- [ ] T036 Lancer la suite complète (`cargo test --workspace --features bridget-daemon/test-support --no-fail-fast` avec la recette) ; rejouer seuls les tests connus sensibles à la charge avant de conclure ; consigner dans `validation/tests-152.md`
- [ ] T037 Relire le diff complet (Articles XIX et XX), écrire `specs/152-reprise-missions/implementation.md` et enregistrer `git diff` dans `specs/152-reprise-missions/validation/diff-152.patch`

## Dépendances

- T001 avant tout. T002 avant T003.
- T003 à T007, T038 à T040 et T047 à T049 (rouges) avant le code qui les rend verts.
- T008 à T011 en parallèle, avant les phases 3 à 7.
- Phase 3 avant Phase 4 (T017 à T020 utilisent l'état de pause lu en T012 et `pause_overlap` de T050).
- T014 avant T015 (la chaîne précède la garde de reprise).
- T022 avant T023 et T024. T023 avant T025. T017 et T041 avant T023 (l'échéance effective précède la décision). T043 avant T023 et T024.
- T038 à T040 (rouges) avant les tâches qui les rendent verts : T038 → T041, T042 ; T039 → T018, T045 ; T047 → T025 ; T048 → T043 ; T049 → T017, T018, T041 ; T040 → T027, T046.
- T044 avant T013 ne s'applique pas : T013 puis T044 (le gestionnaire lit le booléen de `rejected()`).
- T050 (`pause_overlap`) avant T012, T017, T018, T041.
- T026 avant T027. T027 avant T028 et T046.
- T029 avant T030. T030 avant T031.
- Les phases 3 à 7 touchent `native_delegation.rs` : exécution séquentielle.
- T032 à T037 en dernier ; T033, T034 et T035 en parallèle.

## Exemples de parallélisme

- T008, T009, T010, T011 : fichiers distincts.
- T033, T034, T035 : documentation et relecture seules.

## Stratégie

MVP = Phases 1 à 4 : une pause ne fait plus échouer les missions et gèle les
horloges. Puis la reprise après panne (US3), l'annulation (US4), les reçus de
refus (US5), les finitions.

## Charge future (Article XX)

- Réduisent la charge : T014 (une seule façon de désigner l'exécution d'une
  mission), T023 (retrait de la mise à `failed` au redémarrage), T026
  (classification séparée de l'effet).
- Ajoutent de la complexité justifiée : T043 (commande de flotte portée par la tâche, exigée par la contre-revue 152), T012 et T017 (gel des horloges, exigé par
  la contre-revue 150), T020 (enveloppe de lancement figée), T030 (reçus de refus
  limités aux refus dépendants de l'état du monde).
