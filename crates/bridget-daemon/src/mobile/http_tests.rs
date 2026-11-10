    use super::super::store::Scope;
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    fn key() -> String {
        format!(
            "{}{}",
            uuid::Uuid::new_v4().simple(),
            uuid::Uuid::new_v4().simple()
        )
    }
    fn run(future: impl std::future::Future<Output = ()>) {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
            .block_on(future);
    }
    #[test]
    fn user_input181_http_list_parses_pagination_before_authentication() {
        run(async {
            let response=dispatch(fixture(),Request::builder().uri("/v1/user-inputs?limit=20").header("host","bridget.example.net").body(axum::body::Body::empty()).unwrap()).await;
            assert_eq!(response.status(),StatusCode::UNAUTHORIZED,"A valid list query must reach authentication");
        });
    }
    fn fixture() -> Gateway {
        Gateway {
            root: PathBuf::from("/nonexistent-mobile-fixture"),
            config: Config {
                public_url: "https://bridget.example.net".into(),
                listen_addr: "127.0.0.1:9467".parse().unwrap(),
            },
            socket: PathBuf::from("/unavailable-fixture.sock"),
            store: Arc::new(Mutex::new(Store::memory())),
            workers: Arc::new(Semaphore::new(MAX_BLOCKING)),
            bootstrap_rate: Arc::new(Mutex::new(BTreeMap::new())),
        renewal_rate: Arc::new(Mutex::new(BTreeMap::new())),
            followers: Arc::new(Mutex::new(BTreeMap::new())),
            follow_slots: Arc::new(Semaphore::new(MAX_BLOCKING - 2)),
            uploads: Arc::new(Semaphore::new(2)),
        }
    }
    fn grant(g: &Gateway) -> String {
        g.store
            .lock()
            .unwrap()
            .issue_pairing(
                &Scope {
                    all_machines: false,
                    machines: vec!["cartae".into()],
                    all_projects: true,
                    projects: vec![],
                    operations: vec![Operation::Read, Operation::Message],
                },
                now(),
            )
            .unwrap()
            .credential
    }
    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }
    pub(crate) async fn start_native(
        root: &Path,
        socket: &Path,
    ) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        let mut gateway = fixture();
        gateway.socket = socket.to_path_buf();
        gateway.store = Arc::new(Mutex::new(
            Store::open(&root.join("mobile-devices.db")).unwrap(),
        ));
        start(gateway).await
    }
    async fn start(g: Gateway) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
        let inner = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = inner.local_addr().unwrap();
        let listener = LimitedListener {
            inner,
            permits: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        };
        let task = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(handle).with_state(g))
                .await
                .unwrap();
        });
        (addr, task)
    }
    pub(crate) async fn request(
        addr: std::net::SocketAddr,
        method: &str,
        path: &str,
        key: Option<&str>,
        body: &Value,
        extra: &str,
    ) -> (u16, String) {
        let body = if method == "POST" {
            body.to_string()
        } else {
            String::new()
        };
        let auth = key
            .map(|k| format!("Authorization: Bearer {k}\r\n"))
            .unwrap_or_default();
        let source=if extra.to_ascii_lowercase().contains("x-bridget-client-ip:") {""} else {"X-Bridget-Client-IP: 192.0.2.1\r\n"};
        let raw = format!(
            "{method} {path} HTTP/1.1\r\nHost: bridget.example.net\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}{source}{extra}\r\n{body}",
            body.len()
        );
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(raw.as_bytes()).await.unwrap();
        let mut bytes = Vec::new();
        tokio::time::timeout(Duration::from_secs(20), stream.read_to_end(&mut bytes))
            .await
            .unwrap()
            .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        let status = text.split_whitespace().nth(1).unwrap().parse().unwrap();
        (status, text)
    }

    #[test]
    fn mobile177_follow_quota_keeps_interactive_capacity_and_releases_idle_entries() {
        let g = fixture();
        let first = (0..4)
            .map(|_| g.follow_permit("first").unwrap())
            .collect::<Vec<_>>();
        assert!(g.follow_permit("first").is_err());
        let second = g.follow_permit("second").unwrap();
        let third = g.follow_permit("third").unwrap();
        assert!(g.follow_permit("fourth").is_err());
        assert_eq!(g.workers.available_permits(), MAX_BLOCKING);
        drop(first);
        drop(second);
        drop(third);
        for id in 0..100 {
            drop(g.follow_permit(&id.to_string()).unwrap());
        }
        assert_eq!(g.followers.lock().unwrap().len(), 1);
        assert_eq!(g.follow_slots.available_permits(), MAX_BLOCKING - 2);
    }

    #[test]
    fn mobile177_native_follow_uses_one_subscription_and_closes_on_revocation_or_deadline() {
        use bridget_transport::protocol::{ClientCapability, ConnectionRole, MobileOperation};
        use bridget_transport::{DaemonToWrapper as D, WrapperToDaemon as W};
        use std::io::{BufRead, BufReader, Read, Write};
        use std::os::unix::{fs::PermissionsExt, net::UnixListener};
        run(async {
            for mode in ["event", "revoke", "deadline"] {
                let mut g = fixture();
                let grant = g
                    .store
                    .lock()
                    .unwrap()
                    .issue_pairing(
                        &Scope {
                            all_machines: true,
                            machines: vec![],
                            all_projects: true,
                            projects: vec![],
                            operations: vec![Operation::Read],
                        },
                        now(),
                    )
                    .unwrap();
                let credential = key();
                let device = g
                    .store
                    .lock()
                    .unwrap()
                    .pair(
                        &grant.credential,
                        &uuid::Uuid::new_v4().to_string(),
                        &credential,
                        "Phone",
                        now(),
                    )
                    .unwrap();
                let agent = uuid::Uuid::new_v4().to_string();
                let path = std::env::temp_dir()
                    .join(format!("mobile177-follow-{}.sock", uuid::Uuid::new_v4()));
                let listener = UnixListener::bind(&path).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
                g.socket = path.clone();
                let current = g.store.clone();
                let revoked_device = device.device_id.clone();
                let expected_agent = agent.clone();
                let backend = std::thread::spawn(move || {
                    let send = |peer: &mut std::os::unix::net::UnixStream, frame: D| {
                        peer.write_all((serde_json::to_string(&frame).unwrap() + "\n").as_bytes())
                            .unwrap();
                    };
                    let (mut peer, _) = listener.accept().unwrap();
                    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                    let mut reader = BufReader::new(peer.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    assert!(matches!(
                        serde_json::from_str::<W>(&line).unwrap(),
                        W::RoleHandshake {
                            role: ConnectionRole::Client
                        }
                    ));
                    send(
                        &mut peer,
                        D::RoleAccepted {
                            role: ConnectionRole::Client,
                        },
                    );
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    assert!(matches!(
                        serde_json::from_str::<W>(&line).unwrap(),
                        W::ClientHello { .. }
                    ));
                    send(
                        &mut peer,
                        D::ClientWelcome {
                            version: 1,
                            build_id: "fixture177".into(),
                            horizon_secs: 60,
                            issued_at_tolerance_secs: 300,
                            capabilities: vec![ClientCapability::MobileOperationsV1],
                        },
                    );
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    assert!(
                        matches!(serde_json::from_str::<W>(&line).unwrap(),W::MobileOperation{request}
                        if matches!(&request.operation,MobileOperation::AgentJournal{agent_id,..} if agent_id==&expected_agent))
                    );
                    send(
                        &mut peer,
                        D::MobileOperationResult {
                            result: json!({"status":"ok","items":[],
                        "agent_id":expected_agent,"next_seq":0,"gap":null,"caught_up":true}),
                        },
                    );
                    drop(reader);
                    drop(peer);
                    let (mut peer, _) = listener.accept().unwrap();
                    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                    let mut reader = BufReader::new(peer.try_clone().unwrap());
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    assert!(matches!(
                        serde_json::from_str::<W>(&line).unwrap(),
                        W::RoleHandshake {
                            role: ConnectionRole::Attach
                        }
                    ));
                    send(
                        &mut peer,
                        D::RoleAccepted {
                            role: ConnectionRole::Attach,
                        },
                    );
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    assert!(
                        matches!(serde_json::from_str::<W>(&line).unwrap(),W::Subscribe{agent,..} if agent==expected_agent)
                    );
                    send(
                        &mut peer,
                        D::Subscribed {
                            subscription_id: "fixture-follow".into(),
                        },
                    );
                    send(
                        &mut peer,
                        D::SnapshotCaughtUp {
                            subscription_id: "fixture-follow".into(),
                            through_seq: Some(0),
                        },
                    );
                    if mode == "event" {
                        send(&mut peer,D::JournalFragment{subscription_id:"fixture-follow".into(),seq:1,offset:0,final_fragment:true,
                            bytes:serde_json::to_vec(&json!({"seq":1,"event":"update","payload":{"text":"native-live-fixture"}})).unwrap()});
                    } else if mode == "revoke" {
                        current
                            .lock()
                            .unwrap()
                            .revoke(&revoked_device, now())
                            .unwrap();
                    }
                    let mut end = [0u8; 1];
                    assert_eq!(
                        peer.read(&mut end).unwrap(),
                        0,
                        "Le flux natif doit être fermé"
                    );
                });
                if mode == "deadline" {
                    let copy = g.clone();
                    let k = credential.clone();
                    let a = agent.clone();
                    let response = tokio::task::spawn_blocking(move || {
                        execute(
                            &copy,
                            Method::GET,
                            "/v1/events",
                            Some(&k),
                            b"",
                            &super::super::client::DirectoryQuery {
                                agent_id: Some(a),
                                ..Default::default()
                            },
                            Instant::now() + Duration::from_millis(150),
                        )
                    })
                    .await
                    .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
                    let page: Value = serde_json::from_slice(&bytes).unwrap();
                    assert_eq!(page["result"]["idle"], true);
                    assert_eq!(page["result"]["next_seq"], 0);
                } else {
                    let (addr, http) = start(g.clone()).await;
                    let (status, body) = request(
                        addr,
                        "GET",
                        &format!("/v1/events?agent_id={agent}"),
                        Some(&credential),
                        &Value::Null,
                        "",
                    )
                    .await;
                    if mode == "event" {
                        assert_eq!(status, 200);
                        let page: Value =
                            serde_json::from_str(body.split("\r\n\r\n").nth(1).unwrap()).unwrap();
                        assert_eq!(
                            page["result"]["items"][0]["payload"]["text"],
                            "native-live-fixture"
                        );
                        let cursor: FollowCursor =
                            serde_json::from_str(page["result"]["next_cursor"].as_str().unwrap())
                                .unwrap();
                        assert_eq!(cursor.agent_id, agent);
                        assert_eq!(cursor.device_id, device.device_id);
                        assert_eq!(cursor.after_seq, 1);
                    } else {
                        assert_eq!(status, 401);
                        assert!(body.contains("device_revoked"));
                        assert!(!body.contains("native-live-fixture"));
                    }
                    http.abort();
                }
                backend.join().unwrap();
                std::fs::remove_file(path).unwrap();
            }
        });
    }

    #[test]
    fn mobile177_follow_cursor_and_query_cannot_select_another_device_or_ignore_filters() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let token = key();
            let device = g
                .store
                .lock()
                .unwrap()
                .pair(
                    &credential,
                    &uuid::Uuid::new_v4().to_string(),
                    &token,
                    "Phone",
                    now(),
                )
                .unwrap();
            let agent = uuid::Uuid::new_v4().to_string();
            let cursor = FollowCursor {
                version: 1,
                device_id: uuid::Uuid::new_v4().to_string(),
                agent_id: agent.clone(),
                after_seq: 10,
            };
            let (addr, http) = start(g).await;
            let encoded = serde_json::to_string(&cursor)
                .unwrap()
                .bytes()
                .map(|b| format!("%{b:02X}"))
                .collect::<String>();
            let (status, body) = request(
                addr,
                "GET",
                &format!("/v1/events?agent_id={agent}&after={encoded}"),
                Some(&token),
                &Value::Null,
                "",
            )
            .await;
            assert_eq!(status, 409);
            assert!(body.contains("resync_required"));
            for path in [
                "/v1/federation?machine=hidden",
                "/v1/agents?after_seq=10",
                "/v1/agents?tail=true",
                "/v1/events?tail=true",
                "/v1/agents/00000000-0000-4000-8000-000000000001/events?tail=true&after_seq=10",
                "/v1/devices?project_root=/private&project_host=hidden",
            ] {
                assert_eq!(
                    request(addr, "GET", path, Some(&token), &Value::Null, "")
                        .await
                        .0,
                    400
                );
            }
            assert_ne!(device.device_id, cursor.device_id);
            http.abort();
        });
    }

    #[test]
    fn mobile177_device_pages_are_bounded_and_management_never_exposes_credentials() {
        run(async {
            let g = fixture();
            let mut tokens = Vec::new();
            let mut ids = Vec::new();
            for n in 0..3 {
                let scope = Scope {
                    all_machines: true,
                    machines: vec![],
                    all_projects: true,
                    projects: vec![],
                    operations: if n == 0 {
                        vec![Operation::Read, Operation::DevicesManage]
                    } else {
                        vec![Operation::Read]
                    },
                };
                let mut store = g.store.lock().unwrap();
                let pairing = store.issue_pairing(&scope, now()).unwrap();
                let key = key();
                ids.push(
                    store
                        .pair(
                            &pairing.credential,
                            &uuid::Uuid::new_v4().to_string(),
                            &key,
                            &format!("Phone {n}"),
                            now(),
                        )
                        .unwrap()
                        .device_id,
                );
                tokens.push(key);
            }
            let (addr, http) = start(g).await;
            let mut after = None;
            let mut observed = Vec::new();
            loop {
                let path = after
                    .as_ref()
                    .map_or("/v1/devices?limit=1".into(), |cursor| {
                        format!("/v1/devices?limit=1&after={cursor}")
                    });
                let (status, body) =
                    request(addr, "GET", &path, Some(&tokens[0]), &Value::Null, "").await;
                assert_eq!(status, 200);
                assert!(tokens.iter().all(|token| !body.contains(token)));
                let value: Value =
                    serde_json::from_str(body.split("\r\n\r\n").nth(1).unwrap()).unwrap();
                assert_eq!(value["result"]["items"].as_array().unwrap().len(), 1);
                observed.push(
                    value["result"]["items"][0]["device_id"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                );
                after = value["result"]["next_cursor"].as_str().map(str::to_string);
                if after.is_none() {
                    break;
                }
                assert!(observed.len() < 4);
            }
            ids.sort();
            assert_eq!(observed, ids);
            let (status, body) = request(
                addr,
                "GET",
                "/v1/devices",
                Some(&tokens[1]),
                &Value::Null,
                "",
            )
            .await;
            assert_eq!(status, 200);
            let value: Value =
                serde_json::from_str(body.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            assert_eq!(value["result"]["items"].as_array().unwrap().len(), 1);
            assert_eq!(value["result"]["items"][0]["label"], "Phone 1");
            assert!(!body.contains("Phone 0"));
            assert!(!body.contains("Phone 2"));
            http.abort();
        });
    }

    #[test]
    fn mobile177_http_pair_replays_then_revocation_blocks_same_connection_identity() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let device_key = key();
            let payload = json!({"credential":credential,"attempt_id":uuid::Uuid::new_v4().to_string(),"device_key":device_key,"label":"Android"});
            let (addr, server) = start(g.clone()).await;
            let (status, first) = request(addr, "POST", "/v1/pair", None, &payload, "").await;
            assert_eq!(status, 200);
            let (_, again) = request(addr, "POST", "/v1/pair", None, &payload, "").await;
            assert_eq!(
                first.split("\r\n\r\n").nth(1),
                again.split("\r\n\r\n").nth(1)
            );
            assert!(!first.contains(&credential));
            assert!(!first.contains(&device_key));
            let id = g
                .store
                .lock()
                .unwrap()
                .authenticate(&device_key, now())
                .unwrap()
                .device_id;
            assert_eq!(
                request(
                    addr,
                    "GET",
                    "/v1/session",
                    Some(&device_key),
                    &Value::Null,
                    ""
                )
                .await
                .0,
                200
            );
            assert_eq!(
                request(
                    addr,
                    "DELETE",
                    &format!("/v1/devices/{id}"),
                    Some(&device_key),
                    &Value::Null,
                    ""
                )
                .await
                .0,
                200
            );
            let (status, body) = request(
                addr,
                "GET",
                "/v1/session",
                Some(&device_key),
                &Value::Null,
                "",
            )
            .await;
            assert_eq!(status, 401);
            assert!(body.contains("device_revoked"));
            server.abort();
        });
    }

    #[test]
    fn mobile177_http_rotation_reconciles_lost_response_without_reauthorizing_old_key() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let old = key();
            let new = key();
            g.store
                .lock()
                .unwrap()
                .pair(
                    &credential,
                    &uuid::Uuid::new_v4().to_string(),
                    &old,
                    "Phone",
                    now(),
                )
                .unwrap();
            let (addr, server) = start(g).await;
            let payload = json!({"request_id":uuid::Uuid::new_v4().to_string(),"device_key":new});
            assert_eq!(
                request(addr, "POST", "/v1/session/renew", Some(&old), &payload, "")
                    .await
                    .0,
                200
            );
            assert_eq!(
                request(addr, "POST", "/v1/session/renew", Some(&old), &payload, "")
                    .await
                    .0,
                200
            );
            assert_eq!(
                request(addr, "GET", "/v1/session", Some(&old), &Value::Null, "")
                    .await
                    .0,
                401
            );
            assert_eq!(
                request(addr, "GET", "/v1/session", Some(&new), &Value::Null, "")
                    .await
                    .0,
                200
            );
            server.abort();
        });
    }

    #[test]
    fn mobile177_http_rejects_duplicate_auth_foreign_origin_unknown_fields_and_oversized_json() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let device_key = key();
            let (addr, server) = start(g.clone()).await;
            let payload = json!({"credential":credential,"attempt_id":uuid::Uuid::new_v4().to_string(),"device_key":device_key,"label":"Phone","actor":"private-argument"});
            let (status, body) = request(addr, "POST", "/v1/pair", None, &payload, "").await;
            assert_eq!(status, 400);
            assert!(!body.contains("private-argument"));
            assert!(g.store.lock().unwrap().devices().unwrap().is_empty());
            assert_eq!(
                request(
                    addr,
                    "GET",
                    "/v1/status",
                    None,
                    &Value::Null,
                    "Origin: https://foreign.example.net\r\n"
                )
                .await
                .0,
                400
            );
            let duplicate = format!("Authorization: Bearer {}\r\n", key());
            assert_eq!(
                request(
                    addr,
                    "GET",
                    "/v1/session",
                    Some(&device_key),
                    &Value::Null,
                    &duplicate
                )
                .await
                .0,
                401
            );
            assert_eq!(
                request(
                    addr,
                    "POST",
                    "/v1/pair",
                    None,
                    &json!({"padding":"x".repeat(BODY_LIMIT+1)}),
                    ""
                )
                .await
                .0,
                413
            );
            let (status, body) = request(addr, "GET", "/v1/status", None, &Value::Null, "").await;
            assert_eq!(status, 200);
            assert!(body.to_lowercase().contains("cache-control: no-store"));
            assert!(!body.contains("socket"));
            assert!(!body.contains("cartae"));
            server.abort();
        });
    }

    #[test]
    fn mobile177_http_refuses_new_agent_creation_and_limits_device_management() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let device_key = key();
            let me = g
                .store
                .lock()
                .unwrap()
                .pair(
                    &credential,
                    &uuid::Uuid::new_v4().to_string(),
                    &device_key,
                    "Phone",
                    now(),
                )
                .unwrap();
            let other_credential = grant(&g);
            let other_key = key();
            let other = g
                .store
                .lock()
                .unwrap()
                .pair(
                    &other_credential,
                    &uuid::Uuid::new_v4().to_string(),
                    &other_key,
                    "Other",
                    now(),
                )
                .unwrap();
            let (addr, server) = start(g.clone()).await;
            let (status, body) = request(
                addr,
                "GET",
                "/v1/devices",
                Some(&device_key),
                &Value::Null,
                "",
            )
            .await;
            assert_eq!(status, 200);
            assert!(body.contains(&me.device_id));
            assert!(!body.contains(&other.device_id));
            assert_eq!(
                request(
                    addr,
                    "DELETE",
                    &format!("/v1/devices/{}", other.device_id),
                    Some(&device_key),
                    &Value::Null,
                    ""
                )
                .await
                .0,
                403
            );
            assert_eq!(
                request(
                    addr,
                    "POST",
                    "/v1/commands",
                    Some(&device_key),
                    &json!({"type":"create_agent"}),
                    ""
                )
                .await
                .0,
                409
            );
            server.abort();
        });
    }

    #[test]
    fn mobile177_http_invalid_bootstrap_cannot_exhaust_another_source_or_renewal() {
        run(async {
            let g=fixture();let credential=grant(&g);let old=key();let new=key();
            let (addr,server)=start(g.clone()).await;
            for _ in 0..30 { assert!(g.bootstrap_allowed("192.0.2.1")); }
            assert!(!g.bootstrap_allowed("192.0.2.1"));
            let payload=json!({"credential":credential,"attempt_id":uuid::Uuid::new_v4().to_string(),"device_key":old,"label":"Phone"});
            assert_eq!(request(addr,"POST","/v1/pair",None,&payload,"X-Bridget-Client-IP: 192.0.2.2\r\n").await.0,200);
            let renewal=json!({"request_id":uuid::Uuid::new_v4().to_string(),"device_key":new});
            for _ in 0..31 {
                assert_eq!(request(addr,"POST","/v1/session/renew",Some(&key()),&renewal,"").await.0,401);
            }
            assert!(g.renewal_rate.lock().unwrap().is_empty());
            assert_eq!(request(addr,"POST","/v1/session/renew",Some(&old),&renewal,"").await.0,200);
            assert_eq!(request(addr,"POST","/v1/session/renew",Some(&old),&renewal,"").await.0,200);
            assert_eq!(g.renewal_rate.lock().unwrap().len(),1);
            assert!(g.store.lock().unwrap().authenticate(&old,now()).is_err());
            assert!(g.store.lock().unwrap().authenticate(&new,now()).is_ok());
            server.abort();
        });
    }

    #[test]
    fn mobile177_http_saturation_and_rate_limit_do_not_consume_pairing() {
        run(async {
            let g = fixture();
            let credential = grant(&g);
            let held = g
                .workers
                .clone()
                .acquire_many_owned(MAX_BLOCKING as u32)
                .await
                .unwrap();
            let (addr, server) = start(g.clone()).await;
            let payload = json!({"credential":credential,"attempt_id":uuid::Uuid::new_v4().to_string(),"device_key":key(),"label":"Phone"});
            assert_eq!(
                request(addr, "POST", "/v1/pair", None, &payload, "")
                    .await
                    .0,
                429
            );
            drop(held);
            for _ in 0..29 {
                assert!(g.bootstrap_allowed("192.0.2.1"));
            }
            assert!(!g.bootstrap_allowed("192.0.2.1"));
            let (status, body) = request(addr, "POST", "/v1/pair", None, &payload, "").await;
            assert_eq!(status, 429);
            assert!(body.contains("retry-after"));
            assert!(g.store.lock().unwrap().devices().unwrap().is_empty());
            server.abort();
        });
    }

    #[test]
    fn mobile177_tcp_limit_is_held_until_connection_is_dropped() {
        use axum::serve::Listener;
        run(async {
            let inner = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = inner.local_addr().unwrap();
            let permits = Arc::new(Semaphore::new(1));
            let mut listener = LimitedListener {
                inner,
                permits: permits.clone(),
            };
            let client = tokio::net::TcpStream::connect(addr).await.unwrap();
            let (io, _) = listener.accept().await;
            assert_eq!(permits.available_permits(), 0);
            drop(io);
            assert_eq!(permits.available_permits(), 1);
            drop(client);
        });
    }
    #[test]
    fn mobile177_notifications_without_provider_configuration_cannot_claim_enabled() {
        let gateway=fixture();
        let mut scope=Scope {all_machines:true,machines:vec![],all_projects:true,projects:vec![],operations:vec![Operation::Read,Operation::TaskRead]};
        let credential=key();
        { let mut store=gateway.store.lock().unwrap();let grant=store.issue_pairing(&scope,unix_now()).unwrap();
          store.pair(&grant.credential,&uuid::Uuid::new_v4().to_string(),&credential,"Phone",unix_now()).unwrap(); }
        let query=super::super::client::DirectoryQuery::default();
        let read=execute(&gateway,Method::GET,"/v1/notifications",Some(&credential),b"",&query,Instant::now()+REQUEST_BUDGET);
        assert_eq!(read.status(),StatusCode::OK);
        let enabled=execute(&gateway,Method::PUT,"/v1/notifications",Some(&credential),br#"{"enabled":true,"token":"receiver_token_test_123456789"}"#,&query,Instant::now()+REQUEST_BUDGET);
        assert_eq!(enabled.status(),StatusCode::SERVICE_UNAVAILABLE);
        assert!(gateway.store.lock().unwrap().notification_targets(unix_now()).unwrap().is_empty());
        let disabled=execute(&gateway,Method::PUT,"/v1/notifications",Some(&credential),br#"{"enabled":false,"token":null}"#,&query,Instant::now()+REQUEST_BUDGET);
        assert_eq!(disabled.status(),StatusCode::OK);
        scope.all_machines=false;scope.machines=vec!["cartae".into()];
        let limited=key();
        { let mut store=gateway.store.lock().unwrap();let grant=store.issue_pairing(&scope,unix_now()).unwrap();store.pair(&grant.credential,&uuid::Uuid::new_v4().to_string(),&limited,"Limited",unix_now()).unwrap(); }
        let denied=execute(&gateway,Method::GET,"/v1/notifications",Some(&limited),b"",&query,Instant::now()+REQUEST_BUDGET);
        assert_eq!(denied.status(),StatusCode::FORBIDDEN);
    }

