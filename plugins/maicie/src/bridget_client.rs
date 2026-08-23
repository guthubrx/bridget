//! Adaptateur Maicie vers le protocole local public de Bridget.
//!
//! Ce module ne depend volontairement d'aucun crate Bridget : il parle le
//! protocole JSONL public sur socket Unix. Les valeurs filaires sont donc
//! declarees ici et couvertes par les fixtures producteur↔client. Aucun autre
//! module Maicie ne doit ouvrir le socket Bridget directement.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

/// Version du contrat public client Bridget consommee par Maicie.
pub const CLIENT_CONTRACT_VERSION: u16 = 1;

/// Capacites du contrat idempotent que Maicie exige avant tout envoi.
pub const REQUIRED_CLIENT_CAPABILITIES: [&str; 2] = ["send_idempotent", "lookup"];

/// Erreurs explicites de la frontiere de transport Maicie → Bridget.
#[derive(Debug)]
pub enum BridgetClientError {
    Connect {
        path: PathBuf,
        source: std::io::Error,
    },
    Read(std::io::Error),
    Write(std::io::Error),
    Encode(serde_json::Error),
    Decode {
        line: String,
        source: serde_json::Error,
    },
    Closed,
    Protocol(String),
    VersionUnsupported {
        requested: u16,
        received: u16,
    },
    CapabilityMissing {
        capability: String,
    },
    ClientRejected {
        reason: Value,
    },
    RemoteNack {
        id: String,
        reason: String,
    },
    InvalidEnvelope(String),
}

impl fmt::Display for BridgetClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect { path, source } => {
                write!(formatter, "connexion Bridget impossible vers {}: {source}", path.display())
            }
            Self::Read(source) => write!(formatter, "lecture Bridget impossible: {source}"),
            Self::Write(source) => write!(formatter, "ecriture Bridget impossible: {source}"),
            Self::Encode(source) => write!(formatter, "commande Bridget non serialisable: {source}"),
            Self::Decode { line, source } => {
                write!(formatter, "reponse Bridget invalide ({line:?}): {source}")
            }
            Self::Closed => formatter.write_str("socket Bridget ferme avant la reponse"),
            Self::Protocol(detail) => write!(formatter, "protocole Bridget inattendu: {detail}"),
            Self::VersionUnsupported { requested, received } => write!(
                formatter,
                "version Bridget incompatible: Maicie demande {requested}, Bridget a negocie {received}"
            ),
            Self::CapabilityMissing { capability } => {
                write!(formatter, "capacite Bridget obligatoire absente: {capability}")
            }
            Self::ClientRejected { reason } => write!(formatter, "client Bridget refuse: {reason}"),
            Self::RemoteNack { id, reason } => write!(formatter, "Bridget refuse {id}: {reason}"),
            Self::InvalidEnvelope(detail) => write!(formatter, "enveloppe Maicie invalide: {detail}"),
        }
    }
}

impl std::error::Error for BridgetClientError {}

/// Enveloppe publique, exacte, que l'outbox Maicie persistera avant I/O.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub body: String,
    #[serde(default)]
    pub reply: bool,
    #[serde(default = "default_hops")]
    pub hops: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_timeout: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_at: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
}

impl PublicMessage {
    /// Valide ce que Maicie peut verifier sans reinterpretation semantique.
    pub fn validate(&self) -> Result<(), BridgetClientError> {
        for (field, value) in [
            ("id", self.id.as_str()),
            ("from", self.from.as_str()),
            ("to", self.to.as_str()),
            ("body", self.body.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(BridgetClientError::InvalidEnvelope(format!(
                    "{field} ne peut pas etre vide"
                )));
            }
        }
        if self.hops <= 0 {
            return Err(BridgetClientError::InvalidEnvelope(
                "hops doit etre strictement positif".to_string(),
            ));
        }
        if !self.reply && self.reply_timeout.is_some() {
            return Err(BridgetClientError::InvalidEnvelope(
                "reply_timeout exige reply=true".to_string(),
            ));
        }
        Ok(())
    }
}

