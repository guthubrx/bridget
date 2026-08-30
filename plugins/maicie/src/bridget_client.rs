//! Adaptateur Maicie vers le protocole local public de Bridget.
//!
//! Ce module ne depend volontairement d'aucun crate Bridget : il parle le
//! protocole JSONL public sur socket Unix. Les valeurs filaires sont donc
//! declarees ici et couvertes par les fixtures producteur↔client. Aucun autre
//! module Maicie ne doit ouvrir le socket Bridget directement.

use bridget_transport::protocol::ProjectReference;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fmt;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Version du contrat public client Bridget consommee par Maicie.
pub const CLIENT_CONTRACT_VERSION: u16 = 1;

/// Capacites du contrat idempotent que Maicie exige avant tout envoi.
pub const REQUIRED_CLIENT_CAPABILITIES: [&str; 2] = ["send_idempotent", "lookup"];

/// Version du contrat public du guichet Maicie.
pub const GUICHET_CONTRACT_VERSION: u16 = 1;

/// La relève, la réponse et les événements ne sont accessibles que par cette
/// capacité explicitement négociée. Un nom déclaré ne remplace jamais ce
/// contrôle de protocole.
pub const REQUIRED_GUICHET_CAPABILITY: &str = "maicie_guichet";

/// Capacité dédiée au registre de projets. Elle est négociée sur le rôle
/// `service` mais ne réutilise jamais le flux inverse `ServiceRequest`.
pub const REQUIRED_PROJECT_REGISTRY_CAPABILITY: &str = "project_registry_v1";

/// Capacité cursée de coordination. La v1 pousse un historique sans frontière
/// de fraîcheur ; Maicie ne la négocie donc jamais pour ses décisions 016.
pub const REQUIRED_COORDINATION_CAPABILITY: &str = "coordination_events_v2";

/// Version du flux cursé attesté par Bridget.
pub const COORDINATION_STREAM_VERSION: u16 = 2;

/// Bornes de la frontière locale : une réponse Bridget ne peut ni suspendre
/// Maicie indéfiniment ni lui faire accumuler une ligne JSONL illimitée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgetClientLimits {
    pub connect_timeout: Duration,
    pub io_timeout: Duration,
    pub max_frame_bytes: usize,
}

impl Default for BridgetClientLimits {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            io_timeout: Duration::from_secs(5),
            max_frame_bytes: 256 * 1024,
        }
    }
}

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
    Timeout {
        operation: &'static str,
    },
    FrameTooLarge {
        max_frame_bytes: usize,
    },
    ItemLimitExceeded {
        max_items: usize,
    },
    ConnectionUnusable,
    InvalidLimits(String),
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
            Self::Timeout { operation } => {
                write!(formatter, "echeance Bridget depassee pendant {operation}")
            }
            Self::FrameTooLarge { max_frame_bytes } => write!(
                formatter,
                "trame Bridget superieure a la borne de {max_frame_bytes} octets"
            ),
            Self::ItemLimitExceeded { max_items } => write!(
                formatter,
                "snapshot Bridget superieur a la borne de {max_items} elements"
            ),
            Self::ConnectionUnusable => formatter.write_str(
                "connexion Bridget inutilisable apres une reponse ambiguë ; reconnecter avant tout nouvel appel",
            ),
            Self::InvalidLimits(detail) => write!(formatter, "bornes Bridget invalides: {detail}"),
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

/// Projection de validation réservée à la reprise : l'absence de
/// `deny_unknown_fields` est intentionnelle et ne s'applique pas aux nouveaux
/// messages construits via [`PublicMessage`].
#[derive(Debug, Deserialize)]
struct ReplayPublicMessage {
    id: String,
    from: String,
    to: String,
    body: String,
    #[serde(default)]
    reply: bool,
    #[serde(default = "default_hops")]
    hops: i32,
    #[serde(default)]
    reply_timeout: Option<u64>,
    #[serde(default)]
    deadline_at: Option<u64>,
    #[serde(default)]
    in_reply_to: Option<String>,
}

impl ReplayPublicMessage {
    fn into_public_message(self) -> PublicMessage {
        PublicMessage {
            id: self.id,
            from: self.from,
            to: self.to,
            body: self.body,
            reply: self.reply,
            hops: self.hops,
            reply_timeout: self.reply_timeout,
            deadline_at: self.deadline_at,
            in_reply_to: self.in_reply_to,
        }
    }
}

/// Issue durable du contrat client, sans inferrer d'etat Maicie.
///
/// DEUX MÉCANIQUES SERDE, souvent confondues, et elles n'ont pas du tout le
/// même effet quand le daemon évolue :
///
/// - **Les VARIANTES ne sont pas tolérantes.** L'enum est tagué et ne porte
///   aucune variante de repli : un `kind` inconnu ne se perd pas en silence, il
///   fait ÉCHOUER le décodage (`unknown variant`, donc `BridgetClientError::
///   Decode`). Ajouter une variante au protocole filaire CASSE donc ce
///   consommateur — c'est bruyant, pas discret, mais c'est une casse.
/// - **Les CHAMPS, eux, le sont.** Le `#[serde(default)]` ci-dessous rend
///   `None` sans erreur quand le pair n'envoie pas `delivery_id` : là, et là
///   seulement, l'information se perd silencieusement.
///
/// Conséquence tenue par le dépôt : la distinction dépôt-attesté / sort-inconnu
/// vit sur la SURFACE CLIENT (le champ `status` du retour MCP et la sortie du
/// binaire), jamais en variante filaire. Ce serait une casse pure — Maicie
/// dispose déjà de la distinction par `delivery_id`. Voir l'oracle
/// `le_jumeau_refuse_une_variante_inconnue_au_lieu_de_la_perdre`, qui mesure
/// les deux mécaniques plutôt que de les supposer.
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
        /// Tolérant par CHAMP : absent du fil, il vaut `None` sans erreur.
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

/// Contrat effectivement accepté pour la connexion de service Maicie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegotiatedGuichetContract {
    pub version: u16,
    pub horizon_secs: i64,
    pub issued_at_tolerance_secs: i64,
    pub capabilities: BTreeSet<String>,
}

/// Résultat durable d'une opération de guichet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetResult {
    pub issuer_scope: String,
    pub request_id: String,
    pub issue: String,
    pub expires_at: i64,
    pub payload: Option<bridget_transport::protocol::GuichetReplyPayload>,
}

/// Demande relevée de manière exclusive par le service Maicie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetClaim {
    pub issuer_scope: String,
    pub request_id: String,
    pub canonical_request: Vec<u8>,
    pub authorization_attestation:
        Option<bridget_transport::greffe_authorization::GreffeAuthorizationAttestation>,
    pub claimed_at: i64,
    pub claim_generation: u64,
    pub claim_token: String,
    pub claim_lease_expires_at: i64,
    pub expires_at: i64,
}

/// Événement terminal produit exclusivement par Bridget pour le guichet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetLifecycleEvent {
    pub issuer_scope: String,
    pub event_id: String,
    pub request_id: String,
    pub state: String,
    pub observed_at: i64,
    pub in_reply_to: Option<String>,
    pub response_message_id: Option<String>,
}

/// Trame reçue pendant une relève bornée du flux de coordination. Les octets
/// de l'événement restent l'autorité du producteur ; les frontières de flux
/// sont projetées séparément et ne deviennent jamais des faits métier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinationStreamItem {
    Event {
        canonical_bytes: Vec<u8>,
    },
    Lifecycle(GuichetLifecycleEvent),
    SnapshotCaughtUp {
        through_cursor: Option<u64>,
    },
    Gap {
        from_cursor: u64,
        to_cursor: u64,
        reason: String,
    },
    Unavailable {
        reason: String,
    },
}

/// Connexion ponctuelle du service Maicie au flux cursé Bridget.
///
/// Elle est créée au début d'une commande et détruite après un seul snapshot :
/// aucun thread résident ni polling n'est introduit.
pub struct CoordinationClient {
    connection: WireConnection,
    deadline: Instant,
}

/// Connexion de service Maicie au guichet public Bridget.
///
/// Elle ne crée aucun runtime résident : son appelant l'ouvre à l'entrée d'une
/// commande locale et lui transmet, si nécessaire, une échéance absolue unique.
pub struct GuichetClient {
    socket_path: PathBuf,
    issuer_scope: String,
    limits: BridgetClientLimits,
    connection: WireConnection,
    negotiated: NegotiatedGuichetContract,
    deadline: Option<Instant>,
}

/// Connexion ponctuelle de Maicie à la mutation locale du registre de projets.
///
/// Les requêtes sont toujours des octets déjà persistés dans l'outbox Maicie :
/// cette frontière ne lit donc jamais la base privée de Bridget et ne recrée
/// pas une commande pendant une reprise.
pub struct ProjectRegistryClient {
    connection: WireConnection,
    deadline: Option<Instant>,
    limits: BridgetClientLimits,
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

/// Agrégat d'usage renvoyé par Bridget pour une fenêtre fermée.
/// `None` côté appelant signifie « inconnu », jamais un zéro inventé.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UsageWindowAggregate {
    pub turns: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub facturable_tokens: u64,
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
    JournalFragment {
        subscription_id: String,
        seq: u64,
        offset: u64,
        final_fragment: bool,
        bytes: Vec<u8>,
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
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
}

/// Reponse publique d'un SpawnOrder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnOutcome {
    Accepted { command_id: String, name: String },
    Rejected { command_id: String, reason: Value },
    Idempotency(IdempotencyIssue),
}

/// Résultat d'un rejeu SpawnOrder : l'issue est séparée du digest résolu afin
/// que le store continue à ne persister que ses états de coordination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnReplay {
    pub outcome: SpawnOutcome,
    pub definition_digest: Option<String>,
}

/// Connexion client negociee, reservee au SendIdempotent et Lookup publics.
pub struct BridgetClient {
    socket_path: PathBuf,
    issuer_scope: String,
    limits: BridgetClientLimits,
    connection: WireConnection,
    negotiated: NegotiatedContract,
    /// Échéance de passe optionnelle, réservée à la reprise bornée. Une
    /// connexion ordinaire conserve ses délais propres par opération.
    deadline: Option<Instant>,
}

/// Identité que le daemon atteste de lui-même sur la connexion client déjà
/// négociée. Les deux champs sont requis sur le fil ; l'absence de rapport est
/// représentée par une erreur de transport, jamais par une identité locale par
/// défaut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonIdentity {
    pub host: String,
    pub db_path: String,
}

