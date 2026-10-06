# Analyse 138 — Cohérence avant implémentation

Date: 2026-10-06
Décision documentaire après seconde lecture: PASS
Portée: quatre histoires, quatorze exigences, six critères et vingt tâches.
Cette décision autorise le découpage technique. Elle ne déclare aucun code ni
test138 exécuté.

## Première lecture

Le principal a comparé la spécification, le plan, le modèle, le contrat, les
tâches et l'audit de réutilisation. Aucun finding CRITICAL n'a été identifié.
L'audit initial était PASS: 17/17 items. Le complément contrôle est PASS:
18/18 items et cinq critères cochés. Vingt tâches sont maintenant définies.
La lecture indépendante du plan a relevé trois défauts concrets. Ils ont été
retenus après comparaison au code existant, puis corrigés avant l'implémentation.

## Couverture exigences, tâches et critères

| Exigence | Résultat attendu | Tâches | Critères |
|---|---|---|---|
| FR-13801 | Suggestions locales sans repli extérieur | T001/T003–006/T011–012/T016 | SC-13801/04/06 |
| FR-13802 | Global volontaire distinct du mandat | T003–006/T016 | SC-13801/02/06 |
| FR-13803 | Preuves transport/Git et identités robustes | T003–006/T014/T016 | SC-13805/06 |
| FR-13804 | Inconnu visible et legacy averti | T003–006/T007–010/T014/T016 | SC-13801/03/05/06 |
| FR-13805 | Motif volontaire avant dépôt extérieur | T007–010/T014/T016/T019 | SC-13802/03/06 |
| FR-13806 | Warning caller hors corps, sans second dialogue | T007–010/T016/T019 | SC-13803/06 |
| FR-13807 | Mandat borné, réponse corrélée et rappels | T007–013/T016 | SC-13803/04/06 |
| FR-13808 | Tous les lecteurs des fils mixtes contrôlés | T007–010/T014/T016 | SC-13802/03/06 |
| FR-13809 | Boucles locales sans repli automatique | T011–013/T016 | SC-13804/06 |
| FR-13810 | Trois rôles mandatés et ROOT sans mandat | T011–013/T016 | SC-13804/06 |
| FR-13811 | Historique exact et aucun rejeu | T007–008/T014–015/T016 | SC-13806 |
| FR-13812 | Résultat idempotent et enveloppe figée | T007–008/T011–012/T014–015/T019/T020 | SC-13806 |
| FR-13813 | Façades cohérentes et négociation ancien pair | T005–010/T013/T016–017/T019 | SC-13801/02/03/06 |
| FR-13814 | Réutilisation et essais isolés | T002/T004/T008/T012–020 | SC-13806 |

Lors de la seconde lecture, les dix-huit tâches étaient non cochées. Le principal
a ensuite validé T001/T002 et demandé leur clôture: 2/18 tâches terminées, les
seize tâches restantes ouvertes. Le statut In Progress ne déclare aucune livraison.

Complément de couverture: T009/T010 incluent les façades handoff et attach,
après preuve des contrats HandoffTransport/parse_request et JournalRequest
existants. Elles utilisent la garde partagée daemon, sans abstraction nouvelle.

## Anomalies corrigées

| Finding | Preuve observée par le principal | Correction retenue | Vérification prévue |
|---|---|---|---|
| Client background sans contexte utilisable | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:350 retire l'identité T3 héritée | Contexte autonome sur connexion Client négociée, racine/hôte validés, sans Register d'agent ni identité empruntée ; plan/modèle/contrat et T004 alignés | T003/T004 et scénario du client de fond |
| Reprise d'outbox sous un mandat modifié | /Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py:3602 et :3626 décrivent l'enveloppe de notification/reprise | Racine, motif, destinataire et corps figés avant la première tentative ; T011 teste modification après échec | T011/T012 et scénario de reprise d'outbox |
| Ancien serveur susceptible d'ignorer un champ | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-core/src/message.rs:74 ne porte pas deny_unknown_fields | Le client exige la capacité138 avant l'enveloppe à motif ; aucun downgrade sans motif | T009/T010 et scénario serveur sans capacité |
| Membre inconnu susceptible de masquer une divergence connue | Matrice du fil A/B/U relue avec FR-13804/05/08 | Vérifier chaque lecteur ; l'inconnu U ne supprime pas le contrôle A/B ; T014 complété | Fil A/B/U sans motif refusé avant dépôt |

