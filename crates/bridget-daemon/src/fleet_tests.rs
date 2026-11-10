    use super::*;
    use std::fs;
    use std::io::Read;
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command, Stdio};
    use std::sync::{Arc, Barrier};
    use std::thread;

    // Identités v4 stables : le même agent reste identique après rejeu/redémarrage.
    const AGENT_DOMAIN: &str = "00000001-0000-4000-8000-000000000001";
    const AGENT_SCOPE: &str = "00000002-0000-4000-8000-000000000001";
    const AGENT_ADOPT: &str = "00000003-0000-4000-8000-000000000001";
    const AGENT_RELAUNCH: &str = "00000004-0000-4000-8000-000000000001";
    const AGENT_ALPHA: &str = "00000005-0000-4000-8000-000000000001";
    const AGENT_ZETA: &str = "00000006-0000-4000-8000-000000000001";
    const AGENT_UNIQUE: &str = "00000007-0000-4000-8000-000000000001";
    const AGENT_WAIT: &str = "00000008-0000-4000-8000-000000000001";
    const AGENT_QUOTA_A: &str = "00000009-0000-4000-8000-000000000001";
    const AGENT_QUOTA_B: &str = "00000010-0000-4000-8000-000000000001";
    const AGENT_GUARD: &str = "00000011-0000-4000-8000-000000000001";
    const AGENT_EXPIRED: &str = "00000012-0000-4000-8000-000000000001";
    const AGENT_STOP: &str = "00000013-0000-4000-8000-000000000001";
    const AGENT_CRASH: &str = "00000014-0000-4000-8000-000000000001";

    const NOW: i64 = 1_000_000;
    const CRASH_CHILD_ENV: &str = "BRIDGET_T904_CRASH_CHILD";
    const CRASH_ROOT_ENV: &str = "BRIDGET_T904_CRASH_ROOT";
    const CRASH_STAGE_ENV: &str = "BRIDGET_T904_CRASH_STAGE";
    const CRASH_BARRIER_FD: RawFd = 112;

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t904-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn config() -> FleetConfig {
        FleetConfig {
            quota: 2,
            persistent_horizon_secs: 3600,
            ephemeral_horizon_secs: 300,
            issued_at_tolerance_secs: 30,
        }
    }

    fn open(root: &Path) -> FleetSupervisor {
        fs::create_dir_all(root).unwrap();
        FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            config(),
        )
        .unwrap()
    }

    fn order(command_id: &str, name: Option<&str>, persistent: bool) -> SpawnOrder {
        SpawnOrder {
            remote_placement: None,
            posture: None,
            agent_type: "codex".to_string(),
            requested_name: name.map(str::to_string),
            cwd: PathBuf::from("/tmp"),
            persistent,
            command_id: command_id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 60,
            project: None,
            ownership: None,
        }
    }

    fn start(supervisor: &FleetSupervisor, order: &SpawnOrder) -> SpawnLease {
        match supervisor.request_spawn(order, NOW).unwrap() {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("nouvelle commande non démarrée: {other:?}"),
        }
    }

    fn remote_order162() -> SpawnOrder {
        let mut order = order("remote162", Some(AGENT_ALPHA), true);
        order.remote_placement = Some(
            serde_json::from_value(serde_json::json!({
                "machine_id":"cartae", "worker_id":"worker", "provisioning_revision":1,
                "project_id":"project", "mapping_revision":1, "origin_host":"mac",
                "origin_cwd":"/Users/moi/project", "source_root_identity":"git:project",
                "relative_cwd":".", "execution_cwd":"/tmp", "base_commit":null,
                "definition_digest":"a".repeat(64), "permission_digest":"b".repeat(64)
            }))
            .unwrap(),
        );
        order
    }

    #[test]
    fn remote162_placement_fait_partie_du_canon_et_de_la_reservation() {
        let root = test_root("remote162-reservation");
        let supervisor = open(&root);
        let mut order = remote_order162();
        let lease = start(&supervisor, &order);
        assert_eq!(lease.remote_placement, order.remote_placement);
        let scope = supervisor.supervisor_scope();
        let store = IdempotencyStore::open(&root.join("bridget.db")).unwrap();
        assert_eq!(
            store.spawn_commands(&scope).unwrap()[0].remote_placement,
            order.remote_placement
        );
        assert!(matches!(
            supervisor.request_spawn(&order, NOW).unwrap(),
            SpawnSubmission::Await(_)
        ));
        order.remote_placement.as_mut().unwrap().mapping_revision = 2;
        assert!(matches!(
            supervisor.request_spawn(&order, NOW).unwrap(),
            SpawnSubmission::EnvelopeMismatch
        ));
        let local = self::order("local162", Some(AGENT_ZETA), true);
        let local_canon: serde_json::Value =
            serde_json::from_slice(&canonical_order(&local).unwrap()).unwrap();
        assert!(local_canon.get("remote_placement").is_none());
        drop(store);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remote162_placement_persiste_fleet5_et_lease_divergente_refusee() {
        let root = test_root("remote162-fleet");
        let supervisor = open(&root);
        let mut order = remote_order162();
        order.persistent = false;
        let lease = start(&supervisor, &order);
        let mut forged = lease.clone();
        forged.remote_placement = None;
        assert!(matches!(
            supervisor.mark_starting(&forged, NOW, &resolved_test_definition()),
            Err(FleetError::StaleGeneration)
        ));
        assert!(
            supervisor
                .mark_starting(&lease, NOW, &resolved_test_definition())
                .is_err()
        );
        let mut definition = resolved_test_definition();
        definition.digest = "a".repeat(64);
        supervisor.mark_starting(&lease, NOW, &definition).unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        let desired = supervisor.desired_fleet().unwrap();
        assert_eq!(desired.schema, 5);
        assert_eq!(
            desired.equipiers[AGENT_ALPHA].remote_placement,
            order.remote_placement
        );
        assert_eq!(supervisor.active_count(), 1);
        drop(supervisor);
        // La perte de la projection Desired ne prouve pas la fin distante.
        fs::remove_file(root.join("fleet.json")).unwrap();
        let supervisor = open(&root);
        assert_eq!(supervisor.active_count(), 1);
        assert_eq!(supervisor.recovery_candidates()[0].lease, lease);
        // Une preuve terminale explicite libère le quota sans changer l'issue Connected.
        supervisor.invalidate_for_stop(&lease).unwrap();
        assert_eq!(supervisor.active_count(), 0);
        assert!(matches!(
            supervisor.request_spawn(&order, NOW + 2).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Connected { .. })
        ));
        drop(supervisor);
        let supervisor = open(&root);
        assert_eq!(supervisor.active_count(), 0);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remote162_reprise_sans_desired_conserve_nom_quota_et_identite() {
        for (persistent, starting) in [(false, false), (true, true)] {
            let root = test_root("remote162-unknown");
            let supervisor = open(&root);
            let mut order = remote_order162();
            order.persistent = persistent;
            order.ownership = Some(SpawnOwnership {
                parent_instance_id: "parent162".into(),
                parent_execution_id: None,
                objective_id: None,
                delegation_id: None,
                project: None,
                role: "child".into(),
                max_children: Some(4),
                max_depth: Some(4),
            });
            let lease = start(&supervisor, &order);
            if starting {
                let mut definition = resolved_test_definition();
                definition.digest = "a".repeat(64);
                supervisor.mark_starting(&lease, NOW, &definition).unwrap();
            }
            assert!(supervisor.desired_entry(AGENT_ALPHA).unwrap().is_none());
            drop(supervisor);
            let supervisor = open(&root);
            assert_eq!(supervisor.active_count(), 1);
            assert_eq!(supervisor.recovery_candidates()[0].lease, lease);
            assert!(supervisor.desired_entry(AGENT_ALPHA).unwrap().is_none());
            assert!(matches!(
                supervisor.request_spawn(&order, NOW).unwrap(),
                SpawnSubmission::Await(_)
            ));
            assert!(
                !supervisor
                    .expire(&lease.command_id, lease.generation, NOW + 61)
                    .unwrap()
            );
            assert_eq!(supervisor.active_count(), 1);
            let same_name = self::order("other162", Some(AGENT_ALPHA), false);
            assert!(matches!(supervisor.request_spawn(&same_name, NOW).unwrap(),
                SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. }) if category == "name_active"));
            let local = self::order("local162", Some(AGENT_ZETA), false);
            start(&supervisor, &local);
            let over_quota = self::order("over162", Some(AGENT_DOMAIN), false);
            assert!(
                matches!(supervisor.request_spawn(&over_quota, NOW).unwrap(),
                SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. }) if category == "quota_exceeded")
            );
            supervisor.invalidate_for_stop(&lease).unwrap();
            assert_eq!(supervisor.active_count(), 1);
            drop(supervisor);
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn domain_est_persiste_dans_fleet_json_apres_connexion() {
        let root = test_root("domain-connect");
        let supervisor = open(&root);
        let mut spawn = order("command-domain", Some(AGENT_DOMAIN), true);
        spawn.cwd = PathBuf::from("/tmp/bridget");
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        let fleet = supervisor.desired_fleet().unwrap();
        assert_eq!(
            fleet.equipiers[AGENT_DOMAIN].domain.as_deref(),
            Some("bridget")
        );
        let raw = fs::read_to_string(root.join("fleet.json")).unwrap();
        assert!(raw.contains("\"domain\": \"bridget\""), "{raw}");
        supervisor
            .set_desired_domain(AGENT_DOMAIN, Some("nouveau-projet"))
            .unwrap();
        assert_eq!(
            supervisor.desired_fleet().unwrap().equipiers[AGENT_DOMAIN]
                .domain
                .as_deref(),
            Some("nouveau-projet")
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn resolved_test_definition() -> ResolvedAgentDefinition {
        ResolvedAgentDefinition {
            command: "npx".to_string(),
            args: vec!["fixture-acp".to_string()],
            protocol: "acp".to_string(),
            forbidden_env: vec!["API_KEY".to_string()],
            pass_env: Vec::new(),
            claude_config_dir: None,
            codex_home: None,
            permissions: "allow".to_string(),
            queue_capacity: 32,
            notify_timeout_secs: 600,
            mcp: bridget_transport::ResolvedMcpDefinition {
                interactive: "none".to_string(),
                acp_session: false,
            },
            capabilities: bridget_transport::AdapterCapabilities::default(),
            digest: "fixture-digest".to_string(),
        }
    }

    #[test]
    fn scope_superviseur_et_issue_persistante_survivent_au_redemarrage() {
        let root = test_root("scope");
        let supervisor = open(&root);
        let scope = supervisor.supervisor_scope();
        let spawn = order("command-scope", Some(AGENT_SCOPE), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let issue = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        assert!(matches!(
            &issue,
            SpawnCommandIssue::Connected { definition: Some(definition), .. }
                if definition.as_ref() == &resolved_test_definition()
        ));
        drop(supervisor);

        let reopened = open(&root);
        assert_eq!(reopened.supervisor_scope(), scope);
        assert_eq!(
            reopened.request_spawn(&spawn, NOW + 2).unwrap(),
            SpawnSubmission::Terminal(issue)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adoption_exige_une_generation_connectee_et_reconstruit_un_stopped_complet() {
        let root = test_root("adopt-stopped");
        let supervisor = open(&root);
        let spawn = order("command-adopt", Some(AGENT_ADOPT), false);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        supervisor.remove_desired(AGENT_ADOPT).unwrap();

        assert_eq!(
            supervisor.adopt_stopped("inconnu", NOW + 2).unwrap(),
            AdoptStoppedResult::NoManagedHistory
        );
        assert_eq!(
            supervisor.adopt_stopped(AGENT_ADOPT, NOW + 2).unwrap(),
            AdoptStoppedResult::Adopted {
                generation: lease.generation
            }
        );
        let adopted = supervisor
            .desired_entry(AGENT_ADOPT)
            .unwrap()
            .expect("entrée adoptée");
        assert_eq!(adopted.lifecycle_state, DesiredLifecycleState::Stopped);
        assert!(!adopted.persistent);
        assert_eq!(adopted.cwd, PathBuf::from("/tmp"));
        assert_eq!(
            adopted.resolved_definition,
            Some(resolved_test_definition())
        );
        assert_eq!(
            supervisor.adopt_stopped(AGENT_ADOPT, NOW + 3).unwrap(),
            AdoptStoppedResult::AlreadyManaged
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relance_change_de_generation_sans_perdre_le_stopped_et_decommission_reserve_le_nom() {
        let root = test_root("relaunch-decommission");
        let supervisor = open(&root);
        let initial = order("command-initial", Some(AGENT_RELAUNCH), true);
        let initial_lease = start(&supervisor, &initial);
        supervisor
            .mark_starting(&initial_lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&initial_lease, &initial_lease.instance_id, NOW + 1)
            .unwrap();
        assert!(supervisor.mark_stopped(AGENT_RELAUNCH).unwrap());
        drop(supervisor);
        let supervisor = open(&root);
        assert_eq!(
            supervisor
                .desired_entry(AGENT_RELAUNCH)
                .unwrap()
                .unwrap()
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );

        let failed_order = order("command-relaunch-failed", Some(AGENT_RELAUNCH), true);
        let failed_lease = match supervisor.request_relaunch(&failed_order, NOW + 2).unwrap() {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("relance attendue: {other:?}"),
        };
        supervisor
            .mark_starting(&failed_lease, NOW + 2, &resolved_test_definition())
            .unwrap();
        supervisor
            .fail(&failed_lease, "startup_failed", "fixture")
            .unwrap();
        let retained = supervisor
            .desired_entry(AGENT_RELAUNCH)
            .unwrap()
            .expect("stopped conservé après échec");
        assert_eq!(retained.lifecycle_state, DesiredLifecycleState::Stopped);
        assert_eq!(retained.command_id, initial.command_id);

        let success_order = order("command-relaunch-ok", Some(AGENT_RELAUNCH), true);
        let success_lease = match supervisor
            .request_relaunch(&success_order, NOW + 3)
            .unwrap()
        {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("nouvelle relance attendue: {other:?}"),
        };
        assert!(success_lease.generation > initial_lease.generation);
        supervisor
            .mark_starting(&success_lease, NOW + 3, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&success_lease, &success_lease.instance_id, NOW + 4)
            .unwrap();
        let running = supervisor
            .desired_entry(AGENT_RELAUNCH)
            .unwrap()
            .expect("relance connectée");
        assert_eq!(running.lifecycle_state, DesiredLifecycleState::Running);
        assert_eq!(running.command_id, success_order.command_id);

        assert!(supervisor.mark_stopped(AGENT_RELAUNCH).unwrap());
        assert!(supervisor.decommission(AGENT_RELAUNCH).unwrap());
        drop(supervisor);
        let supervisor = open(&root);
        assert_eq!(
            supervisor
                .desired_entry(AGENT_RELAUNCH)
                .unwrap()
                .unwrap()
                .lifecycle_state,
            DesiredLifecycleState::Decommissioned
        );
        let historical = supervisor
            .inner
            .lock()
            .unwrap()
            .idempotency
            .latest_connected_spawn_by_name(AGENT_RELAUNCH)
            .unwrap()
            .expect("historique de la génération relancée conservé");
        assert_eq!(historical.generation, success_lease.generation);
        let reserved = order("command-reuse", Some(AGENT_RELAUNCH), true);
        assert!(matches!(
            supervisor.request_spawn(&reserved, NOW + 5).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. })
                if category == "name_reserved"
        ));
        assert!(!supervisor.named_persistence().contains_key(AGENT_RELAUNCH));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_expose_les_generations_en_vol_dans_l_ordre_des_noms() {
        let root = test_root("recovery-candidates");
        let supervisor = open(&root);
        for (command_id, name) in [("command-z", AGENT_ZETA), ("command-a", AGENT_ALPHA)] {
            let project = (name == AGENT_ALPHA).then(|| ProjectReference {
                project_id: "project-alpha".to_string(),
                binding_generation: 2,
            });
            let mut spawn = order(command_id, Some(name), true);
            spawn.project = project.clone();
            let lease = start(&supervisor, &spawn);
            assert_eq!(lease.project, project);
            supervisor
                .mark_starting(&lease, NOW, &resolved_test_definition())
                .unwrap();
            supervisor
                .desired
                .upsert(
                    name.to_string(),
                    DesiredEquipier {
                        remote_placement: None,
                        agent_type: "codex".to_string(),
                        cwd: PathBuf::from("/tmp"),
                        command_id: command_id.to_string(),
                        generation: lease.generation,
                        created: NOW.to_string(),
                        persistent: true,
                        lifecycle_state: DesiredLifecycleState::Running,
                        resolved_definition: Some(resolved_test_definition()),
                        domain: None,
                        project: project.clone(),
                        agent_link: None,
                        runtime_execution: None,
                    },
                )
                .unwrap();
        }
        drop(supervisor);

        let reopened = open(&root);
        let candidates = reopened.recovery_candidates();
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.lease.name.as_str())
                .collect::<Vec<_>>(),
            [AGENT_ALPHA, AGENT_ZETA]
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.lease.persistent)
        );
        assert!(candidates.iter().any(|candidate| {
            candidate.lease.name == AGENT_ALPHA
                && candidate.lease.project.as_ref().is_some_and(|project| {
                    project.project_id == "project-alpha" && project.binding_generation == 2
                })
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deux_spawns_simultanes_du_meme_nom_reservent_un_seul_slot() {
        let root = test_root("same-name");
        let supervisor = Arc::new(open(&root));
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for command_id in ["command-a", "command-b"] {
            let supervisor = Arc::clone(&supervisor);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                let order = order(command_id, Some(AGENT_UNIQUE), false);
                barrier.wait();
                supervisor.request_spawn(&order, NOW).unwrap()
            }));
        }
        barrier.wait();
        let outcomes: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, SpawnSubmission::Start(_)))
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. })
                        if category == "name_active"
                ))
                .count(),
            1
        );
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retry_en_vol_attend_le_vrai_register_sans_connected_synthetique() {
        let root = test_root("wait");
        let supervisor = Arc::new(open(&root));
        let spawn = order("command-wait", Some(AGENT_WAIT), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let waiter = match supervisor.request_spawn(&spawn, NOW + 1).unwrap() {
            SpawnSubmission::Await(waiter) => waiter,
            other => panic!("retry non rattaché: {other:?}"),
        };
        let waiting = Arc::clone(&supervisor);
        let wait_handle = thread::spawn(move || {
            waiting
                .wait_for_terminal(&waiter, Duration::from_secs(2))
                .unwrap()
        });
        let issue = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 2)
            .unwrap();
        assert_eq!(wait_handle.join().unwrap(), Some(issue));
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn quota_est_reserve_atomiquement_et_le_refus_est_rejouable() {
        let root = test_root("quota");
        fs::create_dir_all(&root).unwrap();
        let supervisor = FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota: 1,
                ..config()
            },
        )
        .unwrap();
        let first = order("command-quota-a", Some(AGENT_QUOTA_A), false);
        let second = order("command-quota-b", Some(AGENT_QUOTA_B), false);
        assert!(matches!(
            supervisor.request_spawn(&first, NOW).unwrap(),
            SpawnSubmission::Start(_)
        ));
        let refusal = supervisor.request_spawn(&second, NOW).unwrap();
        match &refusal {
            SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, reason }) => {
                assert_eq!(category, "quota_exceeded");
                assert_eq!(reason, &quota_exceeded_detail(1));
                assert!(reason.contains("BRIDGET_FLEET_QUOTA"));
                assert!(reason.contains("quota de flotte atteint (1)"));
            }
            other => panic!("refus quota attendu, obtenu {other:?}"),
        }
        assert_eq!(supervisor.request_spawn(&second, NOW + 1).unwrap(), refusal);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolve_fleet_quota_lit_env_invalide_et_zero() {
        assert_eq!(resolve_fleet_quota(None), DEFAULT_FLEET_QUOTA);
        assert_eq!(resolve_fleet_quota(Some(OsString::from("24"))), 24);
        assert_eq!(resolve_fleet_quota(Some(OsString::from("0"))), 0);
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from("abc"))),
            DEFAULT_FLEET_QUOTA
        );
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from(""))),
            DEFAULT_FLEET_QUOTA
        );
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from("-3"))),
            DEFAULT_FLEET_QUOTA
        );
        let root = test_root("quota-zero");
        fs::create_dir_all(&root).unwrap();
        let err = FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota: 0,
                ..config()
            },
        );
        assert!(matches!(
            err,
            Err(FleetError::InvalidOrder("configuration de flotte invalide"))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn from_env_posee_releve_le_quota_consomme_par_le_daemon() {
        let _guard = TestFleetQuotaGuard::set(24);
        assert_eq!(FleetConfig::from_env().quota, 24);
    }

    #[test]
    fn message_refus_reprise_quota_nomme_agent_quota_et_levier() {
        let message = resume_quota_refusal_message("cursor8", 8);
        assert!(message.contains("cursor8"), "{message}");
        assert!(message.contains("quota de flotte atteint (8)"), "{message}");
        assert!(message.contains("BRIDGET_FLEET_QUOTA"), "{message}");
        assert!(message.contains("export BRIDGET_FLEET_QUOTA="), "{message}");
    }

    #[test]
    fn mismatch_expiration_et_register_tardif_sont_terminaux() {
        let root = test_root("guards");
        let supervisor = open(&root);
        let spawn = order("command-guard", Some(AGENT_GUARD), false);
        let lease = start(&supervisor, &spawn);
        let mut divergent = spawn.clone();
        divergent.cwd = PathBuf::from("/var/tmp");
        assert_eq!(
            supervisor.request_spawn(&divergent, NOW).unwrap(),
            SpawnSubmission::EnvelopeMismatch
        );
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        assert!(matches!(
            supervisor.register_connected(&lease, "instance-obsolete", NOW + 1),
            Err(FleetError::StaleGeneration)
        ));
        assert!(
            supervisor
                .expire(&lease.command_id, lease.generation, spawn.deadline_at)
                .unwrap()
        );
        assert!(matches!(
            supervisor.register_connected(&lease, &lease.instance_id, NOW + 61),
            Err(FleetError::StaleGeneration)
        ));
        assert!(matches!(
            supervisor.request_spawn(&spawn, NOW + 61).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Cancelled { .. })
        ));

        let expired = SpawnOrder {
            command_id: "command-expired".to_string(),
            issued_at: NOW - 400,
            deadline_at: NOW + 1,
            ..order("unused", Some(AGENT_EXPIRED), false)
        };
        assert_eq!(
            supervisor.request_spawn(&expired, NOW).unwrap(),
            SpawnSubmission::IdempotencyExpired
        );
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_invalide_un_lancement_avant_toute_io_et_rejoue_cancelled() {
        let root = test_root("stop-starting");
        let supervisor = open(&root);
        let spawn = order("command-stop-starting", Some(AGENT_STOP), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();

        supervisor.invalidate_for_stop(&lease).unwrap();

        assert!(matches!(
            supervisor.request_spawn(&spawn, NOW + 1).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Cancelled { ref reason })
                if reason == "arrêt demandé"
        ));
        assert_eq!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers[AGENT_STOP]
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_connecte_preserve_l_issue_spawn_et_marque_l_agent_arrete() {
        let root = test_root("stop-connected");
        let supervisor = open(&root);
        let spawn = order("command-stop-connected", Some(AGENT_STOP), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let connected = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();

        supervisor.invalidate_for_stop(&lease).unwrap();

        assert_eq!(
            supervisor.request_spawn(&spawn, NOW + 2).unwrap(),
            SpawnSubmission::Terminal(connected)
        );
        assert_eq!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers[AGENT_STOP]
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        fs::remove_dir_all(root).unwrap();
    }

    struct CrashChild(Child);

    impl CrashChild {
        fn terminate(&mut self) {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            let pid = self.0.id();
            let executable = std::env::current_exe().unwrap();
            let observed = Command::new("/bin/ps")
                .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
                .output()
                .unwrap();
            let command = String::from_utf8_lossy(&observed.stdout);
            let owned = command
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<u32>().ok())
                == Some(std::process::id())
                && command.contains(executable.to_string_lossy().as_ref())
                && !command.to_ascii_lowercase().contains("firefox")
                // SAFETY: getpgid ne déréférence aucun pointeur ; ses arguments sont des entiers.
                && unsafe { libc::getpgid(pid as i32) } == pid as i32;
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            assert!(
                owned,
                "le crash doit cibler notre enfant de test et son groupe dédié"
            );
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            assert_eq!(unsafe { libc::kill(pid as i32, libc::SIGKILL) }, 0);
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            loop {
                if let Some(status) = self.0.try_wait().unwrap() {
                    assert!(!status.success());
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "enfant de crash non récolté"
                );
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    impl Drop for CrashChild {
        fn drop(&mut self) {
            // Une barrière EOF/timeout ne doit pas abandonner un enfant suspendu.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.terminate()));
        }
    }

    fn spawn_crash_child(root: &Path, stage: &str) -> (CrashChild, UnixStream) {
        let executable = std::env::current_exe().unwrap();
        assert!(
            executable
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("bridget_daemon"),
            "le sous-processus de crash doit être le binaire de test daemon, jamais Firefox"
        );
        let (parent_barrier, child_barrier) = UnixStream::pair().unwrap();
        parent_barrier
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut command = Command::new(executable);
        for directory in ["home", "state", "tmp"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        command
            .env_clear()
            .env("HOME", root.join("home"))
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            .env("TMPDIR", root.join("tmp"))
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .arg("fleet::tests::crash_child")
            .arg("--exact")
            .arg("--ignored")
            .arg("--nocapture")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env(CRASH_CHILD_ENV, "1")
            .env(CRASH_ROOT_ENV, root)
            .env(CRASH_STAGE_ENV, stage);
        // SAFETY: setpgid ne déréférence aucun pointeur ; ses arguments sont des entiers; fcntl sans argument ne déréférence aucun pointeur; dup2 ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                let source = child_barrier.as_raw_fd();
                if source == CRASH_BARRIER_FD {
                    // dup2 sur lui-même ne retire pas FD_CLOEXEC : le faire ici.
                    let flags = libc::fcntl(source, libc::F_GETFD);
                    if flags < 0 || libc::fcntl(source, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                        return Err(io::Error::last_os_error());
                    }
                } else if libc::dup2(source, CRASH_BARRIER_FD) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        (CrashChild(command.spawn().unwrap()), parent_barrier)
    }

    fn wait_barrier(barrier: &mut UnixStream) {
        let mut byte = [0_u8; 1];
        barrier.read_exact(&mut byte).unwrap();
        assert_eq!(byte[0], b'B');
    }

    fn terminate_child(child: &mut CrashChild) {
        child.terminate();
    }

    fn signal_and_park() {
        let byte = b'B';
        assert_eq!(
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { libc::write(CRASH_BARRIER_FD, (&byte as *const u8).cast(), 1) },
            1
        );
        loop {
            thread::park();
        }
    }

    #[test]
    #[ignore = "sous-processus contrôlé par crash_reel_aux_frontieres_d503"]
    fn crash_child() {
        if std::env::var(CRASH_CHILD_ENV).as_deref() != Ok("1") {
            return;
        }
        let root = PathBuf::from(std::env::var_os(CRASH_ROOT_ENV).unwrap());
        let stage = std::env::var(CRASH_STAGE_ENV).unwrap();
        let supervisor = open(&root);
        let spawn = order("command-crash", Some(AGENT_CRASH), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        match stage.as_str() {
            "before_fleet" => signal_and_park(),
            "after_fleet" => {
                let _ = supervisor.register_connected_observed(
                    &lease,
                    &lease.instance_id,
                    NOW + 1,
                    || {
                        signal_and_park();
                        #[allow(unreachable_code)]
                        Ok(())
                    },
                );
                unreachable!();
            }
            "after_issue" => {
                supervisor
                    .register_connected(&lease, &lease.instance_id, NOW + 1)
                    .unwrap();
                signal_and_park();
            }
            _ => panic!("frontière de crash inconnue"),
        }
    }

    #[test]
    fn crash_reel_aux_frontieres_d503_rejoue_une_issue_unique() {
        for stage in ["before_fleet", "after_fleet", "after_issue"] {
            let root = test_root(stage);
            fs::create_dir_all(&root).unwrap();
            let (mut child, mut barrier) = spawn_crash_child(&root, stage);
            wait_barrier(&mut barrier);
            terminate_child(&mut child);

            let reopened = open(&root);
            let spawn = order("command-crash", Some(AGENT_CRASH), true);
            let outcome = reopened.request_spawn(&spawn, NOW + 2).unwrap();
            match stage {
                "before_fleet" => assert!(matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Failed { .. })
                )),
                "after_fleet" => {
                    let waiter = match outcome {
                        SpawnSubmission::Await(waiter) => waiter,
                        other => panic!("reprise non rattachée: {other:?}"),
                    };
                    let lease = SpawnLease {
                        remote_placement: None,
                        command_id: waiter.command_id,
                        name: AGENT_CRASH.to_string(),
                        instance_id: waiter.instance_id,
                        generation: waiter.generation,
                        deadline_at: waiter.deadline_at,
                        persistent: true,
                        project: None,
                        link_id: None,
                        ownership: None,
                        agent_path: None,
                    };
                    let connected = reopened
                        .register_connected(&lease, &lease.instance_id, NOW + 3)
                        .unwrap();
                    assert!(matches!(connected, SpawnCommandIssue::Connected { .. }));
                }
                "after_issue" => assert!(matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Connected { .. })
                )),
                _ => unreachable!(),
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
