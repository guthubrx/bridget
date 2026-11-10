    use super::*;
    use std::os::unix::fs::DirBuilderExt;

    const SENDER: &str = "89000000-0000-4000-8000-000000000301";
    const TARGET: &str = "89000000-0000-4000-8000-000000000302";

    fn run_pressure(mode: PresenceMode, locked: bool) {
        let root = std::env::temp_dir().join(format!(
            "cp{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let config = DaemonConfig {
            socket_path: root.join("s"),
            db_path: root.join("state.db"),
            log_path: root.join("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let (managed_tx, _managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx).unwrap();
        let mut peers = Vec::new();
        for (conn, agent, mode) in [
            ("sender", SENDER, PresenceMode::Cli),
            ("target", TARGET, mode),
        ] {
            let (daemon, peer) = UnixStream::pair().unwrap();
            state
                .connections
                .insert(conn.into(), Arc::new(Mutex::new(BufWriter::new(daemon))));
            let registered = handle_register_with_channel(
                conn,
                2,
                "fixture".into(),
                agent.into(),
                Some("test".into()),
                Some("acp".into()),
                ChannelReport::Known("unix".into()),
                Some(mode),
                None,
                Some("test".into()),
                Some(format!("instance-{conn}")),
                None,
                false,
                Some(false),
                &mut state,
                false,
            );
            assert!(
                matches!(registered, DaemonToWrapper::Registered { .. }),
                "{registered:?}"
            );
            peers.push(peer);
        }
        state
            .store
            .create_request("cancel-pressure", SENDER, TARGET, 60)
            .unwrap();
        let writer = Arc::clone(state.connections.get("target").unwrap());
        if !locked {
            let locked = writer.lock().unwrap();
            locked.get_ref().set_nonblocking(true).unwrap();
            let bytes = [b'x'; 4096];
            let mut sent = 0usize;
            loop {
                let n = unsafe {
                    libc::send(
                        locked.get_ref().as_raw_fd(),
                        bytes.as_ptr().cast(),
                        bytes.len(),
                        libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
                    )
                };
                if n < 0 {
                    assert_eq!(
                        std::io::Error::last_os_error().kind(),
                        std::io::ErrorKind::WouldBlock
                    );
                    break;
                }
                assert!(n > 0);
                sent += n as usize;
                assert!(sent < 16 * 1024 * 1024, "la socket ne sature pas");
            }
            locked.get_ref().set_nonblocking(false).unwrap();
            // Filet du harnais seulement : nettement au-delà de l'oracle 1,9 s.
            // Le mutant bloquant doit échouer, jamais retenir un thread du banc.
            locked
                .get_ref()
                .set_write_timeout(Some(Duration::from_secs(4)))
                .unwrap();
            assert!(sent > 0, "aucun octet réellement envoyé");
        }
        let held_writer = locked.then(|| writer.lock().unwrap());
        let state = Arc::new(Mutex::new(state));
        let worker_state = Arc::clone(&state);
        let (done_tx, done_rx) = mpsc::channel();
        // Le fait durable est observé via une connexion SQLite indépendante :
        // cette barrière ne dépend ni du mutex global ni d'un sleep métier.
        let db = rusqlite::Connection::open_with_flags(
            &config.db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        db.busy_timeout(Duration::from_millis(50)).unwrap();
        let started = Instant::now();
        let worker = thread::spawn(move || {
            let result = handle_wrapper_message(
                "sender",
                WrapperToDaemon::CancelRequest {
                    id: "cancel-pressure".into(),
                    sender: SENDER.into(),
                    reason: Some("priorité changée".into()),
                },
                &worker_state,
            );
            let _ = done_tx.send(result);
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut durable = false;
        while Instant::now() < deadline {
            durable = db
                .query_row(
                    "SELECT state = 'cancelled' FROM tracked_requests WHERE id = 'cancel-pressure'",
                    [],
                    |r| r.get::<_, bool>(0),
                )
                .unwrap_or(false);
            if durable {
                break;
            }
            thread::yield_now();
        }
        let state_deadline = Instant::now() + Duration::from_millis(200);
        let mut state_available = false;
        while Instant::now() < state_deadline {
            if let Ok(guard) = state.try_lock() {
                state_available = true;
                drop(guard);
                break;
            }
            thread::yield_now();
        }
        let result = done_rx.recv_timeout(Duration::from_secs(2));
        let elapsed = started.elapsed();
        // Nettoyage AVANT les assertions : le mutant ancien reste bloqué ; la
        // libération du verrou et le filet socket de 4 s bornent son nettoyage.
        drop(held_writer);
        for peer in &peers {
            let _ = peer.shutdown(std::net::Shutdown::Both);
        }
        worker.join().unwrap();
        let stored = state
            .lock()
            .unwrap()
            .store
            .get_request("cancel-pressure")
            .unwrap()
            .unwrap();
        drop(db);
        drop(state);
        drop(writer);
        drop(peers);
        std::fs::remove_dir_all(&root).unwrap();
        assert!(
            durable,
            "l'annulation doit être commitée même si la notification échoue"
        );
        assert!(
            state_available,
            "le destinataire retient le mutex global après le commit"
        );
        assert!(
            elapsed < Duration::from_millis(1900),
            "annulation non bornée : {elapsed:?}"
        );
        assert!(
            matches!(result, Ok(Some(DaemonToWrapper::RequestCancelled { state, .. })) if state == "cancelled"),
            "réponse d'annulation absente"
        );
        assert_eq!(stored.state, "cancelled");
        assert_eq!(stored.cancel_reason.as_deref(), Some("priorité changée"));
    }

    #[test]
    fn socket_saturee_ne_bloque_ni_annulation_ni_etat_global() {
        // Deux voies historiques, aucune ne peut redevenir un envoi libre bloquant.
        run_pressure(PresenceMode::Acp, false);
        run_pressure(PresenceMode::Cli, false);
    }

    #[test]
    fn verrou_writer_occupe_ne_bloque_ni_annulation_ni_etat_global() {
        run_pressure(PresenceMode::Acp, true);
    }

    #[test]
    fn notification_conserve_les_octets_tamponnes_et_le_timeout_du_writer() {
        let (socket, peer) = UnixStream::pair().unwrap();
        let previous = Some(Duration::from_millis(270));
        socket.set_write_timeout(previous).unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(socket)));
        let prefix = b"{ \"type\" : \"pong\" }\n";
        writer.lock().unwrap().write_all(prefix).unwrap();
        let message = DaemonToWrapper::CancelDelivery {
            id: "request-exact".into(),
            reason: "annulation attestée".into(),
            interrupt_active: None,
        };
        push_control_message_until(&writer, &message, Instant::now() + Duration::from_secs(1))
            .unwrap();
        let mut reader = BufReader::new(peer);
        let mut first = Vec::new();
        reader.read_until(b'\n', &mut first).unwrap();
        assert_eq!(
            first, prefix,
            "le préfixe doit rester octet-identique et unique"
        );
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(
            matches!(decode::<DaemonToWrapper>(line.trim_end()).unwrap(),
            DaemonToWrapper::CancelDelivery { id, reason, .. }
                if id == "request-exact" && reason == "annulation attestée")
        );
        let writer = writer.lock().unwrap();
        assert!(writer.buffer().is_empty(), "aucun tampon à redélivrer");
        assert_eq!(writer.get_ref().write_timeout().unwrap(), previous);
    }
