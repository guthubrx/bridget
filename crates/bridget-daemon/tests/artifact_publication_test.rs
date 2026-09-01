use bridget_daemon::artifact_service::{ArtifactService, ArtifactServiceError};
use bridget_daemon::artifact_store::{
    ArtifactPersistResult, ArtifactPublicationContext, ArtifactStoreError,
};
use bridget_daemon::artifact_types::{ArtifactPublicationV1, ArtifactState, PublicationReason};
use std::fs;
use std::path::PathBuf;

fn fixture_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bridget-artifact-publication-{label}-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn context(project_id: &str, turn: &str) -> ArtifactPublicationContext {
    ArtifactPublicationContext {
        project_id: project_id.to_string(),
        conversation_reference: format!("conversation:{project_id}:agent-fixture"),
        turn_reference: turn.to_string(),
        created_by: "agent:fixture".to_string(),
        origin_instance: "instance:fixture".to_string(),
        observed_at: 1_800_000_000,
    }
}

fn publication() -> ArtifactPublicationV1 {
    serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json"))
        .expect("fixture de contrat valide")
}

#[test]
fn publication_accepte_rejeu_identique_et_refuse_parametre_ou_provenance_hors_contrat() {
    let root = fixture_root("contract");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();

    let first = service
        .publish(context("project:one", "turn:one"), publication())
        .unwrap();
    let replay = service
        .publish(context("project:one", "turn:one"), publication())
        .unwrap();
    assert_eq!(replay, first.clone().map_created_to_replayed());

    let unknown = r#"{
      "idempotency_key":"fixture-extra",
      "kind":"kpi",
      "title":"Interdit",
      "payload":{},
      "sources":[],
      "publication_reason":"initial",
      "project_id":"project:impose"
    }"#;
    assert!(serde_json::from_str::<ArtifactPublicationV1>(unknown).is_err());

    let mut invalid = publication();
    invalid.idempotency_key = "fixture-provenance-incomplete".to_string();
    invalid.sources[0].content_digest = None;
    assert!(matches!(
        service.publish(context("project:one", "turn:invalid"), invalid),
        Err(ArtifactServiceError::Validation(_))
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn rattachement_tour_et_isolement_projet_restent_attestes() {
    let root = fixture_root("scope");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let created = service
        .publish(context("project:one", "turn:origin"), publication())
        .unwrap();
    let artifact_ref = match created {
        ArtifactPersistResult::Created(receipt) => receipt.artifact_ref,
        ArtifactPersistResult::Replayed(_) => panic!("première publication rejouée"),
    };

    let mut child = publication();
    child.idempotency_key = "fixture-cross-project-child".to_string();
    child.parent_artifact_ref = Some(artifact_ref);
    child.publication_reason = PublicationReason::Refresh;
    assert!(matches!(
        service.publish(context("project:two", "turn:attempt"), child),
        Err(ArtifactServiceError::Store(
            ArtifactStoreError::CrossProject
        ))
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn publication_html_sandboxee_est_acceptee_comme_artefact_canonique() {
    let root = fixture_root("html-sandbox");
    fs::create_dir_all(&root).unwrap();
    let mut service = ArtifactService::open(
        &root.join("bridget.db"),
        &root.join("artifacts"),
        Default::default(),
    )
    .unwrap();
    let publication: ArtifactPublicationV1 = serde_json::from_str(
        r#"{
          "idempotency_key":"fixture-html-sandbox-v1",
          "kind":"html",
          "title":"Simulateur hors ligne",
          "payload":{
            "html":"<!doctype html><title>Simulateur</title><button>Tester</button>",
            "data":{"distance_km":10},
            "inline_height_hint":360
          },
          "sources":[{
            "source_kind":"user_supplied",
            "locator":"conversation:fixture-html",
            "citation":"Données fournies par l’opérateur",
            "access_status":"available"
          }],
          "publication_reason":"initial"
        }"#,
    )
    .unwrap();

    let result = service
        .publish(context("project:html", "turn:html"), publication)
        .unwrap();
    let receipt = match result {
        ArtifactPersistResult::Created(receipt) => receipt,
        ArtifactPersistResult::Replayed(_) => panic!("première publication rejouée"),
    };
    assert_eq!(receipt.state, ArtifactState::Published);
    assert!(
        receipt
            .content_digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );
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
