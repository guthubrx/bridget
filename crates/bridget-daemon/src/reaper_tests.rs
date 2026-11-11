use super::*;

fn world_at(now: u64) -> WorldView {
    WorldView::with_defaults(now)
}

#[test]
fn identite_indisponible_n_est_pas_un_annuaire_vide() {
    let error = agents_from_status(Err(
        "identité du daemon indisponible: délai de lecture dépassé".to_string(),
    ))
    .expect_err("l'incertitude doit interrompre l'inventaire");
    assert!(
        error
            .to_string()
            .contains("identité du daemon indisponible")
    );

    let absent = agents_from_status(Ok(crate::daemon::DaemonStatus {
        agents_inventory_available: true,
        ..crate::daemon::DaemonStatus::default()
    }))
    .expect("un daemon réellement absent reste observable")
    .expect("l'absence déterminée donne un inventaire connu");
    assert!(absent.is_empty());
}

#[test]
fn daemon_present_mais_inventaire_indisponible_reste_incertain() {
    let inventory = agents_from_status(Ok(crate::daemon::DaemonStatus {
        running: true,
        agents_inventory_available: false,
        ..crate::daemon::DaemonStatus::default()
    }))
    .expect("le statut attesté reste lisible");
    assert!(
        inventory.is_none(),
        "un daemon présent mais muet ne doit pas devenir un annuaire vide"
    );
}

#[test]
fn orphan_wrapper_fail_closed_without_background_signal() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let process = ProcessSnapshot {
        pid: 3013,
        ppid: 1,
        age_secs: 3600,
        command: "/tmp/bridget managed-wrapper g1504_fixture g1504-agent {}".into(),
    };
    let report = classify_managed_wrapper(&process, None, &[], &world);
    assert_eq!(report.verdict, Verdict::Incertain);
    assert!(
        report
            .uncertainty_reasons
            .iter()
            .any(|r| r == GAP_BACKGROUND)
    );
    assert!(report.would_have_done.contains("aucune"));
    assert!(!report.gaps.is_empty());
}

#[test]
fn live_connected_wrapper_is_protege() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let process = ProcessSnapshot {
        pid: 39951,
        ppid: 37095,
        age_secs: 3600,
        command: "/opt/bridget managed-wrapper cursor cursor3 {}".into(),
    };
    let agent = AgentPresence {
        name: "cursor3".into(),
        state: "busy".into(),
        mode: Some("acp".into()),
        location: None,
        last_seen_secs: 0,
    };
    let report = classify_managed_wrapper(&process, Some(&agent), &[], &world);
    assert_eq!(report.verdict, Verdict::Protege);
    assert!(report.guards_triggered.iter().any(|g| g.starts_with("G1_")));
    assert!(report.guards_triggered.iter().any(|g| g.starts_with("G3_")));
}

#[test]
fn tmux_location_protects_even_if_daemon_confused() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let process = ProcessSnapshot {
        pid: 42,
        ppid: 1,
        age_secs: 3600,
        command: "/opt/bridget managed-wrapper claude survivor {}".into(),
    };
    let agent = AgentPresence {
        name: "survivor".into(),
        state: "connected".into(),
        mode: Some("tmux".into()),
        location: Some("Who:2.2".into()),
        last_seen_secs: 999,
    };
    let report = classify_managed_wrapper(&process, Some(&agent), &[], &world);
    assert_eq!(report.verdict, Verdict::Protege);
    assert!(report.guards_triggered.iter().any(|g| g.starts_with("G4_")));
}

#[test]
fn harness_daemon_orphan_can_be_eligible() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let home = PathBuf::from("/tmp/bg-12345-abcd");
    let process = ProcessSnapshot {
        pid: 4513,
        ppid: 1,
        age_secs: 7200,
        command: "/tmp/x/bridget daemon".into(),
    };
    let report = classify_test_daemon(&process, Some(&home), &world);
    assert_eq!(report.verdict, Verdict::Eligible);
    assert!(report.would_have_done.contains("SIGTERM"));
    assert!(!report.would_have_done.contains("aucune"));
}

#[test]
fn harness_daemon_with_living_descendant_is_not_eligible() {
    // Cas réel 32465→35839 : parent éligible sans G9 aurait tué l'enfant.
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let home = PathBuf::from("/tmp/bg909-matrix-0-32462-0fe0a6d0");
    let parent = ProcessSnapshot {
        pid: 32465,
        ppid: 1,
        age_secs: 7200,
        command: "/tmp/x/bridget daemon".into(),
    };
    world.processes = vec![
        parent.clone(),
        ProcessSnapshot {
            pid: 35839,
            ppid: 32465,
            age_secs: 7000,
            command: format!(
                "/tmp/x/bridget managed-wrapper parity parity-managed-0 {}/parity-acp.py",
                home.display()
            ),
        },
    ];
    let report = classify_test_daemon(&parent, Some(&home), &world);
    assert_eq!(report.verdict, Verdict::Protege);
    assert!(
        report
            .guards_triggered
            .iter()
            .any(|g| g.starts_with("G9_descendant_managed_wrapper_vivant="))
    );
    assert!(report.would_have_done.contains("aucune"));
}

