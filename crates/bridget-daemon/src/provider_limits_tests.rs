use super::*;

#[test]
fn spec245_quota_migration_preserve_dates_et_reponse_positive() {
    let db = rusqlite::Connection::open_in_memory().unwrap();
    let old = "a".repeat(64);
    let alias = "b".repeat(64);
    let canonical = "c".repeat(64);
    let other = "d".repeat(64);
    let mut limits = ProviderLimits::restore_routing244(&db, 100);
    let mut fact = RateLimitFact {
        window: "seven_day".into(),
        status: "rejected".into(),
        resets_at: Some(1000),
        used_percent: None,
        credits: None,
    };
    limits.record_account244(&db, &old, &fact, RateLimitSource::ClaudeStreamJson, 110);
    limits.record_account244(&db, &alias, &fact, RateLimitSource::ClaudeStreamJson, 120);
    fact.window = "five_hour".into();
    fact.resets_at = Some(900);
    limits.record_account244(
        &db,
        &canonical,
        &fact,
        RateLimitSource::ClaudeStreamJson,
        130,
    );
    limits
        .migrate_account_scope245(&db, &alias, &canonical)
        .unwrap();
    limits
        .migrate_account_scope245(&db, &old, &canonical)
        .unwrap();
    limits
        .migrate_account_scope245(&db, &canonical, &canonical)
        .unwrap();
    let mut restored = ProviderLimits::restore_routing244(&db, 200);
    let seen = &restored.account_refusals244[&canonical]["seven_day"];
    assert_eq!((seen.observed_at, seen.fact.resets_at), (120, Some(1000)));
    assert_eq!(
        restored.account_refusals244[&canonical]["five_hour"].observed_at,
        130
    );
    assert!(
        restored
            .account_quota244(None, Some(&canonical), 200)
            .unwrap()
            .launch_blocked
    );
    for absent in [&old, &alias, &other] {
        assert!(restored.account_quota244(None, Some(absent), 200).is_none());
    }
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM provider_refusals244 WHERE scope IN (?1,?2)",
            rusqlite::params![old, alias],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    fact.status = "allowed".into();
    restored.record_account244(
        &db,
        &canonical,
        &fact,
        RateLimitSource::ClaudeStreamJson,
        210,
    );
    fact.window = "seven_day".into();
    restored.record_account244(
        &db,
        &canonical,
        &fact,
        RateLimitSource::ClaudeStreamJson,
        211,
    );
    let mut restored = ProviderLimits::restore_routing244(&db, 220);
    restored
        .migrate_account_scope245(&db, &old, &canonical)
        .unwrap();
    restored
        .migrate_account_scope245(&db, &alias, &canonical)
        .unwrap();
    assert!(
        restored
            .account_quota244(None, Some(&canonical), 220)
            .is_none()
    );
    assert_eq!(db.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name!='provider_refusals244'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}

#[test]
fn spec245_quota_migration_echec_disque_garde_fait_original() {
    let db = rusqlite::Connection::open_in_memory().unwrap();
    let old = "a".repeat(64);
    let canonical = "c".repeat(64);
    let mut limits = ProviderLimits::restore_routing244(&db, 100);
    let fact = RateLimitFact {
        window: "seven_day".into(),
        status: "rejected".into(),
        resets_at: Some(1000),
        used_percent: None,
        credits: None,
    };
    limits.record_account244(&db, &old, &fact, RateLimitSource::ClaudeStreamJson, 110);
    db.execute_batch("CREATE TRIGGER refuse245 BEFORE DELETE ON provider_refusals244 BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(
        limits
            .migrate_account_scope245(&db, &old, &canonical)
            .is_err()
    );
    assert!(
        limits
            .account_quota244(None, Some(&canonical), 200)
            .unwrap()
            .launch_blocked
    );
    assert!(
        ProviderLimits::restore_routing244(&db, 200)
            .account_quota244(None, Some(&old), 200)
            .unwrap()
            .launch_blocked
    );
    assert_eq!(
        db.query_row(
            "SELECT COUNT(*) FROM provider_refusals244 WHERE scope=?1",
            [canonical],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    db.execute_batch("DROP TRIGGER refuse245").unwrap();
    let mut positive = fact;
    positive.status = "allowed".into();
    let canonical = "c".repeat(64);
    limits.record_account244(
        &db,
        &canonical,
        &positive,
        RateLimitSource::ClaudeStreamJson,
        210,
    );
    assert!(
        limits
            .account_quota244(None, Some(&canonical), 220)
            .is_none()
    );
    let mut restored = ProviderLimits::restore_routing244(&db, 220);
    restored
        .migrate_account_scope245(&db, &old, &canonical)
        .unwrap();
    assert!(
        restored
            .account_quota244(None, Some(&canonical), 220)
            .is_none(),
        "aucune résurrection après échec, réponse positive et redémarrage"
    );
}

#[test]
fn spec244_quota_alias_reprise_compte_et_date_inconnue() {
    let db = rusqlite::Connection::open_in_memory().unwrap();
    let scope = "a".repeat(64);
    let mut limits = ProviderLimits::restore_routing244(&db, 100);
    let mut fact = RateLimitFact {
        window: "unknown".into(),
        status: "rejected".into(),
        resets_at: None,
        used_percent: None,
        credits: None,
    };
    limits.record_account244(&db, &scope, &fact, RateLimitSource::ClaudeStreamJson, 100);
    let restored = ProviderLimits::restore_routing244(&db, 200);
    let quota = restored.account_quota244(None, Some(&scope), 257).unwrap();
    assert!(quota.launch_blocked);
    assert_eq!(
        quota.resets_at, None,
        "la fin du délai de réévaluation n'est pas une date de retour du fournisseur"
    );
    assert!(
        restored
            .account_quota244(None, Some(&"b".repeat(64)), 257)
            .is_none()
    );
    assert!(restored.account_quota244(None, Some(&scope), 400).is_none());
    fact.window = "seven_day".into();
    fact.resets_at = Some(1000);
    limits.record_account244(&db, &scope, &fact, RateLimitSource::ClaudeStreamJson, 150);
    assert!(
        ProviderLimits::restore_routing244(&db, 500)
            .account_quota244(None, Some(&scope), 999)
            .unwrap()
            .launch_blocked
    );
    assert!(
        ProviderLimits::restore_routing244(&db, 1000)
            .account_quota244(None, Some(&scope), 1000)
            .is_none()
    );
    fact.window = "unknown".into();
    fact.resets_at = None;
    fact.status = "allowed".into();
    limits.record_account244(&db, &scope, &fact, RateLimitSource::ClaudeStreamJson, 160);
    assert!(
        ProviderLimits::restore_routing244(&db, 200)
            .account_refusals244
            .get(&scope)
            .is_none_or(|windows| !windows.contains_key("unknown"))
    );
    fact.status = "rejected".into();
    fact.window = "seven_day_opus".into();
    limits.record_account244(&db, &scope, &fact, RateLimitSource::ClaudeStreamJson, 200);
    assert!(!limits.account_refusals244[&scope].contains_key("seven_day_opus"));
}

#[test]
fn spec172_unresolved_claude_rejection_survives_routing_delay_until_recovery() {
    let mut limits = ProviderLimits::default();
    limits.record_scoped(
        "claude",
        "failed-child",
        RateLimitFact {
            window: "unknown".into(),
            status: "rejected".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::ClaudeStreamJson,
        90,
    );
    limits.bind_routing_instance(
        "cartae-claude",
        "claude",
        "failed-child",
        "digest",
        7,
        "scope",
    );
    let dead = std::collections::HashSet::new();
    for now in [137, 390] {
        let quota = limits
            .routing_quota("cartae-claude", "digest", 7, "scope", &dead, now, 30)
            .expect("an unresolved refusal is not a stale availability sample");
        assert!(quota.launch_blocked && quota.limit_reached);
        assert_eq!(
            quota.observed_at, 90,
            "the old observation is not made fresh"
        );
        assert!(quota.resets_at.is_none() && quota.credits.is_none());
    }
    assert!(
        limits
            .routing_quota("cartae-claude", "digest", 7, "scope", &dead, 391, 30)
            .is_none(),
        "unknown after the retry delay is not a positive availability fact"
    );
    limits.record_scoped(
        "claude",
        "recovered-child",
        RateLimitFact {
            window: "unknown".into(),
            status: "allowed".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::ClaudeStreamJson,
        5001,
    );
    limits.bind_routing_instance(
        "cartae-claude",
        "claude",
        "recovered-child",
        "digest",
        7,
        "scope",
    );
    let live = std::collections::HashSet::from(["recovered-child".into()]);
    assert!(
        !limits
            .routing_quota("cartae-claude", "digest", 7, "scope", &live, 5002, 30)
            .unwrap()
            .launch_blocked
    );
}

#[test]
fn spec172_dated_claude_refusal_expires_at_its_real_reset_without_positive_credit() {
    let mut limits = ProviderLimits::default();
    limits.record_scoped(
        "claude",
        "child",
        RateLimitFact {
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(200),
            used_percent: None,
            credits: None,
        },
        RateLimitSource::ClaudeStreamJson,
        90,
    );
    limits.bind_routing_instance("cartae-claude", "claude", "child", "haiku", 7, "scope");
    let dead = std::collections::HashSet::new();
    let before = limits
        .routing_quota("cartae-claude", "opus", 7, "scope", &dead, 199, 30)
        .unwrap();
    assert!(before.launch_blocked && before.exhausted);
    assert_eq!(before.resets_at, Some(200));
    assert!(before.credits.is_none());
    assert!(
        limits
            .routing_quota("cartae-claude", "opus", 7, "scope", &dead, 200, 30)
            .is_none()
    );
}

#[test]
fn spec172_recent_claude_rejection_survives_child_exit_only_for_same_connection() {
    let mut limits = ProviderLimits::default();
    limits.record_scoped(
        "claude",
        "child",
        RateLimitFact {
            window: "unknown".into(),
            status: "rejected".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::ClaudeStreamJson,
        90,
    );
    limits.record_scoped(
        "claude",
        "child",
        RateLimitFact {
            window: "account/credits".into(),
            status: "available".into(),
            resets_at: None,
            used_percent: None,
            credits: Some(bridget_transport::protocol::CodexCredits {
                has_credits: Some(true),
                unlimited: Some(true),
                spend_control_reached: false,
            }),
        },
        RateLimitSource::CodexAppServer,
        90,
    );
    limits.bind_routing_instance("cartae-claude", "claude", "child", "digest", 7, "scope");
    let empty = std::collections::HashSet::new();
    let quota = limits
        .routing_quota("cartae-claude", "digest", 7, "scope", &empty, 100, 30)
        .expect("recent observed rejection remains useful after failed child cleanup");
    assert!(quota.launch_blocked && quota.limit_reached);
    assert!(
        quota.credits.is_none(),
        "no positive credit carried from a dead instance"
    );
    assert!(
        !quota.exhausted && quota.resets_at.is_none(),
        "no window or reset invented"
    );
    for (connection, digest, epoch, now) in [
        ("another-account", "digest", 7, 100),
        ("cartae-claude", "digest", 8, 100),
    ] {
        assert!(
            limits
                .routing_quota(connection, digest, epoch, "scope", &empty, now, 30)
                .is_none()
        );
    }
    assert!(
        limits
            .routing_quota("cartae-claude", "other-model", 7, "scope", &empty, 137, 30)
            .unwrap()
            .launch_blocked
    );
    assert!(
        limits
            .routing_quota(
                "cartae-claude",
                "digest",
                7,
                "changed-account",
                &empty,
                100,
                30
            )
            .is_none()
    );
    limits.record_scoped(
        "claude",
        "child",
        RateLimitFact {
            window: "unknown".into(),
            status: "allowed".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::ClaudeStreamJson,
        101,
    );
    assert!(
        limits
            .routing_quota("cartae-claude", "digest", 7, "scope", &empty, 102, 30)
            .is_none()
    );
    let live = std::collections::HashSet::from(["child".into()]);
    assert!(
        !limits
            .routing_quota("cartae-claude", "digest", 7, "scope", &live, 102, 30)
            .unwrap()
            .launch_blocked
    );
}

#[test]
fn spec168_quota_connexion_instance_definition_epoque_ttl_et_credits_100() {
    use bridget_transport::protocol::CodexCredits;
    let mut limits = ProviderLimits::default();
    let fact = RateLimitFact {
        window: "weekly".into(),
        status: "rate_limit_reached".into(),
        resets_at: Some(500),
        used_percent: Some(100),
        credits: Some(CodexCredits {
            has_credits: Some(true),
            unlimited: Some(false),
            spend_control_reached: false,
        }),
    };
    limits.record_scoped(
        "codex",
        "personal",
        fact.clone(),
        RateLimitSource::CodexAppServer,
        90,
    );
    limits.record_scoped(
        "codex",
        "work",
        RateLimitFact {
            credits: None,
            ..fact
        },
        RateLimitSource::CodexAppServer,
        90,
    );
    limits.bind_routing_instance("personal", "codex", "personal", "digest-a", 7, "scope");
    limits.bind_routing_instance("work", "codex", "work", "digest-b", 7, "scope");
    let live = std::collections::HashSet::from(["personal".into(), "work".into()]);
    let personal = limits
        .routing_quota("personal", "digest-a", 7, "scope", &live, 100, 30)
        .unwrap();
    assert!(personal.exhausted);
    assert!(
        !personal.launch_blocked,
        "crédits utilisables contre fenêtre100%"
    );
    assert!(
        limits
            .routing_quota("work", "digest-b", 7, "scope", &live, 100, 30)
            .unwrap()
            .launch_blocked
    );
    assert!(
        limits
            .routing_quota("personal", "digest-b", 7, "scope", &live, 100, 30)
            .is_none()
    );
    assert!(
        limits
            .routing_quota("personal", "digest-a", 8, "scope", &live, 100, 30)
            .is_none()
    );
    assert!(
        limits
            .routing_quota(
                "personal",
                "digest-a",
                7,
                "scope",
                &std::collections::HashSet::new(),
                100,
                30
            )
            .is_none()
    );
    assert!(
        limits
            .routing_quota("personal", "digest-a", 7, "scope", &live, 121, 30)
            .is_none()
    );
    assert!(
        limits
            .routing_quota("personal", "digest-a", 7, "scope", &live, 89, 30)
            .is_none()
    );
    assert!(
        limits
            .routing_quota("unknown", "digest-a", 7, "scope", &live, 100, 30)
            .is_none()
    );
}

#[test]
fn native157_credits_portee_retrait_et_plafond_instances() {
    use bridget_transport::protocol::CodexCredits;
    let mut limits = ProviderLimits::default();
    let mut fact = RateLimitFact {
        window: "weekly".into(),
        status: "rate_limit_reached".into(),
        resets_at: Some(500),
        used_percent: Some(100),
        credits: None,
    };
    for (has, unlimited, spend, blocked) in [
        (true, false, false, false),
        (false, true, false, false),
        (false, false, false, true),
        (true, false, true, true),
    ] {
        fact.credits = Some(CodexCredits {
            has_credits: Some(has),
            unlimited: Some(unlimited),
            spend_control_reached: spend,
        });
        limits.record_scoped(
            "codex",
            "cartae",
            fact.clone(),
            RateLimitSource::CodexAppServer,
            90,
        );
        let quota = limits.scoped_quota("codex", "cartae", 100).unwrap();
        assert!(quota.exhausted);
        assert_eq!(quota.windows[0]["used_percent"], 100);
        assert_eq!(quota.launch_blocked, blocked);
        assert_eq!(
            limits
                .scoped_exhausted_since("codex", "cartae", 80, 100, None)
                .is_some(),
            blocked
        );
        assert!(
            limits
                .scoped_quota("codex", "another-account", 100)
                .is_none()
        );
        assert!(limits.scoped_quota("codex-pro", "cartae", 100).is_none());
        assert!(limits.quota("codex", 100).unwrap().launch_blocked);
    }
    fact.credits = Some(CodexCredits {
        has_credits: Some(true),
        unlimited: Some(false),
        spend_control_reached: false,
    });
    limits.record_scoped(
        "codex",
        "cartae",
        fact.clone(),
        RateLimitSource::CodexAppServer,
        91,
    );
    limits.record_scoped(
        "codex",
        "other",
        RateLimitFact {
            credits: None,
            ..fact.clone()
        },
        RateLimitSource::CodexAppServer,
        92,
    );
    assert!(
        !limits
            .scoped_quota("codex", "cartae", 100)
            .unwrap()
            .launch_blocked
    );
    assert!(
        limits
            .scoped_quota("codex", "other", 100)
            .unwrap()
            .launch_blocked
    );
    // Mise à jour sparse : ancienne fenêtre préservée, crédit retiré immédiatement.
    limits.record_scoped(
        "codex",
        "cartae",
        RateLimitFact {
            window: "account/credits".into(),
            status: "observed".into(),
            resets_at: None,
            used_percent: None,
            credits: None,
        },
        RateLimitSource::CodexAppServer,
        93,
    );
    let quota = limits.scoped_quota("codex", "cartae", 100).unwrap();
    assert!(quota.launch_blocked);
    assert_eq!(quota.windows.len(), 1);
    assert!(
        limits
            .scoped_exhausted_since("codex", "cartae", 80, 100, None)
            .is_some()
    );
    limits.record_scoped(
        "claude",
        "cartae",
        fact.clone(),
        RateLimitSource::ClaudeStreamJson,
        94,
    );
    assert!(
        limits
            .scoped_quota("claude", "cartae", 100)
            .unwrap()
            .launch_blocked
    );
    // Le contrôle de dépenses prime aussi sans fenêtre ou après son reset.
    let spend = RateLimitFact {
        window: "account/credits".into(),
        status: "observed".into(),
        resets_at: None,
        used_percent: None,
        credits: Some(CodexCredits {
            has_credits: Some(true),
            unlimited: Some(true),
            spend_control_reached: true,
        }),
    };
    limits.record_scoped(
        "codex",
        "spend-only",
        spend.clone(),
        RateLimitSource::CodexAppServer,
        95,
    );
    let quota = limits.scoped_quota("codex", "spend-only", 100).unwrap();
    assert!(quota.launch_blocked);
    assert!(quota.windows.is_empty());
    limits.record_scoped(
        "codex",
        "cartae",
        spend,
        RateLimitSource::CodexAppServer,
        96,
    );
    assert!(
        limits
            .scoped_quota("codex", "cartae", 600)
            .unwrap()
            .launch_blocked
    );
    for instance in 0..4097 {
        limits.record_scoped(
            "codex",
            &format!("instance-{instance}"),
            fact.clone(),
            RateLimitSource::CodexAppServer,
            100 + instance,
        );
    }
    assert_eq!(limits.by_instance.len(), 4096);
    assert!(limits.scoped_quota("codex", "cartae", 100).is_none());
}

#[test]
fn native157_limits_regle_epuisement() {
    use RateLimitSource::{ClaudeStatusLine, ClaudeStreamJson, CodexAppServer};
    for (source, status, percent, reset, expected_exhausted, expected_reached) in [
        (ClaudeStreamJson, "rejected", None, Some(500), true, true),
        (
            ClaudeStreamJson,
            "rejected",
            Some(50),
            Some(500),
            true,
            true,
        ),
        (
            ClaudeStreamJson,
            "allowed",
            Some(100),
            Some(500),
            true,
            true,
        ),
        (
            ClaudeStreamJson,
            "allowed_warning",
            Some(99),
            Some(500),
            false,
            false,
        ),
        (ClaudeStreamJson, "rejected", Some(100), None, false, false),
        (CodexAppServer, "rejected", Some(50), Some(500), false, true),
        (
            CodexAppServer,
            "available",
            Some(100),
            Some(500),
            true,
            true,
        ),
        (
            CodexAppServer,
            "limit_reached",
            None,
            Some(500),
            false,
            true,
        ),
        (
            CodexAppServer,
            "limit_reached",
            Some(100),
            None,
            false,
            true,
        ),
        (CodexAppServer, "available", Some(50), None, false, false),
        (
            ClaudeStatusLine,
            "unknown",
            Some(50),
            Some(500),
            false,
            false,
        ),
        (ClaudeStatusLine, "unknown", None, Some(500), false, false),
        (
            ClaudeStatusLine,
            "unknown",
            Some(100),
            Some(500),
            true,
            true,
        ),
        (ClaudeStatusLine, "unknown", Some(100), None, false, false),
    ] {
        let mut limits = ProviderLimits::default();
        limits.record(
            "registry-provider",
            RateLimitFact {
                credits: None,
                window: "five_hour".into(),
                status: status.into(),
                resets_at: reset,
                used_percent: percent,
            },
            source,
            90,
        );
        let quota = limits.quota("registry-provider", 100).expect("fait connu");
        assert_eq!(
            quota.exhausted, expected_exhausted,
            "{source}/{status}/{percent:?}/{reset:?}"
        );
        assert_eq!(quota.limit_reached, expected_reached);
        assert_eq!(
            quota.resets_at,
            if expected_exhausted { reset } else { None }
        );
        assert_eq!(quota.observed_at, 90);
        assert_eq!(
            quota.windows,
            vec![serde_json::json!({
                "window":"five_hour", "used_percent":percent,
                "resets_at":reset, "exhausted":expected_exhausted,
            })]
        );
        let seen = &limits.by_agent_type["registry-provider"]["five_hour"];
        assert_eq!(seen.source, source);
    }
}

#[test]
fn native157_limits_statut_codex_global() {
    let mut limits = ProviderLimits::default();
    for (window, percent, reset, observed) in [
        ("primary/300m", 100, 500, 90),
        ("secondary/10080m", 61, 600_000, 91),
    ] {
        limits.record(
            "codex-pro",
            RateLimitFact {
                credits: None,
                window: window.into(),
                status: "limit_reached".into(),
                resets_at: Some(reset),
                used_percent: Some(percent),
            },
            RateLimitSource::CodexAppServer,
            observed,
        );
    }
    let quota = limits.quota("codex-pro", 100).expect("deux fenêtres");
    assert!(quota.limit_reached);
    assert!(quota.exhausted);
    assert_eq!(quota.resets_at, Some(500));
    assert_eq!(quota.observed_at, 91);
    assert_eq!(quota.windows[0]["exhausted"], true);
    assert_eq!(quota.windows[1]["exhausted"], false);
    limits.record(
        "codex-pro",
        RateLimitFact {
            credits: None,
            window: "primary/300m".into(),
            status: "limit_reached".into(),
            resets_at: Some(500),
            used_percent: Some(99),
        },
        RateLimitSource::CodexAppServer,
        92,
    );
    let quota = limits.quota("codex-pro", 100).expect("statut global connu");
    assert!(quota.limit_reached);
    assert!(!quota.exhausted);
    assert_eq!(quota.resets_at, None);
    assert_eq!(quota.windows.len(), 2);
}

#[test]
fn native157_limits_fenetre_perimee() {
    let mut limits = ProviderLimits::default();
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(100),
            used_percent: Some(100),
        },
        RateLimitSource::ClaudeStreamJson,
        90,
    );
    assert!(
        limits.by_agent_type.contains_key("claude"),
        "record mémorise la fenêtre avant la lecture"
    );
    assert!(
        limits.quota("claude", 100).is_none(),
        "retour exact atteint"
    );
    assert!(
        limits.by_agent_type["claude"].contains_key("five_hour"),
        "lecture sans purge"
    );
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "unknown".into(),
            status: "unknown".into(),
            resets_at: None,
            used_percent: Some(50),
        },
        RateLimitSource::ClaudeStatusLine,
        100,
    );
    assert_eq!(limits.by_agent_type["claude"].len(), 1);
    assert!(!limits.by_agent_type["claude"].contains_key("five_hour"));
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "already-past".into(),
            status: "rejected".into(),
            resets_at: Some(100),
            used_percent: Some(100),
        },
        RateLimitSource::ClaudeStreamJson,
        100,
    );
    assert_eq!(limits.by_agent_type["claude"].len(), 1);
    let quota = limits
        .quota("claude", 100)
        .expect("fait sans heure conservé");
    assert!(!quota.exhausted);
    assert_eq!(quota.resets_at, None);
    // La dernière observation passée retire aussi l'ancien fait de même fenêtre.
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(500),
            used_percent: Some(100),
        },
        RateLimitSource::ClaudeStreamJson,
        100,
    );
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(95),
            used_percent: Some(100),
        },
        RateLimitSource::ClaudeStreamJson,
        101,
    );
    assert!(!limits.by_agent_type["claude"].contains_key("five_hour"));
    assert!(
        !limits
            .quota("claude", 101)
            .expect("autre fenêtre conservée")
            .exhausted
    );
}

