use super::*;

#[test]
fn spec_091_development_is_scoped_and_denies_extension() {
    use bridget_transport::protocol::SpawnPosture;
    let registry = AgentRegistry::from_json(r#"{"agents":{}}"#, "/tmp/registry-091.json").unwrap();
    let original = registry.resolved_definition("codex").unwrap();
    let scoped = registry
        .for_spawn_posture("codex", SpawnPosture::Development)
        .unwrap();
    let definition = scoped.get("codex").unwrap();
    assert_eq!(definition.permissions, "deny");
    assert!(
        definition
            .args
            .windows(2)
            .any(|pair| pair == ["--sandbox", "workspace-write"])
    );
    assert!(
        definition
            .args
            .iter()
            .any(|arg| arg == "sandbox_workspace_write.network_access=false")
    );
    assert!(!definition.args.iter().any(|arg| arg.contains("bypass")));
    assert_eq!(registry.resolved_definition("codex").unwrap(), original);
    assert!(
        registry
            .for_spawn_posture("claude", SpawnPosture::Development)
            .is_err()
    );
    let frozen = scoped.resolved_definition("codex").unwrap();
    assert_eq!(
        AgentRegistry::from_resolved("codex", &frozen)
            .unwrap()
            .resolved_definition("codex")
            .unwrap(),
        frozen
    );
}

#[test]
#[cfg(target_os = "macos")]
#[ignore = "recette avec le bac à sable du binaire Codex installé"]
fn spec_091_native_development_writes_only_cwd_without_network() {
    use bridget_transport::protocol::SpawnPosture;
    let root = std::env::temp_dir().join(format!("bridget-sandbox-091-{}", uuid::Uuid::new_v4()));
    let cwd = root.join("work");
    let codex_home = root.join("codex");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&codex_home).unwrap();
    // Un profil utilisateur permissif ne doit pas agrandir l'ordre.
    std::fs::write(codex_home.join("config.toml"), "sandbox_mode = \"danger-full-access\"\napproval_policy = \"never\"\n[sandbox_workspace_write]\nnetwork_access = true\nwritable_roots = [\"/tmp\"]\n").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let registry = AgentRegistry::from_json(r#"{"agents":{}}"#, root.join("agents.json")).unwrap();
    let scoped = registry
        .for_spawn_posture("codex", SpawnPosture::Development)
        .unwrap();
    let definition = scoped.get("codex").unwrap();
    let mut args = definition.args.clone();
    args.retain(|arg| arg != "app-server");
    let script = format!(
        "touch inside && if touch ../outside; then exit 91; fi; if /usr/bin/curl --noproxy '*' --connect-timeout 2 --max-time 2 http://{}/; then exit 92; fi",
        listener.local_addr().unwrap()
    );
    let output = std::process::Command::new(&definition.command)
        .args(args)
        .args(["sandbox", "/bin/sh", "-c", &script])
        .current_dir(&cwd)
        .env("CODEX_HOME", &codex_home)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(cwd.join("inside").exists());
    assert!(!root.join("outside").exists());
    assert!(
        matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
    std::fs::remove_dir_all(root).unwrap();
}
use std::os::unix::fs::symlink;

fn test_root(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bridget-registry-{label}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn write_registry(path: &Path, mode: u32) {
    std::fs::write(path, "{\"agents\":{}}").unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

fn remote162_registry_value() -> serde_json::Value {
    serde_json::json!({"machines":{"cartae":{
        "worker_id":"worker-cartae", "provisioning_revision":1,
        "credential_fingerprint":format!("{:x}", Sha256::digest(b"private-worker-credential")),
        "allowed_agent_types":["codex"],
        "mappings":{"cartae":{
            "source_host":"mac", "source_root":"/Users/moi/projects/cartae",
            "remote_root":"/home/moi/20.cartae", "expected_git_identity":"guthubrx/cartae",
            "mapping_revision":1
        }}
    }}})
}

fn admission163_registry_value() -> serde_json::Value {
    serde_json::json!({
        "historical_future_key": {"preserved": true},
        "decision_services": {
            "laya": {
                "machine_id": "decision-host",
                "protocol": "laya_systemone_v1",
                "model": "typed-decisions",
                "endpoint": "https://decision.internal/v1",
            }
        },
        "admission_policy": {
            "revision": 1,
            "profiles": [{
                "id": "local-codex",
                "machine_id": "local",
                "agent_type": "codex",
                "model": "gpt-5.6",
                "effort": "high",
                "description": "Revue locale"
            }]
        }
    })
}

#[test]
fn spec163_configuration_admission_est_stricte_et_desactivee_par_defaut() {
    let value = admission163_registry_value();
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    let policy = registry.admission_policy().unwrap();
    assert!(!policy.auto_engine);
    assert!(!policy.auto_placement);
    assert_eq!(policy.profiles.len(), 1);
    assert_eq!(
        registry.decision_service("laya").unwrap_err(),
        "decision_service_disabled"
    );
    assert_eq!(registry.decision_services()["laya"].health_ttl_seconds, 30);

    let mut unknown = value.clone();
    unknown["decision_services"]["laya"]["secret_ref"] = "forbidden".into();
    assert!(AgentRegistry::from_json(&unknown.to_string(), "/tmp/agents.json").is_err());

    let mut profile_limit = value.clone();
    let profile = profile_limit["admission_policy"]["profiles"][0].clone();
    profile_limit["admission_policy"]["profiles"] =
        serde_json::Value::Array(vec![profile; MAX_ADMISSION_PROFILES + 1]);
    assert!(AgentRegistry::from_json(&profile_limit.to_string(), "/tmp/agents.json").is_err());

    let mut reference = value;
    reference["admission_policy"]["fixed_profile"] = "missing".into();
    assert_eq!(
        AgentRegistry::from_json(&reference.to_string(), "/tmp/agents.json").unwrap_err(),
        "admission_profile_reference_unknown"
    );
}

#[test]
fn spec244_quota_alias_meme_lanceur_sans_confondre_comptes() {
    let mut value = routing168_registry_value();
    value["agents"]["claude-alias"] = value["agents"]["claude"].clone();
    value["agents"]["claude"]["args"] = serde_json::json!(["--model", "claude-opus-5-5"]);
    value["agents"]["claude-alias"]["args"] = serde_json::json!(["--model=claude-haiku-5-5"]);
    let connection = serde_json::json!({"connection_id":"claude-alias", "machine_id":"local", "engine":"claude", "agent_type":"claude-alias", "enabled":true});
    value["routing_policy"]["connections"]
        .as_array_mut()
        .unwrap()
        .push(connection);
    value["routing_policy"]["model_catalog"]["claude-alias"] =
        value["routing_policy"]["model_catalog"]["claude-local"].clone();
    value["agents"]["claude"]["capabilities"] =
        serde_json::json!({"models":{"claude-opus-4-5":{"efforts":["high"]}}});
    value["agents"]["claude-alias"]["capabilities"] =
        serde_json::json!({"models":{"claude-haiku-4-5":{"efforts":[]}}});
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    assert_ne!(
        registry.routing_account_scope_legacy245("claude-local"),
        registry.routing_account_scope_legacy245("claude-alias")
    );
    assert!(registry.routing_account_scope244("claude-local").is_some());
    assert_eq!(
        registry.routing_account_scope244("claude-local"),
        registry.routing_account_scope244("claude-alias")
    );
    assert_ne!(
        registry.routing_account_scope244("claude-local"),
        registry.routing_account_scope244("claude-cartae")
    );
    assert!(
        registry
            .routing_account_scope244("codex-personal")
            .is_none()
    );
    value["agents"]["claude-alias"]["claude_config_dir"] = serde_json::json!("/another/account");
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    assert_ne!(
        registry.routing_account_scope244("claude-local"),
        registry.routing_account_scope244("claude-alias")
    );
}

fn routing168_registry_value() -> serde_json::Value {
    let mut value = remote162_registry_value();
    value["machines"]["cartae"]["enabled"] = true.into();
    value["agents"] = serde_json::json!({
        "codex-personal": {"command":"codex", "protocol":"codex_app_server"},
        "codex-work": {"command":"codex", "protocol":"codex_app_server"},
        "claude": {"command":"claude", "protocol":"claude_stream_json"}
    });
    value["routing_policy"] = serde_json::json!({
        "version": 2,
        "revision": 1,
        "enabled": true,
        "connections": [
            {"connection_id":"codex-personal", "machine_id":"local", "engine":"codex", "agent_type":"codex-personal", "enabled":true},
            {"connection_id":"codex-work", "machine_id":"cartae", "engine":"codex", "agent_type":"codex-work", "enabled":true},
            {"connection_id":"claude-local", "machine_id":"local", "engine":"claude", "agent_type":"claude", "enabled":true},
            {"connection_id":"claude-cartae", "machine_id":"cartae", "engine":"claude", "agent_type":"claude", "enabled":true}
        ],
        "model_catalog": {
            "codex-personal": [
                {"model":"gpt-6.1-sol", "supported_efforts":["low","high"], "default_effort":"high", "provenance":"operator_observation"},
                {"model":"gpt-6.1-luna", "supported_efforts":[], "default_effort":null, "provenance":"codex_model_list"},
                {"model":"gpt-6.1-terra", "supported_efforts":["medium"], "default_effort":"medium", "provenance":"codex_model_list"}
            ],
            "codex-work": [
                {"model":"gpt-6.1-sol", "supported_efforts":["low","high"], "default_effort":"high", "provenance":"operator_observation"},
                {"model":"gpt-6.1-luna", "supported_efforts":[], "default_effort":null, "provenance":"codex_model_list"},
                {"model":"gpt-6.1-terra", "supported_efforts":["medium"], "default_effort":"medium", "provenance":"codex_model_list"}
            ],
            "claude-local": [
                {"model":"claude-sonnet-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                {"model":"claude-opus-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                {"model":"claude-haiku-4-5", "supported_efforts":[], "default_effort":null, "provenance":"declared"}
            ],
            "claude-cartae": [
                {"model":"claude-sonnet-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                {"model":"claude-opus-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                {"model":"claude-haiku-4-5", "supported_efforts":[], "default_effort":null, "provenance":"declared"}
            ]
        },
        "machine_policies": {
            "local": {"enabled":true, "decision_timeout_ms":10000, "service_timeout_ms":2000},
            "cartae": {"enabled":true, "decision_timeout_ms":10000, "service_timeout_ms":2000}
        }
    });
    value
}

#[test]
fn routing168_catalogue_12_preserve_connexions_efforts_et_defaut_natif() {
    let registry = AgentRegistry::from_json(
        &routing168_registry_value().to_string(),
        "/tmp/agents-routing168.json",
    )
    .unwrap();
    let catalogue = registry.routing_catalogue(1_700_000_000).unwrap();
    assert_eq!(catalogue.version, 1);
    assert_eq!(catalogue.options.len(), 12);
    assert_eq!(
        registry.routing_policy().unwrap().machine_policies["local"].compilation_capacity,
        None,
        "absence de déclaration reste inconnue ; aucune vitesse n'est déduite"
    );
    assert!(catalogue.options.iter().any(|option| {
        option.connection_id == "codex-personal"
            && option.model == "gpt-6.1-sol"
            && option
                .supported_efforts
                .iter()
                .map(String::as_str)
                .eq(["low", "high"])
            && option.default_effort.as_deref() == Some("high")
    }));
    assert!(catalogue.options.iter().any(|option| {
        option.connection_id == "codex-work"
            && option.model == "gpt-6.1-luna"
            && option.supported_efforts.is_empty()
            && option.default_effort.is_none()
    }));
    assert!(catalogue.options.iter().any(|option| {
        option.connection_id == "claude-local"
            && option.model == "claude-sonnet-4-5"
            && option.entitlement == RoutingEntitlement::Unknown
    }));
}

#[test]
fn spec221_c05_explicit_tier_replaces_every_override_and_preserves_legacy_launch() {
    let mut registry = AgentRegistry::from_json(
        &routing168_registry_value().to_string(),
        "/tmp/options.json",
    )
    .unwrap();
    let definition = registry.agents.get_mut("codex-work").unwrap();
    definition
        .capabilities
        .models
        .entry("gpt-6.1-sol".into())
        .or_default()
        .service_tiers = vec!["priority".into(), "default".into()];
    definition.capabilities.execution_paths = vec!["codex_app_server".into()];
    definition.args = vec![
        "app-server".into(),
        "-c".into(),
        "service_tier = \"fast\"".into(),
        "--config=service_tier=\"flex\"".into(),
        "-cservice_tier=\"default\"".into(),
        "--config".into(),
        "\"service_tier\" = \"old\"".into(),
        "-c".into(),
        "other=\"kept\"".into(),
        "-c".into(),
        "model=\"gpt-6.1-sol\"".into(),
    ];
    let before = registry.resolved_definition("codex-work").unwrap();
    let legacy = registry
        .clone()
        .pin_delegation_service_tier("codex-work", "gpt-6.1-sol", None)
        .unwrap();
    assert_eq!(legacy.resolved_definition("codex-work").unwrap(), before);
    let pinned = registry
        .clone()
        .pin_delegation_service_tier("codex-work", "gpt-6.1-sol", Some("priority"))
        .unwrap();
    assert_eq!(
        pinned.get("codex-work").unwrap().args,
        vec![
            "app-server",
            "-c",
            "other=\"kept\"",
            "-c",
            "model=\"gpt-6.1-sol\"",
            "-c",
            "service_tier=\"priority\""
        ]
    );
    assert_eq!(registry.resolved_definition("codex-work").unwrap(), before);
    let mixed = pinned
        .pin_delegation_model("codex-work", "gpt-6.1-sol", None)
        .unwrap();
    let definition = mixed.resolved_definition("codex-work").unwrap();
    assert_eq!(
        mixed
            .pin_delegation_service_tier("codex-work", "gpt-6.1-sol", Some("priority"))
            .unwrap()
            .resolved_definition("codex-work")
            .unwrap(),
        definition
    );
    assert_eq!(
        registry
            .clone()
            .pin_delegation_service_tier("codex-work", "gpt-6.1-sol", Some("unknown"))
            .unwrap_err(),
        "model_options_not_permitted"
    );
    assert_eq!(
        registry
            .clone()
            .pin_delegation_service_tier("claude", "claude-sonnet-4-5", Some("priority"))
            .unwrap_err(),
        "model_options_not_permitted"
    );
    registry
        .agents
        .get_mut("codex-work")
        .unwrap()
        .args
        .push("--".into());
    assert_eq!(
        registry
            .pin_delegation_service_tier("codex-work", "gpt-6.1-sol", Some("priority"))
            .unwrap_err(),
        "model_options_launcher_ambiguous"
    );
}

#[test]
fn routing168_129_meme_desactive_refuse_sans_catalogue_partiel() {
    let mut value = routing168_registry_value();
    value["routing_policy"]["connections"].as_array_mut().unwrap().push(
            serde_json::json!({"connection_id":"overflow-disabled", "machine_id":"cartae", "engine":"codex", "agent_type":"codex-work", "enabled":false}),
        );
    value["routing_policy"]["model_catalog"]["overflow-disabled"] = serde_json::Value::Array(
        (0..129)
            .map(|index| {
                serde_json::json!({
                    "model": format!("declared-overflow-{index}"),
                    "supported_efforts": [],
                    "default_effort": null,
                    "provenance": "declared"
                })
            })
            .collect(),
    );
    assert_eq!(
        AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").unwrap_err(),
        "candidate_limit"
    );
}

#[test]
fn routing168_axes_independants_fixes_et_contradiction_restent_des_filtres() {
    let mut value = routing168_registry_value();
    value["routing_policy"]["machine_policies"]["local"]["automatic_axes"] = serde_json::json!({"engine":false, "model":true, "machine":true, "connection":true, "effort":true});
    value["routing_policy"]["machine_policies"]["local"]["fixed_values"] =
        serde_json::json!({"engine":"codex"});
    let registry =
        AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").unwrap();
    let selected = registry
        .select_routing_candidates(
            "local",
            &RoutingCandidateFilters {
                model: Some("gpt-6.1-sol".into()),
                ..RoutingCandidateFilters::default()
            },
        )
        .unwrap();
    assert_eq!(selected.len(), 2, "un modèle conserve les deux machines");
    assert_eq!(selected[0].candidate.machine_id, "cartae");
    assert_eq!(selected[1].candidate.machine_id, "local");
    assert!(
        registry
            .select_routing_candidates(
                "local",
                &RoutingCandidateFilters {
                    connection_id: Some("claude-local".into()),
                    ..RoutingCandidateFilters::default()
                },
            )
            .unwrap()
            .is_empty()
    );
}

#[test]
fn routing168_decideurs_inaccessibles_sont_valides_et_claude_distant_refuse() {
    let mut value = routing168_registry_value();
    value["decision_services"] = serde_json::json!({"laya":{"machine_id":"local", "protocol":"laya_systemone_v1", "model":"routing", "endpoint":"https://decision.internal/v1"}});
    value["routing_policy"]["machine_policies"]["local"]["decision_service"] =
        serde_json::json!({"enabled":true, "accessible":false, "service_id":"laya"});
    value["routing_policy"]["machine_policies"]["local"]["lightweight_decision"] = serde_json::json!({"enabled":true, "accessible":false, "engine":"claude", "connection_id":"claude-local", "model":"claude-sonnet-4-5", "effort":"high"});
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").is_ok());

    value["routing_policy"]["machine_policies"]["local"]["lightweight_decision"]["connection_id"] =
        "claude-cartae".into();
    assert_eq!(
        AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").unwrap_err(),
        "routing_lightweight_decision_invalid"
    );
}

#[test]
fn spec168_telemetrie_cpu_charge_seuils_1_100_fermes() {
    let mut value = routing168_registry_value();
    value["routing_policy"]["machine_policies"]["local"]["telemetry"] =
        serde_json::json!({"enabled":true});
    let defaults = RoutingScoreWeights::default();
    let registry =
        AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-defaults.json").unwrap();
    assert_eq!(
        registry.routing_policy().unwrap().machine_policies["local"]
            .telemetry
            .as_ref()
            .unwrap()
            .weights,
        defaults
    );
    let default_policy = serde_json::to_vec(registry.routing_policy().unwrap()).unwrap();
    let old_snapshot: serde_json::Value = serde_json::from_slice(&default_policy).unwrap();
    assert!(
        old_snapshot["machine_policies"]["local"]["telemetry"]
            .get("weights")
            .is_none(),
        "le JSON historique sans poids reste sans ce champ"
    );
    let restored: RoutingPolicy = serde_json::from_slice(&default_policy).unwrap();
    assert_eq!(
        serde_json::to_vec(&restored).unwrap(),
        default_policy,
        "ancien snapshot sans weights : roundtrip byte-identique et digest conservé"
    );
    value["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
        serde_json::to_value(&defaults).unwrap();
    let explicit =
        AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-defaults.json").unwrap();
    let explicit_policy = serde_json::to_vec(explicit.routing_policy().unwrap()).unwrap();
    assert_eq!(
        default_policy, explicit_policy,
        "poids omis et défauts explicites figent les mêmes octets de politique/digest"
    );
    let default_digest = Sha256::digest(&default_policy);

    value["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
        serde_json::json!({"cpu":7});
    let partial =
        AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-weights.json").unwrap();
    assert_eq!(
        partial.routing_policy().unwrap().machine_policies["local"]
            .telemetry
            .as_ref()
            .unwrap()
            .weights,
        RoutingScoreWeights {
            cpu: 7,
            ..defaults.clone()
        },
        "les champs d'un objet partiel gardent leurs défauts"
    );
    let changed_policy = serde_json::to_vec(partial.routing_policy().unwrap()).unwrap();
    let changed_snapshot: serde_json::Value = serde_json::from_slice(&changed_policy).unwrap();
    assert_eq!(
        changed_snapshot["machine_policies"]["local"]["telemetry"]["weights"]["cpu"], 7,
        "les poids non par défaut restent explicitement figés"
    );
    assert_ne!(
        Sha256::digest(&changed_policy),
        default_digest,
        "les poids configurés sont portés par la politique sérialisée du snapshot/digest"
    );

    for (field, cap) in [
        ("cpu", 20),
        ("ram", 20),
        ("load", 20),
        ("continuity", 20),
        ("compilation", 10),
        ("quota", 5),
        ("reset", 3),
    ] {
        for valid in [0, cap] {
            let mut configured = value.clone();
            configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
                serde_json::json!({});
            configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]
                [field] = serde_json::json!(valid);
            assert!(
                AgentRegistry::from_json(&configured.to_string(), "/tmp/routing168-weights.json")
                    .is_ok(),
                "{field}={valid}"
            );
        }
        for invalid in [cap + 1, 255] {
            let mut configured = value.clone();
            configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
                serde_json::json!({});
            configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]
                [field] = serde_json::json!(invalid);
            assert_eq!(
                AgentRegistry::from_json(&configured.to_string(), "/tmp/routing168-weights.json")
                    .unwrap_err(),
                "routing_score_weights_invalid",
                "{field}={invalid}"
            );
        }
    }
    let mut zero = value.clone();
    zero["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] = serde_json::json!({"cpu":0,"ram":0,"load":0,"continuity":0,"compilation":0,"quota":0,"reset":0});
    assert!(AgentRegistry::from_json(&zero.to_string(), "/tmp/routing168-weights.json").is_ok());
    zero["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]["saturation"] =
        serde_json::json!(0);
    assert!(
        AgentRegistry::from_json(&zero.to_string(), "/tmp/routing168-weights.json").is_err(),
        "la pénalité100 n'est jamais réglable, même quand les autres poids valent0"
    );
    for invalid in [
        serde_json::json!(-1),
        serde_json::json!(256),
        serde_json::json!(1.5),
        serde_json::json!("7"),
        serde_json::Value::Null,
    ] {
        let mut configured = value.clone();
        configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]["cpu"] =
            invalid;
        assert!(
            AgentRegistry::from_json(&configured.to_string(), "/tmp/routing168-weights.json")
                .is_err(),
            "chaque poids exige un entier borné"
        );
    }

    for field in [
        "cpu_saturation_percent",
        "normalized_load_saturation_percent",
    ] {
        for invalid in [0, 101, 255] {
            let mut value = routing168_registry_value();
            value["routing_policy"]["machine_policies"]["local"]["telemetry"] =
                serde_json::json!({"enabled":true});
            value["routing_policy"]["machine_policies"]["local"]["telemetry"][field] =
                serde_json::json!(invalid);
            assert_eq!(
                AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-invalid.json")
                    .unwrap_err(),
                "routing_telemetry_invalid"
            );
        }
    }
}

#[test]
fn routing168_capacite_compilation_est_declarative_et_bornee() {
    let mut value = routing168_registry_value();
    value["routing_policy"]["machine_policies"]["local"]["compilation_capacity"] = 10.into();
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").is_ok());
    value["routing_policy"]["machine_policies"]["local"]["compilation_capacity"] = 11.into();
    assert_eq!(
        AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").unwrap_err(),
        "routing_machine_policy_invalid"
    );
}

#[test]
fn remote162_absence_et_defaut_desactive_ne_creent_aucune_route() {
    let old = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    assert!(old.machines().is_empty());
    assert_eq!(old.machine("cartae").unwrap_err(), "machine_unknown");
    let value = remote162_registry_value();
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    assert_eq!(registry.machine("cartae").unwrap_err(), "machine_disabled");
    let machine = &registry.machines()["cartae"];
    assert!(!machine.permits_agent_type("codex"));
    assert!(
        machine.matches_credential(&IdentityCredential::new("private-worker-credential".into()))
    );
    assert!(!machine.matches_credential(&IdentityCredential::new("another-credential".into())));
    assert!(registry.project_mapping("cartae", "cartae").is_err());
}

#[test]
fn remote162_route_explicitement_activee_authentifie_sans_secret_dans_registre() {
    let mut value = remote162_registry_value();
    value["machines"]["cartae"]["enabled"] = true.into();
    value["machines"]["cartae"]["display_name"] = "Serveur Cartae".into();
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    let machine = registry.machine("cartae").unwrap();
    assert_eq!(machine.display_name.as_deref(), Some("Serveur Cartae"));
    assert!(machine.permits_agent_type("codex"));
    assert!(!machine.permits_agent_type("claude"));
    assert!(
        machine.matches_credential(&IdentityCredential::new("private-worker-credential".into()))
    );
    assert!(!machine.matches_credential(&IdentityCredential::new("another-credential".into())));
    assert!(!format!("{registry:?}").contains("private-worker-credential"));
    assert_eq!(
        registry
            .project_mapping("cartae", "cartae")
            .unwrap()
            .remote_root,
        Path::new("/home/moi/20.cartae")
    );
    assert_eq!(
        registry.project_mapping("cartae", "missing").unwrap_err(),
        "project_mapping_unknown"
    );
    // Aucune résolution de la racine Linux n'est tentée sur le maître.
}

#[test]
fn remote162_definitions_fournisseur_figees_excluent_autorite_machine() {
    let mut value = remote162_registry_value();
    value["machines"]["cartae"]["enabled"] = true.into();
    let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
    let resolved = registry.resolved_definition("codex").unwrap();
    assert!(
        AgentRegistry::from_resolved("codex", &resolved)
            .unwrap()
            .machines()
            .is_empty()
    );
    assert!(
        registry
            .for_spawn_posture(
                "codex",
                bridget_transport::protocol::SpawnPosture::Development
            )
            .unwrap()
            .machines()
            .is_empty()
    );
    assert!(registry.machine("cartae").is_ok());
}

#[test]
fn remote162_configuration_invalide_refusee_atomiquement() {
    for (field, bad) in [
        ("provisioning_revision", serde_json::json!(0)),
        (
            "provisioning_revision",
            serde_json::json!(MAX_MACHINE_REVISION + 1),
        ),
        ("credential_fingerprint", serde_json::json!("not-a-digest")),
        ("worker_id", serde_json::json!("worker\nforged")),
        ("display_name", serde_json::json!("machine\nforged")),
        ("allowed_agent_types", serde_json::json!(["codex", "codex"])),
        ("credential", serde_json::json!("must-not-be-stored")),
    ] {
        let mut value = remote162_registry_value();
        value["machines"]["cartae"][field] = bad;
        assert!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err(),
            "{field}"
        );
    }
    for root in ["relative", "/home/moi/../private", "/home/moi/\u{0}"] {
        let mut value = remote162_registry_value();
        value["machines"]["cartae"]["mappings"]["cartae"]["remote_root"] = root.into();
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
    }
    assert!(AgentRegistry::from_json("{invalid", "/tmp/agents.json").is_err());
}

#[test]
fn remote162_plafonds_et_identite_worker_unique() {
    let mut value = remote162_registry_value();
    let machine = value["machines"]["cartae"].clone();
    value["machines"]["duplicate"] = machine.clone();
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
    value["machines"] = serde_json::json!({});
    for index in 0..MAX_MACHINES {
        let mut item = machine.clone();
        item["worker_id"] = format!("worker-{index}").into();
        value["machines"][format!("machine-{index}")] = item;
    }
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_ok());
    value["machines"]["extra"] = machine;
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
    let mut value = remote162_registry_value();
    let mapping = value["machines"]["cartae"]["mappings"]["cartae"].clone();
    value["machines"]["cartae"]["mappings"] = serde_json::json!({});
    for index in 0..MAX_MACHINE_MAPPINGS {
        value["machines"]["cartae"]["mappings"][format!("project-{index}")] = mapping.clone();
    }
    assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_ok());
    value["machines"]["cartae"]["mappings"]["extra"] = mapping;
    assert!(
        AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_ok(),
        "les mappings sont bornés par les octets du registre, pas 64 projets"
    );
}

#[test]
fn remote162_registre_sur_disque_exige_fichier_et_parent_prives() {
    let root = test_root("remote162-private");
    let path = root.join("agents.json");
    std::fs::write(&path, remote162_registry_value().to_string()).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(
        AgentRegistry::load_from_path(path.clone()).unwrap_err(),
        "machine_registry_parent_not_private"
    );
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(
        AgentRegistry::load_from_path(path.clone())
            .unwrap()
            .machines()
            .len(),
        1
    );
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(AgentRegistry::load_from_path(path.clone()).is_err());
    let link = root.join("link.json");
    symlink(&path, &link).unwrap();
    assert!(AgentRegistry::load_from_path(link).is_err());
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let parent_link = root.join("parent-link");
    symlink(&root, &parent_link).unwrap();
    assert_eq!(
        AgentRegistry::load_from_path(parent_link.join("agents.json")).unwrap_err(),
        "machine_registry_parent_not_private"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn remote162_chemins_proteges_resolus_sur_hote_refusent_tout_chevauchement() {
    let root = test_root("remote162-paths");
    let private = root.join("worker");
    let project = root.join("project");
    std::fs::create_dir_all(private.join("slot")).unwrap();
    std::fs::create_dir(&project).unwrap();
    let protected = vec![private.clone()];
    for path in [&root, &private, &private.join("slot")] {
        assert_eq!(
            validate_unprotected_local_path(path, &protected).unwrap_err(),
            "machine_path_protected"
        );
    }
    let link = root.join("disguised-project");
    symlink(&private, &link).unwrap();
    assert_eq!(
        validate_unprotected_local_path(&link, &protected).unwrap_err(),
        "machine_path_protected"
    );
    assert!(validate_unprotected_local_path(&project, &protected).is_ok());
    assert!(validate_unprotected_local_path(&root.join("absent"), &protected).is_err());
    assert!(validate_unprotected_local_path(&project, &[root.join("unknown-private")]).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn defaults_cover_the_three_priorities() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let codex = registry.get("codex").unwrap();
    assert_eq!(codex.command, NATIVE_CODEX_COMMAND);
    assert!(Path::new(&codex.command).is_absolute());
    assert_eq!(
        codex.args,
        vec![
            "-c",
            "model=\"gpt-5.6-terra\"",
            "--dangerously-bypass-approvals-and-sandbox",
            "app-server",
        ]
    );
    assert_eq!(codex.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
    assert_eq!(codex.protocol, "codex_app_server");
    assert_eq!(codex.permissions, "allow");
    assert_eq!(codex.queue_capacity, 32);
    assert_eq!(codex.notify_timeout_secs, DEFAULT_NOTIFY_TIMEOUT_SECS);
    assert!(codex.pass_env.contains(&"CODEX_HOME".to_string()));
    assert!(!codex.pass_env.contains(&"OPENAI_API_KEY".to_string()));
    assert_eq!(registry.get("gemini").unwrap().args, vec!["--acp"]);
    assert_eq!(codex.mcp.interactive, "codex");
    assert!(!codex.mcp.acp_session);
    assert_eq!(codex.capabilities.execution_paths, vec!["codex_app_server"]);
    assert_eq!(
        codex.capabilities.models.get("gpt-5.6-terra"),
        Some(&ModelCapabilities::default())
    );
    let claude = registry.get("claude").unwrap();
    let expected_claude_command = native_claude_command().unwrap();
    assert_eq!(claude.command, expected_claude_command);
    assert!(Path::new(&claude.command).is_absolute());
    assert_eq!(claude.protocol, "claude_stream_json");
    assert_eq!(
        claude.args,
        [
            "--model",
            "claude-opus-5",
            "--dangerously-skip-permissions",
            "--permission-mode",
            "bypassPermissions",
        ]
    );
    assert!(!claude.mcp.acp_session);
    assert_eq!(
        claude.capabilities.execution_paths,
        vec!["claude_stream_json"]
    );
    assert_eq!(
        claude.capabilities.models.get("claude-opus-5"),
        Some(&ModelCapabilities::default())
    );
    assert_eq!(
        registry.get("gemini").unwrap().mcp.interactive,
        "unsupported"
    );
    let cursor = registry.get("cursor").unwrap();
    assert_eq!(cursor.command, "cursor-agent");
    assert_eq!(cursor.args, vec!["--model", "auto", "acp"]);
    assert_eq!(cursor.protocol, "acp");
    assert_eq!(
        cursor.capabilities.models.get("auto"),
        Some(&ModelCapabilities::default())
    );
    assert_eq!(
        runtime_model_and_effort(&cursor.args),
        Some(("auto".to_string(), None))
    );
}

#[test]
fn spec_076_coordinateurs_decouverte_sont_derives_et_restreints() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let original_codex = registry.get("codex").unwrap();
    let discovery_codex = registry.get("project-discovery-codex").unwrap();
    assert_eq!(original_codex.permissions, "allow");
    assert_eq!(discovery_codex.permissions, "deny");
    assert!(
        discovery_codex
            .args
            .windows(2)
            .any(|arguments| arguments == ["--sandbox", "read-only"]),
        "le coordinateur Codex doit être enfermé en lecture seule"
    );
    assert!(
        discovery_codex
            .args
            .iter()
            .all(|argument| argument != "--dangerously-bypass-approvals-and-sandbox"),
        "aucun bypass Codex ne doit survivre"
    );

    let discovery_claude = registry.get("project-discovery-claude").unwrap();
    assert_eq!(discovery_claude.permissions, "deny");
    assert!(
        discovery_claude
            .args
            .iter()
            .any(|argument| argument == "--restricted")
    );
    assert!(
        discovery_claude
            .args
            .iter()
            .any(|argument| argument == "plan")
    );
    assert!(
        discovery_claude
            .args
            .iter()
            .all(|argument| !argument.contains("dangerously-skip-permissions")),
        "aucun bypass Claude ne doit survivre"
    );
    assert!(
        registry.project_discovery_agent_type("cursor").is_none(),
        "Cursor n'est pas proposé sans preuve de lecture seule"
    );
}

#[test]
fn commande_claude_refuse_home_absent_au_point_de_derivation() {
    assert_eq!(
        native_claude_command_from(None).unwrap_err(),
        "HOME absent pour résoudre le CLI Claude"
    );
    assert_eq!(
        native_claude_command_from(Some(PathBuf::from("home-relatif"))).unwrap_err(),
        "HOME doit être absolu pour résoudre le CLI Claude"
    );
    assert_eq!(
        native_claude_command_from(Some(PathBuf::from("/srv/bridget-test"))).unwrap(),
        "/srv/bridget-test/.local/bin/claude"
    );
}

#[test]
fn etiquette_modele_auto_lue_depuis_les_args_cursor() {
    // Sans --model, who reste vide — c'est le trou constaté sur les
    // anciennes définitions. Avec --model auto, l'étiquette est honnête.
    let observed =
        runtime_model_and_effort(&["--model".to_string(), "auto".to_string(), "acp".to_string()]);
    assert_eq!(observed, Some(("auto".to_string(), None)));
    assert!(
        runtime_model_and_effort(&["acp".to_string()]).is_none(),
        "sans --model, who reste vide"
    );
}

#[test]
fn bypass_permissions_change_le_digest_de_la_definition_claude() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let with_bypass = registry.resolved_definition("claude").unwrap();
    let mut without = registry.get("claude").unwrap().clone();
    without.args = vec!["--model".to_string(), "claude-opus-5".to_string()];
    let old = resolved_definition(&without).unwrap();
    assert_ne!(
        with_bypass.digest, old.digest,
        "ajouter les flags bypass doit changer le digest figé"
    );
    assert!(
        with_bypass
            .args
            .iter()
            .any(|a| a == "--dangerously-skip-permissions")
    );
    assert!(with_bypass.args.iter().any(|a| a == "bypassPermissions"));
}

#[test]
fn user_entry_replaces_its_default() {
    let registry = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"custom","protocol":"tmux"}}}"#,
        "/tmp/agents.json",
    )
    .unwrap();
    assert_eq!(registry.get("codex").unwrap().command, "custom");
    assert!(registry.get("codex").unwrap().forbidden_env.is_empty());
}

#[test]
fn registre_sans_bascule_charge_les_definitions_par_defaut() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    assert!(registry.get("codex").is_ok());
}

#[test]
fn remplacement_sans_forbidden_env_signale_la_perte_de_garde() {
    let warnings = registry_warnings(
        r#"{"agents":{"codex":{"command":"custom"}}}"#,
        Path::new("/tmp/agents.json"),
        &default_agents().unwrap(),
    );
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("codex"));
    assert!(warnings[0].contains("sans déclarer forbidden_env"));

    let explicit = registry_warnings(
        r#"{"agents":{"codex":{"command":"custom","forbidden_env":[]}}}"#,
        Path::new("/tmp/agents.json"),
        &default_agents().unwrap(),
    );
    assert!(explicit.is_empty());
}