/// Frontière filaire observable uniquement par les crash-tests d'outbox.
///
/// Le jalon suit l'écriture complète du JSONL et précède toute lecture de
/// l'accusé : interrompre le processus ici reproduit donc un ACK perdu, pas
/// un arrêt coopératif après la réponse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayIdempotentPhase {
    AfterWriteBeforeAck,
}

impl BridgetClient {
    /// Ouvre et negocie strictement le contrat client public v1.
    pub fn connect(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_limits(socket_path, issuer_scope, BridgetClientLimits::default())
    }

    /// Variante explicite pour les appels CLI/tests qui exigent un budget plus
    /// court. Les bornes sont posees avant le premier handshake.
    pub fn connect_with_limits(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, None)
    }

    /// Variante interne de reprise : toutes les opérations de la connexion
    /// consomment la même échéance absolue, sans la réinitialiser à chaque
    /// phase du protocole.
    pub fn connect_with_limits_until(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, Some(deadline))
    }

    fn connect_with_optional_deadline(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Option<Instant>,
    ) -> Result<Self, BridgetClientError> {
        validate_limits(limits)?;
        let socket_path = socket_path.as_ref().to_path_buf();
        let issuer_scope = issuer_scope.into();
        let connect_deadline = deadline.unwrap_or_else(|| monotonic_now() + limits.connect_timeout);
        let mut connection = WireConnection::connect(&socket_path, limits, connect_deadline)?;

        let role = request_with_deadline(
            &mut connection,
            json!({"type": "RoleHandshake", "role": "client"}),
            deadline,
        )?;
        expect_role_accepted(&role, "client")?;

        let welcome = request_with_deadline(
            &mut connection,
            json!({
                "type": "ClientHello",
                "contract_version": CLIENT_CONTRACT_VERSION,
                "issuer_scope": issuer_scope,
                "capabilities": REQUIRED_CLIENT_CAPABILITIES,
            }),
            deadline,
        )?;
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
            limits,
            connection,
            negotiated,
            deadline,
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

    pub fn limits(&self) -> BridgetClientLimits {
        self.limits
    }

    /// Demande l'identité attestée du daemon sur cette connexion `Client`.
    ///
    /// La négociation ayant déjà eu lieu au constructeur, cette sonde est un
    /// unique aller-retour. Une réponse absente, rejetée ou d'un autre type
    /// reste une erreur distincte : l'appelant peut refuser une écriture sans
    /// confondre un daemon muet avec un contrat incompatible.
    pub fn daemon_identity(&mut self) -> Result<DaemonIdentity, BridgetClientError> {
        let response = self.request(json!({"type": "DaemonIdentityRequest"}))?;
        parse_daemon_identity(response)
    }

    /// Envoie l'enveloppe exacte fournie par l'outbox, sans generer ni muter
    /// le `message_id` client.
    pub fn send_idempotent(
        &mut self,
        message: &PublicMessage,
        message_id: &str,
        issued_at: i64,
    ) -> Result<IdempotencyIssue, BridgetClientError> {
        validate_send_idempotent_frame(
            message,
            message_id,
            issued_at,
            self.limits.max_frame_bytes,
        )?;
        let response = self.request(json!({
            "type": "SendIdempotent",
            "message": message,
            "message_id": message_id,
            "issued_at": issued_at,
        }))?;
        let issue = parse_idempotency_issue(response, message_id);
        self.connection.poison_after(&issue);
        issue
    }

    /// Rejoue les octets filaires déjà persistés par l'outbox Maicie.
    ///
    /// Cette voie est limitée au crate : contrairement à une nouvelle
    /// délégation, elle tolère les champs ajoutés ultérieurement au message
    /// public afin qu'une reprise ne transforme jamais une ancienne enveloppe
    /// valide en message poison. Les octets sont insérés tels quels dans la
    /// commande `SendIdempotent` ; seuls les champs stables sont vérifiés.
    pub(crate) fn replay_idempotent_bytes(
        &mut self,
        message_bytes: &[u8],
        message_id: &str,
        issued_at: i64,
    ) -> Result<IdempotencyIssue, BridgetClientError> {
        self.replay_idempotent_bytes_observed(message_bytes, message_id, issued_at, |_| {})
    }

    /// Variante réservée aux tests de frontière : l'observateur n'altère ni
    /// les octets ni la machine d'état du client.
    pub(crate) fn replay_idempotent_bytes_observed(
        &mut self,
        message_bytes: &[u8],
        message_id: &str,
        issued_at: i64,
        mut observer: impl FnMut(ReplayIdempotentPhase),
    ) -> Result<IdempotencyIssue, BridgetClientError> {
        let replay: ReplayPublicMessage =
            serde_json::from_slice(message_bytes).map_err(|source| BridgetClientError::Decode {
                line: String::from_utf8_lossy(message_bytes).into_owned(),
                source,
            })?;
        let message = replay.into_public_message();
        message.validate()?;
        if message.id != message_id {
            return Err(BridgetClientError::InvalidEnvelope(
                "message.id et message_id doivent etre identiques".to_string(),
            ));
        }
        let request = replay_idempotent_request(message_bytes, message_id, issued_at)?;
        let response = match self.deadline {
            Some(deadline) => {
                self.connection
                    .request_raw_json_until_observed(&request, deadline, || {
                        observer(ReplayIdempotentPhase::AfterWriteBeforeAck)
                    })
            }
            None => {
                let deadline = Instant::now() + self.limits.io_timeout;
                self.connection
                    .request_raw_json_until_observed(&request, deadline, || {
                        observer(ReplayIdempotentPhase::AfterWriteBeforeAck)
                    })
            }
        }?;
        let issue = parse_idempotency_issue(response, message_id);
        self.connection.poison_after(&issue);
        issue
    }

    /// Lit l'issue durable deja associee a une cle d'envoi Maicie.
    pub fn lookup(&mut self, message_id: &str) -> Result<IdempotencyIssue, BridgetClientError> {
        if message_id.trim().is_empty() {
            return Err(BridgetClientError::InvalidEnvelope(
                "message_id ne peut pas etre vide".to_string(),
            ));
        }
        let response = self.request(json!({
            "type": "Lookup",
            "operation_kind": "send",
            "idempotency_key": message_id,
        }))?;
        let issue = parse_idempotency_issue(response, message_id);
        self.connection.poison_after(&issue);
        issue
    }

    fn request(&mut self, value: Value) -> Result<Value, BridgetClientError> {
        request_with_deadline(&mut self.connection, value, self.deadline)
    }

    /// Lit l'annuaire public Bridget sur une connexion ponctuelle non mutante.
    pub fn list_agents(&self) -> Result<Vec<AgentInfo>, BridgetClientError> {
        Self::list_agents_at_with_limits(&self.socket_path, self.limits)
    }

    /// Variante bornée par une échéance absolue. Le budget appartient à
    /// l'appelant : une consultation composée ne repart donc pas avec un
    /// nouveau délai à chaque échange du protocole local public.
    pub fn list_agents_until(
        &self,
        deadline: Instant,
    ) -> Result<Vec<AgentInfo>, BridgetClientError> {
        Self::list_agents_at_with_limits_until(&self.socket_path, self.limits, deadline)
    }

    /// Lit l'annuaire avant toute negociation client. Cette operation reste
    /// disponible pour expliquer une incompatibilite de capacites plutot que
    /// de masquer les agents presents.
    pub fn list_agents_at(
        socket_path: impl AsRef<Path>,
    ) -> Result<Vec<AgentInfo>, BridgetClientError> {
        Self::list_agents_at_with_limits(socket_path, BridgetClientLimits::default())
    }

    pub fn list_agents_at_with_limits(
        socket_path: impl AsRef<Path>,
        limits: BridgetClientLimits,
    ) -> Result<Vec<AgentInfo>, BridgetClientError> {
        Self::list_agents_at_with_limits_until(
            socket_path,
            limits,
            monotonic_now() + limits.connect_timeout + limits.io_timeout,
        )
    }

    fn list_agents_at_with_limits_until(
        socket_path: impl AsRef<Path>,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Vec<AgentInfo>, BridgetClientError> {
        let mut connection = WireConnection::connect(socket_path.as_ref(), limits, deadline)?;
        let response = connection.request_until(json!({"type": "ListAgents"}), deadline)?;
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

    /// Agrège les échantillons d'usage d'un agent dans une fenêtre fermée.
    ///
    /// `Ok(None)` = aucun échantillon attesté → le greffe rend « inconnu ».
    /// Une erreur réseau laisse aussi le greffe sur « inconnu » : la clôture
    /// ne doit jamais inventer un zéro ni échouer faute de sonde.
    pub fn usage_window(
        &self,
        agent: &str,
        from_secs: i64,
        to_secs: i64,
    ) -> Result<Option<UsageWindowAggregate>, BridgetClientError> {
        Self::usage_window_at_with_limits(&self.socket_path, self.limits, agent, from_secs, to_secs)
    }

    pub fn usage_window_at_with_limits(
        socket_path: impl AsRef<Path>,
        limits: BridgetClientLimits,
        agent: &str,
        from_secs: i64,
        to_secs: i64,
    ) -> Result<Option<UsageWindowAggregate>, BridgetClientError> {
        let deadline = monotonic_now() + limits.connect_timeout + limits.io_timeout;
        let mut connection = WireConnection::connect(socket_path.as_ref(), limits, deadline)?;
        let role = connection.request_until(
            json!({"type": "RoleHandshake", "role": "wrapper"}),
            deadline,
        )?;
        expect_role_accepted(&role, "wrapper")?;
        let response = connection.request_until(
            json!({
                "type": "UsageWindow",
                "agent": agent,
                "from_secs": from_secs,
                "to_secs": to_secs,
            }),
            deadline,
        )?;
        match response_type(&response)? {
            "UsageWindowResult" => {
                let aggregate = response.get("aggregate").cloned().unwrap_or(Value::Null);
                if aggregate.is_null() {
                    return Ok(None);
                }
                serde_json::from_value(aggregate)
                    .map(Some)
                    .map_err(|source| BridgetClientError::Decode {
                        line: response.to_string(),
                        source,
                    })
            }
            "Nack" => Err(parse_nack(response)?),
            other => Err(unexpected("UsageWindowResult", other)),
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
        let mut connection = WireConnection::connect(
            &self.socket_path,
            self.limits,
            monotonic_now() + self.limits.connect_timeout,
        )?;
        let role = connection.request(json!({"type": "RoleHandshake", "role": "wrapper"}))?;
        expect_role_accepted(&role, "wrapper")?;
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
        self.subscribe_until(
            agent,
            window,
            monotonic_now() + self.limits.connect_timeout + self.limits.io_timeout * 2,
        )
    }

    /// Ouvre Subscribe en consommant la même échéance absolue pour la
    /// connexion, le rôle Attach et l'écriture de l'abonnement.
    pub fn subscribe_until(
        &self,
        agent: &str,
        window: AttachWindow,
        deadline: Instant,
    ) -> Result<Subscription, BridgetClientError> {
        let mut connection = WireConnection::connect(&self.socket_path, self.limits, deadline)?;
        let role = connection
            .request_until(json!({"type": "RoleHandshake", "role": "attach"}), deadline)?;
        expect_role_accepted(&role, "attach")?;
        connection.send_until(
            json!({
                "type": "Subscribe",
                "agent": agent,
                "window": window,
            }),
            deadline,
        )?;
        Ok(Subscription { connection })
    }

    /// Emet un SpawnOrder deja approuve. Cette methode ne construit aucun
    /// processus et ne relance pas d'elle-meme l'ordre en cas d'incertitude.
    pub fn spawn_order(&self, order: &SpawnOrder) -> Result<SpawnOutcome, BridgetClientError> {
        let mut connection = WireConnection::connect(
            &self.socket_path,
            self.limits,
            monotonic_now() + self.limits.connect_timeout,
        )?;
        let role = connection.request(json!({"type": "RoleHandshake", "role": "wrapper"}))?;
        expect_role_accepted(&role, "wrapper")?;
        let response = connection.request(json!({
            "type": "SpawnOrder",
            "agent_type": order.agent_type,
            "name": order.name,
            "cwd": order.cwd,
            "persistent": order.persistent,
            "command_id": order.command_id,
            "issued_at": order.issued_at,
            "deadline_at": order.deadline_at,
            "project": order.project,
        }))?;
        Ok(parse_spawn_replay(response, &order.command_id)?.outcome)
    }

    /// Le rejeu exact tient lieu de lookup SpawnOrder : le protocole 009 ne
    /// publie volontairement pas de lecture séparée dans la portée interne du
    /// superviseur. Les octets approuvés traversent donc cette frontière sans
    /// désérialisation/résérialisation qui pourrait altérer leur canon.
    pub fn replay_spawn_order_bytes(
        &self,
        spawn_order_bytes: &[u8],
    ) -> Result<SpawnReplay, BridgetClientError> {
        Self::replay_spawn_order_bytes_at(&self.socket_path, self.limits, spawn_order_bytes)
    }

    /// Variante de reprise qui n'ouvre aucune négociation client 012 : un
    /// SpawnOrder appartient au rôle public `wrapper` de la session 009.
    pub fn replay_spawn_order_bytes_at(
        socket_path: impl AsRef<Path>,
        limits: BridgetClientLimits,
        spawn_order_bytes: &[u8],
    ) -> Result<SpawnReplay, BridgetClientError> {
        let command_id = spawn_command_id(spawn_order_bytes)?;
        let mut connection = WireConnection::connect(
            socket_path.as_ref(),
            limits,
            monotonic_now() + limits.connect_timeout,
        )?;
        let role = connection.request(json!({"type": "RoleHandshake", "role": "wrapper"}))?;
        expect_role_accepted(&role, "wrapper")?;
        let response = connection.request_raw_json(spawn_order_bytes)?;
        parse_spawn_replay(response, &command_id)
    }
}

impl GuichetClient {
    /// Ouvre le rôle de service et négocie exclusivement la capacité guichet.
    pub fn connect(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_limits(socket_path, issuer_scope, BridgetClientLimits::default())
    }

    pub fn connect_with_limits(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, None)
    }

    /// Variante de relève : connexion, négociation et chaque requête consomment
    /// la même échéance absolue détenue par la commande locale appelante.
    pub fn connect_with_limits_until(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, Some(deadline))
    }

    fn connect_with_optional_deadline(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Option<Instant>,
    ) -> Result<Self, BridgetClientError> {
        validate_limits(limits)?;
        let socket_path = socket_path.as_ref().to_path_buf();
        let issuer_scope = issuer_scope.into();
        if issuer_scope.len() < 16 || issuer_scope.trim().is_empty() {
            return Err(BridgetClientError::InvalidEnvelope(
                "issuer_scope guichet doit contenir au moins 128 bits opaques".to_string(),
            ));
        }
        let connect_deadline = deadline.unwrap_or_else(|| monotonic_now() + limits.connect_timeout);
        let mut connection = WireConnection::connect(&socket_path, limits, connect_deadline)?;
        let role = request_raw_with_deadline(
            &mut connection,
            &canonical_service_role_handshake()?,
            deadline,
        )?;
        expect_role_accepted(&role, "service")?;
        let welcome = request_raw_with_deadline(
            &mut connection,
            &canonical_service_hello(&issuer_scope)?,
            deadline,
        )?;
        let negotiated = parse_service_welcome(welcome)?;
        if negotiated.version != GUICHET_CONTRACT_VERSION {
            return Err(BridgetClientError::VersionUnsupported {
                requested: GUICHET_CONTRACT_VERSION,
                received: negotiated.version,
            });
        }
        if !negotiated
            .capabilities
            .contains(REQUIRED_GUICHET_CAPABILITY)
        {
            return Err(BridgetClientError::CapabilityMissing {
                capability: REQUIRED_GUICHET_CAPABILITY.to_string(),
            });
        }
        Ok(Self {
            socket_path,
            issuer_scope,
            limits,
            connection,
            negotiated,
            deadline,
        })
    }

    pub fn issuer_scope(&self) -> &str {
        &self.issuer_scope
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn negotiated(&self) -> &NegotiatedGuichetContract {
        &self.negotiated
    }

    pub fn limits(&self) -> BridgetClientLimits {
        self.limits
    }

    /// Relève exactement une entrée FIFO, ou constate explicitement un guichet
    /// vide. La sélection, le claim et son token restent côté Bridget.
    pub fn claim_next(&mut self) -> Result<Option<GuichetClaim>, BridgetClientError> {
        let response = self.request(json!({"type": "guichet_claim_next", "v": 1}))?;
        parse_guichet_claim(response)
    }

    /// Rejoue le claim strictement possédé par cette connexion, sans prolonger
    /// son lease ni lui attribuer une nouvelle génération.
    pub fn claim(
        &mut self,
        issuer_scope: &str,
        request_id: &str,
        claim_token: &str,
    ) -> Result<GuichetClaim, BridgetClientError> {
        let response = self.request(json!({
            "type": "guichet_claim",
            "v": 1,
            "issuer_scope": issuer_scope,
            "request_id": request_id,
            "claim_token": claim_token,
        }))?;
        parse_guichet_claim(response)?.ok_or_else(|| {
            BridgetClientError::Protocol(
                "guichet_claim ne peut pas retourner guichet_empty".to_string(),
            )
        })
    }

    /// Consulte l'issue d'une demande sans jamais créer une nouvelle entrée.
    pub fn lookup(
        &mut self,
        issuer_scope: &str,
        request_id: &str,
    ) -> Result<GuichetResult, BridgetClientError> {
        let response = self.request(json!({
            "type": "guichet_lookup",
            "v": 1,
            "issuer_scope": issuer_scope,
            "request_id": request_id,
        }))?;
        parse_guichet_result(response)
    }

    /// Envoie les octets canoniques déjà persistés d'une réponse de guichet.
    /// Cette méthode est la voie de retry : elle ne désérialise/résérialise pas
    /// l'enveloppe et préserve donc token, génération et charge à l'octet.
    pub fn reply_exact_bytes(
        &mut self,
        response_bytes: &[u8],
    ) -> Result<GuichetResult, BridgetClientError> {
        validate_guichet_reply_bytes(response_bytes, self.limits.max_frame_bytes)?;
        let response = self.request_raw_json(response_bytes)?;
        parse_guichet_result(response)
    }

    /// Lit un événement poussé par Bridget après une réponse durable ou une
    /// transition terminale. Il n'existe aucune émission cliente de cet événement.
    pub fn next_lifecycle_event(&mut self) -> Result<GuichetLifecycleEvent, BridgetClientError> {
        let deadline = self
            .deadline
            .unwrap_or_else(|| monotonic_now() + self.limits.io_timeout);
        let event = self.connection.receive_until(deadline);
        self.connection.poison_after(&event);
        parse_guichet_lifecycle_event(event?)
    }

    fn request(&mut self, value: Value) -> Result<Value, BridgetClientError> {
        request_with_deadline(&mut self.connection, value, self.deadline)
    }

    fn request_raw_json(&mut self, json_bytes: &[u8]) -> Result<Value, BridgetClientError> {
        match self.deadline {
            Some(deadline) => self.connection.request_raw_json_until(json_bytes, deadline),
            None => self.connection.request_raw_json(json_bytes),
        }
    }
}

impl ProjectRegistryClient {
    /// Ouvre le rôle `service` et négocie exclusivement `project_registry_v1`.
    pub fn connect(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_limits(socket_path, issuer_scope, BridgetClientLimits::default())
    }

    pub fn connect_with_limits(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, None)
    }

    pub fn connect_with_limits_until(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Self, BridgetClientError> {
        Self::connect_with_optional_deadline(socket_path, issuer_scope, limits, Some(deadline))
    }

    fn connect_with_optional_deadline(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Option<Instant>,
    ) -> Result<Self, BridgetClientError> {
        validate_limits(limits)?;
        let issuer_scope = issuer_scope.into();
        if issuer_scope.len() < 16 || issuer_scope.trim().is_empty() {
            return Err(BridgetClientError::InvalidEnvelope(
                "issuer_scope registre projet doit contenir au moins 128 bits opaques".to_string(),
            ));
        }
        let connect_deadline = deadline.unwrap_or_else(|| monotonic_now() + limits.connect_timeout);
        let mut connection =
            WireConnection::connect(socket_path.as_ref(), limits, connect_deadline)?;
        let role = request_raw_with_deadline(
            &mut connection,
            &canonical_service_role_handshake()?,
            deadline,
        )?;
        expect_role_accepted(&role, "service")?;
        let welcome = request_raw_with_deadline(
            &mut connection,
            &canonical_project_registry_service_hello(&issuer_scope)?,
            deadline,
        )?;
        let negotiated = parse_service_welcome(welcome)?;
        if negotiated.version != GUICHET_CONTRACT_VERSION {
            return Err(BridgetClientError::VersionUnsupported {
                requested: GUICHET_CONTRACT_VERSION,
                received: negotiated.version,
            });
        }
        if !negotiated
            .capabilities
            .contains(REQUIRED_PROJECT_REGISTRY_CAPABILITY)
        {
            return Err(BridgetClientError::CapabilityMissing {
                capability: REQUIRED_PROJECT_REGISTRY_CAPABILITY.to_string(),
            });
        }
        Ok(Self {
            connection,
            deadline,
            limits,
        })
    }

    /// Envoie les octets canoniques d'un `ProjectBindRequest` déjà persisté et
    /// retourne l'issue exacte, corrélée à la commande sans aucune lecture de
    /// stockage Bridget.
    pub fn bind_exact_bytes(
        &mut self,
        request_bytes: &[u8],
    ) -> Result<bridget_transport::protocol::ProjectBindOutcome, BridgetClientError> {
        let request = replay_project_bind_request(request_bytes)?;
        let frame = project_registry_request_frame(request_bytes)?;
        let response = match self.deadline {
            Some(deadline) => self.connection.request_raw_json_until(&frame, deadline),
            None => self.connection.request_raw_json(&frame),
        }?;
        parse_project_registry_outcome(response, &request.command_id)
    }

    /// Exécute une lecture ou mutation administrative sur le même contrat
    /// dédié. Les réponses restent corrélées par command_id et le client ne
    /// possède toujours aucun accès au stockage privé de Bridget.
    pub fn administer(
        &mut self,
        request: &bridget_transport::protocol::ProjectAdminRequest,
    ) -> Result<bridget_transport::protocol::ProjectAdminOutcome, BridgetClientError> {
        let request_bytes = serde_json::to_vec(request).map_err(BridgetClientError::Encode)?;
        let frame = project_registry_admin_request_frame(&request_bytes)?;
        let response = match self.deadline {
            Some(deadline) => self.connection.request_raw_json_until(&frame, deadline),
            None => self.connection.request_raw_json(&frame),
        }?;
        parse_project_registry_admin_outcome(response, &request.command_id)
    }

    pub fn limits(&self) -> BridgetClientLimits {
        self.limits
    }
}

impl CoordinationClient {
    pub fn connect_with_limits_until(
        socket_path: impl AsRef<Path>,
        issuer_scope: impl Into<String>,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Self, BridgetClientError> {
        validate_limits(limits)?;
        let issuer_scope = issuer_scope.into();
        if issuer_scope.len() < 16 || issuer_scope.trim().is_empty() {
            return Err(BridgetClientError::InvalidEnvelope(
                "issuer_scope coordination doit contenir au moins 128 bits opaques".to_string(),
            ));
        }
        let mut connection = WireConnection::connect(socket_path.as_ref(), limits, deadline)?;
        let role = request_raw_with_deadline(
            &mut connection,
            &canonical_service_role_handshake()?,
            Some(deadline),
        )?;
        expect_role_accepted(&role, "service")?;
        let welcome = request_raw_with_deadline(
            &mut connection,
            &canonical_coordination_service_hello(&issuer_scope)?,
            Some(deadline),
        )?;
        let negotiated = parse_service_welcome(welcome)?;
        if negotiated.version != GUICHET_CONTRACT_VERSION {
            return Err(BridgetClientError::VersionUnsupported {
                requested: GUICHET_CONTRACT_VERSION,
                received: negotiated.version,
            });
        }
        for capability in [
            REQUIRED_GUICHET_CAPABILITY,
            REQUIRED_COORDINATION_CAPABILITY,
        ] {
            if !negotiated.capabilities.contains(capability) {
                return Err(BridgetClientError::CapabilityMissing {
                    capability: capability.to_string(),
                });
            }
        }
        Ok(Self {
            connection,
            deadline,
        })
    }

    /// Envoie une seule demande de snapshot puis lit au plus `max_items`
    /// trames. La borne inclut les terminaux guichet poussés avant le snapshot.
    /// L'absence de frontière terminale est rendue comme une erreur explicite :
    /// l'appelant ne peut donc jamais déclarer ces événements frais.
    pub fn snapshot_after(
        &mut self,
        after_cursor: Option<u64>,
        max_items: usize,
    ) -> Result<Vec<CoordinationStreamItem>, BridgetClientError> {
        if max_items == 0 {
            return Err(BridgetClientError::InvalidLimits(
                "max_items coordination doit être strictement positif".to_string(),
            ));
        }
        self.connection.send_bytes_until(
            &canonical_coordination_subscribe(after_cursor)?,
            self.deadline,
        )?;
        let mut items = Vec::new();
        while items.len() < max_items {
            let (canonical_bytes, frame) = self.connection.receive_raw_until(self.deadline)?;
            let item = parse_coordination_stream_item(frame, canonical_bytes)?;
            let terminal = matches!(
                item,
                CoordinationStreamItem::SnapshotCaughtUp { .. }
                    | CoordinationStreamItem::Gap { .. }
                    | CoordinationStreamItem::Unavailable { .. }
            );
            items.push(item);
            if terminal {
                return Ok(items);
            }
        }
        Err(BridgetClientError::ItemLimitExceeded { max_items })
    }
}

#[derive(Deserialize)]
struct PersistedSpawnOrder {
    #[serde(rename = "type")]
    kind: String,
    command_id: String,
    agent_type: String,
    name: Option<String>,
    cwd: String,
    persistent: bool,
    issued_at: i64,
    deadline_at: i64,
    #[serde(default)]
    project: Option<ProjectReference>,
}

fn spawn_command_id(bytes: &[u8]) -> Result<String, BridgetClientError> {
    let order: PersistedSpawnOrder =
        serde_json::from_slice(bytes).map_err(|source| BridgetClientError::Decode {
            line: String::from_utf8_lossy(bytes).into_owned(),
            source,
        })?;
    if order.kind != "SpawnOrder"
        || order.command_id.trim().is_empty()
        || order.agent_type.trim().is_empty()
        || order.cwd.is_empty()
        || order.issued_at <= 0
        || order.deadline_at <= order.issued_at
        || order.project.as_ref().is_some_and(|project| {
            project.project_id.trim().is_empty() || project.binding_generation == 0
        })
    {
        return Err(BridgetClientError::InvalidEnvelope(
            "SpawnOrder persiste invalide".to_string(),
        ));
    }
    let _ = (order.name, order.persistent, order.project);
    Ok(order.command_id)
}

fn parse_spawn_replay(
    response: Value,
    expected_command_id: &str,
) -> Result<SpawnReplay, BridgetClientError> {
    match response_type(&response)? {
        "SpawnAccepted" => {
            let command_id = required_string(&response, "command_id")?;
            if command_id != expected_command_id {
                return Err(BridgetClientError::Protocol(
                    "SpawnAccepted avec command_id divergent".to_string(),
                ));
            }
            let definition_digest = response
                .get("definition")
                .and_then(Value::as_object)
                .and_then(|definition| definition.get("digest"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            Ok(SpawnReplay {
                outcome: SpawnOutcome::Accepted {
                    command_id,
                    name: required_string(&response, "name")?,
                },
                definition_digest,
            })
        }
        "SpawnRejected" => Ok(SpawnReplay {
            outcome: SpawnOutcome::Rejected {
                command_id: required_string(&response, "command_id")?,
                reason: response.get("reason").cloned().ok_or_else(|| {
                    BridgetClientError::Protocol("SpawnRejected sans reason".to_string())
                })?,
            },
            definition_digest: None,
        }),
        "IdempotencyResult" => Ok(SpawnReplay {
            outcome: SpawnOutcome::Idempotency(parse_idempotency_issue(
                response,
                expected_command_id,
            )?),
            definition_digest: None,
        }),
        "Nack" => Err(parse_nack(response)?),
        other => Err(unexpected("SpawnAccepted/SpawnRejected", other)),
    }
}

fn request_with_deadline(
    connection: &mut WireConnection,
    value: Value,
    deadline: Option<Instant>,
) -> Result<Value, BridgetClientError> {
    match deadline {
        Some(deadline) => connection.request_until(value, deadline),
        None => connection.request(value),
    }
}

fn request_raw_with_deadline(
    connection: &mut WireConnection,
    bytes: &[u8],
    deadline: Option<Instant>,
) -> Result<Value, BridgetClientError> {
    match deadline {
        Some(deadline) => connection.request_raw_json_until(bytes, deadline),
        None => connection.request_raw_json(bytes),
    }
}

#[derive(Serialize)]
struct CanonicalServiceRoleHandshake<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    role: &'a str,
}

#[derive(Serialize)]
struct CanonicalServiceHello<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    version: u16,
    service: &'a str,
    issuer_scope: &'a str,
    capabilities: &'a [&'a str],
}

#[derive(Serialize)]
struct CanonicalCoordinationSubscribe {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(rename = "v")]
    version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    after_cursor: Option<u64>,
}

fn canonical_service_role_handshake() -> Result<Vec<u8>, BridgetClientError> {
    serde_json::to_vec(&CanonicalServiceRoleHandshake {
        kind: "RoleHandshake",
        role: "service",
    })
    .map_err(BridgetClientError::Encode)
}

fn canonical_service_hello(issuer_scope: &str) -> Result<Vec<u8>, BridgetClientError> {
    let capabilities = [REQUIRED_GUICHET_CAPABILITY];
    serde_json::to_vec(&CanonicalServiceHello {
        kind: "ServiceHello",
        version: GUICHET_CONTRACT_VERSION,
        service: "maicie",
        issuer_scope,
        capabilities: &capabilities,
    })
    .map_err(BridgetClientError::Encode)
}

fn canonical_project_registry_service_hello(
    issuer_scope: &str,
) -> Result<Vec<u8>, BridgetClientError> {
    let capabilities = [REQUIRED_PROJECT_REGISTRY_CAPABILITY];
    serde_json::to_vec(&CanonicalServiceHello {
        kind: "ServiceHello",
        version: GUICHET_CONTRACT_VERSION,
        service: "maicie",
        issuer_scope,
        capabilities: &capabilities,
    })
    .map_err(BridgetClientError::Encode)
}

fn canonical_coordination_service_hello(issuer_scope: &str) -> Result<Vec<u8>, BridgetClientError> {
    let capabilities = [
        REQUIRED_GUICHET_CAPABILITY,
        REQUIRED_COORDINATION_CAPABILITY,
    ];
    serde_json::to_vec(&CanonicalServiceHello {
        kind: "ServiceHello",
        version: GUICHET_CONTRACT_VERSION,
        service: "maicie",
        issuer_scope,
        capabilities: &capabilities,
    })
    .map_err(BridgetClientError::Encode)
}

fn canonical_coordination_subscribe(
    after_cursor: Option<u64>,
) -> Result<Vec<u8>, BridgetClientError> {
    serde_json::to_vec(&CanonicalCoordinationSubscribe {
        kind: "coordination_subscribe",
        version: COORDINATION_STREAM_VERSION,
        after_cursor,
    })
    .map_err(BridgetClientError::Encode)
}

/// Vérifie la trame complète `SendIdempotent` avant sa persistance par
/// l'outbox. La borne inclut toujours le délimiteur JSONL final.
pub(crate) fn validate_send_idempotent_frame(
    message: &PublicMessage,
    message_id: &str,
    issued_at: i64,
    max_frame_bytes: usize,
) -> Result<(), BridgetClientError> {
    message.validate()?;
    if message.id != message_id {
        return Err(BridgetClientError::InvalidEnvelope(
            "message.id et message_id doivent etre identiques".to_string(),
        ));
    }
    let message_bytes = serde_json::to_vec(message).map_err(BridgetClientError::Encode)?;
    let encoded = replay_idempotent_request(&message_bytes, message_id, issued_at)?;
    if encoded.len().saturating_add(1) > max_frame_bytes {
        return Err(BridgetClientError::FrameTooLarge { max_frame_bytes });
    }
    Ok(())
}

fn replay_idempotent_request(
    message_bytes: &[u8],
    message_id: &str,
    issued_at: i64,
) -> Result<Vec<u8>, BridgetClientError> {
    let mut request = Vec::with_capacity(message_bytes.len() + 128);
    request.extend_from_slice(br#"{"type":"SendIdempotent","message":"#);
    request.extend_from_slice(message_bytes);
    request.extend_from_slice(br#","message_id":"#);
    serde_json::to_writer(&mut request, message_id).map_err(BridgetClientError::Encode)?;
    request.extend_from_slice(br#","issued_at":"#);
    serde_json::to_writer(&mut request, &issued_at).map_err(BridgetClientError::Encode)?;
    request.push(b'}');
    Ok(request)
}

fn replay_project_bind_request(
    request_bytes: &[u8],
) -> Result<bridget_transport::protocol::ProjectBindRequest, BridgetClientError> {
    serde_json::from_slice(request_bytes).map_err(|source| BridgetClientError::Decode {
        line: String::from_utf8_lossy(request_bytes).into_owned(),
        source,
    })
}

fn project_registry_request_frame(request_bytes: &[u8]) -> Result<Vec<u8>, BridgetClientError> {
    let request = replay_project_bind_request(request_bytes)?;
    if request.command_id.trim().is_empty()
        || request.project_id.trim().is_empty()
        || request.requested_root.trim().is_empty()
        || request.deadline_at < request.issued_at
    {
        return Err(BridgetClientError::InvalidEnvelope(
            "ProjectBindRequest incomplet ou invalide".to_string(),
        ));
    }
    let mut frame = Vec::with_capacity(request_bytes.len() + 48);
    frame.extend_from_slice(br#"{"type":"project_registry_request","request":"#);
    frame.extend_from_slice(request_bytes);
    frame.push(b'}');
    Ok(frame)
}

fn project_registry_admin_request_frame(
    request_bytes: &[u8],
) -> Result<Vec<u8>, BridgetClientError> {
    let request: bridget_transport::protocol::ProjectAdminRequest =
        serde_json::from_slice(request_bytes).map_err(|source| BridgetClientError::Decode {
            line: String::from_utf8_lossy(request_bytes).into_owned(),
            source,
        })?;
    if request.command_id.trim().is_empty() || request.deadline_at < request.issued_at {
        return Err(BridgetClientError::InvalidEnvelope(
            "ProjectAdminRequest incomplet ou invalide".to_string(),
        ));
    }
    let mut frame = Vec::with_capacity(request_bytes.len() + 54);
    frame.extend_from_slice(br#"{"type":"project_registry_admin_request","request":"#);
    frame.extend_from_slice(request_bytes);
    frame.push(b'}');
    Ok(frame)
}

/// Connexion attach dont la lecture est la seule source d'evenements runtime.
pub struct Subscription {
    connection: WireConnection,
}

impl Subscription {
    pub fn next_event(&mut self) -> Result<SubscriptionEvent, BridgetClientError> {
        let response = self.connection.receive()?;
        parse_subscription_event(response)
    }

    /// Lit un événement sans dépasser l'échéance absolue du consommateur.
    pub fn next_event_until(
        &mut self,
        deadline: Instant,
    ) -> Result<SubscriptionEvent, BridgetClientError> {
        self.connection.ensure_usable()?;
        let result = self
            .connection
            .receive_until(deadline)
            .and_then(parse_subscription_event);
        self.connection.poison_after(&result);
        result
    }
}

fn parse_subscription_event(response: Value) -> Result<SubscriptionEvent, BridgetClientError> {
    match response_type(&response)? {
        "Subscribed" => Ok(SubscriptionEvent::Subscribed {
            subscription_id: required_string(&response, "subscription_id")?,
        }),
        "JournalFragment" => Ok(SubscriptionEvent::JournalFragment {
            subscription_id: required_string(&response, "subscription_id")?,
            seq: required_u64(&response, "seq")?,
            offset: required_u64(&response, "offset")?,
            final_fragment: required_bool(&response, "final")?,
            bytes: decode_base64_bytes(&response, "bytes")?,
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
        "JournalReadError" => Ok(SubscriptionEvent::JournalReadError {
            subscription_id: required_string(&response, "subscription_id")?,
            line: required_u64(&response, "line")?,
            offset: required_u64(&response, "offset")?,
            reason: required_string(&response, "reason")?,
        }),
        "End" => Ok(SubscriptionEvent::End {
            subscription_id: required_string(&response, "subscription_id")?,
            reason: required_string(&response, "reason")?,
        }),
        "AttachRejected" | "Nack" => Err(BridgetClientError::Protocol(response.to_string())),
        other => Err(unexpected("evenement d'abonnement", other)),
    }
}

struct WireConnection {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    limits: BridgetClientLimits,
    unusable: bool,
}

impl WireConnection {
    fn connect(
        path: &Path,
        limits: BridgetClientLimits,
        deadline: Instant,
    ) -> Result<Self, BridgetClientError> {
        validate_limits(limits)?;
        let stream = connect_nonblocking(path, deadline)?;
        let reader = BufReader::new(stream.try_clone().map_err(BridgetClientError::Read)?);
        Ok(Self {
            reader,
            writer: stream,
            limits,
            unusable: false,
        })
    }

    fn request(&mut self, value: Value) -> Result<Value, BridgetClientError> {
        let deadline = monotonic_now() + self.limits.io_timeout;
        self.request_until(value, deadline)
    }

    fn request_until(
        &mut self,
        value: Value,
        deadline: Instant,
    ) -> Result<Value, BridgetClientError> {
        self.ensure_usable()?;
        let result = (|| {
            self.send_until(value, deadline)?;
            self.receive_until(deadline)
        })();
        self.poison_after(&result);
        result
    }

    fn request_raw_json(&mut self, json_bytes: &[u8]) -> Result<Value, BridgetClientError> {
        let deadline = monotonic_now() + self.limits.io_timeout;
        self.request_raw_json_until(json_bytes, deadline)
    }

    fn request_raw_json_until(
        &mut self,
        json_bytes: &[u8],
        deadline: Instant,
    ) -> Result<Value, BridgetClientError> {
        self.request_raw_json_until_observed(json_bytes, deadline, || {})
    }

    fn request_raw_json_until_observed(
        &mut self,
        json_bytes: &[u8],
        deadline: Instant,
        mut observer: impl FnMut(),
    ) -> Result<Value, BridgetClientError> {
        self.ensure_usable()?;
        let result = (|| {
            self.send_bytes_until(json_bytes, deadline)?;
            observer();
            self.receive_until(deadline)
        })();
        self.poison_after(&result);
        result
    }

    fn send_until(&mut self, value: Value, deadline: Instant) -> Result<(), BridgetClientError> {
        let bytes = serde_json::to_vec(&value).map_err(BridgetClientError::Encode)?;
        self.send_bytes_until(&bytes, deadline)
    }

    fn send_bytes_until(
        &mut self,
        json_bytes: &[u8],
        deadline: Instant,
    ) -> Result<(), BridgetClientError> {
        if json_bytes.len() + 1 > self.limits.max_frame_bytes {
            return Err(BridgetClientError::FrameTooLarge {
                max_frame_bytes: self.limits.max_frame_bytes,
            });
        }
        let mut bytes = Vec::with_capacity(json_bytes.len() + 1);
        bytes.extend_from_slice(json_bytes);
        bytes.push(b'\n');
        let mut written = 0;
        while written < bytes.len() {
            wait_for_socket(
                self.writer.as_raw_fd(),
                libc::POLLOUT,
                deadline,
                "ecriture socket",
            )?;
            match self.writer.write(&bytes[written..]) {
                Ok(0) => return Err(BridgetClientError::Closed),
                Ok(count) => written += count,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(error) => return Err(write_error(error)),
            }
        }
        Ok(())
    }

    fn receive(&mut self) -> Result<Value, BridgetClientError> {
        self.ensure_usable()?;
        let result = self.receive_until(monotonic_now() + self.limits.io_timeout);
        self.poison_after(&result);
        result
    }

    fn receive_until(&mut self, deadline: Instant) -> Result<Value, BridgetClientError> {
        let (_, value) = self.receive_raw_until(deadline)?;
        Ok(value)
    }

    fn receive_raw_until(
        &mut self,
        deadline: Instant,
    ) -> Result<(Vec<u8>, Value), BridgetClientError> {
        self.ensure_usable()?;
        let mut frame = Vec::new();
        loop {
            // L'échéance est globale : une trame déjà dans BufReader ne doit
            // jamais contourner le budget en étant consommée après coup.
            remaining(deadline).map_err(|_| BridgetClientError::Timeout {
                operation: "lecture socket",
            })?;
            if self.reader.buffer().is_empty() {
                wait_for_socket(
                    self.reader.get_ref().as_raw_fd(),
                    libc::POLLIN,
                    deadline,
                    "lecture socket",
                )?;
            }
            let buffer = match self.reader.fill_buf() {
                Ok(buffer) => buffer,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(error) => return Err(read_error(error)),
            };
            if buffer.is_empty() {
                return Err(BridgetClientError::Closed);
            }
            if let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                if frame.len() + newline + 1 > self.limits.max_frame_bytes {
                    return Err(BridgetClientError::FrameTooLarge {
                        max_frame_bytes: self.limits.max_frame_bytes,
                    });
                }
                frame.extend_from_slice(&buffer[..newline]);
                self.reader.consume(newline + 1);
                break;
            }
            if frame.len() + buffer.len() >= self.limits.max_frame_bytes {
                return Err(BridgetClientError::FrameTooLarge {
                    max_frame_bytes: self.limits.max_frame_bytes,
                });
            }
            let consumed = buffer.len();
            frame.extend_from_slice(buffer);
            self.reader.consume(consumed);
        }
        let line = String::from_utf8(frame.clone()).map_err(|error| {
            BridgetClientError::Protocol(format!("trame Bridget non UTF-8: {error}"))
        })?;
        let value = serde_json::from_str(&line)
            .map_err(|source| BridgetClientError::Decode { line, source })?;
        Ok((frame, value))
    }

    fn ensure_usable(&self) -> Result<(), BridgetClientError> {
        if self.unusable {
            Err(BridgetClientError::ConnectionUnusable)
        } else {
            Ok(())
        }
    }

    fn poison_after<T>(&mut self, result: &Result<T, BridgetClientError>) {
        if matches!(
            result,
            Err(BridgetClientError::Read(_)
                | BridgetClientError::Write(_)
                | BridgetClientError::Decode { .. }
                | BridgetClientError::Closed
                | BridgetClientError::Protocol(_)
                | BridgetClientError::Timeout { .. }
                | BridgetClientError::FrameTooLarge { .. })
        ) {
            self.unusable = true;
        }
    }
}

fn connect_nonblocking(path: &Path, deadline: Instant) -> Result<UnixStream, BridgetClientError> {
    if monotonic_now() >= deadline {
        return Err(BridgetClientError::Timeout {
            operation: "connexion",
        });
    }

    let path_bytes = path.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path_bytes.len() >= address.sun_path.len() || path_bytes.contains(&0) {
        return Err(BridgetClientError::Connect {
            path: path.to_path_buf(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "socket Unix invalide"),
        });
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        address.sun_len = (std::mem::size_of::<libc::sa_family_t>() + path_bytes.len() + 1) as u8;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            path_bytes.as_ptr().cast(),
            address.sun_path.as_mut_ptr(),
            path_bytes.len(),
        );
    }

    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if fd < 0 {
        return Err(connect_error(path, std::io::Error::last_os_error()));
    }
    let close = || unsafe { libc::close(fd) };
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        let error = std::io::Error::last_os_error();
        close();
        return Err(connect_error(path, error));
    }

    let result = unsafe {
        libc::connect(
            fd,
            (&address as *const libc::sockaddr_un).cast(),
            std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    if result < 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            close();
            return Err(connect_error(path, error));
        }
        if let Err(error) = wait_for_socket(fd, libc::POLLOUT, deadline, "connexion") {
            close();
            return Err(error);
        }
        let mut socket_error: libc::c_int = 0;
        let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        let status = unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut socket_error as *mut libc::c_int).cast(),
                &mut length,
            )
        };
        if status < 0 {
            let error = std::io::Error::last_os_error();
            close();
            return Err(connect_error(path, error));
        }
        if socket_error != 0 {
            close();
            return Err(connect_error(
                path,
                std::io::Error::from_raw_os_error(socket_error),
            ));
        }
    }

    Ok(unsafe { UnixStream::from_raw_fd(fd) })
}

fn wait_for_socket(
    fd: libc::c_int,
    events: libc::c_short,
    deadline: Instant,
    operation: &'static str,
) -> Result<(), BridgetClientError> {
    loop {
        let timeout_ms = remaining(deadline)
            .map_err(|_| BridgetClientError::Timeout { operation })?
            .as_millis()
            .min(libc::c_int::MAX as u128) as libc::c_int;
        let mut pollfd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let result = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
        if result == 0 {
            return Err(BridgetClientError::Timeout { operation });
        }
        if result < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(BridgetClientError::Read(error));
        }
        if pollfd.revents & libc::POLLNVAL != 0 {
            return Err(BridgetClientError::Read(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "descripteur socket invalide",
            )));
        }
        if pollfd.revents & (events | libc::POLLERR | libc::POLLHUP) != 0 {
            return Ok(());
        }
    }
}

