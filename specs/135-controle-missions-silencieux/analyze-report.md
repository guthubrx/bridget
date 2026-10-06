# Analyze 135 — 2026-10-05

Statut : PASS après corrections ciblées. Aucun CRITICAL ouvert.

La primitive a été tentée. Le script attendu est absent dans ce dépôt :
`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/135-controle-missions-silencieux/.specify/scripts/bash/check-prerequisites.sh`.
La comparaison spec/plan/tasks/code/tests est donc exécutée manuellement.
Ce remplacement n'est pas présenté comme une exécution du script officiel.

| Exigence | Réalisation dans agent_loop.py | Preuve dans test_agent_loop.py | Tâches |
|---|---|---|---|
| FR-13501 | cmd_add_task:515 | test_add_task_records_owner_due_date_and_expected_result:1159 | T003–004 |
| FR-13502 | cmd_ack:1392, running_task_alerts | test_ack_threshold_then_silence_then_coordinator_then_root:1302 | T005–006 |
| FR-13503 | cmd_progress:1439, running_task_alerts | test_progress_threshold_299_and_300:1320 | T005–006 |
| FR-13504 | cmd_progress, update_task:620 | test_progress_requires_a_typed_reference:1184, test_progress_command_rejects_nonconsecutive_result_replay | T003–004 |
| FR-13505 | advance_issue_state, heartbeat_tick | test_failed_transport_escalates_to_coordinator_in_same_tick:1352, test_missing_all_recipients_writes_one_durable_decision:1361 | T005–006 |
| FR-13506 | issue_key, outbox et reçus acceptés | test_two_subprocess_heartbeats_read_persisted_state_without_duplicate, test_outbox_freezes_replay_after_crash_following_delivery:1478 | T005–006 |
| FR-13507 | mission_digest, groupement par destinataire | test_two_issues_for_same_recipient_produce_one_private_digest:1330, test_same_agent_as_worker_and_coordinator_receives_one_digest | T005–006 |
| FR-13508 | cmd_disposition, cmd_resolve_task | test_disposition_preserves_terminal_verdict:1233, test_terminal_verdict_cannot_be_changed_by_resolve_task:1457 | T007–008 |
| FR-13509 | ready_tasks, paused_work_kinds | test_execution_pause_leaves_code_ready:1403 | T005–006 |
| FR-13510 | open_run_alerts, run_open_obligations | test_migration_preserves_history_and_is_idempotent:1413, test_close_run_refuses_open_work_then_closes_clean_run:1253 | T007–008 |
| FR-13511 | cmd_close_run, run_open_obligations | test_close_run_refuses_open_work_then_closes_clean_run:1253, test_closed_run_skips_refresh_collection_and_delivery | T007–008 |
| FR-13512 | cmd_migrate_run, update_task(touch=False) | test_migration_preserves_history_and_is_idempotent:1413, test_migration_snapshot_does_not_overwrite_concurrent_result | T003–004, T011 |
| FR-13513 | sessions existing_bridget, script officiel | tests de transport Bridget, T011 réel après sauvegarde | T009–011 |
| FR-13514 | mission_digest, empreintes et enums fermés | test_two_issues_for_same_recipient_produce_one_private_digest:1330 | T005–006 |

Chemins des sources :
/Users/moi/.codex/skills/agent-loop/scripts/agent_loop.py
/Users/moi/.codex/skills/agent-loop/tests/test_agent_loop.py

Corrections vérifiées : destinataire injoignable, clé event+task_id, preuve de
pause indépendante, nom réel last_progress_ref, perte de résultat sur lecture
périmée, anti-rejeu A/B/A, sélection de rôle sans écriture en dry-run.

Relecture finale : couverture 14/14 exigences. Aucun besoin supprimé pour
obtenir cette couverture. Une dépendance héritée du scheduler reste explicite :
la résolution des dépendances ajoute leur nombre au coût des parcours. Aucun
chiffre de couverture de lignes n'est revendiqué : coverage n'est pas installé.
