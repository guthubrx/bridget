//! Contrat public de mission lu par Bridget.
//!
//! Ce module ne connaît ni la configuration ni les types privés de Maicie.
//! Il désérialise exclusivement l'instantané JSON atomique versionné.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

pub const MISSION_PROJECTION_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionProjectionV1 {
    pub version: u8,
    pub published_at: i64,
    pub objectives: Vec<MissionObjectiveV1>,
}

impl MissionProjectionV1 {
    pub fn empty() -> Self {
        Self {
            version: MISSION_PROJECTION_VERSION,
            published_at: 0,
            objectives: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionObjectiveV1 {
    pub objective_id: String,
    pub goal: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub delegations: Vec<MissionDelegationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionDelegationV1 {
    pub delegation_id: String,
    pub participant: String,
    pub instruction: String,
    pub state: String,
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub local_delivery_state: Option<String>,
    #[serde(default)]
    pub review: Option<MissionReviewV1>,
    #[serde(default)]
    pub execution: Option<MissionExecutionV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionReviewV1 {
    pub target_ref: String,
    pub reviewed_head: String,
    #[serde(default)]
    pub verdict: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionExecutionV1 {
    pub execution_id: String,
    pub agent_instance_id: String,
    pub provider_kind: String,
    #[serde(default)]
    pub provider_session_id: Option<String>,
    #[serde(default)]
    pub provider_turn_id: Option<String>,
    pub runtime_state: String,
    #[serde(default)]
    pub waiting_reason: Option<String>,
    pub observation_cursor: u64,
    pub freshness: String,
    pub observed_at: i64,
}

#[derive(Debug)]
pub enum MissionProjectionError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Version(u8),
}

impl fmt::Display for MissionProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "projection de mission indisponible: {error}"),
            Self::Json(error) => write!(formatter, "projection de mission illisible: {error}"),
            Self::Version(version) => write!(
                formatter,
                "version de projection de mission inconnue: {version}"
            ),
        }
    }
}

impl std::error::Error for MissionProjectionError {}

/// Reproduit uniquement la convention de nommage publique : aucun contenu de
/// configuration Maicie n'est lu par Bridget.
pub fn public_mission_projection_path(config_path: impl AsRef<Path>) -> PathBuf {
    let config_path = config_path.as_ref();
    let stem = config_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("maicie");
    config_path.with_file_name(format!("{stem}.mission-projection-v1.json"))
}

/// Une projection absente est un état dégradé explicite, pas une mission vide
/// attestée. L'appelant choisit l'affichage ou la posture d'attente adaptée.
pub fn read_public_mission_projection_v1(
    config_path: impl AsRef<Path>,
) -> Result<Option<MissionProjectionV1>, MissionProjectionError> {
    let path = public_mission_projection_path(config_path);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(MissionProjectionError::Io(error)),
    };
    let projection = serde_json::from_slice::<MissionProjectionV1>(&bytes)
        .map_err(MissionProjectionError::Json)?;
    if projection.version != MISSION_PROJECTION_VERSION {
        return Err(MissionProjectionError::Version(projection.version));
    }
    Ok(Some(projection))
}

pub fn retain_living_objectives(mut projection: MissionProjectionV1) -> MissionProjectionV1 {
    projection.objectives.retain(|item| item.state != "clos");
    projection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absence_de_projection_est_degradee_sans_inventer_de_mission() {
        let path = std::env::temp_dir().join(format!(
            "bridget-projection-absente-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        assert_eq!(read_public_mission_projection_v1(&path).unwrap(), None);
    }

    #[test]
    fn version_inconnue_est_refusee_explicitement() {
        let root = std::env::temp_dir().join(format!(
            "bridget-projection-version-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let config = root.join("config.json");
        let projection = public_mission_projection_path(&config);
        std::fs::write(
            &projection,
            r#"{"version":2,"published_at":0,"objectives":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            read_public_mission_projection_v1(&config),
            Err(MissionProjectionError::Version(2))
        ));
        let _ = std::fs::remove_dir_all(root);
    }
}
