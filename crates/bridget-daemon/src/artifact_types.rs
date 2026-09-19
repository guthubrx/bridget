//! Contrats versionnés des artefacts publiés par Bridget.
//!
//! Ce module ne fait ni I/O, ni appel réseau. Il porte uniquement les
//! invariants qui doivent être partagés par le MCP, le stockage et le renderer.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const ARTIFACT_CONTRACT_VERSION: u8 = 1;
pub const MAX_TITLE_BYTES: usize = 240;
pub const MAX_IDEMPOTENCY_KEY_BYTES: usize = 128;
pub const MAX_QUALITY_NOTICE_BYTES: usize = 2_048;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Chart,
    Kpi,
    Table,
    Timeline,
    Image,
    File,
    Html,
}

impl ArtifactKind {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Chart => "chart",
            Self::Kpi => "kpi",
            Self::Table => "table",
            Self::Timeline => "timeline",
            Self::Image => "image",
            Self::File => "file",
            Self::Html => "html",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "chart" => Some(Self::Chart),
            "kpi" => Some(Self::Kpi),
            "table" => Some(Self::Table),
            "timeline" => Some(Self::Timeline),
            "image" => Some(Self::Image),
            "file" => Some(Self::File),
            "html" => Some(Self::Html),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactState {
    Published,
    Partial,
    Unavailable,
    Failed,
    Deleted,
}

impl ArtifactState {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::Partial => "partial",
            Self::Unavailable => "unavailable",
            Self::Failed => "failed",
            Self::Deleted => "deleted",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "published" => Some(Self::Published),
            "partial" => Some(Self::Partial),
            "unavailable" => Some(Self::Unavailable),
            "failed" => Some(Self::Failed),
            "deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationReason {
    Initial,
    Refresh,
    RestoreChanged,
    SaveInteraction,
}

impl PublicationReason {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Refresh => "refresh",
            Self::RestoreChanged => "restore_changed",
            Self::SaveInteraction => "save_interaction",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "initial" => Some(Self::Initial),
            "refresh" => Some(Self::Refresh),
            "restore_changed" => Some(Self::RestoreChanged),
            "save_interaction" => Some(Self::SaveInteraction),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSourceKind {
    Remote,
    LocalProject,
    UserSupplied,
    AgentComputed,
    Restored,
}

impl ArtifactSourceKind {
    pub fn requires_fetched_at(self) -> bool {
        matches!(self, Self::Remote | Self::Restored)
    }