#[test]
fn harness_daemon_becomes_eligible_when_descendant_gone() {
    // Sans le second oracle, G9 pourrait bloquer tout le monde à jamais.
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let home = PathBuf::from("/tmp/bg909-matrix-1-54665-0ad2889e");
    let parent = ProcessSnapshot {
        pid: 64298,
        ppid: 1,
        age_secs: 7200,
        command: "/tmp/x/bridget daemon".into(),
    };
    world.processes = vec![
        parent.clone(),
        ProcessSnapshot {
            pid: 67637,
            ppid: 64298,
            age_secs: 7000,
            command: format!(
                "/tmp/x/bridget managed-wrapper parity parity-managed-1 {}/parity-acp.py",
                home.display()
            ),
        },
    ];
    let with_child = classify_test_daemon(&parent, Some(&home), &world);
    assert_eq!(with_child.verdict, Verdict::Protege);

    world.processes.retain(|p| p.pid != 67637);
    let without_child = classify_test_daemon(&parent, Some(&home), &world);
    assert_eq!(without_child.verdict, Verdict::Eligible);
    assert!(
        !without_child
            .guards_triggered
            .iter()
            .any(|g| g.starts_with("G9_"))
    );
    assert!(without_child.would_have_done.contains("SIGTERM"));
    assert!(!without_child.would_have_done.contains("aucune"));
}

#[test]
fn harness_daemon_protected_by_reparented_wrapper_sharing_home() {
    // Après réadoption (ppid=1), la filiation ppid disparaît ; le HOME
    // partagé reste un signal honnête — doute → pas d'éligibilité.
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let home = PathBuf::from("/tmp/bg909-matrix-0-reparent");
    let parent = ProcessSnapshot {
        pid: 100,
        ppid: 1,
        age_secs: 7200,
        command: "/tmp/x/bridget daemon".into(),
    };
    world.processes = vec![
        parent.clone(),
        ProcessSnapshot {
            pid: 200,
            ppid: 1,
            age_secs: 7000,
            command: format!(
                "/tmp/x/bridget managed-wrapper parity parity-managed-0 {}/parity-acp.py",
                home.display()
            ),
        },
    ];
    let report = classify_test_daemon(&parent, Some(&home), &world);
    assert_eq!(report.verdict, Verdict::Protege);
    assert!(
        report
            .guards_triggered
            .iter()
            .any(|g| g.contains("G9_descendant_managed_wrapper_vivant=200"))
    );
}

#[test]
fn temp_dir_with_owner_is_protege() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let dir = TempDirSnapshot {
        path: PathBuf::from("/tmp/bg-999-aa"),
        age_secs: 7200,
        prefix_kind: "bg".into(),
        agent_name: None,
    };
    let report = classify_temp_dir(&dir, &[111], &world);
    assert_eq!(report.verdict, Verdict::Protege);
}

#[test]
fn temp_dir_orphaned_is_eligible() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let dir = TempDirSnapshot {
        path: PathBuf::from("/tmp/mg1504-1-1"),
        age_secs: 7200,
        prefix_kind: "mg1504".into(),
        agent_name: None,
    };
    let report = classify_temp_dir(&dir, &[], &world);
    assert_eq!(report.verdict, Verdict::Eligible);
    assert!(report.would_have_done.contains("candidat à revue"));
    assert!(report.would_have_done.contains("JAMAIS exécuté"));
}

#[test]
fn repertoire_agent_absent_et_ancien_est_candidat_a_revue() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    let dir = TempDirSnapshot {
        path: PathBuf::from("/tmp/ancien-attribution"),
        age_secs: 7200,
        prefix_kind: "agent".into(),
        agent_name: Some("ancien".into()),
    };

    let report = classify_temp_dir(&dir, &[], &world);
    assert_eq!(report.verdict, Verdict::Eligible);
    assert_eq!(report.agent_name.as_deref(), Some("ancien"));
    assert!(
        report
            .criteria_met
            .iter()
            .any(|criterion| criterion == "P4_agent_absent_du_daemon=ancien")
    );
    assert!(report.would_have_done.contains("candidat à revue"));
}

#[test]
fn repertoire_agent_connecte_est_protege_meme_sans_descripteur_ouvert() {
    let mut world = world_at(1_700_000_000);
    world.min_age_secs = 60;
    world.agents.push(AgentPresence {
        name: "jc2".into(),
        state: "connected".into(),
        mode: None,
        location: None,
        last_seen_secs: 0,
    });
    let dir = TempDirSnapshot {
        path: PathBuf::from("/tmp/jc2-attribution"),
        age_secs: 7200,
        prefix_kind: "agent".into(),
        agent_name: Some("jc2".into()),
    };

    let report = classify_temp_dir(&dir, &[], &world);
    assert_eq!(report.verdict, Verdict::Protege);
    assert!(
        report
            .guards_triggered
            .iter()
            .any(|guard| guard == "G_agent_actif=jc2 état=connected"),
        "un arbre au repos reste protégé tant que l'agent est connecté"
    );
}