#[test]
fn definition_resolue_est_complete_et_son_digest_est_deterministe() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let first = registry.resolved_definition("codex").unwrap();
    let second = registry.resolved_definition("codex").unwrap();
    assert_eq!(first, second);
    assert_eq!(first.command, NATIVE_CODEX_COMMAND);
    assert_eq!(
        first.args,
        vec![
            "-c",
            "model=\"gpt-5.6-terra\"",
            "--dangerously-bypass-approvals-and-sandbox",
            "app-server",
        ]
    );
    assert_eq!(first.protocol, "codex_app_server");
    assert_eq!(first.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
    assert!(first.pass_env.contains(&"CODEX_HOME".to_string()));
    assert_eq!(first.permissions, "allow");
    assert_eq!(first.queue_capacity, 32);
    assert_eq!(first.notify_timeout_secs, DEFAULT_NOTIFY_TIMEOUT_SECS);
    assert_eq!(first.mcp.interactive, "codex");
    assert!(!first.mcp.acp_session);
    assert_eq!(
        first.capabilities.models.get("gpt-5.6-terra"),
        Some(&ModelCapabilities::default())
    );
    assert_eq!(first.digest.len(), 64);

    let baseline = default_agents().unwrap().remove("codex").unwrap();
    let mut mutations = Vec::new();
    let mut changed = baseline.clone();
    changed.command = "other".to_string();
    mutations.push(("command", changed));
    let mut changed = baseline.clone();
    changed.args.push("other".to_string());
    mutations.push(("args", changed));
    let mut changed = baseline.clone();
    changed.protocol = "tmux".to_string();
    mutations.push(("protocol", changed));
    let mut changed = baseline.clone();
    changed.forbidden_env.push("OTHER_KEY".to_string());
    mutations.push(("forbidden_env", changed));
    let mut changed = baseline.clone();
    changed.pass_env.push("OTHER_HOME".to_string());
    mutations.push(("pass_env", changed));
    let mut changed = baseline.clone();
    changed.permissions = "deny".to_string();
    mutations.push(("permissions", changed));
    let mut changed = baseline.clone();
    changed.queue_capacity += 1;
    mutations.push(("queue_capacity", changed));
    let mut changed = baseline.clone();
    changed.notify_timeout_secs += 1;
    mutations.push(("notify_timeout_secs", changed));
    let mut changed = baseline.clone();
    changed.mcp.interactive = "claude".to_string();
    mutations.push(("mcp.interactive", changed));
    let mut changed = baseline.clone();
    changed.mcp.acp_session = !changed.mcp.acp_session;
    mutations.push(("mcp.acp_session", changed));
    let mut changed = baseline.clone();
    changed.capabilities.models.insert(
        "gpt-5.6-terra".to_string(),
        ModelCapabilities {
            service_tiers: Vec::new(),
            efforts: vec!["high".to_string()],
        },
    );
    mutations.push(("capabilities.models", changed));
    for (field, changed) in mutations {
        assert_ne!(
            first.digest,
            resolved_definition(&changed).unwrap().digest,
            "le digest doit changer avec {field}"
        );
    }
    assert!(AgentRegistry::from_resolved("codex", &first).is_ok());
    let mut forged = first;
    forged.queue_capacity += 1;
    assert!(AgentRegistry::from_resolved("codex", &forged).is_err());
}