    pub fn as_db(self) -> &'static str {
        match self {
            Self::Remote => "remote",
            Self::LocalProject => "local_project",
            Self::UserSupplied => "user_supplied",
            Self::AgentComputed => "agent_computed",
            Self::Restored => "restored",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "remote" => Some(Self::Remote),
            "local_project" => Some(Self::LocalProject),
            "user_supplied" => Some(Self::UserSupplied),
            "agent_computed" => Some(Self::AgentComputed),
            "restored" => Some(Self::Restored),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSourceAccess {
    Available,
    Expired,
    Unavailable,
    Blocked,
    Unknown,
}

impl ArtifactSourceAccess {
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Expired => "expired",
            Self::Unavailable => "unavailable",
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "available" => Some(Self::Available),
            "expired" => Some(Self::Expired),
            "unavailable" => Some(Self::Unavailable),
            "blocked" => Some(Self::Blocked),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSourceV1 {
    pub source_kind: ArtifactSourceKind,
    pub locator: String,
    #[serde(default)]
    pub fetched_at: Option<i64>,
    #[serde(default)]
    pub content_digest: Option<String>,
    pub citation: String,
    #[serde(default)]
    pub units: Option<String>,
    #[serde(default)]
    pub transformations: Vec<String>,
    #[serde(default = "default_source_access")]
    pub access_status: ArtifactSourceAccess,
}

fn default_source_access() -> ArtifactSourceAccess {
    ArtifactSourceAccess::Unknown
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactPublicationV1 {
    pub idempotency_key: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub payload: Value,
    pub sources: Vec<ArtifactSourceV1>,
    #[serde(default)]
    pub quality_notices: Vec<String>,
    #[serde(default)]
    pub parent_artifact_ref: Option<String>,
    pub publication_reason: PublicationReason,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReceiptV1 {
    pub artifact_ref: String,
    pub version_ref: String,
    pub state: ArtifactState,
    pub content_digest: String,
    pub warnings: Vec<String>,
    pub conversation_reference: String,
    pub storage_state: ArtifactStorageState,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStorageState {
    Canonical,
    CanonicalAndCached,
}

/// Cause stable d'un échec d'artefact exposée aux surfaces de consultation.
///
/// Ce code est volontairement court, non sensible et distinct du détail
/// technique interne. Il permet à l'interface de proposer une récupération
/// honnête au lieu de laisser une vignette ou un placeholder silencieux.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFailureCode {
    SourceUnavailable,
    SourceTimeout,
    QuotaBlocked,
    BlobMissing,
    ValidationRefused,
    OperationInterrupted,
    StorageUnavailable,
}

impl ArtifactFailureCode {
    pub fn recovery_action(self) -> ArtifactRecoveryAction {
        match self {
            Self::SourceUnavailable | Self::SourceTimeout => {
                ArtifactRecoveryAction::RetryThroughBridget
            }
            Self::QuotaBlocked => ArtifactRecoveryAction::ReviewRetention,
            Self::BlobMissing => ArtifactRecoveryAction::RestoreThroughBridget,
            Self::ValidationRefused => ArtifactRecoveryAction::InspectManifest,
            Self::OperationInterrupted => ArtifactRecoveryAction::ResumeWithAgent,
            Self::StorageUnavailable => ArtifactRecoveryAction::RetryThroughBridget,
        }
    }
}

/// Action sûre proposée à l'opérateur après un échec. Une action n'accorde
/// aucune capacité au renderer : elle renvoie toujours vers Bridget.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRecoveryAction {
    RetryThroughBridget,
    RestoreThroughBridget,
    ReviewRetention,
    InspectManifest,
    ResumeWithAgent,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactFailureReceiptV1 {
    pub code: ArtifactFailureCode,
    pub recovery_action: ArtifactRecoveryAction,
    pub message: String,
}

impl ArtifactFailureReceiptV1 {
    pub fn new(code: ArtifactFailureCode, message: impl Into<String>) -> Self {
        Self {
            code,
            recovery_action: code.recovery_action(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactValidationLimits {
    pub manifest_max_bytes: usize,
    pub structured_payload_max_bytes: usize,
    pub max_sources: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ArtifactValidationError {
    InvalidIdempotencyKey,
    InvalidTitle,
    PayloadTooLarge,
    TooManySources,
    MissingProvenance,
    InvalidSource(&'static str),
    MissingParent,
    UnexpectedParent,
    ManifestTooLarge,
}

impl std::fmt::Display for ArtifactValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdempotencyKey => formatter.write_str("clé d'idempotence invalide"),
            Self::InvalidTitle => formatter.write_str("titre d'artefact invalide"),
            Self::PayloadTooLarge => formatter.write_str("payload d'artefact trop volumineux"),
            Self::TooManySources => formatter.write_str("trop de sources d'artefact"),
            Self::MissingProvenance => formatter.write_str("provenance d'artefact absente"),
            Self::InvalidSource(reason) => {
                write!(formatter, "source d'artefact invalide: {reason}")
            }
            Self::MissingParent => formatter.write_str("référence parent obligatoire"),
            Self::UnexpectedParent => formatter.write_str("référence parent inattendue"),
            Self::ManifestTooLarge => formatter.write_str("manifeste d'artefact trop volumineux"),
        }
    }
}

impl std::error::Error for ArtifactValidationError {}

impl ArtifactPublicationV1 {
    pub fn validate(
        &self,
        limits: ArtifactValidationLimits,
    ) -> Result<(), ArtifactValidationError> {
        if !valid_token(&self.idempotency_key, MAX_IDEMPOTENCY_KEY_BYTES) {
            return Err(ArtifactValidationError::InvalidIdempotencyKey);
        }
        if !valid_human_text(&self.title, MAX_TITLE_BYTES) {
            return Err(ArtifactValidationError::InvalidTitle);
        }
        let payload = canonical_json_bytes(&self.payload);
        if payload.len() > limits.structured_payload_max_bytes {
            return Err(ArtifactValidationError::PayloadTooLarge);
        }
        if self.sources.is_empty() {
            return Err(ArtifactValidationError::MissingProvenance);
        }
        if self.sources.len() > limits.max_sources {
            return Err(ArtifactValidationError::TooManySources);
        }
        for source in &self.sources {
            source.validate()?;
        }
        if self
            .quality_notices
            .iter()
            .any(|notice| !valid_human_text(notice, MAX_QUALITY_NOTICE_BYTES))
        {
            return Err(ArtifactValidationError::InvalidSource("avis de qualité"));
        }
        match self.publication_reason {
            PublicationReason::Initial if self.parent_artifact_ref.is_some() => {
                Err(ArtifactValidationError::UnexpectedParent)
            }
            PublicationReason::Initial => Ok(()),
            _ if self
                .parent_artifact_ref
                .as_deref()
                .is_some_and(|reference| valid_token(reference, 160)) =>
            {
                Ok(())
            }
            _ => Err(ArtifactValidationError::MissingParent),
        }?;
        if self.canonical_bytes().len() > limits.manifest_max_bytes {
            return Err(ArtifactValidationError::ManifestTooLarge);
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_json_bytes(&serde_json::to_value(self).expect("publication sérialisable"))
    }

    pub fn content_digest(&self) -> String {
        sha256_hex(&self.canonical_bytes())
    }
}

impl ArtifactSourceV1 {
    fn validate(&self) -> Result<(), ArtifactValidationError> {
        if !valid_human_text(&self.locator, 2_048) || !valid_human_text(&self.citation, 512) {
            return Err(ArtifactValidationError::InvalidSource(
                "locator ou citation",
            ));
        }
        if self.source_kind == ArtifactSourceKind::Remote && !self.locator.starts_with("https://") {
            return Err(ArtifactValidationError::InvalidSource(
                "URL distante non HTTPS",
            ));
        }
        if self.source_kind.requires_fetched_at() && self.fetched_at.is_none() {
            return Err(ArtifactValidationError::InvalidSource(
                "date de collecte absente",
            ));
        }
        if self
            .content_digest
            .as_deref()
            .is_some_and(|digest| !is_sha256_hex(digest))
        {
            return Err(ArtifactValidationError::InvalidSource("empreinte"));
        }
        if self
            .units
            .as_deref()
            .is_some_and(|units| !valid_human_text(units, 128))
        {
            return Err(ArtifactValidationError::InvalidSource("unité"));
        }
        if self.transformations.len() > 128
            || self
                .transformations
                .iter()
                .any(|value| !valid_human_text(value, 512))
        {
            return Err(ArtifactValidationError::InvalidSource("transformations"));
        }
        Ok(())
    }
}

pub fn canonical_json_bytes(value: &Value) -> Vec<u8> {
    let canonical = canonicalize(value);
    serde_json::to_vec(&canonical).expect("valeur JSON sérialisable")
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        Value::Object(values) => {
            let mut ordered = std::collections::BTreeMap::new();
            for (key, value) in values {
                ordered.insert(key.clone(), canonicalize(value));
            }
            Value::Object(ordered.into_iter().collect())
        }
        _ => value.clone(),
    }
}

fn valid_token(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_human_text(value: &str, max_bytes: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max_bytes && !value.chars().any(char::is_control)
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn limits() -> ArtifactValidationLimits {
        ArtifactValidationLimits {
            manifest_max_bytes: 8 * 1024,
            structured_payload_max_bytes: 4 * 1024,
            max_sources: 2,
        }
    }

    fn source() -> ArtifactSourceV1 {
        ArtifactSourceV1 {
            source_kind: ArtifactSourceKind::Remote,
            locator: "https://example.test/source.json".to_string(),
            fetched_at: Some(1_700_000_000),
            content_digest: None,
            citation: "Source de recette".to_string(),
            units: Some("unités".to_string()),
            transformations: vec!["normalisation".to_string()],
            access_status: ArtifactSourceAccess::Available,
        }
    }

    fn publication() -> ArtifactPublicationV1 {
        ArtifactPublicationV1 {
            idempotency_key: "artifact-001".to_string(),
            kind: ArtifactKind::Chart,
            title: "Évolution".to_string(),
            payload: json!({"series": [{"points": [1, 2]}]}),
            sources: vec![source()],
            quality_notices: Vec::new(),
            parent_artifact_ref: None,
            publication_reason: PublicationReason::Initial,
        }
    }

    #[test]
    fn publication_valide_exige_une_source_utile() {
        assert!(publication().validate(limits()).is_ok());
        let mut invalid = publication();
        invalid.sources.clear();
        assert_eq!(
            invalid.validate(limits()),
            Err(ArtifactValidationError::MissingProvenance)
        );
    }

    #[test]
    fn publication_non_initiale_exige_un_parent() {
        let mut value = publication();
        value.publication_reason = PublicationReason::Refresh;
        assert_eq!(
            value.validate(limits()),
            Err(ArtifactValidationError::MissingParent)
        );
        value.parent_artifact_ref = Some("artifact-123".to_string());
        assert!(value.validate(limits()).is_ok());
    }

    #[test]
    fn digest_ne_depend_pas_de_l_ordre_des_objets_json() {
        let left = json!({"a": 1, "b": {"x": true, "y": false}});
        let right = json!({"b": {"y": false, "x": true}, "a": 1});
        assert_eq!(canonical_json_bytes(&left), canonical_json_bytes(&right));
        assert_eq!(
            sha256_hex(&canonical_json_bytes(&left)),
            sha256_hex(&canonical_json_bytes(&right))
        );
    }

    #[test]
    fn fixtures_de_publication_respectent_le_contrat_v1() {
        for fixture in [
            include_str!("../tests/fixtures/artifacts/chart-external-v1.json"),
            include_str!("../tests/fixtures/artifacts/chart-partial-v1.json"),
            include_str!("../tests/fixtures/artifacts/image-v1.json"),
            include_str!("../tests/fixtures/artifacts/file-v1.json"),
            include_str!("../tests/fixtures/artifacts/generated-provenance-v1.json"),
        ] {
            let publication: ArtifactPublicationV1 =
                serde_json::from_str(fixture).expect("fixture conforme au schéma fermé");
            assert!(publication.validate(limits()).is_ok());
        }
    }
}
