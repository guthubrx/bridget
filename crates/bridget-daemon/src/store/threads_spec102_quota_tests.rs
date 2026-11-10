    use super::*;
    use crate::store::Store;

    const A: &str = "10200000-0000-4000-8000-00000000000a";
    const B: &str = "10200000-0000-4000-8000-00000000000b";

    fn store() -> (Store, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("spec102-quota-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).unwrap();
        (Store::open(&root.join("db.sqlite")).unwrap(), root)
    }

    #[test]
    fn spec147_members_schema_accepts_action_without_touching_old_receipts() {
        let (store, root) = store();
        let id = "14700000-0000-4000-8000-000000000001";
        create(&store, &crate::threads::LIMITS, id);
        let original: Vec<String> = store
            .conn
            .prepare("SELECT result_json FROM thread_operations ORDER BY operation_id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        store.conn.execute("INSERT INTO thread_operations(actor_id,operation_id,action,thread_id,canonical_hash,result_json,created_at) VALUES(?1,'test','add_members',?2,'h','{}',3)", params![A,id]).unwrap();
        ensure_schema(&store.conn).unwrap();
        let preserved: Vec<String> = store.conn.prepare("SELECT result_json FROM thread_operations WHERE operation_id!='test' ORDER BY operation_id").unwrap().query_map([], |r| r.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(preserved, original);
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT MAX(version) FROM thread_schema_migrations",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            3
        );
        drop(store);
        let _ = std::fs::remove_dir_all(root);
    }

    fn legacy_members_schema(store: &Store, version: i64, invalid: bool) {
        let check = if invalid {
            ""
        } else {
            "CHECK(action IN ('create','post','close'))"
        };
        store.conn.execute_batch(&format!("BEGIN IMMEDIATE;
            ALTER TABLE thread_operations RENAME TO old_operations;
            CREATE TABLE thread_operations(actor_id TEXT NOT NULL,operation_id TEXT NOT NULL,action TEXT NOT NULL {check},thread_id TEXT NOT NULL,canonical_hash TEXT NOT NULL,result_json TEXT NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(actor_id,operation_id));
            INSERT INTO thread_operations SELECT * FROM old_operations;
            DROP TABLE old_operations;
            CREATE INDEX idx_thread_operations_thread ON thread_operations(thread_id);
            DELETE FROM thread_schema_migrations;
            INSERT INTO thread_schema_migrations VALUES({version},1);
            COMMIT;")).unwrap();
        if version == 1 {
            store.conn.execute_batch("DROP INDEX idx_discussion_entries_supersedes; ALTER TABLE discussion_entries DROP COLUMN supersedes_seq; ALTER TABLE discussion_entries DROP COLUMN kind;").unwrap();
        }
    }

    #[test]
    fn spec147_members_v1_v2_migration_preserves_pages_receipts_operations_and_indexes() {
        for version in [1, 2] {
            let (store, root) = store();
            let id = "14700000-0000-4000-8000-000000000001";
            create(&store, &crate::threads::LIMITS, id);
            post(
                &store,
                &crate::threads::LIMITS,
                id,
                "CANARY147 exact é\r\n\n",
            );
            let read = ReadRequest {
                actor: B,
                thread_id: id,
                limit: 1,
                byte_budget: 60000,
                receipt_id: "14700000-0000-4000-8000-000000000002",
                receipt_ttl_secs: 600,
                now: 2,
            };
            let page = store.thread_read(&read).unwrap();
            let operations=store.conn.prepare("SELECT actor_id,operation_id,action,thread_id,canonical_hash,result_json,created_at FROM thread_operations ORDER BY operation_id").unwrap().query_map([],|r|Ok(format!("{:?}",(r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,i64>(6)?)))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
            legacy_members_schema(&store, version, false);
            drop(store);
            for _ in 0..2 {
                let migrated = Store::open(&root.join("db.sqlite")).unwrap();
                assert_eq!(
                    migrated.thread_read(&read).unwrap(),
                    match &page {
                        ReadOutcome::Page {
                            page,
                            receipt,
                            expires_at,
                            requested_limit,
                            ..
                        } => ReadOutcome::Page {
                            page: page.clone(),
                            receipt: receipt.clone(),
                            expires_at: *expires_at,
                            requested_limit: *requested_limit,
                            replayed: true
                        },
                        _ => panic!("page attendue"),
                    }
                );
                let actual=migrated.conn.prepare("SELECT actor_id,operation_id,action,thread_id,canonical_hash,result_json,created_at FROM thread_operations ORDER BY operation_id").unwrap().query_map([],|r|Ok(format!("{:?}",(r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,r.get::<_,i64>(6)?)))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
                assert_eq!(actual, operations);
                assert!(schema_ready(&migrated.conn).unwrap());
                assert_eq!(migrated.conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name='idx_thread_operations_thread'",[],|r|r.get::<_,i64>(0)).unwrap(),1);
                assert_eq!(
                    migrated
                        .conn
                        .query_row(
                            "SELECT MAX(version) FROM thread_schema_migrations",
                            [],
                            |r| r.get::<_, i64>(0)
                        )
                        .unwrap(),
                    3
                );
            }
            let _ = std::fs::remove_dir_all(root);
        }
    }

    #[test]
    fn spec147_members_migration_failure_rolls_back_the_only_journal() {
        let (store, root) = store();
        legacy_members_schema(&store, 2, true);
        store.conn.execute("INSERT INTO thread_operations VALUES('actor','bad','invalid','fil','hash','exact',1)",[]).unwrap();
        assert!(ensure_schema(&store.conn).is_err());
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT result_json FROM thread_operations WHERE operation_id='bad'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            "exact"
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE name='thread_operations_v3'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            store
                .conn
                .query_row(
                    "SELECT MAX(version) FROM thread_schema_migrations",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
        drop(store);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec147_members_no_change_does_not_bind_key_and_stale_union_refuses() {
        let (store, root) = store();
        let id = "14700000-0000-4000-8000-000000000001";
        create(&store, &crate::threads::LIMITS, id);
        let c = "14700000-0000-4000-8000-000000000003";
        let op = "14700000-0000-4000-8000-000000000002";
        let old = vec![A.into(), B.into()];
        let request = AddMembers {
            actor: A,
            operation_id: op,
            canonical_hash: "noop",
            thread_id: id,
            candidates: &old,
            expected_members: &old,
            project_warnings: &[],
            now: 2,
        };
        let changes = store.conn.total_changes();
        assert!(matches!(
            store.thread_add_members(request).unwrap(),
            ThreadTxOutcome::NoChange(_)
        ));
        assert_eq!(store.conn.total_changes(), changes);
        assert!(
            store
                .thread_operation_replay(A, op, "anything")
                .unwrap()
                .is_none()
        );
        let mut union = old.clone();
        union.push(c.into());
        union.sort();
        let request = AddMembers {
            actor: A,
            operation_id: op,
            canonical_hash: "add",
            thread_id: id,
            candidates: &[c.into()],
            expected_members: &old,
            project_warnings: &[],
            now: 2,
        };
        assert_eq!(
            store.thread_add_members(request.clone()).unwrap(),
            ThreadTxOutcome::Refused(ThreadRefusal::AudienceChanged)
        );
        assert_eq!(store.conn.total_changes(), changes);
        assert!(matches!(
            store
                .thread_add_members(AddMembers {
                    expected_members: &union,
                    ..request
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
        let changes = store.conn.total_changes();
        let d = "14700000-0000-4000-8000-000000000004";
        let mut stale = old;
        stale.push(d.into());
        stale.sort();
        assert_eq!(
            store
                .thread_add_members(AddMembers {
                    actor: A,
                    operation_id: "14700000-0000-4000-8000-000000000005",
                    canonical_hash: "stale",
                    thread_id: id,
                    candidates: &[d.into()],
                    expected_members: &stale,
                    project_warnings: &[],
                    now: 3
                })
                .unwrap(),
            ThreadTxOutcome::Refused(ThreadRefusal::AudienceChanged)
        );
        assert_eq!(store.conn.total_changes(), changes);
        drop(store);
        let _ = std::fs::remove_dir_all(root);
    }

    fn create(store: &Store, limits: &ThreadLimits, thread_id: &str) {
        let members = vec![A.to_string(), B.to_string()];
        let outcome = store
            .thread_create(CreateThread {
                project_warnings: &[],
                actor: A,
                operation_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                canonical_hash: "h",
                thread_id,
                title: "Quota",
                members: &members,
                now: 1,
                limits,
            })
            .unwrap();
        assert!(matches!(outcome, ThreadTxOutcome::Done(_)), "{outcome:?}");
    }

    fn post(store: &Store, limits: &ThreadLimits, thread_id: &str, body: &str) -> ThreadTxOutcome {
        store
            .thread_post(PostEntry {
                project_warnings: &[],
                actor: A,
                operation_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                canonical_hash: &format!("h-{}", uuid::Uuid::new_v4().simple()),
                thread_id,
                message_id: &uuid::Uuid::new_v4().hyphenated().to_string(),
                body,
                notify: &NotifySpec::None,
                notices: &[],
                reply_to_seq: None,
                ack_receipt: None,
                kind: None,
                supersedes_seq: None,
                now: 2,
                limits,
            })
            .unwrap()
    }

    #[test]
    fn spec146_red_recent_list_is_global_not_uuid_order() {
        let (store, root) = store();
        let old = "14600000-0000-4000-8000-000000000001";
        let recent = "14600000-0000-4000-8000-000000000002";
        create(&store, &crate::threads::LIMITS, old);
        create(&store, &crate::threads::LIMITS, recent);
        post(&store, &crate::threads::LIMITS, recent, "dernier échange");
        let before = store.conn.total_changes();
        let (page, _) = store.thread_list_recent(B, 1, None).unwrap();
        assert_eq!(page[0].summary.thread_id, recent);
        assert_eq!(store.conn.total_changes(), before);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec146_red_history_opens_on_true_last_message() {
        let (store, root) = store();
        let id = "14600000-0000-4000-8000-000000000003";
        create(&store, &crate::threads::LIMITS, id);
        for n in 1..=7 {
            post(&store, &crate::threads::LIMITS, id, &format!("message {n}"));
        }
        let before = store.conn.total_changes();
        let HistoryOutcome::Page(page) = store
            .thread_history_recent(B, id, None, None, 2, 60000)
            .unwrap()
        else {
            panic!("page attendue");
        };
        assert_eq!(page.entries[0]["seq"], 7);
        assert_eq!(store.conn.total_changes(), before);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec146_recent_list_cursor_ties_empty_closed_and_last_sequence_date() {
        let (store, root) = store();
        let plan = store
            .conn
            .prepare(&format!("EXPLAIN QUERY PLAN {RECENT_LIST_SQL}"))
            .unwrap()
            .query_map(
                params![B, Option::<i64>::None, Option::<String>::None, 2],
                |r| r.get::<_, String>(3),
            )
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            plan.iter()
                .any(|line| line.contains("idx_discussion_members_agent")),
            "{plan:?}"
        );
        assert!(
            plan.iter()
                .any(|line| line.starts_with("SEARCH e USING INDEX")
                    && line.contains("(thread_id=? AND seq=?)")),
            "{plan:?}"
        );
        eprintln!("SPEC146 EXPLAIN {plan:?}");
        let ids = (0..4)
            .map(|n| format!("14600000-0000-4000-8000-{n:012}"))
            .collect::<Vec<_>>();
        for id in &ids {
            create(&store, &crate::threads::LIMITS, id);
        }
        post(
            &store,
            &crate::threads::LIMITS,
            &ids[0],
            "first timestamp is not the last sequence",
        );
        post(&store, &crate::threads::LIMITS, &ids[0], "latest sequence");
        store.conn.execute("UPDATE discussion_entries SET created_at=CASE seq WHEN 1 THEN 999 ELSE 50 END WHERE thread_id=?1",[&ids[0]]).unwrap();
        for id in &ids[1..3] {
            post(&store, &crate::threads::LIMITS, id, "equal activity");
            store
                .conn
                .execute(
                    "UPDATE discussion_entries SET created_at=70 WHERE thread_id=?1",
                    [id],
                )
                .unwrap();
        }
        store
            .conn
            .execute(
                "UPDATE discussion_threads SET created_at=60,closed_at=90 WHERE thread_id=?1",
                [&ids[3]],
            )
            .unwrap();
        let before = store.conn.total_changes();
        let mut seen = Vec::new();
        let mut cursor = None;
        loop {
            let parsed = cursor
                .as_deref()
                .map(crate::threads::parse_recent_cursor)
                .transpose()
                .unwrap();
            let (page, next) = store
                .thread_list_recent(B, 1, parsed.as_ref().map(|(date, id)| (*date, id.as_str())))
                .unwrap();
            seen.extend(
                page.into_iter()
                    .map(|t| (t.summary.thread_id, t.last_activity_at)),
            );
            cursor = next;
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(
            seen,
            vec![
                (ids[1].clone(), 70),
                (ids[2].clone(), 70),
                (ids[3].clone(), 60),
                (ids[0].clone(), 50)
            ]
        );
        assert!(
            store
                .thread_list_recent("outsider", 100, None)
                .unwrap()
                .0
                .is_empty()
        );
        assert_eq!(store.conn.total_changes(), before);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec146_recent_history_byte_cursor_global_correction_snapshot_and_gaps() {
        let (store, root) = store();
        let id = "14600000-0000-4000-8000-000000000004";
        create(&store, &crate::threads::LIMITS, id);
        for n in 1..=7 {
            post(
                &store,
                &crate::threads::LIMITS,
                id,
                &format!("{n} é\r\n  {}", "🦀".repeat(200)),
            );
        }
        store.conn.execute("UPDATE discussion_entries SET kind='action',supersedes_seq=1 WHERE thread_id=?1 AND seq=7",[id]).unwrap();
        let before = store.conn.total_changes();
        let HistoryOutcome::Page(first) = store
            .thread_history_recent(B, id, None, None, 200, 1200)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(first.entries.len(), 1);
        assert_eq!(first.entries[0]["seq"], 7);
        assert!(first.has_more);
        assert_eq!(first.through_seq, 7);
        assert_eq!(first.snapshot_seq, 7);
        let HistoryOutcome::Page(old) = store
            .thread_history_recent(B, id, Some(1), Some(7), 200, 60000)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(old.entries[0]["superseded_by_seq"], 7);
        assert!(
            old.entries[0]["body"]
                .as_str()
                .unwrap()
                .starts_with("1 é\r\n  ")
        );
        let HistoryOutcome::Page(past) = store
            .thread_history_recent(B, id, Some(1), Some(6), 200, 60000)
            .unwrap()
        else {
            panic!()
        };
        assert!(past.entries[0].get("superseded_by_seq").is_none());
        let HistoryOutcome::Page(empty) = store
            .thread_history_recent(B, id, Some(0), Some(7), 200, 60000)
            .unwrap()
        else {
            panic!()
        };
        assert!(empty.entries.is_empty());
        assert!(!empty.has_more);
        let HistoryOutcome::Page(clamped) = store
            .thread_history_recent(B, id, Some(100), Some(7), 1, 60000)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(clamped.through_seq, 7);
        assert!(matches!(
            store
                .thread_history_recent("outsider", id, None, None, 20, 60000)
                .unwrap(),
            HistoryOutcome::Refused(ThreadRefusal::ThreadUnavailable)
        ));
        assert!(matches!(
            store
                .thread_history_recent(B, id, None, Some(8), 20, 60000)
                .unwrap(),
            HistoryOutcome::Refused(ThreadRefusal::RangeUnavailable)
        ));
        assert_eq!(store.conn.total_changes(), before);
        // Données anormales trouées : la continuation est une existence, pas min>1.
        store
            .conn
            .execute(
                "DELETE FROM discussion_entries WHERE thread_id=?1 AND seq<7",
                [id],
            )
            .unwrap();
        let HistoryOutcome::Page(gap) = store
            .thread_history_recent(B, id, None, None, 1, 60000)
            .unwrap()
        else {
            panic!()
        };
        assert!(!gap.has_more);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec136_v1_schema_migration_preserves_legacy_and_is_repeatable() {
        let (store, root) = store();
        let id = "13600000-0000-4000-8000-000000000001";
        create(&store, &crate::threads::LIMITS, id);
        post(&store, &crate::threads::LIMITS, id, "legacy exact\né");
        let read_request = ReadRequest {
            actor: B,
            thread_id: id,
            limit: 200,
            byte_budget: 60000,
            receipt_id: "13600000-0000-4000-8000-000000000002",
            receipt_ttl_secs: 600,
            now: 2,
        };
        let pending_read = store.thread_read(&read_request).unwrap();
        let operations: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM thread_operations", [], |r| r.get(0))
            .unwrap();
        // Reproduire le schémaV1 sur disque : mêmes tables/lignes, sans les
        // deux colonnes/indexV2. Aucun processus ou base de production.
        store
            .conn
            .execute_batch(
                "DROP INDEX idx_discussion_entries_supersedes;
            ALTER TABLE discussion_entries DROP COLUMN supersedes_seq;
            ALTER TABLE discussion_entries DROP COLUMN kind;
            DELETE FROM thread_schema_migrations WHERE version>1;
            INSERT OR IGNORE INTO thread_schema_migrations VALUES(1,1);",
            )
            .unwrap();
        drop(store);
        let migrated = Store::open(&root.join("db.sqlite")).unwrap();
        let first = migrated.thread_history(B, id, 1, None, 200, 60000).unwrap();
        assert_eq!(
            migrated
                .conn
                .query_row("SELECT COUNT(*) FROM thread_operations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            operations
        );
        let replay = migrated.thread_read(&read_request).unwrap();
        assert!(
            matches!((&pending_read,&replay), (ReadOutcome::Page { page:p,receipt:r,.. },ReadOutcome::Page { page:p2,receipt:r2,.. }) if p == p2 && r == r2)
        );
        assert!(
            matches!(&first, HistoryOutcome::Page(p) if p.entries[0]["body"] == "legacy exact\né" && p.entries[0].get("kind").is_none())
        );
        assert_eq!(
            migrated
                .thread_show(B, id)
                .unwrap()
                .unwrap()
                .thread
                .body_bytes,
            "legacy exact\né".len() as u64
        );
        drop(migrated);
        let reopened = Store::open(&root.join("db.sqlite")).unwrap();
        assert_eq!(
            reopened.thread_history(B, id, 1, None, 200, 60000).unwrap(),
            first
        );
        let version: i64 = reopened
            .conn
            .query_row(
                "SELECT MAX(version) FROM thread_schema_migrations",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, THREAD_SCHEMA_VERSION);
        let plan: Vec<String> = reopened.conn.prepare("EXPLAIN QUERY PLAN SELECT s.seq FROM discussion_entries e LEFT JOIN discussion_entries s ON s.thread_id=e.thread_id AND s.supersedes_seq=e.seq AND s.supersedes_seq IS NOT NULL AND s.seq<=10 WHERE e.thread_id=?1 AND e.seq>=1 AND e.seq<=10 ORDER BY e.seq LIMIT 200").unwrap().query_map([id], |r| r.get(3)).unwrap().collect::<Result<_,_>>().unwrap();
        assert!(
            plan.iter()
                .any(|line| line.contains("idx_discussion_entries_supersedes")),
            "{plan:?}"
        );
    }

    fn entries(store: &Store, thread_id: &str) -> u64 {
        store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM discussion_entries WHERE thread_id = ?1",
                [thread_id],
                |row| row.get::<_, i64>(0),
            )
            .unwrap() as u64
    }

    #[test]
    fn spec102_v31_plafonds_d_entrees_et_d_octets_n_puis_n_plus_1() {
        let (store, root) = store();
        let limits = ThreadLimits {
            max_threads: 256,
            max_open_per_creator: 32,
            max_entries_per_thread: 3,
            max_body_bytes_per_thread: 10,
            max_body_bytes_total: 15,
        };
        let t1 = "11111111-1111-4111-8111-111111111111";
        let t2 = "22222222-2222-4222-8222-222222222222";
        create(&store, &limits, t1);
        create(&store, &limits, t2);
        // Octets par fil : 4 + 4 acceptés (8), le troisième dépôt de 4 octets dépasse 10.
        assert!(matches!(
            post(&store, &limits, t1, "abcd"),
            ThreadTxOutcome::Done(_)
        ));
        assert!(matches!(
            post(&store, &limits, t1, "efgh"),
            ThreadTxOutcome::Done(_)
        ));
        assert_eq!(
            post(&store, &limits, t1, "ijkl"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_per_thread"))
        );
        assert!(
            matches!(post(&store, &limits, t1, "mn"), ThreadTxOutcome::Done(_)),
            "10 octets exactement acceptés"
        );
        assert_eq!(entries(&store, t1), 3);
        // Entrées par fil : la quatrième est refusée même pour un octet.
        assert_eq!(
            post(&store, &limits, t1, "o"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("entries_per_thread"))
        );
        assert_eq!(entries(&store, t1), 3, "aucune mutation au refus");
        // Total tous fils : 10 déjà consommés, 5 restent ; 6 octets refusés, 5 acceptés.
        assert_eq!(
            post(&store, &limits, t2, "pqrstu"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_total"))
        );
        assert!(matches!(
            post(&store, &limits, t2, "vwxyz"),
            ThreadTxOutcome::Done(_)
        ));
        assert_eq!(
            post(&store, &limits, t2, "!"),
            ThreadTxOutcome::Refused(ThreadRefusal::CapacityExceeded("body_bytes_total"))
        );
        let total: i64 = store
            .conn
            .query_row(
                "SELECT SUM(body_bytes) FROM discussion_threads",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(total, 15, "compteurs cohérents avec les entrées");
        let _ = std::fs::remove_dir_all(root);
    }