#[test]
fn profil_claude_est_absolu_reserve_au_transport_claude_et_fige_dans_le_digest() {
    let root = test_root("claude-config-dir");
    let profile = root.join("profile");
    std::fs::create_dir_all(&profile).unwrap();
    std::fs::set_permissions(&profile, std::fs::Permissions::from_mode(0o700)).unwrap();
    let profile = profile.to_string_lossy().into_owned();
    let registry = AgentRegistry::from_json(
        &serde_json::json!({
            "agents": {
                "glm": {
                    "command": "claude",
                    "protocol": "claude_stream_json",
                    "claude_config_dir": profile
                }
            }
        })
        .to_string(),
        "/tmp/agents.json",
    )
    .unwrap();
    let resolved = registry.resolved_definition("glm").unwrap();
    assert_eq!(
        resolved.claude_config_dir.as_deref(),
        Some(profile.as_str())
    );

    let without_profile = AgentRegistry::from_json(
        r#"{"agents":{"anthropic":{"command":"claude","protocol":"claude_stream_json"}}}"#,
        "/tmp/agents.json",
    )
    .unwrap()
    .resolved_definition("anthropic")
    .unwrap();
    assert_ne!(resolved.digest, without_profile.digest);

    let missing_profile = AgentRegistry::from_json(
        r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json"}}}"#,
        "/tmp/agents.json",
    )
    .unwrap_err();
    assert!(missing_profile.contains("claude_config_dir requis"));

    let relative = AgentRegistry::from_json(
            r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json","claude_config_dir":"relative"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
    assert!(relative.contains("claude_config_dir absolu"));

    let wrong_transport = AgentRegistry::from_json(
            r#"{"agents":{"cursor":{"command":"cursor-agent","protocol":"acp","claude_config_dir":"/tmp/profile"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
    assert!(wrong_transport.contains("claude_stream_json"));

    let inherited = AgentRegistry::from_json(
            r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json","claude_config_dir":"/tmp/profile","pass_env":["CLAUDE_CONFIG_DIR"]}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
    assert!(inherited.contains("CLAUDE_CONFIG_DIR"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn definition_figee_historique_reste_reprise_avec_la_capacite_acp_de_migration() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let definition = registry.get("codex").unwrap().clone();
    let mut legacy = resolved_definition(&definition).unwrap();
    legacy.digest = legacy_resolved_digest(&definition).unwrap();
    let mut value = serde_json::to_value(&legacy).unwrap();
    value.as_object_mut().unwrap().remove("capabilities");
    let decoded: ResolvedAgentDefinition = serde_json::from_value(value).unwrap();
    assert_eq!(decoded.capabilities, AdapterCapabilities::default());
    assert!(AgentRegistry::from_resolved("codex", &decoded).is_ok());
}

#[test]
fn definition_figee_avant_profil_conserve_ses_capacites_a_la_reprise() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let definition = registry.get("codex").unwrap().clone();
    let mut before_profile = resolved_definition(&definition).unwrap();
    before_profile.digest = pre_profile_resolved_digest(&definition).unwrap();
    let mut value = serde_json::to_value(&before_profile).unwrap();
    value.as_object_mut().unwrap().remove("claude_config_dir");
    let decoded: ResolvedAgentDefinition = serde_json::from_value(value).unwrap();

    let restored = AgentRegistry::from_resolved("codex", &decoded).unwrap();
    assert_eq!(
        restored.get("codex").unwrap().capabilities,
        definition.capabilities
    );
}

#[test]
fn lecture_privee_refuse_permissions_larges_et_accepte_0600() {
    let root = test_root("permissions");
    let path = root.join("agents.json");
    write_registry(&path, 0o644);
    let error = AgentRegistry::load_from_path(path.clone()).unwrap_err();
    assert!(error.contains("permissions 0644"));
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(AgentRegistry::load_from_path(path).is_ok());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn lecture_privee_refuse_un_lien_symbolique() {
    let root = test_root("symlink");
    let target = root.join("target.json");
    let link = root.join("agents.json");
    write_registry(&target, 0o600);
    symlink(&target, &link).unwrap();
    let error = AgentRegistry::load_from_path(link).unwrap_err();
    assert!(error.contains("liens symboliques sont interdits"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn empty_command_is_rejected() {
    let result =
        AgentRegistry::from_json(r#"{"agents":{"codex":{"command":""}}}"#, "/tmp/agents.json");
    assert!(result.unwrap_err().contains("command vide"));
}

#[test]
fn invalid_protocol_is_rejected() {
    let result = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"codex","protocol":"bad"}}}"#,
        "/tmp/agents.json",
    );
    assert!(result.unwrap_err().contains("protocol invalide"));
}

#[test]
fn invalid_mcp_interactive_is_rejected() {
    let result = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"codex","mcp":{"interactive":"global-file"}}}}"#,
        "/tmp/agents.json",
    );
    assert!(result.unwrap_err().contains("mcp.interactive invalide"));
}

#[test]
fn matrice_de_capacites_refuse_les_champs_inconnus_et_les_doublons() {
    let unknown = AgentRegistry::from_json(
        r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp"],"future":true}}}}"#,
        "/tmp/agents.json",
    );
    assert!(unknown.unwrap_err().contains("unknown field `future`"));

    let duplicate = AgentRegistry::from_json(
        r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp","acp"]}}}}"#,
        "/tmp/agents.json",
    );
    assert!(
        duplicate
            .unwrap_err()
            .contains("chemin de capacité dupliqué")
    );
}

#[test]
fn invalid_permissions_are_rejected() {
    let result = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"codex","permissions":"ask"}}}"#,
        "/tmp/agents.json",
    );
    assert!(result.unwrap_err().contains("permissions invalides"));
}

