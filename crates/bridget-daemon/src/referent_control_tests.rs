use super::*;

fn conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    ensure_schema(&conn).unwrap();
    conn
}

fn mutation<'a>(command_id: &'a str, generation: u64) -> ControlMutation<'a> {
    ControlMutation {
        command_id,
        expected_generation: generation,
        paused: None,
        auto_objectives_cap: None,
        reason: None,
        actor: "humain",
        now: 1_788_400_000,
        agent_posture: None,
        auto_reassignment: None,
    }
}

#[test]
fn spec_088_base_neuve_est_prudente_et_base_existante_garde_son_comportement() {
    // Neuve : découverte, réassignation différée.
    let fresh = read(&conn()).unwrap();
    assert_eq!(fresh.agent_posture, Some(AgentPosture::Discovery));
    assert_eq!(fresh.auto_reassignment, Some(false));
    // Existante : la table 087 sans les colonnes, avec sa ligne ⇒ complète, active.
    // Schéma EXACT de a931a8ac (SPEC-087) : la contrainte CHECK du journal
    // ne connaît que trois kinds, et une ligne d'historique existe.
    let legacy = Connection::open_in_memory().unwrap();
    legacy
            .execute_batch(
                "CREATE TABLE control_state (
                    id INTEGER PRIMARY KEY CHECK (id = 1), generation INTEGER NOT NULL,
                    paused INTEGER NOT NULL CHECK (paused IN (0, 1)), paused_since INTEGER, paused_by TEXT,
                    pause_reason TEXT, auto_objectives_cap INTEGER NOT NULL, updated_at INTEGER NOT NULL);
                 CREATE TABLE control_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER NOT NULL, actor TEXT NOT NULL,
                    kind TEXT NOT NULL CHECK (kind IN ('pause_on', 'pause_off', 'budget_set')),
                    reason TEXT, generation_after INTEGER NOT NULL, command_id TEXT NOT NULL UNIQUE);
                 INSERT INTO control_state VALUES (1, 1, 0, NULL, NULL, NULL, 5, 0);
                 INSERT INTO control_events (at, actor, kind, reason, generation_after, command_id)
                    VALUES (10, 'humain', 'budget_set', NULL, 1, 'c-hist');",
            )
            .unwrap();
    ensure_schema(&legacy).unwrap();
    let migrated = read(&legacy).unwrap();
    assert_eq!(migrated.agent_posture, Some(AgentPosture::Complete));
    assert_eq!(migrated.auto_reassignment, Some(true));
    assert_eq!(
        history(&legacy, 5).unwrap().len(),
        1,
        "l'historique est recopié"
    );
    // Un changement de droits est réellement accepté par le journal migré.
    let state = set(
        &legacy,
        ControlMutation {
            auto_reassignment: Some(false),
            ..mutation("r-legacy", 1)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(state.generation, 2);
    assert_eq!(history(&legacy, 1).unwrap()[0].kind, "rights_set");
    // Idempotent : une seconde passe ne change rien.
    ensure_schema(&legacy).unwrap();
    assert_eq!(read(&legacy).unwrap(), state);
}

#[test]
fn spec_088_droits_journalises_sous_la_meme_generation() {
    let conn = conn();
    let state = set(
        &conn,
        ControlMutation {
            agent_posture: Some(AgentPosture::Complete),
            ..mutation("r1", 0)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(state.generation, 1);
    assert_eq!(state.agent_posture, Some(AgentPosture::Complete));
    assert_eq!(
        state.auto_reassignment,
        Some(false),
        "la réassignation ne bouge pas seule"
    );
    let events = history(&conn, 5).unwrap();
    assert_eq!(events[0].kind, "rights_set");
    assert_eq!(events[0].generation_after, 1);
    // Rejeu du même command_id : état rendu, pas de génération.
    let replayed = set(
        &conn,
        ControlMutation {
            agent_posture: Some(AgentPosture::Complete),
            ..mutation("r1", 0)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(replayed.generation, 1);
    // Même valeur, autre commande ⇒ rien à changer.
    assert_eq!(
        set(
            &conn,
            ControlMutation {
                agent_posture: Some(AgentPosture::Complete),
                ..mutation("r2", 1)
            },
        )
        .unwrap()
        .unwrap_err(),
        ControlStateRefusal::NothingToChange
    );
    // Posture + plafond dans la même mutation : une seule génération, journal du plafond.
    let both = set(
        &conn,
        ControlMutation {
            auto_reassignment: Some(true),
            auto_objectives_cap: Some(20),
            ..mutation("r3", 1)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(both.generation, 2);
    assert_eq!(both.auto_reassignment, Some(true));
    assert_eq!(both.auto_objectives_cap, 20);
    assert_eq!(history(&conn, 1).unwrap()[0].generation_after, 2);
    // La pause n'est jamais touchée par une mutation de droits.
    assert!(!both.paused);
}

#[test]
fn etat_par_defaut_actif_avec_plafond_cinq() {
    let state = read(&conn()).unwrap();
    assert!(!state.paused);
    assert_eq!(state.generation, 0);
    assert_eq!(state.auto_objectives_cap, DEFAULT_AUTO_OBJECTIVES_CAP);
    assert_eq!(state.paused_since, None);
}

#[test]
fn pause_puis_reprise_journalisees_avec_generation() {
    let conn = conn();
    let state = set(
        &conn,
        ControlMutation {
            paused: Some(true),
            reason: Some("revue"),
            ..mutation("c1", 0)
        },
    )
    .unwrap()
    .unwrap();
    assert!(state.paused);
    assert_eq!(state.generation, 1);
    assert_eq!(state.paused_since, Some(1_788_400_000));
    assert_eq!(state.paused_by.as_deref(), Some("humain"));
    assert_eq!(state.pause_reason.as_deref(), Some("revue"));
    let state = set(
        &conn,
        ControlMutation {
            paused: Some(false),
            ..mutation("c2", 1)
        },
    )
    .unwrap()
    .unwrap();
    assert!(!state.paused);
    assert_eq!(state.generation, 2);
    assert_eq!(state.paused_since, None);
    let events = history(&conn, 10).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, "pause_off");
    assert_eq!(events[1].kind, "pause_on");
    assert_eq!(events[1].reason.as_deref(), Some("revue"));
}

#[test]
fn generation_inattendue_refusee_sans_ecriture() {
    let conn = conn();
    let refusal = set(
        &conn,
        ControlMutation {
            paused: Some(true),
            ..mutation("c1", 7)
        },
    )
    .unwrap()
    .unwrap_err();
    assert_eq!(
        refusal,
        ControlStateRefusal::GenerationMismatch { current: 0 }
    );
    assert!(!read(&conn).unwrap().paused);
    assert!(history(&conn, 10).unwrap().is_empty());
}

#[test]
fn plafond_hors_bornes_et_absence_de_changement_refuses() {
    let conn = conn();
    let refusal = set(
        &conn,
        ControlMutation {
            auto_objectives_cap: Some(0),
            ..mutation("c1", 0)
        },
    )
    .unwrap()
    .unwrap_err();
    assert!(matches!(
        refusal,
        ControlStateRefusal::BudgetOutOfRange { min: 1, max: 100 }
    ));
    let refusal = set(
        &conn,
        ControlMutation {
            paused: Some(false),
            auto_objectives_cap: Some(DEFAULT_AUTO_OBJECTIVES_CAP),
            ..mutation("c1", 0)
        },
    )
    .unwrap()
    .unwrap_err();
    assert_eq!(refusal, ControlStateRefusal::NothingToChange);
}

#[test]
fn rejeu_du_meme_command_id_rend_l_etat_sans_nouvel_evenement() {
    let conn = conn();
    let first = set(
        &conn,
        ControlMutation {
            auto_objectives_cap: Some(9),
            ..mutation("c1", 0)
        },
    )
    .unwrap()
    .unwrap();
    let replay = set(
        &conn,
        ControlMutation {
            auto_objectives_cap: Some(9),
            ..mutation("c1", 0)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(first, replay);
    assert_eq!(history(&conn, 10).unwrap().len(), 1);
    assert_eq!(history(&conn, 10).unwrap()[0].kind, "budget_set");
}

#[test]
fn etat_relu_apres_reouverture() {
    let dir = std::env::temp_dir().join(format!("rc-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("control.db");
    {
        let conn = Connection::open(&path).unwrap();
        ensure_schema(&conn).unwrap();
        set(
            &conn,
            ControlMutation {
                paused: Some(true),
                ..mutation("c1", 0)
            },
        )
        .unwrap()
        .unwrap();
    }
    let conn = Connection::open(&path).unwrap();
    ensure_schema(&conn).unwrap();
    let state = read(&conn).unwrap();
    assert!(state.paused, "la pause survit à la réouverture");
    assert_eq!(state.generation, 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn guidance156_pause_garde_table() {
    let mut state = read(&conn()).unwrap();
    for paused in [false, true] {
        state.paused = paused;
        assert_eq!(
            admit_autonomous_effect(AutonomousEffect::AgentGuidance, &state),
            if paused {
                Admission::Deferred { motif: "pause" }
            } else {
                Admission::Admitted
            }
        );
    }
}

#[test]
fn control158_halt_admission_4_combinaisons() {
    let mut control = read(&conn()).unwrap();
    for (paused, halted, expected) in [
        (false, false, Admission::Admitted),
        (true, false, Admission::Deferred { motif: "pause" }),
        (
            false,
            true,
            Admission::Deferred {
                motif: "delegations_halted",
            },
        ),
        (
            true,
            true,
            Admission::Deferred {
                motif: "delegations_halted",
            },
        ),
    ] {
        control.paused = paused;
        let halt = crate::delegation::HaltState {
            halted,
            since: None,
            by: None,
            reason: None,
            generation: 0,
        };
        assert_eq!(admit_native_delegation(&control, &halt), expected);
    }
}

#[test]
fn garde_unique_table_de_verite() {
    let mut active = read(&conn()).unwrap();
    for effect in [
        AutonomousEffect::ProjectRound,
        AutonomousEffect::ReminderNudge,
        AutonomousEffect::Continuation,
        AutonomousEffect::NativeDelegation,
        AutonomousEffect::NativeMissionDispatch,
    ] {
        assert_eq!(
            admit_autonomous_effect(effect, &active),
            Admission::Admitted
        );
    }
    active.paused = true;
    for effect in [
        AutonomousEffect::ProjectRound,
        AutonomousEffect::ReminderNudge,
        AutonomousEffect::Continuation,
        AutonomousEffect::NativeDelegation,
        AutonomousEffect::NativeMissionDispatch,
    ] {
        assert_eq!(
            admit_autonomous_effect(effect, &active),
            Admission::Deferred { motif: "pause" }
        );
    }
}

#[test]
fn principal_humain_reconnu_par_perimetre_seulement() {
    assert_eq!(
        human_principal_actor(&crate::communication::issuer_scope("bridget-ui-control")),
        Some("humain")
    );
    assert_eq!(
        human_principal_actor(&crate::communication::issuer_scope("bridget-control-cli")),
        Some("cli")
    );
    assert_eq!(
        human_principal_actor(&crate::communication::issuer_scope("bridget-ui")),
        None
    );
    assert_eq!(human_principal_actor("n-importe-quoi"), None);
}

#[test]
fn native152_horloges_pause_overlap_durable() {
    for (events, from, to, expected) in [
        (vec![], 0, 50, 0),
        (vec![(10, true), (30, false)], 0, 50, 20),
        (vec![(10, true)], 0, 30, 20),
        (
            vec![(10, true), (20, false), (25, true), (35, false), (40, true)],
            0,
            50,
            30,
        ),
        (vec![(10, true), (30, false)], 20, 40, 10),
        (
            vec![(10, true), (10, false), (10, true), (20, false)],
            10,
            30,
            10,
        ),
    ] {
        let conn = conn();
        for (index, (at, paused)) in events.into_iter().enumerate() {
            let command_id = format!("pause-{index}");
            set(
                &conn,
                ControlMutation {
                    paused: Some(paused),
                    now: at,
                    ..mutation(&command_id, index as u64)
                },
            )
            .unwrap()
            .unwrap();
        }
        assert_eq!(pause_overlap(&conn, from, to).unwrap(), expected);
    }
}

#[test]
fn native160_gel_union_intervalles() {
    for (name, events, waits, from, to, expected) in [
        ("aucune_attente", vec![], vec![], 0, 50, 0),
        (
            "attente_seule",
            vec![],
            vec![(10, 10, None::<i64>)],
            0,
            50,
            30,
        ),
        (
            "pause_seule",
            vec![(10, true), (30, false)],
            vec![],
            0,
            50,
            20,
        ),
        (
            "pause_dans_attente",
            vec![(600, true), (1200, false)],
            vec![(0, 1770, Some(1800))],
            0,
            1800,
            1800,
        ),
        (
            "chevauchement_partiel",
            vec![(0, true), (30, false)],
            vec![(20, 20, None)],
            0,
            60,
            50,
        ),
        (
            "ouverte_avant_borne",
            vec![],
            vec![(10, 20, None)],
            0,
            40,
            30,
        ),
        (
            "ouverte_apres_borne",
            vec![],
            vec![(10, 20, None)],
            0,
            90,
            40,
        ),
        (
            "entrees_closes",
            vec![],
            vec![(10, 100, Some(30)), (40, 100, Some(50))],
            0,
            200,
            30,
        ),
    ] {
        let conn = conn();
        for (index, (at, paused)) in events.into_iter().enumerate() {
            let command_id = format!("pause-160-{index}");
            set(
                &conn,
                ControlMutation {
                    paused: Some(paused),
                    now: at,
                    ..mutation(&command_id, index as u64)
                },
            )
            .unwrap()
            .unwrap();
        }
        // Forme du payload historique151, déjà utilisée par legacy_task151.
        let mut task: crate::delegation::Task = serde_json::from_value(serde_json::json!({
            "task_id":"gel160", "owner":"parent", "owner_instance":"parent-instance",
            "origin_owner_instance":"parent-instance",
            "request":{"operation":"delegate","request_id":"gel160", "agent_type":"fixture",
                "model":"fixture","task":"mission","cwd":"/tmp"},
            "definition":{"command":"fixture","args":[],"protocol":"acp","forbidden_env":[],
                "pass_env":[],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600,
                "mcp":{"interactive":"none","acp_session":false},"digest":"fixture"},
            "cwd":"/tmp","child":"child","child_instance":null,"mission":"mission160",
            "created_at":0,"state":"working","result":null,"error":null,"result_sent":false
        }))
        .unwrap();
        task.quota_waits = waits
            .into_iter()
            .enumerate()
            .map(
                |(index, (since, reset_at, until))| crate::delegation::QuotaWait {
                    since,
                    reset_at,
                    round: (index + 1) as u8,
                    identity: format!("limit-resume:gel160:{}:{reset_at}", index + 1),
                    execution_id: format!("execution-{index}"),
                    phase: if until.is_some() {
                        crate::delegation::QuotaWaitPhase::Done
                    } else {
                        crate::delegation::QuotaWaitPhase::Waiting
                    },
                    until,
                    resume_execution_id: None,
                },
            )
            .collect();
        assert_eq!(
            frozen_overlap(&conn, &task, from, to).unwrap(),
            expected,
            "{name}"
        );
    }
}

#[test]
fn resume_lisible() {
    let mut state = read(&conn()).unwrap();
    assert_eq!(
        summary_line(&state, 10),
        "Contrôle : autonomie active · plafond objectifs automatiques 5"
    );
    state.paused = true;
    state.paused_since = Some(0);
    assert_eq!(
        summary_line(&state, 2 * 3600 + 600),
        "Contrôle : pause depuis 2 h 10 · plafond objectifs automatiques 5"
    );
}
