use super::*;
use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, symlink};

#[test]
fn spec174_namespace_errors_are_distinct_and_isolated() {
    if let Ok(case) = std::env::var("IDENTITY_TEST_CASE") {
        let native = resolve_current_identity().unwrap_err();
        let mcp = resolve_current_mcp_identity().unwrap_err();
        if case == "missing-identity" {
            assert_eq!(native, IdentityError::IdentityNotFound);
            assert_eq!(mcp, IdentityError::IdentityNotFound);
        } else {
            assert_eq!(native.code(), "namespace_configuration_invalid");
            if case == "invalid-proof" {
                assert_eq!(mcp.code(), "t3_session_unavailable");
            } else {
                assert_eq!(mcp.code(), "namespace_configuration_invalid");
                assert!(!mcp.remediation().contains("private-test-directory"));
            }
        }
        return;
    }
    let root = root("namespace174");
    for case in [
        "invalid-namespace",
        "missing-identity",
        "invalid-proof",
        "removed-runtime",
        "missing-home",
    ] {
        let home = if case == "missing-identity" || case == "removed-runtime" {
            root.clone()
        } else {
            PathBuf::from("private-test-directory")
        };
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "mcp_identity::tests::spec174_namespace_errors_are_distinct_and_isolated",
            ])
            .env_clear()
            .env("HOME", &root)
            .env(
                "PATH",
                std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
            )
            .env("BRIDGET_HOME", &home)
            .env("BRIDGET_SOCKET", home.join("bridget.sock"))
            .env("IDENTITY_TEST_CASE", case);
        if case == "invalid-proof" {
            command
                .env("BRIDGET_T3_MCP_ENDPOINT", "https://invalid.example")
                .env("BRIDGET_T3_MCP_AUTHORIZATION", "invalid-test-proof");
        } else if case == "removed-runtime" {
            command.env("BRIDGET_RUNTIME_TEST", "removed");
        } else if case == "missing-home" {
            command
                .env_remove("HOME")
                .env_remove("BRIDGET_HOME")
                .env_remove("BRIDGET_SOCKET");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{case}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(root).unwrap();
}

const AGENT_A: &str = "96389249-07a4-4e29-83f0-9c46bd775021";
const AGENT_B: &str = "da78fd70-41e8-424c-a88d-e29e2c5babcd";

#[test]
fn session_proof_preserves_native_child_limits_and_rejects_conflicting_identity() {
    let identity = |name: &str| ResolvedIdentity {
        name: name.into(),
        instance_id: format!("instance:{name}"),
        delegated_origin: None,
    };
    let session = identity(AGENT_A);
    let mut child = session.clone();
    child.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "child:1".into(),
    });
    assert_eq!(
        combine_session_identity(Some(session.clone()), Ok(child.clone()), true).unwrap(),
        child
    );
    assert_eq!(
        combine_session_identity(Some(session.clone()), Ok(identity(AGENT_B)), false),
        Err(IdentityError::T3SessionUnavailable)
    );
    assert_eq!(
        combine_session_identity(
            Some(session.clone()),
            Err(IdentityError::IdentityNotFound),
            false
        )
        .unwrap(),
        session.clone()
    );
    assert_eq!(
        combine_session_identity(Some(session), Err(IdentityError::IdentityNotFound), true),
        Err(IdentityError::IdentityNotFound)
    );
    assert_eq!(
        combine_session_identity(None, Err(IdentityError::LegacyMarker), false),
        Err(IdentityError::LegacyMarker)
    );
}

#[test]
fn spec099_preuve_privee_atomique_bornee_sans_symlink() {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("bg-proof-{}", uuid::Uuid::new_v4().simple()));
    let credential = IdentityCredential::new("secret-fixture".into());
    save_credential(&root, AGENT_A, "instance-1", credential.clone()).unwrap();
    let path = credential_path(&root, "instance-1");
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        load_credential(&root, AGENT_A, "instance-1").unwrap(),
        credential
    );
    assert!(load_credential(&root, AGENT_B, "instance-1").is_err());
    assert!(!format!("{credential:?}").contains("secret-fixture"));
    fs::remove_file(&path).unwrap();
    let target = root.join("target");
    fs::write(&target, "ne pas modifier").unwrap();
    symlink(&target, &path).unwrap();
    assert!(load_credential(&root, AGENT_A, "instance-1").is_err());
    assert!(save_credential(&root, AGENT_A, "instance-1", credential.clone()).is_err());
    assert_eq!(fs::read_to_string(&target).unwrap(), "ne pas modifier");
    fs::remove_file(&path).unwrap();
    fs::write(&path, vec![b'x'; MAX_MARKER_BYTES as usize + 1]).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(load_credential(&root, AGENT_A, "instance-1").is_err());
    fs::remove_dir_all(root).unwrap();
}