fn connect_error(path: &Path, source: std::io::Error) -> BridgetClientError {
    BridgetClientError::Connect {
        path: path.to_path_buf(),
        source,
    }
}

fn validate_limits(limits: BridgetClientLimits) -> Result<(), BridgetClientError> {
    if limits.connect_timeout.is_zero() || limits.io_timeout.is_zero() {
        return Err(BridgetClientError::InvalidLimits(
            "les delais doivent etre strictement positifs".to_string(),
        ));
    }
    if limits.max_frame_bytes == 0 {
        return Err(BridgetClientError::InvalidLimits(
            "max_frame_bytes doit etre strictement positif".to_string(),
        ));
    }
    Ok(())
}

/// Source monotone unique des échéances I/O. La production utilise l'horloge
/// système ; les tests unitaires peuvent avancer une horloge contrôlée entre
/// deux phases filaires afin de prouver qu'aucune ne renouvelle le budget.
fn monotonic_now() -> Instant {
    #[cfg(test)]
    if let Some(clock) = test_clock_slot()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .as_ref()
        .cloned()
    {
        return clock.now();
    }
    Instant::now()
}

fn remaining(deadline: Instant) -> Result<Duration, BridgetClientError> {
    deadline
        .checked_duration_since(monotonic_now())
        .filter(|duration| !duration.is_zero())
        .ok_or(BridgetClientError::Timeout {
            operation: "I/O socket",
        })
}