La preuve est une lecture du code. Le comportement corrigé devra être démontré
par les tests. Aucune capacité du vieux serveur n'est supposée depuis sa seule
sérialisation. Aucun consentement historique n'est ajouté. Le complément
contrôle ci-dessous justifie une seule colonne de warnings durable.

## Deuxième lecture

Les trois corrections du plan ont été rapprochées du modèle, du contrat et des
tâches. T004, T011 et T014 ont reçu les assertions manquantes. Le Gherkin138
décrit les cas observables avant toute implémentation. Chaque FR et SC possède
une couverture identifiée. Aucun finding CRITICAL ni arbitrage ouvert ne reste
dans cette lecture documentaire.

L'avertissement est établi dans la prévalidation avant dépôt ou notification.
Le caller le reçoit dans le résultat de l'outil après traitement. Cette séquence
n'est pas un dialogue humain de confirmation en deux temps. La demande explicite
valide suffit. Le warning ne modifie pas le corps remis.

La règle de portée garde le contexte de collaboration. Elle n'est pas une
protection générale contre un client malveillant. Les accès et preuves existants
restent requis. La compatibilité inconnue est visible et ne neutralise jamais
une divergence connue entre d'autres membres.

## Gates documentaires

Réutilisation: PASS. Couverture: 14/14 FR, 6/6 SC et 4/4 US.
Complexité: O(1) par destinataire, O(A) annuaire et O(M) audience bornée.
Minimalisme: pas de registre, table, dépendance ou service supplémentaire.
Une colonne de warnings de contrôle constitue l'exception explicite justifiée.
État de validation du produit: non validé ; les suites et preuves restent à produire.
Prochaine étape: autorisation d'implémentation par le principal après cette lecture.

## Analyze complémentaire — consigne injectée

La lecture de handle_execution_control dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs:8656
confirme un chemin Client + ExecutionControlV1 pour SteerCurrent.message.
Ce contenu entre dans le contexte de la cible et relève de FR-13805/06/12/13.
L'absence de garde de portée sur ce chemin est un finding à corriger, pas un
comportement accepté. T019 ajoute d'abord les tests RED, puis la garde commune.

reserve_control_command et init_schema dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs:1084
et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs:1531
offrent la réservation, la comparaison exacte et la table durable existantes.
Décision: une colonne JSON project_warnings avec défaut [], migration compatible
selon les helpers d'introspection existants. Aucun nouveau service ni table.
Ne jamais détourner canonical_bytes ou refusal_reason pour conserver le warning.

Seconde lecture du complément: plan, modèle, contrat, audit18/18, T019 et Gherkin
sont cohérents. Canon legacy et Interrupt sans message restent inchangés.
Le résultat accepté est rendu avant toute garde mutable, même après changement
des faits et restart ; aucune nouvelle injection. Warning établi avant effet,
retourné au caller hors corps, sans seconde confirmation humaine.
PASS documentaire complémentaire ; aucun CRITICAL documentaire ouvert.
T019 reste non cochée. Aucun test de contrôle nouveau n'est revendiqué passé.

## Avancement autorisé

Le principal a validé T011/T012/T013 sur
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop.md
et sa propre exécution dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-main-validation.log:
145 tests, OK, exit0 rapporté. Les deux findings adverses CAS de première
réservation et digest de mandats ont leurs RED/correctifs/tests consignés.
Une seconde revue a trouvé une course P1 résiduelle : un résultat publié après
unlock mais avant archive peut perdre son chemin canonique. T012 conserve sa
clôture historique validée. T020 corrective a imposé un test interleave
RED puis une publication/archive sous le même verrou existant. Elle dépend de
T012. Les correctifs de course et E/S ont ensuite été validés par le principal :
152 tests Python PASS (124 existants + 28 nouveaux), exit0, revue corrective
indépendante APPROVE. Preuve :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-final-validation.log.
T020 est clôturée sur son autorisation. Cette étape était à 6/20 terminées.
Le PASS documentaire T019 n'était pas un PASS produit.
Les crashs machine, rollback défaillant, writers externes et chemins legacy
non convertis restent hors des garanties de cette corrective.

