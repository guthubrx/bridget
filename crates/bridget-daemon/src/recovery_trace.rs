//! Trace durable des équipiers qui ne reviennent pas après un redémarrage.
//!
//! Fichier voisin de `fleet.json` : lisible par `bridget reprise` et par la
//! page. Le cas zéro perte n'écrit pas le fichier (aucun bruit).
//!
//! `named-roster.json` n'est pas un snapshot d'équipe à ressusciter : il ne
//! sert qu'à nommer les absents non persistants, invisibles de `fleet.json`.

use bridget_transport::fsutil::write_private_file_atomic;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const REPORT_FILE_NAME: &str = "recovery-losses.json";
pub const ROSTER_FILE_NAME: &str = "named-roster.json";
pub const TRACE_SCHEMA_VERSION: u64 = 1;

pub const REASON_NON_PERSISTENT: &str = "non_persistant";
pub const REASON_QUOTA: &str = "quota_flotte";
pub const REASON_FROZEN_DEFINITION: &str = "definition_figee_absente";
pub const REASON_RECOVERY_FAILED: &str = "reprise_refusee";
pub const REASON_ABSENT_FROM_FLEET: &str = "absent_de_fleet";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryLossEntry {
    pub name: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryLossReport {
    pub schema: u64,
    pub recorded_at: i64,
    pub absents: Vec<RecoveryLossEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedRosterEntry {
    #[serde(rename = "type")]
    pub agent_type: String,
    pub persistent: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct NamedRosterFile {
    #[serde(default)]
    schema: u64,
    #[serde(default)]
    named: BTreeMap<String, NamedRosterEntry>,
}

pub fn report_path(fleet_path: &Path) -> PathBuf {
    fleet_path.with_file_name(REPORT_FILE_NAME)
}

pub fn roster_path(fleet_path: &Path) -> PathBuf {
    fleet_path.with_file_name(ROSTER_FILE_NAME)
}

pub fn load_report(path: &Path) -> io::Result<Option<RecoveryLossReport>> {
    match fs::read(path) {
        Ok(bytes) => {
            let report: RecoveryLossReport = serde_json::from_slice(&bytes)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            if report.absents.is_empty() {
                Ok(None)
            } else {
                Ok(Some(report))
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Écrit la trace si et seulement s'il y a des absents. Zéro perte : le
/// fichier précédent est retiré, rien n'est posé à la place.
pub fn persist_report(
    path: &Path,
    recorded_at: i64,
    mut absents: Vec<RecoveryLossEntry>,
) -> io::Result<()> {
    absents.sort_by(|left, right| left.name.cmp(&right.name));
    absents.dedup_by(|left, right| left.name == right.name);
    if absents.is_empty() {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    } else {
        let report = RecoveryLossReport {
            schema: TRACE_SCHEMA_VERSION,
            recorded_at,
            absents,
        };
        let mut bytes = serde_json::to_vec_pretty(&report)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        bytes.push(b'\n');
        write_private_file_atomic(path, &bytes)
    }
}

pub struct NamedRosterStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl NamedRosterStore {
    pub fn at_path(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn remember(&self, name: String, entry: NamedRosterEntry) {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut file = self.load_unlocked();
        file.named.insert(name, entry);
        self.persist_unlocked(&file);
    }

    pub fn forget(&self, name: &str) {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut file = self.load_unlocked();
        if file.named.remove(name).is_none() {
            return;
        }
        self.persist_unlocked(&file);
    }

    pub fn drain_non_persistent(&self) -> Vec<(String, NamedRosterEntry)> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut file = self.load_unlocked();
        let mut drained = Vec::new();
        file.named.retain(|name, entry| {
            if entry.persistent {
                true
            } else {
                drained.push((name.clone(), entry.clone()));
                false
            }
        });
        if !drained.is_empty() {
            self.persist_unlocked(&file);
        }
        drained
    }

    pub fn persistent_entries(&self) -> Vec<(String, NamedRosterEntry)> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        self.load_unlocked()
            .named
            .into_iter()
            .filter(|(_, entry)| entry.persistent)
            .collect()
    }

    /// Persistance attestée, par nom, en une seule lecture.
    ///
    /// Le roster est indexé par nom : chaque génération connectée écrase la
    /// précédente via `remember`. La dernière valeur est donc structurellement
    /// la courante — il n'y a pas de ligne périmée à trier, contrairement à
    /// l'historique `spawn_commands` où plusieurs générations d'un même nom
    /// coexistent. C'est aussi la source exacte que lit `drain_non_persistent` :
    /// ce qui est publié ici est ce qui décide du drain, sans second oracle.
    ///
    /// Complexité : O(n log n) pour n noms au roster, une seule E/S.
    pub fn persistence_by_name(&self) -> BTreeMap<String, bool> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        self.load_unlocked()
            .named
            .into_iter()
            .map(|(name, entry)| (name, entry.persistent))
            .collect()
    }

    fn load_unlocked(&self) -> NamedRosterFile {
        match fs::read(&self.path) {
            Ok(bytes) => match serde_json::from_slice(&bytes) {
                Ok(file) => file,
                Err(error) => {
                    warn!(
                        "named-roster illisible {}: {error} — reprise sans ce roster",
                        self.path.display()
                    );
                    NamedRosterFile {
                        schema: TRACE_SCHEMA_VERSION,
                        named: BTreeMap::new(),
                    }
                }
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => NamedRosterFile {
                schema: TRACE_SCHEMA_VERSION,
                named: BTreeMap::new(),
            },
            Err(error) => {
                warn!(
                    "named-roster inaccessible {}: {error} — reprise sans ce roster",
                    self.path.display()
                );
                NamedRosterFile {
                    schema: TRACE_SCHEMA_VERSION,
                    named: BTreeMap::new(),
                }
            }
        }
    }

    fn persist_unlocked(&self, file: &NamedRosterFile) {
        if file.named.is_empty() {
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => warn!("named-roster non retiré {}: {error}", self.path.display()),
            }
            return;
        }
        let payload = NamedRosterFile {
            schema: TRACE_SCHEMA_VERSION,
            named: file.named.clone(),
        };
        let bytes = match serde_json::to_vec_pretty(&payload) {
            Ok(mut bytes) => {
                bytes.push(b'\n');
                bytes
            }
            Err(error) => {
                warn!("named-roster non sérialisable: {error}");
                return;
            }
        };
        if let Err(error) = write_private_file_atomic(&self.path, &bytes) {
            warn!("named-roster non écrit {}: {error}", self.path.display());
        }
    }
}

/// Domaine minimal pour `fleet.json` : basename du cwd (même idée que le
/// dérivé hors dépôt git du wrapper). Suffit à recomposer le regroupement.
pub fn domain_from_cwd(cwd: &Path) -> Option<String> {
    cwd.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
}

pub fn resolved_domain(explicit: Option<&str>, cwd: &Path) -> Option<String> {
    explicit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| domain_from_cwd(cwd))
}

pub fn non_persistent_spawn_warning(name: &str) -> String {
    format!("{name} ne survivra pas au redémarrage (spawn sans --persistent)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-d20-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn zero_perte_n_ecrit_pas_de_bruit() {
        let root = temp_dir("zero");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(REPORT_FILE_NAME);
        fs::write(&path, "{}\n").unwrap();
        persist_report(&path, 1, Vec::new()).unwrap();
        assert!(!path.exists(), "zéro perte doit retirer le fichier");
        assert!(load_report(&path).unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn crash_simule_nomme_chaque_absent_avec_sa_raison() {
        let root = temp_dir("absents");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(REPORT_FILE_NAME);
        persist_report(
            &path,
            42,
            vec![
                RecoveryLossEntry {
                    name: "cursor3".to_string(),
                    reason: REASON_NON_PERSISTENT.to_string(),
                    detail: Some("spawn sans --persistent".to_string()),
                },
                RecoveryLossEntry {
                    name: "agent-09".to_string(),
                    reason: REASON_QUOTA.to_string(),
                    detail: Some("quota de flotte atteint (8)".to_string()),
                },
            ],
        )
        .unwrap();
        let report = load_report(&path).unwrap().expect("trace attendue");
        assert_eq!(report.absents.len(), 2);
        assert_eq!(report.absents[0].name, "agent-09");
        assert_eq!(report.absents[0].reason, REASON_QUOTA);
        assert_eq!(report.absents[1].name, "cursor3");
        assert_eq!(report.absents[1].reason, REASON_NON_PERSISTENT);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn roster_draine_les_non_persistants_et_garde_les_persistants() {
        let root = temp_dir("roster");
        fs::create_dir_all(&root).unwrap();
        let store = NamedRosterStore::at_path(root.join(ROSTER_FILE_NAME));
        store.remember(
            "ephemere".into(),
            NamedRosterEntry {
                agent_type: "cursor".into(),
                persistent: false,
                domain: Some("bridget".into()),
            },
        );
        store.remember(
            "durable".into(),
            NamedRosterEntry {
                agent_type: "cursor".into(),
                persistent: true,
                domain: Some("bridget".into()),
            },
        );
        let drained = store.drain_non_persistent();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].0, "ephemere");
        let again = store.drain_non_persistent();
        assert!(again.is_empty());
        let _ = fs::remove_dir_all(root);
    }

    /// ORACLE — la persistance publiée est celle de la DERNIÈRE génération
    /// connectée, jamais une antérieure.
    ///
    /// Le 28/08, la même question posée à l'historique `spawn_commands` groupé
    /// sans tri a rendu des lignes périmées : cinq agents vivants annoncés
    /// `persistent=0` alors qu'ils venaient d'être recréés avec `--persistent`.
    /// Le roster ne peut pas produire cette réponse : une seule entrée par nom,
    /// écrasée à chaque génération.
    #[test]
    fn persistance_publiee_est_celle_de_la_derniere_generation() {
        let root = temp_dir("roster-derniere-gen");
        fs::create_dir_all(&root).unwrap();
        let store = NamedRosterStore::at_path(root.join(ROSTER_FILE_NAME));
        // Génération n : lancé sans --persistent, l'erreur du 28/08.
        store.remember(
            "jc1-flux".into(),
            NamedRosterEntry {
                agent_type: "claude".into(),
                persistent: false,
                domain: Some("bridget".into()),
            },
        );
        // Génération n+1 : recréé avec --persistent, même nom.
        store.remember(
            "jc1-flux".into(),
            NamedRosterEntry {
                agent_type: "claude".into(),
                persistent: true,
                domain: Some("bridget".into()),
            },
        );
        store.remember(
            "ephemere".into(),
            NamedRosterEntry {
                agent_type: "claude".into(),
                persistent: false,
                domain: Some("bridget".into()),
            },
        );

        let persistence = store.persistence_by_name();
        assert_eq!(
            persistence.get("jc1-flux"),
            Some(&true),
            "la génération recréée avec --persistent doit primer sur la précédente"
        );
        assert_eq!(
            persistence.get("ephemere"),
            Some(&false),
            "un agent réellement non persistant doit rester lisible comme tel"
        );
        assert_eq!(
            persistence.get("agent-hors-flotte"),
            None,
            "un agent absent du roster n'a pas de persistance inventée"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn roster_illisible_se_degrade_sans_erreur() {
        let root = temp_dir("roster-ko");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(ROSTER_FILE_NAME);
        fs::write(&path, "ce n'est pas du json").unwrap();
        let store = NamedRosterStore::at_path(&path);
        assert!(store.drain_non_persistent().is_empty());
        assert!(store.persistent_entries().is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