#[cfg(test)]
struct TestMonotonicClock {
    base: Instant,
    elapsed_ms: std::sync::atomic::AtomicU64,
}

#[cfg(test)]
impl TestMonotonicClock {
    fn now(&self) -> Instant {
        self.base + Duration::from_millis(self.elapsed_ms.load(std::sync::atomic::Ordering::SeqCst))
    }

    fn advance(&self, duration: Duration) {
        self.elapsed_ms.fetch_add(
            u64::try_from(duration.as_millis()).expect("durée de test en millisecondes"),
            std::sync::atomic::Ordering::SeqCst,
        );
    }
}

#[cfg(test)]
fn test_clock_slot() -> &'static std::sync::Mutex<Option<std::sync::Arc<TestMonotonicClock>>> {
    static CLOCK: std::sync::OnceLock<
        std::sync::Mutex<Option<std::sync::Arc<TestMonotonicClock>>>,
    > = std::sync::OnceLock::new();
    CLOCK.get_or_init(|| std::sync::Mutex::new(None))
}

fn read_error(source: std::io::Error) -> BridgetClientError {
    if matches!(
        source.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        BridgetClientError::Timeout {
            operation: "lecture socket",
        }
    } else {
        BridgetClientError::Read(source)
    }
}

fn write_error(source: std::io::Error) -> BridgetClientError {
    if matches!(
        source.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    ) {
        BridgetClientError::Timeout {
            operation: "ecriture socket",
        }
    } else {
        BridgetClientError::Write(source)
    }
}

