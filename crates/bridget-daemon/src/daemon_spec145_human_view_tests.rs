    use super::*;
    use serde_json::{Value, json};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    const T3: &str = "89000000-0000-4000-8000-000000000145";

    fn read_request(version: u16) -> WrapperToDaemon {
        serde_json::from_value(json!({"type":"HumanThreadViewV1","request":{
            "version":version,"t3_thread_id":T3,"project_root":"/fixture",
            "request":{"action":"list"}
        }}))
        .expect("contrat humain disponible")
    }

    fn response(state: &Arc<Mutex<DaemonState>>, conn: &str, version: u16) -> Value {
        serde_json::to_value(handle_wrapper_message(conn, read_request(version), state).unwrap())
            .unwrap()["result"]
            .clone()
    }

    fn call(state: &Arc<Mutex<DaemonState>>, root: &str, action: Value) -> Value {
        let request = serde_json::from_value(json!({"type":"HumanThreadViewV1","request":{
            "version":1,"t3_thread_id":T3,"project_root":root,"request":action
        }}))
        .unwrap();
        serde_json::to_value(handle_wrapper_message("human",request,state).unwrap()).unwrap()["result"].clone()
    }

    fn snapshot(state: &Arc<Mutex<DaemonState>>) -> Vec<(String, Vec<String>)> {
        let st = state.lock().unwrap();
        let db = rusqlite::Connection::open_with_flags(
            &st.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let tables = db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap()
            .query_map([],|row|row.get::<_,String>(0)).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
        tables
            .into_iter()
            .map(|table| {
                let mut stmt = db
                    .prepare(&format!("SELECT * FROM \"{}\"", table.replace('"', "\"\"")))
                    .unwrap();
                let cols = stmt.column_count();
                let mut rows = stmt
                    .query_map([], |row| {
                        (0..cols)
                            .map(|col| row.get::<_, rusqlite::types::Value>(col))
                            .collect::<Result<Vec<_>, _>>()
                            .map(|row| format!("{row:?}"))
                    })
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                rows.sort();
                (table, rows)
            })
            .collect()
    }

    fn unbound_thread_fixture(
        label: &str,
        t3_thread_id: &str,
    ) -> (Arc<Mutex<DaemonState>>, String, UnixStream) {
        let (mut st, _) = presence_tests::state_with_registered_agent(label);
        st.conn_counter = 1; // conn-1 appartient déjà à la fixture héritée.
        let peer = spec094_live_test_connection(&mut st, "t3");
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "t3",
                2,
                "fixture".into(),
                crate::t3code::stable_uuid(t3_thread_id),
                Some(host.clone()),
                Some("t3code".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Cli),
                None,
                Some("test".into()),
                Some(crate::t3code::stable_uuid(&format!(
                    "instance:{t3_thread_id}"
                ))),
                None,
                false,
                Some(false),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connection_roles
            .insert("human".into(), ConnectionRole::Client);
        st.client_negotiations.insert(
            "human".into(),
            NegotiatedClient {
                version: 1,
                issuer_scope: "spec145-human".into(),
                capabilities: vec![serde_json::from_value(json!("human_thread_view_v1")).unwrap()],
            },
        );
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let state = Arc::new(Mutex::new(st));
        assert!(matches!(
            announce_communication_project(
                &state,
                "t3",
                &root,
                bridget_transport::protocol::CommunicationProjectSource::T3,
                &host,
                None
            ),
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ));
        (state, root, peer)
    }

    fn fixture(label: &str) -> (Arc<Mutex<DaemonState>>, String, UnixStream) {
        let (state, root, peer) = unbound_thread_fixture(label, T3);
        let fact = WrapperToDaemon::T3ThreadBindingFact {
            version: bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION,
            t3_thread_id: T3.into(),
        };
        assert!(handle_wrapper_message("t3", fact, &state).is_none());
        (state, root, peer)
    }

    #[test]
    fn spec172_t3_opaque_binding_accepts_attested_primary_identity() {
        for (index, id) in [
            T3,
            "mcp:f5a66bd1-7cef-4d2c-888b-06dd7d8263d3",
            "thread:mcp:scope172:request172:0",
        ]
        .into_iter()
        .enumerate()
        {
            let (state, _root, _peer) = unbound_thread_fixture(&format!("opaque172-{index}"), id);
            let before = snapshot(&state);
            let response = handle_wrapper_message(
                "t3",
                WrapperToDaemon::T3ThreadBindingFact {
                    version: bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION,
                    t3_thread_id: id.into(),
                },
                &state,
            );
            assert!(
                response.is_none(),
                "identifiant T3 attesté refusé : {id}: {response:?}"
            );
            {
                let st = state.lock().unwrap();
                let agent = crate::t3code::stable_uuid(id);
                let project = agent_communication_project(&st, &agent);
                assert_eq!(
                    human_authority_guard(&st, id, &project),
                    Ok(("t3".into(), agent))
                );
            }
            let after = snapshot(&state);
            let tracking = after.iter().find(|(name,_)|name=="agent_tracking").unwrap();
            assert_eq!(tracking.1.len(),1,"la liaison conserve une seule fiche native");
            assert_eq!(
                after.into_iter().filter(|(name,_)|name!="agent_tracking").collect::<Vec<_>>(),
                before.into_iter().filter(|(name,_)|name!="agent_tracking").collect::<Vec<_>>(),
                "seule la métadonnée de liaison est persistée"
            );
        }
    }

    #[test]
    fn spec172_t3_opaque_binding_preserves_identifier_and_authority_guards() {
        let id = "mcp:f5a66bd1-7cef-4d2c-888b-06dd7d8263d3";
        for (index, invalid) in [
            String::new(),
            "thread\nforged".into(),
            "mcp:\u{202e}forged".into(),
            "mcp:\u{200b}forged".into(),
            "x".repeat(2049),
        ]
        .into_iter()
        .enumerate()
        {
            let (state, _root, _peer) =
                unbound_thread_fixture(&format!("badopaque172-{index}"), &invalid);
            let before = snapshot(&state);
            let response = handle_wrapper_message(
                "t3",
                WrapperToDaemon::T3ThreadBindingFact {
                    version: bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION,
                    t3_thread_id: invalid,
                },
                &state,
            );
            assert!(matches!(response, Some(DaemonToWrapper::Nack { .. })));
            assert!(state.lock().unwrap().t3_thread_bindings.is_empty());
            assert_eq!(snapshot(&state), before);
        }
        for case in [
            "version",
            "foreign_identity",
            "auxiliary",
            "unattested_project",
        ] {
            let (state, _root, _peer) = unbound_thread_fixture(&format!("guardopaque172-{case}"), id);
            let before = snapshot(&state);
            {
                let mut st = state.lock().unwrap();
                match case {
                    "auxiliary" => {
                        st.auxiliary_connections.insert("t3".into());
                    }
                    "unattested_project" => {
                        st.t3_project_connections.remove("t3");
                    }
                    _ => {}
                }
            }
            let response = handle_wrapper_message(
                "t3",
                WrapperToDaemon::T3ThreadBindingFact {
                    version: if case == "version" {
                        99
                    } else {
                        bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION
                    },
                    t3_thread_id: if case == "foreign_identity" {
                        "mcp:other-thread".into()
                    } else {
                        id.into()
                    },
                },
                &state,
            );
            assert!(
                matches!(response, Some(DaemonToWrapper::Nack { .. })),
                "{case}"
            );
            assert!(
                state.lock().unwrap().t3_thread_bindings.is_empty(),
                "{case}"
            );
            assert_eq!(snapshot(&state), before, "{case}");
        }
    }

    #[test]
    fn spec147_watch_real_socket_accepts_human_ready_without_business_writes() {
        let (state, root, _peer) = fixture("h147-red");
        let before = snapshot(&state);
        let (mut stream, server) = UnixStream::pair().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let server_state = state.clone();
        let handle = thread::spawn(move || {
            handle_connection(server, server_state).map_err(|e| e.to_string())
        });
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        for request in [
            json!({"type":"RoleHandshake","role":"client"}),
            json!({"type":"ClientHello","contract_version":1,"issuer_scope":"scope147_human_watch_00000000","capabilities":["human_thread_watch_v1"]}),
            json!({"type":"HumanThreadWatchV1","request":{"version":1,"t3_thread_id":T3,"project_root":root}}),
        ] {
            writeln!(stream, "{request}").unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let response: Value = serde_json::from_str(&line).unwrap();
            if request["type"] == "HumanThreadWatchV1" {
                assert_eq!(response["event"]["status"], "ready", "{response}");
                assert_eq!(response["event"]["seq"], 0);
            }
        }
        drop(reader);
        drop(stream);
        handle.join().unwrap().unwrap();
        assert_eq!(snapshot(&state), before);
    }

    fn watch_socket(
        state: &Arc<Mutex<DaemonState>>,
        root: &str,
    ) -> (
        BufReader<UnixStream>,
        thread::JoinHandle<Result<(), String>>,
    ) {
        let (mut client, server) = UnixStream::pair().unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let shared = state.clone();
        let handle =
            thread::spawn(move || handle_connection(server, shared).map_err(|e| e.to_string()));
        let mut reader = BufReader::new(client.try_clone().unwrap());
        for request in [
            json!({"type":"RoleHandshake","role":"client"}),
            json!({"type":"ClientHello","contract_version":1,"issuer_scope":"scope147_human_watch_fixture00","capabilities":["human_thread_watch_v1"]}),
        ] {
            writeln!(client, "{request}").unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(serde_json::from_str::<Value>(&line).unwrap()["type"] != "ClientRejected");
        }
        writeln!(client,"{}",json!({"type":"HumanThreadWatchV1","request":{"version":1,"t3_thread_id":T3,"project_root":root}})).unwrap();
        drop(client);
        (reader, handle)
    }

    fn watch_event(reader: &mut BufReader<UnixStream>) -> Value {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        serde_json::from_str::<Value>(&line).unwrap()["event"].clone()
    }

    fn operation(state: &Arc<Mutex<DaemonState>>, action: Value) -> Value {
        let request=serde_json::from_value(json!({"type":"ThreadRequest","request":{
            "version":1,"cross_project_reason":"Échange synthétique explicite pour la recette147", "request":action
        }})).unwrap();
        serde_json::to_value(handle_wrapper_message("t3",request,state).unwrap()).unwrap()["result"]["result"].clone()
    }

    fn create_action() -> Value {
        json!({"action":"create","title":"Titre secret147", "members":["89000000-0000-4000-8000-000000000102"], "operation_id":uuid::Uuid::new_v4().to_string()})
    }

    #[test]
    fn spec147_members_capability_guard_refuses_before_mutation() {
        let (state, _, _peer) = fixture("h147members-guard");
        let created = operation(&state, create_action());
        let before = snapshot(&state);
        let refused = operation(
            &state,
            json!({"action":"add_members","thread_id":created["thread_id"],"members":[],"operation_id":uuid::Uuid::new_v4().to_string()}),
        );
        assert_eq!(refused["code"], "thread_members_unsupported");
        assert_eq!(snapshot(&state), before);
    }

    #[test]
    fn spec147_members_pending_wake_is_unchanged_and_watch_only_follows_commit() {
        use crate::store::threads::{NotifySpec, PostEntry, ThreadTxOutcome};
        let (state, root, mut peer) = fixture("h147members-silent");
        let c = "14700000-0000-4000-8000-000000000003";
        let a = crate::t3code::stable_uuid(T3);
        let mut b_peer = spec094_live_test_connection(&mut state.lock().unwrap(), "conn-1");
        let mut c_peer = {
            let mut st = state.lock().unwrap();
            let peer = spec094_live_test_connection(&mut st, "member-c");
            let host = st.host.clone();
            assert!(matches!(
                handle_register_with_channel(
                    "member-c",
                    2,
                    "fixture-c".into(),
                    c.into(),
                    Some(host),
                    Some("fixture".into()),
                    ChannelReport::Known("unix".into()),
                    Some(PresenceMode::Cli),
                    None,
                    Some("test".into()),
                    Some("member-c-instance".into()),
                    None,
                    false,
                    Some(false),
                    &mut st,
                    false
                ),
                DaemonToWrapper::Registered { .. }
            ));
            peer
        };
        let created = operation(&state, create_action());
        let id = created["thread_id"].as_str().unwrap();
        let (mut watch, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut watch)["status"], "ready");
        let pending = {
            let st = state.lock().unwrap();
            let notify = NotifySpec::Targets(vec!["89000000-0000-4000-8000-000000000102".into()]);
            assert!(matches!(
                st.store
                    .thread_post(PostEntry {
                        actor: &a,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: "pending-member-proof",
                        thread_id: id,
                        message_id: &uuid::Uuid::new_v4().to_string(),
                        body: "ancienne action",
                        notify: &notify,
                        notices: &[],
                        project_warnings: &[],
                        reply_to_seq: None,
                        ack_receipt: None,
                        kind: None,
                        supersedes_seq: None,
                        now: unix_now_secs(),
                        limits: &crate::threads::LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
            st.store
                .thread_show("89000000-0000-4000-8000-000000000102", id)
                .unwrap()
                .unwrap()
                .own_wake
                .unwrap()
        };
        assert_eq!(pending.state, "pending");
        let (instance, credential) = {
            let st = state.lock().unwrap();
            (
                st.conn_instances["t3"].clone(),
                st.identity_credentials["t3"].clone(),
            )
        };
        let _aux_peer = spec094_live_test_connection(&mut state.lock().unwrap(), "member-aux");
        assert!(matches!(
            handle_wrapper_message(
                "member-aux",
                WrapperToDaemon::RegisterAuxiliary {
                    agent_id: a.clone(),
                    instance_id: instance,
                    credential,
                },
                &state
            ),
            Some(DaemonToWrapper::Registered { .. })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "member-aux",
                WrapperToDaemon::ClientHello {
                    contract_version: 1,
                    issuer_scope: "membership147_primary_fixture".into(),
                    capabilities: vec![
                        ClientCapability::ThreadMembersV1,
                        ClientCapability::CommunicationProjectsV1
                    ]
                },
                &state
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
        let member_operation = |action: Value| {
            let request = serde_json::from_value(json!({"type":"ThreadRequest","request":{
                "version":1,"cross_project_reason":"Recette membres147 explicite","request":action
            }}))
            .unwrap();
            serde_json::to_value(handle_wrapper_message("member-aux", request, &state).unwrap())
                .unwrap()["result"]["result"]
                .clone()
        };
        let action = json!({"action":"add_members","thread_id":id,"members":[c],"operation_id":uuid::Uuid::new_v4().to_string()});
        let added = member_operation(action.clone());
        assert_eq!(added["status"], "members_added", "{added}");
        assert_eq!(watch_event(&mut watch)["seq"], 1);
        assert_eq!(member_operation(action), added);
        assert_eq!(
            member_operation(
                json!({"action":"add_members","thread_id":id,"members":[c],"operation_id":uuid::Uuid::new_v4().to_string()})
            )["status"],
            "no_change"
        );
        assert_eq!(
            state
                .lock()
                .unwrap()
                .store
                .thread_show("89000000-0000-4000-8000-000000000102", id)
                .unwrap()
                .unwrap()
                .own_wake
                .as_ref(),
            Some(&pending)
        );
        assert_eq!(
            state
                .lock()
                .unwrap()
                .store
                .thread_show(c, id)
                .unwrap()
                .unwrap()
                .own
                .acked_seq,
            0
        );
        use std::io::Read;
        for primary in [&mut peer, &mut b_peer, &mut c_peer] {
            primary.set_nonblocking(true).unwrap();
            assert!(
                matches!(primary.read(&mut [0]),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock),
                "aucun Deliver même avec pending préexistant"
            );
        }
        watch
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        assert!(
            watch.read_line(&mut String::new()).is_err(),
            "replay/NoChange silencieux"
        );
        drop(watch);
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn spec147_ready_and_terminal_survive_saturation_and_overflow() {
        use bridget_transport::protocol::{
            HUMAN_THREAD_WATCH_MAX_SEQ, HumanThreadViewError as R, HumanThreadWatchEvent as E,
        };
        let mut queue = HumanWatchQueue::new();
        let original = queue.generation.clone();
        for _ in 0..100 {
            queue.changed();
            assert!(queue.pending.len() <= HUMAN_WATCH_QUEUE_MAX);
        }
        assert!(matches!(
            queue.pending.pop_front(),
            Some(E::Ready { seq: 0, .. })
        ));
        assert!(matches!(
            queue.pending.pop_front(),
            Some(E::Resync { seq: 100, .. })
        ));
        queue.changed();
        queue.terminate(R::ProjectMismatch);
        for _ in 0..100 {
            queue.changed();
        }
        assert_eq!(
            queue.pending.pop_front(),
            Some(E::error(R::ProjectMismatch))
        );
        assert!(queue.pending.is_empty());
        let mut overflow = HumanWatchQueue::new();
        overflow.seq = HUMAN_THREAD_WATCH_MAX_SEQ;
        overflow.changed();
        assert!(matches!(
            overflow.pending.pop_front(),
            Some(E::Error {
                code: R::ResponseTooLarge,
                ..
            })
        ));
        assert_ne!(original, HumanWatchQueue::new().generation);
    }

    #[test]
    fn spec147_revocation_before_first_output_does_not_claim_ready() {
        use bridget_transport::protocol::{HumanThreadViewError as R, HumanThreadWatchEvent as E};
        let mut queue = HumanWatchQueue::new();
        queue.terminate(R::BindingUnavailable);
        assert!(matches!(
            queue.pending.pop_front(),
            Some(E::Error {
                code: R::BindingUnavailable,
                ..
            })
        ));
        assert!(queue.pending.is_empty());
    }

    #[test]
    fn spec147_mutations_replay_reads_ack_foreign_and_rename_are_filtered() {
        let (state, root, mut peer) = fixture("h147m");
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["seq"], 0);
        let create = create_action();
        let created = operation(&state, create.clone());
        assert_eq!(created["status"], "created", "{created}");
        let thread_id = created["thread_id"].as_str().unwrap();
        assert_eq!(watch_event(&mut reader)["seq"], 1);
        assert_eq!(operation(&state, create), created);
        let post = json!({"action":"post","thread_id":thread_id,"body":"secret147 body","notify":[],
            "operation_id":uuid::Uuid::new_v4().to_string(),"kind":"action"});
        let posted = operation(&state, post.clone());
        assert_eq!(posted["status"], "posted", "{posted}");
        assert_eq!(watch_event(&mut reader)["seq"], 2);
        assert_eq!(operation(&state, post), posted);
        let replacement = json!({"action":"post","thread_id":thread_id,"body":"replacement147","notify":[],
            "operation_id":uuid::Uuid::new_v4().to_string(),"kind":"action","supersedes_seq":1});
        assert_eq!(operation(&state, replacement)["status"], "posted");
        assert_eq!(watch_event(&mut reader)["seq"], 3);
        let read = operation(
            &state,
            json!({"action":"read","thread_id":thread_id,"limit":20}),
        );
        assert_eq!(read["status"], "read", "{read}");
        let receipt = read["receipt"].as_str().unwrap();
        assert_eq!(
            operation(
                &state,
                json!({"action":"ack","thread_id":thread_id,"receipt":receipt})
            )["status"],
            "acknowledged"
        );
        assert_eq!(
            operation(
                &state,
                json!({"action":"post","thread_id":"14700000-0000-4000-8000-000000000099","body":"foreign-secret147","notify":[],"operation_id":uuid::Uuid::new_v4().to_string()})
            )["status"],
            "error"
        );
        {
            let mut st = state.lock().unwrap();
            use crate::store::threads::CreateThread;
            let members: [String; 2] = [
                "14700000-0000-4000-8000-000000000097".into(),
                "14700000-0000-4000-8000-000000000098".into(),
            ];
            st.store
                .thread_create(CreateThread {
                    actor: &members[0],
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: "foreign147",
                    thread_id: "14700000-0000-4000-8000-000000000096",
                    title: "foreign-secret147",
                    members: &members,
                    project_warnings: &[],
                    now: 1,
                    limits: &crate::threads::LIMITS,
                })
                .unwrap();
            publish_human_thread_change(&mut st, "14700000-0000-4000-8000-000000000096");
        }
        let close = json!({"action":"close","thread_id":thread_id,"operation_id":uuid::Uuid::new_v4().to_string()});
        let closed = operation(&state, close.clone());
        assert_eq!(closed["status"], "closed");
        assert_eq!(
            watch_event(&mut reader)["seq"],
            4,
            "replay/read/ACK/refus/fil étranger restent silencieux"
        );
        assert_eq!(operation(&state, close), closed);
        let rename = || {
            serde_json::from_value(json!({"type":"display_name_set","request":{"version":1,"display_name":"Nom synthétique147"}})).unwrap()
        };
        assert!(matches!(
            handle_wrapper_message("t3", rename(), &state),
            Some(DaemonToWrapper::DisplayNameResult { .. })
        ));
        assert_eq!(watch_event(&mut reader)["seq"], 5);
        handle_wrapper_message("t3", rename(), &state);
        let mut fd = libc::pollfd {
            fd: reader.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert_eq!(
            unsafe { libc::poll(&mut fd, 1, 150) },
            0,
            "renommage identique et replay de clôture muets"
        );
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            std::io::Read::read(&mut peer, &mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock,
            "aucune remise agent produite par watch"
        );
        drop(reader);
        handle.join().unwrap().unwrap();
        assert!(state.lock().unwrap().human_thread_watches.is_empty());
    }

    #[test]
    fn spec147_authority_loss_is_terminal_without_business_writes() {
        for case in ["binding", "ambiguous", "instance", "connection", "project"] {
            let (state, root, _peer) = fixture(&format!("h147{case}"));
            let before = snapshot(&state);
            let (mut reader, handle) = watch_socket(&state, &root);
            assert_eq!(watch_event(&mut reader)["status"], "ready");
            {
                let mut st = state.lock().unwrap();
                match case {
                    "binding" => {
                        st.t3_thread_bindings.insert("t3".into(), None);
                    }
                    "ambiguous" => {
                        st.t3_thread_bindings
                            .insert("duplicate".into(), Some(T3.into()));
                    }
                    "instance" => {
                        st.conn_instances.insert("t3".into(), "invalid".into());
                    }
                    "connection" => {
                        st.connections.remove("t3");
                    }
                    "project" => {
                        st.communication_projects.get_mut("t3").unwrap().2 = true;
                    }
                    _ => unreachable!(),
                }
                invalidate_human_authority(&mut st);
            }
            let event = watch_event(&mut reader);
            assert_eq!(event["status"], "error");
            assert_eq!(
                event["code"],
                if case == "project" {
                    "project_mismatch"
                } else {
                    "binding_unavailable"
                }
            );
            assert_eq!(reader.read_line(&mut String::new()).unwrap(), 0);
            drop(reader);
            handle.join().unwrap().unwrap();
            assert!(state.lock().unwrap().human_thread_watches.is_empty());
            assert_eq!(snapshot(&state), before);
        }
    }

    #[test]
    fn spec147_idle_watch_does_not_scan_authority_or_content() {
        let (state, root, _peer) = fixture("h147idle");
        {
            let mut st = state.lock().unwrap();
            for index in 0..256 {
                st.t3_thread_bindings.insert(
                    format!("idle-{index}"),
                    Some(uuid::Uuid::new_v4().to_string()),
                );
            }
        }
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["status"], "ready");
        thread::sleep(Duration::from_millis(150));
        let checks = || {
            state
                .lock()
                .unwrap()
                .human_thread_watches
                .values()
                .next()
                .unwrap()
                .queue
                .guard_checks
        };
        let before = checks();
        let started = Instant::now();
        thread::sleep(Duration::from_millis(550));
        let extra = checks() - before;
        eprintln!(
            "SPEC147 idle réel : {extra} scans sous verrou sur257bindings en {:?}",
            started.elapsed()
        );
        drop(reader);
        handle.join().unwrap().unwrap();
        assert_eq!(extra, 0);
    }

    #[test]
    fn spec147_takeover_renames_the_old_holder_and_invalidates_its_viewers() {
        use crate::store::threads::CreateThread;
        const HOLDER: &str = "14700000-0000-4000-8000-000000000093";
        let (state, root, _peer) = fixture("h147name");
        let _requester_peer = {
            let mut st = state.lock().unwrap();
            spec094_live_test_connection(&mut st, "conn-1")
        };
        {
            let st = state.lock().unwrap();
            let mut profiles = crate::agent_profile::AgentProfileStore::open(&st.db_path).unwrap();
            profiles.ensure_agent_ids([HOLDER]).unwrap();
            profiles
                .rename_display_name(HOLDER, "Nom transférable147", |_| false)
                .unwrap();
            let mut members = vec![crate::t3code::stable_uuid(T3), HOLDER.into()];
            members.sort();
            st.store
                .thread_create(CreateThread {
                    actor: &members[0],
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: "takeover147",
                    thread_id: "14700000-0000-4000-8000-000000000092",
                    title: "Ancien détenteur147",
                    members: &members,
                    project_warnings: &[],
                    now: 1,
                    limits: &crate::threads::LIMITS,
                })
                .unwrap();
        }
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["status"], "ready");
        let rename=serde_json::from_value(json!({"type":"display_name_set","request":{"version":1,"display_name":"Nom transférable147"}})).unwrap();
        let response = handle_wrapper_message("conn-1", rename, &state).unwrap();
        assert!(
            matches!(
                response,
                DaemonToWrapper::DisplayNameResult {
                    outcome: bridget_transport::protocol::DisplayNameOutcome::Applied { .. }
                }
            ),
            "{response:?}"
        );
        let mut fd = libc::pollfd {
            fd: reader.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut fd, 1, 300) };
        if ready > 0 {
            assert_eq!(watch_event(&mut reader)["status"], "changed");
        }
        drop(reader);
        handle.join().unwrap().unwrap();
        assert!(
            ready > 0,
            "un fil de l'ancien détenteur doit être invalidé aussi"
        );
    }

    #[test]
    fn spec147_runtime_authority_hooks_terminate_an_idle_watch() {
        for case in ["announce_project", "binding_fact", "unregister", "prune"] {
            let (state, root, mut peer) = fixture(&format!("h147hook{case}"));
            let (mut reader, handle) = watch_socket(&state, &root);
            assert_eq!(watch_event(&mut reader)["status"], "ready");
            let before = snapshot(&state);
            match case {
                "announce_project" => {
                    let host = state.lock().unwrap().host.clone();
                    announce_communication_project(
                        &state,
                        "t3",
                        "/nonexistent-spec147-project",
                        bridget_transport::protocol::CommunicationProjectSource::T3,
                        &host,
                        None,
                    );
                }
                "binding_fact" => {
                    handle_wrapper_message(
                        "t3",
                        WrapperToDaemon::T3ThreadBindingFact {
                            version: 1,
                            t3_thread_id: "14700000-0000-4000-8000-000000000099".into(),
                        },
                        &state,
                    );
                }
                "unregister" => {
                    handle_wrapper_message("t3", WrapperToDaemon::Unregister, &state);
                }
                "prune" => {
                    let mut st = state.lock().unwrap();
                    let name = st.presences[&st.conn_instances["t3"]].name.clone();
                    prune_agent_presence(&mut st, &name);
                }
                _ => unreachable!(),
            }
            let refusal = watch_event(&mut reader);
            assert_eq!(refusal["status"], "error");
            assert_eq!(refusal["code"], "binding_unavailable");
            assert_eq!(reader.read_line(&mut String::new()).unwrap(), 0);
            drop(reader);
            handle.join().unwrap().unwrap();
            assert_eq!(
                snapshot(&state),
                before,
                "aucune table métier changée par le suivi"
            );
            assert!(state.lock().unwrap().human_thread_watches.is_empty());
            peer.set_nonblocking(true).unwrap();
            assert_eq!(
                std::io::Read::read(&mut peer, &mut [0]).unwrap_err().kind(),
                std::io::ErrorKind::WouldBlock
            );
        }
    }

    fn cli_binary() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("bridget")
    }

    // T040 : réutilise fixture145, son vrai état et handle_connection. Le
    // shell représente seulement le parent fournisseur : aucun modèle.
    struct Spec147McpChild {
        child: std::process::Child,
        input: Option<std::process::ChildStdin>,
        output: std::process::ChildStdout,
        mcp_pid: u32,
        mcp_birth: u64,
        deadline: Instant,
    }

    impl Spec147McpChild {
        fn request(&mut self, request: Value) -> Value {
            writeln!(self.input.as_mut().unwrap(), "{request}").unwrap();
            self.response()
        }

        fn response(&mut self) -> Value {
            use std::io::Read;
            let deadline = (Instant::now() + Duration::from_secs(2)).min(self.deadline);
            let mut line = Vec::new();
            loop {
                let mut bytes = [0_u8; 4096];
                match self.output.read(&mut bytes) {
                    Ok(0) => panic!("MCP privé a fermé stdout avant sa réponse"),
                    Ok(count) => {
                        line.extend_from_slice(&bytes[..count]);
                        // Borne du banc : le catalogue complet tools/list pèse
                        // environ 88 Kio depuis la session 228 (schémas complets).
                        assert!(line.len() <= 256 * 1024, "réponse MCP non bornée");
                        if line.ends_with(b"\n") {
                            return serde_json::from_slice(&line).unwrap();
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(error) => panic!("lecture MCP privée : {error}"),
                }
                assert!(Instant::now() < deadline, "réponse MCP absente après2s");
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    impl Drop for Spec147McpChild {
        fn drop(&mut self) {
            drop(self.input.take()); // EOF arrête MCP puis son shell parent.
            let deadline = Instant::now() + Duration::from_secs(2);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            // Seulement nos deux PID observés ; aucune cible de groupe et
            // aucun SIGKILL. Vérifier commande/naissance avant chaque TERM.
            for (pid, birth) in [
                (self.mcp_pid, Some(self.mcp_birth)),
                (
                    if self.child.id() == self.mcp_pid {
                        0
                    } else {
                        self.child.id()
                    },
                    None,
                ),
            ] {
                if pid == 0 {
                    continue;
                }
                if birth.is_some_and(|expected| {
                    crate::managed_process::process_birth(pid).ok() != Some(expected)
                }) {
                    continue;
                }
                let Ok(observed) = std::process::Command::new("/bin/ps")
                    .args(["-p", &pid.to_string(), "-o", "command="])
                    .output()
                else {
                    continue;
                };
                let command = String::from_utf8_lossy(&observed.stdout);
                let owned = if pid == self.mcp_pid {
                    command.contains(cli_binary().to_string_lossy().as_ref())
                        && crate::mcp_identity::process_parent(pid).ok()
                            == Some(if self.child.id() == pid {
                                std::process::id()
                            } else {
                                self.child.id()
                            })
                } else {
                    command.contains("BRIDGET_147_TEST_BINARY")
                        && crate::mcp_identity::process_parent(pid).ok() == Some(std::process::id())
                };
                if owned && !command.to_ascii_lowercase().contains("firefox") {
                    unsafe {
                        libc::kill(pid as libc::pid_t, libc::SIGTERM);
                    }
                    thread::sleep(Duration::from_secs(3));
                }
            }
            let _ = self.child.try_wait();
        }
    }

    fn spec147_native_mcp_case(mode: &str) -> Value {
        use crate::store::threads::{CreateThread, NotifySpec, PostEntry, ThreadTxOutcome};
        use std::sync::atomic::{AtomicBool, Ordering};
        const B: &str = "89000000-0000-4000-8000-000000000102";
        const C: &str = "14700000-0000-4000-8000-000000000003";
        const TA: &str = "14700000-aaaa-4aaa-8aaa-000000000001";
        const TB: &str = "14700000-bbbb-4bbb-8bbb-000000000002";
        let (state, _root, mut peer_a) = fixture(&format!("h147mcp-{mode}"));
        let mut peer_b = spec094_live_test_connection(&mut state.lock().unwrap(), "conn-1");
        let directory = std::fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!("h147mcp-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let _cleanup = FixtureRoot(directory.clone());
        let socket = directory.join("s");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let a = crate::t3code::stable_uuid(T3);
        let (instance_a, credential_a, credential_b) = {
            let st = state.lock().unwrap();
            for (actor, id, title) in [
                (&a, TA, "CANARY147_NATIVE_A"),
                (&B.to_string(), TB, "CANARY147_NATIVE_B"),
            ] {
                assert!(matches!(
                    st.store
                        .thread_create(CreateThread {
                            actor,
                            operation_id: id,
                            canonical_hash: id,
                            thread_id: id,
                            title,
                            members: &[actor.clone(), C.into()],
                            project_warnings: &[],
                            now: 1,
                            limits: &crate::threads::LIMITS
                        })
                        .unwrap(),
                    ThreadTxOutcome::Done(_)
                ));
            }
            (
                st.conn_instances["t3"].clone(),
                st.identity_credentials["t3"].clone(),
                st.identity_credentials["conn-1"].clone(),
            )
        };
        if mode == "members" {
            let st = state.lock().unwrap();
            for seq in 1..=137 {
                let body = format!("CANARY147 historique {seq} É🦀\r\n  exact\n\n");
                let notify = NotifySpec::Targets(if seq <= 2 { vec![C.into()] } else { vec![] });
                let kind = match seq {
                    1 | 2 => Some(bridget_transport::protocol::ThreadEntryKind::Action),
                    3 => None,
                    _ => Some(bridget_transport::protocol::ThreadEntryKind::History),
                };
                assert!(matches!(
                    st.store
                        .thread_post(PostEntry {
                            actor: &a,
                            operation_id: &uuid::Uuid::new_v4().to_string(),
                            canonical_hash: &format!("members-native-{seq}"),
                            thread_id: TA,
                            message_id: &uuid::Uuid::new_v4().to_string(),
                            body: &body,
                            notify: &notify,
                            notices: &[],
                            project_warnings: &[],
                            reply_to_seq: None,
                            ack_receipt: None,
                            kind,
                            supersedes_seq: (seq == 2).then_some(1),
                            now: seq,
                            limits: &crate::threads::LIMITS,
                        })
                        .unwrap(),
                    ThreadTxOutcome::Done(_)
                ));
            }
        }
        crate::mcp_identity::save_credential(
            &directory,
            &a,
            &instance_a,
            if mode == "forged" {
                credential_b.clone()
            } else {
                credential_a
            },
        )
        .unwrap();
        crate::mcp_identity::save_credential(&directory, B, "instance-1", credential_b).unwrap();
        let name_a = directory.join("agent-names/native-a");
        let name_b = directory.join("agent-names/native-b");
        bridget_transport::fsutil::write_private_file_atomic(&name_a, a.as_bytes()).unwrap();
        bridget_transport::fsutil::write_private_file_atomic(&name_b, B.as_bytes()).unwrap();
        if matches!(mode, "closest" | "bad-with-ancestor" | "members") {
            crate::mcp_identity::write_marker(
                &directory.join("agent-pids"),
                std::process::id(),
                crate::managed_process::process_birth(std::process::id()).unwrap(),
                "instance-1",
                &name_b,
            )
            .unwrap();
        }
        let before = snapshot(&state);
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let serving = state.clone();
        let server = thread::spawn(move || {
            let mut handles = Vec::new();
            while !stopping.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        // Même mode bloquant que l'accept loop de production.
                        // macOS hérite O_NONBLOCK du listener ; une grande page
                        // ne doit pas devenir une trame JSONL tronquée.
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        let st = serving.clone();
                        handles.push(thread::spawn(move || {
                            handle_connection(stream, st).map_err(|error| error.to_string())
                        }));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(error) => panic!("listener MCP privé : {error}"),
                }
            }
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
        });
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut child = std::process::Command::new("/bin/sh")
                .args([
                    "-c",
                    "\"$BRIDGET_147_TEST_BINARY\" mcp; result=$?; exit \"$result\"",
                ])
                .env_clear()
                .env("HOME", &directory)
                .env("BRIDGET_HOME", &directory)
                .env("BRIDGET_SOCKET", &socket)
                .env("PATH", "/usr/bin:/bin")
                .env("BRIDGET_147_TEST_BINARY", cli_binary())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let shell_pid = child.id();
            let output = child.stdout.take().unwrap();
            let mut process = Spec147McpChild {
                input: child.stdin.take(),
                output,
                child,
                mcp_pid: 0,
                mcp_birth: 0,
                deadline: Instant::now() + Duration::from_secs(10),
            };
            let deadline = Instant::now() + Duration::from_secs(2);
            let mcp_pid = loop {
                let observed = std::process::Command::new("/bin/ps")
                    .args(["-axo", "pid=,ppid=,command="])
                    .output()
                    .unwrap();
                let found = String::from_utf8_lossy(&observed.stdout)
                    .lines()
                    .find_map(|line| {
                        let mut fields = line.split_whitespace();
                        let pid = fields.next()?.parse::<u32>().ok()?;
                        let parent = fields.next()?.parse::<u32>().ok()?;
                        (parent == shell_pid
                            && line.contains(cli_binary().to_string_lossy().as_ref())
                            && line.ends_with(" mcp"))
                        .then_some(pid)
                    });
                if let Some(pid) = found {
                    break pid;
                }
                assert!(Instant::now() < deadline, "enfant MCP OS absent");
                thread::sleep(Duration::from_millis(10));
            };
            unsafe {
                let flags = libc::fcntl(process.output.as_raw_fd(), libc::F_GETFL);
                assert!(flags >= 0);
                assert_eq!(
                    libc::fcntl(
                        process.output.as_raw_fd(),
                        libc::F_SETFL,
                        flags | libc::O_NONBLOCK
                    ),
                    0
                );
            }
            process.mcp_pid = mcp_pid;
            process.mcp_birth = crate::managed_process::process_birth(mcp_pid).unwrap();
            if mode != "absent" {
                let birth = crate::managed_process::process_birth(shell_pid).unwrap();
                crate::mcp_identity::write_marker(
                    &directory.join("agent-pids"),
                    shell_pid,
                    if mode.starts_with("bad-") {
                        birth + 1
                    } else {
                        birth
                    },
                    &instance_a,
                    &name_a,
                )
                .unwrap();
            }
            let init =
                process.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
            assert!(init.get("result").is_some(), "{init}");
            writeln!(
                process.input.as_mut().unwrap(),
                "{}",
                json!({"jsonrpc":"2.0","method":"notifications/initialized"})
            )
            .unwrap();
            let catalogue = process.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
            assert!(
                catalogue["result"]["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|tool| tool["name"] == "bridget_thread")
            );
            let mut observed = process.request(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bridget_thread","arguments":{"action":"list"}}}));
            if mode == "members" {
                let schema = &catalogue["result"]["tools"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|tool| tool["name"] == "bridget_thread")
                    .unwrap()["inputSchema"];
                assert!(schema.to_string().contains("add_members"));
                let key = uuid::Uuid::new_v4().to_string();
                let request = json!({"action":"add_members","thread_id":TA,"members":[B.to_uppercase(),B],"operation_id":key.to_uppercase(),"cross_project_reason":"Recette native147 explicitement autorisée"});
                observed = process.request(json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"bridget_thread","arguments":request}}));
                let added = observed["result"]["structuredContent"].clone();
                assert_eq!(added["status"], "members_added", "{observed}");
                let replay = process.request(json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"bridget_thread","arguments":{"action":"add_members","thread_id":TA,"members":[B],"operation_id":key,"cross_project_reason":"Recette native147 explicitement autorisée"}}}));
                assert_eq!(replay["result"]["structuredContent"], added);
                let cli_deadline = process.deadline;
                let cli = |args: &[&str]| {
                    let mut child = std::process::Command::new(cli_binary())
                        .args(args)
                        .env_clear()
                        .env("HOME", &directory)
                        .env("BRIDGET_HOME", &directory)
                        .env("BRIDGET_SOCKET", &socket)
                        .env("PATH", "/usr/bin:/bin")
                        .stdin(Stdio::null())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                        .unwrap();
                    let pid = child.id();
                    let output = child.stdout.take().unwrap();
                    let mut owned = Spec147McpChild {
                        child,
                        input: None,
                        output,
                        mcp_pid: pid,
                        // Un CLI court peut déjà être sorti (ESRCH sous charge) :
                        // Drop le constate par try_wait, et 0 n'autorise aucun TERM.
                        mcp_birth: crate::managed_process::process_birth(pid).unwrap_or(0),
                        deadline: cli_deadline,
                    };
                    unsafe {
                        let flags = libc::fcntl(owned.output.as_raw_fd(), libc::F_GETFL);
                        assert!(flags >= 0);
                        assert_eq!(
                            libc::fcntl(
                                owned.output.as_raw_fd(),
                                libc::F_SETFL,
                                flags | libc::O_NONBLOCK
                            ),
                            0
                        );
                    }
                    let result = owned.response();
                    loop {
                        if let Some(status) = owned.child.try_wait().unwrap() {
                            assert!(status.success(), "CLI {args:?} : {status}, {result}");
                            break;
                        }
                        assert!(
                            Instant::now() < cli_deadline,
                            "CLI privé n'a pas terminé dans son budget"
                        );
                        thread::sleep(Duration::from_millis(5));
                    }
                    drop(owned);
                    assert!(
                        crate::managed_process::process_birth(pid).is_err(),
                        "CLI privé encore vivant"
                    );
                    result
                };
                let key_b = uuid::Uuid::new_v4().to_string();
                let add_b = [
                    "thread",
                    "add-members",
                    TB,
                    "--member",
                    &a,
                    "--id",
                    &key_b,
                    "--cross-project-reason",
                    "Recette native147 explicitement autorisée",
                ];
                let added_b = cli(&add_b);
                assert_eq!(added_b["status"], "members_added", "{added_b}");
                assert_eq!(cli(&add_b), added_b, "rejeu CLI exact");
                assert_eq!(
                    cli(&["thread", "show", TA])["creator_id"],
                    a,
                    "CLI identifié B lit désormais A"
                );
                let after_add = snapshot(&state);
                for (table, rows) in &before {
                    if !matches!(table.as_str(), "discussion_members" | "thread_operations") {
                        assert_eq!(
                            &after_add.iter().find(|(name, _)| name == table).unwrap().1,
                            rows,
                            "ajout ne touche pas les anciennes entrées, lectures, wakes ou timestamps : {table}"
                        );
                    }
                }
                let mut through = 0;
                let mut pages = 0;
                loop {
                    let page = cli(&["thread", "read", TA, "--limit", "50"]);
                    assert_eq!(page["base_seq"], through);
                    assert_eq!(page["snapshot_seq"], 137);
                    for entry in page["entries"].as_array().unwrap() {
                        let seq = entry["seq"].as_u64().unwrap();
                        if matches!(seq, 2 | 3) {
                            assert_eq!(
                                entry["body"],
                                format!("CANARY147 historique {seq} É🦀\r\n  exact\n\n")
                            );
                        } else {
                            assert!(
                                entry.get("body").is_none(),
                                "history/superseded restent références"
                            );
                        }
                        if seq == 1 {
                            assert_eq!(entry["superseded_by_seq"], 2);
                        }
                    }
                    through = page["through_seq"].as_u64().unwrap();
                    let ack = cli(&["thread", "ack", TA, page["receipt"].as_str().unwrap()]);
                    assert_eq!(ack["status"], "acknowledged");
                    pages += 1;
                    if page["has_more"] == false {
                        break;
                    }
                    assert!(pages < 4, "pagination bornée sur 137 entrées");
                }
                assert_eq!((pages, through), (3, 137));
                for from in [1, 51, 101] {
                    let history = cli(&[
                        "thread",
                        "history",
                        TA,
                        "--from-seq",
                        &from.to_string(),
                        "--to-seq",
                        "137",
                        "--limit",
                        "50",
                    ]);
                    for entry in history["entries"].as_array().unwrap() {
                        let seq = entry["seq"].as_u64().unwrap();
                        assert_eq!(
                            entry["body"],
                            format!("CANARY147 historique {seq} É🦀\r\n  exact\n\n"),
                            "historique exact pour nouveau membre"
                        );
                    }
                }
                let st = state.lock().unwrap();
                let all = NotifySpec::All;
                let future = st
                    .store
                    .thread_post(PostEntry {
                        actor: &a,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: "members-future-all",
                        thread_id: TA,
                        message_id: &uuid::Uuid::new_v4().to_string(),
                        body: "nouvelle action",
                        notify: &all,
                        notices: &[],
                        project_warnings: &[],
                        reply_to_seq: None,
                        ack_receipt: None,
                        kind: Some(bridget_transport::protocol::ThreadEntryKind::Action),
                        supersedes_seq: None,
                        now: 138,
                        limits: &crate::threads::LIMITS,
                    })
                    .unwrap();
                match future {
                    ThreadTxOutcome::Done(result) => {
                        assert_eq!(result["targets"], json!([C, B]))
                    }
                    other => panic!("futur notify all : {other:?}"),
                }
            }
            if matches!(mode, "closest" | "bad-with-ancestor") {
                let (id, actor) = if mode == "closest" {
                    (TA, a.as_str())
                } else {
                    (TB, B)
                };
                let shown = process.request(json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"bridget_thread","arguments":{"action":"show","thread_id":id}}}));
                assert_eq!(
                    shown["result"]["structuredContent"]["status"], "shown",
                    "{shown}"
                );
                assert_eq!(shown["result"]["structuredContent"]["creator_id"], actor);
            }
            eprintln!(
                "SPEC147 native-MCP mode={mode} namespace={} shell={shell_pid} child={mcp_pid} code={} status={}",
                directory.display(),
                observed["result"]["code"],
                observed["result"]["structuredContent"]["status"]
            );
            drop(process);
            assert!(
                crate::managed_process::process_birth(shell_pid).is_err(),
                "shell privé encore vivant"
            );
            assert!(
                crate::managed_process::process_birth(mcp_pid).is_err(),
                "MCP privé encore vivant"
            );
            observed
        }));
        stop.store(true, Ordering::Release);
        server.join().unwrap();
        if mode != "members" {
            assert_eq!(
                snapshot(&state),
                before,
                "MCP list/connect/aux/EOF ne modifie aucune table"
            );
        }
        use std::io::Read;
        for peer in [&mut peer_a, &mut peer_b] {
            peer.set_nonblocking(true).unwrap();
            let mut byte = [0];
            assert!(
                matches!(peer.read(&mut byte), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "aucun Deliver ni autre trame primaire attendu"
            );
        }
        match observed {
            Ok(value) => value,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    #[test]
    fn spec147_members_native_cli_mcp_paginated_history_and_no_old_replay() {
        let observed = spec147_native_mcp_case("members");
        assert_eq!(
            observed["result"]["structuredContent"]["status"],
            "members_added"
        );
    }

    #[test]
    fn spec147_native_mcp_closest_valid_parent_is_the_exact_identity() {
        let observed = spec147_native_mcp_case("closest");
        assert_ne!(observed["result"]["isError"], true, "{observed}");
        let payload = &observed["result"]["structuredContent"];
        assert_eq!(payload["status"], "listed");
        assert_eq!(payload["threads"].as_array().unwrap().len(), 1);
        assert_eq!(payload["threads"][0]["title"], "CANARY147_NATIVE_A");
    }

    #[test]
    fn spec147_native_mcp_absent_bad_birth_and_forged_credential_refuse() {
        for mode in ["absent", "bad-only", "forged"] {
            let observed = spec147_native_mcp_case(mode);
            assert_eq!(observed["result"]["isError"], true, "{mode}: {observed}");
            assert_eq!(
                observed["result"]["code"],
                if mode == "forged" {
                    "auxiliary_identity_unproven"
                } else {
                    "identity_not_found"
                }
            );
            assert!(!observed.to_string().contains("CANARY147_NATIVE_"));
        }
    }

    #[test]
    fn spec147_native_mcp_recycled_marker_keeps_the_existing_valid_ancestor_rule() {
        let observed = spec147_native_mcp_case("bad-with-ancestor");
        assert_ne!(observed["result"]["isError"], true, "{observed}");
        let payload = &observed["result"]["structuredContent"];
        assert_eq!(payload["status"], "listed");
        assert_eq!(payload["threads"].as_array().unwrap().len(), 1);
        assert_eq!(payload["threads"][0]["title"], "CANARY147_NATIVE_B");
    }

    #[test]
    #[ignore = "interop T3 réelle explicite avec runner Node isolé"]
    fn spec147_t3_service_real_cli_interop() {
        let program = std::env::var("BRIDGET_SPEC147_T3_PROGRAM")
            .expect("exécutable Node exact fourni par la laneRPC");
        let script = std::env::var("BRIDGET_SPEC147_T3_SCRIPT")
            .expect("runner Reader isolé exact fourni par la laneRPC");
        let (state, root, mut peer) = fixture("h147rpc");
        let created = operation(&state, create_action());
        assert_eq!(created["status"], "created");
        let thread_id = created["thread_id"].as_str().unwrap().to_string();
        let directory =
            std::env::temp_dir().join(format!("h147rpc-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let socket = directory.join("d.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        eprintln!(
            "SPEC147 interop namespace={} socket={} CLI={} root={} T3={T3}",
            directory.display(),
            socket.display(),
            cli_binary().display(),
            root
        );
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let serving = state.clone();
        let server = thread::spawn(move || {
            let mut handles = Vec::new();
            while !stopping.load(std::sync::atomic::Ordering::Acquire) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        let st = serving.clone();
                        handles.push(thread::spawn(move || {
                            handle_connection(stream, st).map_err(|e| e.to_string())
                        }));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(error) => panic!("listener privé147 : {error}"),
                }
            }
            for handle in handles {
                handle.join().unwrap().unwrap();
            }
        });
        const BODY: &str = "CANARY147 interop exact é🦀\nPreuve synthétique.";
        let recovery = std::env::var("BRIDGET_147_RECOVERY").as_deref() == Ok("1");
        let marker = directory.join("ready.txt");
        let mut child = std::process::Command::new(&program)
            .arg(&script)
            .env("BRIDGET_HOME", &directory)
            .env("BRIDGET_SOCKET", &socket)
            .env("BRIDGET_147_BINARY", cli_binary())
            .env("BRIDGET_147_ROOT", &root)
            .env("BRIDGET_147_T3_THREAD", T3)
            .env("BRIDGET_147_EXPECT_BODY", BODY)
            .env("BRIDGET_147_EXPECT_THREAD", &thread_id)
            .env("BRIDGET_147_READY_MARKER", &marker)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut interrupted = false;
        let subscribed = loop {
            let ready_count = std::fs::read_to_string(&marker)
                .unwrap_or_default()
                .lines()
                .filter(|line| *line == "ready")
                .count();
            if recovery && ready_count == 1 && !interrupted {
                let writer = {
                    let st = state.lock().unwrap();
                    let conn = st.human_thread_watches.keys().next().unwrap();
                    st.connections[conn].clone()
                };
                let socket = writer.lock().unwrap().get_ref().try_clone().unwrap();
                socket.shutdown(std::net::Shutdown::Both).unwrap();
                interrupted = true;
            }
            if !state.lock().unwrap().human_thread_watches.is_empty()
                && (!recovery || ready_count >= 2)
            {
                break true;
            }
            if child.try_wait().unwrap().is_some() || Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(10));
        };
        if subscribed {
            let posted = operation(
                &state,
                json!({"action":"post","thread_id":thread_id,"body":BODY,"notify":[],"kind":"history","operation_id":uuid::Uuid::new_v4().to_string()}),
            );
            assert_eq!(posted["status"], "posted");
        }
        let after_mutation = snapshot(&state);
        let output = child.wait_with_output().unwrap(); // Runner Effect borné, scope libère ses CLI.
        stop.store(true, std::sync::atomic::Ordering::Release);
        server.join().unwrap();
        eprintln!(
            "SPEC147 Reader réel stdout : {}",
            String::from_utf8_lossy(&output.stdout)
        );
        eprintln!(
            "SPEC147 Reader réel stderr : {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(subscribed, "Node doit négocier le vrai watch humain147");
        assert!(
            output.status.success(),
            "interop Node échouée : {}",
            output.status
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(if recovery {
                "SPEC147_INTEROP_RUNTIME_RECOVERY_OK"
            } else {
                "SPEC147_INTEROP_READER_OK"
            })
        );
        if recovery {
            assert!(interrupted);
            assert_eq!(
                std::fs::read_to_string(&marker)
                    .unwrap()
                    .lines()
                    .collect::<Vec<_>>(),
                ["ready", "masked", "ready", "changed"]
            );
        }
        assert_eq!(
            snapshot(&state),
            after_mutation,
            "Reader/watch/inspect ne changent aucune table"
        );
        assert!(state.lock().unwrap().human_thread_watches.is_empty());
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            std::io::Read::read(&mut peer, &mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn cli_event(reader: &mut BufReader<std::process::ChildStdout>) -> Value {
        let mut fd = libc::pollfd {
            fd: reader.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut fd, 1, 6000) } > 0,
            "CLI prêt sous6s"
        );
        let mut line = String::new();
        assert!(reader.read_line(&mut line).unwrap() > 0);
        let event: bridget_transport::protocol::HumanThreadWatchEvent =
            serde_json::from_str(&line).unwrap();
        serde_json::to_value(event).unwrap()
    }

    #[test]
    fn spec147_cli_real_watch_mutations_replay_idle_and_cancel_ten_cycles() {
        let (state, root, mut peer) = fixture("h147cli");
        let directory =
            std::env::temp_dir().join(format!("h147cli-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let socket = directory.join("d.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let mut generations = HashSet::new();
        for cycle in 0..10 {
            let before = snapshot(&state);
            let server = state.clone();
            let incoming = listener.try_clone().unwrap();
            let handle = thread::spawn(move || {
                handle_connection(incoming.accept().unwrap().0, server).map_err(|e| e.to_string())
            });
            let mut child = std::process::Command::new(cli_binary())
                .env("BRIDGET_HOME", &directory)
                .env("BRIDGET_SOCKET", &socket)
                .args([
                    "thread",
                    "watch",
                    "--t3-thread",
                    T3,
                    "--project-root",
                    &root,
                    "--json",
                ])
                .stdout(Stdio::piped())
                .spawn()
                .expect("construire bridget147 avant ce test");
            let mut reader = BufReader::new(child.stdout.take().unwrap());
            let ready = cli_event(&mut reader);
            assert_eq!(ready["status"], "ready");
            assert_eq!(ready["seq"], 0);
            assert!(generations.insert(ready["generation"].as_str().unwrap().to_owned()));
            assert_eq!(
                snapshot(&state),
                before,
                "watch/hello ne modifient aucune table métier"
            );
            if cycle == 0 {
                // Au-delà du budget unary historique, aucun contenu n'est relu.
                thread::sleep(Duration::from_millis(10_100));
                assert_eq!(state.lock().unwrap().human_thread_watches.len(), 1);
                let create = create_action();
                let created = operation(&state, create.clone());
                assert_eq!(created["status"], "created");
                let changed = cli_event(&mut reader);
                assert_eq!(changed["status"], "changed");
                assert_eq!(changed["seq"], 1);
                assert_eq!(changed.as_object().unwrap().len(), 4);
                assert_eq!(operation(&state, create), created);
                let thread_id = created["thread_id"].as_str().unwrap();
                let post = json!({"action":"post","thread_id":thread_id,"body":"CANARY147 exact é🦀","notify":[],"operation_id":uuid::Uuid::new_v4().to_string()});
                let posted = operation(&state, post.clone());
                assert_eq!(posted["status"], "posted");
                assert_eq!(cli_event(&mut reader)["seq"], 2);
                assert_eq!(operation(&state, post), posted);
                let close = json!({"action":"close","thread_id":thread_id,"operation_id":uuid::Uuid::new_v4().to_string()});
                assert_eq!(operation(&state, close)["status"], "closed");
                assert_eq!(cli_event(&mut reader)["seq"], 3, "replay muet sur vrai CLI");
            }
            drop(reader); // Fermer le pipe doit terminer même sans prochain signal.
            let deadline = Instant::now() + Duration::from_secs(3);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let writers: Vec<_> = {
                        let st = state.lock().unwrap();
                        st.human_thread_watches
                            .keys()
                            .filter_map(|conn| st.connections.get(conn).cloned())
                            .collect()
                    };
                    for writer in writers {
                        let _ = writer
                            .lock()
                            .unwrap()
                            .get_ref()
                            .shutdown(std::net::Shutdown::Both);
                    }
                    let status = child.wait().unwrap();
                    panic!("pipe fermé n'a pas libéré le CLI : {status}");
                }
                thread::sleep(Duration::from_millis(10));
            };
            assert_eq!(status.code(), Some(0));
            handle.join().unwrap().unwrap();
            assert!(state.lock().unwrap().human_thread_watches.is_empty());
            if cycle > 0 {
                assert_eq!(snapshot(&state), before);
            }
        }
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            std::io::Read::read(&mut peer, &mut [0]).unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
        drop(listener);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn spec147_authority_is_rechecked_after_waiting_for_the_writer() {
        let (state, root, _peer) = fixture("h147emit");
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["status"], "ready");
        let writer = {
            let st = state.lock().unwrap();
            let conn = st.human_thread_watches.keys().next().unwrap();
            st.connections[conn].clone()
        };
        let held = writer.lock().unwrap();
        assert_eq!(operation(&state, create_action())["status"], "created");
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            if state
                .lock()
                .unwrap()
                .human_thread_watches
                .values()
                .all(|watch| watch.queue.pending.is_empty())
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "le worker doit avoir sorti changed de la file"
            );
            thread::sleep(Duration::from_millis(1));
        }
        handle_wrapper_message("t3", WrapperToDaemon::Unregister, &state);
        drop(held);
        assert_eq!(
            watch_event(&mut reader)["status"],
            "error",
            "aucun changed après révocation pendant attente writer"
        );
        assert_eq!(reader.read_line(&mut String::new()).unwrap(), 0);
        drop(reader);
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn spec147_emit_does_not_hold_writer_while_waiting_for_state() {
        let (state, root, _peer) = fixture("h147locks");
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["status"], "ready");
        let writer = {
            let st = state.lock().unwrap();
            let conn = st.human_thread_watches.keys().next().unwrap();
            st.connections[conn].clone()
        };
        let held_writer = writer.lock().unwrap();
        operation(&state, create_action());
        let deadline = Instant::now() + Duration::from_millis(500);
        while state
            .lock()
            .unwrap()
            .human_thread_watches
            .values()
            .any(|watch| !watch.queue.pending.is_empty())
        {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
        let held_state = state.lock().unwrap();
        drop(held_writer);
        thread::sleep(Duration::from_millis(30));
        let deadline = Instant::now() + Duration::from_millis(300);
        let writer_released = loop {
            if writer.try_lock().is_ok() {
                break true;
            }
            if Instant::now() >= deadline {
                break false;
            }
            thread::sleep(Duration::from_millis(1));
        };
        drop(held_state);
        assert_eq!(watch_event(&mut reader)["status"], "changed");
        drop(reader);
        handle.join().unwrap().unwrap();
        assert!(
            writer_released,
            "un contrôle d'autorité ne bloque jamais state avec writer détenu"
        );
    }

    #[test]
    fn spec147_slow_writer_does_not_hold_mutation_lock() {
        let (state, root, _peer) = fixture("h147slow");
        let (mut reader, handle) = watch_socket(&state, &root);
        assert_eq!(watch_event(&mut reader)["status"], "ready");
        let writer = {
            let st = state.lock().unwrap();
            let conn = st.human_thread_watches.keys().next().unwrap();
            st.connections[conn].clone()
        };
        let held = writer.lock().unwrap();
        let started = Instant::now();
        let created = operation(&state, create_action());
        assert_eq!(created["status"], "created");
        assert!(started.elapsed() < Duration::from_millis(500));
        assert_eq!(
            reader.read_line(&mut String::new()).unwrap(),
            0,
            "sortie bornée fermée même si writer verrouillé"
        );
        drop(held);
        drop(reader);
        handle.join().unwrap().unwrap();
        assert!(state.lock().unwrap().human_thread_watches.is_empty());
    }

    #[test]
    fn spec145_dormant_binding_project_and_disconnect_are_live_facts() {
        let (state, root, _peer) = fixture("h145b");
        let before = snapshot(&state);
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["status"],
            "listed"
        );
        assert_eq!(
            call(&state, "/nonexistent-project", json!({"action":"list"}))["result"]["code"],
            "project_mismatch"
        );
        {
            let mut st = state.lock().unwrap();
            st.conn_instances
                .insert("t3".into(), "wrong-instance".into());
        }
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        state.lock().unwrap().conn_instances.insert(
            "t3".into(),
            crate::t3code::stable_uuid(&format!("instance:{T3}")),
        );
        state.lock().unwrap().connections.remove("t3");
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        state.lock().unwrap().mark_unreachable("t3");
        assert!(!state.lock().unwrap().t3_thread_bindings.contains_key("t3"));
        assert!(!state.lock().unwrap().t3_project_connections.contains("t3"));
        assert_eq!(
            snapshot(&state),
            before,
            "toutes les tables métier restent identiques"
        );
    }

    #[test]
    fn spec145_binding_ambiguity_auxiliary_and_conflicting_fact_fail_closed() {
        let (state, root, _peer) = fixture("h145a");
        state
            .lock()
            .unwrap()
            .t3_thread_bindings
            .insert("duplicate".into(), Some(T3.into()));
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        state.lock().unwrap().t3_thread_bindings.remove("duplicate");
        state
            .lock()
            .unwrap()
            .auxiliary_connections
            .insert("t3".into());
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        state.lock().unwrap().auxiliary_connections.remove("t3");
        state
            .lock()
            .unwrap()
            .conn_names
            .insert("t3".into(), "14500000-0000-4000-8000-000000000099".into());
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        state
            .lock()
            .unwrap()
            .conn_names
            .insert("t3".into(), crate::t3code::stable_uuid(T3));
        let fact=serde_json::from_value(json!({"type":"T3ThreadBindingFact","version":1,"t3_thread_id":"14500000-0000-4000-8000-000000000099"})).unwrap();
        assert!(matches!(
            handle_wrapper_message("t3", fact, &state),
            Some(DaemonToWrapper::Nack { .. })
        ));
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
    }

    #[test]
    fn spec145_reconnect_requires_fresh_primary_binding() {
        let (state, root, _old_peer) = fixture("h145r");
        let host = state.lock().unwrap().host.clone();
        let _fresh_peer = {
            let mut st = state.lock().unwrap();
            st.connections.remove("t3");
            st.router.unregister_by_conn("t3");
            st.mark_unreachable("t3");
            st.conn_names.remove("t3");
            let peer = spec094_live_test_connection(&mut st, "t3-new");
            let registered = handle_register_with_channel(
                "t3-new",
                2,
                "fixture".into(),
                crate::t3code::stable_uuid(T3),
                Some(host.clone()),
                Some("t3code".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Cli),
                None,
                Some("test".into()),
                Some(crate::t3code::stable_uuid(&format!("instance:{T3}"))),
                None,
                false,
                Some(false),
                &mut st,
                false,
            );
            assert!(
                matches!(registered, DaemonToWrapper::Registered { .. }),
                "réponse réelle reconnexion: {registered:?}"
            );
            peer
        };
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable"
        );
        announce_communication_project(
            &state,
            "t3-new",
            &root,
            bridget_transport::protocol::CommunicationProjectSource::T3,
            &host,
            None,
        );
        let fact = serde_json::from_value(
            json!({"type":"T3ThreadBindingFact","version":1,"t3_thread_id":T3}),
        )
        .unwrap();
        assert!(handle_wrapper_message("t3-new", fact, &state).is_none());
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["status"],
            "listed"
        );
        assert!(!state.lock().unwrap().t3_thread_bindings.contains_key("t3"));
    }

    #[test]
    fn spec145_git_canonical_root_worktree_alias_and_source_are_attested() {
        let (state, root, _peer) = fixture("h145g");
        let canonical = state.lock().unwrap().communication_projects["t3"]
            .0
            .clone()
            .unwrap()
            .root;
        assert_ne!(
            root, canonical,
            "workspace et gitcommon sont deux chemins distincts"
        );
        let primary = std::path::Path::new(&canonical).parent().unwrap();
        assert_eq!(
            call(&state, primary.to_str().unwrap(), json!({"action":"list"}))["result"]["status"],
            "listed"
        );
        let alias =
            std::env::temp_dir().join(format!("h145-alias-{}", uuid::Uuid::new_v4().simple()));
        std::os::unix::fs::symlink(&root, &alias).unwrap();
        assert_eq!(
            call(&state, alias.to_str().unwrap(), json!({"action":"list"}))["result"]["status"],
            "listed"
        );
        std::fs::remove_file(alias).unwrap();
        let unrelated =
            std::env::temp_dir().join(format!("h145-git-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&unrelated).unwrap();
        let _cleanup = FixtureRoot(unrelated.clone());
        assert!(
            std::process::Command::new("git")
                .args(["init", "--quiet"])
                .arg(&unrelated)
                .status()
                .unwrap()
                .success()
        );
        assert_eq!(
            call(
                &state,
                unrelated.to_str().unwrap(),
                json!({"action":"list"})
            )["result"]["code"],
            "project_mismatch"
        );
        let host = state.lock().unwrap().host.clone();
        announce_communication_project(
            &state,
            "t3",
            &root,
            bridget_transport::protocol::CommunicationProjectSource::Git,
            &host,
            None,
        );
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable",
            "un fait Git ne prouve pas une source T3"
        );
        announce_communication_project(
            &state,
            "t3",
            &root,
            bridget_transport::protocol::CommunicationProjectSource::T3,
            &host,
            None,
        );
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable",
            "restaurer projet ne ressuscite pas binding"
        );
        let fact = serde_json::from_value(
            json!({"type":"T3ThreadBindingFact","version":1,"t3_thread_id":T3}),
        )
        .unwrap();
        assert!(handle_wrapper_message("t3", fact, &state).is_none());
        announce_communication_project(
            &state,
            "t3",
            unrelated.to_str().unwrap(),
            bridget_transport::protocol::CommunicationProjectSource::T3,
            &host,
            None,
        );
        assert!(state.lock().unwrap().communication_projects["t3"].2);
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "binding_unavailable",
            "conflit retire autorité"
        );
    }

    #[test]
    fn spec145_real_client_lifecycle_and_refusal_do_not_mutate_business_tables() {
        let (state, root, mut peer) = fixture("h145l");
        let directory =
            std::env::temp_dir().join(format!("h145-s-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .unwrap();
        let _cleanup = FixtureRoot(directory.clone());
        let socket = directory.join("s");
        let listener = UnixListener::bind(&socket).unwrap();
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
        let before = snapshot(&state);
        let presence_before = {
            let st = state.lock().unwrap();
            st.presences
                .iter()
                .map(|(id, p)| (id.clone(), p.state.clone(), p.capacity_seen, p.link_seen))
                .collect::<Vec<_>>()
        };
        {
            let mut st = state.lock().unwrap();
            st.attach_subscriptions.insert(
                "spec145-maintenance-sentinel".into(),
                AttachSubscription {
                    agent: crate::t3code::stable_uuid(T3),
                    attach_conn: "desktop".into(),
                    wrapper_conn: "t3".into(),
                    caught_up: true,
                },
            );
            st.view_closed_tx
                .send("spec145-maintenance-sentinel".into())
                .unwrap();
        }
        for action in [
            json!({"action":"list"}),
            json!({"action":"show","thread_id":"14500000-0000-4000-8000-000000000099"}),
        ] {
            let server = state.clone();
            let listener = listener.try_clone().unwrap();
            let handle = std::thread::spawn(move || {
                handle_connection(listener.accept().unwrap().0, server).map_err(|e| e.to_string())
            });
            let request = serde_json::from_value(
                json!({"version":1,"t3_thread_id":T3,"project_root":root,"request":action}),
            )
            .unwrap();
            let result = crate::communication::client::human_thread_view(&socket, request).unwrap();
            let value = serde_json::to_value(result).unwrap();
            assert!(matches!(
                value["result"]["status"].as_str(),
                Some("listed" | "error")
            ));
            handle.join().unwrap().unwrap();
            assert_eq!(
                snapshot(&state),
                before,
                "connect/hello/read/disconnect complets restent sans écriture"
            );
        }
        let binary = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("bridget");
        for action in ["list", "show"] {
            let server = state.clone();
            let listener = listener.try_clone().unwrap();
            let handle = std::thread::spawn(move || {
                handle_connection(listener.accept().unwrap().0, server).map_err(|e| e.to_string())
            });
            let mut command = std::process::Command::new(&binary);
            command
                .env("BRIDGET_HOME", &directory)
                .env("BRIDGET_SOCKET", &socket)
                .args([
                    "thread",
                    "inspect",
                    "--t3-thread",
                    T3,
                    "--project-root",
                    &root,
                    "--action",
                    action,
                    "--json",
                ]);
            if action == "show" {
                command.args(["--thread", "14500000-0000-4000-8000-000000000099"]);
            }
            let output = command
                .output()
                .expect("construire le binaire bridget avant ce test bout-en-bout");
            assert_eq!(
                output.status.code(),
                Some(if action == "list" { 0 } else { 2 })
            );
            let value: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                value["result"]["status"],
                if action == "list" { "listed" } else { "error" }
            );
            handle.join().unwrap().unwrap();
            assert_eq!(snapshot(&state), before, "CLI complet sans écriture métier");
        }
        {
            let server = state.clone();
            let listener = listener.try_clone().unwrap();
            let handle = std::thread::spawn(move || {
                handle_connection(listener.accept().unwrap().0, server).map_err(|e| e.to_string())
            });
            let mut stream = UnixStream::connect(&socket).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            writeln!(stream,"{}",json!({"type":"HumanThreadViewV1","request":{
                "version":1,"t3_thread_id":T3,"project_root":root,"request":{"action":"ack","thread_id":"14500000-0000-4000-8000-000000000099"}
            }})).unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let value: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(value["result"]["result"]["code"], "invalid_request");
            drop(reader);
            drop(stream);
            handle.join().unwrap().unwrap();
            assert_eq!(
                snapshot(&state),
                before,
                "refus wire fermé sans écriture ni maintenance"
            );
        }
        let presence_after = {
            let st = state.lock().unwrap();
            st.presences
                .iter()
                .map(|(id, p)| (id.clone(), p.state.clone(), p.capacity_seen, p.link_seen))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            state.lock().unwrap().view_closed_rx.try_recv().unwrap(),
            "spec145-maintenance-sentinel",
            "le cycle humain ne draine pas la maintenance des vues agent"
        );
        assert!(
            state
                .lock()
                .unwrap()
                .attach_subscriptions
                .contains_key("spec145-maintenance-sentinel")
        );
        assert_eq!(presence_after, presence_before);
        peer.set_nonblocking(true).unwrap();
        assert_eq!(
            std::io::Read::read(&mut peer, &mut [0u8; 1])
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn spec145_history_three_pages_exact_body_and_business_nonmutation() {
        use crate::store::threads::{CreateThread, NotifySpec, PostEntry, ThreadTxOutcome};
        let (state, root, mut peer) = fixture("h145p");
        const THREAD: &str = "14500000-0000-4000-8000-000000000001";
        let actor = crate::t3code::stable_uuid(T3);
        let body = "  exact é🦀\n\n```rust\nlet x = 1;\n```\n ";
        {
            let st = state.lock().unwrap();
            let mut members = vec![actor.clone(), "14500000-0000-4000-8000-000000000002".into()];
            members.sort();
            assert!(matches!(
                st.store
                    .thread_create(CreateThread {
                        actor: &actor,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: "create",
                        thread_id: THREAD,
                        title: "Lecture",
                        members: &members,
                        project_warnings: &[],
                        now: 1,
                        limits: &crate::threads::LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
            for seq in 1..=7 {
                assert!(matches!(
                    st.store
                        .thread_post(PostEntry {
                            actor: &actor,
                            operation_id: &uuid::Uuid::new_v4().to_string(),
                            canonical_hash: &format!("post-{seq}"),
                            thread_id: THREAD,
                            message_id: &uuid::Uuid::new_v4().to_string(),
                            body,
                            notify: &NotifySpec::None,
                            notices: &[],
                            project_warnings: &[],
                            reply_to_seq: None,
                            ack_receipt: None,
                            kind: Some(bridget_transport::protocol::ThreadEntryKind::History),
                            supersedes_seq: None,
                            now: seq,
                            limits: &crate::threads::LIMITS
                        })
                        .unwrap(),
                    ThreadTxOutcome::Done(_)
                ));
            }
        }
        let before = snapshot(&state);
        let listed = call(&state, &root, json!({"action":"list","limit":1}));
        assert_eq!(
            listed["result"]["threads"][0]["members"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(
            listed["result"]["threads"][0]["members"]
                .as_array()
                .unwrap()
                .iter()
                .any(|m| m["name"].is_null())
        );
        let shown = call(&state, &root, json!({"action":"show","thread_id":THREAD}));
        assert_eq!(shown["result"]["status"], "shown");
        assert!(shown["result"].get("own_acked_seq").is_none());
        assert!(shown["result"].get("own_wake").is_none());
        let mut seqs = Vec::new();
        let mut witnesses = json!({"list":listed,"show":shown});
        for from in [1, 4, 7] {
            let page = call(
                &state,
                &root,
                json!({"action":"history","thread_id":THREAD,"from_seq":from,"to_seq":7,"limit":3}),
            );
            if from == 1 {
                witnesses["history"] = page.clone();
            }
            assert_eq!(page["result"]["snapshot_seq"], 7);
            for entry in page["result"]["entries"].as_array().unwrap() {
                assert_eq!(entry["body"], body);
                assert_eq!(entry["kind"], "history");
                assert!(entry.get("author_name").is_some());
                seqs.push(entry["seq"].as_u64().unwrap());
            }
        }
        assert_eq!(seqs, (1..=7).collect::<Vec<_>>());
        witnesses["error"] = call(
            &state,
            &root,
            json!({"action":"show","thread_id":"14500000-0000-4000-8000-000000000099"}),
        );
        if let Ok(path) = std::env::var("BRIDGET_SPEC145_WITNESS") {
            std::fs::write(path, serde_json::to_vec_pretty(&witnesses).unwrap()).unwrap();
        }
        for action in [
            json!({"action":"list","limit":0}),
            json!({"action":"history","thread_id":THREAD,"limit":201}),
            json!({"action":"history","thread_id":THREAD,"from_seq":0}),
            json!({"action":"history","thread_id":THREAD,"from_seq":u64::MAX,"to_seq":u64::MAX}),
        ] {
            assert_eq!(
                call(&state, &root, action)["result"]["code"],
                "invalid_request"
            );
        }
        assert_eq!(
            call(
                &state,
                &root,
                json!({"action":"show","thread_id":"14500000-0000-4000-8000-000000000099"})
            )["result"]["code"],
            "thread_unavailable"
        );
        assert_eq!(
            snapshot(&state),
            before,
            "ACK, curseurs, réveils, émissions et missions inchangés"
        );
        peer.set_nonblocking(true).unwrap();
        let mut bytes = [0u8; 1];
        assert_eq!(
            std::io::Read::read(&mut peer, &mut bytes)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::WouldBlock,
            "aucun dispatch sur la socket agent"
        );
    }

    #[test]
    fn spec145_client_only_negotiated_binding_and_version() {
        let (state, _) = presence_tests::state_with_registered_agent("h145");
        let state = Arc::new(Mutex::new(state));
        let refusal = response(&state, "cold", 1);
        assert_eq!(refusal["result"]["code"], "binding_unavailable");
        assert!(refusal["subject"].is_null());
        assert!(!state.lock().unwrap().connection_roles.contains_key("cold"));
        {
            let mut st = state.lock().unwrap();
            st.connection_roles
                .insert("human".into(), ConnectionRole::Client);
            st.client_negotiations.insert(
                "human".into(),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: "spec145-human".into(),
                    capabilities: vec![
                        serde_json::from_value(json!("human_thread_view_v1")).unwrap(),
                    ],
                },
            );
        }
        assert_eq!(
            response(&state, "human", 2)["result"]["code"],
            "unsupported_version"
        );
        assert_eq!(
            response(&state, "human", 1)["result"]["code"],
            "binding_unavailable"
        );
        assert!(matches!(
            handle_wrapper_message(
                "human",
                WrapperToDaemon::ThreadRequest {
                    request: bridget_transport::protocol::ThreadRequest {
                        version: 1,
                        cross_project_reason: None,
                        request: bridget_transport::protocol::ThreadAction::List {
                            limit: None,
                            after_thread_id: None
                        }
                    }
                },
                &state
            ),
            Some(DaemonToWrapper::ClientRejected { .. })
        ));
    }

    #[test]
    fn spec146_recent_singleton_negotiation_and_refusals_do_not_mutate() {
        let (state, root, mut peer) = fixture("h146-cap");
        let before = snapshot(&state);
        assert_eq!(
            call(&state, &root, json!({"action":"list_recent"}))["result"]["code"],
            "unsupported_version"
        );
        // La fixture145 était déjà négociée : une nouvelle connexion humaine
        // commence après RoleHandshake, avant son unique ClientHello.
        state.lock().unwrap().client_negotiations.remove("human");
        let hello = WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("spec146-human"),
            capabilities: vec![ClientCapability::HumanThreadViewRecentV1],
        };
        let welcome = handle_wrapper_message("human", hello, &state).unwrap();
        let observed = serde_json::to_value(&welcome).unwrap();
        eprintln!("SPEC146 bienvenue réelle : {observed}");
        assert!(
            matches!(welcome,DaemonToWrapper::ClientWelcome {capabilities,..} if capabilities==[ClientCapability::HumanThreadViewRecentV1]),
            "{observed}"
        );
        assert_eq!(
            call(&state, &root, json!({"action":"list_recent"}))["result"]["status"],
            "listed_recent"
        );
        assert_eq!(
            call(&state, &root, json!({"action":"list"}))["result"]["code"],
            "unsupported_version"
        );
        assert_eq!(
            call(&state, "/missing", json!({"action":"list_recent"}))["result"]["code"],
            "project_mismatch"
        );
        assert_eq!(
            call(
                &state,
                &root,
                json!({"action":"history_recent","thread_id":"14600000-0000-4000-8000-000000000099"})
            )["result"]["code"],
            "thread_unavailable"
        );
        state
            .lock()
            .unwrap()
            .client_negotiations
            .get_mut("human")
            .unwrap()
            .capabilities
            .push(ClientCapability::SendIdempotent);
        assert_eq!(
            call(&state, &root, json!({"action":"list_recent"}))["result"]["code"],
            "unsupported_version"
        );
        assert_eq!(snapshot(&state), before);
        peer.set_nonblocking(true).unwrap();
        let mut byte = [0];
        assert_eq!(
            std::io::Read::read(&mut peer, &mut byte)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

    #[test]
    fn spec146_recent_hello_and_negotiated_client_skip_attach_maintenance() {
        let (state, _, _peer) = fixture("h146-maintenance");
        state.lock().unwrap().client_negotiations.remove("human");
        state
            .lock()
            .unwrap()
            .view_closed_tx
            .send("pending-hello".into())
            .unwrap();
        handle_wrapper_message(
            "human",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("spec146-human"),
                capabilities: vec![ClientCapability::HumanThreadViewRecentV1],
            },
            &state,
        )
        .unwrap();
        assert_eq!(
            state.lock().unwrap().view_closed_rx.try_recv().unwrap(),
            "pending-hello"
        );
        state
            .lock()
            .unwrap()
            .view_closed_tx
            .send("pending-negotiated".into())
            .unwrap();
        handle_wrapper_message("human", WrapperToDaemon::ListAgents, &state).unwrap();
        assert_eq!(
            state.lock().unwrap().view_closed_rx.try_recv().unwrap(),
            "pending-negotiated"
        );
    }