fn default_hops() -> i32 {
    4
}

/// Issue durable du contrat client, sans inferrer d'etat Maicie.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IdempotencyIssue {
    Accepted {
        expires_at: i64,
    },
    Rejected {
        category: String,
        reason: String,
        expires_at: i64,
    },
    OutcomeUnknown {
        expires_at: i64,
        #[serde(default)]
        delivery_id: Option<String>,
    },
    EnvelopeMismatch,
    IdempotencyExpired,
    InvalidIssuedAt,
}

/// Contrat effectivement accepte par Bridget pour cette connexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedContract {
    pub version: u16,
    pub horizon_secs: i64,
    pub issued_at_tolerance_secs: i64,
    pub capabilities: BTreeSet<String>,
}

/// Information factuelle issue de l'annuaire Bridget.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
}

fn unknown_os() -> String {
    "inconnu".to_string()
}

/// Etat public retourne par une annulation de demande suivie.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Cancellation {
    pub id: String,
    pub state: String,
}

/// Fenetre publique d'un abonnement attach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum AttachWindow {
    Today,
    Seq(u64),
    Date(String),
}

/// Evenement factuel de la session 008, sans interpretation de son contenu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionEvent {
    Subscribed {
        subscription_id: String,
    },
    SnapshotCaughtUp {
        subscription_id: String,
        through_seq: Option<u64>,
    },
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        reason: Option<String>,
    },
    End {
        subscription_id: String,
        reason: String,
    },
}

/// Ordre SpawnOrder deja construit et approuve par les couches superieures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpawnOrder {
    pub agent_type: String,
    pub name: Option<String>,
    pub cwd: String,
    pub persistent: bool,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
}

/// Reponse publique d'un SpawnOrder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnOutcome {
    Accepted { command_id: String, name: String },
    Rejected { command_id: String, reason: Value },
    Idempotency(IdempotencyIssue),
}

/// Connexion client negociee, reservee au SendIdempotent et Lookup publics.
pub struct BridgetClient {
    socket_path: PathBuf,
    issuer_scope: String,
    connection: WireConnection,
    negotiated: NegotiatedContract,
}