#[test]
fn native157_limits_fenetres_independantes() {
    let mut limits = ProviderLimits::default();
    for (window, reset) in [("five_hour", 500), ("seven_day", 6000)] {
        limits.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: window.into(),
                status: "rejected".into(),
                resets_at: Some(reset),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            90,
        );
    }
    let quota = limits.quota("claude", 100).expect("fenêtres indépendantes");
    assert_eq!(quota.windows.len(), 2);
    assert_eq!(
        quota.resets_at,
        Some(6000),
        "retour le plus tardif des épuisées"
    );
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "seven_day".into(),
            status: "unknown".into(),
            resets_at: Some(6000),
            used_percent: Some(50),
        },
        RateLimitSource::ClaudeStatusLine,
        101,
    );
    let quota = limits.quota("claude", 102).expect("aucune fenêtre effacée");
    assert_eq!(quota.windows.len(), 2);
    assert_eq!(
        quota.resets_at,
        Some(500),
        "fenêtre non épuisée exclue du retour"
    );
    assert_eq!(quota.observed_at, 101);
    assert_eq!(
        limits.by_agent_type["claude"]["seven_day"].source,
        RateLimitSource::ClaudeStatusLine
    );
    assert_eq!(
        limits.by_agent_type["claude"]["five_hour"]
            .fact
            .used_percent,
        Some(100)
    );
}

