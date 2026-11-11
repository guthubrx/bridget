use super::*;

const UUID_A: &str = "14700000-aaaa-4aaa-8aaa-00000000000a";
const UUID_B: &str = "14700000-bbbb-4bbb-8bbb-00000000000b";
const UUID_T: &str = "14700000-cccc-4ccc-8ccc-00000000000c";
const UUID_OP: &str = "14700000-dddd-4ddd-8ddd-00000000000d";

#[test]
fn tracking230_groupe_exclut_dormant_sans_accumuler_et_rejoue_identique() {
    let f = UuidFixture::new();
    let third = "23000000-0000-4000-8000-000000000003";
    f.store
        .connection()
        .execute(
            "INSERT INTO discussion_members(thread_id,agent_id) VALUES(?1,?2)",
            rusqlite::params![UUID_T, third],
        )
        .unwrap();
    let agent:bridget_transport::protocol::ScopedAgentInfo=serde_json::from_value(json!({"agent_id":UUID_B,"display_name":"Dormant","agent_type":"fixture","connection_id":"test","host":"test","transport":"fixture","state":"busy","last_seen_secs":0,"reconnect_count":0,"communication_project":null,"project_relation":"unknown"})).unwrap();
    let sleep = |value: bool| {
        crate::agent_tracking::set(
            f.store.connection(),
            "human",
            &uuid::Uuid::new_v4().to_string(),
            b"fixture",
            &agent,
            value,
            Some(false),
            None,
            10,
        )
        .unwrap()
    };
    sleep(true);
    struct NativeDirectory;
    impl Directory for NativeDirectory {
        fn known_agent(&self, _: &str) -> bool {
            true
        }
        fn member_facts(&self, _: &str) -> MemberFacts {
            MemberFacts::default()
        }
    }
    let request = ThreadRequest {
        version: 1,
        cross_project_reason: None,
        request: ThreadAction::Post {
            thread_id: UUID_T.into(),
            body: "Message conservé pour les lecteurs".into(),
            notify: ThreadNotify::All(bridget_transport::protocol::ThreadNotifyAll::All),
            operation_id: UUID_OP.into(),
            reply_to_seq: None,
            ack_receipt: None,
            kind: None,
            supersedes_seq: None,
        },
    };
    let first = handle_with_change(&f.store, &NativeDirectory, UUID_A, request.clone(), 20)
        .0
        .result;
    assert_eq!(first["status"], "posted", "{first}");
    assert!(
        first.to_string().contains("dormant_members_ignored"),
        "{first}"
    );
    let wakes = f.store.thread_wake_candidates(20, 20).unwrap();
    assert!(!wakes.iter().any(|w| w.agent_id == UUID_B));
    assert!(
        wakes.iter().any(|w| w.agent_id == third),
        "les autres membres sont sollicités"
    );
    sleep(false);
    let replay = handle_with_change(&f.store, &NativeDirectory, UUID_A, request, 21)
        .0
        .result;
    assert_eq!(replay, first);
    assert!(
        !f.store
            .thread_wake_candidates(21, 20)
            .unwrap()
            .iter()
            .any(|w| w.agent_id == UUID_B),
        "aucun rattrapage au réveil"
    );
    assert_eq!(
        f.store
            .connection()
            .query_row(
                "SELECT body FROM discussion_entries WHERE thread_id=?1",
                [UUID_T],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Message conservé pour les lecteurs"
    );
}

struct UuidDirectory;
impl Directory for UuidDirectory {
    fn known_agent(&self, id: &str) -> bool {
        [UUID_A, UUID_B].contains(&id)
    }
    fn member_facts(&self, _: &str) -> MemberFacts {
        MemberFacts::default()
    }
    fn communication_scope(
        &self,
        _: &str,
        _: &[String],
        _: Option<&str>,
    ) -> Result<Vec<ProjectWarning>, Value> {
        Ok(Vec::new())
    }
}

struct UuidFixture {
    store: Store,
    root: std::path::PathBuf,
}
impl UuidFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("uuid147-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        let store = Store::open(&root.join("db")).unwrap();
        assert!(matches!(
            store
                .thread_create(CreateThread {
                    actor: UUID_A,
                    operation_id: "14700000-eeee-4eee-8eee-00000000000e",
                    canonical_hash: "seed",
                    thread_id: UUID_T,
                    title: "Titre É exact",
                    members: &[UUID_A.into(), UUID_B.into()],
                    project_warnings: &[],
                    now: 1,
                    limits: &LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
        Self { store, root }
    }
    fn run(&self, actor: &str, request: ThreadAction) -> (Value, Option<String>) {
        let (result, changed) = handle_with_change(
            &self.store,
            &UuidDirectory,
            actor,
            ThreadRequest {
                version: 1,
                cross_project_reason: Some("Motif É inchangé".into()),
                request,
            },
            2,
        );
        (result.result, changed)
    }
    fn post(&self, upper: bool, notify: Vec<String>) -> (Value, Option<String>) {
        self.run(
            UUID_A,
            ThreadAction::Post {
                thread_id: if upper {
                    UUID_T.to_uppercase()
                } else {
                    UUID_T.into()
                },
                operation_id: if upper {
                    UUID_OP.to_uppercase()
                } else {
                    UUID_OP.into()
                },
                body: "É 🦀\r\n  Exact\n\n".into(),
                notify: ThreadNotify::Targets(notify),
                reply_to_seq: None,
                ack_receipt: None,
                kind: None,
                supersedes_seq: None,
            },
        )
    }
}
impl Drop for UuidFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn spec147_members_add_normalized_replay_no_change_and_cursor_zero() {
    let fixture = UuidFixture::new();
    struct Known;
    impl Directory for Known {
        fn known_agent(&self, _: &str) -> bool {
            true
        }
        fn member_facts(&self, _: &str) -> MemberFacts {
            MemberFacts::default()
        }
        fn communication_scope(
            &self,
            _: &str,
            _: &[String],
            _: Option<&str>,
        ) -> Result<Vec<ProjectWarning>, Value> {
            Ok(vec![])
        }
    }
    let c = "14700000-ffff-4fff-8fff-00000000000f";
    let run = |op: &str, members: Value| {
        let request = serde_json::from_value(json!({"version":1,"request":{"action":"add_members","thread_id":UUID_T.to_uppercase(),"members":members,"operation_id":op}})).unwrap();
        let (result, changed) = handle_with_change(&fixture.store, &Known, UUID_A, request, 9);
        (result.result, changed)
    };
    fixture.post(false, vec![]);
    let before = fixture.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
    let op = "14700000-eeee-4eee-8eee-00000000000f";
    let (added, changed) = run(&op.to_uppercase(), json!([c.to_uppercase(), c, UUID_B]));
    assert_eq!(added["status"], "members_added");
    assert_eq!(added["added_members"], json!([c]));
    assert_eq!(changed.as_deref(), Some(UUID_T));
    let (replayed, changed) = run(op, json!([UUID_B, c, c.to_uppercase()]));
    assert_eq!(replayed, added);
    assert!(changed.is_none());
    let (noop, changed) = run("14700000-eeee-4eee-8eee-000000000010", json!([c]));
    assert_eq!(noop["status"], "no_change");
    assert!(changed.is_none());
    let view = fixture.store.thread_show(c, UUID_T).unwrap().unwrap();
    assert_eq!(view.own.acked_seq, 0);
    assert_eq!(view.thread, before.thread);
    assert!(view.own_wake.is_none());
    let (bad, changed) = run(op, json!([UUID_B]));
    assert_eq!(bad["code"], "envelope_mismatch");
    assert!(changed.is_none());
}

#[test]
fn spec147_members_refusals_and_project_guard_cover_whole_audience() {
    let f = UuidFixture::new();
    struct Checked(std::cell::RefCell<Vec<String>>);
    impl Directory for Checked {
        fn known_agent(&self, id: &str) -> bool {
            id != "14700000-ffff-4fff-8fff-000000000099"
        }
        fn member_facts(&self, _: &str) -> MemberFacts {
            MemberFacts::default()
        }
        fn communication_scope(
            &self,
            _: &str,
            members: &[String],
            reason: Option<&str>,
        ) -> Result<Vec<ProjectWarning>, Value> {
            *self.0.borrow_mut() = members.to_vec();
            if reason.is_none() {
                return Err(error(
                    "cross_project_reason_required",
                    "motif requis",
                    false,
                ));
            }
            Ok(vec![])
        }
    }
    let directory = Checked(std::cell::RefCell::new(vec![]));
    let c = "14700000-ffff-4fff-8fff-00000000000f";
    let run = |actor: &str, members: Value, reason: Option<&str>| {
        let request = ThreadRequest {version:1,cross_project_reason:reason.map(str::to_string),request:serde_json::from_value(json!({"action":"add_members","thread_id":UUID_T,"members":members,"operation_id":uuid::Uuid::new_v4().to_string()})).unwrap()};
        let (result, changed) = handle_with_change(&f.store, &directory, actor, request, 2);
        (result.result, changed)
    };
    let before = f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
    for (actor, members, code) in [
        (UUID_B, json!([c]), "creator_required"),
        (c, json!([c]), "thread_unavailable"),
        (UUID_A, json!([]), "invalid_request"),
        (UUID_A, json!(["bad"]), "invalid_request"),
        (
            UUID_A,
            json!(["14700000-ffff-4fff-8fff-000000000099"]),
            "unknown_member",
        ),
    ] {
        let (result, changed) = run(actor, members, Some("mandat"));
        assert_eq!(result["code"], code, "{result}");
        assert!(changed.is_none());
        assert_eq!(
            f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap(),
            before
        );
    }
    let (refused, changed) = run(UUID_A, json!([c]), None);
    assert_eq!(refused["code"], "cross_project_reason_required");
    assert!(changed.is_none());
    assert_eq!(
        &*directory.0.borrow(),
        &vec![UUID_A.to_string(), UUID_B.to_string(), c.to_string()]
    );
    let candidates: Vec<_> = (0..14)
        .map(|n| format!("14700000-ffff-4fff-8fff-{n:012x}"))
        .collect();
    assert_eq!(
        run(UUID_A, json!(candidates), Some("mandat")).0["status"],
        "members_added"
    );
    let view = f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
    assert_eq!(view.members.len(), 16);
    assert_eq!(
        run(UUID_A, json!([c, c.to_uppercase()]), Some("mandat")).0["code"],
        "capacity_exceeded"
    );
    assert_eq!(
        run(
            UUID_A,
            json!([candidates[0].clone(), candidates[0].to_uppercase()]),
            Some("mandat")
        )
        .0["status"],
        "no_change"
    );
    f.run(
        UUID_A,
        ThreadAction::Close {
            thread_id: UUID_T.into(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        },
    );
    assert_eq!(
        run(UUID_A, json!([candidates[0].clone()]), Some("mandat")).0["code"],
        "thread_closed"
    );
}

#[test]
fn spec147_uuid_create_replays_across_case_without_second_commit() {
    for upper_first in [false, true] {
        let f = UuidFixture::new();
        let action = |upper: bool| ThreadAction::Create {
            title: "Titre É exact".into(),
            members: vec![if upper {
                UUID_B.to_uppercase()
            } else {
                UUID_B.into()
            }],
            operation_id: if upper {
                UUID_OP.to_uppercase()
            } else {
                UUID_OP.into()
            },
        };
        let (first, changed) = f.run(UUID_A, action(upper_first));
        assert_eq!(first["status"], "created", "{first}");
        assert_eq!(changed.as_deref(), first["thread_id"].as_str());
        let before = f.store.connection().total_changes();
        let (replay, changed) = f.run(UUID_A, action(!upper_first));
        assert_eq!(replay, first);
        assert!(changed.is_none());
        assert_eq!(f.store.connection().total_changes(), before);
    }
}

#[test]
fn spec147_uuid_post_replays_across_case_and_keeps_exact_body() {
    for upper_first in [false, true] {
        let f = UuidFixture::new();
        let (first, changed) = f.post(
            upper_first,
            vec![if upper_first {
                UUID_B.to_uppercase()
            } else {
                UUID_B.into()
            }],
        );
        assert_eq!(first["status"], "posted", "{first}");
        assert_eq!(changed.as_deref(), Some(UUID_T));
        let before = f.store.connection().total_changes();
        let (replay, changed) = f.post(
            !upper_first,
            vec![if upper_first {
                UUID_B.into()
            } else {
                UUID_B.to_uppercase()
            }],
        );
        assert_eq!(replay, first);
        assert!(changed.is_none());
        assert_eq!(f.store.connection().total_changes(), before);
        let (history, _) = f.run(
            UUID_A,
            ThreadAction::History {
                thread_id: UUID_T.into(),
                from_seq: None,
                to_seq: None,
                limit: None,
            },
        );
        assert_eq!(history["entries"][0]["body"], "É 🦀\r\n  Exact\n\n");
    }
}

#[test]
fn spec147_uuid_close_replays_across_case_without_second_commit() {
    for upper_first in [false, true] {
        let f = UuidFixture::new();
        let action = |upper: bool| ThreadAction::Close {
            thread_id: if upper {
                UUID_T.to_uppercase()
            } else {
                UUID_T.into()
            },
            operation_id: if upper {
                UUID_OP.to_uppercase()
            } else {
                UUID_OP.into()
            },
        };
        let (first, changed) = f.run(UUID_A, action(upper_first));
        assert_eq!(first["status"], "closed", "{first}");
        assert_eq!(changed.as_deref(), Some(UUID_T));
        let before = f.store.connection().total_changes();
        let (replay, changed) = f.run(UUID_A, action(!upper_first));
        assert_eq!(replay, first);
        assert!(changed.is_none());
        assert_eq!(f.store.connection().total_changes(), before);
    }
}

#[test]
fn spec147_uuid_ack_receipt_is_case_insensitive_but_invalid_stays_typed() {
    let f = UuidFixture::new();
    assert_eq!(f.post(false, vec![]).0["status"], "posted");
    let (page, _) = f.run(
        UUID_B,
        ThreadAction::Read {
            thread_id: UUID_T.into(),
            limit: None,
        },
    );
    let receipt = page["receipt"].as_str().unwrap();
    let (ack, _) = f.run(
        UUID_B,
        ThreadAction::Ack {
            thread_id: UUID_T.to_uppercase(),
            receipt: receipt.to_uppercase(),
        },
    );
    assert_eq!(ack["status"], "acknowledged", "{ack}");
    let before = f.store.connection().total_changes();
    let (again, _) = f.run(
        UUID_B,
        ThreadAction::Ack {
            thread_id: UUID_T.into(),
            receipt: receipt.into(),
        },
    );
    assert_eq!(again["status"], "already_acknowledged");
    assert_eq!(f.store.connection().total_changes(), before);
    let (bad, _) = f.run(
        UUID_B,
        ThreadAction::Ack {
            thread_id: UUID_T.into(),
            receipt: "bad".into(),
        },
    );
    assert_eq!(bad["code"], "receipt_invalid");
    assert_eq!(f.store.connection().total_changes(), before);
    let g = UuidFixture::new();
    assert_eq!(g.post(false, vec![]).0["status"], "posted");
    let (page, _) = g.run(
        UUID_B,
        ThreadAction::Read {
            thread_id: UUID_T.into(),
            limit: None,
        },
    );
    let receipt = page["receipt"].as_str().unwrap();
    let action = |upper: bool| ThreadAction::Post {
        thread_id: if upper {
            UUID_T.to_uppercase()
        } else {
            UUID_T.into()
        },
        operation_id: if upper {
            UUID_OP.to_uppercase()
        } else {
            UUID_OP.into()
        },
        body: "Réponse exacte".into(),
        notify: ThreadNotify::Targets(vec![]),
        reply_to_seq: None,
        ack_receipt: Some(if upper {
            receipt.to_uppercase()
        } else {
            receipt.into()
        }),
        kind: None,
        supersedes_seq: None,
    };
    let (posted, changed) = g.run(UUID_B, action(true));
    assert_eq!(posted["status"], "posted", "{posted}");
    assert_eq!(changed.as_deref(), Some(UUID_T));
    let before = g.store.connection().total_changes();
    let (replayed, changed) = g.run(UUID_B, action(false));
    assert_eq!(replayed, posted);
    assert!(changed.is_none());
    assert_eq!(g.store.connection().total_changes(), before);
}

#[test]
fn spec147_uuid_reads_show_history_use_the_same_thread() {
    let f = UuidFixture::new();
    assert_eq!(f.post(false, vec![]).0["status"], "posted");
    for action in [
        ThreadAction::Show {
            thread_id: UUID_T.to_uppercase(),
        },
        ThreadAction::History {
            thread_id: UUID_T.to_uppercase(),
            from_seq: None,
            to_seq: None,
            limit: None,
        },
        ThreadAction::Read {
            thread_id: UUID_T.to_uppercase(),
            limit: None,
        },
    ] {
        let (result, changed) = f.run(UUID_B, action);
        assert_ne!(result["status"], "error", "{result}");
        assert_eq!(result["thread_id"], UUID_T);
        assert!(changed.is_none());
    }
}

#[test]
fn spec147_uuid_members_and_targets_deduplicate_before_hash_and_membership() {
    let f = UuidFixture::new();
    let mixed_b = "14700000-bBbB-4bBb-8BbB-00000000000B";
    let (first, _) = f.run(
        UUID_A,
        ThreadAction::Create {
            title: "Identités".into(),
            operation_id: "14700000-dDdD-4dDd-8DdD-00000000000D".into(),
            members: vec![UUID_A.to_uppercase(), mixed_b.into(), UUID_B.into()],
        },
    );
    assert_eq!(first["status"], "created", "{first}");
    let before = f.store.connection().total_changes();
    let (replay, changed) = f.run(
        UUID_A,
        ThreadAction::Create {
            title: "Identités".into(),
            operation_id: UUID_OP.into(),
            members: vec![UUID_B.into()],
        },
    );
    assert_eq!(replay, first);
    assert!(changed.is_none());
    assert_eq!(f.store.connection().total_changes(), before);
    // Une autre clé est requise pour Post : la clé Create ne change pas d'enveloppe.
    let g = UuidFixture::new();
    let (posted, _) = g.post(
        true,
        vec![UUID_A.to_uppercase(), mixed_b.into(), UUID_B.into()],
    );
    assert_eq!(posted["status"], "posted", "{posted}");
    let (replayed, changed) = g.post(false, vec![UUID_A.into(), UUID_B.into()]);
    assert_eq!(replayed, posted);
    assert!(changed.is_none());
}

#[test]
fn spec147_uuid_invalid_forms_never_mutate_and_human_canonical_stays_strict() {
    let f = UuidFixture::new();
    let before = f.store.connection().total_changes();
    for invalid in [
        UUID_T.replace('-', ""),
        format!("{{{UUID_T}}}"),
        format!("urn:uuid:{UUID_T}"),
        format!(" {UUID_T}"),
        "invalid".into(),
    ] {
        let actions = [
            ThreadAction::Show {
                thread_id: invalid.clone(),
            },
            ThreadAction::Create {
                title: "Refus".into(),
                members: vec![UUID_B.into()],
                operation_id: invalid.clone(),
            },
            ThreadAction::Create {
                title: "Refus membre".into(),
                members: vec![invalid.clone()],
                operation_id: UUID_OP.into(),
            },
            ThreadAction::Close {
                thread_id: UUID_T.into(),
                operation_id: invalid.clone(),
            },
            ThreadAction::Post {
                thread_id: UUID_T.into(),
                operation_id: invalid.clone(),
                body: "Exact".into(),
                notify: ThreadNotify::Targets(vec![]),
                reply_to_seq: None,
                ack_receipt: None,
                kind: None,
                supersedes_seq: None,
            },
            ThreadAction::Post {
                thread_id: UUID_T.into(),
                operation_id: UUID_OP.into(),
                body: "Exact".into(),
                notify: ThreadNotify::Targets(vec![invalid.clone()]),
                reply_to_seq: None,
                ack_receipt: None,
                kind: None,
                supersedes_seq: None,
            },
            ThreadAction::Post {
                thread_id: UUID_T.into(),
                operation_id: UUID_OP.into(),
                body: "Exact".into(),
                notify: ThreadNotify::Targets(vec![]),
                reply_to_seq: None,
                ack_receipt: Some(invalid.clone()),
                kind: None,
                supersedes_seq: None,
            },
        ];
        for action in actions {
            let (bad, changed) = f.run(UUID_A, action);
            assert_eq!(bad["code"], "invalid_request", "{bad}");
            assert!(changed.is_none());
        }
    }
    assert_eq!(f.store.connection().total_changes(), before);
    assert!(canonical_uuid(&UUID_T.to_uppercase()).is_none());
    assert_eq!(canonical_uuid(UUID_T).as_deref(), Some(UUID_T));
    assert!(parse_recent_cursor(&format!("2:{}", UUID_T.to_uppercase())).is_err());
    assert!(parse_recent_cursor(&format!("2:{UUID_T}")).is_ok());
}

#[test]
fn spec147_done_keeps_a_change_marker_but_replay_keeps_only_the_receipt() {
    let value = json!({"status":"posted","thread_id":"14700000-0000-4000-8000-000000000001"});
    let mut changed = None;
    assert_eq!(
        tx_result(ThreadTxOutcome::Done(value.clone()), &mut changed).unwrap(),
        value
    );
    assert_eq!(changed.as_deref(), value["thread_id"].as_str());
    let mut replayed = None;
    assert_eq!(
        tx_result(ThreadTxOutcome::Replayed(value.clone()), &mut replayed).unwrap(),
        value
    );
    assert!(replayed.is_none());
}

#[test]
fn spec146_real_projection_descending_pages_exact_bodies_and_fixed_snapshot() {
    use bridget_transport::protocol::HumanThreadViewAction as A;
    let root = std::env::temp_dir().join(format!("h146-export-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&root).unwrap();
    let db = root.join("db");
    let store = Store::open(&db).unwrap();
    let mut profiles = crate::agent_profile::AgentProfileStore::open(&db).unwrap();
    let members = vec![
        "14600000-0000-4000-8000-000000000001".to_string(),
        "14600000-0000-4000-8000-000000000002".to_string(),
    ];
    profiles.ensure_agent_ids(members.iter().cloned()).unwrap();
    for (n, id) in members.iter().enumerate() {
        store.connection().execute("UPDATE agent_profiles SET display_name=?2,display_name_normalized=?2 WHERE agent_id=?1",rusqlite::params![id,format!("recette-{n}")]).unwrap();
    }
    let actor = &members[0];
    let thread = "14600000-0000-4001-8000-000000000001";
    for n in 1..=3 {
        let id = format!("14600000-0000-4001-8000-{n:012}");
        assert!(matches!(
            store
                .thread_create(CreateThread {
                    actor,
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: &id,
                    thread_id: &id,
                    title: &format!("Recette {n}"),
                    members: &members,
                    project_warnings: &[],
                    now: n,
                    limits: &LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
    }
    let mut bodies = serde_json::Map::new();
    for n in 1..=137 {
        let body = format!(
            "Message {n} é 🦀\r\n  espaces conservés\n\n{}",
            "0123456789 ".repeat(155)
        );
        bodies.insert(n.to_string(), json!(body));
        assert!(matches!(
            store
                .thread_post(PostEntry {
                    project_warnings: &[],
                    actor,
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: &format!("entry-{n}"),
                    thread_id: thread,
                    message_id: &uuid::Uuid::new_v4().to_string(),
                    body: &body,
                    notify: &NotifySpec::None,
                    notices: &[],
                    reply_to_seq: None,
                    ack_receipt: None,
                    kind: Some(if n == 1 || n == 137 {
                        ThreadEntryKind::Action
                    } else {
                        ThreadEntryKind::History
                    }),
                    supersedes_seq: (n == 137).then_some(1),
                    now: 100 + n,
                    limits: &LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
    }
    let before = store.connection().total_changes();
    let listed = human_view(
        &store,
        actor,
        &A::ListRecent {
            limit: Some(2),
            after: None,
        },
    );
    for action in [
        A::ListRecent {
            limit: Some(0),
            after: None,
        },
        A::ListRecent {
            limit: Some(101),
            after: None,
        },
        A::ListRecent {
            limit: Some(1),
            after: Some("not-a-cursor".into()),
        },
        A::HistoryRecent {
            thread_id: thread.into(),
            before_seq: Some(9_007_199_254_740_992),
            to_seq: None,
            limit: Some(1),
        },
        A::HistoryRecent {
            thread_id: thread.into(),
            before_seq: None,
            to_seq: Some(9_007_199_254_740_992),
            limit: Some(1),
        },
        A::HistoryRecent {
            thread_id: thread.into(),
            before_seq: None,
            to_seq: None,
            limit: Some(201),
        },
    ] {
        let refused = serde_json::to_value(human_view(&store, actor, &action)).unwrap();
        assert_eq!(refused["result"]["code"], "invalid_request");
        assert!(refused["subject"].is_null());
    }
    let shown = human_view(
        &store,
        actor,
        &A::Show {
            thread_id: thread.into(),
        },
    );
    let mut pages = Vec::new();
    let mut cursor = None;
    let mut all = Vec::new();
    loop {
        let projected = human_view(
            &store,
            actor,
            &A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: cursor,
                to_seq: Some(137),
                limit: Some(100),
            },
        );
        let value = serde_json::to_value(&projected).unwrap();
        let result = &value["result"];
        assert_eq!(result["status"], "history_recent");
        assert_eq!(result["snapshot_seq"], 137);
        for e in result["entries"].as_array().unwrap() {
            let seq = e["seq"].as_u64().unwrap();
            assert_eq!(e["body"], bodies[&seq.to_string()]);
            all.push(seq);
            if seq == 1 {
                assert_eq!(e["superseded_by_seq"], 137);
            }
        }
        cursor = result["next_before_seq"].as_u64();
        pages.push(value);
        if cursor.is_none() {
            break;
        }
    }
    assert!(pages.len() > 3);
    assert_eq!(all, (1..=137).rev().collect::<Vec<_>>());
    assert_eq!(store.connection().total_changes(), before);
    let output = json!({"listed":listed,"shown":shown,"pages":pages,"body_by_seq":bodies});
    if let Ok(directory) = std::env::var("BRIDGET_SPEC146_EXPORT_DIR") {
        std::fs::write(
            std::path::Path::new(&directory).join("interop-146.json"),
            serde_json::to_vec_pretty(&output).unwrap(),
        )
        .unwrap();
    }
    // Publication après ouverture : la même borne ne voit pas138.
    assert!(matches!(
        store
            .thread_post(PostEntry {
                project_warnings: &[],
                actor,
                operation_id: &uuid::Uuid::new_v4().to_string(),
                canonical_hash: "after-snapshot",
                thread_id: thread,
                message_id: &uuid::Uuid::new_v4().to_string(),
                body: "nouveau138",
                notify: &NotifySpec::None,
                notices: &[],
                reply_to_seq: None,
                ack_receipt: None,
                kind: Some(ThreadEntryKind::History),
                supersedes_seq: None,
                now: 300,
                limits: &LIMITS
            })
            .unwrap(),
        ThreadTxOutcome::Done(_)
    ));
    let still = serde_json::to_value(human_view(
        &store,
        actor,
        &A::HistoryRecent {
            thread_id: thread.into(),
            before_seq: None,
            to_seq: Some(137),
            limit: Some(1),
        },
    ))
    .unwrap();
    assert_eq!(still["result"]["entries"][0]["seq"], 137);
    let fresh = serde_json::to_value(human_view(
        &store,
        actor,
        &A::HistoryRecent {
            thread_id: thread.into(),
            before_seq: None,
            to_seq: None,
            limit: Some(1),
        },
    ))
    .unwrap();
    assert_eq!(fresh["result"]["entries"][0]["seq"], 138);
    drop(profiles);
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec146_recent_projection_limits_and_cursor_fail_closed() {
    for input in [
        "",
        "-1:14600000-0000-4000-8000-000000000001",
        "01:14600000-0000-4000-8000-000000000001",
        "+1:14600000-0000-4000-8000-000000000001",
        "9007199254740992:14600000-0000-4000-8000-000000000001",
        "1:14600000-0000-4000-8000-000000000001:extra",
    ] {
        assert!(parse_recent_cursor(input).is_err(), "{input}");
    }
    assert!(parse_recent_cursor("0:14600000-0000-4000-8000-000000000001").is_ok());
}

#[test]
fn spec145_projection_bound_and_member_revocation() {
    use bridget_transport::protocol::HumanThreadViewAction as A;
    let root =
        std::env::temp_dir().join(format!("h145-projection-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&root).unwrap();
    let db = root.join("db");
    let store = Store::open(&db).unwrap();
    let mut profiles = crate::agent_profile::AgentProfileStore::open(&db).unwrap();
    let mut members = (0..16)
        .map(|n| format!("14500000-0000-4000-8000-{n:012}"))
        .collect::<Vec<_>>();
    members.sort();
    profiles.ensure_agent_ids(members.iter().cloned()).unwrap();
    for (n, member) in members.iter().enumerate() {
        store.connection().execute("UPDATE agent_profiles SET display_name=?2,display_name_normalized=?3 WHERE agent_id=?1",
                rusqlite::params![member,format!("{}{}","🦀".repeat(78),n),format!("member-{n}")]).unwrap();
    }
    let actor = &members[0];
    for n in 0..24 {
        let thread = format!("14500000-0000-4001-8000-{n:012}");
        assert!(matches!(
            store
                .thread_create(CreateThread {
                    actor,
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: &thread,
                    thread_id: &thread,
                    title: "Projection bornée",
                    members: &members,
                    project_warnings: &[],
                    now: 1,
                    limits: &LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
    }
    let before = store.connection().total_changes();
    let refused = serde_json::to_value(human_view(
        &store,
        actor,
        &A::List {
            limit: Some(100),
            after_thread_id: None,
        },
    ))
    .unwrap();
    assert_eq!(refused["result"]["code"], "response_too_large");
    assert!(refused["subject"].is_null());
    let recent_refused = serde_json::to_value(human_view(
        &store,
        actor,
        &A::ListRecent {
            limit: Some(100),
            after: None,
        },
    ))
    .unwrap();
    assert_eq!(recent_refused["result"]["code"], "response_too_large");
    assert!(recent_refused["subject"].is_null());
    assert_eq!(store.connection().total_changes(), before);
    let first = human_view(
        &store,
        actor,
        &A::List {
            limit: Some(10),
            after_thread_id: None,
        },
    );
    let first = serde_json::to_value(first).unwrap();
    assert_eq!(first["result"]["threads"].as_array().unwrap().len(), 10);
    let second = serde_json::to_value(human_view(
        &store,
        actor,
        &A::List {
            limit: Some(10),
            after_thread_id: Some(first["result"]["next_after"].as_str().unwrap().into()),
        },
    ))
    .unwrap();
    assert!(
        first["result"]["threads"]
            .as_array()
            .unwrap()
            .iter()
            .all(|a| second["result"]["threads"]
                .as_array()
                .unwrap()
                .iter()
                .all(|b| a["thread_id"] != b["thread_id"]))
    );
    let thread = "14500000-0000-4001-8000-000000000000";
    store
        .connection()
        .execute(
            "DELETE FROM discussion_members WHERE thread_id=?1 AND agent_id=?2",
            rusqlite::params![thread, actor],
        )
        .unwrap();
    let before = store.connection().total_changes();
    let refused = serde_json::to_value(human_view(
        &store,
        actor,
        &A::Show {
            thread_id: thread.into(),
        },
    ))
    .unwrap();
    assert_eq!(refused["result"]["code"], "thread_unavailable");
    assert_eq!(store.connection().total_changes(), before);
    drop(store);
    drop(profiles);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec138_thread_reason_absence_is_legacy_but_null_is_invalid() {
    let legacy =
        json!({"version":1,"request":{"action":"list","limit":null,"after_thread_id":null}});
    let parsed: ThreadRequest = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), legacy);
    let mut invalid = legacy;
    invalid["cross_project_reason"] = Value::Null;
    assert!(serde_json::from_value::<ThreadRequest>(invalid).is_err());
}

#[test]
fn spec102_v05_uuid_canonique_et_bornes_de_page() {
    assert!(canonical_uuid("10200000-0000-4000-8000-00000000000a").is_some());
    assert!(
        canonical_uuid("10200000-0000-4000-8000-00000000000A").is_none(),
        "majuscules refusées"
    );
    assert!(canonical_uuid("102000000000400080000000000000a").is_none());
    assert!(canonical_uuid("nom-lisible").is_none());
    assert_eq!(page_limit(None, PAGE_DEFAULT, PAGE_MAX).unwrap(), 50);
    assert_eq!(page_limit(Some(200), PAGE_DEFAULT, PAGE_MAX).unwrap(), 200);
    assert!(page_limit(Some(201), PAGE_DEFAULT, PAGE_MAX).is_err());
    assert!(page_limit(Some(0), PAGE_DEFAULT, PAGE_MAX).is_err());
    assert!(page_limit(Some(101), LIST_DEFAULT, LIST_MAX).is_err());
}

#[test]
fn spec102_v14_corps_et_titre_bornes_sans_coupe() {
    assert!(validate_body("").is_err());
    assert!(validate_body("   \n\t").is_err());
    assert!(validate_body(&"x".repeat(BODY_MAX_BYTES)).is_ok());
    assert!(validate_body(&"x".repeat(BODY_MAX_BYTES + 1)).is_err());
    assert!(validate_body("é".repeat(BODY_MAX_BYTES / 2).as_str()).is_ok());
    assert_eq!(
        validate_title("  Relecture sécurité  ").unwrap(),
        "Relecture sécurité"
    );
    assert!(validate_title(" ").is_err());
    assert!(validate_title(&"t".repeat(TITLE_MAX_CHARS)).is_ok());
    assert!(validate_title(&"t".repeat(TITLE_MAX_CHARS + 1)).is_err());
    // Une entrée pleine d'échappements JSON peut dépasser le budget d'entrée
    // alors que son corps respecte 16 Kio : le contrôle porte sur le JSON.
    let quoted = "\"\\".repeat(BODY_MAX_BYTES / 2);
    assert!(validate_body(&quoted).is_ok());
    assert!(entry_json_len("m", "a", &quoted, &[], None) > BODY_MAX_BYTES);
    assert!(
        entry_json_len("m", "a", &"x".repeat(BODY_MAX_BYTES), &[], None) <= ENTRY_MAX_JSON_BYTES
    );
}

#[test]
fn spec102_v20_empreinte_stable_et_sensible() {
    let a = digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"b"]);
    assert_eq!(
        a,
        digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"b"])
    );
    assert_ne!(
        a,
        digest(&[b"post", b"v1", b"t", b"corps ", b"targets", b"b"]),
        "espace final compte"
    );
    assert_ne!(
        a,
        digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"c"])
    );
    // Les frontières de champs sont encodées : "ab"+"c" ≠ "a"+"bc".
    assert_ne!(digest(&[b"ab", b"c"]), digest(&[b"a", b"bc"]));
}

#[test]
fn spec102_v23_alerte_neutre_sans_corps_ni_titre() {
    let body = notice_body("33333333-3333-4333-8333-333333333333", 20);
    assert!(body.contains("33333333-3333-4333-8333-333333333333"));
    assert!(body.contains("jusqu'à 20"));
    assert!(body.contains("Ne réponds pas par message direct"));
    assert!(body.contains("même read sur le même"));
    assert!(body.contains("history ne fait pas avancer le repère"));
    assert!(!body.contains("Bridget thread"));
    assert_eq!(notice_message_id("t", "b", 3), "thread-notice:t:b:3");
    assert_ne!(
        notice_message_id("t", "b", 3),
        notice_message_id("t", "b", 4)
    );
}

#[test]
fn spec102_v22_version_inconnue_refusee_avant_toute_lecture() {
    struct NoDirectory;
    impl Directory for NoDirectory {
        fn known_agent(&self, _: &str) -> bool {
            false
        }
        fn member_facts(&self, _: &str) -> MemberFacts {
            MemberFacts::default()
        }
    }
    let root =
        std::env::temp_dir().join(format!("spec102-threads-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap();
    let store = Store::open(&root.join("db.sqlite")).unwrap();
    let outcome = handle(
        &store,
        &NoDirectory,
        "10200000-0000-4000-8000-00000000000a",
        ThreadRequest {
            cross_project_reason: None,
            version: 2,
            request: ThreadAction::List {
                limit: None,
                after_thread_id: None,
            },
        },
        1,
    );
    assert_eq!(outcome.result["code"], "unsupported_version");
    let outcome = handle(
        &store,
        &NoDirectory,
        "10200000-0000-4000-8000-00000000000a",
        ThreadRequest {
            cross_project_reason: None,
            version: 1,
            request: ThreadAction::Show {
                thread_id: "pas-un-uuid".into(),
            },
        },
        1,
    );
    assert_eq!(outcome.result["code"], "invalid_request");
    let _ = std::fs::remove_dir_all(root);
}
