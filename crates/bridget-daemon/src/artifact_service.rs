//! Orchestration atomique de publication des artefacts Bridget.
//!
//! Cette frontière reçoit une identité, un projet, une conversation et un tour
//! déjà attestés par le daemon. L'appelant ne peut donc pas publier dans le
//! projet d'un autre agent ni choisir une visibilité plus large.

use crate::artifact_blob_store::{ArtifactBlobStore, ArtifactBlobStoreError};
use crate::artifact_policy::{ArtifactPolicy, PublicationCapacity};
use crate::artifact_store::{
    ArtifactBlobReference, ArtifactPersistResult, ArtifactPublicationContext, ArtifactStore,
    ArtifactStoreError, ArtifactStoreWrite,
};
use crate::artifact_types::{
    ArtifactFailureCode, ArtifactFailureReceiptV1, ArtifactKind, ArtifactPublicationV1,
    ArtifactReceiptV1, ArtifactState, ArtifactStorageState,
};
use std::path::Path;
use uuid::Uuid;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ArtifactServiceMetrics {
    pub publication_attempts: u64,
    pub publication_failures: u64,
    pub restored_versions: u64,
    pub deleted_artifacts: u64,
    pub canonical_blob_bytes: u64,
    pub evictions: u64,
}

#[derive(Debug)]
pub enum ArtifactServiceError {
    Validation(String),
    PolicyBlocked,
    BlobUnavailable,
    Interrupted,
    Store(ArtifactStoreError),
    BlobStore(ArtifactBlobStoreError),
}

