//! Journal de télémétrie Maicie, corrélé sans recopier les corps Bridget.
//!
//! Les événements restent volontairement factuels : une ligne associe un
//! objectif, une délégation et le message Bridget concerné, sans contenir le
//! texte d'une instruction ou d'une réponse.

use std::io::{self, Write};

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

/// Écrit une ligne JSONL complète puis la rend visible au lecteur.
///
/// La sérialisation est limitée au schéma [`TelemetryEvent`], qui ne possède
/// aucun champ permettant d'insérer un corps de message par défaut.
pub fn write_event(mut writer: impl Write, event: &TelemetryEvent) -> io::Result<()> {
    serde_json::to_writer(&mut writer, event).map_err(io::Error::other)?;
    writer.write_all(b"\n")?;
    writer.flush()
}