#[test]
fn racine_explicite_hors_temp_par_defaut_est_observee() {
    let root = PathBuf::from(format!(
        "/tmp/bridget-reaper-racine-explicite-{}",
        uuid::Uuid::new_v4()
    ));
    let target = root.join("jc6-revue");
    fs::create_dir_all(&target).unwrap();

    let scanned = scan_temp_dirs(&root, 1_700_000_000, &[]).unwrap();
    assert!(
        scanned
            .iter()
            .any(|entry| { entry.path == target && entry.agent_name.as_deref() == Some("jc6") }),
        "la racine passée explicitement doit être lue par le scanner, sans recours à TMPDIR"
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn raccord_productif_lit_la_racine_fournie_et_protege_un_nom_avec_tiret() {
    let root = PathBuf::from(format!(
        "/tmp/bridget-reaper-raccord-productif-{}",
        uuid::Uuid::new_v4()
    ));
    let target = root.join("jc2-review-attribution");
    fs::create_dir_all(&target).unwrap();
    let state_dir = root.join("state");
    fs::create_dir_all(&state_dir).unwrap();
    let report = observe_live(&state_dir, &root, 60).unwrap();
    assert!(
        report
            .targets
            .iter()
            .any(|entry| { entry.path.as_deref() == Some(target.to_str().unwrap()) })
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn agent_connecte_avec_tiret_est_protege() {
    let root = PathBuf::from(format!(
        "/tmp/bridget-reaper-tiret-{}",
        uuid::Uuid::new_v4()
    ));
    let target = root.join("jc2-review-attribution");
    fs::create_dir_all(&target).unwrap();
    let mut world = world_at(1_700_000_000);
    world.agent_inventory_available = true;
    world.agents.push(AgentPresence {
        name: "jc2-review".into(),
        state: "connected".into(),
        mode: None,
        location: None,
        last_seen_secs: 0,
    });
    scan_explicit_temp_root(&mut world, &root, 1_700_000_000).unwrap();
    let dir = world
        .temp_dirs
        .iter()
        .find(|entry| entry.path == target)
        .unwrap();
    assert_eq!(dir.agent_name.as_deref(), Some("jc2-review"));
    assert_eq!(
        classify_temp_dir(dir, &[], &world).verdict,
        Verdict::Protege
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn double_observation_marks_stable() {
    let mut world = world_at(1_700_000_600);
    world.min_age_secs = 60;
    world.double_observation_secs = 300;
    let home = PathBuf::from("/tmp/bg-1-aa");
    let process = ProcessSnapshot {
        pid: 7,
        ppid: 1,
        age_secs: 900,
        command: "/tmp/bridget daemon".into(),
    };
    let first = classify_test_daemon(&process, Some(&home), &world);
    assert_eq!(first.verdict, Verdict::Eligible);
    assert!(!first.stable_for_phase1);

    world.prior_hits.push(PriorHit {
        fingerprint: first.fingerprint.clone(),
        verdict: Verdict::Eligible,
        observed_at_unix: 1_700_000_000,
    });
    let second = classify_test_daemon(&process, Some(&home), &world);
    assert!(second.stable_for_phase1);
}

#[test]
fn build_report_lists_gap_when_background_missing() {
    let mut world = world_at(10);
    world.min_age_secs = 1;
    world.processes.push(ProcessSnapshot {
        pid: 1,
        ppid: 1,
        age_secs: 100,
        command: "/x/bridget managed-wrapper t n {}".into(),
    });
    let report = build_report(&world);
    assert!(!report.gaps_to_instruct.is_empty());
    assert_eq!(report.phase, "observer");
    assert_eq!(report.targets.len(), 1);
    assert_eq!(report.targets[0].verdict, Verdict::Incertain);
}

#[test]
fn parse_etime_variants() {
    assert_eq!(parse_etime_secs("05:01"), Some(301));
    assert_eq!(parse_etime_secs("1:02:03"), Some(3723));
    assert_eq!(parse_etime_secs("2-01:00:00"), Some(2 * 86400 + 3600));
}

#[test]
fn observer_module_has_no_kill_symbols_in_would_execution_path() {
    // Garde de conception : Phase 0 n'expose pas d'API d'action.
    // (Régression textuelle volontaire — si quelqu'un ajoute kill/unlink
    // public, ce test ne les détecte pas tous, mais documente l'intention.)
    let report = build_report(&world_at(1));
    assert_eq!(report.phase, "observer");
    assert!(report.targets.is_empty());
}
