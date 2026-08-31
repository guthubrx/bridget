//! Projection de lecture versionnée destinée aux interfaces Maicie.
//!
//! Cette frontière est volontairement une API de bibliothèque : un relais ne
//! doit jamais analyser le rendu de ligne de commande, ni ouvrir directement
//! la base SQLite privée de Maicie.

use crate::config::{ConfigError, MaicieConfig};
use crate::runtime::project_binding_observation;
use crate::store::{MaicieStore, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const UI_MISSION_PROJECTION_VERSION: u8 = 1;

/// Contrat public, réduit et versionné pour les consommateurs Bridget.
///
/// Il ne sérialise jamais les tables métier Maicie : les états restent des
/// libellés fermés et les références d'exécution demeurent opaques.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiMissionProjectionV1 {
    pub version: u8,
    pub published_at: i64,
    pub objectives: Vec<UiObjectiveProjection>,
    #[serde(default)]
    pub projects: Vec<UiProjectProjection>,
}

/// État projet lisible par l'interface. Les deux statuts restent séparés :
/// Maicie décrit le métier, Bridget la liaison technique et sa fraîcheur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiProjectProjection {
    pub project_id: String,
    pub maicie_status: String,
    pub bridget_status: String,
    pub freshness: String,
    pub observed_at: i64,
    pub next_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiObjectiveProjection {
    pub objective_id: String,
    pub goal: String,
    pub state: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub delegations: Vec<UiDelegationProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDelegationProjection {
    pub delegation_id: String,
    pub participant: String,
    pub instruction: String,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_delivery_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<UiReviewProjection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<UiExecutionProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiReviewProjection {
    pub target_ref: String,
    pub reviewed_head: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiExecutionProjection {
    pub execution_id: String,
    pub agent_instance_id: String,
    pub provider_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_turn_id: Option<String>,
    pub runtime_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waiting_reason: Option<String>,
    pub observation_cursor: u64,
    pub freshness: String,
    pub observed_at: i64,
}

#[derive(Debug)]
pub enum UiProjectionError {
    Config(ConfigError),
    Store(StoreError),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for UiProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "configuration Maicie invalide: {error}"),
            Self::Store(error) => write!(formatter, "projection Maicie indisponible: {error}"),
            Self::Io(error) => write!(formatter, "projection Maicie non publiable: {error}"),
            Self::Json(error) => write!(formatter, "projection Maicie non sérialisable: {error}"),
        }
    }
}

impl std::error::Error for UiProjectionError {}

/// Emplacement public, déterministe et adjacent à la configuration explicitement
/// fournie. Bridget lit ce contrat, jamais la base SQLite Maicie.
pub fn public_mission_projection_path(config_path: impl AsRef<Path>) -> PathBuf {
    let config_path = config_path.as_ref();
    let stem = config_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("maicie");
    config_path.with_file_name(format!("{stem}.mission-projection-v1.json"))
}

/// Produit le contrat public par écriture temporaire puis renommage atomique.
/// Produit le contrat public par écriture temporaire puis renommage atomique.
pub fn publish_ui_mission_projection_v1(
    config_path: impl AsRef<Path>,
) -> Result<PathBuf, UiProjectionError> {
    let config_path = config_path.as_ref();
    let projection = read_ui_mission_projection_v1(config_path)?;
    let destination = public_mission_projection_path(config_path);
    write_public_mission_projection_v1(&destination, &projection)?;
    Ok(destination)
}

