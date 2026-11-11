use super::*;
fn registry() -> AgentRegistry {
    let mut models = serde_json::Map::new();
    for name in ["one", "two", "foreign", "off"] {
        models.insert(name.into(),json!([{"model":"fixture","supported_efforts":[],"default_effort":null,"provenance":"declared"}]));
    }
    AgentRegistry::from_json(&json!({
            "agents":{"fixture":{"command":"/fixture path/claude","protocol":"claude_stream_json"}},
            "machines":{"foreign":{"enabled":true,"worker_id":"foreign-worker","provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["fixture"],"mappings":{}}},
            "routing_policy":{"version":2,"revision":1,"enabled":true,
                "connections":[
                    {"connection_id":"one","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"two","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"foreign","machine_id":"foreign","engine":"claude","agent_type":"fixture","enabled":true},
                    {"connection_id":"off","machine_id":"local","engine":"claude","agent_type":"fixture","enabled":false}
                ],"model_catalog":models,"machine_policies":{}
            }}).to_string(),"/tmp/clients209.json").unwrap()
}
#[test]
fn spec209_local_connections_deduplicate_and_require_consent_before_upgrade() {
    let r = registry();
    let mut calls = 0;
    let report = report_with(&r, "here", "here", |program, engine| {
        calls += 1;
        assert_eq!(program, "/fixture path/claude");
        assert_eq!(engine, "claude");
        ("outdated".into(), Some("2.1.251".into()))
    })
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(report["clients"].as_array().unwrap().len(), 1);
    assert_eq!(report["clients"][0]["connections"], json!(["one", "two"]));
    assert_eq!(report["verdict"], "client_update_required");
    assert!(
        report["clients"][0]["recovery"]
            .as_str()
            .unwrap()
            .contains("Souhaitez-vous autoriser")
    );
    assert!(!report["authentication_checked"].as_bool().unwrap());
    assert!(valid_report(&report, "here"));
    assert!(!valid_report(&report, "foreign"));
    let mut bad = report.clone();
    bad["verdict"] = json!("compatible");
    assert!(!valid_report(&bad, "here"));
    bad = report.clone();
    bad["clients"][0]["state"] = json!("compatible");
    assert!(!valid_report(&bad, "here"));
    bad = report.clone();
    bad["secret"] = json!("private");
    assert!(!valid_report(&bad, "here"));
    let mut bad = report.clone();
    bad["clients"] = json!([]);
    bad["verdict"] = json!("compatible");
    assert!(!inventory_matches(&bad, &r, "here", "here"));
    bad = report.clone();
    bad["clients"][0]["minimum"] = json!("0.0.1");
    bad["policy"]["requirements"]["claude"]["minimum"] = json!("0.0.1");
    assert!(!valid_report(&bad, "here"));
    assert_eq!(
        report["policy"]["requirements"]["claude"]["provenance"],
        "provider_minimum_208"
    );
    assert!(inventory_matches(&report, &r, "here", "here"));
}
#[test]
fn spec209_numeric_versions_and_unknown_are_never_guessed() {
    for (engine, text, want) in [
        ("claude", "2.1.312 (Claude Code)", Some("2.1.312")),
        ("codex", "codex-cli 0.163.0", Some("0.163.0")),
        ("claude", "private api_error 400 version 2.1.280", None),
        ("codex", "codex-cli 0.163.0-beta", None),
        ("claude", "02.1.312 (Claude Code)", None),
    ] {
        assert_eq!(parse_version(engine, text.as_bytes()).as_deref(), want);
    }
    assert!(triplet("2.1.280") > triplet("2.1.99"));
    assert!(parse_version("claude", &[255]).is_none());
    for state in ["probe_failed", "version_unverified", "probe_timeout"] {
        let report = report_with(&registry(), "here", "here", |_, _| (state.into(), None)).unwrap();
        assert_eq!(report["verdict"], "client_check_unavailable");
        assert!(valid_report(&report, "here"));
        assert!(!report.to_string().contains("private"));
    }
    let report = report_with(&registry(), "here", "here", |_, _| {
        ("not_installed".into(), None)
    })
    .unwrap();
    assert_eq!(report["verdict"], "compatible");
    assert_eq!(report["clients"][0]["state"], "not_installed");
}
#[test]
fn spec209_dedicated_worker_identity_is_not_inferred_from_hostname() {
    let value = json!({
            "agents":{"fixture":{"command":"/fixture/claude","protocol":"claude_stream_json"}},
            "machines":{"sysle2":{"enabled":true,"worker_id":"worker-sysle2","provisioning_revision":1,
                "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["fixture"],"mappings":{}}},
            "routing_policy":{"version":2,"revision":1,"enabled":true,
                "connections":[{"connection_id":"claude-sys","machine_id":"sysle2","engine":"claude","agent_type":"fixture","enabled":true}],
                "model_catalog":{"claude-sys":[{"model":"fixture","supported_efforts":[],"default_effort":null,"provenance":"declared"}]},
                "machine_policies":{}}});
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/worker209.json").unwrap();
    let worker = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
        .join(".config/bridget/agents.json");
    let scope = machine_scope(&registry, &worker, "backup").unwrap();
    assert_eq!(scope, "sysle2");
    let report = report_with(&registry, "backup", &scope, |_, _| {
        ("compatible".into(), Some("2.1.310".into()))
    })
    .unwrap();
    assert_eq!(report["clients"].as_array().unwrap().len(), 1);
    assert_eq!(report["host"], "backup");
    assert_eq!(report["machine_id"], "sysle2");
    assert!(inventory_matches(&report, &registry, "backup", &scope));
    assert!(valid_report(&report, "sysle2"));
    assert!(machine_scope(&registry, Path::new("/tmp/global/agents.json"), "backup").is_err());
    let mut empty = report.clone();
    empty["clients"] = json!([]);
    assert!(!inventory_matches(&empty, &registry, "backup", &scope));
}

#[test]
fn spec209_update_report_keeps_legacy_wire_and_checks_diagnostics() {
    use bridget_transport::protocol::{UpdateRequest, UpdateStage};
    let report = report_with(&registry(), "here", "here", |_, _| {
        ("outdated".into(), Some("2.1.251".into()))
    })
    .unwrap();
    let q = UpdateRequest::Report {
        host: "here".into(),
        installed: "0.6.3".into(),
        available: Some("0.6.4".into()),
        stage: UpdateStage::ClientUpdateRequired,
        client_compatibility: Some(report.clone()),
    };
    let encoded = serde_json::to_value(q).unwrap();
    assert_eq!(encoded["stage"], "client_update_required");
    let decoded: UpdateRequest = serde_json::from_value(encoded).unwrap();
    let UpdateRequest::Report {
        client_compatibility: Some(checked),
        ..
    } = decoded
    else {
        panic!()
    };
    assert!(valid_report(&checked, "here"));
    let legacy = json!({"action":"report","host":"here","installed":"0.6.3","available":null,"stage":"ready"});
    let old: UpdateRequest = serde_json::from_value(legacy.clone()).unwrap();
    assert_eq!(serde_json::to_value(old).unwrap(), legacy);
    let mut bad = report;
    bad["clients"][0]["installed"] = json!("2.1.312");
    assert!(!valid_report(&bad, "here"));
}

#[test]
fn spec209_unconfirmed_probe_stop_is_explicit_without_forced_kill() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("clients209-term-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("claude");
    std::fs::write(&file, "#!/bin/sh\ntrap '' TERM\nsleep 2\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o700)).unwrap();
    let result = probe(
        file.to_str().unwrap(),
        "claude",
        Instant::now() + Duration::from_millis(300),
    );
    let pid = result
        .0
        .strip_prefix("probe_stop_unconfirmed:")
        .unwrap()
        .parse::<i32>()
        .unwrap();
    let report = report_with(&registry(), "here", "here", |_, _| result.clone()).unwrap();
    assert_eq!(report["verdict"], "client_check_unavailable");
    assert_eq!(report["clients"][0]["probe_pid"], pid);
    assert!(valid_report(&report, "here"));
    assert!(
        report["clients"][0]["recovery"]
            .as_str()
            .unwrap()
            .contains("Ne pas répéter automatiquement")
    );
    // Cette fixture termine naturellement après2s ; récolter notre seul enfant connu.
    // SAFETY: les pointeurs passés à waitpid désignent des variables locales valides pendant l'appel.
    assert_eq!(unsafe { libc::waitpid(pid, std::ptr::null_mut(), 0) }, pid);
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[test]
fn spec209_real_version_only_probe_is_bounded_and_excludes_raw_errors() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("clients209-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("claude");
    for (body, state, version) in [
        (
            "[ \"$#\" = 1 ] && [ \"$1\" = --version ] && [ \"$DISABLE_AUTOUPDATER\" = 1 ] || exit 9\nprintf '2.1.312 (Claude Code)\\n'",
            "compatible",
            Some("2.1.312"),
        ),
        (
            "printf '2.1.251 (Claude Code)\\n'",
            "outdated",
            Some("2.1.251"),
        ),
        (
            "printf 'PRIVATE-provider-message\\n'",
            "version_unverified",
            None,
        ),
        (
            "printf 'PRIVATE-provider-message' >&2\nexit 1",
            "probe_failed",
            None,
        ),
        (
            "while :; do printf 'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'; done",
            "output_limit",
            None,
        ),
        ("sleep 10", "probe_timeout", None),
    ] {
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let start = Instant::now();
        let result = probe(
            path.to_str().unwrap(),
            "claude",
            start + Duration::from_millis(300),
        );
        assert_eq!(result, (state.into(), version.map(str::to_string)));
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        probe(
            path.to_str().unwrap(),
            "claude",
            Instant::now() + Duration::from_secs(1)
        )
        .0,
        "not_installed"
    );
    assert_eq!(
        probe("/bin/sh", "claude", Instant::now() + Duration::from_secs(1)).0,
        "launcher_unverified"
    );
    std::fs::remove_dir(&dir).unwrap();
}