## État après validation des tranches

Le principal a relu les sources/tests, les logs noyau25 PASS, contrôle processus
réel1 PASS/1 helper ignoré, façades25 PASS et recette Agent Loop réelle1 PASS.
Les passages25 PASS concernent le même filtre et ne sont pas additionnés.
Skills Bridget et Agent Loop Codex/Claude validées. Il autorise la clôture
T003–T010, T014–T016 et T019. État actuel : 18/20, In Progress ; T017/T018 ouvertes.
La revue indépendante T019 est APPROVE, même fournisseur et borne T019 seulement.

La première vague RED commune comporte cinq échecs réels. Les autres tests
n'ont pas tous une RED séparée exécutée ; aucune preuve n'est inventée.
Le workspace complet a d'abord échoué à compiler sur quatre mises à jour
mécaniques de tests (deux champs protocole, deux patterns Ack), corrigées par
le principal sans logique de production. Deuxième run en cours ; aucun PASS
global ni statut Implemented. L'Analyze final relève toujours de T018.

## Matrice finale après gel des sources

Le principal a confirmé le gel des sources. Les 111 repères ci-dessous ont été
rafraîchis depuis les fichiers. Cette matrice couvre les 14 FR et six SC.
Elle remplace les repères provisoires transmis avant le test homonyme et
l'extraction Agent Loop. Elle ne ferme ni T017 ni T018.

Notation : A:36 signifie le chemin absolu du catalogue A, ligne36.
Chaque ligne du catalogue nomme exactement la fonction ou le test concerné.
Une fonction et son test peuvent partager le même fichier. Le repère de branche
DirectoryScoped B:11515 appartient à handle_wrapper_message B:10927.

### Catalogue exact des fonctions et tests

#### A — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication.rs

- Fonction `validate_cross_project_reason` : 12.
- Fonction `project_relation` : 25.
- Fonction `project_scope` : 36.
- Fonction `resolve_communication_project` : 66.
- Fonction `canonical_send` : 184.
- Test `spec138_scope_matrix_and_reason_bounds` : 228.
- Test `spec138_git_environment_does_not_override_project` : 262.
- Test `spec138_real_worktree_and_symlink_share_common_root` : 301.
- Test `spec138_same_basename_repositories_remain_other_projects` : 353.
- Test `spec138_reason_is_part_of_canonical_envelope` : 404.
- Test `spec138_explicit_null_reason_is_not_legacy_absence` : 416.

#### B — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/daemon.rs

- Fonction `connection_communication_project` : 7926.
- Fonction `agent_communication_project` : 7942.
- Fonction `announce_communication_project` : 7956.
- Fonction `prepare_dispatch` : 8148.
- Fonction `control_result_with_project` : 8627.
- Fonction `handle_execution_control` : 8669.
- Fonction `handle_wrapper_message` : 10927.
- Test `spec138_direct_guard_and_scoped_directory_precede_delivery` : 9734.
- Test `spec138_project_announcement_recovers_absence_but_not_conflict_and_aux_inherits` : 9800.
- Test `spec138_reply_grant_and_warning_survive_replay_after_closed_request` : 9888.
- Test `spec138_steer_guard_refuses_crossproject_before_provider` : 10052.

#### C — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/threads.rs

- Fonction `communication_scope` : 63.
- Fonction `create` : 373.
- Fonction `show` : 485.
- Fonction `post` : 524.
- Fonction `read` : 677.
- Test `spec138_thread_reason_absence_is_legacy_but_null_is_invalid` : 853.

#### D — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec102_threads_test.rs

- Test `spec138_unknown_legacy_thread_returns_durable_warning` : 107.
- Test `spec138_mixed_thread_checks_all_readers_even_with_unknown` : 130.
- Test `spec138_accepted_thread_replay_precedes_changed_project_guard_after_restart` : 191.
- Test `spec138_invalid_reason_never_creates_a_thread` : 228.
- Test `spec136_history_silent_and_exact` : 312.
- Test `spec136_structured_history_survives_daemon_restart` : 566.

#### E — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/execution_store.rs

