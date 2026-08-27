//! Projection de lecture versionnée destinée aux interfaces Maicie.
//!
//! Cette frontière est volontairement une API de bibliothèque : un relais ne
//! doit jamais analyser le rendu de ligne de commande, ni ouvrir directement
//! la base SQLite privée de Maicie.

use crate::config::{ConfigError, MaicieConfig};
use crate::domain::{DecisionCoordination, Delegation, ObjectifCoordonne};
use crate::review_continuity::StoredReviewVerdict;
use crate::store::{MaicieStore, RemiseLocale, StoreError};
use serde::Serialize;
use std::collections::BTreeMap;
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
    /// Verdicts terminaux relus depuis les octets du guichet. Leur continuité
    /// Git reste une observation de la surface consommatrice.
    pub review_verdicts: Vec<StoredReviewVerdict>,
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
    let mut verdicts_by_objective = BTreeMap::new();
    for verdict in store.review_verdicts().map_err(UiProjectionError::Store)? {
        verdicts_by_objective
            .entry(verdict.objective_id)
            .or_insert_with(Vec::new)
            .push(verdict);
    }
    let objectives = store
        .objective_snapshots(None)
        .map_err(UiProjectionError::Store)?
        .into_iter()
        .map(|snapshot| UiObjectiveProjection {
            review_verdicts: verdicts_by_objective
                .remove(&snapshot.objective.id)
                .unwrap_or_default(),
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

/// Pour l'instantané page : n'envoie que les objectifs encore vivants.
/// L'historique `clos` reste dans le greffe ; la page ne l'affiche pas.
pub fn retain_living_objectives(mut projection: UiMissionProjectionV1) -> UiMissionProjectionV1 {
    use crate::domain::EtatObjectif;
    projection
        .objectives
        .retain(|item| item.objective.etat != EtatObjectif::Clos);
    projection
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EtatObjectif, ModeObjectif};
    use crate::store::MaicieStore;
    use uuid::Uuid;

    fn objective(etat: EtatObjectif, but: &str) -> UiObjectiveProjection {
        UiObjectiveProjection {
            objective: ObjectifCoordonne {
                id: Uuid::new_v4(),
                but: but.to_string(),
                mode: ModeObjectif::Delegue,
                etat,
                cree_at: 1,
                mis_a_jour_at: 1,
                synthese: None,
                decision_en_attente_id: None,
                suite: None,
                depends_on: Vec::new(),
                references: Vec::new(),
            },
            delegations: Vec::new(),
            decisions: Vec::new(),
            local_deliveries: Vec::new(),
            review_verdicts: Vec::new(),
        }
    }

    #[test]
    fn projection_ui_v1_expose_les_octets_durables_sans_rendu_cli() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "maicie-ui-projection-root-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = root.join("store.sqlite");
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
                    review_verdicts: Vec::new(),
                })
                .collect(),
        };
        let json = serde_json::to_string(&projection).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"objectives\":[]"));
        drop(store);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn instantane_page_n_envoie_que_les_objectifs_vivants() {
        let living_but = "mandat vivant unique-temoin";
        let closed_but = "historique clos ne-doit-pas-partir";
        let projection = UiMissionProjectionV1 {
            version: UI_MISSION_PROJECTION_VERSION,
            objectives: vec![
                objective(EtatObjectif::Clos, closed_but),
                objective(EtatObjectif::EnCoordination, living_but),
                objective(EtatObjectif::Clos, "autre clos"),
                objective(EtatObjectif::AEvaluer, "a evaluer vivant"),
            ],
        };
        let filtered = retain_living_objectives(projection);
        assert_eq!(filtered.objectives.len(), 2);
        assert!(
            filtered
                .objectives
                .iter()
                .all(|item| item.objective.etat != EtatObjectif::Clos)
        );
        assert!(
            filtered
                .objectives
                .iter()
                .any(|item| item.objective.but == living_but)
        );
        assert!(
            !filtered
                .objectives
                .iter()
                .any(|item| item.objective.but == closed_but)
        );
    }
}