impl BridgetClient {
    /// Ouvre et negocie strictement le contrat client public v1.
    pub fn connect(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
    ) -> Result<Self, BridgetClientError> {
        let socket_path = socket_path.as_ref().to_path_buf();
        let issuer_scope = issuer_scope.into();
        let mut connection = WireConnection::connect(&socket_path)?;

        let role = connection.request(json!({"type": "RoleHandshake", "role": "client"}))?;
        expect_role_accepted(&role, "client")?;

        let welcome = connection.request(json!({
            "type": "ClientHello",
            "contract_version": CLIENT_CONTRACT_VERSION,
            "issuer_scope": issuer_scope,
            "capabilities": REQUIRED_CLIENT_CAPABILITIES,
        }))?;
        let negotiated = parse_client_welcome(welcome)?;
        if negotiated.version != CLIENT_CONTRACT_VERSION {
            return Err(BridgetClientError::VersionUnsupported {
                requested: CLIENT_CONTRACT_VERSION,
                received: negotiated.version,
            });
        }
        for capability in REQUIRED_CLIENT_CAPABILITIES {
            if !negotiated.capabilities.contains(capability) {
                return Err(BridgetClientError::CapabilityMissing {
                    capability: capability.to_string(),
                });
            }
        }

        Ok(Self {
            socket_path,
            issuer_scope,
            connection,
            negotiated,
        })
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn issuer_scope(&self) -> &str {
        &self.issuer_scope
    }

    pub fn negotiated(&self) -> &NegotiatedContract {
        &self.negotiated
    }

    /// Envoie l'enveloppe exacte fournie par l'outbox, sans generer ni muter
    /// le `message_id` client.
    pub fn send_idempotent(
        &mut self,
        message: &PublicMessage,
        message_id: &str,
        issued_at: i64,
    ) -> Result<IdempotencyIssue, BridgetClientError> {
        message.validate()?;
        if message.id != message_id {
            return Err(BridgetClientError::InvalidEnvelope(
                "message.id et message_id doivent etre identiques".to_string(),
            ));
        }
        let response = self.connection.request(json!({
            "type": "SendIdempotent",
            "message": message,
            "message_id": message_id,
            "issued_at": issued_at,
        }))?;
        parse_idempotency_issue(response, message_id)
    }

    /// Lit l'issue durable deja associee a une cle d'envoi Maicie.
    pub fn lookup(&mut self, message_id: &str) -> Result<IdempotencyIssue, BridgetClientError> {
        if message_id.trim().is_empty() {
            return Err(BridgetClientError::InvalidEnvelope(
                "message_id ne peut pas etre vide".to_string(),
            ));
        }
        let response = self.connection.request(json!({
            "type": "Lookup",
            "operation_kind": "send",
            "idempotency_key": message_id,
        }))?;
        parse_idempotency_issue(response, message_id)
    }

    /// Lit l'annuaire public Bridget sur une connexion ponctuelle non mutante.
    pub fn list_agents(&self) -> Result<Vec<AgentInfo>, BridgetClientError> {
        Self::list_agents_at(&self.socket_path)
    }

    /// Lit l'annuaire avant toute negociation client. Cette operation reste
    /// disponible pour expliquer une incompatibilite de capacites plutot que
    /// de masquer les agents presents.
    pub fn list_agents_at(
        socket_path: impl AsRef<Path>,
    ) -> Result<Vec<AgentInfo>, BridgetClientError> {
        let mut connection = WireConnection::connect(socket_path.as_ref())?;
        let response = connection.request(json!({"type": "ListAgents"}))?;
        match response_type(&response)? {
            "AgentList" => {
                serde_json::from_value(response.get("agents").cloned().ok_or_else(|| {
                    BridgetClientError::Protocol("AgentList sans agents".to_string())
                })?)
                .map_err(|source| BridgetClientError::Decode {
                    line: response.to_string(),
                    source,
                })
            }
            "Nack" => Err(parse_nack(response)?),
            other => Err(unexpected("AgentList", other)),
        }
    }

    /// Relaye une annulation par l'interface publique historique. Bridget reste
    /// l'autorite de l'issue ; Maicie ne modifie jamais une demande elle-meme.
    pub fn cancel_request(
        &self,
        id: &str,
        sender: &str,
        reason: Option<&str>,
    ) -> Result<Cancellation, BridgetClientError> {
        let mut connection = WireConnection::connect(&self.socket_path)?;
        let response = connection.request(json!({
            "type": "CancelRequest",
            "id": id,
            "sender": sender,
            "reason": reason,
        }))?;
        match response_type(&response)? {
            "RequestCancelled" => {
                serde_json::from_value(response).map_err(|source| BridgetClientError::Decode {
                    line: "RequestCancelled".to_string(),
                    source,
                })
            }
            "Nack" => Err(parse_nack(response)?),
            other => Err(unexpected("RequestCancelled", other)),
        }
    }

    /// Ouvre une connexion attach publique. Le consommateur lit ensuite les
    /// evenements `Gap`/`End` sans jamais ouvrir le journal interne Bridget.
    pub fn subscribe(
        &self,
        agent: &str,
        window: AttachWindow,
    ) -> Result<Subscription, BridgetClientError> {
        let mut connection = WireConnection::connect(&self.socket_path)?;
        let role = connection.request(json!({"type": "RoleHandshake", "role": "attach"}))?;
        expect_role_accepted(&role, "attach")?;
        connection.send(json!({
            "type": "Subscribe",
            "agent": agent,
            "window": window,
        }))?;
        Ok(Subscription { connection })
    }

    /// Emet un SpawnOrder deja approuve. Cette methode ne construit aucun
    /// processus et ne relance pas d'elle-meme l'ordre en cas d'incertitude.
    pub fn spawn_order(&self, order: &SpawnOrder) -> Result<SpawnOutcome, BridgetClientError> {
        let mut connection = WireConnection::connect(&self.socket_path)?;
        let response = connection.request(json!({
            "type": "SpawnOrder",
            "agent_type": order.agent_type,
            "name": order.name,
            "cwd": order.cwd,
            "persistent": order.persistent,
            "command_id": order.command_id,
            "issued_at": order.issued_at,
            "deadline_at": order.deadline_at,
        }))?;
        match response_type(&response)? {
            "SpawnAccepted" => Ok(SpawnOutcome::Accepted {
                command_id: required_string(&response, "command_id")?,
                name: required_string(&response, "name")?,
            }),
            "SpawnRejected" => Ok(SpawnOutcome::Rejected {
                command_id: required_string(&response, "command_id")?,
                reason: response.get("reason").cloned().ok_or_else(|| {
                    BridgetClientError::Protocol("SpawnRejected sans reason".to_string())
                })?,
            }),
            "IdempotencyResult" => Ok(SpawnOutcome::Idempotency(parse_idempotency_issue(
                response,
                &order.command_id,
            )?)),
            "Nack" => Err(parse_nack(response)?),
            other => Err(unexpected("SpawnAccepted/SpawnRejected", other)),
        }
    }
}

/// Connexion attach dont la lecture est la seule source d'evenements runtime.
pub struct Subscription {
    connection: WireConnection,
}

impl Subscription {
    pub fn next_event(&mut self) -> Result<SubscriptionEvent, BridgetClientError> {
        let response = self.connection.receive()?;
        match response_type(&response)? {
            "Subscribed" => Ok(SubscriptionEvent::Subscribed {
                subscription_id: required_string(&response, "subscription_id")?,
            }),
            "SnapshotCaughtUp" => Ok(SubscriptionEvent::SnapshotCaughtUp {
                subscription_id: required_string(&response, "subscription_id")?,
                through_seq: optional_u64(&response, "through_seq")?,
            }),
            "Gap" => Ok(SubscriptionEvent::Gap {
                subscription_id: required_string(&response, "subscription_id")?,
                from_seq: required_u64(&response, "from_seq")?,
                to_seq: required_u64(&response, "to_seq")?,
                reason: optional_string(&response, "reason")?,
            }),
            "End" => Ok(SubscriptionEvent::End {
                subscription_id: required_string(&response, "subscription_id")?,
                reason: required_string(&response, "reason")?,
            }),
            "AttachRejected" | "Nack" => Err(BridgetClientError::Protocol(response.to_string())),
            other => Err(unexpected("evenement d'abonnement", other)),
        }
    }
}

struct WireConnection {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

impl WireConnection {
    fn connect(path: &Path) -> Result<Self, BridgetClientError> {
        let stream = UnixStream::connect(path).map_err(|source| BridgetClientError::Connect {
            path: path.to_path_buf(),
            source,
        })?;
        let reader = BufReader::new(stream.try_clone().map_err(BridgetClientError::Read)?);
        Ok(Self {
            reader,
            writer: BufWriter::new(stream),
        })
    }

