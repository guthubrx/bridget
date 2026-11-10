    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    fn subject() -> HumanInboxSubject {
        HumanInboxSubject {
            objective_id: Some("o1".to_string()),
            delegation_id: Some("d1".to_string()),
            ..HumanInboxSubject::default()
        }
    }

    fn request<'a>(subject: &'a HumanInboxSubject, options: &'a [String]) -> DepositRequest<'a> {
        DepositRequest {
            dedup_key: "chain-exhausted:d1",
            kind: HumanInboxKind::ChainExhausted,
            subject,
            context: r#"{"summary":"chaîne épuisée","attempts":2}"#,
            options,
            producer: HumanInboxProducer::Guichet,
            now: 100,
        }
    }

    #[test]
    fn schema_check_derive_des_enums() {
        let conn = conn();
        let ddl: String = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = 'human_inbox'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        for kind in HumanInboxKind::ALL {
            assert!(ddl.contains(&format!("'{}'", kind.as_sql())), "{ddl}");
        }
        assert!(
            ddl.contains("'closed_self'") && ddl.contains("'guichet'"),
            "{ddl}"
        );
    }

    #[test]
    fn depot_dedoublonne_parmi_les_items_ouverts() {
        let conn = conn();
        let subject = subject();
        let options = vec!["reassign:a1".to_string(), "cancel".to_string()];
        let first = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(first.created);
        let second = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(!second.created);
        assert_eq!(second.item_id, first.item_id);
        assert_eq!(second.occurrences, 2);
        let (items, open) = list(&conn, HumanInboxListFilter::Open, 10).unwrap();
        assert_eq!(open, 1);
        assert_eq!(items[0].occurrences, 2);
        assert_eq!(items[0].kind, HumanInboxKind::ChainExhausted);
        assert_eq!(items[0].subject, subject);
    }

    #[test]
    fn depot_invalide_refuse() {
        let conn = conn();
        let subject = subject();
        let empty: Vec<String> = Vec::new();
        assert_eq!(
            deposit(&conn, request(&subject, &empty))
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::InvalidRequest
        );
        let options = vec!["ack".to_string()];
        let big = "x".repeat(MAX_CONTEXT_BYTES + 1);
        let refused = deposit(
            &conn,
            DepositRequest {
                context: &big,
                ..request(&subject, &options)
            },
        )
        .unwrap()
        .unwrap_err();
        assert_eq!(refused, HumanInboxRefusal::InvalidRequest);
    }

    #[test]
    fn resolution_verifie_options_et_etat() {
        let conn = conn();
        let subject = subject();
        let options = vec!["reassign:a1".to_string(), "cancel".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert_eq!(
            resolve(&conn, "absent", "cancel", "humain", 200)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::UnknownItem
        );
        assert_eq!(
            resolve(&conn, &deposited.item_id, "raise_budget", "humain", 200)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::ChoiceNotOffered
        );
        let resolved = resolve(&conn, &deposited.item_id, "cancel", "humain", 200)
            .unwrap()
            .unwrap();
        assert_eq!(resolved.state, HumanInboxState::Resolved);
        assert_eq!(resolved.decision.as_ref().unwrap().choice, "cancel");
        assert_eq!(
            resolve(&conn, &deposited.item_id, "cancel", "humain", 201)
                .unwrap()
                .unwrap_err(),
            HumanInboxRefusal::AlreadyResolved
        );
        // Une nouvelle occurrence après résolution crée un item neuf.
        let again = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(again.created);
        assert_ne!(again.item_id, deposited.item_id);
    }

    #[test]
    fn decision_relue_jusqu_a_l_acquittement() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Guichet, 10)
                .unwrap()
                .is_empty()
        );
        let resolved = resolve(&conn, &deposited.item_id, "ack", "humain", 200)
            .unwrap()
            .unwrap();
        let decision_id = resolved.decision.unwrap().decision_id;
        let first = pending_decisions(&conn, HumanInboxProducer::Guichet, 10).unwrap();
        let second = pending_decisions(&conn, HumanInboxProducer::Guichet, 10).unwrap();
        assert_eq!(first.len(), 1, "la relève ne marque rien");
        assert_eq!(first, second);
        assert_eq!(first[0].decision_id, decision_id);
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Daemon, 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(ack(&conn, &decision_id, 300).unwrap(), Some(300));
        assert_eq!(
            ack(&conn, &decision_id, 400).unwrap(),
            Some(300),
            "idempotent"
        );
        assert_eq!(ack(&conn, "inconnue", 400).unwrap(), None);
        assert!(
            pending_decisions(&conn, HumanInboxProducer::Guichet, 10)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            get(&conn, &deposited.item_id).unwrap().unwrap().acked_at,
            Some(300)
        );
    }

    #[test]
    fn fermeture_par_le_producteur_et_rappel() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let deposited = deposit(&conn, request(&subject, &options))
            .unwrap()
            .unwrap();
        assert!(
            due_for_reminder(&conn, 100 + 3599, 3600)
                .unwrap()
                .is_empty()
        );
        assert_eq!(due_for_reminder(&conn, 100 + 3600, 3600).unwrap().len(), 1);
        mark_reminded(&conn, &deposited.item_id, 100 + 3600).unwrap();
        assert!(
            due_for_reminder(&conn, 100 + 3601, 3600)
                .unwrap()
                .is_empty()
        );
        assert!(close_self(&conn, &deposited.item_id, "objet disparu", 500).unwrap());
        assert!(!close_self(&conn, &deposited.item_id, "objet disparu", 501).unwrap());
        let item = get(&conn, &deposited.item_id).unwrap().unwrap();
        assert_eq!(item.state, HumanInboxState::ClosedSelf);
        assert!(item.context.contains("closed_reason"));
        assert_eq!(open_count(&conn).unwrap(), 0);
    }

    #[test]
    fn canal_externe_commande_factice_et_echec_consignes() {
        let conn = conn();
        let subject = subject();
        let options = vec!["ack".to_string()];
        let dir = std::env::temp_dir().join(format!("hi-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sink = dir.join("recu.json");
        let ok_config = HumanChannelConfig {
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                format!("cat > {}", sink.display()),
            ],
            reminder_after_secs: 3600,
        };
        let deposited = deposit_and_notify(&conn, request(&subject, &options), Some(&ok_config))
            .unwrap()
            .unwrap();
        let received: ChannelSummary =
            serde_json::from_str(&std::fs::read_to_string(&sink).unwrap()).unwrap();
        assert_eq!(received.item_id, deposited.item_id);
        assert_eq!(received.summary, "chaîne épuisée");
        assert!(!received.reminder);
        let attempts = notify_attempts(&conn, &deposited.item_id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert!(attempts[0].ok);

        let failing = HumanChannelConfig {
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "exit 3".to_string(),
            ],
            reminder_after_secs: 3600,
        };
        let reminded = remind_overdue(&conn, Some(&failing), 100 + 7200).unwrap();
        assert_eq!(reminded, 1);
        let attempts = notify_attempts(&conn, &deposited.item_id).unwrap();
        assert_eq!(attempts.len(), 2);
        assert!(!attempts[1].ok);
        assert!(
            attempts[1]
                .error
                .as_deref()
                .unwrap_or("")
                .contains("sortie 3")
        );
        assert_eq!(
            open_count(&conn).unwrap(),
            1,
            "l'échec du canal ne perd rien"
        );

        // Même oracle087 : stdin rempli et stderr continu ne prolongent pas
        // le budget commun. Le shell est le seul enfant et respecte SIGTERM.
        let blocked = HumanChannelConfig {
            command: vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "trap 'exit 0' TERM; while :; do printf 'canal bloque\\n' >&2; done".to_string(),
            ],
            reminder_after_secs: 3600,
        };
        let large_options = vec!["x".repeat(256 * 1024)];
        let started = std::time::Instant::now();
        let blocked_item = deposit_and_notify(
            &conn,
            DepositRequest {
                dedup_key: "canal-bloque",
                ..request(&subject, &large_options)
            },
            Some(&blocked),
        )
        .unwrap()
        .unwrap();
        assert!(
            started.elapsed() < std::time::Duration::from_secs(5),
            "stdin et stderr doivent rendre la main après le budget du canal"
        );
        let attempts = notify_attempts(&conn, &blocked_item.item_id).unwrap();
        assert_eq!(attempts.len(), 1);
        assert!(!attempts[0].ok);
        let error = attempts[0].error.as_deref().unwrap_or("");
        assert!(error.contains("délai canal dépassé"));
        assert!(
            !error.contains("; arrêt:"),
            "le canal factice doit être récolté après SIGTERM"
        );
        assert_eq!(
            open_count(&conn).unwrap(),
            2,
            "le dépôt bloqué reste durable"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn configuration_du_canal_exige_0600_et_chemin_absolu() {
        let dir = std::env::temp_dir().join(format!("hc-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("human-channel.json");
        assert_eq!(
            load_channel_config(&path).unwrap(),
            None,
            "absent = aucun canal"
        );
        std::fs::write(&path, r#"{"command":["/usr/bin/true"]}"#).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
            // Seules les permissions changent : un refus ne doit pas dépendre
            // du libellé humain ni être causé par une commande déjà invalide.
            assert!(load_channel_config(&path).is_err());
            assert_eq!(
                std::fs::read(&path).unwrap(),
                br#"{"command":["/usr/bin/true"]}"#
            );
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
            assert!(load_channel_config(&path).unwrap().is_some());
        }
        std::fs::write(&path, r#"{"command":["relatif/notify"]}"#).unwrap();
        assert!(load_channel_config(&path).unwrap_err().contains("absolu"));
        std::fs::write(
            &path,
            r#"{"command":["/usr/bin/true"],"reminder_after_secs":30}"#,
        )
        .unwrap();
        assert!(load_channel_config(&path).unwrap_err().contains("60"));
        std::fs::write(&path, r#"{"command":["/usr/bin/true"]}"#).unwrap();
        let config = load_channel_config(&path).unwrap().unwrap();
        assert_eq!(config.reminder_after_secs, DEFAULT_REMINDER_AFTER_SECS);
        let _ = std::fs::remove_dir_all(dir);
    }
