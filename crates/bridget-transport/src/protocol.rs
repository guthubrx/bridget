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
        /// Regroupement de travail de l'agent : nom du dépôt d'où il a été
        /// lancé, ou domaine choisi explicitement s'il en existe un.
        #[serde(default)]
        domain: Option<String>,
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
    /// Rapporter le modèle et le niveau d'effort courants d'un agent.
    ///
    /// `agent` désigne l'agent observé, et non la connexion émettrice : le hook
    /// Claude et `bridget runtime` passent par le client CLI, dont la connexion
    /// est éphémère et distincte de celle de l'agent. Même motif que `Rename`.
    ///
    /// Une observation est atomique : le couple `(model, effort)` remplace en
    /// bloc l'état connu. Un `effort` absent signifie « observé absent » et
    /// efface la valeur précédente — un modèle sans réglage d'effort ne doit
    /// pas hériter de l'effort du modèle précédent.
    Runtime {
        agent: String,
        model: String,
        #[serde(default)]
        effort: Option<String>,
        source: RuntimeSource,
    },
    /// Remplacer le domaine d'un agent, ou revenir au domaine dérivé.
    ///
    /// `domain: None` signifie « réinitialiser » : le daemon reprend alors le
    /// domaine annoncé à l'enregistrement.
    Domain {
        agent: String,
        #[serde(default)]
        domain: Option<String>,
    },
    /// Déclarer la disponibilité d'un agent.
    ///
    /// `until_secs` est un horodatage Unix jusqu'auquel l'agent refuse d'être
    /// dérangé. `None` lève le statut immédiatement. Représenter une échéance
    /// plutôt qu'un booléen rend l'expiration automatique sans tâche de fond.
    Availability {
        agent: String,
        #[serde(default)]
        until_secs: Option<u64>,
    },
}

/// Origine d'une observation de runtime. Énumération fermée : une valeur
/// inconnue rend le message indécodable plutôt que d'entrer dans l'annuaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeSource {
    /// Lu dans le fichier rollout d'un agent Codex.
    #[serde(rename = "codex-rollout")]
    CodexRollout,
    /// Rapporté par le hook Stop d'un agent Claude Code.
    #[serde(rename = "claude-hook")]
    ClaudeHook,
    /// Déclaré explicitement via `bridget runtime`.
    #[serde(rename = "declared")]
    Declared,
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            RuntimeSource::CodexRollout => "codex-rollout",
            RuntimeSource::ClaudeHook => "claude-hook",
            RuntimeSource::Declared => "declared",
        };
        f.write_str(label)
    }
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
    /// Regroupement de travail, `None` si indéterminable.
    #[serde(default)]
    pub domain: Option<String>,
    /// Modèle courant, `None` tant qu'aucune observation n'a eu lieu.
    #[serde(default)]
    pub model: Option<String>,
    /// Niveau d'effort courant. `None` couvre deux cas indiscernables pour un
    /// lecteur : jamais observé, ou observé absent (modèle sans réglage
    /// d'effort). Les deux s'affichent de la même façon.
    #[serde(default)]
    pub effort: Option<String>,
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
            domain: Some("bridget".to_string()),
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
                domain,
            } => {
                assert_eq!(agent_type, "codex");
                assert!(name.is_none());
                assert_eq!(host.as_deref(), Some("test-host"));
                assert_eq!(transport.as_deref(), Some("unix"));
                assert_eq!(os.as_deref(), Some("Linux"));
                assert_eq!(instance_id.as_deref(), Some("instance-test"));
                assert_eq!(domain.as_deref(), Some("bridget"));
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
    fn test_encode_decode_runtime() {
        let msg = WrapperToDaemon::Runtime {
            agent: "agent-2".to_string(),
            model: "claude-opus-5".to_string(),
            effort: Some("high".to_string()),
            source: RuntimeSource::ClaudeHook,
        };
        let json = encode(&msg).unwrap();
        assert!(json.contains("\"type\":\"Runtime\""));
        assert!(json.contains("\"source\":\"claude-hook\""));
        match decode(&json).unwrap() {
            WrapperToDaemon::Runtime {
                agent,
                model,
                effort,
                source,
            } => {
                assert_eq!(agent, "agent-2");
                assert_eq!(model, "claude-opus-5");
                assert_eq!(effort.as_deref(), Some("high"));
                assert_eq!(source, RuntimeSource::ClaudeHook);
            }
            other => panic!("mauvais type: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_effort_absent_est_decodable() {
        // Cas Haiku : le modèle n'expose aucun niveau d'effort.
        let json = r#"{"type":"Runtime","agent":"agent-2","model":"claude-haiku-4-5","source":"codex-rollout"}"#;
        match decode(json).unwrap() {
            WrapperToDaemon::Runtime { effort, .. } => assert!(effort.is_none()),
            other => panic!("mauvais type: {:?}", other),
        }
    }

    #[test]
    fn test_runtime_source_inconnue_est_refusee() {
        let json = r#"{"type":"Runtime","agent":"agent-2","model":"x","source":"inventee"}"#;
        assert!(decode::<WrapperToDaemon>(json).is_err());
    }

    #[test]
    fn test_agent_info_sans_runtime_reste_decodable() {
        // Compatibilité ascendante : un daemon d'une version antérieure ne
        // sérialise ni model ni effort.
        let json = r#"{"name":"agent-2","agent_type":"claude","connection_id":"conn-1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0}"#;
        let info: AgentInfo = decode(json).unwrap();
        assert!(info.model.is_none());
        assert!(info.effort.is_none());
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