#[test]
fn native157_limits_cle_agent_type() {
    let mut limits = ProviderLimits::default();
    assert!(limits.quota("unknown-provider", 100).is_none());
    for (agent_type, percent, source) in [
        ("claude-child", 100, RateLimitSource::ClaudeStreamJson),
        ("glm", 50, RateLimitSource::ClaudeStatusLine),
        ("codex-pro", 100, RateLimitSource::CodexAppServer),
    ] {
        limits.record(
            agent_type,
            RateLimitFact {
                credits: None,
                window: "shared-window".into(),
                status: "unknown".into(),
                resets_at: Some(500),
                used_percent: Some(percent),
            },
            source,
            90,
        );
    }
    assert!(limits.quota("claude-child", 100).unwrap().exhausted);
    assert!(!limits.quota("glm", 100).unwrap().exhausted);
    assert!(limits.quota("codex-pro", 100).unwrap().exhausted);
    assert!(limits.quota("claude-stream-json", 100).is_none());
    assert!(limits.quota("claude-statusline", 100).is_none());
    assert!(limits.quota("codex-app-server", 100).is_none());
    assert_eq!(limits.by_agent_type.len(), 3);
}

#[test]
fn native157_limits_borne_huit_fenetres() {
    let mut limits = ProviderLimits::default();
    for index in 0..8 {
        limits.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: format!("w{index}"),
                status: "unknown".into(),
                resets_at: None,
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            90 + index,
        );
    }
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "w0".into(),
            status: "unknown".into(),
            resets_at: None,
            used_percent: Some(50),
        },
        RateLimitSource::ClaudeStatusLine,
        100,
    );
    assert_eq!(limits.by_agent_type["claude"].len(), 8);
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "w8".into(),
            status: "unknown".into(),
            resets_at: None,
            used_percent: Some(50),
        },
        RateLimitSource::ClaudeStatusLine,
        101,
    );
    let windows = &limits.by_agent_type["claude"];
    assert_eq!(windows.len(), 8);
    assert!(
        !windows.contains_key("w1"),
        "éviction du plus ancien après l'upsert"
    );
    assert!(windows.contains_key("w0") && windows.contains_key("w8"));

    let mut tied = ProviderLimits::default();
    for window in ["c", "b", "a", "d", "e", "f", "g", "h", "z"] {
        tied.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: window.into(),
                status: "unknown".into(),
                resets_at: None,
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            90,
        );
    }
    let windows = &tied.by_agent_type["claude"];
    assert_eq!(windows.len(), 8);
    assert!(
        !windows.contains_key("a"),
        "égalité départagée par clé lexicale"
    );
    assert!(windows.contains_key("z"));

    let mut purged = ProviderLimits::default();
    for index in 0..8 {
        purged.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: format!("w{index}"),
                status: "unknown".into(),
                resets_at: Some(if matches!(index, 1 | 2) { 95 } else { 500 }),
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            90,
        );
    }
    purged.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "w8".into(),
            status: "unknown".into(),
            resets_at: Some(500),
            used_percent: Some(50),
        },
        RateLimitSource::ClaudeStatusLine,
        100,
    );
    let windows = &purged.by_agent_type["claude"];
    assert_eq!(windows.len(), 7, "purge avant éviction à la neuvième clé");
    assert!(windows.contains_key("w0"));
    assert!(!windows.contains_key("w1") && !windows.contains_key("w2"));
}
#[test]
fn native160_limits_fait_ancien_ignore() {
    for (observed, expected) in [(89, None), (90, None), (91, Some((500, 91)))] {
        let mut limits = ProviderLimits::default();
        limits.record(
            "claude-child",
            RateLimitFact {
                credits: None,
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(500),
                used_percent: None,
            },
            RateLimitSource::ClaudeStreamJson,
            observed,
        );
        assert_eq!(
            limits.attested_exhausted_since("claude-child", 90, 100, None),
            expected
        );
        assert!(
            limits.quota("claude-child", 100).unwrap().exhausted,
            "l'admission157 garde rejected sans pourcentage"
        );
    }
}