    fn request(&mut self, value: Value) -> Result<Value, BridgetClientError> {
        self.send(value)?;
        self.receive()
    }

    fn send(&mut self, value: Value) -> Result<(), BridgetClientError> {
        serde_json::to_writer(&mut self.writer, &value).map_err(BridgetClientError::Encode)?;
        self.writer
            .write_all(b"\n")
            .map_err(BridgetClientError::Write)?;
        self.writer.flush().map_err(BridgetClientError::Write)
    }

    fn receive(&mut self) -> Result<Value, BridgetClientError> {
        let mut line = String::new();
        let read = self
            .reader
            .read_line(&mut line)
            .map_err(BridgetClientError::Read)?;
        if read == 0 {
            return Err(BridgetClientError::Closed);
        }
        serde_json::from_str(line.trim_end())
            .map_err(|source| BridgetClientError::Decode { line, source })
    }
}

fn parse_client_welcome(response: Value) -> Result<NegotiatedContract, BridgetClientError> {
    match response_type(&response)? {
        "ClientWelcome" => {
            let version = u16::try_from(required_u64(&response, "version")?).map_err(|_| {
                BridgetClientError::Protocol("version Bridget hors plage u16".to_string())
            })?;
            let horizon_secs = required_i64(&response, "horizon_secs")?;
            let issued_at_tolerance_secs = required_i64(&response, "issued_at_tolerance_secs")?;
            if horizon_secs <= 0 || issued_at_tolerance_secs < 0 {
                return Err(BridgetClientError::Protocol(
                    "horizon ou tolerance Bridget invalide".to_string(),
                ));
            }
            let capabilities = response
                .get("capabilities")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    BridgetClientError::Protocol("ClientWelcome sans capabilities".to_string())
                })?
                .iter()
                .map(|value| {
                    value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                        BridgetClientError::Protocol("capabilite Bridget non textuelle".to_string())
                    })
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            Ok(NegotiatedContract {
                version,
                horizon_secs,
                issued_at_tolerance_secs,
                capabilities,
            })
        }
        "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("ClientWelcome", other)),
    }
}