fn parse_service_welcome(response: Value) -> Result<NegotiatedGuichetContract, BridgetClientError> {
    match response_type(&response)? {
        "ServiceWelcome" => {
            let version = u16::try_from(required_u64(&response, "version")?).map_err(|_| {
                BridgetClientError::Protocol("version service Bridget hors plage u16".to_string())
            })?;
            let horizon_secs = required_i64(&response, "horizon_secs")?;
            let issued_at_tolerance_secs = required_i64(&response, "issued_at_tolerance_secs")?;
            if horizon_secs <= 0 || issued_at_tolerance_secs < 0 {
                return Err(BridgetClientError::Protocol(
                    "horizon ou tolerance guichet invalide".to_string(),
                ));
            }
            let capabilities = response
                .get("capabilities")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    BridgetClientError::Protocol("ServiceWelcome sans capabilities".to_string())
                })?
                .iter()
                .map(|value| {
                    value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                        BridgetClientError::Protocol("capabilite guichet non textuelle".to_string())
                    })
                })
                .collect::<Result<BTreeSet<_>, _>>()?;
            Ok(NegotiatedGuichetContract {
                version,
                horizon_secs,
                issued_at_tolerance_secs,
                capabilities,
            })
        }
        "ClientRejected" | "ServiceRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("ServiceWelcome", other)),
    }
}