#[test]
fn zero_queue_capacity_is_rejected() {
    let result = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"codex","queue_capacity":0}}}"#,
        "/tmp/agents.json",
    );
    assert!(result.unwrap_err().contains("queue_capacity nul"));
}

#[test]
fn unknown_type_names_its_source_and_choices() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let error = registry.get("inconnu").unwrap_err();
    assert!(error.contains("/tmp/agents.json"));
    assert!(error.contains("codex"));
}

#[test]
fn unknown_entry_key_is_reported() {
    let warnings = registry_warnings(
        r#"{"agents":{"codex":{"command":"codex","forbidden_env":[],"surprise":true}}}"#,
        Path::new("/tmp/agents.json"),
        &default_agents().unwrap(),
    );
    assert_eq!(
        warnings,
        vec!["clé inconnue 'surprise' pour 'codex' dans /tmp/agents.json"]
    );
}

#[test]
fn valid_entry_keys_produce_no_warning() {
    let warnings = registry_warnings(
        r#"{"agents":{"codex":{"command":"codex","args":[],"protocol":"acp","forbidden_env":[],"pass_env":["CODEX_HOME"],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600}}}"#,
        Path::new("/tmp/agents.json"),
        &default_agents().unwrap(),
    );
    assert!(warnings.is_empty());
}

