use bridget_daemon::artifact_store::{
    ArtifactBlobReference, ArtifactPersistResult, ArtifactPublicationContext, ArtifactStore,
    ArtifactStoreError, ArtifactStoreWrite,
};
use bridget_daemon::artifact_types::{
    ArtifactPublicationV1, ArtifactReceiptV1, ArtifactState, ArtifactStorageState,
    PublicationReason, sha256_hex,
};
use std::fs;
use std::path::PathBuf;

const PROJECT: &str = "project:fixtures";
const CONVERSATION: &str = "conversation:fixtures";

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-artifact-store-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn publication() -> ArtifactPublicationV1 {
    serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json"))
        .expect("fixture publication valide")
}

fn write(
    artifact_ref: &str,
    version_ref: &str,
    existing_artifact_ref: Option<&str>,
    mut publication: ArtifactPublicationV1,
    now: i64,
    blobs: Vec<ArtifactBlobReference>,
) -> ArtifactStoreWrite {
    if let Some(existing) = existing_artifact_ref {
        publication.parent_artifact_ref = Some(existing.to_string());
        publication.publication_reason = PublicationReason::Refresh;
    }
    let state = if publication.quality_notices.is_empty() {
        ArtifactState::Published
    } else {
        ArtifactState::Partial
    };
    let receipt = ArtifactReceiptV1 {
        artifact_ref: artifact_ref.to_string(),
        version_ref: version_ref.to_string(),
        state,
        content_digest: publication.content_digest(),
        warnings: publication.quality_notices.clone(),
        conversation_reference: CONVERSATION.to_string(),
        storage_state: ArtifactStorageState::Canonical,
    };
    ArtifactStoreWrite {
        artifact_ref: artifact_ref.to_string(),
        existing_artifact_ref: existing_artifact_ref.map(ToString::to_string),
        version_ref: version_ref.to_string(),
        context: ArtifactPublicationContext {
            project_id: PROJECT.to_string(),
            conversation_reference: CONVERSATION.to_string(),
            turn_reference: format!("turn:{now}"),
            created_by: "agent:fixture".to_string(),
            origin_instance: "instance:fixture".to_string(),
            observed_at: now,
        },
        publication,
        state,
        receipt,
        blob_references: blobs,
    }
}

