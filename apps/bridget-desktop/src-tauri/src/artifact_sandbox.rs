//! Contrat local du runtime d'artefacts HTML non fiables.
//!
//! Ce module ne rend ni ne charge de contenu : il conserve les bornes et les
//! tickets qui empêchent le cadre HTML de devenir une seconde surface de
//! confiance. Les identifiants pointent vers les versions canoniques du daemon.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

pub const MAX_INLINE_HEIGHT: u16 = 1_200;
pub const MAX_UI_STATE_BYTES: usize = 128 * 1024;
pub const SANDBOX_TICKET_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SandboxArtifactVersionV1 {
    pub artifact_id: Uuid,
    pub version_id: Uuid,
    pub html_blob_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_blob_sha256: Option<String>,
    pub runtime_policy: SandboxRuntimePolicyV1,
    pub inline_height_hint: u16,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_version_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxRuntimePolicyV1 {
    SandboxV1,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SandboxRuntimeStateV1 {
    pub frame_instance_id: Uuid,
    pub artifact_id: Uuid,
    pub version_id: Uuid,
    pub ui_state: Value,
    pub requested_height: u16,
    pub status: SandboxRuntimeStatusV1,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxRuntimeStatusV1 {
    Ready,
    Degraded,
    Blocked,
    Closed,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BrowserNavigationRequestV1 {
    pub request_id: Uuid,
    pub initiator: BrowserNavigationInitiatorV1,
    pub target_kind: BrowserNavigationTargetKindV1,
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gesture_at_unix_ms: Option<u64>,
    pub decision: BrowserNavigationDecisionV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserNavigationInitiatorV1 {
    Operator,
    ConversationLink,
    ArtifactSource,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserNavigationTargetKindV1 {
    HttpsUrl,
    PublishedLocal,
    ArtifactVersion,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserNavigationDecisionV1 {
    Opened,
    Blocked,
    Cancelled,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxTicketV1 {
    pub ticket_id: Uuid,
    pub artifact_id: Uuid,
    pub version_id: Uuid,
    pub expires_at: SystemTime,
}

impl SandboxTicketV1 {
    pub fn issue(artifact_id: Uuid, version_id: Uuid, now: SystemTime) -> Self {
        Self {
            ticket_id: Uuid::new_v4(),
            artifact_id,
            version_id,
            expires_at: now + SANDBOX_TICKET_TTL,
        }
    }

    pub fn accepts(&self, artifact_id: Uuid, version_id: Uuid, now: SystemTime) -> bool {
        self.artifact_id == artifact_id && self.version_id == version_id && now <= self.expires_at
    }
}

pub fn validate_runtime_state(state: &SandboxRuntimeStateV1) -> Result<(), &'static str> {
    if state.requested_height > MAX_INLINE_HEIGHT {
        return Err("La hauteur demandée dépasse 1 200 px.");
    }
    let serialized = serde_json::to_vec(&state.ui_state).map_err(|_| "État JSON invalide.")?;
    if serialized.len() > MAX_UI_STATE_BYTES {
        return Err("L'état temporaire dépasse 128 Kio.");
    }
    if contains_url(&state.ui_state) {
        return Err("L'état temporaire ne peut pas introduire d'URL.");
    }
    Ok(())
}

fn contains_url(value: &Value) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| key.to_ascii_lowercase().contains("url") || contains_url(value)),
        Value::Array(values) => values.iter().any(contains_url),
        Value::String(value) => {
            value.starts_with("http:") || value.starts_with("https:") || value.starts_with("file:")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_etat_et_hauteur_restent_bornes() {
        let artifact = Uuid::new_v4();
        let version = Uuid::new_v4();
        let now = SystemTime::now();
        let ticket = SandboxTicketV1::issue(artifact, version, now);
        assert!(ticket.accepts(artifact, version, now));
        assert!(!ticket.accepts(Uuid::new_v4(), version, now));
        let state = SandboxRuntimeStateV1 {
            frame_instance_id: Uuid::new_v4(),
            artifact_id: artifact,
            version_id: version,
            ui_state: serde_json::json!({"filter": "2026"}),
            requested_height: 400,
            status: SandboxRuntimeStatusV1::Ready,
        };
        assert!(validate_runtime_state(&state).is_ok());
        assert!(
            validate_runtime_state(&SandboxRuntimeStateV1 {
                requested_height: 1201,
                ..state
            })
            .is_err()
        );
    }
}
