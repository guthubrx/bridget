use bridget_daemon::project_runtime::ProjectResourceCatalog;
use bridget_transport::protocol::{ProjectResourceKind, ProjectResourceRef};
use std::fs;
use std::os::unix::fs::MetadataExt;

#[test]
fn spec_067_catalogue_refuse_une_source_sans_projet_autorise() {
    let catalog = ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":["/tmp/extensions"],"secret_roots":["/tmp/secrets"],"sources":[]}"#,
    )
    .unwrap();
    assert!(catalog.source_for("project-a", "catalog:absent").is_err());
}

#[test]
fn spec_067_catalogue_refuse_projet_etranger_et_wildcard() {
    let catalog = ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":["/tmp/extensions"],"secret_roots":["/tmp/secrets"],"sources":[{"source_ref":"catalog:skill","kind":"extension","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/extensions/skill","source_revision":1,"allowed_project_ids":["project-a"]}]}"#,
    )
    .unwrap();
    assert!(catalog.source_for("project-b", "catalog:skill").is_err());
    assert!(ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":[],"secret_roots":[],"sources":[{"source_ref":"catalog:bad","kind":"extension","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/a","source_revision":1,"allowed_project_ids":["*"]}]}"#,
    )
    .is_err());
}

#[test]
fn spec_067_catalogue_refuse_source_hors_racines() {
    assert!(ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":["/tmp/extensions"],"secret_roots":["/tmp/secrets"],"sources":[{"source_ref":"catalog:outside","kind":"extension","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/outside","source_revision":1,"allowed_project_ids":["project-a"]}]}"#,
    )
    .is_err());
}

#[test]
fn spec_067_catalogue_refuse_un_secret_place_sous_les_extensions() {
    assert!(ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":["/tmp/extensions"],"secret_roots":["/tmp/secrets"],"sources":[{"source_ref":"catalog:wrong-root","kind":"secret_file","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/extensions/credential","source_revision":1,"allowed_project_ids":["project-a"]}]}"#,
    )
    .is_err());
}

#[test]
fn spec_067_catalogue_refuse_collision_de_source() {
    assert!(ProjectResourceCatalog::from_json(
        r#"{"contract_version":1,"extension_roots":["/tmp/extensions"],"secret_roots":["/tmp/secrets"],"sources":[{"source_ref":"catalog:duplicate","kind":"extension","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/extensions/a","source_revision":1,"allowed_project_ids":["project-a"]},{"source_ref":"catalog:duplicate","kind":"extension","expected_uid":1002,"expected_gid":1002,"canonical_path":"/tmp/extensions/b","source_revision":2,"allowed_project_ids":["project-a"]}]}"#,
    )
    .is_err());
}

#[test]
fn spec_067_attestation_secrete_ne_porte_ni_valeur_ni_contenu() {
    let source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/project-profile/secrets/file-secret.txt");
    let root = source.parent().unwrap();
    let metadata = fs::metadata(&source).unwrap();
    let catalog = ProjectResourceCatalog::from_json(&format!(
        r#"{{"contract_version":1,"extension_roots":[],"secret_roots":["{}"],"sources":[{{"source_ref":"catalog:secret","kind":"secret_file","expected_uid":{},"expected_gid":{},"canonical_path":"{}","source_revision":7,"allowed_project_ids":["project-a"]}}]}}"#,
        root.display(),
        metadata.uid(),
        metadata.gid(),
        source.display(),
    ))
    .unwrap();
    let resolved = catalog.attest("project-a", "catalog:secret").unwrap();
    let stamp = resolved.secret_stamp.unwrap();
    let serialized = serde_json::to_string(&stamp).unwrap();
    assert!(!serialized.contains("S067_SYNTHETIC_SECRET_FILE"));
    assert!(resolved.attestation_digest.starts_with("sha256:"));
    let resources = catalog
        .resolve_refs(
            "project-a",
            &[ProjectResourceRef {
                resource_id: "credential-file".to_string(),
                kind: ProjectResourceKind::SecretFile,
                source_ref: "catalog:secret".to_string(),
                destination: "/run/bridget/secrets/credential-file".to_string(),
                version: Some("fixture-v1".to_string()),
                content_digest: None,
                generation: 1,
            }],
        )
        .unwrap();
    assert_eq!(resources[0].source_revision, 7);
    assert_eq!(
        resources[0].reference.destination,
        "/run/bridget/secrets/credential-file"
    );
}
