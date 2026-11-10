    use super::*;

    #[test]
    fn native160_execution_reconstruction_non_qualifiee_refusee() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission160");
        store
            .admit_starting_message(&message, "quota-parent", 100)
            .unwrap();
        store
            .transition_if_current(
                "quota-parent",
                "starting",
                0,
                1,
                "failed",
                "provider_failed",
                105,
            )
            .unwrap();
        assert_eq!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:1:200",
                    "instance",
                    "quota-child",
                    240
                )
                .unwrap(),
            QuotaRecoveryOutcome::NotQualified
        );
        assert!(store.execution_snapshot("quota-child").unwrap().is_none());
        assert_eq!(
            store
                .execution_snapshot("quota-parent")
                .unwrap()
                .unwrap()
                .state,
            "failed"
        );
    }

    #[test]
    fn native160_execution_reconstruction_parent_non_feuille_refuse() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission160");
        store
            .admit_starting_message(&message, "quota-parent", 100)
            .unwrap();
        assert!(matches!(
            store
                .reconstruct_active_for_agent("child", "instance", "ordinary-child", 105)
                .unwrap(),
            ExecutionRecoveryOutcome::Reconstructed(_)
        ));
        let parent = store.execution_snapshot("quota-parent").unwrap().unwrap();
        store
            .transition_if_current(
                &parent.execution_id,
                &parent.state,
                parent.revision,
                parent.generation,
                "failed",
                "late_failure",
                108,
            )
            .unwrap();
        store
            .qualify_quota_failure("quota-parent", "limit-resume:task:1:200", 200, 110)
            .unwrap();
        assert_eq!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:1:200",
                    "instance",
                    "quota-child",
                    240
                )
                .unwrap(),
            QuotaRecoveryOutcome::NotLeaf
        );
        assert!(store.execution_snapshot("quota-child").unwrap().is_none());
        assert_eq!(
            store
                .latest_in_chain("quota-parent")
                .unwrap()
                .unwrap()
                .execution_id,
            "ordinary-child"
        );
    }

    #[test]
    fn native160_execution_reconstruction_deux_identites_refusees() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission160");
        store
            .admit_starting_message(&message, "quota-parent", 100)
            .unwrap();
        store
            .transition_if_current(
                "quota-parent",
                "starting",
                0,
                1,
                "failed",
                "provider_failed",
                105,
            )
            .unwrap();
        store
            .qualify_quota_failure("quota-parent", "limit-resume:task:1:200", 200, 110)
            .unwrap();
        assert_eq!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:2:300",
                    "instance",
                    "wrong-child",
                    240
                )
                .unwrap(),
            QuotaRecoveryOutcome::NotQualified
        );
        assert!(matches!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:1:200",
                    "instance",
                    "quota-child",
                    240
                )
                .unwrap(),
            QuotaRecoveryOutcome::Reconstructed(_)
        ));
        assert_eq!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:2:300",
                    "instance",
                    "second-child",
                    241
                )
                .unwrap(),
            QuotaRecoveryOutcome::NotQualified
        );
        assert!(store.execution_snapshot("wrong-child").unwrap().is_none());
        assert!(store.execution_snapshot("second-child").unwrap().is_none());
        assert_eq!(store.conn.query_row("SELECT COUNT(*) FROM execution_continuations WHERE parent_execution_id='quota-parent'", [],
            |row| row.get::<_, u32>(0)).unwrap(), 1);
    }

    #[test]
    fn native160_execution_reconstruction_transaction_atomique() {
        for trigger in [
            "BEFORE INSERT ON executions",
            "BEFORE INSERT ON execution_continuations",
            "BEFORE UPDATE OF resume_execution_id ON execution_quota_resumes",
        ] {
            let store = ExecutionStore::open_in_memory().unwrap();
            let message = bridget_core::BridgetMessage::new("parent", "child", "mission160");
            store
                .admit_starting_message(&message, "quota-parent", 100)
                .unwrap();
            store
                .transition_if_current(
                    "quota-parent",
                    "starting",
                    0,
                    1,
                    "failed",
                    "provider_failed",
                    105,
                )
                .unwrap();
            store
                .qualify_quota_failure("quota-parent", "limit-resume:task:1:200", 200, 110)
                .unwrap();
            let parent = store.execution_snapshot("quota-parent").unwrap();
            store.conn.execute_batch(&format!("CREATE TEMP TRIGGER quota160_abort {trigger} BEGIN SELECT RAISE(ABORT,'quota reconstruction rollback'); END;")).unwrap();
            assert!(
                matches!(
                    store.reconstruct_quota_failed(
                        "quota-parent",
                        "limit-resume:task:1:200",
                        "instance",
                        "quota-child",
                        240
                    ),
                    Err(rusqlite::Error::SqliteFailure(_, _))
                ),
                "{trigger}"
            );
            assert!(
                store.execution_snapshot("quota-child").unwrap().is_none(),
                "{trigger}"
            );
            assert!(
                store.continuation_for("quota-child").unwrap().is_none(),
                "{trigger}"
            );
            assert_eq!(
                store.quota_failure_for("quota-parent").unwrap().unwrap().3,
                None,
                "{trigger}"
            );
            assert_eq!(
                store.execution_snapshot("quota-parent").unwrap(),
                parent,
                "{trigger}"
            );
        }
    }

    #[test]
    fn native160_execution_reconstruction_rejeu() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission160");
        let project = ProjectReference {
            project_id: "project160".into(),
            binding_generation: 3,
        };
        store
            .admit_starting_message_for_project(&message, "quota-parent", Some(&project), 100)
            .unwrap();
        store
            .transition_if_current(
                "quota-parent",
                "starting",
                0,
                1,
                "failed",
                "provider_failed",
                105,
            )
            .unwrap();
        let parent = store.execution_snapshot("quota-parent").unwrap();
        // T004 : le chemin ordinaire continue d'exclure failed ; aucun changement de contrat.
        assert_eq!(
            store
                .reconstruct_active_for_agent("child", "instance", "ordinary-child", 120)
                .unwrap(),
            ExecutionRecoveryOutcome::None
        );
        store
            .qualify_quota_failure("quota-parent", "limit-resume:task:1:200", 200, 110)
            .unwrap();
        let QuotaRecoveryOutcome::Reconstructed(recovery) = store
            .reconstruct_quota_failed(
                "quota-parent",
                "limit-resume:task:1:200",
                "instance",
                "quota-child",
                240,
            )
            .unwrap()
        else {
            panic!("qualification et parent failed feuille doivent permettre la reprise");
        };
        assert_eq!(recovery.parent_execution_id, "quota-parent");
        assert_eq!(recovery.message, message);
        assert_eq!(recovery.snapshot.generation, 2);
        assert_eq!(recovery.snapshot.project, Some(project));
        assert_eq!(recovery.snapshot.created_at, 240);
        let first = store.continuation_for("quota-child").unwrap().unwrap();
        assert_eq!(first.mode, ContinuationMode::Reconstructed);
        assert_eq!(first.reason.as_deref(), Some("usage_limit_reset"));
        assert_eq!(first.observed_at, 240);
        assert_eq!(
            store
                .reconstruct_quota_failed(
                    "quota-parent",
                    "limit-resume:task:1:200",
                    "new-instance",
                    "discarded-child",
                    999
                )
                .unwrap(),
            QuotaRecoveryOutcome::AlreadyResumed("quota-child".into())
        );
        assert!(
            store
                .execution_snapshot("discarded-child")
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .continuation_for("quota-child")
                .unwrap()
                .unwrap()
                .observed_at,
            240
        );
        assert_eq!(store.execution_snapshot("quota-parent").unwrap(), parent);
        assert_eq!(
            store.execution_snapshot("quota-child").unwrap(),
            Some(recovery.snapshot)
        );
        assert_eq!(
            store.execution_message("quota-child").unwrap(),
            Some(message)
        );
        assert_eq!(
            store
                .quota_failure_for("quota-parent")
                .unwrap()
                .unwrap()
                .3
                .as_deref(),
            Some("quota-child")
        );
    }

    #[test]
    fn native160_execution_quota_creation() {
        let store = ExecutionStore::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), 12);
        store
            .record_starting("quota-submission", "quota-execution", "child", 100)
            .unwrap();
        assert!(matches!(
            store
                .transition_if_current(
                    "quota-execution",
                    "starting",
                    0,
                    1,
                    "failed",
                    "provider_failed",
                    105
                )
                .unwrap(),
            ConditionalTransition::Applied(_)
        ));
        let row = ("limit-resume:task:1:200".to_string(), 200, 110, None);
        assert_eq!(
            store
                .qualify_quota_failure("quota-execution", &row.0, 200, 110)
                .unwrap(),
            Some(row.clone())
        );
        assert_eq!(
            store.quota_failure_for("quota-execution").unwrap(),
            Some(row)
        );
        assert_eq!(
            store
                .execution_snapshot("quota-execution")
                .unwrap()
                .unwrap()
                .state,
            "failed"
        );
    }

    #[test]
    fn native160_execution_quota_rejeu() {
        let store = ExecutionStore::open_in_memory().unwrap();
        store
            .record_starting("quota-submission", "quota-execution", "child", 100)
            .unwrap();
        store
            .transition_if_current(
                "quota-execution",
                "starting",
                0,
                1,
                "failed",
                "provider_failed",
                105,
            )
            .unwrap();
        let initial = store
            .qualify_quota_failure("quota-execution", "limit-resume:task:1:200", 200, 110)
            .unwrap();
        assert_eq!(
            initial,
            Some(("limit-resume:task:1:200".to_string(), 200, 110, None))
        );
        assert_eq!(
            store
                .qualify_quota_failure("quota-execution", "limit-resume:task:1:200", 999, 120)
                .unwrap(),
            initial
        );
        assert_eq!(store.quota_failure_for("quota-execution").unwrap(), initial);
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM execution_quota_resumes", [], |row| {
                    row.get::<_, u32>(0)
                })
                .unwrap(),
            1
        );
    }

    #[test]
    fn native160_execution_quota_non_failed_refusee() {
        let store = ExecutionStore::open_in_memory().unwrap();
        store
            .record_starting("quota-submission", "quota-execution", "child", 100)
            .unwrap();
        assert_eq!(
            store
                .qualify_quota_failure("quota-execution", "limit-resume:task:1:200", 200, 110)
                .unwrap(),
            None
        );
        assert_eq!(store.quota_failure_for("quota-execution").unwrap(), None);
        assert_eq!(
            store
                .execution_snapshot("quota-execution")
                .unwrap()
                .unwrap()
                .state,
            "starting"
        );
    }

    #[test]
    fn native160_execution_quota_identite_double_refusee() {
        let store = ExecutionStore::open_in_memory().unwrap();
        for execution in ["quota-first", "quota-second"] {
            store
                .record_starting(execution, execution, "child", 100)
                .unwrap();
            store
                .transition_if_current(
                    execution,
                    "starting",
                    0,
                    1,
                    "failed",
                    "provider_failed",
                    105,
                )
                .unwrap();
        }
        let original = store
            .qualify_quota_failure("quota-first", "limit-resume:task:1:200", 200, 110)
            .unwrap();
        assert!(original.is_some());
        assert_eq!(
            store
                .qualify_quota_failure("quota-first", "limit-resume:task:2:300", 300, 120)
                .unwrap(),
            None
        );
        assert_eq!(
            store
                .qualify_quota_failure("quota-second", "limit-resume:task:1:200", 200, 120)
                .unwrap(),
            None
        );
        assert_eq!(store.quota_failure_for("quota-first").unwrap(), original);
        assert_eq!(store.quota_failure_for("quota-second").unwrap(), None);
        assert_eq!(
            store
                .conn
                .query_row("SELECT COUNT(*) FROM execution_quota_resumes", [], |row| {
                    row.get::<_, u32>(0)
                })
                .unwrap(),
            1
        );
    }

    #[test]
    fn native152_chaine_generation_et_reprises_de_pause() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission");
        let root = "z-root";
        store.admit_starting_message(&message, root, 100).unwrap();
        assert_eq!(
            store.latest_in_chain(root).unwrap(),
            store.execution_snapshot(root).unwrap()
        );
        for (index, child) in ["y-recovery", "x-recovery", "w-recovery", "v-recovery"]
            .into_iter()
            .enumerate()
        {
            let previous = store.latest_in_chain(root).unwrap().unwrap();
            let paused = index % 2 == 0;
            if paused {
                store
                    .record_pause_interruption(&previous.execution_id, "child", 1, 100)
                    .unwrap();
                assert!(matches!(
                    store
                        .transition_if_current(
                            &previous.execution_id,
                            &previous.state,
                            previous.revision,
                            previous.generation,
                            "interrupted",
                            "control_pause",
                            100,
                        )
                        .unwrap(),
                    ConditionalTransition::Applied(_)
                ));
            }
            let ExecutionRecoveryOutcome::Reconstructed(recovery) = store
                .reconstruct_active_for_agent("child", "instance", child, 100)
                .unwrap()
            else {
                panic!("la continuation doit être reconstruite");
            };
            assert_eq!(recovery.message, message);
            assert_eq!(recovery.parent_execution_id, previous.execution_id);
            assert_eq!(recovery.snapshot.generation, index as u64 + 2);
            assert_eq!(recovery.snapshot.created_at, 100);
            assert_eq!(
                store.latest_in_chain(root).unwrap(),
                Some(recovery.snapshot.clone())
            );
            let continuation = store.continuation_for(child).unwrap().unwrap();
            assert_eq!(continuation.parent_execution_id, previous.execution_id);
            if paused {
                let resumed: (i64, String) = store
                    .conn
                    .query_row(
                        "SELECT resumed_at,resume_execution_id FROM control_pause_interruptions WHERE execution_id=?1",
                        [&previous.execution_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .unwrap();
                assert_eq!(resumed, (100, child.to_string()));
                assert_eq!(continuation.execution_id, resumed.1);
            }
        }
    }

    #[test]
    fn native152_chaine_annulation_consomme_la_pause_sans_reprise() {
        let store = ExecutionStore::open_in_memory().unwrap();
        let message = bridget_core::BridgetMessage::new("parent", "child", "mission");
        store.admit_starting_message(&message, "root", 100).unwrap();
        store
            .record_pause_interruption("root", "child", 1, 110)
            .unwrap();
        store
            .transition_if_current(
                "root",
                "starting",
                0,
                1,
                "interrupted",
                "control_pause",
                110,
            )
            .unwrap();
        assert!(store.abandon_pause_interruption("root", 120).unwrap());
        assert!(!store.abandon_pause_interruption("root", 130).unwrap());
        let saved: (i64, Option<String>) = store
            .conn
            .query_row(
                "SELECT resumed_at,resume_execution_id FROM control_pause_interruptions WHERE execution_id='root'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(saved, (120, None));
        assert!(store.pending_pause_interruptions().unwrap().is_empty());
        assert!(
            store
                .recoverable_execution_ids_for_agent("child")
                .unwrap()
                .is_empty()
        );
        assert!(store.continuation_for("root").unwrap().is_none());
        assert_eq!(
            store.execution_snapshot("root").unwrap().unwrap().state,
            "interrupted"
        );
    }

    #[test]
    fn spec_087_une_remise_de_focus_passe_devant_le_travail_ordinaire() {
        let mut ordinary = bridget_core::BridgetMessage::new("a", "b", "x");
        ordinary.references = vec!["project:p@1".to_string()];
        assert_eq!(ExecutionStore::queue_priority_for(&ordinary, 0), 0);
        assert_eq!(ExecutionStore::queue_priority_for(&ordinary, 7), 7);
        let mut focus = bridget_core::BridgetMessage::new("a", "b", "x");
        focus.references = vec!["project:p@1".to_string(), "focus:o1".to_string()];
        assert_eq!(
            ExecutionStore::queue_priority_for(&focus, 0),
            FOCUS_QUEUE_PRIORITY
        );
        assert_eq!(
            ExecutionStore::queue_priority_for(&focus, FOCUS_QUEUE_PRIORITY + 1),
            FOCUS_QUEUE_PRIORITY + 1
        );
    }
