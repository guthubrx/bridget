use bridget_daemon::artifact_service::{ArtifactService, ArtifactServiceError};
use bridget_daemon::artifact_store::{
    ArtifactPersistResult, ArtifactPublicationContext, ArtifactStore,
};
use bridget_daemon::artifact_types::{ArtifactPublicationV1, PublicationReason};
use std::fs;
use std::path::PathBuf;

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-artifact-service-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn context(turn: &str) -> ArtifactPublicationContext {
    ArtifactPublicationContext {
        project_id: "project:service-fixture".to_string(),
        conversation_reference: "conversation:service-fixture".to_string(),
        turn_reference: turn.to_string(),
        created_by: "agent:service-fixture".to_string(),
        origin_instance: "instance:service-fixture".to_string(),
        observed_at: 1_800_000_000,
    }
}

fn external_publication() -> ArtifactPublicationV1 {
    serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json"))
        .expect("fixture publication valide")
}

#[test]
fn publication_attribuee_est_rejouable_et_versionnee_sans_recalculer_le_quota() {
    let root = fixture_root("publish");
    fs::create_dir_all(&root).unwrap();
    let db = root.join("bridget.db");
    let artifacts = root.join("artifacts");
    let mut service = ArtifactService::open(&db, &artifacts, Default::default()).unwrap();

    let first = service
        .publish(context("turn:one"), external_publication())
        .unwrap();
    let artifact_ref = match &first {
        ArtifactPersistResult::Created(receipt) => receipt.artifact_ref.clone(),
        ArtifactPersistResult::Replayed(_) => panic!("première publication rejouée"),
    };
    let replay = service
        .publish(context("turn:one"), external_publication())
        .unwrap();
    assert_eq!(replay, first.clone().map_created_to_replayed());

    let mut child = external_publication();
    child.idempotency_key = "fixture-chart-external-refresh-v1".to_string();
    child.parent_artifact_ref = Some(artifact_ref);
    child.publication_reason = PublicationReason::Refresh;
    let created_child = service.publish(context("turn:two"), child).unwrap();
    let child_receipt = match created_child {
        ArtifactPersistResult::Created(receipt) => receipt,
        ArtifactPersistResult::Replayed(_) => panic!("version enfant rejouée"),
    };
    assert_ne!(
        child_receipt.version_ref,
        match first {
            ArtifactPersistResult::Created(receipt) => receipt.version_ref,
            ArtifactPersistResult::Replayed(_) => unreachable!(),
        }
    );

    drop(service);
    let store = ArtifactStore::open(&db).unwrap();
    let detail = store
        .version_detail("project:service-fixture", &child_receipt.version_ref)
        .unwrap()
        .unwrap();
    assert!(detail.parent_version_ref.is_some());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn html_inerte_est_canonique_et_blob_absent_ne_devient_pas_un_succes() {
    let root = fixture_root("refusal");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let mut html = external_publication();
    html.kind = bridget_daemon::artifact_types::ArtifactKind::Html;
    html.idempotency_key = "fixture-html-sandbox-v1".to_owned();
    html.payload = serde_json::json!({
        "html": "<main><h1>Visualisation</h1><script>window.parent.postMessage({type:'sandbox.resize',height:420}, '*')</script></main>",
        "data": { "series": [1, 2, 3] },
        "inline_height_hint": 420,
    });
    let initial = service.publish(context("turn:html"), html.clone()).unwrap();
    let parent = match initial {
        ArtifactPersistResult::Created(receipt) => receipt.artifact_ref,
        ArtifactPersistResult::Replayed(_) => panic!("création HTML rejouée"),
    };
    html.idempotency_key = "fixture-html-sandbox-child-v1".to_owned();
    html.payload["ui_state"] = serde_json::json!({ "filter": "2026" });
    let child = service
        .refresh(context("turn:html-child"), &parent, html)
        .unwrap();
    assert!(matches!(child, ArtifactPersistResult::Created(_)));
    let mut file: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/file-v1.json")).unwrap();
    file.idempotency_key = "fixture-file-absent-v1".to_string();
    assert!(matches!(
        service.publish(context("turn:file"), file),
        Err(ArtifactServiceError::BlobUnavailable)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cycle_de_vie_restaure_la_version_exacte_et_actualise_en_enfant() {
    let root = fixture_root("lifecycle");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let created = service
        .publish(context("turn:initial"), external_publication())
        .unwrap();
    let artifact_ref = match created {
        ArtifactPersistResult::Created(receipt) => receipt.artifact_ref,
        ArtifactPersistResult::Replayed(_) => panic!("la création ne peut pas être rejouée"),
    };
    service
        .soft_delete("project:service-fixture", &artifact_ref, 1_800_000_010)
        .unwrap();
    assert!(
        service
            .restore("project:service-fixture", &artifact_ref)
            .unwrap()
            .is_empty()
    );

    let mut refreshed = external_publication();
    refreshed.idempotency_key = "fixture-chart-service-refresh".to_string();
    let child = service
        .refresh(context("turn:refresh"), &artifact_ref, refreshed)
        .unwrap();
    let child_version = match child {
        ArtifactPersistResult::Created(receipt) => receipt.version_ref,
        ArtifactPersistResult::Replayed(_) => panic!("le refresh doit créer une enfant"),
    };
    assert_ne!(child_version, "artifact-version:missing");
    let metrics = service.metrics();
    assert_eq!(metrics.publication_attempts, 2);
    assert_eq!(metrics.publication_failures, 0);
    assert_eq!(metrics.restored_versions, 1);
    assert_eq!(metrics.deleted_artifacts, 1);
    let _ = fs::remove_dir_all(root);
}

trait PersistResultExpectation {
    fn map_created_to_replayed(self) -> Self;
}

impl PersistResultExpectation for ArtifactPersistResult {
    fn map_created_to_replayed(self) -> Self {
        match self {
            Self::Created(receipt) | Self::Replayed(receipt) => Self::Replayed(receipt),
        }
    }
}