fn parse_project_registry_outcome(
    response: Value,
    expected_command_id: &str,
) -> Result<bridget_transport::protocol::ProjectBindOutcome, BridgetClientError> {
    match response_type(&response)? {
        "project_registry_outcome" => {
            let outcome = response.get("outcome").cloned().ok_or_else(|| {
                BridgetClientError::Protocol("issue registre projet absente".to_string())
            })?;
            let outcome: bridget_transport::protocol::ProjectBindOutcome =
                serde_json::from_value(outcome).map_err(|source| BridgetClientError::Decode {
                    line: response.to_string(),
                    source,
                })?;
            if outcome.command_id != expected_command_id {
                return Err(BridgetClientError::Protocol(
                    "issue registre projet corrélée à une autre commande".to_string(),
                ));
            }
            Ok(outcome)
        }
        "ServiceRejected" | "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("project_registry_outcome", other)),
    }
}

fn parse_project_registry_admin_outcome(
    response: Value,
    expected_command_id: &str,
) -> Result<bridget_transport::protocol::ProjectAdminOutcome, BridgetClientError> {
    match response_type(&response)? {
        "project_registry_admin_outcome" => {
            let outcome = response.get("outcome").cloned().ok_or_else(|| {
                BridgetClientError::Protocol("issue administrative projet absente".to_string())
            })?;
            let outcome: bridget_transport::protocol::ProjectAdminOutcome =
                serde_json::from_value(outcome).map_err(|source| BridgetClientError::Decode {
                    line: response.to_string(),
                    source,
                })?;
            if outcome.command_id != expected_command_id {
                return Err(BridgetClientError::Protocol(
                    "issue administrative corrélée à une autre commande".to_string(),
                ));
            }
            Ok(outcome)
        }
        "ServiceRejected" | "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("project_registry_admin_outcome", other)),
    }
}

fn parse_guichet_claim(response: Value) -> Result<Option<GuichetClaim>, BridgetClientError> {
    match response_type(&response)? {
        "guichet_empty" => Ok(None),
        "guichet_claimed" => Ok(Some(GuichetClaim {
            issuer_scope: required_string(&response, "issuer_scope")?,
            request_id: required_string(&response, "request_id")?,
            canonical_request: decode_base64_bytes(&response, "canonical_request")?,
            authorization_attestation: response
                .get("authorization_attestation")
                .filter(|value| !value.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|_| {
                    BridgetClientError::Protocol(
                        "attestation d'autorisation guichet invalide".to_string(),
                    )
                })?,
            claimed_at: required_i64(&response, "claimed_at")?,
            claim_generation: required_u64(&response, "claim_generation")?,
            claim_token: required_string(&response, "claim_token")?,
            claim_lease_expires_at: required_i64(&response, "claim_lease_expires_at")?,
            expires_at: required_i64(&response, "expires_at")?,
        })),
        "Nack" => Err(parse_nack(response)?),
        "ClientRejected" | "ServiceRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("guichet_claimed", other)),
    }
}