impl std::fmt::Display for ArtifactServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(message) => write!(formatter, "publication refusée: {message}"),
            Self::PolicyBlocked => formatter.write_str("publication bloquée par le quota"),
            Self::BlobUnavailable => formatter.write_str("blob canonique absent"),
            Self::Interrupted => {
                formatter.write_str("publication interrompue avant validation atomique")
            }
            Self::Store(error) => error.fmt(formatter),
            Self::BlobStore(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ArtifactServiceError {}

impl From<ArtifactStoreError> for ArtifactServiceError {
    fn from(error: ArtifactStoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ArtifactBlobStoreError> for ArtifactServiceError {
    fn from(error: ArtifactBlobStoreError) -> Self {
        Self::BlobStore(error)
    }
}

pub struct ArtifactService {
    policy: ArtifactPolicy,
    store: ArtifactStore,
    blobs: ArtifactBlobStore,
    metrics: ArtifactServiceMetrics,
}

impl ArtifactService {
    pub fn open(
        database_path: &Path,
        artifact_root: &Path,
        policy: ArtifactPolicy,
    ) -> Result<Self, ArtifactServiceError> {
        Ok(Self {
            policy,
            store: ArtifactStore::open(database_path)?,
            blobs: ArtifactBlobStore::open(artifact_root, policy)?,
            metrics: ArtifactServiceMetrics::default(),
        })
    }

    pub fn publish(
        &mut self,
        context: ArtifactPublicationContext,
        publication: ArtifactPublicationV1,
    ) -> Result<ArtifactPersistResult, ArtifactServiceError> {
        self.publish_cancellable(context, publication, || false)
    }

    /// Point de raccordement coopératif pour l'annulation d'un tour. Le
    /// magasin n'est atteint qu'après chaque barrière d'annulation : une
    /// publication interrompue ne laisse donc aucune version partielle.
    pub fn publish_cancellable(
        &mut self,
        context: ArtifactPublicationContext,
        publication: ArtifactPublicationV1,
        cancelled: impl Fn() -> bool,
    ) -> Result<ArtifactPersistResult, ArtifactServiceError> {
        self.metrics.publication_attempts = self.metrics.publication_attempts.saturating_add(1);
        let result = self.publish_checked(context, publication, &cancelled);
        if result.is_err() {
            self.metrics.publication_failures = self.metrics.publication_failures.saturating_add(1);
            let _ = self.store.record_metric("publication_failures");
        }
        self.metrics.canonical_blob_bytes = self.total_published_blob_bytes().unwrap_or(0);
        result
    }

    fn publish_checked(
        &mut self,
        context: ArtifactPublicationContext,
        mut publication: ArtifactPublicationV1,
        cancelled: &impl Fn() -> bool,
    ) -> Result<ArtifactPersistResult, ArtifactServiceError> {
        if cancelled() {
            return Err(ArtifactServiceError::Interrupted);
        }
        self.policy
            .validate_publication(&publication)
            .map_err(|error| ArtifactServiceError::Validation(error.to_string()))?;
        validate_provenance(&publication)?;
        self.prepare_html_publication(&mut publication)?;
        if cancelled() {
            return Err(ArtifactServiceError::Interrupted);
        }
        let blob_references = self.resolve_blob_references(&publication)?;
        let mut projected_bytes = 0_u64;
        for blob in &blob_references {
            let already_published = self.store.blob_reference_count(&blob.digest)?.unwrap_or(0) > 0;
            if !already_published {
                projected_bytes = projected_bytes.saturating_add(blob.byte_length);
            }
        }
        let existing_total = self.total_published_blob_bytes()?;
        if self
            .policy
            .publication_capacity(existing_total, projected_bytes)
            == PublicationCapacity::Blocked
        {
            return Err(ArtifactServiceError::PolicyBlocked);
        }
        let artifact_ref = publication
            .parent_artifact_ref
            .clone()
            .unwrap_or_else(|| format!("artifact:{}", Uuid::new_v4()));
        let version_ref = format!("artifact-version:{}", Uuid::new_v4());
        let state = if publication.quality_notices.is_empty() {
            ArtifactState::Published
        } else {
            ArtifactState::Partial
        };
        let receipt = ArtifactReceiptV1 {
            artifact_ref: artifact_ref.clone(),
            version_ref: version_ref.clone(),
            state,
            content_digest: publication.content_digest(),
            warnings: publication.quality_notices.clone(),
            conversation_reference: context.conversation_reference.clone(),
            storage_state: ArtifactStorageState::Canonical,
        };
        if cancelled() {
            return Err(ArtifactServiceError::Interrupted);
        }
        self.store
            .persist_publication(&ArtifactStoreWrite {
                artifact_ref,
                existing_artifact_ref: publication.parent_artifact_ref.clone(),
                version_ref,
                context,
                publication,
                state,
                receipt,
                blob_references,
            })
            .map_err(ArtifactServiceError::Store)
    }

    /// Produit une version enfant explicite depuis un contenu déjà collecté ou
    /// calculé par Bridget. La publication d'origine n'est jamais modifiée.
    pub fn refresh(
        &mut self,
        context: ArtifactPublicationContext,
        parent_artifact_ref: &str,
        mut publication: ArtifactPublicationV1,
    ) -> Result<ArtifactPersistResult, ArtifactServiceError> {
        publication.parent_artifact_ref = Some(parent_artifact_ref.to_string());
        publication.publication_reason = crate::artifact_types::PublicationReason::Refresh;
        self.publish(context, publication)
    }

    /// Restaure la même version historique après vérification de ses blobs.
    /// Un blob absent est un échec explicite, jamais une vignette vide.
    pub fn restore(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
    ) -> Result<Vec<String>, ArtifactServiceError> {
        let digests = self.store.artifact_blob_digests(project_id, artifact_ref)?;
        for digest in &digests {
            if !self.blobs.contains(digest)? {
                return Err(ArtifactServiceError::BlobUnavailable);
            }
        }
        let restored = self.store.restore(project_id, artifact_ref)?;
        self.metrics.restored_versions = self.metrics.restored_versions.saturating_add(1);
        let _ = self.store.record_metric("restored_versions");
        self.metrics.canonical_blob_bytes = self.total_published_blob_bytes().unwrap_or(0);
        Ok(restored)
    }

    pub fn set_pinned(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
        pinned: bool,
    ) -> Result<(), ArtifactServiceError> {
        self.store.set_pinned(project_id, artifact_ref, pinned)?;
        Ok(())
    }

    pub fn soft_delete(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
        deleted_at: i64,
    ) -> Result<Vec<String>, ArtifactServiceError> {
        let deleted = self
            .store
            .soft_delete(project_id, artifact_ref, deleted_at)?;
        self.metrics.deleted_artifacts = self.metrics.deleted_artifacts.saturating_add(1);
        self.metrics.evictions = self.metrics.evictions.saturating_add(1);
        let _ = self.store.record_metric("evictions");
        self.metrics.canonical_blob_bytes = self.total_published_blob_bytes().unwrap_or(0);
        Ok(deleted)
    }

    pub fn metrics(&self) -> ArtifactServiceMetrics {
        self.metrics.clone()
    }

    pub fn persisted_metrics(
        &self,
    ) -> Result<crate::artifact_store::ArtifactMetricsSnapshot, ArtifactServiceError> {
        self.store
            .metrics_snapshot()
            .map_err(ArtifactServiceError::Store)
    }

    /// Réponse sûre pour les relais UI : aucune chaîne d'erreur interne, une
    /// cause stable et l'action de récupération qui reste sous autorité
    /// Bridget.
    pub fn failure_receipt(error: &ArtifactServiceError) -> ArtifactFailureReceiptV1 {
        let code = match error {
            ArtifactServiceError::Validation(_) => ArtifactFailureCode::ValidationRefused,
            ArtifactServiceError::PolicyBlocked => ArtifactFailureCode::QuotaBlocked,
            ArtifactServiceError::BlobUnavailable => ArtifactFailureCode::BlobMissing,
            ArtifactServiceError::Interrupted => ArtifactFailureCode::OperationInterrupted,
            ArtifactServiceError::Store(_) | ArtifactServiceError::BlobStore(_) => {
                ArtifactFailureCode::StorageUnavailable
            }
        };
        ArtifactFailureReceiptV1::new(code, error.to_string())
    }

    pub fn ingest_blob(&self, bytes: &[u8]) -> Result<String, ArtifactServiceError> {
        self.blobs
            .put(bytes)
            .map_err(ArtifactServiceError::BlobStore)
    }

    /// Même magasin canonique que la publication ; la portée a déjà été
    /// attestée par le daemon. Ni chemin physique, ni accès par digest seul.
    pub fn read(
        &self,
        scope: &str,
        request: &bridget_transport::protocol::ArtifactReadRequest,
    ) -> Result<
        bridget_transport::protocol::ArtifactReadOutcome,
        bridget_transport::protocol::ArtifactReadRefusal,
    > {
        use bridget_transport::protocol::{
            ARTIFACT_READ_VERSION, ArtifactReadKind, ArtifactReadOutcome,
            ArtifactReadRefusal as Refusal, MAX_ARTIFACT_READ_BYTES,
        };
        if request.version != ARTIFACT_READ_VERSION {
            return Err(Refusal::UnsupportedVersion);
        }
        if request.limit == 0
            || request.limit > MAX_ARTIFACT_READ_BYTES
            || request.artifact_ref.is_empty()
            || request.artifact_ref.len() > bridget_transport::protocol::MAX_ARTIFACT_REF_BYTES
            || request.version_ref.is_empty()
            || request.version_ref.len() > bridget_transport::protocol::MAX_ARTIFACT_REF_BYTES
        {
            return Err(Refusal::InvalidRequest);
        }
        let detail = self
            .store
            .version_detail(scope, &request.version_ref)
            .map_err(|error| match error {
                ArtifactStoreError::CorruptManifest => Refusal::CorruptContent,
                _ => Refusal::StorageUnavailable,
            })?
            .filter(|detail| detail.item.artifact_ref == request.artifact_ref)
            .ok_or(Refusal::NotFound)?;
        let (bytes, total_len, digest) = match &request.kind {
            ArtifactReadKind::Manifest => {
                let total_len = detail.manifest_bytes.len() as u64;
                if request.offset > total_len {
                    return Err(Refusal::OffsetOutOfRange);
                }
                let end = request
                    .offset
                    .saturating_add(u64::from(request.limit))
                    .min(total_len);
                (
                    detail.manifest_bytes[request.offset as usize..end as usize].to_vec(),
                    total_len,
                    detail.content_digest,
                )
            }
            ArtifactReadKind::Blob { digest } => {
                if digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                {
                    return Err(Refusal::InvalidRequest);
                }
                let blob = self
                    .store
                    .version_blob(scope, &request.version_ref, digest)
                    .map_err(|_| Refusal::StorageUnavailable)?
                    .ok_or(Refusal::BlobNotLinked)?;
                if request.offset > blob.byte_length {
                    return Err(Refusal::OffsetOutOfRange);
                }
                let bytes = self
                    .blobs
                    .read_slice(digest, blob.byte_length, request.offset, request.limit)
                    .map_err(|error| match error {
                        ArtifactBlobStoreError::Missing => Refusal::ContentUnavailable,
                        ArtifactBlobStoreError::Corrupt { .. }
                        | ArtifactBlobStoreError::Policy(_) => Refusal::CorruptContent,
                        ArtifactBlobStoreError::InvalidDigest => Refusal::InvalidRequest,
                        ArtifactBlobStoreError::Io(_) => Refusal::StorageUnavailable,
                    })?;
                (bytes, blob.byte_length, digest.clone())
            }
        };
        let end = request.offset + bytes.len() as u64;
        Ok(ArtifactReadOutcome::Chunk {
            bytes,
            total_len,
            digest,
            offset: request.offset,
            next_offset: (end < total_len).then_some(end),
        })
    }

    pub fn blob_bytes(&self, digest: &str) -> Result<Vec<u8>, ArtifactServiceError> {
        self.blobs
            .read(digest)
            .map_err(ArtifactServiceError::BlobStore)
    }

    fn resolve_blob_references(
        &self,
        publication: &ArtifactPublicationV1,
    ) -> Result<Vec<ArtifactBlobReference>, ArtifactServiceError> {
        let Some(digest) = publication
            .payload
            .get("blob_digest")
            .and_then(|value| value.as_str())
        else {
            return Ok(Vec::new());
        };
        if !self.blobs.contains(digest)? {
            return Err(ArtifactServiceError::BlobUnavailable);
        }
        let media_type = publication
            .payload
            .get("media_type")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                ArtifactServiceError::Validation("media_type de blob absent".to_string())
            })?;
        let path = self.blobs.canonical_path(digest)?;
        let byte_length = std::fs::metadata(&path)
            .map_err(ArtifactBlobStoreError::Io)?
            .len();
        Ok(vec![ArtifactBlobReference {
            digest: digest.to_string(),
            media_type: media_type.to_string(),
            byte_length,
            canonical_path: path.display().to_string(),
        }])
    }

    fn prepare_html_publication(
        &self,
        publication: &mut ArtifactPublicationV1,
    ) -> Result<(), ArtifactServiceError> {
        if publication.kind != ArtifactKind::Html {
            return Ok(());
        }
        let html = publication
            .payload
            .get("html")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| ArtifactServiceError::Validation("html canonique absent".to_owned()))?;
        if html.len() > 512 * 1024 {
            return Err(ArtifactServiceError::Validation(
                "contenu HTML supérieur à 512 Kio".to_owned(),
            ));
        }
        let digest = self.ingest_blob(html.as_bytes())?;
        let payload = publication
            .payload
            .as_object_mut()
            .ok_or_else(|| ArtifactServiceError::Validation("payload HTML invalide".to_owned()))?;
        payload.insert("blob_digest".to_owned(), serde_json::Value::String(digest));
        payload.insert(
            "media_type".to_owned(),
            serde_json::Value::String("text/html; charset=utf-8".to_owned()),
        );
        payload
            .entry("runtime_policy".to_owned())
            .or_insert_with(|| serde_json::Value::String("sandbox-v1".to_owned()));
        Ok(())
    }

    fn total_published_blob_bytes(&self) -> Result<u64, ArtifactServiceError> {
        self.store
            .total_published_blob_bytes()
            .map_err(ArtifactServiceError::Store)
    }
}

