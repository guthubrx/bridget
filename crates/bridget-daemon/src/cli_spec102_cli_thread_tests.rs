    use super::*;
    use bridget_transport::protocol::{ThreadAction, ThreadNotify, ThreadNotifyAll};

    const A: &str = "10200000-0000-4000-8000-00000000000a";
    const B: &str = "10200000-0000-4000-8000-00000000000b";
    const T: &str = "33333333-3333-4333-8333-333333333333";
    const OP: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn spec147_read_sequence_options_explain_ack_then_same_read() {
        for option in ["--from-seq", "--to-seq"] {
            let error =
                parse_thread_args(&args(&["read", T, option, "31"]), &resolver).unwrap_err();
            assert!(error.contains(option));
            assert!(error.contains("ack"), "{error}");
            assert!(error.contains("même read"), "{error}");
            assert!(error.contains("history"), "{error}");
        }
        let next = parse_thread_args(&args(&["read", T, "--limit", "30"]), &resolver).unwrap();
        assert!(matches!(
            next,
            ThreadAction::Read {
                limit: Some(30),
                ..
            }
        ));
        let history =
            parse_thread_args(&args(&["history", T, "--from-seq", "31"]), &resolver).unwrap();
        assert!(matches!(
            history,
            ThreadAction::History {
                from_seq: Some(31),
                ..
            }
        ));
    }

    #[test]
    fn spec147_members_cli_reuses_members_and_strict_options() {
        let upper = B.to_uppercase();
        let action = parse_thread_args(
            &args(&[
                "add-members",
                T,
                "--member",
                &upper,
                "--member",
                "Bruno",
                "--id",
                OP,
            ]),
            &|name| {
                assert_eq!(name, "Bruno");
                Ok(B.into())
            },
        )
        .unwrap();
        let value = serde_json::to_value(action).unwrap();
        assert_eq!(value["action"], "add_members");
        assert_eq!(value["members"], serde_json::json!([B, B]));
        for invalid in [
            args(&["add-members", T, "--member", B]),
            args(&["add-members", T, "--id", OP, "--all"]),
            args(&["add-members", T, "--id", OP, "--", "body"]),
            args(&["add-members", T, "--member", "missing", "--id", OP]),
        ] {
            assert!(parse_thread_args(&invalid, &|_| Err("inconnu".into())).is_err());
        }
    }

    #[test]
    fn spec147_uuid_cli_members_and_notify_bypass_name_resolution() {
        let resolve =
            |name: &str| -> Result<String, String> { panic!("UUID pris pour un nom : {name}") };
        let upper = B.to_uppercase();
        let create = parse_thread_args(
            &args(&[
                "create",
                "--title",
                "Titre É exact",
                "--member",
                &upper,
                "--id",
                OP,
            ]),
            &resolve,
        )
        .unwrap();
        assert!(
            matches!(create, ThreadAction::Create { members, title, .. } if members == vec![B.to_string()] && title == "Titre É exact")
        );
        let post = parse_thread_args(
            &args(&[
                "post",
                T,
                "--notify",
                &upper,
                "--id",
                OP,
                "--",
                "É\r\n  Exact\n",
            ]),
            &resolve,
        )
        .unwrap();
        assert!(
            matches!(post, ThreadAction::Post { notify: ThreadNotify::Targets(targets), body, .. } if targets == vec![B.to_string()] && body == "É\r\n  Exact\n")
        );
    }

    #[test]
    fn spec147_uuid_cli_keeps_explicit_names_and_rejects_invalid_declared_members() {
        let create = parse_thread_args(
            &args(&[
                "create",
                "--title",
                "Noms",
                "--member",
                "Bridget-enhance",
                "--id",
                OP,
            ]),
            &resolver,
        )
        .unwrap();
        assert!(
            matches!(create, ThreadAction::Create { members, .. } if members == vec![B.to_string()])
        );
        assert!(
            parse_thread_args(
                &args(&[
                    "create",
                    "--title",
                    "Refus",
                    "--member",
                    "{AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA}",
                    "--id",
                    OP
                ]),
                &resolver
            )
            .is_err()
        );
        assert!(
            parse_thread_watch(&args(&[
                "--t3-thread",
                &B.to_uppercase(),
                "--project-root",
                "/fixture",
                "--json"
            ]))
            .is_err()
        );
        assert!(
            parse_thread_inspect(&args(&[
                "--t3-thread",
                &B.to_uppercase(),
                "--project-root",
                "/fixture",
                "--action",
                "list_recent",
                "--json"
            ]))
            .is_err()
        );
    }

    fn resolver(name: &str) -> Result<String, String> {
        resolve_name_in_directory(
            name,
            [
                ("Bridget-enhance", B),
                ("Daily", A),
                ("Daily", B),
                ("Daily", B),
            ]
            .into_iter(),
        )
    }

    #[test]
    fn spec102_v29_marqueur_thread_notice_reconnu_et_ancien_format_conserve() {
        assert_eq!(
            thread_notice_marker(
                r#"{"kind":"thread_notice","thread_id":"33333333-3333-4333-8333-333333333333"}"#
            )
            .as_deref(),
            Some("33333333-3333-4333-8333-333333333333")
        );
        assert!(thread_notice_marker("10200000-0000-4000-8000-00000000000a\tmsg-1").is_none());
        assert!(thread_notice_marker("10200000-0000-4000-8000-00000000000a").is_none());
        assert!(thread_notice_marker(r#"{"kind":"autre","thread_id":"x"}"#).is_none());
    }

    #[test]
    fn spec132_le_client_attend_le_delai_de_lancement_demande() {
        let order = |issued_at: i64, deadline_at: i64| WrapperToDaemon::SpawnOrder {
            agent_type: "codex".into(),
            project: None,
            ownership: None,
            agent_id: None,
            cwd: "/tmp".into(),
            persistent: false,
            command_id: "c".into(),
            issued_at,
            deadline_at,
            posture: None,
        };
        let base = crate::communication::client::DAEMON_BUDGET;
        assert_eq!(
            spawn_wait_budget(&order(100, 190)),
            std::time::Duration::from_secs(95)
        );
        assert_eq!(
            spawn_wait_budget(&order(100, 102)),
            base,
            "jamais moins que le budget usuel"
        );
        assert_eq!(
            spawn_wait_budget(&order(100, 90)),
            base,
            "échéance incohérente : budget usuel"
        );
    }

    #[test]
    fn spec127_debut_d_uuid_unique_accepte_ambigu_ou_court_refuse() {
        let directory = [
            ("alpha", "3b09f990-eec1-4038-8a9b-29d1065d00b3"),
            ("beta", "3b09f991-0000-4000-8000-000000000001"),
            ("gamma", "c8c1bbac-72e7-4bc8-8e42-66e84e9f9150"),
        ];
        let resolve = |name: &str| resolve_name_in_directory(name, directory.iter().copied());
        assert_eq!(
            resolve("c8c1bb").unwrap(),
            "c8c1bbac-72e7-4bc8-8e42-66e84e9f9150"
        );
        assert_eq!(
            resolve("alpha").unwrap(),
            "3b09f990-eec1-4038-8a9b-29d1065d00b3"
        );
        assert!(resolve("3b09f99").unwrap_err().contains("ambigu"));
        assert!(
            resolve("c8c1b").unwrap_err().contains("inconnu"),
            "5 caractères : trop court"
        );
        assert!(resolve("ffffff").unwrap_err().contains("inconnu"));
    }

    #[test]
    fn spec136_cli_structured_options_are_strict() {
        let parsed = parse_thread_args(
            &args(&[
                "post",
                T,
                "--kind",
                "decision",
                "--supersedes",
                "4",
                "--notify",
                B,
                "--id",
                OP,
                "--",
                "consigne",
            ]),
            &resolver,
        )
        .unwrap();
        let value = serde_json::to_value(parsed).unwrap();
        assert_eq!(value["kind"], "decision");
        assert_eq!(value["supersedes_seq"], 4);
        for invalid in [
            args(&[
                "post", T, "--silent", "--id", OP, "--kind", "typo", "--", "x",
            ]),
            args(&[
                "post",
                T,
                "--silent",
                "--id",
                OP,
                "--supersedes",
                "0",
                "--",
                "x",
            ]),
            args(&["read", T, "--kind", "action"]),
        ] {
            assert!(parse_thread_args(&invalid, &resolver).is_err());
        }
    }

    #[test]
    fn spec102_v07_cli_noms_ambigus_refuses() {
        let unique = parse_thread_args(
            &args(&[
                "create",
                "--title",
                "T",
                "--member",
                "Bridget-enhance",
                "--id",
                OP,
            ]),
            &resolver,
        )
        .unwrap();
        assert!(
            matches!(unique, ThreadAction::Create { members, .. } if members == vec![B.to_string()])
        );
        let ambiguous = parse_thread_args(
            &args(&["post", T, "--notify", "Daily", "--id", OP, "--", "x"]),
            &resolver,
        )
        .unwrap_err();
        assert!(
            ambiguous.contains("ambigu") && ambiguous.contains("2 agents"),
            "{ambiguous}"
        );
        let unknown = parse_thread_args(
            &args(&["post", T, "--notify", "Inconnu", "--id", OP, "--", "x"]),
            &resolver,
        )
        .unwrap_err();
        assert!(unknown.contains("inconnu"), "{unknown}");
        // Un UUID canonique est une adresse stable : jamais résolu ni renommé.
        let direct = parse_thread_args(
            &args(&["post", T, "--notify", A, "--id", OP, "--", "x"]),
            &|_| Err("annuaire interdit".into()),
        )
        .unwrap();
        assert!(
            matches!(direct, ThreadAction::Post { notify: ThreadNotify::Targets(t), .. } if t == vec![A.to_string()])
        );
    }

    #[test]
    fn spec102_v33_cli_thread_flags_stricts_et_formes_exactes() {
        let silent = parse_thread_args(
            &args(&["post", T, "--silent", "--id", OP, "--", "un", "texte"]),
            &resolver,
        )
        .unwrap();
        assert!(
            matches!(silent, ThreadAction::Post { ref body, notify: ThreadNotify::Targets(ref t), .. } if body == "un texte" && t.is_empty())
        );
        let all = parse_thread_args(
            &args(&[
                "post",
                T,
                "--all",
                "--id",
                OP,
                "--reply-to",
                "3",
                "--ack",
                OP,
                "--",
                "x",
            ]),
            &resolver,
        )
        .unwrap();
        assert!(matches!(
            all,
            ThreadAction::Post {
                notify: ThreadNotify::All(ThreadNotifyAll::All),
                reply_to_seq: Some(3),
                ack_receipt: Some(_),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["list", "--limit", "5", "--after", T]), &resolver).unwrap(),
            ThreadAction::List {
                limit: Some(5),
                after_thread_id: Some(_)
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["show", T]), &resolver).unwrap(),
            ThreadAction::Show { .. }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["read", T, "--limit", "200"]), &resolver).unwrap(),
            ThreadAction::Read {
                limit: Some(200),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["ack", T, OP]), &resolver).unwrap(),
            ThreadAction::Ack { .. }
        ));
        assert!(matches!(
            parse_thread_args(
                &args(&["history", T, "--from-seq", "2", "--to-seq", "9"]),
                &resolver
            )
            .unwrap(),
            ThreadAction::History {
                from_seq: Some(2),
                to_seq: Some(9),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["close", T, "--id", OP]), &resolver).unwrap(),
            ThreadAction::Close { .. }
        ));
        for invalid in [
            vec!["post", T, "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--all", "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--", "x"],
            vec!["post", T, "--silent", "--id", OP],
            vec!["post", T, "--silent", "--id", OP, "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--id", OP, "--from", A, "--", "x"],
            vec![
                "post",
                T,
                "--silent",
                "--id",
                OP,
                "--reply-to",
                "0",
                "--",
                "x",
            ],
            vec!["create", "--title", "T", "--member", B],
            vec!["create", "--member", B, "--id", OP],
            vec!["show"],
            vec!["show", T, "extra"],
            vec!["ack", T],
            vec!["read", T, "--limit", "abc"],
            vec!["history", T, "--", "texte"],
            vec!["close", T],
            vec!["summary", T],
            vec!["list", "--script", "/tmp/x.sh"],
        ] {
            assert!(
                parse_thread_args(&args(&invalid), &resolver).is_err(),
                "{invalid:?} aurait dû être refusé"
            );
        }
    }

    #[test]
    fn spec138_thread_accepts_explicit_reason_outside_exact_body() {
        let parsed = parse_thread_args(
            &args(&[
                "post",
                T,
                "--silent",
                "--id",
                OP,
                "--cross-project-reason",
                "Comparer",
                "--",
                "exact body",
            ]),
            &resolver,
        )
        .unwrap();
        assert!(matches!(parsed, ThreadAction::Post { body, .. } if body == "exact body"));
    }
