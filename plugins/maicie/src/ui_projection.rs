//! Projection de lecture versionnée destinée aux interfaces Maicie.
//!
//! Cette frontière est volontairement une API de bibliothèque : un relais ne
//! doit jamais analyser le rendu de ligne de commande, ni ouvrir directement
//! la base SQLite privée de Maicie.

use crate::config::{ConfigError, MaicieConfig};
use crate::domain::{DecisionCoordination, Delegation, ObjectifCoordonne};
use crate::store::{MaicieStore, RemiseLocale, StoreError};
use serde::Serialize;
use std::fmt;
use std::path::Path;

pub const UI_MISSION_PROJECTION_VERSION: u8 = 1;

/// Instantané métier local, sans observation Bridget déduite ou cachée.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiMissionProjectionV1 {
    pub version: u8,
    pub objectives: Vec<UiObjectiveProjection>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UiObjectiveProjection {
    pub objective: ObjectifCoordonne,
    pub delegations: Vec<Delegation>,
    pub decisions: Vec<DecisionCoordination>,
    pub local_deliveries: Vec<RemiseLocale>,
}

#[derive(Debug)]
pub enum UiProjectionError {
    Config(ConfigError),
    Store(StoreError),
}

impl fmt::Display for UiProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "configuration Maicie invalide: {error}"),
            Self::Store(error) => write!(formatter, "projection Maicie indisponible: {error}"),
        }
    }
}

impl std::error::Error for UiProjectionError {}

/// Lit l'état durable Maicie depuis une configuration explicite.
///
/// Le chemin de configuration est délibérément requis par l'appelant : aucune
/// interface ne déduit un emplacement de base ou ne lit une sortie CLI.
pub fn read_ui_mission_projection_v1(
    config_path: impl AsRef<Path>,
) -> Result<UiMissionProjectionV1, UiProjectionError> {
    let config = MaicieConfig::load(config_path).map_err(UiProjectionError::Config)?;
    let store = MaicieStore::open(&config.database_path).map_err(UiProjectionError::Store)?;
    let objectives = store
        .objective_snapshots(None)
        .map_err(UiProjectionError::Store)?
        .into_iter()
        .map(|snapshot| UiObjectiveProjection {
            objective: snapshot.objective,
            delegations: snapshot.delegations,
            decisions: snapshot.decisions,
            local_deliveries: snapshot.remises_locales,
        })
        .collect();
    Ok(UiMissionProjectionV1 {
        version: UI_MISSION_PROJECTION_VERSION,
        objectives,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MaicieStore;

    #[test]
    fn projection_ui_v1_expose_les_octets_durables_sans_rendu_cli() {
        let path = std::env::temp_dir().join(format!(
            "maicie-ui-projection-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let store = MaicieStore::open(&path).unwrap();
        let projection = UiMissionProjectionV1 {
            version: UI_MISSION_PROJECTION_VERSION,
            objectives: store
                .objective_snapshots(None)
                .unwrap()
                .into_iter()
                .map(|snapshot| UiObjectiveProjection {
                    objective: snapshot.objective,
                    delegations: snapshot.delegations,
                    decisions: snapshot.decisions,
                    local_deliveries: snapshot.remises_locales,
                })
                .collect(),
        };
        let json = serde_json::to_string(&projection).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"objectives\":[]"));
        drop(store);
        std::fs::remove_file(path).unwrap();
    }
}