fn parse_guichet_result(response: Value) -> Result<GuichetResult, BridgetClientError> {
    match response_type(&response)? {
        "guichet_result" => Ok(GuichetResult {
            issuer_scope: required_string(&response, "issuer_scope")?,
            request_id: required_string(&response, "request_id")?,
            issue: required_string(&response, "issue")?,
            expires_at: required_i64(&response, "expires_at")?,
            payload: response
                .get("payload")
                .filter(|value| !value.is_null())
                .cloned()
                .map(serde_json::from_value)
                .transpose()
                .map_err(|_| {
                    BridgetClientError::Protocol("payload terminal guichet invalide".to_string())
                })?,
        }),
        "Nack" => Err(parse_nack(response)?),
        "ClientRejected" | "ServiceRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("guichet_result", other)),
    }
}

fn parse_guichet_lifecycle_event(
    response: Value,
) -> Result<GuichetLifecycleEvent, BridgetClientError> {
    if response_type(&response)? != "request_lifecycle_event" {
        return Err(unexpected(
            "request_lifecycle_event",
            response_type(&response)?,
        ));
    }
    let state = required_string(&response, "state")?;
    if !matches!(state.as_str(), "answered" | "cancelled" | "timed_out") {
        return Err(BridgetClientError::Protocol(
            "etat terminal guichet inconnu".to_string(),
        ));
    }
    Ok(GuichetLifecycleEvent {
        issuer_scope: required_string(&response, "issuer_scope")?,
        event_id: required_string(&response, "event_id")?,
        request_id: required_string(&response, "request_id")?,
        state,
        observed_at: required_i64(&response, "observed_at")?,
        in_reply_to: optional_string(&response, "in_reply_to")?,
        response_message_id: optional_string(&response, "response_message_id")?,
    })
}

fn parse_coordination_stream_item(
    response: Value,
    canonical_bytes: Vec<u8>,
) -> Result<CoordinationStreamItem, BridgetClientError> {
    match response_type(&response)? {
        "coordination_event" => {
            if required_u64(&response, "v")? != u64::from(COORDINATION_STREAM_VERSION)
                || response.get("cursor").and_then(Value::as_u64).is_none()
            {
                return Err(BridgetClientError::Protocol(
                    "événement de coordination non cursé".to_string(),
                ));
            }
            Ok(CoordinationStreamItem::Event { canonical_bytes })
        }
        "request_lifecycle_event" => Ok(CoordinationStreamItem::Lifecycle(
            parse_guichet_lifecycle_event(response)?,
        )),
        "coordination_snapshot_caught_up" => {
            if required_u64(&response, "v")? != u64::from(COORDINATION_STREAM_VERSION) {
                return Err(BridgetClientError::Protocol(
                    "snapshot de coordination de version inattendue".to_string(),
                ));
            }
            Ok(CoordinationStreamItem::SnapshotCaughtUp {
                through_cursor: optional_u64(&response, "through_cursor")?,
            })
        }
        "coordination_gap" => Ok(CoordinationStreamItem::Gap {
            from_cursor: required_u64(&response, "from_cursor")?,
            to_cursor: required_u64(&response, "to_cursor")?,
            reason: required_string(&response, "reason")?,
        }),
        "coordination_unavailable" => Ok(CoordinationStreamItem::Unavailable {
            reason: required_string(&response, "reason")?,
        }),
        "Nack" => Err(parse_nack(response)?),
        "ClientRejected" | "ServiceRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("événement de coordination", other)),
    }
}

fn validate_guichet_reply_bytes(
    bytes: &[u8],
    max_frame_bytes: usize,
) -> Result<(), BridgetClientError> {
    if bytes.len() + 1 > max_frame_bytes {
        return Err(BridgetClientError::FrameTooLarge { max_frame_bytes });
    }
    let value: Value =
        serde_json::from_slice(bytes).map_err(|source| BridgetClientError::Decode {
            line: String::from_utf8_lossy(bytes).into_owned(),
            source,
        })?;
    if value.get("type").and_then(Value::as_str) != Some("guichet_reply")
        || value.get("v").and_then(Value::as_u64) != Some(1)
    {
        return Err(BridgetClientError::InvalidEnvelope(
            "reponse guichet persistée invalide".to_string(),
        ));
    }
    for field in [
        "issuer_scope",
        "request_id",
        "claim_token",
        "response_message_id",
        "outcome",
    ] {
        if value
            .get(field)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(BridgetClientError::InvalidEnvelope(format!(
                "reponse guichet sans {field}"
            )));
        }
    }
    if value
        .get("claim_generation")
        .and_then(Value::as_u64)
        .is_none()
    {
        return Err(BridgetClientError::InvalidEnvelope(
            "reponse guichet sans claim_generation".to_string(),
        ));
    }
    Ok(())
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

fn parse_daemon_identity(response: Value) -> Result<DaemonIdentity, BridgetClientError> {
    match response_type(&response)? {
        "DaemonIdentityReport" => Ok(DaemonIdentity {
            host: required_string(&response, "host")?,
            db_path: required_string(&response, "db_path")?,
        }),
        "ClientRejected" => Err(BridgetClientError::ClientRejected {
            reason: response.get("reason").cloned().unwrap_or(Value::Null),
        }),
        other => Err(unexpected("DaemonIdentityReport", other)),
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

fn required_bool(response: &Value, field: &str) -> Result<bool, BridgetClientError> {
    response
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} absent ou invalide")))
}

fn decode_base64_bytes(response: &Value, field: &str) -> Result<Vec<u8>, BridgetClientError> {
    use base64::Engine;

    let encoded = response
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| BridgetClientError::Protocol(format!("champ {field} absent ou invalide")))?;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| {
            BridgetClientError::Protocol(format!("champ {field} base64 invalide: {error}"))
        })
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

#[cfg(test)]
mod tests {
    use super::{
        BridgetClient, BridgetClientError, BridgetClientLimits, GuichetClient,
        ProjectRegistryClient, PublicMessage, ReplayPublicMessage, TestMonotonicClock,
        monotonic_now, replay_idempotent_request, test_clock_slot, validate_send_idempotent_frame,
    };
    use serde_json::json;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::sync::{Arc, MutexGuard, OnceLock};
    use std::thread;
    use std::time::{Duration, Instant};

    /// Ce que le jumeau `IdempotencyIssue` fait d'une variante qu'il ne connaît
    /// pas — mesuré, pas supposé, parce que la réponse décide du périmètre d'un
    /// changement de statut.
    ///
    /// Le jumeau est un enum tagué SANS variante de repli. Une variante inconnue
    /// sur le fil ne se perd donc pas en silence : elle fait ÉCHOUER le décodage.
    /// C'est plus sûr qu'un oubli muet, mais ça signifie qu'ajouter une variante
    /// `in_flight` au protocole filaire casserait ce consommateur — et ce, sans
    /// rien lui apporter, puisque `delivery_id` lui donne déjà la distinction.
    ///
    /// D'où la frontière tenue par le lot : le statut distinct vit sur la
    /// SURFACE CLIENT (le champ `status` du retour MCP), le fil reste inchangé.
    /// Si ce test se met à passer avec un `Ok`, c'est qu'une variante de repli a
    /// été ajoutée au jumeau, et la frontière peut être rediscutée.
    #[test]
    fn le_jumeau_refuse_une_variante_inconnue_au_lieu_de_la_perdre() {
        let inconnue = json!({ "kind": "in_flight", "expires_at": 1_700_000_060, "delivery_id": "livraison-1" });
        let decode: Result<super::IdempotencyIssue, _> = serde_json::from_value(inconnue);
        let erreur = decode.expect_err(
            "une variante inconnue doit être refusée : si elle passe, le consommateur \
             lit un état qu'il n'a pas compris",
        );
        assert!(
            erreur.to_string().contains("unknown variant"),
            "le refus doit nommer la variante inconnue, sinon le diagnostic est illisible: {erreur}"
        );

        // Contre-épreuve : la variante CONNUE, elle, se décode — sans quoi le
        // test ci-dessus passerait pour n'importe quelle raison (typo de champ,
        // tag absent) et ne prouverait rien sur les variantes.
        let connue = json!({ "kind": "outcome_unknown", "expires_at": 1_700_000_060, "delivery_id": "livraison-1" });
        let decode: super::IdempotencyIssue = serde_json::from_value(connue).unwrap();
        assert_eq!(
            decode,
            super::IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: Some("livraison-1".to_string()),
            }
        );