- Fonction `reserve_control_command` : 1073.
- Fonction `mark_control_dispatched` : 1144.
- Fonction `lookup_control_command` : 1189.
- Fonction `init_schema` : 1530.

#### F — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/execution_store_test.rs

- Test `spec_138_control_warnings_new_store_has_empty_default` : 14.
- Test `spec_138_control_warnings_migrate_legacy_commands_idempotently` : 61.
- Test `spec_138_control_warnings_survive_restart_and_outcomes_without_rewriting_canon` : 112.

#### G — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/tests/spec138_project_test.rs

- Test `spec138_real_daemon_steer_scope_and_durable_replay` : 101.

#### H — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/mcp.rs

- Fonction `execute_send` : 664.
- Fonction `execute_who` : 1031.
- Fonction `project_reason` : 1821.
- Test `spec138_who_inherits_project_and_defaults_to_local_directory` : 4654.
- Test `spec138_valid_send_reason_reaches_transport_without_rewriting_body` : 4712.
- Test `spec138_invalid_reasons_and_scope_are_rejected_before_transport` : 4735.
- Test `spec138_old_server_without_capability_receives_no_message` : 4769.
- Test `spec138_send_returns_structured_warning_outside_exact_body` : 4829.
- Test `spec138_real_daemon_mcp_directory_and_cross_project_send_share_guard` : 4922.
- Test `spec138_real_agent_loop_cli_daemon_scoped_replay` : 4928.

#### I — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/cli.rs

- Fonction `parse_thread_args` : 1096.
- Fonction `cmd_send` : 2775.
- Fonction `send_idempotent_to_daemon_at_with_project` : 3628.
- Fonction `cmd_agents` : 5028.
- Fonction `emit_project_warnings` : 5126.
- Fonction `cmd_who` : 5476.
- Test `spec138_thread_accepts_explicit_reason_outside_exact_body` : 1545.
- Test `spec138_directory_accepts_explicit_global_and_project_root` : 6638.

#### J — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/communication/client.rs

- Fonction `negotiate_projects` : 29.
- Fonction `announce_client_project` : 52.
- Fonction `directory_request` : 69.

#### K — /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/scripts/agent_loop.py

- Fonction `update_task` : 628.
- Fonction `archive_previous_result` : 924.
- Fonction `write_task_result` : 989.
- Fonction `bridget_agents` : 1035.
- Fonction `bridget_agents_by_id` : 1067.
- Fonction `resolve_bridget_target` : 1081.
- Fonction `bridget_scope_context` : 1128.
- Fonction `send_bridget_message` : 1141.
- Fonction `cmd_attach_agent` : 1403.
- Fonction `prepare_bridget_dispatch` : 1990.
- Fonction `reserve_bridget_dispatch` : 2035.
- Fonction `dispatch_existing_bridget` : 2079.
- Fonction `cmd_dispatch` : 2120.
- Fonction `notify_mission_events` : 3552.

#### L — /Volumes/8TB2/01-workflow/git-worktrees/dotfiles/138-agent-loop-project/codex/.codex/skills/agent-loop/tests/test_agent_loop.py

- Test `test_directory_local_empty_never_falls_back_to_global` : 54.
- Test `test_legacy_unknown_directory_does_not_recruit_from_process_cwd` : 63.
- Test `test_attach_outside_requires_targeted_reason_without_writing` : 68.
- Test `test_root_configuration_alone_cannot_wake_outside_project` : 86.
- Test `test_local_busy_suggestion_is_retained_without_global_retry` : 116.
- Test `test_three_explicit_mandates_preserve_all_reminders` : 125.
- Test `test_two_mandates_same_recipient_remain_distinct_across_ticks` : 144.
- Test `test_outbox_freezes_root_reason_and_target_before_crash` : 167.
- Test `test_mandate_cannot_follow_changed_target_or_role` : 214.
- Test `test_dispatch_retries_exact_envelope_after_transport_failure` : 239.
- Test `test_two_concurrent_dispatch_reservations_send_only_once` : 309.
- Test `test_result_published_after_reservation_before_old_archive_stays_canonical` : 334.
- Test `test_archive_and_result_publication_share_the_same_real_flock` : 360.
- Test `test_archive_event_io_failure_occurs_only_after_durable_task_commit` : 405.
- Test `test_archive_task_write_failure_rolls_back_exact_result_and_task` : 423.
- Test `test_publication_task_write_failure_preserves_old_result_or_absence` : 444.
- Test `test_publication_archive_rename_failure_never_deletes_existing_result` : 464.
- Test `test_publication_result_write_failure_restores_previous_exact_document` : 478.
- Test `test_only_structured_project_warning_is_returned_to_loop_caller` : 498.