#[test]
fn migration_publication_et_rejeu_sont_atomiques() {
    let root = fixture_root("replay");
    fs::create_dir_all(&root).unwrap();
    let db = root.join("bridget.db");
    let mut store = ArtifactStore::open(&db).unwrap();
    let input = write(
        "artifact:one",
        "artifact-version:one",
        None,
        publication(),
        1_800_000_000,
        Vec::new(),
    );
    let first = store.persist_publication(&input).unwrap();
    assert!(matches!(first, ArtifactPersistResult::Created(_)));
    let second = store.persist_publication(&input).unwrap();
    assert_eq!(second, first.clone().map_created_to_replayed());
    let listed = store.list_project(PROJECT, 10, None).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].version_ref, "artifact-version:one");
    assert!(
        store
            .version_detail("project:other", "artifact-version:one")
            .unwrap()
            .is_none()
    );
    let references = store
        .list_conversation_references(PROJECT, CONVERSATION, 10)
        .unwrap();
    assert_eq!(references.len(), 1);
    assert_eq!(references[0].version_ref, "artifact-version:one");
    assert!(
        store
            .list_conversation_references("project:other", CONVERSATION, 10)
            .unwrap()
            .is_empty()
    );
    drop(store);
    ArtifactStore::open(&db).unwrap();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn version_enfant_et_suppression_recuperable_respectent_les_references() {
    let root = fixture_root("lifecycle");
    fs::create_dir_all(&root).unwrap();
    let db = root.join("bridget.db");
    let mut store = ArtifactStore::open(&db).unwrap();
    let digest = sha256_hex(b"blob partage");
    let blob = ArtifactBlobReference {
        digest: digest.clone(),
        media_type: "text/plain".to_string(),
        byte_length: 12,
        canonical_path: root.join("blobs").join(&digest).display().to_string(),
    };
    let initial = write(
        "artifact:two",
        "artifact-version:two-1",
        None,
        publication(),
        1_800_000_001,
        vec![blob.clone()],
    );
    store.persist_publication(&initial).unwrap();
    let mut refreshed = publication();
    refreshed.idempotency_key = "fixture-chart-refresh-v1".to_string();
    let child = write(
        "artifact:two",
        "artifact-version:two-2",
        Some("artifact:two"),
        refreshed,
        1_800_000_002,
        vec![blob],
    );
    store.persist_publication(&child).unwrap();
    let detail = store
        .version_detail(PROJECT, "artifact-version:two-2")
        .unwrap()
        .unwrap();
    assert_eq!(
        detail.parent_version_ref.as_deref(),
        Some("artifact-version:two-1")
    );
    assert_eq!(store.blob_reference_count(&digest).unwrap(), Some(2));
    store.set_pinned(PROJECT, "artifact:two", true).unwrap();
    assert_eq!(
        store
            .soft_delete(PROJECT, "artifact:two", 1_800_000_003)
            .unwrap(),
        vec![digest.clone()]
    );
    assert_eq!(store.blob_reference_count(&digest).unwrap(), Some(0));
    assert!(store.list_project(PROJECT, 10, None).unwrap().is_empty());
    assert_eq!(
        store.restore(PROJECT, "artifact:two").unwrap(),
        vec![digest.clone()]
    );
    assert_eq!(store.blob_reference_count(&digest).unwrap(), Some(2));
    assert_eq!(store.list_project(PROJECT, 10, None).unwrap().len(), 1);
    assert!(matches!(
        store.restore("project:other", "artifact:two"),
        Err(ArtifactStoreError::CrossProject)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn recherche_reste_dans_le_projet_et_partage_est_explicitement_cible() {
    let root = fixture_root("search-share");
    fs::create_dir_all(&root).unwrap();
    let db = root.join("bridget.db");
    let mut store = ArtifactStore::open(&db).unwrap();
    let input = write(
        "artifact:search",
        "artifact-version:search-1",
        None,
        publication(),
        1_800_000_005,
        Vec::new(),
    );
    store.persist_publication(&input).unwrap();
    let mut other_input = write(
        "artifact:other-project",
        "artifact-version:other-project-1",
        None,
        publication(),
        1_800_000_007,
        Vec::new(),
    );
    other_input.context.project_id = "project:other".to_string();
    other_input.context.conversation_reference = "conversation:other".to_string();
    other_input.receipt.artifact_ref = "artifact:other-project".to_string();
    other_input.receipt.version_ref = "artifact-version:other-project-1".to_string();
    other_input.receipt.conversation_reference = "conversation:other".to_string();
    store.persist_publication(&other_input).unwrap();
    assert_eq!(
        store
            .search_project(PROJECT, "Prix", 10, None)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        store
            .search_project("project:other", "Prix", 10, None)
            .unwrap()
            .len(),
        1,
        "chaque projet retrouve ses propres publications seulement"
    );
    assert_eq!(store.list_project(PROJECT, 10, None).unwrap().len(), 1);
    assert_eq!(
        store.list_project("project:other", 10, None).unwrap().len(),
        1
    );
    assert!(
        store
            .version_detail(PROJECT, "artifact-version:other-project-1")
            .unwrap()
            .is_none(),
        "une version devinée d'un autre projet ne doit jamais être lisible"
    );
    let share = store
        .share_with_agent(
            PROJECT,
            "artifact-version:search-1",
            "agent:reader",
            "agent:fixture",
            1_800_000_006,
        )
        .unwrap();
    assert!(share.starts_with("artifact-share:artifact-version:search-1:"));
    assert!(matches!(
        store.share_with_agent(
            "project:other",
            "artifact-version:search-1",
            "agent:reader",
            "agent:fixture",
            1_800_000_006,
        ),
        Err(ArtifactStoreError::UnknownArtifact)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manifeste_corrompu_est_refuse_plutot_qu_affiche() {
    let root = fixture_root("corruption");
    fs::create_dir_all(&root).unwrap();
    let db = root.join("bridget.db");
    let mut store = ArtifactStore::open(&db).unwrap();
    let input = write(
        "artifact:three",
        "artifact-version:three-1",
        None,
        publication(),
        1_800_000_004,
        Vec::new(),
    );
    store.persist_publication(&input).unwrap();
    drop(store);
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute(
        "UPDATE artifact_versions SET manifest_json = X'00' WHERE version_ref = ?1",
        ["artifact-version:three-1"],
    )
    .unwrap();
    drop(conn);
    let store = ArtifactStore::open(&db).unwrap();
    assert!(matches!(
        store.version_detail(PROJECT, "artifact-version:three-1"),
        Err(ArtifactStoreError::CorruptManifest)
    ));
    let _ = fs::remove_dir_all(root);
}

trait PersistResultExpectation {
    fn map_created_to_replayed(self) -> Self;
}

impl PersistResultExpectation for ArtifactPersistResult {
    fn map_created_to_replayed(self) -> Self {
        match self {
            Self::Created(receipt) => Self::Replayed(receipt),
            Self::Replayed(receipt) => Self::Replayed(receipt),
        }
    }
}