fn write_public_mission_projection_v1(
    destination: &Path,
    projection: &UiMissionProjectionV1,
) -> Result<(), UiProjectionError> {
    let payload = serde_json::to_vec(projection).map_err(UiProjectionError::Json)?;
    let parent = destination.parent().ok_or_else(|| {
        UiProjectionError::Io(std::io::Error::other("parent de projection absent"))
    })?;
    let filename = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            UiProjectionError::Io(std::io::Error::other("nom de projection invalide"))
        })?;
    let temporary = parent.join(format!(".{filename}.{}.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(UiProjectionError::Io)?;
    let write_result = (|| -> Result<(), UiProjectionError> {
        file.write_all(&payload).map_err(UiProjectionError::Io)?;
        file.sync_all().map_err(UiProjectionError::Io)?;
        fs::rename(&temporary, destination).map_err(UiProjectionError::Io)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}
fn json_label(value: &impl Serialize) -> Result<String, UiProjectionError> {
    match serde_json::to_value(value).map_err(UiProjectionError::Json)? {
        serde_json::Value::String(label) => Ok(label),
        value => Ok(value.to_string()),
    }
}

fn projection_execution(
    projection: crate::domain::ExecutionProjection,
) -> Result<UiExecutionProjection, UiProjectionError> {
    Ok(UiExecutionProjection {
        execution_id: projection.reference.execution_id,
        agent_instance_id: projection.reference.agent_instance_id,
        provider_kind: projection.reference.provider_kind,
        provider_session_id: projection.reference.provider_session_id,
        provider_turn_id: projection.reference.provider_turn_id,
        runtime_state: projection.runtime_state,
        waiting_reason: projection.waiting_reason,
        observation_cursor: projection.observation_cursor,
        freshness: json_label(&projection.freshness)?,
        observed_at: projection.observed_at,
    })
}

/// Lit l'état durable Maicie et le réduit au contrat public.
pub fn read_ui_mission_projection_v1(
    config_path: impl AsRef<Path>,
) -> Result<UiMissionProjectionV1, UiProjectionError> {
    let config = MaicieConfig::load(config_path).map_err(UiProjectionError::Config)?;
    let store = MaicieStore::open(&config.database_path).map_err(UiProjectionError::Store)?;
    let mut verdicts_by_delegation = BTreeMap::new();
    for verdict in store.review_verdicts().map_err(UiProjectionError::Store)? {
        verdicts_by_delegation.insert(verdict.delegation_id, verdict);
    }
    let mut objectives = Vec::new();
    for snapshot in store
        .objective_snapshots(None)
        .map_err(UiProjectionError::Store)?
    {
        let mut delegations = Vec::new();
        for delegation in snapshot.delegations {
            let delivery = snapshot
                .remises_locales
                .iter()
                .find(|delivery| delivery.delegation_id == delegation.id);
            let review = delegation
                .review_target
                .as_ref()
                .map(|target| {
                    Ok(UiReviewProjection {
                        target_ref: target.target_ref.clone(),
                        reviewed_head: target.expected_head.clone(),
                        verdict: verdicts_by_delegation
                            .get(&delegation.id)
                            .map(|verdict| json_label(&verdict.evidence.verdict))
                            .transpose()?,
                    })
                })
                .transpose()?;
            let execution = store
                .execution_projection_for_delegation(delegation.id)
                .map_err(UiProjectionError::Store)?
                .map(projection_execution)
                .transpose()?;
            delegations.push(UiDelegationProjection {
                delegation_id: delegation.id.to_string(),
                participant: delegation.participant,
                instruction: delegation.instruction,
                state: json_label(&delegation.etat)?,
                message_id: delivery.map(|delivery| delivery.message_id.to_string()),
                local_delivery_state: delivery
                    .map(|delivery| json_label(&delivery.state))
                    .transpose()?,
                review,
                execution,
            });
        }
        objectives.push(UiObjectiveProjection {
            objective_id: snapshot.objective.id.to_string(),
            goal: snapshot.objective.but,
            state: json_label(&snapshot.objective.etat)?,
            created_at: snapshot.objective.cree_at,
            updated_at: snapshot.objective.mis_a_jour_at,
            delegations,
        });
    }
    let projects = store
        .project_identities()
        .map_err(UiProjectionError::Store)?
        .into_iter()
        .map(|identity| {
            let observation = project_binding_observation(&identity, None);
            Ok(UiProjectProjection {
                project_id: observation.project_id,
                maicie_status: json_label(&observation.maicie_status)?,
                bridget_status: observation
                    .bridget_status
                    .map(|status| json_label(&status))
                    .transpose()?
                    .unwrap_or_else(|| "unavailable".to_string()),
                freshness: json_label(&observation.freshness)?,
                observed_at: observation.observed_at,
                next_action: observation.next_action.to_string(),
            })
        })
        .collect::<Result<Vec<_>, UiProjectionError>>()?;
    let published_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| UiProjectionError::Io(std::io::Error::other(error)))?
        .as_secs() as i64;
    Ok(UiMissionProjectionV1 {
        version: UI_MISSION_PROJECTION_VERSION,
        published_at,
        objectives,
        projects,
    })
}
/// Pour l'instantané page : n'envoie que les objectifs encore vivants.
/// L'historique `clos` reste dans le greffe ; la page ne l'affiche pas.
pub fn retain_living_objectives(mut projection: UiMissionProjectionV1) -> UiMissionProjectionV1 {
    projection.objectives.retain(|item| item.state != "clos");
    projection
}

/// Pour l'instantané page : n'envoie que les objectifs encore vivants.
/// L'historique `clos` reste dans le greffe ; la page ne l'affiche pas.
#[cfg(test)]
mod tests {
    use super::*;

    fn objective(state: &str, goal: &str) -> UiObjectiveProjection {
        UiObjectiveProjection {
            objective_id: format!("objective-{state}"),
            goal: goal.to_string(),
            state: state.to_string(),
            created_at: 1,
            updated_at: 2,
            delegations: Vec::new(),
        }
    }

    fn projection(objectives: Vec<UiObjectiveProjection>) -> UiMissionProjectionV1 {
        UiMissionProjectionV1 {
            version: UI_MISSION_PROJECTION_VERSION,
            published_at: 42,
            objectives,
            projects: Vec::new(),
        }
    }

    #[test]
    fn projection_ui_v1_est_reduite_au_contrat_public() {
        let encoded = serde_json::to_string(&projection(vec![objective("ouvert", "but")])).unwrap();
        assert!(encoded.contains("\"version\":1"));
        assert!(encoded.contains("\"published_at\":42"));
        assert!(encoded.contains("\"goal\":\"but\""));
        assert!(!encoded.contains("review_verdicts"));
        assert!(!encoded.contains("local_deliveries"));
        assert!(!encoded.contains("decisions"));
    }

    #[test]
    fn projection_ui_projet_expose_l_indisponibilite_sans_racine_hote() {
        let mut rendered = projection(Vec::new());
        rendered.projects.push(UiProjectProjection {
            project_id: "project-1".to_string(),
            maicie_status: "active".to_string(),
            bridget_status: "unavailable".to_string(),
            freshness: "unavailable".to_string(),
            observed_at: 12,
            next_action: "project status --project-id".to_string(),
        });
        let encoded = serde_json::to_string(&rendered).expect("projection sérialisable");
        assert!(encoded.contains("\"bridget_status\":\"unavailable\""));
        assert!(encoded.contains("\"freshness\":\"unavailable\""));
        assert!(!encoded.contains("/srv/"));
        assert!(!encoded.contains("canonical_root"));
    }

    #[test]
    fn ecriture_atomique_produit_un_json_complet_sans_temporaire() {
        let root = std::env::temp_dir().join(format!(
            "maicie-public-projection-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let destination = root.join("mission-projection-v1.json");
        let expected = projection(vec![objective("ouvert", "mission attestee")]);
        write_public_mission_projection_v1(&destination, &expected).unwrap();
        let actual: UiMissionProjectionV1 =
            serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
        assert_eq!(actual, expected);
        assert!(!root.join(".mission-projection-v1.json.tmp").exists());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn instantane_page_n_envoie_que_les_objectifs_vivants() {
        let living_goal = "mandat vivant unique-temoin";
        let closed_goal = "historique clos ne-doit-pas-partir";
        let filtered = retain_living_objectives(projection(vec![
            objective("clos", closed_goal),
            objective("en_coordination", living_goal),
            objective("clos", "autre clos"),
            objective("a_evaluer", "a evaluer vivant"),
        ]));
        assert_eq!(filtered.objectives.len(), 2);
        assert!(filtered.objectives.iter().all(|item| item.state != "clos"));
        assert!(
            filtered
                .objectives
                .iter()
                .any(|item| item.goal == living_goal)
        );
        assert!(
            !filtered
                .objectives
                .iter()
                .any(|item| item.goal == closed_goal)
        );
    }

    #[test]
    fn publication_depuis_configuration_ne_lit_que_le_magasin_maicie() {
        let root = std::env::temp_dir().join(format!(
            "maicie-projection-config-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        MaicieStore::open(&database).unwrap();
        let config = root.join("config.json");
        std::fs::write(
            &config,
            serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "bridget_socket": "/tmp/maicie-projection-test.sock",
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": []
            }))
            .unwrap(),
        )
        .unwrap();

        let destination = publish_ui_mission_projection_v1(&config).unwrap();
        let actual: UiMissionProjectionV1 =
            serde_json::from_slice(&std::fs::read(&destination).unwrap()).unwrap();
        assert_eq!(actual.version, UI_MISSION_PROJECTION_VERSION);
        assert!(actual.objectives.is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