#[test]
fn native160_limits_codex_global_sans_fenetre_100_ignore() {
    let mut limits = ProviderLimits::default();
    limits.record(
        "codex-pro",
        RateLimitFact {
            credits: None,
            window: "secondary".into(),
            status: "limit_reached".into(),
            resets_at: Some(600_000),
            used_percent: None,
        },
        RateLimitSource::CodexAppServer,
        99,
    );
    assert_eq!(
        limits.attested_exhausted_since("codex-pro", 80, 100, None),
        None
    );
    let quota = limits.quota("codex-pro", 100).unwrap();
    assert!(quota.limit_reached && !quota.exhausted);
    limits.record(
        "codex-pro",
        RateLimitFact {
            credits: None,
            window: "primary".into(),
            status: "limit_reached".into(),
            resets_at: Some(500),
            used_percent: Some(100),
        },
        RateLimitSource::CodexAppServer,
        100,
    );
    assert_eq!(
        limits.attested_exhausted_since("codex-pro", 80, 101, None),
        Some((500, 100)),
        "la fenêtre globale non attestée n'allonge ni reset ni observation"
    );
}

#[test]
fn native160_limits_ancienne_observation_perimee_ignore() {
    let mut limits = ProviderLimits::default();
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(99),
            used_percent: Some(100),
        },
        RateLimitSource::ClaudeStreamJson,
        95,
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 104, None),
        None
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 104, Some(100)),
        None,
        "observation antérieure au hold : aucun Past inventé"
    );
    assert!(limits.quota("claude", 104).is_none());
    assert!(
        limits.by_agent_type["claude"].contains_key("five_hour"),
        "lecture sans purge"
    );
}

