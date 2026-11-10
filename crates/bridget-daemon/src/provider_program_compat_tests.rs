    use super::*;
    fn fixture() -> (PathBuf, SourceEnvironment) {
        let p = std::env::temp_dir().join(format!("bpc-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir(&p).unwrap();
        fs::create_dir(p.join(".cache")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o700)).unwrap();
        let source = SourceEnvironment::from([("HOME".into(), p.as_os_str().into())]);
        (p, source)
    }
    fn baseline(program: &Path) -> crate::registry::AgentDefinition {
        let r=crate::registry::AgentRegistry::from_json(&json!({"agents":{"fixture":{"command":program,"protocol":"codex_app_server","args":["app-server"],"capabilities":{"execution_paths":["codex_app_server"],"observed":{"binary_path":program,"binary_version":"old","binary_digest":"0".repeat(64),"contract_version":"historical","operations":["steer","interrupt"]}}}}}).to_string(),"/tmp/compat-registry.json").unwrap();
        r.get("fixture").unwrap().clone()
    }
    fn checked(program: &Path) -> CheckedContract {
        CheckedContract {
            version: 1,
            contract: contract_id(),
            program: program.to_string_lossy().into(),
            digest: program_digest(program).unwrap(),
            binary_version: "not_reported".into(),
            observed_at: 0,
            compatible: true,
            verified: true,
            operations: vec![ProviderOperation::Steer, ProviderOperation::Resume],
        }
    }
    fn hello() -> Value {
        json!({"userAgent":"codex_cli_rs/0.166.0 (fixture)","codexHome":"private","platformFamily":"unix","platformOs":"linux"})
    }
    #[test]
    fn spec221_program_compat_changed_image_requalified_without_profile_mutation() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program-a").unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).unwrap();
        let def = baseline(&program);
        let original = def.clone();
        assert_eq!(
            validate_upgrade(&def.command, &source, &program_digest(&program).unwrap()),
            Err("program_upgrade_check_pending")
        );
        checked(&program).publish(&source, &hello()).unwrap();
        crate::lifecycle::launch_preflight(
            &crate::registry::AgentRegistry::from_json(
                &json!({"agents":{"fixture":def}}).to_string(),
                "/tmp/compat-registry.json",
            )
            .unwrap(),
            "fixture",
            &source,
        )
        .unwrap();
        let runtime = runtime_observation(&original, &source).unwrap().unwrap();
        assert_eq!(runtime.binary_version, "0.166.0");
        assert_eq!(runtime.operations, vec![ProviderOperation::Steer]);
        assert_eq!(
            original
                .capabilities
                .observed
                .as_ref()
                .unwrap()
                .binary_digest,
            "0".repeat(64)
        );
        fs::write(&program, b"program-b").unwrap();
        assert_eq!(
            runtime_observation(&original, &source).unwrap_err(),
            "program_upgrade_check_pending"
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec244_echec_durable_de_qualification_devient_un_refus_nomme() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program-244").unwrap();
        let name = program.to_str().unwrap();
        let digest = program_digest(&program).unwrap();
        // Échec récent : encore en attente, mais cause et extrait gardés.
        record_check_failure(&source, name, &digest, "program_unavailable",
            b"bridget socket trop longue\n\x1b[31m");
        assert_eq!(validate_upgrade(name, &source, &digest), Err("program_upgrade_check_pending"));
        let fiche = read_failure(&source, name).unwrap();
        assert_eq!(fiche["cause"], "program_unavailable");
        assert_eq!(fiche["stderr"], "bridget socket trop longue [31m");
        // Échec durable : refus nommé, plus jamais « en attente » sans fin.
        let path = failure_path(&source, name).unwrap();
        let mut old = fiche.clone();
        old["first_failed_at"] = json!(now_secs() - CHECK_FAILURE_GRACE_SECS - 1);
        fs::write(&path, old.to_string()).unwrap();
        assert_eq!(validate_upgrade(name, &source, &digest), Err("program_check_unavailable"));
        // Un nouvel échec garde la première date ; une autre image repart de zéro.
        record_check_failure(&source, name, &digest, "timed_out", b"");
        assert_eq!(read_failure(&source, name).unwrap()["first_failed_at"], old["first_failed_at"]);
        assert_eq!(validate_upgrade(name, &source, "autre"), Err("program_upgrade_check_pending"));
        // Une qualification réussie efface la fiche.
        clear_check_failure(&source, name);
        assert!(read_failure(&source, name).is_none());
    }

    #[test]
    fn spec221_program_compat_current_image_keeps_historical_operations() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let mut def = baseline(&program);
        def.capabilities.observed.as_mut().unwrap().binary_digest =
            program_digest(&program).unwrap();
        assert_eq!(
            runtime_observation(&def, &source).unwrap(),
            def.capabilities.observed
        );
        assert!(CheckedContract::needed(&def, &source).unwrap().is_none());
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_private_and_current_proof_only() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let digest = program_digest(&program).unwrap();
        checked(&program).publish(&source, &hello()).unwrap();
        let p = proof_path(&source, program.to_str().unwrap()).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        let saved = p.with_extension("saved");
        fs::rename(&p, &saved).unwrap();
        std::os::unix::fs::symlink(&saved, &p).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_unknown_schema_is_not_positive() {
        assert!(accepts(
            &json!({}),
            &json!({"type":"object","required":["x"],"properties":{"x":{"type":"string","enum":["ok"]}},"additionalProperties":false}),
            &json!({"x":"ok"}),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"object","required":["new_required"]}),
            &json!({"x":"ok"}),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"string","pattern":".*"}),
            &json!("value"),
            0
        ));
        assert!(!accepts(
            &json!({}),
            &json!({"type":"object","additionalProperties":{"type":"integer"}}),
            &json!({"x":"text"}),
            0
        ));
    }
    #[test]
    fn spec221_program_compat_incompatible_is_cached_without_rechecking() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let mut c = checked(&program);
        c.compatible = false;
        c.publish(&source, &hello()).unwrap();
        let def = baseline(&program);
        assert!(matches!(
            CheckedContract::needed(&def, &source),
            Err("program_protocol_incompatible")
        ));
        assert_eq!(
            runtime_observation(&def, &source).unwrap_err(),
            "program_protocol_incompatible"
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_capture_is_scoped_and_not_shared() {
        let ((), cause) = capture(|| record_failure("program_upgrade_check_pending"));
        assert_eq!(cause, Some(UpgradeRefusal::Pending));
        assert_eq!(capture(|| ()).1, None);
        capture(|| {
            record_failure("program_protocol_incompatible");
            assert_eq!(
                std::thread::spawn(|| capture(|| ()).1).join().unwrap(),
                None
            );
            assert_eq!(
                capture(|| record_failure("unknown")).1,
                Some(UpgradeRefusal::Unavailable)
            );
            assert_eq!(FAILURE.get(), Some(UpgradeRefusal::Incompatible));
        });
        assert_eq!(FAILURE.get(), None);
    }
    #[test]
    fn spec221_program_compat_unknown_dialect_is_unavailable_not_incompatible() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let schema = home.join("unknown.json");
        fs::write(&schema, json!({"type":"string","pattern":".*"}).to_string()).unwrap();
        assert_eq!(
            schema_accepts_file(&schema, &json!("text")),
            Err("program_schema_unverified")
        );
        let mut value = checked(&program);
        value.compatible = false;
        value.verified = false;
        value.publish(&source, &hello()).unwrap();
        assert_eq!(
            validate_upgrade(
                program.to_str().unwrap(),
                &source,
                &program_digest(&program).unwrap()
            ),
            Err("program_check_unavailable")
        );
        assert!(
            CheckedContract::needed(&baseline(&program), &source)
                .unwrap()
                .is_some()
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_cache_parent_and_atomic_failure_leave_no_temporary() {
        let (home, source) = fixture();
        fs::remove_dir(home.join(".cache")).unwrap();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        checked(&program).publish(&source, &hello()).unwrap();
        let path = proof_path(&source, program.to_str().unwrap()).unwrap();
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert_eq!(
            checked(&program).publish(&source, &hello()),
            Err("program_check_unavailable")
        );
        assert!(
            fs::read_dir(root(&source).unwrap()).unwrap().all(|p| !p
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_checker_revision_invalidates_proof() {
        let mut hash = Sha256::new();
        for (name, bytes) in [
            (
                "crates/bridget-daemon/src/provider_program_compat.rs",
                include_bytes!("provider_program_compat.rs").as_slice(),
            ),
            (
                "crates/bridget-daemon/src/provider_catalogue220.rs",
                include_bytes!("provider_catalogue220.rs").as_slice(),
            ),
            (
                "crates/bridget-transport/src/codex_app_server.rs",
                include_bytes!("../../bridget-transport/src/codex_app_server.rs").as_slice(),
            ),
        ] {
            hash.update(name.as_bytes());
            hash.update(bytes);
        }
        hash.update(env!("CARGO_PKG_VERSION").as_bytes());
        assert_eq!(
            format!("{:x}", hash.finalize()),
            env!("BRIDGET_PROGRAM_CONTRACT_DIGEST")
        );
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let value = checked(&program);
        value.publish(&source, &hello()).unwrap();
        let path = proof_path(&source, program.to_str().unwrap()).unwrap();
        let mut previous: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        previous["contract"] = json!("old-checker");
        fs::write(&path, previous.to_string()).unwrap();
        assert_eq!(
            validate_upgrade(
                program.to_str().unwrap(),
                &source,
                &program_digest(&program).unwrap()
            ),
            Err("program_upgrade_check_pending")
        );
        fs::remove_dir_all(home).unwrap();
    }
    #[test]
    fn spec221_program_compat_optional_unknown_schema_and_unreadable_parent_are_unavailable() {
        let (home, source) = fixture();
        let program = home.join("codex");
        fs::write(&program, b"program").unwrap();
        let schemas = home.join("schemas");
        fs::create_dir_all(schemas.join("v1")).unwrap();
        fs::create_dir_all(schemas.join("v2")).unwrap();
        for name in [
            "ClientRequest.json",
            "v1/InitializeParams.json",
            "v2/ThreadStartParams.json",
            "v2/TurnStartParams.json",
            "v2/TurnInterruptParams.json",
            "v2/TurnSteerParams.json",
            "v2/ThreadResumeParams.json",
            "v2/ThreadForkParams.json",
        ] {
            fs::write(schemas.join(name), "{}").unwrap();
        }
        fs::write(
            schemas.join("v2/TurnSteerParams.json"),
            r#"{"unsupportedKeyword":true}"#,
        )
        .unwrap();
        let digest = program_digest(&program).unwrap();
        let value =
            CheckedContract::from_schemas(&schemas, program.to_str().unwrap(), &digest).unwrap();
        assert!(!value.verified);
        assert!(!value.is_compatible());
        value.publish(&source, &hello()).unwrap();
        assert_eq!(
            validate_upgrade(program.to_str().unwrap(), &source, &digest),
            Err("program_check_unavailable")
        );
        // Real filesystem refusal; tests normally run as the unprivileged worker user.
        if unsafe { libc::geteuid() } != 0 {
            let parent = home.join(".cache");
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o000)).unwrap();
            let result = validate_upgrade(program.to_str().unwrap(), &source, &digest);
            fs::set_permissions(&parent, fs::Permissions::from_mode(0o700)).unwrap();
            assert_eq!(result, Err("program_check_unavailable"));
        }
        fs::remove_dir_all(home).unwrap();
    }
