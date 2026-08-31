//! Projection locale de flotte, réduite avant de franchir l'IPC Tauri.
//!
//! Les réponses relayées peuvent contenir des chemins de projets et d'autres
//! détails d'origine. Ce module les lit seulement pour joindre projet et agent,
//! puis ne conserve que les champs nécessaires à la navigation Desktop.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const REMOTE_UI_VERSION: u8 = 1;
pub const LOCAL_SOURCE_ID: &str = "local";
pub const LOCAL_SOURCE_LABEL: &str = "Cet ordinateur";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Ssh,
    Local,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FleetSourceInput {
    pub source_id: String,
    pub label: String,
    pub kind: SourceKind,
    pub connection_state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DesktopFleetSnapshotV1 {
    pub version: u8,
    pub sources: Vec<DesktopFleetSource>,
    pub agents: Vec<DesktopFleetAgent>,
}

impl DesktopFleetSnapshotV1 {
    pub fn from_sources(sources: Vec<SourceProjection>) -> Self {
        let agents = sources
            .iter()
            .flat_map(|projection| projection.agents.iter().cloned())
            .collect();
        let sources = sources
            .into_iter()
            .map(|projection| projection.source)
            .collect();
        Self {
            version: 1,
            sources,
            agents,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DesktopFleetSource {
    pub source_id: String,
    pub label: String,
    pub kind: SourceKind,
    pub connection_state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub projects: Vec<DesktopFleetProject>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DesktopFleetProject {
    pub project_id: String,
    pub display_name: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DesktopFleetAgent {
    pub key: String,
    pub source_id: String,
    pub source_label: String,
    pub name: String,
    pub display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_state: Option<String>,
    pub alerts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_at: Option<i64>,
    pub unread: usize,
    pub is_coordinator: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceProjection {
    pub source: DesktopFleetSource,
    pub agents: Vec<DesktopFleetAgent>,
}

#[derive(Debug)]
pub enum FleetProjectionError {
    Snapshot,
    Projects,
}

impl std::fmt::Display for FleetProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Snapshot => "La réponse de flotte de cette source est illisible.",
            Self::Projects => "La liste des projets de cette source est illisible.",
        })
    }
}

impl std::error::Error for FleetProjectionError {}

#[derive(Deserialize)]
struct RemoteSnapshot {
    version: u8,
    agents: Vec<RemoteAgent>,
}

#[derive(Deserialize)]
struct RemoteAgent {
    name: String,
    #[serde(default)]
    profile: Option<RemoteProfile>,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    state: String,
    #[serde(default)]
    wait_state: Option<String>,
    #[serde(default)]
    alerts: Vec<String>,
    #[serde(default)]
    last_message_at: Option<i64>,
    #[serde(default)]
    unread: usize,
    #[serde(default)]
    agent_link: Option<RemoteAgentLink>,
}

#[derive(Deserialize)]
struct RemoteProfile {
    display_name: String,
}

#[derive(Deserialize)]
struct RemoteAgentLink {
    role: String,
}

#[derive(Deserialize)]
struct RemoteProjects {
    version: u8,
    projects: Vec<RemoteProject>,
}

#[derive(Deserialize)]
struct RemoteProject {
    project_id: String,
    display_name: String,
    state: String,
}

/// Réduit une réponse source possédée. Les octets sont lus hors des verrous de
/// connexion : le calcul ne bloque donc jamais le contrôle des tunnels.
pub fn project_source(
    input: FleetSourceInput,
    snapshot_body: &[u8],
    projects_body: &[u8],
) -> Result<SourceProjection, FleetProjectionError> {
    let snapshot: RemoteSnapshot =
        serde_json::from_slice(snapshot_body).map_err(|_| FleetProjectionError::Snapshot)?;
    let projects: RemoteProjects =
        serde_json::from_slice(projects_body).map_err(|_| FleetProjectionError::Projects)?;
    if snapshot.version != REMOTE_UI_VERSION {
        return Err(FleetProjectionError::Snapshot);
    }
    if projects.version != REMOTE_UI_VERSION {
        return Err(FleetProjectionError::Projects);
    }
    let project_names = projects
        .projects
        .iter()
        .map(|project| (project.project_id.clone(), project.display_name.clone()))
        .collect::<HashMap<_, _>>();
    let source = DesktopFleetSource {
        source_id: input.source_id.clone(),
        label: input.label.clone(),
        kind: input.kind,
        connection_state: input.connection_state,
        error: None,
        projects: projects
            .projects
            .into_iter()
            .map(|project| DesktopFleetProject {
                project_id: project.project_id,
                display_name: project.display_name,
                state: project.state,
            })
            .collect(),
    };
    let agents = snapshot
        .agents
        .into_iter()
        .map(|agent| {
            let project_name = agent
                .project_id
                .as_deref()
                .and_then(|project_id| project_names.get(project_id).cloned());
            DesktopFleetAgent {
                key: format!("{}:{}", input.source_id, agent.name),
                source_id: input.source_id.clone(),
                source_label: input.label.clone(),
                display_name: agent
                    .profile
                    .map(|profile| profile.display_name)
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or_else(|| agent.name.clone()),
                name: agent.name,
                project_id: agent.project_id,
                project_name,
                state: agent.state,
                wait_state: agent.wait_state,
                alerts: agent.alerts,
                last_message_at: agent.last_message_at,
                unread: agent.unread,
                is_coordinator: agent
                    .agent_link
                    .as_ref()
                    .is_some_and(|link| link.role == "coordinator"),
            }
        })
        .collect();
    Ok(SourceProjection { source, agents })
}

pub fn unavailable_source(
    mut input: FleetSourceInput,
    error: impl Into<String>,
) -> SourceProjection {
    input.connection_state = "failed".to_owned();
    source_without_snapshot(input, Some(error.into()))
}

pub fn source_without_snapshot(input: FleetSourceInput, error: Option<String>) -> SourceProjection {
    SourceProjection {
        source: DesktopFleetSource {
            source_id: input.source_id,
            label: input.label,
            kind: input.kind,
            connection_state: input.connection_state,
            error,
            projects: Vec::new(),
        },
        agents: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_path_n_est_pas_un_champ_de_sortie() {
        let source = FleetSourceInput {
            source_id: "one".into(),
            label: "One".into(),
            kind: SourceKind::Ssh,
            connection_state: "connected".into(),
        };
        let projection = project_source(
            source,
            br#"{ "version": 1, "agents": [] }"#,
            br#"{ "version": 1, "projects": [{ "project_id": "p", "display_name": "P", "state": "active", "canonical_path": "/private" }] }"#,
        )
        .unwrap();
        let encoded =
            serde_json::to_string(&DesktopFleetSnapshotV1::from_sources(vec![projection])).unwrap();
        assert!(!encoded.contains("canonical_path"));
        assert!(!encoded.contains("/private"));
    }
}