#[derive(Default)]
struct Fixture(BTreeMap<u32, (u64, u32)>);
impl ProcessTree for Fixture {
    fn birth(&self, pid: u32) -> Option<u64> {
        self.0.get(&pid).map(|value| value.0)
    }
    fn parent(&self, pid: u32) -> Option<u32> {
        self.0.get(&pid).map(|value| value.1)
    }
}
fn root(label: &str) -> PathBuf {
    // Le scan refuse à juste titre /tmp quand le chemin réel est /private/tmp.
    let root = fs::canonicalize(std::env::temp_dir())
        .unwrap()
        // Nom court : la socket doit rester sous 104 octets (SUN_LEN de macOS).
        .join(format!(
            "bmi-{label}-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
    private_dir(&root);
    root
}
fn private_dir(path: &Path) {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .unwrap();
}
fn private_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes.as_ref()).unwrap();
}
fn marker(root: &Path, pid: u32, birth: u64, instance: &str, name: &str) -> PathBuf {
    let names = root.join("names");
    private_dir(&names);
    let path = names.join(format!("{pid}.txt"));
    private_write(&path, name);
    private_dir(&root.join("agent-pids"));
    // Précréer le fichier privé, sans changer le umask global des tests.
    private_write(root.join("agent-pids").join(pid.to_string()), []);
    write_marker(&root.join("agent-pids"), pid, birth, instance, &path).unwrap();
    path
}

