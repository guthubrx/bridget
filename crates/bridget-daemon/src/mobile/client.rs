//! Projections mobiles des lectures natives ; autorisation avant pagination.
use super::store::{Device, Operation};
use bridget_transport::{
    DaemonToWrapper,
    protocol::{CommunicationDirectoryScope, CommunicationProject, ScopedAgentInfo},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DirectoryQuery {
    pub machine: Option<String>,
    pub project_host: Option<String>,
    pub project_root: Option<String>,
    pub after: Option<String>,
    pub limit: Option<usize>,
    pub after_seq: Option<u64>,
    pub tail: Option<bool>,
    pub offset: Option<u32>,
    pub choice: Option<String>,
    pub agent_id: Option<String>,
}

impl DirectoryQuery {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if self.choice.as_ref().is_some_and(|s| s.len() > 1024)
            || self.limit.is_some_and(|n| n == 0 || n > 50)
            || self
                .machine
                .as_ref()
                .is_some_and(|s| s.len() > 256 || s.chars().any(char::is_control))
            || self.project_host.is_some() != self.project_root.is_some()
            || self.project_host.as_ref().is_some_and(|s| s.len() > 256)
            || self.project_root.as_ref().is_some_and(|s| s.len() > 4096)
            || self
                .after
                .as_ref()
                .is_some_and(|s| s.len() > 8192 || s.chars().any(char::is_control))
        {
            return Err("invalid_request");
        }
        Ok(())
    }

    fn matches(&self, agent: &ScopedAgentInfo) -> bool {
        self.machine.as_ref().is_none_or(|m| m == &agent.agent.host)
            && self.project_host.as_ref().is_none_or(|h| {
                agent
                    .communication_project
                    .as_ref()
                    .is_some_and(|p| &p.host == h && Some(&p.root) == self.project_root.as_ref())
            })
    }
}

pub(super) fn directory(socket: &Path) -> Result<Vec<ScopedAgentInfo>, &'static str> {
    match crate::communication::client::directory_request(
        None,
        socket,
        CommunicationDirectoryScope::Global,
        None,
    ) {
        Ok(DaemonToWrapper::CommunicationDirectory { agents, .. }) if agents.len() <= 10_000 => {
            Ok(agents)
        }
        _ => Err("endpoint_unavailable"),
    }
}

pub(super) fn permitted<'a>(
    device: &Device,
    agents: &'a [ScopedAgentInfo],
    op: Operation,
) -> Vec<&'a ScopedAgentInfo> {
    agents
        .iter()
        .filter(|a| {
            device
                .scope
                .permits(op, &a.agent.host, a.communication_project.as_ref())
        })
        .collect()
}

pub(super) fn agents_page(
    device: &Device,
    agents: &[ScopedAgentInfo],
    query: &DirectoryQuery,
) -> Result<Value, &'static str> {
    query.validate()?;
    let mut visible = permitted(device, agents, Operation::Read);
    visible.retain(|a| query.matches(a));
    visible.sort_by(|a, b| a.agent.agent_id.cmp(&b.agent.agent_id));
    if visible
        .windows(2)
        .any(|w| w[0].agent.agent_id == w[1].agent.agent_id)
    {
        return Err("endpoint_unavailable");
    }
    let mut selected = visible
        .into_iter()
        .filter(|a| query.after.as_ref().is_none_or(|id| &a.agent.agent_id > id));
    let limit = query.limit.unwrap_or(50);
    let page: Vec<_> = selected.by_ref().take(limit).collect();
    let more = selected.next().is_some();
    let after = if more {
        page.last().map(|a| a.agent.agent_id.clone())
    } else {
        None
    };
    Ok(json!({"items":page.iter().map(|a| json!({
        "agent_id":a.agent.agent_id,"display_name":a.agent.display_name,"machine":a.agent.host,
        "project":a.communication_project,"state":a.agent.state,"observed_age_secs":a.agent.last_seen_secs,
        "agent_type":a.agent.agent_type,"transport":a.agent.transport,"model":a.agent.model
    })).collect::<Vec<_>>(),"next_cursor":after,"snapshot":"current_native_directory"}))
}

pub(super) fn projects_page(
    device: &Device,
    agents: &[ScopedAgentInfo],
    query: &DirectoryQuery,
) -> Result<Value, &'static str> {
    query.validate()?;
    let mut projects = BTreeMap::new();
    for agent in permitted(device, agents, Operation::Read)
        .into_iter()
        .filter(|a| query.matches(a))
    {
        if let Some(p) = &agent.communication_project {
            projects.insert((p.host.clone(), p.root.clone()), p);
        }
    }
    // Le repère JSON d'identité reste une valeur opaque ; aucun accès par chemin.
    let mut selected = projects
        .into_values()
        .filter_map(|p| {
            let cursor = serde_json::to_string(&(p.host.as_str(), p.root.as_str())).ok()?;
            Some((cursor, p))
        })
        .collect::<Vec<_>>();
    selected.sort_by(|a, b| a.0.cmp(&b.0));
    selected.retain(|(id, _)| query.after.as_ref().is_none_or(|after| id > after));
    let more = selected.len() > query.limit.unwrap_or(50);
    selected.truncate(query.limit.unwrap_or(50));
    let after = if more {
        selected.last().map(|a| a.0.clone())
    } else {
        None
    };
    Ok(json!({"items":selected.iter().map(|(_,p)|p).collect::<Vec<_>>(),"next_cursor":after}))
}

