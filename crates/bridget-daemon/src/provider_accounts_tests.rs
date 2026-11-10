    use super::*;
    use std::collections::BTreeMap;
    use std::ffi::OsString;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("bridget-accounts-{}", uuid::Uuid::new_v4()));
            fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> PathBuf {
            self.0.join("agents.json")
        }
        fn registry(&self) -> String {
            json!({"agents":{"native-test":{
                "command":"codex","args":["app-server"],"protocol":"codex_app_server",
                "pass_env":["CODEX_HOME"],"capabilities":{"execution_paths":["codex_app_server"],
                    "models":{"m000":{"efforts":["low"]}}}}},
                "routing_policy":{"version":2,"revision":1,"enabled":true,
                    "connections":[{"connection_id":"main","machine_id":"local","engine":"codex","agent_type":"native-test","enabled":true}],
                    "model_catalog":{"main":[{"model":"m000","supported_efforts":["low"],"default_effort":"low","provenance":"declared","observed_at":null}]},
                    "machine_policies":{"local":{"enabled":true}}}}).to_string()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn spec243_verrou_bref_attendu_puis_occupation_durable_refusee() {
        use std::thread;
        use std::time::{Duration, Instant};
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        let hold = |duration: Duration| {
            let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
                .mode(0o600).open(f.0.join("project-preparation.lock")).unwrap();
            assert_eq!(unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) }, 0);
            thread::spawn(move || { thread::sleep(duration); drop(lock); })
        };
        // Détention brève (préparation courte, fork→exec voisin) : attendue.
        let brief = hold(Duration::from_millis(300));
        let started = Instant::now();
        apply(&f.path(), &digest(raw.as_bytes()), "prepare", "work", Some("main"), "local").unwrap();
        assert!(started.elapsed() >= Duration::from_millis(250));
        brief.join().unwrap();
        // Occupation durable : refus explicite après la borne, registre inchangé.
        let current = crate::registry::read_private_registry(&f.path()).unwrap();
        let durable = hold(Duration::from_secs(7));
        assert_eq!(
            apply(&f.path(), &digest(current.as_bytes()), "remove", "work", None, "local"),
            Err("account_registry_busy")
        );
        assert_eq!(crate::registry::read_private_registry(&f.path()).unwrap(), current);
        durable.join().unwrap();
    }

    #[test]
    fn spec221_c06_prepare_pin_and_remove_keep_main_login_and_saved_profile() {
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        let main = f.0.join("principal");
        private_dir(&main).unwrap();
        let auth = main.join("auth.json");
        write_new(&auth, b"MAIN_SYNTHETIC").unwrap();
        let preview =
            prepare_change(&raw, &f.path(), "local", "prepare", "work", Some("main")).unwrap();
        assert!(!f.0.join("provider-accounts").exists());
        assert_eq!(preview.view["auth"], "not_verified");
        assert_eq!(
            preview.next["routing_policy"]["connections"][0],
            serde_json::from_str::<Value>(&raw).unwrap()["routing_policy"]["connections"][0]
        );
        assert!(
            preview.view["login"]["clear_environment"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "OPENAI_API_KEY")
        );
        let done = apply(
            &f.path(),
            &digest(raw.as_bytes()),
            "prepare",
            "work",
            Some("main"),
            "local",
        )
        .unwrap();
        assert_eq!(done["status"], "applied_local");
        let content = crate::registry::read_private_registry(&f.path()).unwrap();
        let registry = AgentRegistry::from_json(&content, f.path()).unwrap();
        let child = registry.get("account-work").unwrap();
        let profile = f.0.join("provider-accounts/work/codex");
        assert_eq!(child.codex_home.as_deref(), profile.to_str());
        assert!(!child.pass_env.iter().any(|v| v == "CODEX_HOME"));
        let source = BTreeMap::from([
            ("HOME".into(), OsString::from(&f.0)),
            ("PATH".into(), OsString::from("/usr/bin:/bin")),
            ("CODEX_HOME".into(), main.as_os_str().to_owned()),
        ]);
        let before = source.clone();
        let env = crate::lifecycle::build_environment(child, &source).unwrap();
        assert_eq!(env["CODEX_HOME"], profile.as_os_str());
        assert_eq!(env["HOME"], source["HOME"]);
        assert_eq!(source, before);
        assert_eq!(fs::read(&auth).unwrap(), b"MAIN_SYNTHETIC");
        assert!(!profile.join("auth.json").exists());
        let enabled = apply(
            &f.path(),
            &digest(content.as_bytes()),
            "enable",
            "work",
            None,
            "local",
        )
        .unwrap();
        assert_eq!(enabled["auth"], "not_verified");
        let active = crate::registry::read_private_registry(&f.path()).unwrap();
        apply(
            &f.path(),
            &digest(active.as_bytes()),
            "remove",
            "work",
            None,
            "local",
        )
        .unwrap();
        let removed = crate::registry::read_private_registry(&f.path()).unwrap();
        let parsed = AgentRegistry::from_json(&removed, f.path()).unwrap();
        assert!(
            !parsed
                .routing_policy()
                .unwrap()
                .connections
                .iter()
                .any(|c| c.connection_id == "account-work")
        );
        assert!(
            !parsed
                .routing_policy()
                .unwrap()
                .model_catalog
                .contains_key("account-work")
        );
        assert!(profile.is_dir());
        assert!(parsed.get("account-work").is_err());
        // Forme exacte du lecteur antérieur : matrice fermée à trois champs.
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[expect(dead_code, reason = "schéma hérité désérialisé pour vérifier le refus des champs inconnus ; ses champs ne sont pas lus")]
        struct LegacyCapabilities {
            #[serde(default)] execution_paths:Vec<String>,
            #[serde(default)] models:serde_json::Value,
            #[serde(default)] observed:Option<serde_json::Value>,
        }
        let guarded=serde_json::to_value(&child.capabilities).unwrap();
        assert!(guarded["account_profiles"]==true);
        assert!(serde_json::from_value::<LegacyCapabilities>(guarded).is_err());
        assert!(serde_json::from_value::<LegacyCapabilities>(json!({"execution_paths":["acp"]})).is_ok());

        assert!(
            AgentRegistry::from_resolved(
                "account-work",
                &registry.resolved_definition("account-work").unwrap()
            )
            .is_ok()
        );
        assert_eq!(fs::read(&auth).unwrap(), b"MAIN_SYNTHETIC");
    }

    #[test]
    fn spec221_c06_claude_native_profile_isolates_both_stores_without_settings_override() {
        use std::os::unix::fs::PermissionsExt;
        let f=Fixture::new();
        let program=f.0.join("claude");
        // Même fixture binaire que native149_capture : un script shell est
        // volontairement opaque et ne doit pas servir de faux témoin natif.
        fs::copy("/bin/echo",&program).unwrap();
        fs::set_permissions(&program,fs::Permissions::from_mode(0o700)).unwrap();
        let program_before=fs::read(&program).unwrap();
        let mut value:Value=serde_json::from_str(&f.registry()).unwrap();
        value["agents"]["native-test"]["command"]=json!(program);
        value["agents"]["native-test"]["args"]=json!([]);
        value["agents"]["native-test"]["protocol"]=json!("claude_stream_json");
        value["agents"]["native-test"]["capabilities"]["execution_paths"]=json!(["claude_stream_json"]);
        value["routing_policy"]["connections"][0]["engine"]=json!("claude");
        let raw=value.to_string();
        write_new(&f.path(),raw.as_bytes()).unwrap();
        let done=apply(&f.path(),&digest(raw.as_bytes()),"prepare","work",Some("main"),"local").unwrap();
        assert_eq!(done["login"]["args"],json!(["auth","login"]));
        let profile=f.0.join("provider-accounts/work/claude");
        assert_eq!(done["login"]["environment"]["ANTHROPIC_CONFIG_DIR"],json!(profile.join("anthropic")));
        let content=crate::registry::read_private_registry(&f.path()).unwrap();
        let registry=AgentRegistry::from_json(&content,f.path()).unwrap();
        let definition=registry.get("account-work").unwrap();
        assert!(definition.args.is_empty());
        let source=BTreeMap::from([("HOME".into(),f.0.as_os_str().to_owned()),
            ("PATH".into(),OsString::from("/usr/bin:/bin")),
            ("ANTHROPIC_CONFIG_DIR".into(),OsString::from("/principal"))]);
        let env=crate::lifecycle::build_environment(definition,&source).unwrap();
        assert_eq!(env["CLAUDE_CONFIG_DIR"],profile.as_os_str());
        assert_eq!(env["ANTHROPIC_CONFIG_DIR"],profile.join("anthropic").as_os_str());
        let context=crate::native_permissions::capture_context_with_env(definition,&f.0,&env).unwrap();
        assert!(context.get("settings_overrides").is_none());
        let mut args=definition.args.clone();
        crate::native_permissions::apply_child_arguments("claude_stream_json",
            &json!({"kind":"claude","permission_mode":"bypassPermissions",
                "tools":{"type":"preset","preset":"claude_code"}}),&mut args).unwrap();
        assert!(!args.iter().any(|a|a=="--settings"));
        assert_eq!(fs::read(&program).unwrap(),program_before);
    }

    #[test]
    fn spec221_c06_stale_preview_unknown_directory_and_secret_template_refuse_before_write() {
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        assert_eq!(
            apply(
                &f.path(),
                &"0".repeat(64),
                "prepare",
                "work",
                Some("main"),
                "local"
            )
            .unwrap_err(),
            "account_registry_changed"
        );
        assert_eq!(fs::read_to_string(f.path()).unwrap(), raw);
        assert!(!f.0.join("provider-accounts").exists());
        let mut value: Value = serde_json::from_str(&raw).unwrap();
        value["agents"]["native-test"]["args"] = json!([
            "-c",
            r#"model_providers.private.api_key="SYNTHETIC""#,
            "app-server"
        ]);
        assert_eq!(
            prepare_change(
                &value.to_string(),
                &f.path(),
                "local",
                "prepare",
                "work",
                Some("main")
            )
            .err(),
            Some("account_template_needs_review")
        );
        value["agents"]["native-test"]["args"] = json!(["app-server"]);
        value["routing_policy"]["connections"][0]["machine_id"] = json!("other");
        value["routing_policy"]["connections"][0]["enabled"] = json!(false);
        assert_eq!(
            prepare_change(
                &value.to_string(),
                &f.path(),
                "local",
                "prepare",
                "work",
                Some("main")
            )
            .err(),
            Some("account_remote_prepare_required")
        );
        assert_eq!(
            prepare_change(
                &raw,
                &f.path(),
                "local",
                "prepare",
                "../wrong",
                Some("main")
            )
            .err(),
            Some("account_id_invalid")
        );
        private_dir(&f.0.join("provider-accounts")).unwrap();
        private_dir(&f.0.join("provider-accounts/work")).unwrap();
        assert_eq!(
            prepare_change(&raw, &f.path(), "local", "prepare", "work", Some("main")).err(),
            Some("account_directory_unregistered")
        );
        assert_eq!(fs::read_to_string(f.path()).unwrap(), raw);
    }

    #[test]
    fn spec221_c06_profile_digest_and_missing_directory_cannot_fall_back_to_main() {
        let f = Fixture::new();
        let profile = f.0.join("isolated");
        private_dir(&profile).unwrap();
        let value = json!({"agents":{"private-codex":{"command":"codex","protocol":"codex_app_server","codex_home":profile,"capabilities":{"account_profiles":true}}}});
        let registry = AgentRegistry::from_json(&value.to_string(), f.path()).unwrap();
        let resolved = registry.resolved_definition("private-codex").unwrap();
        let loaded = AgentRegistry::from_resolved("private-codex", &resolved).unwrap();
        assert_eq!(
            loaded.get("private-codex").unwrap().codex_home,
            resolved.codex_home
        );
        let mut changed = resolved.clone();
        changed.codex_home = None;
        assert!(AgentRegistry::from_resolved("private-codex", &changed).is_err());
        let source = BTreeMap::from([("HOME".into(), OsString::from(&f.0))]);
        fs::remove_dir(&profile).unwrap();
        assert!(
            crate::lifecycle::build_environment(registry.get("private-codex").unwrap(), &source)
                .is_err()
        );
        let mut inherited = value.clone();
        inherited["agents"]["private-codex"]["pass_env"] = json!(["CODEX_HOME"]);
        assert!(AgentRegistry::from_json(&inherited.to_string(), f.path()).is_err());
        let plain = AgentRegistry::from_json("{}", f.path())
            .unwrap()
            .resolved_definition("codex")
            .unwrap();
        assert!(
            serde_json::to_value(plain)
                .unwrap()
                .get("codex_home")
                .is_none()
        );
    }
