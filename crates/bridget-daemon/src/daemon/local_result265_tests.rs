use super::*;

#[test]
fn spec265_native_timeout_closes_tracking_after_cleanup_without_reopening_the_task() {
    let (state, config, mut task, _parent, _child) = native160_mission(false);
    task.state = "failed".into();
    task.error = Some("mission_reply_timeout".into());
    task.cleanup_done = true;
    task.guarded = false;
    task.guard_released = true;
    task.failure_sent = true;
    task.acknowledged_at = Some(unix_timestamp());
    state.lock().unwrap().delegation_store.save(&task).unwrap();
    let before = serde_json::to_value(
        state
            .lock()
            .unwrap()
            .delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    {
        let st = state.lock().unwrap();
        assert!(st.pending_replies.iter().any(|p| p.msg_id == task.mission));
        st.store.connection().execute_batch("CREATE TRIGGER timeout265_fail BEFORE UPDATE OF state ON tracked_requests WHEN NEW.state='timed_out' BEGIN SELECT RAISE(ABORT,'test265'); END;").unwrap();
    }
    tick(&state);
    {
        let st = state.lock().unwrap();
        assert_eq!(
            st.store.get_request(&task.mission).unwrap().unwrap().state,
            "open"
        );
        assert!(st.pending_replies.iter().any(|p| p.msg_id == task.mission));
        st.store
            .connection()
            .execute_batch("DROP TRIGGER timeout265_fail;")
            .unwrap();
    }
    for _ in 0..2 {
        tick(&state);
    }
    let st = state.lock().unwrap();
    assert_eq!(
        st.store.get_request(&task.mission).unwrap().unwrap().state,
        "timed_out"
    );
    assert!(!st.pending_replies.iter().any(|p| p.msg_id == task.mission));
    assert_eq!(
        serde_json::to_value(st.delegation_store.get(&task.task_id).unwrap().unwrap()).unwrap(),
        before
    );
    drop(st);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_other_failure_and_foreign_tracking_are_not_reclassified_as_timeout() {
    let (state, config, mut task, _parent, _child) = native160_mission(false);
    let mut st = state.lock().unwrap();
    task.state = "failed".into();
    task.error = Some("provider_failed".into());
    assert!(!settle_terminal_tracked_request247(&mut st, &task).unwrap());
    task.error = Some("mission_reply_timeout".into());
    task.owner = "foreign-owner".into();
    assert!(!settle_terminal_tracked_request247(&mut st, &task).unwrap());
    task.owner = st.store.get_request(&task.mission).unwrap().unwrap().sender;
    task.child = "foreign-child".into();
    assert!(!settle_terminal_tracked_request247(&mut st, &task).unwrap());
    assert_eq!(
        st.store.get_request(&task.mission).unwrap().unwrap().state,
        "open"
    );
    drop(st);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_terminal_without_result_refuses_ack_and_preserves_the_outbox() {
    let (state, config, mut task, _parent, _child) = native160_mission(false);
    let root = config.db_path.parent().unwrap().join("late265");
    let outbox =
        crate::reply_outbox::ReplyOutbox::open(&root, task.child_instance.as_deref().unwrap())
            .unwrap();
    let mut original = bridget_core::BridgetMessage::new(&task.owner, &task.child, "mission");
    original.id = task.mission.clone();
    let mut reply =
        bridget_core::BridgetMessage::new(&task.child, &task.owner, "finale tardive conservée");
    reply.in_reply_to = Some(task.mission.clone());
    let reply = outbox.retain(&original, reply, unix_timestamp()).unwrap();
    for terminal in ["failed", "cancelled", "cancelling"] {
        task.state = terminal.into();
        state.lock().unwrap().delegation_store.save(&task).unwrap();
        let response =
            handle_wrapper_message("child-conn", WrapperToDaemon::Send(reply.clone()), &state);
        assert!(
            matches!(response, Some(DaemonToWrapper::Nack { reason, .. }) if reason.starts_with("retained_reply_native_terminal:"))
        );
        let saved = state
            .lock()
            .unwrap()
            .delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap();
        assert_eq!(saved.state, terminal);
        assert!(saved.result.is_none());
        assert_eq!(
            outbox.pending(unix_timestamp()).unwrap().unwrap().body,
            reply.body
        );
    }
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_closed_tracking_does_not_hold_a_pending_reminder_forever() {
    let (state, config, mut task, _parent, _child) = native160_mission(false);
    let mut st = state.lock().unwrap();
    task.result = Some("rapport".into());
    st.delegation_store.save(&task).unwrap();
    st.store
        .connection()
        .execute(
            "UPDATE tracked_requests SET state='cancelled' WHERE id=?1",
            [&task.mission],
        )
        .unwrap();
    let now = Instant::now();
    st.pending_replies
        .iter_mut()
        .find(|p| p.msg_id == task.mission)
        .unwrap()
        .created_at = now - Duration::from_secs(1000);
    super::super::collect_reminder_actions(&mut st, now);
    assert!(!st.pending_replies.iter().any(|p| p.msg_id == task.mission));
    assert_eq!(
        st.store.get_request(&task.mission).unwrap().unwrap().state,
        "cancelled"
    );
    drop(st);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_remote_and_foreign_replies_still_require_their_authority() {
    let (state, config, mut task, _parent, _child) = native160_mission(false);
    let st = state.lock().unwrap();
    let instance = task.child_instance.as_deref().unwrap();
    let mut reply = bridget_core::BridgetMessage::new(&task.child, &task.owner, "rapport");
    reply.in_reply_to = Some(task.mission.clone());
    assert_eq!(
        super::super::authorize_wrapper_reply(&st, "conn-1", &reply, instance),
        Err("wrapper_reply_identity_required".into())
    );
    task.remote_placement = Some(bridget_transport::protocol::RemotePlacement {
        machine_id: "worker-host".into(),
        worker_id: "worker".into(),
        provisioning_revision: 1,
        project_id: "project".into(),
        mapping_revision: 1,
        origin_host: st.host.clone(),
        origin_cwd: task.cwd.clone(),
        source_root_identity: "git:test265".into(),
        relative_cwd: ".".into(),
        execution_cwd: task.cwd.clone(),
        base_commit: None,
        definition_digest: "a".repeat(64),
        permission_digest: "b".repeat(64),
    });
    st.delegation_store.save(&task).unwrap();
    assert_eq!(
        super::super::authorize_wrapper_reply(&st, "child-conn", &reply, instance),
        Err("remote_result_requires_worker".into())
    );
    assert!(
        st.delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .result
            .is_none()
    );
    drop(st);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_local_result_settles_tracking_after_a_sql_failure_without_new_work() {
    let (state, config, task, _parent, _child) = native160_mission(false);
    let mut st = state.lock().unwrap();
    st.store.connection().execute_batch("CREATE TRIGGER local265_answer_fail BEFORE UPDATE OF state ON tracked_requests WHEN NEW.state='answered' BEGIN SELECT RAISE(ABORT,'test265'); END;").unwrap();
    let mut reply = bridget_core::BridgetMessage::new(&task.child, &task.owner, "rapport durable");
    reply.in_reply_to = Some(task.mission.clone());
    assert!(capture_reply(&mut st, "child-conn", &reply).unwrap());
    assert!(result_suppresses_reminders(&st, &task.mission));
    assert_eq!(
        st.delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .result
            .as_deref(),
        Some("rapport durable")
    );
    assert_eq!(
        st.store.get_request(&task.mission).unwrap().unwrap().state,
        "open"
    );
    st.store
        .connection()
        .execute_batch("DROP TRIGGER local265_answer_fail;")
        .unwrap();
    drop(st);
    tick(&state);
    let st = state.lock().unwrap();
    assert_eq!(
        st.store.get_request(&task.mission).unwrap().unwrap().state,
        "answered"
    );
    assert!(!st.pending_replies.iter().any(|p| p.msg_id == task.mission));
    assert_eq!(
        st.delegation_store
            .get(&task.task_id)
            .unwrap()
            .unwrap()
            .mission,
        task.mission
    );
    drop(st);
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_local_retained_final_is_captured_once_before_ack() {
    let (state, config, task, _parent, _child) = native160_mission(false);
    let root = config.db_path.parent().unwrap().join("outbox265");
    let outbox =
        crate::reply_outbox::ReplyOutbox::open(&root, task.child_instance.as_deref().unwrap())
            .unwrap();
    let mut original = bridget_core::BridgetMessage::new(&task.owner, &task.child, "mission");
    original.id = task.mission.clone();
    original.reply = true;
    let mut reply = bridget_core::BridgetMessage::new(&task.child, &task.owner, "rapport complet");
    reply.in_reply_to = Some(task.mission.clone());
    let reply = outbox.retain(&original, reply, unix_timestamp()).unwrap();
    for _ in 0..2 {
        let response =
            handle_wrapper_message("child-conn", WrapperToDaemon::Send(reply.clone()), &state);
        assert!(
            matches!(response, Some(DaemonToWrapper::Ack { .. })),
            "{response:?}"
        );
        let st = state.lock().unwrap();
        let saved = st.delegation_store.get(&task.task_id).unwrap().unwrap();
        assert_eq!(saved.result.as_deref(), Some("rapport complet"));
        assert_eq!(saved.state, "waiting_for_children");
        assert_eq!(
            st.store.get_request(&task.mission).unwrap().unwrap().state,
            "answered"
        );
        assert!(!st.pending_replies.iter().any(|p| p.msg_id == task.mission));
    }
    drop(state);
    cleanup(&config);
}

#[test]
fn spec265_busy_idle_cycles_do_not_repeat_an_already_sent_reminder() {
    let (state, config, task, _parent, _child) = native160_mission(false);
    let mut st = state.lock().unwrap();
    let now = Instant::now();
    let pending = st
        .pending_replies
        .iter_mut()
        .find(|p| p.msg_id == task.mission)
        .unwrap();
    pending.created_at = now - Duration::from_secs(50);
    pending.escalation_level = 1;
    let instance = task.child_instance.as_ref().unwrap();
    for _ in 0..3 {
        st.presences.get_mut(instance).unwrap().state = "busy".into();
        let busy = super::super::collect_reminder_actions(&mut st, now);
        assert!(!busy.iter().any(|a| matches!(
            a,
            super::super::ReminderAction::Gentle { .. } | super::super::ReminderAction::Firm { .. }
        )));
        st.presences.get_mut(instance).unwrap().state = "idle".into();
        let idle = super::super::collect_reminder_actions(&mut st, now);
        assert!(
            !idle.iter().any(|a| matches!(
                a,
                super::super::ReminderAction::Gentle { .. }
                    | super::super::ReminderAction::Firm { .. }
            )),
            "un niveau déjà envoyé ne doit pas être réarmé"
        );
    }
    // Le palier suivant reste autorisé, une fois, après le tour actif.
    let later = now + Duration::from_secs(40);
    st.presences.get_mut(instance).unwrap().state = "busy".into();
    super::super::collect_reminder_actions(&mut st, later);
    st.presences.get_mut(instance).unwrap().state = "idle".into();
    let actions = super::super::collect_reminder_actions(&mut st, later);
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, super::super::ReminderAction::Firm { .. }))
            .count(),
        1
    );
    drop(st);
    drop(state);
    cleanup(&config);
}
