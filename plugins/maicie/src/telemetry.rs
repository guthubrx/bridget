//! Journal de télémétrie Maicie, corrélé sans recopier les corps Bridget.
//!
//! Les événements restent volontairement factuels : une ligne associe un
//! objectif, une délégation et le message Bridget concerné, sans contenir le
//! texte d'une instruction ou d'une réponse.

use std::{
    fs::{self, DirBuilder, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{EtatFlux, SourceSnapshot};

/// Nature factuelle de la ligne de télémétrie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryKind {
    Transition,
    Correlation,
    Decision,
}

/// Événement structuré, corrélé et sans champ de contenu libre.
///
/// Les identifiants sont obligatoires afin que chaque ligne soit rattachable
/// de manière stable à l'objectif, à la délégation et à la livraison Bridget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryEvent {
    pub observed_at: i64,
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub message_id: Uuid,
    pub source: SourceSnapshot,
    pub freshness: EtatFlux,
    pub event: TelemetryKind,
}

/// Journal durable de télémétrie appartenant exclusivement à Maicie.
pub struct TelemetryJournal {
    path: PathBuf,
    file: File,
}

impl TelemetryJournal {
    /// Ouvre un journal privé en append, sans accepter de chemin sans parent.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .ok_or_else(|| io::Error::other("journal de télémétrie sans répertoire parent"))?;
        ensure_private_directory(parent)?;

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(&path)?;
        ensure_private_file(&path)?;
        Ok(Self { path, file })
    }

    /// Ajoute une ligne complète, la vide vers le noyau puis la synchronise.
    pub fn append(&mut self, event: &TelemetryEvent) -> io::Result<()> {
        serde_json::to_writer(&mut self.file, event).map_err(io::Error::other)?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;
        self.file.sync_data()
    }

    /// Chemin du journal, exposé pour les lecteurs et les tests de reprise.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn ensure_private_directory(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(path)?;

    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "répertoire de télémétrie non privé",
        ));
    }
    Ok(())
}

fn ensure_private_file(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o777 != 0o600 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "fichier de télémétrie non privé",
        ));
    }
    Ok(())
}