fn validate_provenance(publication: &ArtifactPublicationV1) -> Result<(), ArtifactServiceError> {
    for source in &publication.sources {
        match source.source_kind {
            crate::artifact_types::ArtifactSourceKind::Remote => {
                if source.content_digest.is_none() || source.fetched_at.is_none() {
                    return Err(ArtifactServiceError::Validation(
                        "source distante sans contenu attesté".to_string(),
                    ));
                }
            }
            crate::artifact_types::ArtifactSourceKind::LocalProject => {
                if source.content_digest.is_none() {
                    return Err(ArtifactServiceError::Validation(
                        "source projet sans contenu attesté".to_string(),
                    ));
                }
            }
            crate::artifact_types::ArtifactSourceKind::AgentComputed => {
                if source.content_digest.is_none() || source.transformations.is_empty() {
                    return Err(ArtifactServiceError::Validation(
                        "provenance calculée incomplète".to_string(),
                    ));
                }
            }
            crate::artifact_types::ArtifactSourceKind::UserSupplied => {}
            crate::artifact_types::ArtifactSourceKind::Restored => {
                if source.content_digest.is_none() || source.fetched_at.is_none() {
                    return Err(ArtifactServiceError::Validation(
                        "restauration sans contenu attesté".to_string(),
                    ));
                }
            }
        }
    }
    Ok(())
}