#### M — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code_contract.rs

- Fonction `parse_snapshot` : 271.
- Test `spec138_snapshot_uses_project_id_not_worktree_or_title` : 749.
- Test `spec138_snapshot_missing_or_ambiguous_project_stays_unknown` : 771.

#### N — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/wrapper.rs

- Fonction `announce_wrapper_project` : 1637.
- Test `spec138_owner_announces_git_cwd_on_each_registration_without_consuming_frames` : 6841.

#### O — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/t3code.rs

- Fonction `report_project_context` : 1532.
- Test `spec138_t3_owner_announces_unknown_project_instead_of_using_adapter_cwd` : 4181.
- Test `spec138_t3_reannounces_only_changed_project_facts_and_keeps_verdict_out_of_conversation` : 4210.

#### P — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/idempotency/send_delivery.rs

- Fonction `correlated_project_reason` : 28.

#### Q — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/store/threads.rs

- Fonction `thread_operation_replay` : 812.
- Fonction `thread_post` : 908.

#### R — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/handoff.rs

- Fonction `parse_request` : 548.
- Test `spec138_handoff_accepts_structured_reason_without_body_rewrite` : 905.

#### S — /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/crates/bridget-daemon/src/attach.rs

- Fonction `read` : 50.
- Test `spec138_journal_accepts_structured_reason_for_forwarding` : 5909.

### Exigences fonctionnelles

| Exigence | Fonctions exactes via catalogue | Tests exacts via catalogue | Assertion déterminante |
|---|---|---|---|
| FR-13801 | B:10927/11515 ; K:1035 | B:9734 ; H:4654 ; I:6638 ; L:54/63/116 | Local par défaut, inconnu vide, absence/busy sans fallback global |
| FR-13802 | B:10927/11515 ; J:69 ; I:5028/5476 ; H:1031 | B:9734 ; H:4654/4922 ; I:6638 | Global volontaire et relations distinctes, sans mandat implicite |
| FR-13803 | A:66 ; B:7956 ; M:271 ; N:1637 ; O:1532 | A:262/301/353 ; B:9800 ; M:749/771 ; N:6841 ; O:4210 | Git common-dir/hôte, worktree/symlink, homonymes, identité attestée plutôt que titre/domaine |
| FR-13804 | A:25/36 ; B:7956 ; K:1035 | A:228 ; B:9800 ; D:107 ; L:63 ; M:771 ; O:4181 | Absence/conflit restent unknown ; envoi legacy averti, pas de suggestion locale |
| FR-13805 | A:12/36 ; B:8148/8669 ; C:373/524 | B:9734/10052 ; D:130/228 ; G:101 ; H:4922 | Sans motif : refus avant dépôt, ACK, injection et push fournisseur |
| FR-13806 | A:36 ; B:8627 ; E:1144 ; I:5126 ; K:1141 | A:228 ; D:107 ; H:4829 ; G:101 ; F:112 ; L:498 | Warning prévalidé hors corps, résultat caller et reprise durable ; pas de second dialogue |
| FR-13807 | B:8148 ; P:28 ; K:1128/3552 | B:9888 ; L:125/144/214 | OPEN inverse authentique, UUID/rôle exacts ; mandat distinct par mission |
| FR-13808 | C:63/373/524 | D:130/228 | Tous les lecteurs A/B/U, notify[] non privé ; refus sans ACK partiel ni dépôt |
| FR-13809 | K:1035/1403/1990/2120 | L:54/63/68/116 ; H:4928 | Suggestions locales, pas de recrutement extérieur implicite ; attach contrôlé |
| FR-13810 | K:1128/3552 | L:86/125/144/214 | ROOT sans mandat : décision ; trois rôles mandatés : rappels conservés |
| FR-13811 | C:485/524/677 ; Q:812 | D:130/191/312/566 | Lecture/contenu exact, history silencieux, aucune notification supplémentaire |
| FR-13812 | A:184 ; Q:812 ; E:1073/1144/1189 ; B:8669 ; K:628/924/989/2035/2079 | A:404 ; D:107/191 ; F:61/112 ; G:101 ; B:9888 ; L:167/239/309/334/360/405/423/444/464/478 | Canon et reçu exacts ; source/motif figés ; CAS unique ; résultat canonique protégé aux pannes E/S ordinaires |
| FR-13813 | A:36 ; B:8148/8669 ; J:29 ; H:664/1031 ; I:2775/5476 ; R:548 ; S:50 ; K:1141 | H:4712/4735/4769/4922/4928 ; I:1545 ; R:905 ; S:5909 ; G:101 | Même garde CLI/MCP/daemon/fils/Loop ; ancien serveur refuse avant enveloppe à motif |
| FR-13814 | B:7956 ; E:1530 ; Q:908 ; K:628/2035/2079 | F:14/61/112 ; G:101 ; H:4928 ; D:191 ; L:360 | Store et harnais existants, namespace privé ; absence de commit/install/restart vérifiée par preuves opératoires |

