# L6 — avis durable indépendant de pending_replies
Lecture ciblée 2026-10-12T20:42:32.787956+00:00, sans code ni exécution.

failed_for_owner lit uniquement le payloadTask : owner,state=failed,failure_sent=0,cleanup_done=1 (LIMIT16) : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/delegation.rs:962. Les échecs nonnettoyés viennent de pending à949 ; aucune de ces requêtes ne consulte pending_replies.

tick charge failed_for_owner pour les propriétaires présents : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon/native_delegation.rs:3432. La branche failed à3543 construit l'avis depuis Task.owner/task_id/error, lui donne id=native-failure-task et originSystem, puis send_as avec task.created_at (canonstable). L'ACK/phase de cet avis se lit sur cette clé propre à4476, sans suivi de mission.

send_as/with_result_authority à2732–2787 établit l'identité interne et appelle handle_idempotent_send_with_result_authority. La remise durable et son writer différé sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/160-reprise-quota/crates/bridget-daemon/src/daemon.rs:12049–12238 et6020–6075. ReplyTracking concerne seulement le message remis ; remember_reply_cycle à11652 est conditionnel àmessage.reply. Ce chemin ne requiert pas un pending_replies d'origine pour native-failure-task.

**Conclusion : retirer le suivi de mission àT015 ne retire ni la Task ni son avis idempotent.** Dépendances véritables : Taskfailed persistée, failure_sentfalse, propriétaire vivant et attesté, workspace_report_ready, accèsDB et remise. Le parent absent reporte l'avis ; la reconnexion le rend sélectionnable. L'absence d'ACK ne prouve pas une livraison ; preuve distincte via send_delivery_phase. La fonction deliver_native_content n'existe pas sous ce nom dans N/D actuels ; le chemin réel cité ci-dessus est le réemploi.

M4ROOT reçu : finir l'intentTask avec sa qualification déjàpersistée et la même identity ; qualificationorpheline sansTaskQuotaWait ne reconstruit pas. Stockage indisponible : checkpoint et erreur propagée, jamais succès inventé. Aucun nouvel état/API/test/scénario proposé. Aucun WT/Git/Cargo/DB/service exécuté.