#[test]
fn pass_env_est_borne_et_valide_atomiquement() {
    for (json, detail) in [
        (
            r#"{"agents":{"x":{"command":"x","pass_env":["INVALIDE-TIRET"]}}}"#,
            "variable pass_env invalide",
        ),
        (
            r#"{"agents":{"x":{"command":"x","pass_env":["HOME","HOME"]}}}"#,
            "pass_env dupliquée",
        ),
        (
            r#"{"agents":{"x":{"command":"x","pass_env":["API_KEY"],"forbidden_env":["API_KEY"]}}}"#,
            "à la fois dans pass_env et forbidden_env",
        ),
    ] {
        let error = AgentRegistry::from_json(json, "/tmp/agents.json").unwrap_err();
        assert!(error.contains(detail), "erreur inattendue: {error}");
    }
    let entries = (0..=MAX_PASS_ENV_ENTRIES)
        .map(|index| format!("VAR_{index}"))
        .collect::<Vec<_>>();
    let json = serde_json::json!({"agents":{"x":{"command":"x","pass_env":entries}}});
    let error = AgentRegistry::from_json(&json.to_string(), "/tmp/agents.json").unwrap_err();
    assert!(error.contains("pass_env dépasse"));
}

#[test]
fn generic_codex_command_resolves_through_the_registry() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    assert_eq!(registry.type_for_command("codex").unwrap(), "codex");
}