#[test]
fn relit_le_fichier_identite_et_les_ancetres_valides() {
    let root = root("rename");
    let name = marker(&root, 12, 120, "instance-1", AGENT_A);
    let tree = Fixture(BTreeMap::from([
        (42, (420, 30)),
        (30, (300, 12)),
        (12, (120, 1)),
    ]));
    assert_eq!(
        resolve_with(
            Some(&name),
            &root.join("agent-pids"),
            Some("instance-1"),
            42,
            &tree
        ),
        Ok(AGENT_A.into())
    );
    // Changement de fixture d'identité : prouve la relecture, PAS un
    // renommage métier (le nom affiché peut changer sans changer l'UUID).
    private_write(&name, AGENT_B);
    assert_eq!(
        resolve_with(
            None,
            &root.join("agent-pids"),
            Some("instance-1"),
            42,
            &tree
        ),
        Ok(AGENT_B.into())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn premier_agent_du_meme_binaire_gagne_dans_la_filiation() {
    let root = root("chain");
    let markers = root.join("agent-pids");
    private_dir(&markers);
    let _ = marker(&root, 10, 100, "instance-a", AGENT_A);
    let _ = marker(&root, 20, 200, "instance-b", AGENT_B);
    let chain = Fixture(BTreeMap::from([
        (50, (500, 40)),
        (40, (400, 30)),
        (30, (300, 20)),
        (20, (200, 10)),
        (10, (100, 1)),
    ]));
    assert_eq!(
        resolve_with(None, &markers, Some("instance-b"), 50, &chain),
        Ok(AGENT_B.into())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn traverse_une_chaine_npx_de_trois_processus() {
    let root = root("npx-chain");
    let markers = root.join("agent-pids");
    let _ = marker(&root, 20, 200, "instance-1", AGENT_A);
    let chain = Fixture(BTreeMap::from([
        (70, (700, 60)),
        (60, (600, 50)),
        (50, (500, 40)),
        (40, (400, 20)),
        (20, (200, 1)),
    ]));
    assert_eq!(
        resolve_with(None, &markers, Some("instance-1"), 70, &chain),
        Ok(AGENT_A.into())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn resout_la_filiation_sans_variables_applicatives() {
    let root = root("sans-env");
    let markers = root.join("agent-pids");
    let _ = marker(&root, 20, 200, "instance-marquee", AGENT_A);
    let chain = Fixture(BTreeMap::from([(42, (420, 20)), (20, (200, 1))]));

    assert_eq!(
        resolve_identity_with(None, &markers, None, 42, &chain),
        Ok(ResolvedIdentity {
            name: AGENT_A.into(),
            instance_id: "instance-marquee".into(),
            delegated_origin: None,
        })
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec133_mcp_resout_la_preuve_enfant_sans_promouvoir_la_cli() {
    let root = root("delegated");
    let parent_name = marker(&root, 10, 100, "instance-parent", AGENT_A);
    private_dir(&root.join("delegated-pids"));
    let delegated = DelegatedPidMarker {
        pid: 20,
        birth: 200,
        parent_instance_id: "instance-parent".into(),
        parent_name_file: parent_name,
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
        owner: "30-300-11111111-1111-4111-8111-111111111111".into(),
    };
    private_write(
        root.join("delegated-pids/20"),
        serde_json::to_vec(&delegated).unwrap(),
    );
    let tree = Fixture(BTreeMap::from([
        (42, (420, 20)),
        (20, (200, 10)),
        (10, (100, 1)),
    ]));

    let resolved = resolve_identity_with_delegation(
        None,
        &root.join("agent-pids"),
        &root.join("delegated-pids"),
        None,
        42,
        &tree,
        true,
    )
    .unwrap();
    assert_eq!(resolved.name, AGENT_A);
    assert_eq!(resolved.instance_id, "instance-parent");
    assert_eq!(
        resolved.delegated_origin,
        Some(bridget_core::DelegatedOrigin {
            provider: "codex".into(),
            child_ref: "0123456789abcdef".into(),
        })
    );
    assert_eq!(
        resolve_identity_with_delegation(
            None,
            &root.join("agent-pids"),
            &root.join("delegated-pids"),
            None,
            42,
            &tree,
            false,
        ),
        Err(IdentityError::DelegatedMcpOnly)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec133_preuve_enfant_invalide_interdit_le_repli_principal() {
    let root = root("delegated-invalid");
    let parent_name = marker(&root, 10, 100, "instance-parent", AGENT_A);
    private_dir(&root.join("delegated-pids"));
    let delegated = DelegatedPidMarker {
        pid: 20,
        birth: 199,
        parent_instance_id: "instance-parent".into(),
        parent_name_file: parent_name,
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
        owner: "30-300-11111111-1111-4111-8111-111111111111".into(),
    };
    private_write(
        root.join("delegated-pids/20"),
        serde_json::to_vec(&delegated).unwrap(),
    );
    let tree = Fixture(BTreeMap::from([
        (42, (420, 20)),
        (20, (200, 10)),
        (10, (100, 1)),
    ]));
    assert!(has_delegated_marker(
        &root.join("delegated-pids"),
        42,
        &tree
    ));
    assert!(!has_delegated_marker(&root.join("missing"), 42, &tree));

    for allow_delegated in [false, true] {
        assert_eq!(
            resolve_identity_with_delegation_scoped(
                None,
                &root.join("agent-pids"),
                &root.join("delegated-pids"),
                None,
                42,
                &tree,
                Some(&root),
                allow_delegated,
            ),
            Err(IdentityError::IdentityNotFound)
        );
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec133_preuve_enfant_symlink_ou_trop_grande_est_refusee_sans_repli() {
    use std::os::unix::fs::symlink;

    let root = root("delegated-bounded");
    let _ = marker(&root, 10, 100, "instance-parent", AGENT_A);
    private_dir(&root.join("delegated-pids"));
    let tree = Fixture(BTreeMap::from([
        (42, (420, 20)),
        (20, (200, 10)),
        (10, (100, 1)),
    ]));
    let outside = root.join("outside");
    private_write(&outside, b"{}");
    symlink(&outside, root.join("delegated-pids/20")).unwrap();
    assert_eq!(
        resolve_identity_with_delegation_scoped(
            None,
            &root.join("agent-pids"),
            &root.join("delegated-pids"),
            None,
            42,
            &tree,
            Some(&root),
            true,
        ),
        Err(IdentityError::IdentityNotFound)
    );
    fs::remove_file(root.join("delegated-pids/20")).unwrap();
    private_write(
        root.join("delegated-pids/20"),
        vec![b'x'; MAX_MARKER_BYTES as usize + 1],
    );
    assert_eq!(
        resolve_identity_with_delegation_scoped(
            None,
            &root.join("agent-pids"),
            &root.join("delegated-pids"),
            None,
            42,
            &tree,
            Some(&root),
            true,
        ),
        Err(IdentityError::IdentityNotFound)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn refuse_pid_recycle_hors_agent_legacy_et_instance_divergente() {
    let root = root("negative");
    let markers = root.join("agent-pids");
    private_dir(&markers);
    let _ = marker(&root, 20, 200, "instance-1", AGENT_A);
    let recycled = Fixture(BTreeMap::from([(20, (201, 1))]));
    assert_eq!(
        resolve_with(None, &markers, Some("instance-1"), 20, &recycled),
        Err(IdentityError::IdentityNotFound)
    );
    assert_eq!(
        resolve_with(None, &markers, Some("instance-1"), 99, &Fixture::default()),
        Err(IdentityError::IdentityNotFound)
    );
    let matching_birth = Fixture(BTreeMap::from([(20, (200, 1))]));
    assert_eq!(
        resolve_with(
            None,
            &markers,
            Some("instance-divergente"),
            20,
            &matching_birth
        ),
        Err(IdentityError::IdentityNotFound)
    );
    private_write(markers.join("77"), "ancien-nom");
    let legacy = Fixture(BTreeMap::from([(77, (770, 1))]));
    assert_eq!(
        resolve_with(None, &markers, Some("instance-1"), 77, &legacy),
        Err(IdentityError::LegacyMarker)
    );
    assert_eq!(IdentityError::LegacyMarker.code(), "legacy_marker");
    assert!(
        IdentityError::LegacyMarker
            .remediation()
            .contains("relancez")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ignore_un_nom_invalide_avant_de_consulter_les_ancetres() {
    let root = root("invalid-name");
    let _name = marker(&root, 12, 120, "instance-1", AGENT_A);
    let dynamic_name = root.join("dynamic-name");
    private_write(&dynamic_name, "agent invalide");
    let tree = Fixture(BTreeMap::from([(42, (420, 12)), (12, (120, 1))]));
    assert_eq!(
        resolve_with(
            Some(&dynamic_name),
            &root.join("agent-pids"),
            Some("instance-1"),
            42,
            &tree
        ),
        Ok(AGENT_A.into())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn inventaire_mesure_sur_l_hote_distingue_vivant_et_pid_recycle() {
    let root = root("scan-live-stale");
    let markers = root.join("agent-pids");
    let _ = marker(&root, 20, 200, "instance-nouvelle", AGENT_A);
    let _ = marker(&root, 30, 300, "instance-ancienne", AGENT_B);
    let processes = Fixture(BTreeMap::from([(20, (200, 1)), (30, (301, 1))]));

    let inventory = scan_marker_directory_with(
        &markers,
        "hote-mesure".to_string(),
        1_788_200_000,
        &processes,
    )
    .unwrap();

    assert_eq!(inventory.source.host, "hote-mesure");
    assert_eq!(inventory.source.marker_directory, markers);
    assert_eq!(inventory.live.len(), 1);
    assert_eq!(inventory.live[0].principal, AGENT_A);
    assert_eq!(inventory.live[0].instance_id, "instance-nouvelle");
    assert_eq!(inventory.stale.len(), 1);
    assert_eq!(inventory.stale[0].marker, "30");
    assert_eq!(inventory.stale[0].reason, StaleMarkerReason::BirthMismatch);
    assert!(inventory.complete);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn zero_marqueur_vivant_est_une_erreur_et_non_un_inventaire_vide() {
    let root = root("scan-zero-live");
    let markers = root.join("agent-pids");
    let _ = marker(&root, 20, 200, "instance-ancienne", AGENT_A);

    assert_eq!(
        scan_marker_directory_with(
            &markers,
            "hote-mesure".to_string(),
            1_788_200_000,
            &Fixture::default(),
        )
        .unwrap_err(),
        MarkerScanError::NoLiveMarker
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn entree_malformee_ou_repertoire_lie_interdit_un_succes_partiel() {
    let root = root("scan-invalid");
    let markers = root.join("agent-pids");
    private_dir(&markers);
    private_write(markers.join("20"), "pas-du-json");
    assert_eq!(
        scan_marker_directory_with(
            &markers,
            "hote-mesure".to_string(),
            1_788_200_000,
            &Fixture(BTreeMap::from([(20, (200, 1))])),
        )
        .unwrap_err(),
        MarkerScanError::InvalidMarker {
            marker: "20".to_string()
        }
    );

    fs::remove_file(markers.join("20")).unwrap();
    let _ = marker(&root, 20, 200, "instance-vivante", AGENT_A);
    let linked = root.join("linked-agent-pids");
    symlink(&markers, &linked).unwrap();
    assert_eq!(
        scan_marker_directory_with(
            &linked,
            "hote-mesure".to_string(),
            1_788_200_000,
            &Fixture(BTreeMap::from([(20, (200, 1))])),
        )
        .unwrap_err(),
        MarkerScanError::DirectoryNotCanonical
    );

    fs::remove_dir_all(root).unwrap();
}