La règle Bridget est dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/skills/bridget/SKILL.md:31.
Son warning commence :72, contrôle :78, capacité :95 et mandats :99.
Elle a été rapprochée des APIs et validée par le principal. Une validation du
frontmatter ne prouve pas seule un comportement ; les recettes réelles H:4922/4928
et G:101 complètent cette preuve.

### Critères de succès

| Critère | Fonctions via catalogue | Tests exacts via catalogue | Mesure ou borne |
|---|---|---|---|
| SC-13801 | B:10927/11515 ; A:25 ; K:1035 | B:9734 ; H:4654/4922 ; I:6638 ; L:54/63 | Les UUID du local/global/unknown sont comparés, aucun unknown classé local |
| SC-13802 | A:36 ; B:8148/8669 ; C:373/524 | B:9734/10052 ; D:130/228 ; G:101 ; H:4922/4928 | Aucun dépôt/push/ACK partiel sur les refus connus testés |
| SC-13803 | A:36 ; B:8627 ; E:1144 ; I:5126 ; K:1141 | A:228 ; D:107 ; H:4829/4928 ; G:101 ; F:112 ; L:498 | Warning avant effet, caller hors corps ; résultat stable après reprise |
| SC-13804 | K:1035/1128/1403/1990/2120/3552 | L:54/63/68/86/116/125/144/214 | Zéro envoi sans mandat ; chacun des trois rappels explicitement mandatés vérifié |
| SC-13805 | A:66 ; B:7956 ; M:271 ; N:1637 ; O:1532 | A:262/301/353 ; B:9800 ; M:749/771 ; N:6841 ; O:4210 | Dépôt/worktree/symlink réels et deux dépôts de même basename distincts ; plus de gap homonyme |
| SC-13806 | A:184 ; B:8148/8669 ; C:677 ; Q:812 ; E:1073/1189 ; K:628/989/2035/2079 | H:4922/4928 ; D:191/312/566 ; F:61/112 ; G:101 ; L:167/239/360/423/444/464/478 | Matrice réelle façades/Loop et reprises ; zéro livraison supplémentaire, bornes E/S explicites |

### Preuves après gel et bornes

Le vrai test homonyme A:353 a été exécuté : un PASS, zéro FAIL, aucun ignoré.
Capture relue :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/core-homonyms-green.log.
Il ferme la borne signalée dans la matrice provisoire. Les tests canon et null
sont maintenant A:404 et A:416, pas les anciens repères.

Le principal confirme les 152 tests Python après extraction, soit 124 existants
et 28 nouveaux. Le rapport propriétaire et ses annotations de coût sont dans
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop.md.
La recette réelle avait un PASS avant extraction ; son nouveau passage est
en cours au moment de cet état. Aucun résultat futur n'est anticipé.

T020 couvre les pannes OSError ordinaires dans existing_bridget initial et
write_task_result. Les writers sans verrou, crash machine entre remplacements,
volume disparu, rollback lui-même défaillant et backends legacy non convertis
ne sont pas présentés comme couverts. T019 respecte l'Interrupt sans message,
le canon sans motif et les anciennes lignes à warnings[].

