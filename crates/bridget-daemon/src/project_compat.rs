//! Représentations historiques du projet, conservées pour lire les données et leurs portées.
//! Aucun lancement, chemin de ressources ou accès système ne vit dans ce module.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEnvironmentState {
    Absent,
    Creating,
    Ready,
    Running,
    Stopping,
    Stopped,
    Degraded,
    RecreateRequired,
}

/// Valeur fermée du réglage expert. Le défaut explicite évite qu'une migration
/// ou une absence de ligne transforme un ancien serveur en serveur inscriptible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DogfoodingBridgetMode {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DogfoodingBridgetState {
    pub project_id: String,
    pub binding_generation: u64,
    pub setting_generation: u64,
    pub mode: DogfoodingBridgetMode,
}

#[cfg(test)]
mod tests {
    use super::ProjectEnvironmentState;

    #[test]
    fn etats_historiques_conservent_les_octets_json_sans_admission_runtime() {
        // Corpus indépendant de l'encodeur : un renommage ou changement de
        // représentation casse la lecture des données pré-extraction.
        for (state, bytes) in [
            (ProjectEnvironmentState::Absent, b"\"absent\"".as_slice()),
            (
                ProjectEnvironmentState::Creating,
                b"\"creating\"".as_slice(),
            ),
            (ProjectEnvironmentState::Ready, b"\"ready\"".as_slice()),
            (ProjectEnvironmentState::Running, b"\"running\"".as_slice()),
            (
                ProjectEnvironmentState::Stopping,
                b"\"stopping\"".as_slice(),
            ),
            (ProjectEnvironmentState::Stopped, b"\"stopped\"".as_slice()),
            (
                ProjectEnvironmentState::Degraded,
                b"\"degraded\"".as_slice(),
            ),
            (
                ProjectEnvironmentState::RecreateRequired,
                b"\"recreate_required\"".as_slice(),
            ),
        ] {
            assert_eq!(serde_json::to_vec(&state).unwrap(), bytes);
            assert_eq!(
                serde_json::from_slice::<ProjectEnvironmentState>(bytes).unwrap(),
                state
            );
        }
        assert!(serde_json::from_str::<ProjectEnvironmentState>("\"future\"").is_err());
    }
}