#[test]
fn native_custom_command_resolves_without_its_internal_discovery_variant() {
    let registry = AgentRegistry::from_json(
            r#"{"agents":{"provider":{"command":"/tmp/custom-provider","protocol":"claude_stream_json","args":["--model","fixture"]}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
    // Le type restreint reste disponible pour la politique de lancement,
    // mais ne constitue pas un deuxième fournisseur déclaré par l'humain.
    assert_eq!(
        registry
            .get("project-discovery-provider")
            .unwrap()
            .permissions,
        "deny"
    );
    assert_eq!(
        registry.type_for_command("/tmp/custom-provider").unwrap(),
        "provider"
    );
    assert_eq!(
        registry.type_for_command("custom-provider").unwrap(),
        "provider"
    );
}

#[test]
fn historic_interactive_aliases_remain_available() {
    for (command, agent_type) in [
        ("codex", "codex"),
        ("claude", "claude"),
        ("gemini", "gemini"),
        ("gclaude", "claude"),
        ("claude-son", "claude"),
    ] {
        assert_eq!(AgentRegistry::interactive_alias(command), Some(agent_type));
    }
}

#[test]
fn ambiguous_command_is_refused_without_map_order_fallback() {
    let registry = AgentRegistry::from_json(
        r#"{"agents":{"one":{"command":"bridge"},"two":{"command":"bridge"}}}"#,
        "/tmp/agents.json",
    )
    .unwrap();
    let error = registry.type_for_command("bridge").unwrap_err();
    assert!(error.contains("commande ambiguë 'bridge'"));
    assert!(error.contains("one"));
    assert!(error.contains("two"));
}

#[test]
fn undeclared_generic_command_names_source_and_choices() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    let error = registry.type_for_command("not-declared").unwrap_err();
    assert!(error.contains("/tmp/agents.json"));
    assert!(error.contains(&format!("codex ({NATIVE_CODEX_COMMAND})")));
}

