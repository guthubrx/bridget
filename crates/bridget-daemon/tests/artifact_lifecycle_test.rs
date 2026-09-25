use bridget_daemon::artifact_service::{ArtifactService, ArtifactServiceError};
use bridget_daemon::artifact_store::{ArtifactPersistResult, ArtifactPublicationContext};
use bridget_daemon::artifact_types::{
    ArtifactFailureCode, ArtifactPublicationV1, ArtifactRecoveryAction,
};
use std::fs;
use std::path::PathBuf;

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-artifact-lifecycle-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn context() -> ArtifactPublicationContext {
    ArtifactPublicationContext {
        project_id: "project:lifecycle".to_string(),
        conversation_reference: "conversation:project:lifecycle:agent".to_string(),
        turn_reference: "turn:lifecycle".to_string(),
        created_by: "agent:lifecycle".to_string(),
        origin_instance: "instance:lifecycle".to_string(),
        observed_at: 1_800_000_000,
    }
}

#[test]
fn restauration_incomplete_ne_fabrique_pas_de_faux_succes() {
    let root = fixture_root("missing-blob");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let publication: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/file-v1.json")).unwrap();
    assert!(matches!(
        service.publish(context(), publication),
        Err(ArtifactServiceError::BlobUnavailable)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn pin_suppression_et_restauration_ne_changent_pas_la_version() {
    let root = fixture_root("immutable");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let publication: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json")).unwrap();
    let receipt = match service.publish(context(), publication).unwrap() {
        ArtifactPersistResult::Created(receipt) => receipt,
        ArtifactPersistResult::Replayed(_) => panic!("création inattendue rejouée"),
    };
    service
        .set_pinned("project:lifecycle", &receipt.artifact_ref, true)
        .unwrap();
    service
        .soft_delete("project:lifecycle", &receipt.artifact_ref, 1_800_000_010)
        .unwrap();
    service
        .restore("project:lifecycle", &receipt.artifact_ref)
        .unwrap();
    let metrics = service.metrics();
    assert_eq!(metrics.deleted_artifacts, 1);
    assert_eq!(metrics.restored_versions, 1);
    assert_eq!(metrics.evictions, 1);
    let persisted = service.persisted_metrics().unwrap();
    assert_eq!(persisted.restored_versions, 1);
    assert_eq!(persisted.evictions, 1);
    assert_eq!(persisted.canonical_blob_bytes, 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn annulation_avant_ecriture_ne_cree_aucune_version_partielle() {
    let root = fixture_root("cancelled");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let publication: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json")).unwrap();
    let error = service
        .publish_cancellable(context(), publication, || true)
        .unwrap_err();
    assert!(matches!(error, ArtifactServiceError::Interrupted));
    let receipt = ArtifactService::failure_receipt(&error);
    assert_eq!(receipt.code, ArtifactFailureCode::OperationInterrupted);
    assert_eq!(
        receipt.recovery_action,
        ArtifactRecoveryAction::ResumeWithAgent
    );
    let store =
        bridget_daemon::artifact_store::ArtifactStore::open(&root.join("bridget.db")).unwrap();
    assert!(
        store
            .list_project("project:lifecycle", 10, None)
            .unwrap()
            .is_empty()
    );
    let _ = fs::remove_dir_all(root);
}