### Lecture documentaire contradictoire puis seconde lecture

La première lecture a retenu les incohérences suivantes : texte T019 encore
ouvert malgré sa case fermée ; checklist Draft périmée ; zéro colonne non borné
dans la recherche ; états anciens du journal lisibles comme état courant.
Corrections : clôture T019 sur preuves réelles, checklist In Progress, zéro
colonne de consentement de fil avec exception T019, récapitulatif append-only
du journal. Les résultats historiques restent conservés.

La contre-lecture a aussi demandé un vrai test homonyme, maintenant PASS, et
la borne de complexité du dispatch. L'extraction réutilise les trois responsabilités
concrètes : préparation, réservation transactionnelle, remise. Elle ne crée pas
de moteur de mission, de framework ni de verrou parallèle.

Seconde lecture : les 14 FR et six SC possèdent des fonctions et tests nommés.
Les repères sont ceux du gel confirmé, T019/T020 inclus. Couverture documentaire
PASS ; aucun CRITICAL documentaire identifié. Ceci n'est pas l'Analyze final
du principal ni un PASS global d'exécution. T017/T018 restent ouvertes, état18/20
In Progress. Le champ de fiche Tests0/0 reste un compteur initial à recalibrer
par le principal ; il ne décrit pas l'absence de tests ciblés exécutés.
Aucun Converge ni audit n'a été exécuté dans cette lecture.

## État documentaire gelé avant la phase suivante

Le principal a relu la matrice14FR/6SC, les sources modifiées et leurs tests.
Aucun gap fonctionnel n'est confirmé après cette lecture. Le test réel homonyme
et l'extraction Loop ont reçu APPROVE en revue indépendante du même fournisseur.
La revue legacy089 concurrence/skill et celle des synchronisations des tests
transport ont aussi reçu APPROVE. Les oracles sont conservés ; aucun résultat
attendu n'a été supprimé pour masquer un échec. Ces revues ne sont pas des
revues inter-fournisseurs ni une preuve de réussite globale du workspace.

Comptage unique138 confirmé par le principal : 35 tests Rust et 28 tests Python,
soit63 tests propres à la fonction. Ce nombre ne somme pas les passages répétés
du même filtre. Les152 tests Python comprennent124 anciens et28 nouveaux.

| Contrôle après extraction | Résultat relu | Capture |
|---|---|---|
| Python complet | 152 tests, OK, en1,068s ; PASS confirmé par le principal | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/python-after-extraction-final.log |
| Agent Loop/CLI/daemon réel opt-in | 1 PASS, 0 FAIL, 0 ignoré, en1,55s | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/loop-real-after-extraction.log |
| Build release | Finished release profile optimized ; PASS confirmé par le principal | /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/release-final.log |

WorkspaceV4 était encore en cours au signal initial du principal. À la relecture,
la capture ci-dessous se termine par error:1 target failed, managed_parity_test.
Le test ancien matrice_fr008_compare_le_meme_corpus_et_les_frames_attach donne
6 PASS/1 FAIL dans ce target. Le principal relève une relance légitime au
cinquième tour ; la suite du diagnostic et les adaptations d'anciens tests de
présence globale seront consignées par lui, sans prédiction de réussite.
Capture observée :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/workspace-final-v4.log.
Aucun PASS global n'est déclaré sur ce run. T017/T018 restent ouvertes.

Le scan adverse sécurité/fiabilité/minimalisme n'a prouvé aucun défaut de
production supplémentaire sur son périmètre relu. Ce constat ne ferme pas
l'échec managed_parity ni ne prouve l'absence universelle de défaut. Les limites
T020 restent celles des pannes E/S ordinaires, du verrou commun et des chemins
initialexisting_bridget/write_task_result. Crash machine, panne du rollback,
writers externes sans verrou et backends legacy non convertis restent hors borne.

Les repères SKILL72/78/95/99 ont été vérifiés par recherche dans la source.
État documentaire gelé pour cette lecture :18/20, In Progress ; aucune case
T017/T018 modifiée. Aucun verdict Converge ou audit, ni résultat futur, n'est écrit.

## Analyze final après validation V5 et audit — 2026-10-06