pub(super) fn authorized_agent<'a>(
    device: &Device,
    agents: &'a [ScopedAgentInfo],
    id: &str,
    op: Operation,
) -> Result<&'a ScopedAgentInfo, &'static str> {
    let mut found = agents.iter().filter(|a| a.agent.agent_id == id);
    let agent = found.next().ok_or("not_found")?;
    if found.next().is_some() {
        return Err("endpoint_unavailable");
    }
    if !device
        .scope
        .permits(op, &agent.agent.host, agent.communication_project.as_ref())
    {
        return Err("not_found");
    }
    Ok(agent)
}

/// L'issuer du reçu est calculé par le maître depuis la clé d'appareil, jamais fourni par HTTP.
pub(crate) fn mobile_operation(
    socket: &Path,
    credential: &str,
    operation: bridget_transport::protocol::MobileOperation,
) -> Result<Value, &'static str> {
    use crate::communication::client::DaemonConnection;
    use bridget_transport::{
        WrapperToDaemon,
        protocol::{
            CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdentityCredential,
            MobileOperationRequest,
        },
    };
    let mut connection = DaemonConnection::connect(socket).map_err(|_| "endpoint_unavailable")?;
    if !matches!(
        connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        }),
        Ok(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ) {
        return Err("endpoint_unavailable");
    }
    if !matches!(connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:crate::communication::issuer_scope("mobile-operation"),
        capabilities:vec![ClientCapability::MobileOperationsV1],
    }),Ok(DaemonToWrapper::ClientWelcome{version:CLIENT_CONTRACT_VERSION,capabilities,..})
        if capabilities==[ClientCapability::MobileOperationsV1])
    {
        return Err("capability_unavailable");
    }
    match connection.exchange(&WrapperToDaemon::MobileOperation {
        request: MobileOperationRequest {
            credential: IdentityCredential::new(credential.into()),
            operation,
        },
    }) {
        Ok(DaemonToWrapper::MobileOperationResult { result }) => Ok(result),
        Err(crate::communication::client::ClientError::Technical {
            code: "update_in_progress",
            ..
        }) => Err("update_in_progress"),
        _ => Err("outcome_unknown"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::store::Scope;
    use super::*;
    fn agent(id: &str, host: &str, root: &str) -> ScopedAgentInfo {
        serde_json::from_value(json!({
            "agent_id":id,"display_name":id,"agent_type":"custom","connection_id":"private-connection",
            "host":host,"transport":"fixture","state":"working","last_seen_secs":2,"reconnect_count":0,
            "communication_project":{"host":host,"root":root},"project_relation":"same"
        })).unwrap()
    }
    fn device() -> Device {
        Device {
            device_id: uuid::Uuid::new_v4().to_string(),
            label: "Phone".into(),
            scope: Scope {
                all_machines: false,
                machines: vec!["cartae".into()],
                all_projects: false,
                projects: vec![CommunicationProject {
                    host: "cartae".into(),
                    root: "/project/a".into(),
                }],
                operations: vec![Operation::Read, Operation::Message],
            },
            created_at: 1,
            expires_at: 100,
            revoked_at: None,
        }
    }
    #[test]
    fn mobile177_directory_authorizes_before_pagination_and_hides_neighbouring_projects() {
        let d = device();
        let list = vec![
            agent("00", "secret", "/private"),
            agent("01", "cartae", "/project/b"),
            agent("02", "cartae", "/project/a"),
            agent("03", "cartae", "/project/a"),
        ];
        let q = DirectoryQuery {
            limit: Some(1),
            ..Default::default()
        };
        let first = agents_page(&d, &list, &q).unwrap();
        assert_eq!(first["items"][0]["agent_id"], "02");
        assert_eq!(first["next_cursor"], "02");
        let second = agents_page(
            &d,
            &list,
            &DirectoryQuery {
                after: Some("02".into()),
                ..q
            },
        )
        .unwrap();
        assert_eq!(second["items"][0]["agent_id"], "03");
        assert!(second["next_cursor"].is_null());
        let projected = serde_json::to_string(&first).unwrap();
        assert!(!projected.contains("private"));
        assert!(!projected.contains("connection_id"));
        assert_eq!(
            projects_page(&d, &list, &Default::default()).unwrap()["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            authorized_agent(&d, &list, "01", Operation::Read).unwrap_err(),
            "not_found"
        );
    }
    #[test]
    fn mobile177_directory_empty_scope_never_becomes_global_and_filters_cannot_grant() {
        let mut d = device();
        let list = vec![
            agent("01", "cartae", "/project/a"),
            agent("02", "mac", "/project/a"),
        ];
        assert!(
            agents_page(
                &d,
                &list,
                &DirectoryQuery {
                    machine: Some("mac".into()),
                    ..Default::default()
                }
            )
            .unwrap()["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        d.scope.machines.clear();
        assert!(
            agents_page(&d, &list, &Default::default()).unwrap()["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(serde_json::from_str::<DirectoryQuery>(r#"{"actor":"human"}"#).is_err());
        assert!(
            agents_page(
                &d,
                &list,
                &DirectoryQuery {
                    limit: Some(51),
                    ..Default::default()
                }
            )
            .is_err()
        );
    }
}
