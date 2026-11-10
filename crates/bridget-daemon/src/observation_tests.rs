    use super::*;
    #[test]
    fn spec135_repeated_turn_ends_never_wake_the_owner() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "source", vec![Kind::TurnEnded], now);
        let receipt = state.request("owner", long_sub(Kind::TurnEnded), now, 100);
        assert_eq!(receipt["status"], "subscribed");
        for minutes in [0, 1, 5, 6, 9, 12, 18, 19, 22, 22, 25] {
            assert!(
                state
                    .observe(
                        Fact {
                            event: Kind::TurnEnded,
                            agent: "source".into(),
                            host: "h".into(),
                            file: None,
                            other_agent: None,
                        },
                        now + Duration::from_secs(minutes * 60)
                    )
                    .is_empty()
            );
        }
        let list = state.request("owner", Request::List {}, now, 100);
        assert_eq!(list["subscriptions"].as_array().unwrap().len(), 1);
        assert_eq!(list["subscriptions"][0]["suppressed_total"], 11);
        assert_eq!(
            list["subscriptions"][0]["notification_mode"],
            "journal_only"
        );
        let types = state.request("owner", Request::Types {}, now, 100);
        assert!(
            types["events"][0]["meaning"]
                .as_str()
                .unwrap()
                .contains("once=true")
        );
    }

    #[test]
    fn spec135_legacy_repeated_turn_subscription_stays_silent_after_restore() {
        let now = Instant::now();
        // Format effectivement stocké avant la correction, sans nouveau champ.
        let bytes = br#"[{"id":"legacy","owner":"owner","event":"turn_ended","agent":"source","file":null,"once":false,"expires_at":3700}]"#;
        let mut state = Observations::restore(bytes, now, 110).unwrap();
        assert!(state.owner_returned("owner", now).is_empty());
        assert!(
            state
                .due_source_notices(now + SOURCE_NOTICE_GRACE)
                .is_empty()
        );
        state.set_source("conn", "source", vec![Kind::TurnEnded], now);
        assert!(
            state
                .due_source_notices(now + SOURCE_NOTICE_GRACE)
                .is_empty()
        );
        assert!(state.report_gap("source", 7, now).is_empty());
        assert!(
            state
                .observe(
                    Fact {
                        event: Kind::TurnEnded,
                        agent: "source".into(),
                        host: "h".into(),
                        file: None,
                        other_agent: None,
                    },
                    now
                )
                .is_empty()
        );
        let list = state.request("owner", Request::List {}, now, 110);
        assert_eq!(list["subscriptions"][0]["id"], "legacy");
        assert_eq!(list["subscriptions"][0]["expires_at"], 3700);
        assert_eq!(list["subscriptions"][0]["state"], "active");
        assert_eq!(list["subscriptions"][0]["facts_lost_total"], 7);
        assert_eq!(list["subscriptions"][0]["observation_gaps"], 1);
        assert_eq!(
            list["subscriptions"][0]["notification_mode"],
            "journal_only"
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&state.snapshot().unwrap()).unwrap(),
            serde_json::from_slice::<Value>(bytes).unwrap()
        );
    }

    #[test]
    fn spec135_one_shot_wait_and_permission_alerts_still_wake_the_owner() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source(
            "conn",
            "source",
            vec![Kind::TurnEnded, Kind::PermissionRequired],
            now,
        );
        state.request("owner", long_sub(Kind::TurnEnded), now, 100);
        let receipt = state.request("owner", sub(Kind::TurnEnded, true), now, 100);
        assert_eq!(receipt["subscription"]["notification_mode"], "notify");
        let fact = Fact {
            event: Kind::TurnEnded,
            agent: "source".into(),
            host: "h".into(),
            file: None,
            other_agent: None,
        };
        assert_eq!(state.observe(fact.clone(), now).len(), 1);
        assert!(
            state
                .observe(fact, now + Duration::from_secs(60))
                .is_empty()
        );
        state.request("owner", long_sub(Kind::PermissionRequired), now, 100);
        for seconds in [1, 60] {
            let notices = state.observe(
                Fact {
                    event: Kind::PermissionRequired,
                    agent: "source".into(),
                    host: "h".into(),
                    file: None,
                    other_agent: None,
                },
                now + Duration::from_secs(seconds),
            );
            assert_eq!(notices.len(), 1);
            assert!(notices[0].body.contains("permission_required"));
        }
    }

    #[test]
    fn spec101_gap_is_counted_filtered_and_notification_rate_bounded() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        state.set_source("b", "b", vec![Kind::TurnEnded], now);
        let for_agent = |agent: &str| Request::Sub {
            event: Kind::TurnEnded,
            agent: Some(agent.into()),
            file: None,
            once: true,
            ttl_secs: Some(60),
        };
        state.request("owner-a", for_agent("a"), now, 100);
        state.request("owner-b", for_agent("b"), now, 100);
        let notices = state.report_gap("a", 44, now);
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].owner, "owner-a");
        assert!(notices[0].body.contains("observation_gap"));
        assert!(state.report_gap("a", 2, now).is_empty());
        assert_eq!(state.facts_lost, 46);
        let list = state.request("owner-a", Request::List {}, now, 100);
        assert_eq!(list["subscriptions"][0]["facts_lost_total"], 46);
        assert_eq!(list["subscriptions"][0]["state"], "active");
        assert_eq!(
            state.subscriptions.len(),
            2,
            "la lacune ne consomme jamais once"
        );
    }
    #[test]
    fn spec101_sources_are_required_and_loss_is_explicit() {
        let now = Instant::now();
        let mut state = Observations::default();
        assert_eq!(
            state.request("owner", sub(Kind::TurnEnded, false), now, 100)["reason"],
            "no_compatible_source"
        );
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        assert!(state.due_source_notices(now).is_empty());
        let catalogue = state.request("owner", Request::Types {}, now, 100);
        assert_eq!(catalogue["events"][0]["available"], true);
        assert_eq!(catalogue["events"][0]["sources"], json!(["a"]));
        assert_eq!(catalogue["events"][2]["available"], false);
        assert_eq!(
            state.request("owner", sub(Kind::FileWritten, false), now, 100)["reason"],
            "no_compatible_source"
        );
        state.request(
            "owner",
            Request::Sub {
                event: Kind::TurnEnded,
                agent: None,
                file: None,
                once: true,
                ttl_secs: Some(3600),
            },
            now,
            100,
        );
        state.remove_source("conn", now);
        assert_eq!(
            state.request("owner", Request::List {}, now, 100)["subscriptions"][0]["state"],
            "source_unavailable"
        );
        let later = now + SOURCE_NOTICE_GRACE;
        assert_eq!(state.due_source_notices(later).len(), 1);
        state.set_source("new", "a", vec![Kind::TurnEnded], later);
        assert_eq!(
            state.request("owner", Request::List {}, later, 100)["subscriptions"][0]["state"],
            "active"
        );
        assert_eq!(
            state.due_source_notices(later + SOURCE_NOTICE_GRACE).len(),
            1
        );
    }

    #[test]
    fn spec119_aller_retour_bref_de_source_sans_avis() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::PermissionRequired], now);
        state.request("owner", long_sub(Kind::PermissionRequired), now, 100);
        state.remove_source("conn", now);
        state.set_source(
            "conn2",
            "a",
            vec![Kind::PermissionRequired],
            now + Duration::from_secs(3),
        );
        assert!(
            state
                .due_source_notices(now + SOURCE_NOTICE_MAX_DELAY * 2)
                .is_empty(),
            "revenu à l'état annoncé : rien à dire"
        );
        // Une interruption qui dure est annoncée, une seule fois.
        state.remove_source("conn2", now);
        let bodies: Vec<_> = (0..120)
            .flat_map(|s| state.due_source_notices(now + Duration::from_secs(s)))
            .map(|n| n.body)
            .collect();
        assert_eq!(bodies.len(), 1);
        assert!(bodies[0].contains("source_unavailable"));
    }

    #[test]
    fn spec119_source_qui_clignote_avis_borne_par_periode() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::PermissionRequired], now);
        state.request("owner", long_sub(Kind::PermissionRequired), now, 100);
        // Le cas du 25/09 : indisponible puis disponible toutes les 3 s, 10 min.
        let mut notices = Vec::new();
        for tick in 0..600u64 {
            let at = now + Duration::from_secs(tick);
            match tick % 6 {
                0 => state.remove_source("conn", at),
                3 => state.set_source("conn", "a", vec![Kind::PermissionRequired], at),
                _ => {}
            }
            notices.extend(state.due_source_notices(at).into_iter().map(|n| n.body));
        }
        assert_eq!(
            notices.len(),
            1,
            "un avis par période de 5 min, pas un par bascule"
        );
        assert!(notices[0].contains("source instable"));
    }

    #[test]
    fn spec122_restore_resumes_and_absolute_ttl_is_preserved() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        state.request("owner", sub(Kind::TurnEnded, true), now, 100);
        let bytes = state.snapshot().unwrap();
        // Source pas encore revenue : indisponible, annoncée seulement si cela dure.
        let mut absent = Observations::restore(&bytes, now, 110).unwrap();
        assert_eq!(
            absent.request("owner", Request::List {}, now, 110)["subscriptions"][0]["state"],
            "source_unavailable"
        );
        let later = now + SOURCE_NOTICE_GRACE;
        assert!(
            absent.due_source_notices(later)[0]
                .body
                .contains("source indisponible")
        );
        // Source revenue : reprise sans nouvel abonnement, un seul avis au propriétaire.
        let mut restored = Observations::restore(&bytes, now, 110).unwrap();
        restored.set_source("conn", "a", vec![Kind::TurnEnded], now);
        assert_eq!(
            restored.request("owner", Request::List {}, now, 110)["subscriptions"][0]["state"],
            "active"
        );
        let notices = restored.owner_returned("owner", now);
        assert_eq!(notices.len(), 1);
        assert!(notices[0].body.contains("repris automatiquement"));
        assert!(restored.owner_returned("owner", now).is_empty());
        assert!(
            restored.due_source_notices(later).is_empty(),
            "retour à l'état connu"
        );
        assert_eq!(
            restored
                .observe(
                    Fact {
                        event: Kind::TurnEnded,
                        agent: "a".into(),
                        host: "h".into(),
                        file: None,
                        other_agent: None
                    },
                    now
                )
                .len(),
            1,
            "les faits futurs sont remis"
        );
        assert!(
            Observations::restore(&bytes, now, 161)
                .unwrap()
                .subscriptions
                .is_empty()
        );
        assert!(Observations::restore(b"invalid", now, 110).is_err());
    }
    fn long_sub(event: Kind) -> Request {
        Request::Sub {
            event,
            agent: None,
            file: None,
            once: false,
            ttl_secs: Some(3600),
        }
    }

    fn sub(event: Kind, once: bool) -> Request {
        Request::Sub {
            event,
            agent: None,
            file: None,
            once,
            ttl_secs: Some(60),
        }
    }
    fn file(agent: &str, host: &str, path: &str) -> Fact {
        Fact {
            event: Kind::FileWritten,
            agent: agent.into(),
            host: host.into(),
            file: Some(path.into()),
            other_agent: None,
        }
    }
    #[test]
    fn spec100_once_owner_expiry_and_filters() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        let receipt = state.request("owner", sub(Kind::TurnEnded, true), now, 100);
        let id = receipt["subscription"]["id"].as_str().unwrap().to_string();
        assert_eq!(
            state.request("other", Request::Unsub { id }, now, 100)["status"],
            "rejected"
        );
        let fact = Fact {
            event: Kind::TurnEnded,
            agent: "a".into(),
            host: "h".into(),
            file: None,
            other_agent: None,
        };
        assert_eq!(state.observe(fact.clone(), now).len(), 1);
        assert!(state.observe(fact.clone(), now).is_empty());
        state.request("owner", sub(Kind::TurnEnded, false), now, 100);
        assert!(
            state
                .observe(fact, now + Duration::from_secs(61))
                .is_empty()
        );
        assert!(path_matches("*.rs", "/project/src/lib.rs"));
        assert!(!path_matches("/other/*", "/project/src/lib.rs"));
    }
    #[test]
    fn native154_worktree_paths_do_not_collide() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::FileWritten], now);
        state.request("owner", sub(Kind::FileCollision, false), now, 100);
        assert!(state.observe(file("a", "h", "/depot/a.rs"), now).is_empty());
        assert!(
            state
                .observe(
                    file("b", "h", "/depot/.worktrees/12345678/a.rs"),
                    now + Duration::from_secs(1)
                )
                .is_empty()
        );
        assert_eq!(
            state
                .observe(file("b", "h", "/depot/a.rs"), now + Duration::from_secs(2))
                .len(),
            1
        );
    }

    #[test]
    fn spec100_collision_is_scoped_and_deduplicated() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::FileWritten], now);
        state.request("owner", sub(Kind::FileCollision, false), now, 100);
        assert!(state.observe(file("a", "h", "/p/x"), now).is_empty());
        assert!(state.observe(file("a", "h", "/p/./x"), now).is_empty());
        assert!(state.observe(file("b", "other", "/p/x"), now).is_empty());
        assert!(state.observe(file("b", "h", "/p/y"), now).is_empty());
        assert_eq!(state.observe(file("b", "h", "/p/x"), now).len(), 1);
        assert!(state.observe(file("a", "h", "/p/x"), now).is_empty());
        assert!(
            state
                .observe(file("c", "h", "/p/x"), now + Duration::from_secs(31))
                .is_empty()
        );
        assert_eq!(normalized_absolute("/a/../p/x"), Some("/p/x".into()));
        assert_eq!(normalized_absolute("relative"), None);
    }
    #[test]
    fn spec100_subscriptions_are_bounded_and_unknown_fields_refused() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        for _ in 0..16 {
            assert_eq!(
                state.request("owner", sub(Kind::TurnEnded, false), now, 0)["status"],
                "subscribed"
            );
        }
        assert_eq!(
            state.request("owner", sub(Kind::TurnEnded, false), now, 0)["reason"],
            "subscription_limit"
        );
        assert!(
            serde_json::from_value::<Request>(json!({"action":"list","owner":"victim"})).is_err()
        );
        for owner in 1..8 {
            for _ in 0..16 {
                state.request(
                    &format!("owner{owner}"),
                    sub(Kind::TurnEnded, false),
                    now,
                    0,
                );
            }
        }
        assert_eq!(
            state.request("new-owner", sub(Kind::TurnEnded, false), now, 0)["reason"],
            "subscription_limit"
        );
        let list = state.request("owner", Request::List {}, now, 0);
        let id = list["subscriptions"][0]["id"].as_str().unwrap();
        assert_eq!(
            state.request("owner", Request::Unsub { id: id.into() }, now, 0)["status"],
            "unsubscribed"
        );
        assert_eq!(
            state.request("owner", Request::List {}, now, 0)["subscriptions"]
                .as_array()
                .unwrap()
                .len(),
            15
        );
    }

    #[test]
    fn spec100_rate_cache_and_combined_filters_are_bounded() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::FileWritten], now);
        state.request(
            "owner",
            Request::Sub {
                event: Kind::FileWritten,
                agent: Some("a".into()),
                file: Some("/p/*.rs".into()),
                once: false,
                ttl_secs: Some(60),
            },
            now,
            0,
        );
        assert!(state.observe(file("b", "h", "/p/x.rs"), now).is_empty());
        assert!(state.observe(file("a", "h", "/p/x.txt"), now).is_empty());
        assert_eq!(state.observe(file("a", "h", "/p/x.rs"), now).len(), 1);
        assert!(state.observe(file("a", "h", "/p/y.rs"), now).is_empty());
        assert_eq!(
            state.request("owner", Request::List {}, now, 0)["subscriptions"][0]["suppressed_total"],
            1
        );
        assert_eq!(
            state
                .observe(file("a", "h", "/p/z.rs"), now + Duration::from_millis(201))
                .len(),
            1
        );
        for i in 0..4097 {
            state.observe(file("b", "h", &format!("/cache/{i}")), now);
        }
        assert!(state.writes.len() <= 4096);
        assert!(state.evicted_writes >= 4096);
    }