        // Et le `serde(default)` du champ, lui, EST silencieux : un fil sans
        // `delivery_id` rend `None` sans erreur. C'est la mécanique à ne pas
        // confondre avec la précédente — le champ absent se perd en silence, la
        // variante inconnue non.
        let sans_champ = json!({ "kind": "outcome_unknown", "expires_at": 1_700_000_060 });
        let decode: super::IdempotencyIssue = serde_json::from_value(sans_champ).unwrap();
        assert_eq!(
            decode,
            super::IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: None,
            }
        );
    }

    /// Oracle du contrat d'attribution : la requête doit passer APRÈS le
    /// handshake Client, sur la même connexion, sans recommencer une seconde
    /// négociation. Le contrôle positif prouve que la garde future n'est pas
    /// un refus systématique.
    #[test]
    fn daemon_identity_emprunte_la_connexion_client_deja_negociee() {
        let socket = identity_socket("rapport");
        let listener = UnixListener::bind(&socket).expect("socket identité");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion client");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            accept_client_contract(&mut reader, &mut writer);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"DaemonIdentityRequest"}),
                "la sonde doit être le second aller-retour, pas une seconde poignée client"
            );
            write_json(
                &mut writer,
                json!({
                    "type":"DaemonIdentityReport",
                    "host":"machine-registre",
                    "db_path":"/var/lib/bridget/bridget.db"
                }),
            );
        });

        let mut client = BridgetClient::connect_with_limits(
            &socket,
            "maicie-locality-guard",
            BridgetClientLimits::default(),
        )
        .expect("contrat client");
        let identity = client
            .daemon_identity()
            .expect("rapport d'identité conforme");
        assert_eq!(identity.host, "machine-registre");
        assert_eq!(identity.db_path, "/var/lib/bridget/bridget.db");

        server.join().expect("serveur identité");
        let _ = std::fs::remove_file(socket);
    }

    /// Un refus de matrice ne doit jamais être aplati en absence de réponse :
    /// la garde refusera dans les deux cas, mais l'opérateur doit savoir si la
    /// sonde est interdite plutôt que croire le daemon simplement indisponible.
    #[test]
    fn daemon_identity_conserve_le_refus_de_role_comme_diagnostic_distinct() {
        let socket = identity_socket("refus-role");
        let listener = UnixListener::bind(&socket).expect("socket identité");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion client");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            accept_client_contract(&mut reader, &mut writer);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"DaemonIdentityRequest"})
            );
            write_json(
                &mut writer,
                json!({"type":"ClientRejected","reason":"MessageOutsideClientRole"}),
            );
        });

        let mut client =
            BridgetClient::connect(&socket, "maicie-locality-guard").expect("contrat client");
        let error = client
            .daemon_identity()
            .expect_err("un ClientRejected ne doit pas devenir une identité absente");
        assert!(matches!(error, BridgetClientError::ClientRejected { .. }));

        server.join().expect("serveur identité");
        let _ = std::fs::remove_file(socket);
    }

    /// L'absence de trame est un fait différent d'un rejet : le client doit
    /// remonter la fermeture pour que la garde refuse sans présenter la sonde
    /// comme saine. L'attente est bornée par les limites du client.
    #[test]
    fn daemon_identity_conserve_l_absence_de_rapport_comme_fermeture() {
        let socket = identity_socket("absence");
        let listener = UnixListener::bind(&socket).expect("socket identité");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion client");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            accept_client_contract(&mut reader, &mut writer);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"DaemonIdentityRequest"})
            );
            // La connexion se ferme sans rapport : ce n'est pas un succès par défaut.
        });

        let mut client =
            BridgetClient::connect(&socket, "maicie-locality-guard").expect("contrat client");
        let error = client
            .daemon_identity()
            .expect_err("une fermeture sans rapport doit rester observable");
        assert!(matches!(error, BridgetClientError::Closed));

        server.join().expect("serveur identité");
        let _ = std::fs::remove_file(socket);
    }

    /// Une réponse d'un autre type est une rupture de contrat, pas une
    /// attestation permissive. L'oracle empêche d'ajouter un `unwrap_or` qui
    /// cacherait une sonde cassée derrière un comportement local apparemment sain.
    #[test]
    fn daemon_identity_refuse_une_reponse_de_type_inattendu() {
        let socket = identity_socket("type-inattendu");
        let listener = UnixListener::bind(&socket).expect("socket identité");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion client");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            accept_client_contract(&mut reader, &mut writer);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"DaemonIdentityRequest"})
            );
            write_json(&mut writer, json!({"type":"AgentList","agents":[]}));
        });

        let mut client =
            BridgetClient::connect(&socket, "maicie-locality-guard").expect("contrat client");
        let error = client
            .daemon_identity()
            .expect_err("un type inattendu doit être refusé");
        assert!(matches!(error, BridgetClientError::Protocol(_)));

        server.join().expect("serveur identité");
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn reprise_conserve_les_extensions_inconnues_octet_pour_octet() {
        let message = br#"{"id":"message-1","from":"maicie","to":"prospective","body":"preuve","future_extension":{"level":2}}"#;
        let replay: ReplayPublicMessage = serde_json::from_slice(message).unwrap();
        assert_eq!(replay.into_public_message().id, "message-1");

        let request = replay_idempotent_request(message, "message-1", 1_700_000_000).unwrap();
        assert!(
            request
                .windows(message.len())
                .any(|candidate| candidate == message),
            "la sous-enveloppe persistée doit être injectée sans réécriture"
        );
    }

    #[test]
    fn registre_projet_negocie_la_capacite_et_rejoue_les_octets_persistes() {
        let socket = identity_socket("registre-projet");
        let listener = UnixListener::bind(&socket).expect("socket registre projet");
        let request = br#"{"contract_version":1,"command_id":"project-command-1","issued_at":1788000000,"deadline_at":1788000060,"project_id":"project-opaque-1","requested_root":"/srv/projects/fixture","backend":"host"}"#.to_vec();
        let expected_frame = format!(
            "{{\"type\":\"project_registry_request\",\"request\":{}}}",
            String::from_utf8_lossy(&request)
        );
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion registre projet");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"RoleHandshake","role":"service"})
            );
            write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
            assert_eq!(
                read_json(&mut reader),
                json!({
                    "type":"ServiceHello",
                    "version":1,
                    "service":"maicie",
                    "issuer_scope":"maicie-registry-0123456789abcdef",
                    "capabilities":["project_registry_v1"]
                })
            );
            write_json(
                &mut writer,
                json!({
                    "type":"ServiceWelcome",
                    "version":1,
                    "horizon_secs":60,
                    "issued_at_tolerance_secs":5,
                    "capabilities":["project_registry_v1"]
                }),
            );
            let mut raw = String::new();
            reader.read_line(&mut raw).expect("requête registre projet");
            assert_eq!(raw.trim_end(), expected_frame);
            write_json(
                &mut writer,
                json!({
                    "type":"project_registry_outcome",
                    "outcome":{
                        "contract_version":1,
                        "command_id":"project-command-1",
                        "project_id":"project-opaque-1",
                        "status":"active",
                        "binding_generation":1,
                        "backend":"host",
                        "reason":null,
                        "existing_project_id":null,
                        "existing_binding_generation":null,
                        "observed_at":1788000001
                    }
                }),
            );
        });

        let mut client =
            ProjectRegistryClient::connect(&socket, "maicie-registry-0123456789abcdef")
                .expect("négociation registre projet");
        let outcome = client
            .bind_exact_bytes(&request)
            .expect("issue registre projet");
        assert_eq!(outcome.command_id, "project-command-1");
        assert_eq!(outcome.binding_generation, Some(1));
        server.join().expect("serveur registre projet");
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn borne_de_trame_compte_le_delimiteur() {
        let message = PublicMessage {
            id: "message-1".to_string(),
            from: "maicie".to_string(),
            to: "prospective".to_string(),
            body: "preuve".to_string(),
            reply: false,
            hops: 4,
            reply_timeout: None,
            deadline_at: None,
            in_reply_to: None,
        };
        let request = json!({
            "type": "SendIdempotent",
            "message": &message,
            "message_id": "message-1",
            "issued_at": 1_700_000_000_i64,
        });
        let without_delimiter = serde_json::to_vec(&request).unwrap().len();

        assert!(matches!(
            validate_send_idempotent_frame(&message, "message-1", 1_700_000_000, without_delimiter,),
            Err(BridgetClientError::FrameTooLarge { .. })
        ));
        validate_send_idempotent_frame(&message, "message-1", 1_700_000_000, without_delimiter + 1)
            .unwrap();
    }

    #[test]
    fn echeance_guichet_absolue_interdit_la_phase_suivante_sans_sommeil_reel() {
        let (clock, _reset) = install_test_clock();
        let socket = std::path::PathBuf::from(format!(
            "/tmp/mgc-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("socket guichet");
        let server_clock = Arc::clone(&clock);
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion service");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            assert_eq!(
                read_json(&mut reader),
                json!({"type":"RoleHandshake","role":"service"})
            );

            // La barrière avance l'horloge contrôlée avant que le client ne
            // reçoive RoleAccepted. Une échéance renouvelée par phase enverrait
            // alors ServiceHello ; l'échéance absolue correcte l'interdit.
            server_clock.advance(Duration::from_millis(100));
            write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
            reader
                .get_mut()
                .set_read_timeout(Some(Duration::from_secs(1)))
                .expect("timeout harnais");
            let mut forbidden = String::new();
            assert!(
                matches!(reader.read_line(&mut forbidden), Ok(0)),
                "mutation échéance renouvelée : ServiceHello ne doit jamais être atteint ({forbidden:?})"
            );
        });

        let limits = BridgetClientLimits {
            connect_timeout: Duration::from_millis(100),
            io_timeout: Duration::from_millis(100),
            max_frame_bytes: 64 * 1024,
        };
        let deadline = monotonic_now() + Duration::from_millis(100);
        let error = match GuichetClient::connect_with_limits_until(
            &socket,
            "scope-guichet-0123456789abcdef",
            limits,
            deadline,
        ) {
            Ok(_) => panic!("la seconde phase doit consommer l'échéance déjà épuisée"),
            Err(error) => error,
        };
        assert!(matches!(error, BridgetClientError::Timeout { .. }));
        server.join().expect("serveur de barrière");
        let _ = std::fs::remove_file(socket);
    }

    fn install_test_clock() -> (Arc<TestMonotonicClock>, TestClockReset) {
        static SERIAL: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
        let serial = SERIAL
            .get_or_init(|| std::sync::Mutex::new(()))
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let clock = Arc::new(TestMonotonicClock {
            base: Instant::now(),
            elapsed_ms: std::sync::atomic::AtomicU64::new(0),
        });
        *test_clock_slot()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(Arc::clone(&clock));
        (clock, TestClockReset { _serial: serial })
    }

    struct TestClockReset {
        _serial: MutexGuard<'static, ()>,
    }

    impl Drop for TestClockReset {
        fn drop(&mut self) {
            *test_clock_slot()
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()) = None;
        }
    }

    fn identity_socket(label: &str) -> std::path::PathBuf {
        let socket = std::env::temp_dir().join(format!(
            "maicie-daemon-identity-{label}-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let _ = std::fs::remove_file(&socket);
        socket
    }

    fn accept_client_contract(
        reader: &mut BufReader<UnixStream>,
        writer: &mut BufWriter<UnixStream>,
    ) {
        assert_eq!(
            read_json(reader),
            json!({"type":"RoleHandshake","role":"client"})
        );
        write_json(writer, json!({"type":"RoleAccepted","role":"client"}));
        let hello = read_json(reader);
        assert_eq!(hello["type"], "ClientHello");
        assert_eq!(hello["contract_version"], 1);
        assert!(
            hello["issuer_scope"]
                .as_str()
                .is_some_and(|scope| !scope.is_empty()),
            "la sonde doit porter un issuer_scope client non vide"
        );
        assert_eq!(hello["capabilities"], json!(["send_idempotent", "lookup"]));
        write_json(
            writer,
            json!({
                "type":"ClientWelcome",
                "version":1,
                "horizon_secs":60,
                "issued_at_tolerance_secs":5,
                "capabilities":["send_idempotent","lookup"]
            }),
        );
    }

    fn read_json(reader: &mut BufReader<UnixStream>) -> serde_json::Value {
        let mut line = String::new();
        reader.read_line(&mut line).expect("lecture JSONL");
        serde_json::from_str(&line).expect("JSONL valide")
    }

    fn write_json(writer: &mut BufWriter<UnixStream>, value: serde_json::Value) {
        serde_json::to_writer(&mut *writer, &value).expect("écriture JSONL");
        writer.write_all(b"\n").expect("délimiteur JSONL");
        writer.flush().expect("flush JSONL");
    }
}