État actuel : Implemented,20/20. Transition constatée : les snapshots18/20 et V4 rouge ci-dessus restent historiques ; le principal a clôturé T017 sur V5 (19/20), puis a autorisé T018 après l'audit validé (20/20). Aucun résultat historique n'est réécrit.

Le principal a relu la matrice14FR/6SC, ses111 repères, les sources et tests gelés. Aucun gap fonctionnel confirmé. T019 et T020 inclus. Première lecture contradictoire puis seconde lecture : les erreurs de recette socket/filtre et de découverte188→177 ont été corrigées ; aucun CRITICAL résiduel. La qualité88 donne A- et non A ; le global pondéré98,333 donne A.

WorkspaceV5 relu :1633 PASS/0 FAIL/55 ignored, sur78 résumés externes filtered=0. Les trois résumés internes ne sont pas ajoutés. Python152 PASS=124+28. Les63 tests138 uniques=35Rust+28Python sont un sous-ensemble, pas1633+152+63. Recette opt-in réelle après dernière source :1 PASS, pas un test unique supplémentaire. fmt/clippy -Dwarnings/release après fixtures exit0. Skills Bridget et Agent Loop validées. BDD29 scénarios écrits, pas exécutés comme Gherkin.

La fixture managed_parity finale (+106/−10) conserve strictement les quatre IDs métier. Les rappels système sont séparés uniquement si leur ID, demande et génération sont attestés dans guichet_coordination_events, avec corps exact et journal complet. Pas de filtre libre par texte ou nom. La relance de production reste active. Revue indépendante APPROVE puis V5 PASS ; V4 rouge conservé.

Converge pass1 manuel : début18:52:52UTC, fin18:56:19UTC,207s ; verdict CONVERGED. SHA tasks avant/après identique9947694758ffadb912aae5dc096b34397fe5390b1a68e6220e5f15526aac77a0. Cette empreinte porte sur l'état19/20 avant la clôture T018 ; elle ne prétend pas couvrir les écritures documentaires ultérieures. Aucun Converge2 revendiqué.

Audit v14 du diff : A98,333,0C/H,5MED ouverts non bloquants. Cycles1/scoring validés par le script canonique :exit0,0erreur,0warning, puis revalidation après correction du compteur177. Scope54 fichiers utiles,35 sources ; 100% des hunks+contexte+source neuve, pas100% du dépôt. Performance/UX N/A,module08 non actif. Phase9 corrections non engagée : WIP isolé/baseline historique rouge, aucun candidatC/H ; aucun faux dépôt propre. Baseline absente non créée. Le principal a effectué une seconde lecture indépendante des rapports et validé le même résultat. Capture :
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/2026-10-06/session-2026-10-06-spec-138-01/scoring.md
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/2026-10-06/session-2026-10-06-spec-138-01/grade.json
/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/audits/2026-10-06/session-2026-10-06-spec-138-01/validation.log

Revue adverse finale après Converge : APPROVE borné, même fournisseur. Aucun cas exigé oublié confirmé. Preuves : fil A/B/U et notify[] (spec102_threads_test.rs:130), retry figé après mutation de mandat/source/corps/cible (Looptests:167/239), ROOT sans mandat zéro envoi et trois rôles mandatés rappelés (Looptests:86/125). Bornes T020 conservées : writers participants sous verrou, pannes E/S ordinaires ; pas crashmachine, rollback défaillant ou anciens backends non convertis. UNKNOWN legacy averti reste possible ; garde de contexte, pas frontière anti-malveillance.

Analyze final documentaire PASS dans ce périmètre. Aucune écriture de code/test, aucun commit, fusion, push, installation ou déploiement dans cette clôture. Toutes les primitives natives absentes ont été remplacées par les protocoles manuels annoncés ; aucun faux lancement d'outil.

Scope documentaire final :56 fichiers utiles,35 sources,16 auxiliaires neufs. Le snapshot54 ci-dessus précédait convergence-report.md et validation/results.json, tous deux ajoutés et relus. Compteur contexte177 horsspecs/preuves. Revalidation officielle après ces ajouts :exit0,0erreur,0warning. Écritures clôturées le2026-10-06T19:03:33Z ; pass2 laissé au principal, aucun résultat futur déclaré.