#[test]
fn native160_limits_hold_fait_expire_admis() {
    let mut limits = ProviderLimits::default();
    limits.record(
        "claude",
        RateLimitFact {
            credits: None,
            window: "five_hour".into(),
            status: "rejected".into(),
            resets_at: Some(103),
            used_percent: None,
        },
        RateLimitSource::ClaudeStreamJson,
        102,
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 104, None),
        None
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 104, Some(100)),
        Some((103, 102))
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 110, Some(100)),
        Some((103, 102)),
        "un tick retardé ne change pas l'instant d'observation admissible"
    );
    assert!(
        limits.quota("claude", 104).is_none(),
        "admission157 inchangée malgré Past160"
    );
}

#[test]
fn native160_limits_hold_filtre_chaque_fenetre_avant_agregation() {
    let mut limits = ProviderLimits::default();
    for (window, reset, observed) in [("admissible", 130, 102), ("tardive", 200, 106)] {
        limits.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: window.into(),
                status: "unknown".into(),
                resets_at: Some(reset),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStatusLine,
            observed,
        );
    }
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 110, Some(100)),
        Some((130, 102)),
        "la fenêtre tardive est écartée avant reset_max et observed_max"
    );
    for (window, reset, observed) in [("retour_maximal", 400, 101), ("borne_inclusive", 150, 105)] {
        limits.record(
            "claude",
            RateLimitFact {
                credits: None,
                window: window.into(),
                status: "unknown".into(),
                resets_at: Some(reset),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStatusLine,
            observed,
        );
    }
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 110, Some(100)),
        Some((400, 105)),
        "maxima indépendants sur les seules observations jusqu'à since_hold+5 inclus"
    );
    assert_eq!(
        limits.attested_exhausted_since("claude", 90, 110, None),
        Some((400, 106)),
        "hors hold, aucune borne de cinq secondes ajoutée à157"
    );
}