#[test]
fn pont_zed_codex_est_refuse_avant_lancement() {
    let registry = AgentRegistry::from_json(
            r#"{"agents":{"legacy":{"command":"npx","args":["@zed-industries/codex-acp@0.16.0","-c","model=\"gpt-5.6-terra\""],"protocol":"acp"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
    let definition = registry.get("legacy").unwrap();
    let refusal = reject_retired_zed_bridge(definition).unwrap_err();
    assert!(matches!(
        refusal,
        SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/codex-acp")
    ));
    assert!(matches!(
        validate_launch_capabilities("legacy", definition),
        Err(SpawnRefusal::EnvUnfit { .. })
    ));
}

#[test]
fn pont_zed_refuse_les_quatre_formes_de_chemin_vivantes() {
    // C1 revue G10 : basename + retrait @version — les formes qui
    // échappaient à la comparaison naïve `== "codex-acp"`.
    let forms = [
        (
            "/opt/homebrew/bin/codex-acp",
            Vec::<&str>::new(),
            "@zed-industries/codex-acp",
        ),
        (
            "/opt/bridget-fixtures/bin/claude-code-acp",
            vec![],
            "@zed-industries/claude-code-acp",
        ),
        (
            "./node_modules/.bin/codex-acp",
            vec![],
            "@zed-industries/codex-acp",
        ),
        ("npx", vec!["codex-acp@0.16.0"], "@zed-industries/codex-acp"),
    ];
    for (command, args, package) in forms {
        let args_json = serde_json::to_string(&args).unwrap();
        let json = format!(
            r#"{{"agents":{{"legacy":{{"command":"{command}","args":{args_json},"protocol":"acp"}}}}}}"#
        );
        let registry = AgentRegistry::from_json(&json, "/tmp/agents.json").unwrap();
        let definition = registry.get("legacy").unwrap();
        let refusal = reject_retired_zed_bridge(definition).unwrap_err();
        assert!(
            matches!(
                &refusal,
                SpawnRefusal::EnvUnfit { detail } if detail.contains(package)
            ),
            "forme non refusée: command={command} args={args:?} → {refusal:?}"
        );
    }
}

#[test]
fn pont_zed_claude_est_refuse_avant_lancement() {
    let registry = AgentRegistry::from_json(
        r#"{"agents":{"legacy":{"command":"claude-code-acp","protocol":"acp"}}}"#,
        "/tmp/agents.json",
    )
    .unwrap();
    let definition = registry.get("legacy").unwrap();
    let refusal = reject_retired_zed_bridge(definition).unwrap_err();
    assert!(matches!(
        refusal,
        SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/claude-code-acp")
    ));
}

#[test]
fn acp_generique_cursor_et_gemini_restent_admis() {
    let registry = AgentRegistry::from_json(
            r#"{"agents":{
                "cursor":{"command":"cursor-agent","args":["--model","auto","acp"],"protocol":"acp","capabilities":{"execution_paths":["acp"],"models":{"auto":{"efforts":[]}}}},
                "gemini":{"command":"gemini","args":["--acp"],"protocol":"acp"}
            }}"#,
            "/tmp/agents.json",
        )
        .unwrap();
    reject_retired_zed_bridge(registry.get("cursor").unwrap()).unwrap();
    reject_retired_zed_bridge(registry.get("gemini").unwrap()).unwrap();
    validate_launch_capabilities("cursor", registry.get("cursor").unwrap()).unwrap();
    // gemini sans modèle explicite : la matrice accepte le chemin seul
    validate_launch_capabilities("gemini", registry.get("gemini").unwrap()).unwrap();
}

