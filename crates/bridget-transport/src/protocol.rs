//! Définitions des messages JSON du protocole daemon/wrapper.
//!
//! Deux directions :
//! - WrapperToDaemon : ce que le wrapper envoie au daemon
//! - DaemonToWrapper : ce que le daemon envoie au wrapper

use bridget_core::BridgetMessage;
use serde::{Deserialize, Serialize};

/// Messages envoyés par le wrapper vers le daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WrapperToDaemon {
    /// S'enregistrer auprès du daemon.
    Register {
        agent_type: String,
        name: Option<String>,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        transport: Option<String>,
        #[serde(default)]
        os: Option<String>,
        #[serde(default)]
        instance_id: Option<String>,
    },
    /// Se désenregistrer.
    Unregister,
    /// Renommer un agent déjà enregistré.
    Rename { current_name: String, name: String },
    /// Envoyer un message à un autre agent.
    Send(BridgetMessage),
    /// Annuler une demande suivie appartenant à l'agent courant.
    CancelRequest {
        id: String,
        sender: String,
        reason: Option<String>,
    },
    /// Lister les demandes suivies de l'agent courant.
    ListRequests { sender: String },
    /// Signal de vie (périodique).
    Heartbeat,
    /// Demander la liste des agents connectés.
    ListAgents,
}

/// Messages envoyés par le daemon vers le wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonToWrapper {
    /// Confirmation d'enregistrement avec le nom final.
    Registered { name: String },
    /// Confirmation d'un renommage.
    Renamed { old_name: String, name: String },
    /// Livrer un message à l'agent.
    Deliver(BridgetMessage),
    /// Acquittement d'un envoi.
    Ack { id: String },
    /// Refus d'un envoi avec raison.
    Nack { id: String, reason: String },
    /// Le daemon s'éteint.
    Disconnect,
    /// Réponse à ListAgents.
    AgentList { agents: Vec<AgentInfo> },
    /// État final d'une annulation.
    RequestCancelled { id: String, state: String },
    /// Liste des demandes suivies accessibles à l'agent courant.
    RequestList { requests: Vec<RequestInfo> },
}

/// Sérialise un message en ligne JSON (newline-delimited JSON).
pub fn encode<T: Serialize>(msg: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}

/// Désérialise une ligne JSON.
pub fn decode<T: for<'de> Deserialize<'de>>(line: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(line)
}

/// Information sur un agent connecté.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub agent_type: String,
    pub connection_id: String,
    pub host: String,
    pub transport: String,
    #[serde(default = "unknown_os")]
    pub os: String,
    pub state: String,
    pub last_seen_secs: u64,
    pub reconnect_count: u32,
}

fn unknown_os() -> String {
    "inconnu".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub id: String,
    pub target: String,
    pub state: String,
    pub deadline_at: i64,
    pub cancel_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_register() {
        let msg = WrapperToDaemon::Register {
            agent_type: "codex".to_string(),
            name: None,
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            os: Some("Linux".to_string()),
            instance_id: Some("instance-test".to_string()),
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Register\""));
        let decoded: WrapperToDaemon = decode(&json).unwrap();
        match decoded {
            WrapperToDaemon::Register {
                agent_type,
                name,
                host,
                transport,
                os,
                instance_id,
            } => {
                assert_eq!(agent_type, "codex");
                assert!(name.is_none());
                assert_eq!(host.as_deref(), Some("test-host"));
                assert_eq!(transport.as_deref(), Some("unix"));
                assert_eq!(os.as_deref(), Some("Linux"));
                assert_eq!(instance_id.as_deref(), Some("instance-test"));
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn test_encode_decode_deliver() {
        let msg = BridgetMessage::new("claude-1", "codex-1", "hello");
        let dtw = DaemonToWrapper::Deliver(msg.clone());
        let json = encode(&dtw).unwrap();
        assert!(json.contains("\"type\":\"Deliver\""));
        let decoded: DaemonToWrapper = decode(&json).unwrap();
        match decoded {
            DaemonToWrapper::Deliver(m) => {
                assert_eq!(m.from, "claude-1");
                assert_eq!(m.body, "hello");
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn test_encode_decode_send() {
        let msg = BridgetMessage::new("codex-1", "claude-1", "réponse");
        let wtd = WrapperToDaemon::Send(msg);
        let json = encode(&wtd).unwrap();
        let decoded: WrapperToDaemon = decode(&json).unwrap();
        match decoded {
            WrapperToDaemon::Send(m) => assert_eq!(m.body, "réponse"),
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn test_encode_decode_rename() {
        let msg = WrapperToDaemon::Rename {
            current_name: "codex-1".to_string(),
            name: "analyse".to_string(),
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Rename\""));
        assert!(
            matches!(decode(&json).unwrap(), WrapperToDaemon::Rename { current_name, name } if current_name == "codex-1" && name == "analyse")
        );

        let response = DaemonToWrapper::Renamed {
            old_name: "codex-1".to_string(),
            name: "analyse".to_string(),
        };
        assert!(
            matches!(decode(&encode(&response).unwrap()).unwrap(), DaemonToWrapper::Renamed { old_name, name } if old_name == "codex-1" && name == "analyse")
        );
    }

    #[test]
    fn test_encode_decode_nack() {
        let dtw = DaemonToWrapper::Nack {
            id: "abc123".to_string(),
            reason: "agent introuvable".to_string(),
        };
        let json = encode(&dtw).unwrap();
        let decoded: DaemonToWrapper = decode(&json).unwrap();
        match decoded {
            DaemonToWrapper::Nack { id, reason } => {
                assert_eq!(id, "abc123");
                assert_eq!(reason, "agent introuvable");
            }
            _ => panic!("mauvais type"),
        }
    }
}
