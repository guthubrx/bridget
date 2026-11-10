    use super::*;
    use std::time::Duration;

    fn base_snapshot(now: SystemTime) -> RepriseSnapshot {
        RepriseSnapshot {
            now,
            repo: PathBuf::from("/repo"),
            socket_path: PathBuf::from("/resolved/cache/bridget.sock"),
            db_path: PathBuf::from("/resolved/cache/bridget.db"),
            binary_cli: Some(PathBuf::from("/resolved/bin/bridget")),
            status: Ok(DaemonStatus {
                running: true,
                agents_inventory_available: true,
                agents: vec![AgentInfo {
                    agent_id: uuid::Uuid::new_v4().to_string(),
                    display_name: "coderBridget".to_string(),
                    agent_type: "codex".to_string(),
                    connection_id: "conn-1".to_string(),
                    host: "local".to_string(),
                    transport: "unix".to_string(),
                    channel: None,
                    state: "connected".to_string(),
                    last_seen_secs: 0,
                    reconnect_count: 0,
                    mode: None,
                    location: None,
                    os: "macOS".to_string(),
                    domain: None,
                    model: None,
                    effort: None,
                    rate_limits: Default::default(),
                    model_mismatch: None,
                    disk_space: None,
                    provider: None,
                    persistent: None,
                    execution: None,
                    agent_link: None,
                }],
                message_count: Some(3),
                build_id: Some("abc123".to_string()),
                // Le daemon de la fixture tourne AILLEURS : c'est le cas fédéré,
                // le seul où l'attribution est vérifiable.
                daemon_host: Some("monordinateur".to_string()),
                daemon_db_path: Some("/Users/moi/.cache/bridget/bridget.db".to_string()),
            }),
            open_requests: Ok(vec![]),
            recent_messages: Ok(vec![]),
            git: Ok(GitSnapshot {
                head: "deadbeef".to_string(),
                head_message: "test".to_string(),
                worktrees: vec![],
            }),
            pin: None,
            recovery_losses: Ok(None),
        }
    }

    #[test]
    fn bloc_0_expose_les_chemins_resolus_de_status_sans_reconstruire_home() {
        let card = render_card(&base_snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        ));
        assert!(
            card.contains("socket: /resolved/cache/bridget.sock"),
            "la carte doit recopier le chemin résolu de status: {card}"
        );
        assert!(card.contains("db_locale: /resolved/cache/bridget.db"));
        assert!(
            !card.contains("$HOME") && !card.contains("~/.cache"),
            "aucun chemin reconstruit depuis HOME: {card}"
        );
        assert!(card.contains("build_id_daemon: abc123"));
        assert!(card.contains("coderBridget (codex, connected)"));
        assert!(
            !card.contains("pertes_reprise"),
            "zéro perte ne doit pas faire de bruit: {card}"
        );
    }

    /// POINT 2 — la carte décrivait la flotte à partir d'un fichier qui n'est
    /// pas le sien. Chaque chemin doit désormais porter SA machine.
    ///
    /// La fixture place le daemon sur une AUTRE machine que le client : c'est la
    /// seule configuration où l'attribution est vérifiable.
    ///
    /// Mutant qui tue ce test : réafficher `snapshot.db_path` sous la clé `db`
    /// → l'assertion sur le chemin du daemon meurt, et celle qui interdit de
    /// présenter la base locale comme celle du daemon meurt aussi.
    #[test]
    fn la_carte_attribue_chaque_base_a_sa_machine() {
        let card = render_card(&base_snapshot(
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        ));
        assert!(
            card.contains("machine_daemon: monordinateur"),
            "la carte doit nommer la machine du daemon: {card}"
        );
        assert!(
            card.contains("db: /Users/moi/.cache/bridget/bridget.db"),
            "la clé db doit porter la base ATTESTÉE par le daemon: {card}"
        );
        assert!(
            card.contains("db_locale: /resolved/cache/bridget.db"),
            "la base d'ici doit être nommée comme locale: {card}"
        );
        assert!(
            !card.contains("db: /resolved/cache/bridget.db"),
            "la base locale ne doit jamais être présentée comme celle du daemon: {card}"
        );
    }

    /// POINT 2 — la trace de reprise locale décrit une AUTRE flotte quand la
    /// base du daemon n'est pas la nôtre : on ne la lit pas, et on dit pourquoi.
    ///
    /// Mutant qui tue ce test : lire la trace sans la garde → `recovery_losses`
    /// redevient `Ok(..)` et l'assertion meurt.
    #[test]
    fn la_trace_de_reprise_n_est_pas_lue_quand_la_base_est_ailleurs() {
        let config = DaemonConfig {
            socket_path: std::env::temp_dir()
                .join(format!("b89-reprise-{}.sock", uuid::Uuid::new_v4())),
            ..DaemonConfig::default()
        };
        let snapshot = collect_snapshot(
            &config,
            Path::new("/repo"),
            None,
            SystemTime::UNIX_EPOCH + Duration::from_secs(10),
        );
        let error = snapshot
            .recovery_losses
            .expect_err("sans base attestée, la trace locale ne doit pas être lue");
        // Daemon injoignable : rien n'est attesté. Le motif rendu doit être
        // celui de l'INDÉTERMINATION, pas celui de deux machines différentes —
        // les deux n'appellent pas le même geste de la part d'un opérateur.
        assert!(
            error.contains(MOTIF_INDETERMINATION),
            "le refus doit dire pourquoi: {error}"
        );
        assert!(
            !error.contains(MOTIF_BASES_DIFFERENTES),
            "le motif générique ne doit pas être rendu pour une indétermination: {error}"
        );
    }

    #[test]
    fn source_indisponible_se_declare_sans_inventer_ni_echouer_en_bloc() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.status = Ok(DaemonStatus {
            running: false,
            agents: vec![],
            agents_inventory_available: false,
            message_count: None,
            build_id: None,
            daemon_host: None,
            daemon_db_path: None,
        });
        snapshot.open_requests = Err("daemon hors ligne (socket absente)".to_string());
        snapshot.recent_messages = Err("daemon hors ligne (socket absente)".to_string());
        snapshot.git = Err("git indisponible: ENOENT".to_string());

        let card = render_card(&snapshot);
        assert!(card.contains("daemon_en_ligne: false"));
        assert!(card.contains("socket: /resolved/cache/bridget.sock"));
        assert!(card.contains("indisponible: daemon hors ligne"));
        assert!(card.contains("indisponible: git indisponible"));
        assert!(
            !card.contains("inventé") && !card.contains("/tmp/bridget.sock"),
            "pas de valeur inventée: {card}"
        );
    }

    #[test]
    fn sonde_identite_indisponible_ne_devient_pas_absence_d_agents() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.status =
            Err("identité du daemon indisponible: délai de lecture de 2 s dépassé".to_string());

        let card = render_card(&snapshot);
        assert!(card.contains("daemon_en_ligne: false"));
        assert!(card.contains("identité du daemon indisponible"));
        assert!(card.contains("délai de lecture de 2 s dépassé"));
        assert!(!card.contains("(aucun agent connecté)"));
    }

    #[test]
    fn carte_nomme_chaque_absent_de_la_trace_de_reprise() {
        let mut snapshot = base_snapshot(SystemTime::UNIX_EPOCH + Duration::from_secs(10));
        snapshot.recovery_losses = Ok(Some(crate::recovery_trace::RecoveryLossReport {
            schema: 1,
            recorded_at: 42,
            absents: vec![
                crate::recovery_trace::RecoveryLossEntry {
                    name: "cursor3".to_string(),
                    reason: crate::recovery_trace::REASON_NON_PERSISTENT.to_string(),
                    detail: Some("spawn sans --persistent".to_string()),
                },
                crate::recovery_trace::RecoveryLossEntry {
                    name: "cursor8".to_string(),
                    reason: crate::recovery_trace::REASON_QUOTA.to_string(),
                    detail: Some("quota de flotte atteint (8)".to_string()),
                },
            ],
        }));
        let card = render_card(&snapshot);
        assert!(card.contains("name: cursor3"));
        assert!(card.contains("reason: non_persistant"));
        assert!(card.contains("name: cursor8"));
        assert!(card.contains("reason: quota_flotte"));
    }

    #[test]
    fn pin_plus_vieux_d_une_heure_s_affiche_comme_perime() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
        let mut snapshot = base_snapshot(now);
        snapshot.pin = Some(Ok(LoadedPin {
            pin: PinFile {
                prochain: Some(PinProchain {
                    geste: "ne doit plus être affiché comme frais".to_string(),
                    proprietaire: Some("coderBridget".to_string()),
                }),
                couloirs: vec![PinCouloir {
                    id: "A".to_string(),
                    owner: "coderBridget".to_string(),
                    fichiers: vec![],
                    worktree: None,
                }],
                pieges: Some(PinPieges {
                    actifs_session: vec!["piège périmé".to_string()],
                }),
            },
            path: PathBuf::from("/repo/docs/carte-reprise.pin.yaml"),
            modified_at: now - PIN_STALE_AFTER - Duration::from_secs(1),
        }));

        assert!(is_pin_stale(
            now,
            now - PIN_STALE_AFTER - Duration::from_secs(1)
        ));
        let card = render_card(&snapshot);
        assert!(
            card.contains("PIN périmé") || card.contains("⚠ PIN périmé"),
            "pin périmé attendu: {card}"
        );
        assert!(
            !card.contains("ne doit plus être affiché comme frais"),
            "geste frais du pin périmé ne doit pas apparaître: {card}"
        );
    }

    #[test]
    fn pin_frais_expose_le_geste() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(10_000);
        let mut snapshot = base_snapshot(now);
        snapshot.pin = Some(Ok(LoadedPin {
            pin: PinFile {
                prochain: Some(PinProchain {
                    geste: "livrer carte reprise".to_string(),
                    proprietaire: None,
                }),
                ..PinFile::default()
            },
            path: PathBuf::from("/repo/docs/carte-reprise.pin.yaml"),
            modified_at: now - Duration::from_secs(60),
        }));
        let card = render_card(&snapshot);
        assert!(
            card.contains("geste: \"livrer carte reprise\"")
                || card.contains("geste: livrer carte reprise")
        );
    }

    #[test]
    fn pin_yaml_reel_est_charge_depuis_le_fichier() {
        let chemin = std::env::temp_dir().join(format!("b174-pin-{}.yaml", uuid::Uuid::new_v4()));
        fs::write(
            &chemin,
            "couloirs:\n  - id: A\n    owner: coderBridget\n    fichiers: [src/a.rs]\nprochain:\n  geste: livrer carte reprise\npieges:\n  actifs_session: [piège]\n",
        )
        .unwrap();
        let loaded = load_pin(&chemin);
        let _ = fs::remove_file(&chemin);
        let loaded = loaded.unwrap();
        assert_eq!(loaded.pin.couloirs[0].owner, "coderBridget");
        assert_eq!(loaded.pin.prochain.unwrap().geste, "livrer carte reprise");
        assert_eq!(
            loaded.pin.pieges.unwrap().actifs_session,
            vec!["piège".to_string()]
        );
    }

    #[test]
    fn pin_yaml_avec_balise_include_est_refuse() {
        let chemin = std::env::temp_dir().join(format!("b174-pin-{}.yaml", uuid::Uuid::new_v4()));
        fs::write(&chemin, "prochain:\n  geste: !include secret.yaml\n").unwrap();
        let loaded = load_pin(&chemin);
        let _ = fs::remove_file(&chemin);
        assert!(loaded.is_err_and(|motif| motif.contains("pin YAML invalide")));
    }

    /// M4 — le contrôle POSITIF qui manquait.
    ///
    /// Un mutant rendant la condition toujours fausse laissait toute la famille
    /// `reprise::tests` verte : le test existant ne prouvait que le REFUS
    /// distant, jamais qu'une trace locale attestée reste lisible. Les deux
    /// issues sont désormais éprouvées sur la même fonction.
    ///
    /// Mutant qui tue ce test : rendre la condition toujours fausse → le premier
    /// cas meurt en affichant le motif de refus qu'il n'aurait pas dû recevoir.
    #[test]
    fn la_trace_est_lue_ici_et_refusee_ailleurs() {
        let chemin = PathBuf::from("/home/moi/.cache/bridget/bridget.db");

        // POSITIF : même machine, même chemin → la trace nous concerne.
        assert_eq!(
            recovery_trace_scope(
                Some("cartae"),
                Some("/home/moi/.cache/bridget/bridget.db"),
                "cartae",
                &chemin
            ),
            Ok(()),
            "une base attestée locale doit rester lisible"
        );

        // NÉGATIF, et c'est le cas piégeux : MÊME CHEMIN, autre machine.
        let refus = recovery_trace_scope(
            Some("monordinateur"),
            Some("/home/moi/.cache/bridget/bridget.db"),
            "cartae",
            &chemin,
        )
        .expect_err("un daemon distant ne rend pas la trace locale lisible");
        assert!(
            refus.contains("sur cartae") && refus.contains("sur monordinateur"),
            "le motif doit nommer LES DEUX machines : {refus}"
        );
        // Contrôle de séparation dans l'autre sens : deux machines ATTESTÉES et
        // différentes doivent recevoir le motif générique, jamais celui de
        // l'indétermination.
        assert!(
            refus.contains(MOTIF_BASES_DIFFERENTES) && !refus.contains(MOTIF_INDETERMINATION),
            "deux machines nommées ne relèvent pas de l'indétermination : {refus}"
        );

        // Daemon antérieur : non attesté, donc refus explicite.
        let inconnu = recovery_trace_scope(None, None, "cartae", &chemin)
            .expect_err("sans attestation, la trace n'est pas réputée locale");
        assert!(
            inconnu.contains(crate::build_info::MACHINE_NON_ATTESTEE),
            "{inconnu}"
        );

        // DEUX HÔTES INDÉTERMINÉS, MÊME CHEMIN — le cas de rc7. Refusé, et le
        // motif doit dire POURQUOI : machine non attestée, pas « deux machines
        // différentes ». Les deux refus n'appellent pas le même geste.
        let repli = recovery_trace_scope(
            Some(bridget_core::HOTE_NON_ATTESTE),
            Some("/home/moi/.cache/bridget/bridget.db"),
            bridget_core::HOTE_NON_ATTESTE,
            &chemin,
        )
        .expect_err("deux machines indéterminées ne sont pas la même machine");
        // LES DEUX CONDITIONS. La mention « machine non attestée » figure AUSSI
        // dans le motif générique, qui affiche l'hôte du daemon : l'exiger seule
        // ne sépare pas les issues. Il faut le motif spécifique PRÉSENT et le
        // motif générique ABSENT — sinon un code qui ne changerait rien du tout
        // passerait le test.
        assert!(
            repli.contains(MOTIF_INDETERMINATION),
            "le motif doit nommer l'indétermination : {repli}"
        );
        assert!(
            !repli.contains(MOTIF_BASES_DIFFERENTES),
            "le motif générique des bases différentes ne doit PAS être rendu ici : {repli}"
        );
        // Rendu unique des DEUX côtés : la machine locale non attestée se lit
        // comme telle, jamais sous sa valeur de repli brute.
        assert!(
            !repli.contains(&format!("sur {}", bridget_core::HOTE_NON_ATTESTE)),
            "la machine locale non attestée doit se lire comme telle : {repli}"
        );
    }
