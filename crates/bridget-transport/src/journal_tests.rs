    #[test]
    fn spec100_observation_requires_confirmed_structured_write() {
        use super::*;
        let payload = confirmed_write_payload("src/lib.rs", Path::new("/project")).unwrap();
        let entry = JournalEntry::new(1, "now", "s", "update", None, payload);
        assert_eq!(
            observation_metadata(&entry),
            Some((
                crate::protocol::ObservationKind::FileWritten,
                Some("/project/src/lib.rs".into())
            ))
        );
        let attempt = JournalEntry::new(
            2,
            "now",
            "s",
            "update",
            None,
            serde_json::json!({"kind":"file","text":"/project/src/lib.rs"}),
        );
        assert!(observation_metadata(&attempt).is_none());
        assert!(tool_write_path("Read", &serde_json::json!({"file_path":"/p/x"})).is_none());
        assert!(tool_write_path("Bash", &serde_json::json!({"command":"echo x > /p/x"})).is_none());
        assert!(confirmed_write_payload("x", Path::new("relative")).is_none());
    }

    #[test]
    fn spec100_observation_feed_is_independent_and_bounded() {
        use super::*;
        let feed = JournalLiveFeed::default();
        for seq in 1..=WRITER_QUEUE_CAPACITY {
            feed.observation_sender
                .try_send(crate::protocol::WrapperToDaemon::ObservedActivity {
                    seq: seq as u64,
                    event: crate::protocol::ObservationKind::PermissionRequired,
                    file: None,
                })
                .unwrap();
        }
        assert!(
            feed.observation_sender
                .try_send(crate::protocol::WrapperToDaemon::ObservedActivity {
                    seq: 999,
                    event: crate::protocol::ObservationKind::PermissionRequired,
                    file: None,
                })
                .is_err()
        );
        let (events, _) = feed.take_observations();
        assert_eq!(events.len(), WRITER_QUEUE_CAPACITY);
        assert!(feed.after(None).events.is_empty());
    }

    #[test]
    fn spec100_post_flush_turns_do_not_reobserve_notifications() {
        let root = root("spec100-post-flush");
        let feed = JournalLiveFeed::default();
        let writer = JournalWriter::start_with_live_feed(
            &root,
            "agent",
            "session",
            Arc::new(Mutex::new(AcpEventQueue::default())),
            Some(feed.clone()),
        )
        .unwrap();
        writer
            .enqueue(
                "turn_end",
                Some("human"),
                json!({"stop_reason":"cancelled"}),
            )
            .unwrap();
        writer
            .enqueue(
                "turn_end",
                Some("bridget-observation:123"),
                json!({"stop_reason":"completed"}),
            )
            .unwrap();
        writer
            .enqueue("turn_end", Some("unattributed-native"), json!({}))
            .unwrap();
        writer
            .enqueue(
                "update",
                Some("bridget-observation:123"),
                confirmed_write_payload("/p/x", Path::new("/p")).unwrap(),
            )
            .unwrap();
        writer
            .enqueue(
                "update",
                Some("human"),
                confirmed_write_payload("/p/x", Path::new("/p")).unwrap(),
            )
            .unwrap();
        writer.stop();
        let (facts, lost) = feed.take_observations();
        assert_eq!(lost, 0);
        assert_eq!(facts.len(), 2);
        assert!(matches!(
            facts[0],
            crate::protocol::WrapperToDaemon::ObservedActivity {
                seq: 1,
                event: crate::protocol::ObservationKind::TurnEnded,
                ..
            }
        ));
        assert!(matches!(
            facts[1],
            crate::protocol::WrapperToDaemon::ObservedActivity {
                seq: 5,
                event: crate::protocol::ObservationKind::FileWritten,
                ..
            }
        ));
        assert_eq!(
            feed.after(None).events.len(),
            5,
            "le journal attach n'est pas consommé par les observations"
        );
    }
    use super::*;
    use serde_json::json;

    fn root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bridget-journal-{name}-{}", std::process::id()))
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn sonde_complete_ne_modifie_pas_les_deux_bornes_historiques() {
        for (all, suffix, expected) in [
            (false, "probe-boundaries", vec![1, 3]),
            (true, "probe-all", vec![1, 2, 3]),
        ] {
            let root = root(suffix);
            let probe = if all {
                AppendLatencyProbe::install_all_events(&root)
            } else {
                AppendLatencyProbe::install(&root)
            };
            let writer = JournalWriter::start(
                &root,
                "probe",
                "session",
                Arc::new(Mutex::new(AcpEventQueue::default())),
            )
            .unwrap();
            for event in ["turn_start", "reasoning", "turn_end"] {
                writer.enqueue(event, Some("message"), json!({})).unwrap();
            }
            writer.stop();
            // Mutation all_events toujours vrai : le premier oracle reçoit
            // [1,2,3] ; toujours faux : le second perd la séquence 2.
            assert_eq!(
                probe
                    .take_samples()
                    .iter()
                    .map(|s| s.seq)
                    .collect::<Vec<_>>(),
                expected
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn sequence_survives_rotation_and_payload_is_versioned() {
        let root = root("rotation");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        assert_eq!(
            journal
                .append_at(
                    "2026-08-22",
                    "2026-08-22T00:00:00Z",
                    "turn_start",
                    Some("m1"),
                    json!({"from":"alice","reply":true,"body":"bonjour"})
                )
                .unwrap(),
            1
        );
        assert_eq!(
            journal
                .append_at(
                    "2026-08-23",
                    "2026-08-23T00:00:00Z",
                    "turn_end",
                    Some("m1"),
                    json!({"stop_reason":"end_turn","routed_to":"alice"})
                )
                .unwrap(),
            2
        );
        let event = valid_events(&root.join("codex-1/2026-08-23.jsonl"))
            .pop()
            .unwrap();
        assert_eq!(event["v"], 1);
        assert_eq!(event["seq"], 2);
        assert_eq!(event["payload"]["routed_to"], "alice");
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(root.join("codex-1/2026-08-23.jsonl"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn next_sequence_recovers_from_normal_empty_partial_and_corrupt_tails() {
        for (name, contents, expected) in [
            ("normal", "{\"v\":1,\"seq\":4}\n", 5),
            ("empty", "", 1),
            ("partial", "{\"v\":1,\"seq\":4}\n{\"v\":1,\"seq\":", 5),
            ("corrupt", "{\"v\":1,\"seq\":4}\nnot-json\n", 5),
        ] {
            let root = root(name);
            let directory = root.join("codex-1");
            create_private_dir(&directory).unwrap();
            fs::write(directory.join("2026-08-22.jsonl"), contents).unwrap();
            let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
            assert_eq!(
                journal
                    .append_at(
                        "2026-08-23",
                        "2026-08-23T00:00:00Z",
                        "error",
                        None,
                        json!({"reason":"test"})
                    )
                    .unwrap(),
                expected
            );
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn partial_tail_is_isolated_before_a_same_day_append() {
        let root = root("same-day-tail");
        let directory = root.join("codex-1");
        create_private_dir(&directory).unwrap();
        let path = directory.join("2026-08-22.jsonl");
        fs::write(&path, "{\"v\":1,\"seq\":4}\n{\"v\":1,\"seq\":").unwrap();
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        assert_eq!(
            journal
                .append_at(
                    "2026-08-22",
                    "2026-08-22T01:00:00Z",
                    "error",
                    None,
                    json!({"reason":"reprise"})
                )
                .unwrap(),
            5
        );
        let entries = valid_events(&path);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry["seq"].as_u64())
                .collect::<Vec<_>>(),
            vec![Some(4), Some(5)]
        );
        assert_eq!(entries[1]["payload"]["reason"], "reprise");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_omits_an_absent_message_id() {
        let root = root("optional-message-id");
        let writer = JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(AcpEventQueue::default())),
        )
        .unwrap();
        writer
            .enqueue("error", None, json!({"reason":"global"}))
            .unwrap();
        writer.stop();
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let event = valid_events(&path).pop().unwrap();
        assert!(event.get("message_id").is_none());
        assert_eq!(event["payload"], json!({"reason":"global"}));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_journal_writer_refuse_un_kind_hors_vocabulaire() {
        let root = root("act-kind-contract");
        let writer = JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(AcpEventQueue::default())),
        )
        .unwrap();
        writer
            .enqueue("update", Some("m1"), json!({"kind":"tool","text":"Read"}))
            .expect("écriture tool valide doit passer d'abord");
        let rejected = writer.enqueue(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected
                .as_ref()
                .is_err_and(|detail| detail.contains("hors vocabulaire")),
            "TEMOIN: écriture hors vocabulaire doit être refusée, got {rejected:?}"
        );
        writer.stop();
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_session_journal_append_at_refuse_un_kind_hors_vocabulaire() {
        // Bouche le trou « mutant append_at-retirée » : enqueue seul ne suffit
        // pas — append_at est aussi un point d'entrée public.
        let root = root("act-kind-append-at");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        journal
            .append("update", Some("m1"), json!({"kind":"tool","text":"Read"}))
            .expect("écriture tool valide doit passer d'abord");
        let rejected = journal.append(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected.as_ref().is_err_and(|err| {
                err.kind() == std::io::ErrorKind::InvalidInput
                    && err.to_string().contains("hors vocabulaire")
            }),
            "TEMOIN append_at: hors vocabulaire doit être refusé, got {rejected:?}"
        );
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_append_entry_refuse_meme_si_enqueue_contourne() {
        // Dette 1 : garde sur le chemin writer. Preuve d'abord qu'un cas valide
        // passe — sinon l'oracle d'absence tourne sur une projection vide.
        let root = root("act-kind-append-entry");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        journal
            .append_entry_bypassing_public_guards_for_test(
                "update",
                Some("m1"),
                json!({"kind":"tool","text":"Read"}),
            )
            .expect("append_entry tool valide doit passer d'abord");
        let rejected = journal.append_entry_bypassing_public_guards_for_test(
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            rejected.as_ref().is_err_and(|err| {
                err.kind() == std::io::ErrorKind::InvalidInput
                    && err.to_string().contains("hors vocabulaire")
            }),
            "TEMOIN append_entry: hors vocabulaire doit être refusé même hors enqueue, got {rejected:?}"
        );
        let path = std::fs::read_dir(root.join("codex-1"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = valid_events(&path);
        assert_eq!(
            entries.len(),
            1,
            "le kind refusé ne doit jamais atteindre le JSONL"
        );
        assert_eq!(entries[0]["payload"]["kind"], "tool");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_retire_garde_append_entry_tue_TEMOIN_append_entry() {
        // Simule le trou d'avant la dette 1 : append_entry écrit sans valider.
        fn broken_append_entry_no_guard(
            journal: &mut SessionJournal,
            event: &str,
            message_id: Option<&str>,
            payload: Value,
        ) -> std::io::Result<JournalLiveEvent> {
            let (_, timestamp) = now_date_and_timestamp();
            let mut entry = JournalEntry::new(0, &timestamp, "", event, message_id, payload);
            let (date, timestamp) = now_date_and_timestamp();
            entry.seq = journal.next_seq;
            entry.ts = timestamp;
            entry.session_id.clone_from(&journal.session_id);
            let seq = entry.seq;
            let bytes = journal.append_entry_at(&date, entry)?;
            journal.next_seq = journal.next_seq.saturating_add(1);
            Ok(JournalLiveEvent { seq, bytes })
        }
        let root = root("act-kind-append-entry-mutant");
        let mut journal = SessionJournal::new(&root, "codex-1", "session-1").unwrap();
        broken_append_entry_no_guard(
            &mut journal,
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        )
        .expect("le mutant laisse passer");
        let healthy = journal.append_entry_bypassing_public_guards_for_test(
            "update",
            Some("m2"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        assert!(
            healthy
                .as_ref()
                .is_err_and(|err| err.to_string().contains("hors vocabulaire")),
            "TEMOIN_append_entry doit mourir si la garde writer est retirée: {healthy:?}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn writer_emits_a_terminal_event_when_an_append_fails() {
        let root = root("write-failure");
        let events = Arc::new(Mutex::new(AcpEventQueue::default()));
        let writer = JournalWriter::start(&root, "codex-1", "session-1", events.clone()).unwrap();
        fs::remove_dir_all(root.join("codex-1")).unwrap();
        writer
            .enqueue("error", None, json!({"reason":"test"}))
            .unwrap();
        for _ in 0..20 {
            if events
                .lock()
                .unwrap()
                .iter()
                .any(|event| matches!(event, AcpEvent::JournalFailed { .. }))
            {
                writer.stop();
                fs::remove_dir_all(root).unwrap();
                return;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
        panic!("l'échec d'écriture du journal n'est pas devenu terminal");
    }

    #[test]
    fn versioned_reader_fixtures_skip_partial_and_corrupt_lines() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        assert_eq!(valid_events(&fixtures.join("complete-turn.jsonl")).len(), 3);
        assert_eq!(valid_events(&fixtures.join("error.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("permission.jsonl")).len(), 1);
        let rotation = fixtures.join("rotation");
        let mut rotated = std::fs::read_dir(rotation)
            .unwrap()
            .filter_map(Result::ok)
            .flat_map(|entry| valid_events(&entry.path()))
            .collect::<Vec<_>>();
        rotated.sort_by_key(|event| event["seq"].as_u64());
        assert_eq!(
            rotated
                .iter()
                .map(|event| event["seq"].as_u64())
                .collect::<Vec<_>>(),
            vec![Some(5), Some(6)]
        );
        assert_eq!(valid_events(&fixtures.join("partial-tail.jsonl")).len(), 1);
        assert_eq!(valid_events(&fixtures.join("corrupt-line.jsonl")).len(), 1);
    }

    #[test]
    fn lecteur_incremental_garde_la_queue_partielle_et_signale_la_ligne_corrompue() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let mut partial = IncrementalJournalReader::new(fixtures.join("partial-tail.jsonl"));
        let mut partial_items = Vec::new();
        for _ in 0..32 {
            partial_items.extend(partial.read_chunk(7).unwrap());
        }
        assert_eq!(
            partial_items
                .iter()
                .filter(|item| matches!(item, JournalReadItem::Event(_)))
                .count(),
            1
        );
        assert_eq!(
            partial_items
                .iter()
                .filter(|item| matches!(item, JournalReadItem::Unreadable(_)))
                .count(),
            1
        );

        let root = root("tail-without-newline");
        create_private_dir(&root).unwrap();
        let path = root.join("journal.jsonl");
        fs::write(&path, b"{\"v\":1,\"seq\":9").unwrap();
        assert!(
            IncrementalJournalReader::new(&path)
                .read_chunk(64)
                .unwrap()
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();

        let mut corrupt = IncrementalJournalReader::new(fixtures.join("corrupt-line.jsonl"));
        let items = corrupt.read_chunk(16 * 1024).unwrap();
        assert!(matches!(items[0], JournalReadItem::Event(_)));
        assert!(
            matches!(items[1], JournalReadItem::Unreadable(JournalUnreadableLine { line: 2, offset }) if offset > 0)
        );
    }

    #[test]
    fn lecteur_borne_une_ligne_surdimensionnee_et_reprend_apres_newline() {
        let root = root("oversized");
        create_private_dir(&root).unwrap();
        let path = root.join("x.jsonl");
        fs::write(
            &path,
            [
                b"{\"v\":1,\"seq\":42,\"x\":\"".as_slice(),
                &vec![b'x'; MAX_INCREMENTAL_LINE_BYTES + 32],
                b"\"}\n{\"v\":1,\"seq\":43}\n",
            ]
            .concat(),
        )
        .unwrap();
        let mut reader = IncrementalJournalReader::new(&path);
        let mut seen = Vec::new();
        for _ in 0..80 {
            seen.extend(reader.read_chunk(65_536).unwrap());
            assert!(reader.buffered_len() <= MAX_INCREMENTAL_LINE_BYTES);
        }
        assert!(matches!(
            seen.iter()
                .find(|item| matches!(item, JournalReadItem::Oversized { .. })),
            Some(JournalReadItem::Oversized { seq: Some(42), .. })
        ));
        assert!(matches!(
            seen.last(),
            Some(JournalReadItem::Event(JournalReadEvent { seq: 43, .. }))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lecteur_mesure_la_borne_sur_la_premiere_ligne_et_non_le_tampon_entier() {
        fn valid_line(seq: u64, length: usize) -> Vec<u8> {
            let prefix = format!("{{\"v\":1,\"seq\":{seq},\"body\":\"").into_bytes();
            let suffix = b"\"}";
            assert!(length >= prefix.len() + suffix.len());
            [
                prefix.as_slice(),
                &vec![b'x'; length - prefix.len() - suffix.len()],
                suffix,
            ]
            .concat()
        }

        let root = root("incremental-boundaries");
        create_private_dir(&root).unwrap();

        let near = root.join("near.jsonl");
        let mut near_contents = valid_line(40, MAX_INCREMENTAL_LINE_BYTES - 8);
        near_contents.extend_from_slice(b"\n{\"v\":1,\"seq\":41}\n");
        fs::write(&near, near_contents).unwrap();
        let mut near_reader = IncrementalJournalReader::new(&near);
        let near_items = near_reader
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 64)
            .unwrap();
        assert!(
            near_items
                .iter()
                .all(|item| !matches!(item, JournalReadItem::Oversized { .. }))
        );
        assert_eq!(
            near_items
                .iter()
                .filter_map(|item| match item {
                    JournalReadItem::Event(event) => Some(event.seq),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![40, 41]
        );

        let exact = root.join("exact.jsonl");
        let mut exact_contents = valid_line(42, MAX_INCREMENTAL_LINE_BYTES);
        exact_contents.push(b'\n');
        fs::write(&exact, exact_contents).unwrap();
        let exact_items = IncrementalJournalReader::new(&exact)
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 1)
            .unwrap();
        assert!(matches!(
            exact_items.as_slice(),
            [JournalReadItem::Event(JournalReadEvent { seq: 42, .. })]
        ));

        let too_large = root.join("too-large.jsonl");
        let mut too_large_contents = valid_line(43, MAX_INCREMENTAL_LINE_BYTES + 1);
        too_large_contents.push(b'\n');
        fs::write(&too_large, too_large_contents).unwrap();
        let too_large_items = IncrementalJournalReader::new(&too_large)
            .read_chunk(MAX_INCREMENTAL_LINE_BYTES + 2)
            .unwrap();
        assert!(matches!(
            too_large_items.as_slice(),
            [JournalReadItem::Oversized { seq: Some(43), .. }]
        ));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fenetres_resolvent_rotation_seq_inclusif_et_vide() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/journal");
        let rotation = resolve_window(
            &fixtures.join("rotation"),
            &AttachWindow::Seq(5),
            "2026-08-24",
        )
        .unwrap();
        assert_eq!(rotation.files.len(), 2);
        assert_eq!(rotation.from_seq, Some(5));

        let mut first = IncrementalJournalReader::new(&rotation.files[0]);
        let mut first_items = Vec::new();
        while first.next_offset() < fs::metadata(&rotation.files[0]).unwrap().len() {
            first_items.extend(first.read_chunk(9).unwrap());
        }
        assert_eq!(
            first.next_offset(),
            fs::metadata(&rotation.files[0]).unwrap().len()
        );
        assert!(
            matches!(first_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 5 && event.offset == 0 && event.line == 1)
        );
        assert!(
            first.read_chunk(9).unwrap().is_empty(),
            "le premier fichier est consommé avant la rotation"
        );

        let mut second = IncrementalJournalReader::new(&rotation.files[1]);
        let mut second_items = Vec::new();
        while second.next_offset() < fs::metadata(&rotation.files[1]).unwrap().len() {
            second_items.extend(second.read_chunk(9).unwrap());
        }
        assert_eq!(
            second.next_offset(),
            fs::metadata(&rotation.files[1]).unwrap().len()
        );
        assert!(
            matches!(second_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 6 && event.offset == 0 && event.line == 1)
        );
        assert_eq!(
            first_items
                .iter()
                .chain(second_items.iter())
                .filter_map(|item| match item {
                    JournalReadItem::Event(event) => Some(event.seq),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            vec![5, 6]
        );

        let partial_tail = fixtures.join("partial-tail.jsonl");
        let mut old = IncrementalJournalReader::new(&partial_tail);
        let mut old_items = Vec::new();
        while old.next_offset() < fs::metadata(&partial_tail).unwrap().len() {
            old_items.extend(old.read_chunk(7).unwrap());
        }
        assert!(
            matches!(old_items.first(), Some(JournalReadItem::Event(event)) if event.seq == 7 && event.offset == 0)
        );
        assert!(matches!(
            old_items.get(1),
            Some(JournalReadItem::Unreadable(_))
        ));
        assert!(
            old.read_chunk(7).unwrap().is_empty(),
            "la queue partielle ancienne ne bloque pas la rotation"
        );
        let partial_root = root("partial-tail-before-next");
        create_private_dir(&partial_root).unwrap();
        let path_after_partial = partial_root.join("2026-08-25.jsonl");
        fs::write(&path_after_partial, b"{\"v\":1,\"seq\":8}\n").unwrap();
        let mut next = IncrementalJournalReader::new(path_after_partial);
        let next_items = next.read_chunk(32).unwrap();
        assert!(
            matches!(next_items.as_slice(), [JournalReadItem::Event(event)] if event.seq == 8 && event.offset == 0)
        );
        fs::remove_dir_all(partial_root).unwrap();

        let empty_root = root("empty-window");
        create_private_dir(&empty_root).unwrap();
        assert!(
            resolve_window(&empty_root, &AttachWindow::Today, "2026-08-22")
                .unwrap()
                .files
                .is_empty()
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("2026-08-23".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::FutureDate)
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("bad".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::InvalidDate)
        );
        assert_eq!(
            resolve_window(
                &empty_root,
                &AttachWindow::Date("2026-08-21".to_string()),
                "2026-08-22"
            ),
            Err(JournalWindowError::DateOutsideRetention)
        );
        fs::remove_dir_all(&empty_root).unwrap();

        let tail_root = root("tail-window");
        create_private_dir(&tail_root).unwrap();
        let mut lines = String::new();
        for seq in 1..=100 {
            lines.push_str(&format!("{{\"v\":1,\"seq\":{seq}}}\n"));
        }
        fs::write(tail_root.join("2026-08-22.jsonl"), lines).unwrap();
        let tail = resolve_window(&tail_root, &AttachWindow::Tail(30), "2026-08-22").unwrap();
        assert_eq!(tail.from_seq, Some(71));
        let older = resolve_window(&tail_root, &AttachWindow::Seq(21), "2026-08-22").unwrap();
        assert_eq!(older.from_seq, Some(21));
        assert!(
            older.from_seq.unwrap() < tail.from_seq.unwrap(),
            "from_seq plus bas doit donner l'ancien au-delà du Tail"
        );
        fs::remove_dir_all(tail_root).unwrap();

        assert!(!is_date("2026-02-31"));
        assert!(is_date("2024-02-29"));
        assert!(!is_date("2026-02-29"));
        assert!(!is_date("2026-04-31"));
    }