#[test]
fn defauts_natifs_codex_et_claude_ne_passent_pas_par_zed() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
    for name in ["codex", "claude"] {
        let definition = registry.get(name).unwrap();
        reject_retired_zed_bridge(definition).unwrap();
        assert!(!definition.command.contains("npx"));
        assert!(
            !definition
                .args
                .iter()
                .any(|arg| zed_bridge_token(arg).is_some())
        );
        assert_ne!(definition.protocol, "acp");
    }
}

/// Contrôle positif Cc d'abord. Puis chaîne portant un vrai bidi.
/// Preuve que le test meurt sans la garde Cf : U+202E n'est pas `is_control`.
#[test]
fn oracle_capability_refuse_une_valeur_bidi_reelle() {
    assert!(
        !valid_capability_value("model\u{0007}x"),
        "PROMESSE — un Cc (BEL) DOIT être refusé"
    );
    assert!(
        valid_capability_value("gpt-5.6-terra"),
        "PROMESSE — une valeur légitime DOIT passer"
    );
    let bidi = "model\u{202e}ledom";
    assert!(
        !bidi.chars().any(char::is_control),
        "preuve d'aveuglement Cc : sans is_format_character cette chaîne passerait"
    );
    assert!(
        !valid_capability_value(bidi),
        "oracle — une capacité portant U+202E DOIT être refusée"
    );
    assert!(
        !valid_capability_value("claude\u{2066}opus"),
        "oracle — isolat U+2066 (Trojan Source) DOIT être refusé"
    );
}