fn expect_role_accepted(response: &Value, role: &str) -> Result<(), BridgetClientError> {
    match response_type(response)? {
        "RoleAccepted" if response.get("role").and_then(Value::as_str) == Some(role) => Ok(()),
        "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("RoleAccepted", other)),
    }
}

fn parse_idempotency_issue(
    response: Value,
    expected_key: &str,
) -> Result<IdempotencyIssue, BridgetClientError> {
    match response_type(&response)? {
        "IdempotencyResult" => {
            if response.get("operation_kind").and_then(Value::as_str) != Some("send") {
                return Err(BridgetClientError::Protocol(
                    "issue idempotente hors operation send".to_string(),
                ));
            }
            let key = required_string(&response, "idempotency_key")?;
            if key != expected_key {
                return Err(BridgetClientError::Protocol(format!(
                    "issue Bridget pour {key}, attendue pour {expected_key}"
                )));
            }
            serde_json::from_value(response.get("issue").cloned().ok_or_else(|| {
                BridgetClientError::Protocol("issue idempotente absente".to_string())
            })?)
            .map_err(|source| BridgetClientError::Decode {
                line: response.to_string(),
                source,
            })
        }
        "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        "Nack" => Err(parse_nack(response)?),
        other => Err(unexpected("IdempotencyResult", other)),
    }
}

fn parse_nack(response: Value) -> Result<BridgetClientError, BridgetClientError> {
    Ok(BridgetClientError::RemoteNack {
        id: required_string(&response, "id")?,
        reason: required_string(&response, "reason")?,
    })
}

fn response_type(response: &Value) -> Result<&str, BridgetClientError> {
    response
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| BridgetClientError::Protocol("reponse sans type".to_string()))
}

fn required_string(response: &Value, field: &str) -> Result<String, BridgetClientError> {
    response
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} absent ou vide")))
}

fn optional_string(response: &Value, field: &str) -> Result<Option<String>, BridgetClientError> {
    response
        .get(field)
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} non textuel")))
        })
        .transpose()
}

fn required_u64(response: &Value, field: &str) -> Result<u64, BridgetClientError> {
    response
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} absent ou invalide")))
}

fn optional_u64(response: &Value, field: &str) -> Result<Option<u64>, BridgetClientError> {
    response
        .get(field)
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} invalide")))
        })
        .transpose()
}

fn required_i64(response: &Value, field: &str) -> Result<i64, BridgetClientError> {
    response
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} absent ou invalide")))
}

fn unexpected(expected: &str, received: &str) -> BridgetClientError {
    BridgetClientError::Protocol(format!("{expected} attendu, {received} recu"))
}
