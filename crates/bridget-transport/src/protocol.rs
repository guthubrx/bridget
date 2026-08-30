//! Définitions des messages JSON du protocole daemon/wrapper.
//!
//! Deux directions :
//! - WrapperToDaemon : ce que le wrapper envoie au daemon
//! - DaemonToWrapper : ce que le daemon envoie au wrapper

use bridget_core::BridgetMessage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Rôle négocié au début d'une connexion persistante avec le daemon.
///
/// L'absence de négociation reste implicitement un wrapper pour préserver les
/// agents 007 déjà déployés. Un client attach doit en revanche s'annoncer
/// explicitement avant toute souscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionRole {
    Wrapper,
    Attach,
    Client,
    Service,
}

/// Mode réel de présence d'un agent.
///
/// Cette information décrit le chemin d'attelage (et non le transport réseau)
/// qui a effectivement enregistré l'agent. Une absence conserve la
/// compatibilité des enregistrements antérieurs au champ et ne doit jamais
/// être interprétée par déduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceMode {
    Acp,
    Tmux,
    Cli,
}

impl PresenceMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Acp => "acp",
            Self::Tmux => "tmux",
            Self::Cli => "cli",
        }
    }
}

/// Déclaration filaire du canal de connexion.
///
/// Ces trois états ne sont pas interchangeables : un ancien producteur peut
/// omettre le champ sans invalider le dernier fait connu, tandis qu'un
/// producteur récent doit pouvoir déclarer explicitement que le canal est
/// inconnu. Sur le fil, ils deviennent respectivement une clé absente,
/// `channel: null` et `channel: "…"`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ChannelReport {
    #[default]
    Omitted,
    Unknown,
    Known(String),
}

impl ChannelReport {
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    pub fn reported(value: Option<String>) -> Self {
        match value {
            Some(value) => Self::Known(value),
            None => Self::Unknown,
        }
    }

    pub const fn is_omitted(&self) -> bool {
        matches!(self, Self::Omitted)
    }

    pub fn as_deref(&self) -> Option<&str> {
        match self {
            Self::Known(value) => Some(value),
            Self::Omitted | Self::Unknown => None,
        }
    }
}

impl From<Option<String>> for ChannelReport {
    fn from(value: Option<String>) -> Self {
        Self::reported(value)
    }
}

fn serialize_channel_report<S>(report: &ChannelReport, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match report {
        ChannelReport::Known(value) => serializer.serialize_str(value),
        ChannelReport::Unknown | ChannelReport::Omitted => serializer.serialize_none(),
    }
}

fn deserialize_channel_report<'de, D>(deserializer: D) -> Result<ChannelReport, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(ChannelReport::reported)
}

/// Version actuellement publiée du contrat idempotent local.
pub const CLIENT_CONTRACT_VERSION: u16 = 1;
/// Version du contrat de service du guichet Maicie.
pub const SERVICE_CONTRACT_VERSION: u16 = 1;
/// Version du contrat local de registre de projets.
pub const PROJECT_REGISTRY_CONTRACT_VERSION: u16 = 1;
/// Version requise uniquement lorsqu'une délégation transporte une cible de
/// revue. Les autres opérations restent en v1 afin que `ServiceHello` et les
/// clients historiques ne négocient pas une capacité qu'ils n'utilisent pas.
pub const REVIEW_DELEGATE_CONTRACT_VERSION: u16 = 2;
/// Version de l'extension de faits attestés pour la coordination active.
///
/// Elle reste une capacité négociée du contrat de service v1 : les clients 015
/// qui ne la demandent pas ne reçoivent aucune trame 016.
pub const COORDINATION_EVENTS_VERSION: u16 = 1;
/// Version de la relève cursée de coordination. Elle complète, sans modifier,
/// l'émission historique v1.
pub const COORDINATION_STREAM_VERSION: u16 = 2;

/// Capacité optionnelle du client idempotent. L'énumération fermée évite une
/// dégradation silencieuse lorsqu'un client demande une capacité inconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    SendIdempotent,
    Lookup,
    ExecutionControlV1,
}

/// Commande neutre et versionnée du plan de contrôle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionControlOperation {
    QueueOnly,
    TriggerTurn,
    SteerCurrent,
    Interrupt,
    PauseQueue,
    ResumeQueue,
    CancelQueued,
}

/// Commande de contrôle rejouable, toujours corrélée à une exécution Bridget.
///
/// Le message est présent uniquement pour une opération qui injecte du texte
/// dans un tour fournisseur. Les opérations sans prompt le laissent absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionControlCommand {
    pub version: u16,
    pub command_id: String,
    pub execution_id: String,
    pub generation: u64,
    pub revision: u64,
    pub operation: ExecutionControlOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<BridgetMessage>,
}

/// Refus fermé : une commande absente de la négociation ne devient jamais un
/// best effort fondé sur le nom du fournisseur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionControlRefusal {
    CapabilityNotNegotiated,
    CapabilityUnavailable,
    GenerationMismatch,
    RevisionMismatch,
    ExecutionNotFound,
    TerminalExecution,
    InvalidCommand,
    TargetUnavailable,
    MessageRequired,
}

/// Issue publique immédiate d'une commande de contrôle.
///
/// `OutcomeUnknown` indique la remise au wrapper sans résultat connu. `Accepted`
/// ou `Refused` ne sont publiés qu'après son accusé durable. L'issue fournisseur
/// reste une transition d'exécution corrélée, pas une promesse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionControlOutcome {
    Accepted,
    Refused(ExecutionControlRefusal),
    OutcomeUnknown,
}

/// Issue technique d une politique d autonomie. Ces états ne portent aucune
/// décision de mission : ils expliquent uniquement pourquoi Bridget ne crée
/// pas de tour supplémentaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionBudgetOutcome {
    Paused,
    Blocked,
    UsageLimit,
    BudgetLimit,
    Terminated,
}

/// Transition runtime corrélée à une exécution Bridget.
///
/// Les identifiants fournisseur restent hors de cette trame : le wrapper ne
/// rapporte que le fait déjà normalisé par son adaptateur et le daemon vérifie
/// état, révision et génération avant toute écriture durable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionStateTransition {
    pub execution_id: String,
    pub generation: u64,
    pub expected_state: String,
    pub expected_revision: u64,
    pub next_state: String,
    pub reason: String,
    pub observed_at: i64,
}

/// Référence fournisseur attestée, attachée à une exécution Bridget précise.
///
/// Elle est distincte d'une transition d'état : le daemon vérifie le propriétaire
/// et la génération avant de la persister et ne l'emploie jamais pour déduire
/// une décision métier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionProviderContext {
    pub execution_id: String,
    pub generation: u64,
    pub provider_kind: String,
    pub execution_path: String,
    pub observation: ProviderObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_thread_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_turn_id: Option<String>,
    pub observed_at: i64,
}
/// Capacité explicitement négociée par un service local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCapability {
    MaicieGuichet,
    /// Registre local Bridget, exclusivement négocié par le service Maicie.
    ProjectRegistryV1,
    CoordinationEventsV1,
    /// Relève bornée et cursée des faits 016. La v1 reste disponible pour les
    /// consommateurs qui n'ont besoin que du rejeu initial historique.
    CoordinationEventsV2,
}

/// Backend d'exécution admis par le registre de projets.
///
/// La v1 n'accepte qu'une racine validée directement sur l'hôte du daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectBackend {
    Host,
}

/// Requête de liaison de projet portée par la variante dédiée du protocole.
///
/// Le contrôle de l'UID pair reste hors du JSON : il est effectué par le
/// daemon sur la socket Unix avant de décoder cette charge métier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindRequest {
    pub contract_version: u16,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub project_id: String,
    pub requested_root: String,
    pub backend: ProjectBackend,
}

/// État terminal d'une tentative de liaison de projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectBindStatus {
    Active,
    BindingFailed,
    RegistrationConflict,
}

/// Raisons fermées exposées par le registre de projets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRegistryRefusal {
    InvalidContract,
    InvalidProjectId,
    InvalidAbsoluteRoot,
    RootMissing,
    RootNotDirectory,
    RootOutsideAllowedPrefixes,
    RootTooBroad,
    RootAlreadyBound,
    ProjectAlreadyBoundElsewhere,
    RebindRequired,
    ProjectDisabled,
    EnvelopeMismatch,
    IdempotencyExpired,
    StoreUnavailable,
    RegistrationConflict,
    ProjectRegistryCapabilityMissing,
    ProjectRegistryVersionUnsupported,
    LocalOperatorRequired,
    PeerUidMismatch,
    ProjectRootPolicyUnavailable,
    ProjectRootPolicyInvalid,
    ProjectRootPolicyPermissionsInvalid,
}

/// Issue terminale d'une demande de liaison, sans chemin canonique exposé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub project_id: String,
    pub status: ProjectBindStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<ProjectBackend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_binding_generation: Option<u64>,
    pub observed_at: i64,
}

/// Opération administrative locale du registre. Les mutations restent
/// strictement sur la connexion Service Maicie négociée; `List` et `Status`
/// sont des lectures explicites et ne créent aucune liaison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAdminOperation {
    List,
    Status,
    Rebind,
    Disable,
    ReviewProjectReconcile,
}

/// Requête versionnée dédiée aux lectures et mutations administratives. Une
/// action porte un command_id stable, y compris quand elle n'écrit rien, pour
/// garder diagnostics et retentatives corrélables sans détourner ServiceRequest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAdminRequest {
    pub contract_version: u16,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub operation: ProjectAdminOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_root: Option<String>,
}

/// Projection technique publique d'une liaison, sans racine hôte ni contenu
/// de dépôt. La référence de racine auditée ne quitte jamais le store Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectBindingStatus {
    Active,
    Disabled,
    PathMissing,
    PendingBinding,
    BindingFailed,
    Unregistered,
}

/// Référence opaque et durable d'un projet admis. Elle ne contient jamais de
/// racine hôte, de domaine ou de donnée fournisseur: Bridget peut la propager
/// sans devenir autorité métier sur le projet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectReference {
    pub project_id: String,
    pub binding_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindingProjection {
    pub project_id: String,
    pub state: ProjectBindingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<ProjectBackend>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    pub observed_at: i64,
}

/// Issue corrélée de l'administration du registre. Une mutation effective
/// renvoie une seule projection et un rejet ne fabrique jamais d'audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAdminOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub operation: ProjectAdminOperation,
    pub bindings: Vec<ProjectBindingProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    pub observed_at: i64,
}

/// Refus structurés de la frontière réservée aux services.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServiceRefusal {
    RoleHandshakeRequired,
    ServiceRoleRequired,
    NegotiationRequired,
    AlreadyNegotiated,
    UnsupportedVersion { supported_versions: Vec<u16> },
    InvalidIssuerScope,
    ReservedServiceRequired,
    InvalidEnvelope,
    CapabilityRequired,
    MessageOutsideServiceRole,
    TransitionInvalid,
    CanonicalBytesMismatch,
    IdempotencyExpired,
    ClaimStale,
    DeclaredSenderMismatch,
    ReservedTargetRequired,
    InvalidIssuedAt,
    FrameTooLarge,
    GreffeAuthorizationDenied,
}

/// Opérations fermées que Bridget peut déposer dans le guichet Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceRequestOperation {
    DeliveryReport,
    MissionStatus,
    DeadlineQuestion,
    Delegate,
    RegistreAdd,
    ObjectiveClose,
}

/// Verdict fermé d'une revue. Le transport conserve le fait déclaré ; seule
/// la greffe Maicie décide s'il correspond au mandat durable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewVerdict {
    Approve,
    ApproveWithChanges,
    Amender,
    Stop,
}

impl ReviewVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::ApproveWithChanges => "approve_with_changes",
            Self::Amender => "amender",
            Self::Stop => "stop",
        }
    }
}

/// Cible Git gelée dans un mandat de revue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewTarget {
    pub target_ref: String,
    pub expected_head: String,
}

impl ReviewTarget {
    /// Sépare `<remote>/<branche>` après validation de la forme fermée.
    pub fn remote_and_branch(&self) -> Option<(&str, &str)> {
        let (remote, branch) = self.target_ref.split_once('/')?;
        valid_git_remote(remote)
            .then_some(())
            .and_then(|_| valid_git_branch(branch).then_some((remote, branch)))
    }

    pub fn is_valid(&self) -> bool {
        self.remote_and_branch().is_some() && is_canonical_git_sha(&self.expected_head)
    }
}

/// Observations Git produites par le binaire de dépôt, jamais fournies comme
/// valeurs libres pour `measured_head` ou `observed_target_head`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewVerdictEvidence {
    pub verdict: ReviewVerdict,
    pub target_ref: String,
    pub expected_head: String,
    pub measured_head: String,
    pub observed_target_head: String,
}

impl ReviewVerdictEvidence {
    pub fn is_valid(&self) -> bool {
        ReviewTarget {
            target_ref: self.target_ref.clone(),
            expected_head: self.expected_head.clone(),
        }
        .is_valid()
            && is_canonical_git_sha(&self.measured_head)
            && is_canonical_git_sha(&self.observed_target_head)
    }
}

pub fn is_canonical_git_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_git_remote(remote: &str) -> bool {
    !remote.is_empty()
        && remote.len() <= 128
        && remote
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && remote
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn valid_git_branch(branch: &str) -> bool {
    !branch.is_empty()
        && branch.len() <= 383
        && !branch.starts_with(['-', '/', '.'])
        && !branch.ends_with(['/', '.'])
        && branch != "@"
        && !branch.contains("..")
        && !branch.contains("@{")
        && !branch.contains("//")
        && !branch
            .bytes()
            .any(|byte| byte <= b' ' || byte == 0x7f || b"~^:?*[\\".contains(&byte))
        && branch
            .split('/')
            .all(|part| !part.is_empty() && !part.starts_with('.') && !part.ends_with(".lock"))
}

/// Charge canonique d'un dépôt de guichet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ServiceRequestPayload {
    DeliveryReport {
        objective_id: String,
        delegation_id: String,
        delivery_hash: String,
        in_reply_to: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        review_verdict: Option<ReviewVerdictEvidence>,
    },
    Delegation {
        delegation_id: String,
    },
    Delegate {
        goal: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        review_target: Option<ReviewTarget>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        explicit_target: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        required_tags: Vec<String>,
        duration: GuichetDurationClass,
        suite: ServiceSuiteDeclaration,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        depends_on: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        references: Vec<String>,
    },
    RegistreAdd {
        line: String,
    },
    ObjectiveClose {
        objective_id: String,
        reason: String,
    },
}

impl ServiceRequestPayload {
    /// Version minimale que le producteur doit annoncer pour cette charge.
    ///
    /// Le daemon valide séparément la paire version/charge : cette méthode
    /// évite seulement que les producteurs CLI et MCP divergent.
    pub fn required_contract_version(&self) -> u16 {
        match self {
            Self::Delegate {
                review_target: Some(_),
                ..
            } => REVIEW_DELEGATE_CONTRACT_VERSION,
            _ => SERVICE_CONTRACT_VERSION,
        }
    }
}

/// Déclaration de suite structurée : aucune valeur libre ne peut jouer le rôle
/// de preuve. Un objectif nommé reste un identifiant vérifiable côté maître.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ServiceSuiteDeclaration {
    Aucune,
    Objectif { objective_id: String },
}

/// Issue fermée qu'un service Maicie atteste au guichet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetOutcome {
    Accepted,
    RequestAlreadyTerminal,
    RecipientUnavailable,
    Refused,
}

/// Motif fermé d'un refus déterministe rendu par Maicie après la relève.
///
/// Il décrit une demande bien formée mais impossible à appliquer au registre
/// local. Une corruption du store ou une erreur de transport ne passe jamais
/// par cette voie : ces situations restent des erreurs techniques.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetRefusalReason {
    DelegationMissing,
    RelationInvalid,
    EnvelopeMismatch,
    ReviewVerdictRequired,
    ReviewVerdictUnexpected,
    ReviewMandateMismatch,
    TargetHeadMoved,
    TargetHeadMovedAndMeasuredHeadMismatch,
    MeasuredHeadMismatch,
    SuiteNoneWithUnclassifiedCitation,
    OperationNotAvailable,
    MutationInvalid,
    TargetUnavailable,
    ObjectiveMissing,
    ObjectiveAlreadyClosed,
    AuthorizationDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetDelegateMutationStatus {
    Created,
    SelectionRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetRegistreAddStatus {
    Appended,
    IdempotentNoop,
}

/// Fait terminal attesté uniquement par Bridget pour une demande du guichet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetLifecycleState {
    Answered,
    Cancelled,
    TimedOut,
}

/// Fait de coordination transport attesté exclusivement par Bridget.
///
/// Cette énumération est volontairement fermée : Maicie ne déduit jamais une
/// relance d'un texte ou d'une échéance locale, et une valeur future exige une
/// capacité/version explicitement négociée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationEventKind {
    ReminderSent,
}

/// Charge canonique d'une réponse Maicie. L'ordre de déclaration est l'ordre
/// filaire normatif du contrat 015.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GuichetReplyPayload {
    DeliveryReport {
        objective_id: String,
        delegation_id: String,
        delivery_hash: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        review_verdict: Option<ReviewVerdictEvidence>,
    },
    MissionStatus {
        delegation_id: String,
        objective_id: String,
        coordination_state: GuichetCoordinationState,
        local_delivery: GuichetLocalDelivery,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transport_observation: Option<GuichetTransportObservation>,
        freshness: GuichetFreshness,
    },
    DeadlineQuestion {
        delegation_id: String,
        duration_class: GuichetDurationClass,
        deadline_at: i64,
    },
    Delegate {
        status: GuichetDelegateMutationStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        objective_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delegation_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        participant: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        candidates: Vec<String>,
        waiting_on_prerequisites: bool,
        replayed: bool,
    },
    RegistreAdd {
        status: GuichetRegistreAddStatus,
        constat_id: String,
    },
    ObjectiveClose {
        objective_id: String,
        decision_id: String,
        replayed: bool,
    },
    /// Refus fermé sans projection inventée quand les faits locaux demandés
    /// par la requête n'existent pas ou ne sont pas corrélés.
    Refused {
        operation: ServiceRequestOperation,
        reason: GuichetRefusalReason,
    },
}

/// Projection fermée d'une coordination Maicie : Bridget la transporte sans
/// jamais en déduire ni la compléter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetCoordinationState {
    Open,
    EnCoordination,
    AEvaluer,
    Synthetise,
    Clos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetLocalDeliveryState {
    Pending,
    Accepted,
    OutcomeUnknown,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuichetLocalDelivery {
    pub state: GuichetLocalDeliveryState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetTransportState {
    Connected,
    Unavailable,
    Gap,
    Ended,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuichetTransportObservation {
    pub state: GuichetTransportState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_state: Option<String>,
    pub observed_at: i64,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetFreshness {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetDurationClass {
    Courte,
    Normale,
    Longue,
}

/// Refus structurés de la frontière publique client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ClientRefusal {
    RoleHandshakeRequired,
    ClientRoleRequired,
    NegotiationRequired,
    AlreadyNegotiated,
    UnsupportedVersion { supported_versions: Vec<u16> },
    InvalidIssuerScope,
    ActiveScopeLimit,
    CapabilityNotNegotiated,
    MessageOutsideClientRole,
}

/// Table fermée des refus d'un ordre de lancement géré par le daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpawnRefusal {
    /// Le daemon joint son instantané du registre : le CLI ne relit jamais le
    /// fichier utilisateur et ne peut donc pas présenter une liste divergente.
    UnknownType {
        #[serde(default)]
        requested_type: String,
        #[serde(default)]
        known_types: Vec<String>,
        #[serde(default)]
        registry: String,
    },
    CommandMissing {
        command: String,
        registry: String,
    },
    /// La définition figée ne déclare pas la capacité indispensable au
    /// lancement demandé. Ce refus intervient avant toute réservation de
    /// lancement neuve et avant tout processus.
    UnsupportedCapability {
        agent_type: String,
        model: String,
        capability: String,
    },
    BillingGuard {
        variable: String,
    },
    NameActive,
    EnvUnfit {
        detail: String,
    },
    /// Le répertoire de travail est absent **du système de fichiers où le
    /// daemon a cherché**. Les deux hôtes sont portés par le refus : un
    /// opérateur fédéré cherchait du mauvais côté du tunnel faute de savoir
    /// quelle machine avait rendu le verdict.
    ///
    /// `#[serde(default)]` : un daemon antérieur sérialise `{}`, le champ
    /// devient vide et le rendu dit « machine non attestée ».
    CwdGone {
        #[serde(default)]
        searched_on: String,
        #[serde(default)]
        requested_from: String,
    },
    /// Le `cwd` d'un lancement projet ne correspond ni à la racine liée ni à
    /// un worktree Git rattaché à cette racine. Aucun chemin hôte n'est révélé
    /// au demandeur.
    ProjectCwdMismatch {
        project_id: String,
    },
    NegotiationFailed {
        detail: String,
    },
    SpawnTimeout,
    QuotaExceeded {
        limit: usize,
    },
    DaemonRecovering,
    IdempotencyExpired,
}

/// Résultat synchrone et fermé d'un ordre d'arrêt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StopOutcome {
    Stopped,
    StoppedForced { survivors_killed: usize },
    NotManaged,
    NotFound,
    Timeout { state: String },
}

/// Résultat fermé d'une relance du même agent logique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RelaunchOutcome {
    Started { name: String, generation: u64 },
    AlreadyRunning,
    NotManaged,
    NotFound,
    NotRelaunchable { reason: String },
    Rejected { reason: SpawnRefusal },
    Timeout { state: String },
}

/// Résultat fermé du retrait d'un agent de la flotte visible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecommissionOutcome {
    Decommissioned,
    DecommissionedForced { survivors_killed: usize },
    AlreadyDecommissioned,
    NotManaged,
    NotFound,
    Timeout { state: String },
}

/// Résultat fermé de l'import explicite d'un ancien agent arrêté dans le
/// registre durable du cycle de vie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AdoptStoppedOutcome {
    Adopted { generation: u64 },
    AlreadyManaged,
    NotStopped,
    NoManagedHistory,
    IncompleteHistory { reason: String },
}

/// Issue calculée d'une opération client idempotente. `OutcomeUnknown` est
/// informatif : il impose un `Lookup` ou le rejeu strict de la même enveloppe,
/// jamais une nouvelle émission avec une nouvelle clé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delivery_id: Option<String>,
    },
    /// Destinataire purgé : le sort n'est PAS inconnu — la remise est orpheline.
    Orphaned {
        expires_at: i64,
        delivery_id: String,
        reason: String,
    },
    EnvelopeMismatch,
    IdempotencyExpired,
    InvalidIssuedAt,
}

/// Fenêtre d'historique demandée par une vue attach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum AttachWindow {
    Today,
    Seq(u64),
    Date(String),
    /// Les `n` dernières séquences présentes dans le journal (tous fichiers).
    /// Résolue en `Seq(from)` inclusif au moment de l'abonnement.
    Tail(u64),
}

/// Refus explicitement typés du plan de contrôle attach.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachRefusal {
    AgentUnknown,
    AgentStopped,
    /// Le pilote n'a pas attesté de journal append-only disponible. La
    /// compatibilité attach dépend de ce fait, jamais du protocole du pilote.
    JournalUnavailable,
    /// Variante historique conservée pour les pairs plus anciens. Les
    /// nouveaux refus attach doivent employer `JournalUnavailable`.
    AgentNotAcp,
    WrapperUnavailable,
    CommandQueueSaturated,
    InvalidDate,
    FutureDate,
    DateOutsideRetention,
    ReplyNotAllowed,
    MessageOutsideAttachRole,
}

/// Taille maximale d'un fragment d'événement sur le fil attach.
pub const MAX_ATTACH_SERIALIZED_FRAME_BYTES: usize = 256 * 1024;
/// Borne de charge utile avant encodage base64. Le worker vérifie ensuite la
/// taille JSON réelle afin que la frame filaire reste sous la borne ci-dessus.
pub const MAX_ATTACH_FRAGMENT_BYTES: usize = 190 * 1024;

mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
        for chunk in bytes.chunks(3) {
            let value = (u32::from(chunk[0]) << 16)
                | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
                | u32::from(*chunk.get(2).unwrap_or(&0));
            encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
            encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);
            encoded.push(if chunk.len() > 1 {
                TABLE[((value >> 6) & 0x3f) as usize] as char
            } else {
                '='
            });
            encoded.push(if chunk.len() > 2 {
                TABLE[(value & 0x3f) as usize] as char
            } else {
                '='
            });
        }
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        if encoded.len() % 4 != 0 {
            return Err(serde::de::Error::custom("base64 incomplet"));
        }
        let mut bytes = Vec::with_capacity(encoded.len() / 4 * 3);
        for block in encoded.as_bytes().chunks(4) {
            let value = block
                .iter()
                .enumerate()
                .try_fold(0_u32, |value, (index, byte)| {
                    let bits = match byte {
                        b'A'..=b'Z' => byte - b'A',
                        b'a'..=b'z' => byte - b'a' + 26,
                        b'0'..=b'9' => byte - b'0' + 52,
                        b'+' => 62,
                        b'/' => 63,
                        b'=' if index >= 2 => 0,
                        _ => return Err(serde::de::Error::custom("base64 invalide")),
                    };
                    Ok((value << 6) | u32::from(bits))
                })?;
            bytes.push((value >> 16) as u8);
            if block[2] != b'=' {
                bytes.push((value >> 8) as u8);
            }
            if block[3] != b'=' {
                bytes.push(value as u8);
            }
        }
        Ok(bytes)
    }
}

/// Contexte de propriété transmis avec une création d'équipier. Bridget le
/// persiste comme un fait runtime sans en déduire de transition métier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnOwnership {
    pub parent_instance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_execution_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
    pub role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_children: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
}

/// Événement cursé de la descendance d'un wrapper. Le parent est déduit de la
/// connexion qui interroge et n'est donc jamais choisi par son pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLinkEventFrame {
    pub cursor: u64,
    pub event_id: String,
    pub link_id: String,
    pub child_instance_id: String,
    pub state: String,
    pub observed_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
}

/// Catégorie fermée d'un fait runtime d'enfant destiné à son coordinateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegatedRuntimeEventKind {
    Warning,
    Failed,
}

impl DelegatedRuntimeEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "warning" => Some(Self::Warning),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Projection durable, redacted et cursée d'un incident runtime délégué.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelegatedRuntimeEventFrame {
    pub cursor: u64,
    pub event_id: String,
    pub link_id: String,
    pub child_instance_id: String,
    pub child_execution_id: String,
    pub kind: DelegatedRuntimeEventKind,
    pub code: String,
    pub reference: String,
    pub observed_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
}

/// Messages envoyés par le wrapper vers le daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WrapperToDaemon {
    /// Négocie un rôle avant l'usage d'une connexion persistante.
    RoleHandshake {
        role: ConnectionRole,
    },
    /// Négocie le contrat client public, uniquement après RoleAccepted(Client).
    ClientHello {
        contract_version: u16,
        issuer_scope: String,
        capabilities: Vec<ClientCapability>,
    },
    /// Négocie le contrat du service Maicie, uniquement après RoleAccepted(Service).
    ServiceHello {
        version: u16,
        service: String,
        issuer_scope: String,
        capabilities: Vec<ServiceCapability>,
    },
    /// Demande locale Maicie vers Bridget. Elle ne reprend pas le sens inverse
    /// de `ServiceRequest`, qui reste un dépôt Bridget vers le guichet Maicie.
    #[serde(rename = "project_registry_request")]
    ProjectRegistryRequest {
        request: ProjectBindRequest,
    },
    /// Lecture ou mutation administrative du registre, toujours sur le même
    /// rôle Service authentifié que l'enregistrement initial.
    #[serde(rename = "project_registry_admin_request")]
    ProjectRegistryAdminRequest {
        request: ProjectAdminRequest,
    },
    /// Ouvre une relève bornée des faits de coordination v2. Le curseur est
    /// opaque pour le consommateur : Bridget seul lui donne un ordre durable.
    #[serde(rename = "coordination_subscribe")]
    CoordinationSubscribe {
        #[serde(rename = "v")]
        version: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        after_cursor: Option<u64>,
    },
    /// Dépôt durable produit par un wrapper enregistré vers le guichet Maicie.
    #[serde(rename = "service_request")]
    ServiceRequest {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        issued_at: i64,
        from: String,
        to: String,
        operation: ServiceRequestOperation,
        payload: ServiceRequestPayload,
    },
    /// Relève FIFO bornée d'une demande du guichet. T1504 en assure la persistance.
    #[serde(rename = "guichet_claim_next")]
    GuichetClaimNext {
        #[serde(rename = "v")]
        version: u16,
    },
    /// Rejeu strict d'un claim existant, sous son token de lease courant.
    #[serde(rename = "guichet_claim")]
    GuichetClaim {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        claim_token: String,
    },
    /// Consultation bornée d'une demande du guichet.
    #[serde(rename = "guichet_lookup")]
    GuichetLookup {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
    },
    /// Réponse de service corrélée à un claim. T1504 valide et persiste son canon.
    #[serde(rename = "guichet_reply")]
    GuichetReply {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        claim_generation: u64,
        claim_token: String,
        response_message_id: String,
        in_reply_to: String,
        outcome: GuichetOutcome,
        payload: GuichetReplyPayload,
    },
    /// Envoi à clé client. T1205 raccorde cette variante au socle durable.
    SendIdempotent {
        message: BridgetMessage,
        message_id: String,
        issued_at: i64,
    },
    /// Lecture d'une issue à l'intérieur de la portée négociée.
    Lookup {
        operation_kind: String,
        idempotency_key: String,
    },
    /// Commande de contrôle corrélée réservée aux clients ayant négocié
    /// `execution_control_v1`.
    ControlExecution {
        command: ExecutionControlCommand,
    },
    /// Le wrapper a traité l'ordre de contrôle. Il ne rapporte pas l'issue du
    /// fournisseur, qui reste publiée comme transition d'exécution.
    ControlExecutionReported {
        issuer_scope: String,
        command_id: String,
        execution_id: String,
        accepted: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refusal_reason: Option<String>,
    },
    /// Accusé durable de remise envoyé exclusivement par un wrapper.
    DeliverAcked {
        delivery_id: String,
        delivery_generation: u64,
    },
    /// Le wrapper a persisté Seen sans pouvoir confirmer l'injection.
    DeliveryIndeterminate {
        delivery_id: String,
        delivery_generation: u64,
    },
    /// Ordre idempotent de lancement d'un équipier supervisé.
    /// Attend un changement de descendance du wrapper connecté. Le curseur
    /// permet une reprise exacte après déconnexion ; le délai est borné daemon.
    WaitAgentLinks {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        after_cursor: Option<u64>,
        timeout_ms: u64,
    },
    /// Fait runtime redacted émis par l'enfant. Le daemon déduit le lien et
    /// le parent depuis la connexion, jamais depuis cette trame.
    #[serde(rename = "delegated_runtime_event")]
    DelegatedRuntimeEvent {
        execution_id: String,
        kind: DelegatedRuntimeEventKind,
        code: String,
        reference: String,
    },
    /// Accusé de la remise observée par le wrapper parent.
    #[serde(rename = "delegated_runtime_event_acknowledged")]
    DelegatedRuntimeEventAcknowledged {
        event_id: String,
    },
    SpawnOrder {
        agent_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        project: Option<ProjectReference>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ownership: Option<SpawnOwnership>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        cwd: String,
        persistent: bool,
        command_id: String,
        issued_at: i64,
        deadline_at: i64,
    },
    /// Ordre corrélé d'arrêt d'un équipier supervisé.
    StopOrder {
        name: String,
        command_id: String,
    },
    /// Relance corrélée d'un agent géré durablement arrêté.
    RelaunchOrder {
        name: String,
        command_id: String,
    },
    /// Retrait corrélé de la flotte visible, sans purge d'historique.
    DecommissionOrder {
        name: String,
        command_id: String,
    },
    /// Migration explicite d'un agent historique arrêté vers le registre v4.
    AdoptStoppedOrder {
        name: String,
        command_id: String,
    },
    /// Ouvrir un abonnement à la vue d'un équipier.
    Subscribe {
        agent: String,
        window: AttachWindow,
    },
    /// Fermer un abonnement sans fermer la connexion attach.
    Unsubscribe {
        subscription_id: String,
    },
    /// Confirmation du wrapper : le daemon peut alors l'annoncer à la vue.
    Subscribed {
        subscription_id: String,
    },
    /// Fragment binaire d'une ligne JSONL versionnée.
    JournalFragment {
        subscription_id: String,
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Fragment live indépendant des vues ; le daemon le multiplexe vers les
    /// abonnements dont le rejeu est terminé.
    LiveJournalFragment {
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Marque la frontière entre le rejeu et le suivi continu.
    SnapshotCaughtUp {
        subscription_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_seq: Option<u64>,
    },
    /// Signale une plage volontairement non rendue par une vue lente.
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Ligne de journal illisible sans séquence exploitable.
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
    },
    /// Termine un abonnement, sans impliquer la fermeture de connexion.
    End {
        subscription_id: String,
        reason: String,
    },
    /// Refus typé produit par le wrapper lors de la préparation d'un relais.
    AttachRejected {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscription_id: Option<String>,
        reason: AttachRefusal,
    },
    /// S'enregistrer auprès du daemon.
    Register {
        agent_type: String,
        /// Identité déclarée une fois à l'ouverture de la connexion. Le daemon
        /// ne vérifie pas encore la filiation du processus pair : un client
        /// local parlant le protocole brut peut donc déclarer un autre nom.
        name: Option<String>,
        #[serde(default)]
        host: Option<String>,
        #[serde(default)]
        transport: Option<String>,
        /// Canal de connexion au daemon (`unix`, `ssh-unix`, ...), distinct
        /// du protocole d'agent porté par la présence. Une omission historique
        /// conserve le dernier fait connu ; `null` l'efface explicitement.
        #[serde(
            default,
            skip_serializing_if = "ChannelReport::is_omitted",
            serialize_with = "serialize_channel_report",
            deserialize_with = "deserialize_channel_report"
        )]
        channel: ChannelReport,
        /// Mode d'attelage réellement emprunté. Son absence représente un
        /// enregistrement historique, jamais un mode à deviner.
        #[serde(default)]
        mode: Option<PresenceMode>,
        /// Localisation interactive connue, au format `session:window.pane`
        /// pour tmux. Elle reste absente lorsqu'elle n'est pas attestée.
        #[serde(default)]
        location: Option<String>,
        #[serde(default)]
        os: Option<String>,
        #[serde(default)]
        /// Instance déclarée avec le nom. Elle réduit les sources d'identité,
        /// mais ne constitue ni une authentification ni une preuve de filiation.
        instance_id: Option<String>,
        /// Regroupement de travail de l'agent : nom du dépôt d'où il a été
        /// lancé, ou domaine choisi explicitement s'il en existe un.
        #[serde(default)]
        domain: Option<String>,
        /// État ACP re-déclaré après chaque reconnexion du wrapper.
        #[serde(default)]
        turn_in_progress: bool,
        /// `Some(false)` est envoyé par les wrappers qui connaissent
        /// `JournalReady`. L'absence est réservée à la transition des
        /// wrappers historiques, dont le journal ACP est déjà une garantie du
        /// chemin de lancement ; elle ne doit jamais faire rétrograder une
        /// présence attachable lors d'un redémarrage de daemon.
        #[serde(default)]
        journal_available: Option<bool>,
    },
    /// Fait d'espace disque relevé par le wrapper juste après son
    /// enregistrement. Informatif uniquement : le daemon le projette dans
    /// l'annuaire sans l'utiliser pour accepter, refuser ou arrêter un agent.
    DiskSpace {
        fact: DiskSpaceFact,
    },
    /// Le pilote a ouvert son journal append-only pour cette connexion. Ce
    /// signal distinct du Register évite de déduire attach du mode ACP.
    JournalReady,
    /// Fait fournisseur corrélé à une exécution. Les versions anciennes ne
    /// l'émettent pas, ce qui laisse le contexte explicitement absent.
    ExecutionProviderObserved {
        context: ExecutionProviderContext,
    },
    /// Se désenregistrer.
    Unregister,
    /// Renommer un agent déjà enregistré.
    Rename {
        current_name: String,
        name: String,
    },
    /// Envoyer un message à un autre agent.
    Send(BridgetMessage),
    /// Refus terminal asynchrone d'une livraison déjà acquittée par le daemon.
    DeliveryRejected {
        id: String,
        reason: String,
    },
    /// Transition dédiée du tour ACP, distincte de l'observation `Runtime`.
    /// Fait runtime corrélé émis par un wrapper managed.
    ///
    /// Le daemon applique cette transition par comparaison état-révision-
    /// génération et ignore donc une sortie tardive ou mal corrélée.
    ExecutionStateChanged {
        transition: ExecutionStateTransition,
    },
    TurnState {
        in_progress: bool,
    },
    /// Annuler une demande suivie appartenant à l'agent courant.
    CancelRequest {
        id: String,
        sender: String,
        reason: Option<String>,
    },
    /// Lister les demandes suivies de l'agent courant.
    ListRequests {
        sender: String,
        limit: u16,
    },
    /// Projeter le ledger détenu par le daemon, pour un client fédéré qui ne
    /// possède pas sa base SQLite locale.
    LedgerProjection {
        scope: LedgerScope,
        limit: u16,
    },
    /// Signal de vie (périodique).
    Heartbeat,
    /// Demander la liste des agents connectés.
    ListAgents,
    /// Demande au daemon ce qu'il atteste de LUI-MÊME : sa machine et sa base.
    ///
    /// Un client fédéré ne peut pas les déduire — il affichait jusqu'ici SES
    /// chemins à côté de chiffres venus d'ici. Message dédié plutôt qu'un champ
    /// de plus sur `ClientWelcome` : aucune construction existante à modifier,
    /// donc aucun fichier tiers touché.
    DaemonIdentityRequest,
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
    /// Rapporter le modèle réellement servi, tel qu'annoncé par le flux.
    ///
    /// Ce n'est pas une observation `Runtime` : aucune source fermée n'est
    /// ajoutée. L'absence de ce message n'autorise aucun verdict d'écart.
    /// Le daemon compare au modèle épinglé et n'en tire aucune décision
    /// automatique — affichage et journal seulement.
    ServedModel {
        agent: String,
        model: String,
    },
    /// Rapporter un fait de limite attesté par le pilote d'un agent.
    ///
    /// L'absence de ce message ne permet aucune déduction : une limite inconnue
    /// reste inconnue. Cette observation n'autorise ni refus ni bascule.
    RateLimit {
        agent: String,
        /// Fenêtre opaque du fournisseur, par exemple `five_hour`.
        window: String,
        /// Statut opaque attesté, par exemple `allowed` ou `rejected`.
        status: String,
        /// Instant Unix de retour fourni par le fournisseur, absent si inconnu.
        #[serde(default)]
        resets_at: Option<i64>,
        /// Pourcentage de volume consommé dans la fenêtre, absent si non attesté.
        #[serde(default)]
        used_percent: Option<u8>,
        source: RateLimitSource,
    },
    /// Rapporter une consommation de tour attestée par le pilote.
    ///
    /// Chaque message est un échantillon horodaté côté daemon. L'absence
    /// d'échantillon dans une fenêtre de mission reste « inconnu », jamais zéro.
    Usage {
        agent: String,
        input_tokens: u64,
        /// Corrélation facultative vers l exécution qui a produit l échantillon.
        /// Sans elle, le fait reste consultable par agent mais ne peut pas
        /// alimenter un budget d autonomie.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        execution_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        execution_generation: Option<u64>,
        output_tokens: u64,
        cache_creation_input_tokens: u64,
        cache_read_input_tokens: u64,
        source: UsageSource,
    },
    /// Agréger les échantillons d'usage d'un agent dans une fenêtre fermée.
    /// Réponse : `UsageWindowResult`. Aucun échantillon → `aggregate: None`
    /// (inconnu), jamais un agrégat à zéro inventé.
    UsageWindow {
        agent: String,
        from_secs: i64,
        to_secs: i64,
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
    /// Réponse `thread/start` effectivement lue du pilote Codex natif.
    #[serde(rename = "codex-app-server")]
    CodexAppServer,
    /// Rapporté par le hook Stop d'un agent Claude Code.
    #[serde(rename = "claude-hook")]
    ClaudeHook,
    /// Lu de manière périodique dans le transcript JSONL de Claude Code.
    #[serde(rename = "claude-transcript")]
    ClaudeTranscript,
    /// Déclaré explicitement via `bridget runtime`.
    #[serde(rename = "declared")]
    Declared,
}

impl std::fmt::Display for RuntimeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            RuntimeSource::CodexRollout => "codex-rollout",
            RuntimeSource::CodexAppServer => "codex-app-server",
            RuntimeSource::ClaudeHook => "claude-hook",
            RuntimeSource::ClaudeTranscript => "claude-transcript",
            RuntimeSource::Declared => "declared",
        };
        f.write_str(label)
    }
}

/// Origine d'une observation de limite. Elle est fermée afin qu'un fournisseur
/// inconnu ne puisse pas se faire passer pour une capacité attestée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateLimitSource {
    /// Événement `rate_limit_event` effectivement lu du flux Claude natif.
    #[serde(rename = "claude-stream-json")]
    ClaudeStreamJson,
    /// Réponse ou notification `account/rateLimits/*` de Codex app-server.
    #[serde(rename = "codex-app-server")]
    CodexAppServer,
    /// Bloc `rate_limits` du payload StatusLine de Claude Code, relevé par le
    /// hook. Distinct de `claude-stream-json` : ce n'est ni le même canal ni
    /// les mêmes champs — le StatusLine atteste un pourcentage consommé et un
    /// instant de retour, jamais un statut `allowed`/`rejected`. C'est la
    /// seule source de limite pour un Claude interactif, qui n'a pas de flux
    /// natif.
    #[serde(rename = "claude-statusline")]
    ClaudeStatusLine,
}

impl std::fmt::Display for RateLimitSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitSource::ClaudeStreamJson => f.write_str("claude-stream-json"),
            RateLimitSource::CodexAppServer => f.write_str("codex-app-server"),
            RateLimitSource::ClaudeStatusLine => f.write_str("claude-statusline"),
        }
    }
}

/// Origine d'une observation de consommation. Fermée : un fournisseur inconnu
/// ne peut pas se faire passer pour une capacité attestée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageSource {
    /// Compteurs lus dans le flux Claude `stream-json` (message.usage / result).
    #[serde(rename = "claude-stream-json")]
    ClaudeStreamJson,
}

impl std::fmt::Display for UsageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageSource::ClaudeStreamJson => f.write_str("claude-stream-json"),
        }
    }
}

/// Compteurs d'un échantillon de tour. `facturable` = in + out + cache_create ;
/// `cache_read` reste hors facturable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageTokens {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

impl UsageTokens {
    pub fn facturable_tokens(self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cache_creation_input_tokens)
    }
}

/// Agrégat de consommation sur une fenêtre. `facturable` = in + out +
/// cache_create ; `cache_read` reste séparé (leçon du comparatif).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageAggregate {
    pub turns: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub facturable_tokens: u64,
}

/// Messages envoyés par le daemon vers le wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedMcpDefinition {
    pub interactive: String,
    pub acp_session: bool,
}

/// Matrice déclarative des capacités réellement promises par un adaptateur.
///
/// Elle est portée par la définition résolue et donc par son digest : une
/// reprise ne relit jamais une sonde volatile du pilote pour décider si elle
/// peut démarrer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdapterCapabilities {
    #[serde(default)]
    pub execution_paths: Vec<String>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelCapabilities>,
    /// Faits de fournisseur relevés avant activation. Leur absence garde la
    /// compatibilité de registre mais interdit toute opération qui les exige.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<ProviderObservation>,
}

impl Default for AdapterCapabilities {
    fn default() -> Self {
        // Les définitions historiques étaient toutes lancées par ACP. Ce seul
        // chemin reste la valeur de migration ; un modèle explicite demeure
        // absent tant qu'il n'est pas réellement déclaré.
        Self {
            execution_paths: vec!["acp".to_string()],
            models: BTreeMap::new(),
            observed: None,
        }
    }
}

/// Opération effectivement attestée par une version fournisseur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderOperation {
    Interrupt,
    Steer,
    Resume,
    Fork,
    Approval,
}

/// Baseline versionnée, sans environnement ni contenu utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderObservation {
    pub binary_path: String,
    pub binary_version: String,
    pub binary_digest: String,
    pub contract_version: String,
    #[serde(default)]
    pub operations: Vec<ProviderOperation>,
}
impl ProviderObservation {
    /// Une capacité absente ou refusée reste indisponible. Cette vérification
    /// commune évite que chaque superviseur interprète la baseline autrement.
    pub fn supports(&self, operation: ProviderOperation) -> bool {
        self.operations.contains(&operation)
    }
}

/// Vue publique réduite d'une baseline fournisseur. Les chemins et empreintes
/// restent dans le registre de lancement: l'UI ne reçoit que la version, le
/// contrat et les opérations effectivement attestées.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderUiProjection {
    pub binary_version: String,
    pub contract_version: String,
    #[serde(default)]
    pub operations: Vec<ProviderOperation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

/// Capacités opaques déclarées pour un modèle précis, sans substitution.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModelCapabilities {
    #[serde(default)]
    pub efforts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedAgentDefinition {
    pub command: String,
    pub args: Vec<String>,
    pub protocol: String,
    pub forbidden_env: Vec<String>,
    /// Noms des variables héritées ; leurs valeurs secrètes ne sont jamais
    /// persistées ni exposées dans la preuve publique.
    pub pass_env: Vec<String>,
    pub permissions: String,
    /// Non-secret profile directory passed to the spawn without serializing its secrets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_config_dir: Option<String>,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    pub mcp: ResolvedMcpDefinition,
    #[serde(default)]
    pub capabilities: AdapterCapabilities,
    /// SHA-256 hexadécimal de tous les paramètres runtime précédents.
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DaemonToWrapper {
    /// Le rôle demandé est accepté pour cette connexion.
    RoleAccepted { role: ConnectionRole },
    /// Contrat et capacités réellement négociés avec un client public.
    ClientWelcome {
        version: u16,
        #[serde(default = "unknown_build_id")]
        build_id: String,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ClientCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice client.
    ClientRejected { reason: ClientRefusal },
    /// Contrat et capacité réellement négociés avec un service Maicie.
    ServiceWelcome {
        version: u16,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ServiceCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice de service.
    ServiceRejected { reason: ServiceRefusal },
    /// Issue terminale du registre local, corrélée à la commande Maicie.
    #[serde(rename = "project_registry_outcome")]
    ProjectRegistryOutcome { outcome: ProjectBindOutcome },
    /// Issue corrélée d'une lecture ou mutation administrative du registre.
    #[serde(rename = "project_registry_admin_outcome")]
    ProjectRegistryAdminOutcome { outcome: ProjectAdminOutcome },
    /// Issue durable ou calculée d'une opération du guichet.
    #[serde(rename = "guichet_result")]
    GuichetResult {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        issue: String,
        expires_at: i64,
        /// Charge terminale relue depuis les octets durables du maître. Elle
        /// reste absente tant que l'issue n'est pas terminale.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<GuichetReplyPayload>,
    },
    /// Une seule demande a été relevée sous une lease durable.
    #[serde(rename = "guichet_claimed")]
    GuichetClaimed {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        request_id: String,
        #[serde(with = "base64_bytes")]
        canonical_request: Vec<u8>,
        /// Attestation produite par le daemon et persistée hors des octets
        /// canoniques fournis par l'appelant.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        authorization_attestation:
            Option<crate::greffe_authorization::GreffeAuthorizationAttestation>,
        claimed_at: i64,
        claim_generation: u64,
        claim_token: String,
        claim_lease_expires_at: i64,
        expires_at: i64,
    },
    /// Aucun élément FIFO n'est actuellement relevable.
    #[serde(rename = "guichet_empty")]
    GuichetEmpty {
        #[serde(rename = "v")]
        version: u16,
    },
    /// Fait terminal durable, émis exclusivement par Bridget vers le service
    /// Maicie après la transition SQLite correspondante.
    #[serde(rename = "request_lifecycle_event")]
    RequestLifecycleEvent {
        #[serde(rename = "v")]
        version: u16,
        issuer_scope: String,
        event_id: String,
        request_id: String,
        state: GuichetLifecycleState,
        observed_at: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        response_message_id: Option<String>,
    },
    /// Fait non terminal de coordination, envoyé seulement aux services ayant
    /// négocié `coordination_events_v1` en plus de `maicie_guichet`.
    #[serde(rename = "coordination_event")]
    CoordinationEvent {
        #[serde(rename = "v")]
        version: u16,
        event_id: String,
        request_id: String,
        kind: CoordinationEventKind,
        reminder_message_id: String,
        recipient: String,
        generation: u64,
        observed_at: i64,
        /// Présent seulement sur la relève cursée v2. Le conserver dans
        /// l'événement rend le rejeu exactement vérifiable par le client.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cursor: Option<u64>,
    },
    /// Frontière explicite entre le rejeu cursé et le suivi transport. Avant
    /// cette trame, un consommateur ne peut jamais déclarer l'observation
    /// fraîche.
    #[serde(rename = "coordination_snapshot_caught_up")]
    CoordinationSnapshotCaughtUp {
        #[serde(rename = "v")]
        version: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_cursor: Option<u64>,
    },
    /// Un curseur ne peut pas être repris sans trou attesté. Ce n'est pas un
    /// fait métier et il reste visible jusqu'à une nouvelle relève complète.
    #[serde(rename = "coordination_gap")]
    CoordinationGap {
        #[serde(rename = "v")]
        version: u16,
        from_cursor: u64,
        to_cursor: u64,
        reason: String,
    },
    /// Bridget ne peut pas lire la source durable de coordination. Cette
    /// observation est distincte d'une lacune de curseur et ne vaut jamais
    /// fraîcheur implicite.
    #[serde(rename = "coordination_unavailable")]
    CoordinationUnavailable {
        #[serde(rename = "v")]
        version: u16,
        reason: String,
    },
    /// Issue durable ou calculée d'un `SendIdempotent`.
    IdempotencyResult {
        operation_kind: String,
        idempotency_key: String,
        issue: IdempotencyIssue,
    },
    /// Issue immédiate de la validation Bridget d'une commande de contrôle.
    ControlExecutionResult {
        command_id: String,
        execution_id: String,
        outcome: ExecutionControlOutcome,
    },
    /// Ordre à destination du wrapper qui porte l'exécution ciblée.
    /// Delta de descendance du wrapper demandeur, lu ou réveillé à partir du
    /// journal durable. Une réponse vide indique uniquement le timeout borné.
    AgentLinkEvents {
        events: Vec<AgentLinkEventFrame>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_cursor: Option<u64>,
        timed_out: bool,
    },
    /// Fait runtime durable à convertir en notification système non intrusive.
    #[serde(rename = "delegated_runtime_event")]
    DelegatedRuntimeEvent { event: DelegatedRuntimeEventFrame },
    ///
    /// Cette trame ne franchit jamais la frontière client publique.
    ControlExecutionDispatch {
        issuer_scope: String,
        command: ExecutionControlCommand,
    },
    /// Succès d'un spawn, émis seulement après le `Register` réel.
    SpawnAccepted {
        command_id: String,
        name: String,
        /// `None` n'est toléré que pour le rejeu d'une issue créée avant la
        /// migration du registre résolu ; tout nouveau spawn fournit `Some`.
        definition: Option<ResolvedAgentDefinition>,
    },
    /// Refus terminal et rejouable d'un spawn.
    SpawnRejected {
        command_id: String,
        reason: SpawnRefusal,
    },
    /// Issue synchrone d'un ordre d'arrêt.
    StopResult {
        command_id: String,
        outcome: StopOutcome,
    },
    /// Issue synchrone d'une relance, émise après connexion réelle.
    RelaunchResult {
        command_id: String,
        outcome: RelaunchOutcome,
    },
    /// Issue synchrone d'un décommissionnement.
    DecommissionResult {
        command_id: String,
        outcome: DecommissionOutcome,
    },
    /// Issue synchrone de l'adoption d'un agent historique arrêté.
    AdoptStoppedResult {
        command_id: String,
        outcome: AdoptStoppedOutcome,
    },
    /// Remise aval réservée au wrapper destinataire.
    DeliverIdempotent {
        delivery_id: String,
        recipient_instance_id: String,
        delivery_generation: u64,
        expires_at: i64,
        message: BridgetMessage,
    },
    /// Souscription du daemon vers le wrapper lecteur du journal.
    Subscribe {
        subscription_id: String,
        agent: String,
        window: AttachWindow,
    },
    /// Désabonnement relayé au wrapper.
    Unsubscribe { subscription_id: String },
    /// Confirmation d'abonnement envoyée à la vue après acceptation wrapper.
    Subscribed { subscription_id: String },
    /// Fragment d'événement relayé à la vue attachée.
    JournalFragment {
        subscription_id: String,
        seq: u64,
        offset: u64,
        #[serde(rename = "final")]
        final_fragment: bool,
        #[serde(with = "base64_bytes")]
        bytes: Vec<u8>,
    },
    /// Le rejeu est terminé ; `through_seq` est absent si la fenêtre est vide.
    SnapshotCaughtUp {
        subscription_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        through_seq: Option<u64>,
    },
    /// Lacune de rendu coalescée, émise avant l'événement suivant conservé.
    Gap {
        subscription_id: String,
        from_seq: u64,
        to_seq: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Diagnostic relayé quand une ligne n'a pas de séquence à lacuner.
    JournalReadError {
        subscription_id: String,
        line: u64,
        offset: u64,
        reason: String,
    },
    /// Fin motivée d'un abonnement attach.
    End {
        subscription_id: String,
        reason: String,
    },
    /// Refus typé du plan de contrôle attach.
    AttachRejected {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subscription_id: Option<String>,
        reason: AttachRefusal,
        /// Mode attesté qui motive un refus d'attachement. Absent pour les
        /// refus sans agent ou émis par un wrapper qui ne connaît pas la
        /// présence complète.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mode: Option<PresenceMode>,
        /// Localisation interactive attestée, uniquement utile pour le mode
        /// tmux. Elle n'est jamais déduite ni reconstruite côté client.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        location: Option<String>,
    },
    /// Confirmation d'enregistrement avec le nom final.
    Registered { name: String },
    /// Confirmation d'un renommage.
    Renamed { old_name: String, name: String },
    /// Livrer un message à l'agent.
    Deliver(BridgetMessage),
    /// Livrer un travail dont l'exécution durable a déjà été admise.
    ///
    /// Les wrappers historiques continuent de recevoir Deliver. Cette
    /// variante n'est utilisée qu'après activation explicite de la double
    /// écriture du plan de contrôle.
    DeliverExecution {
        message: BridgetMessage,
        execution_id: String,
        generation: u64,
        revision: u64,
    },
    /// Retirer un message de la file du transport, sans l'injecter.
    CancelDelivery { id: String, reason: String },
    /// Acquittement d'un envoi.
    Ack { id: String },
    /// Refus d'un envoi avec raison.
    Nack { id: String, reason: String },
    /// Échec terminal différé d'un envoi attach déjà acquitté.
    DeliveryRejected { id: String, reason: String },
    /// Le daemon s'éteint.
    Disconnect,
    /// Réponse à ListAgents.
    AgentList { agents: Vec<AgentInfo> },
    /// Machine, base et instance attestées par le daemon lui-même.
    ///
    /// `instance_id` est renouvelé à chaque démarrage : contrairement à
    /// `build_id`, il distingue deux daemons successifs du même binaire sur le
    /// même hôte et la même base.
    DaemonIdentityReport {
        host: String,
        db_path: String,
        instance_id: String,
    },
    /// Réponse à UsageWindow. `aggregate: None` signifie « aucun échantillon
    /// attesté dans la fenêtre » — le greffe doit rendre « inconnu », pas zéro.
    UsageWindowResult {
        agent: String,
        #[serde(default)]
        aggregate: Option<UsageAggregate>,
    },
    /// État final d'une annulation.
    RequestCancelled { id: String, state: String },
    /// Liste des demandes suivies accessibles à l'agent courant.
    RequestList { requests: Vec<RequestInfo> },
    /// Projection bornée du ledger, indépendante de tout rendu CLI.
    LedgerProjection {
        messages: Vec<LedgerMessage>,
        requests: Vec<RequestInfo>,
    },
}

fn unknown_build_id() -> String {
    "unknown".to_string()
}

/// Sous-ensembles fermés de la projection de lecture du ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerScope {
    Messages,
    Requests,
    Both,
}

/// État de remise exposé au lecteur du ledger : distinguer « émis » et « vu »
/// sans diluer l'accusé (Seen ≠ Injected ≠ PromptDispatched).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerDeliveryStatus {
    /// Remise encore en `dispatching` — visible, pas encore accusée.
    EnVol,
    /// Remise `acked` — le destinataire a accusé.
    Recu,
    /// Remise passée en quarantaine absorbante.
    Indetermine,
    /// Destinataire purgé : sort connu (orphelin), distinct de l'inconnu.
    Orphelin,
}

impl LedgerDeliveryStatus {
    pub fn from_phase(phase: &str) -> Option<Self> {
        match phase {
            "dispatching" => Some(Self::EnVol),
            "acked" => Some(Self::Recu),
            "indeterminate" => Some(Self::Indetermine),
            "orphaned" => Some(Self::Orphelin),
            _ => None,
        }
    }

    pub fn label_fr(self) -> &'static str {
        match self {
            Self::EnVol => "en vol",
            Self::Recu => "reçu",
            Self::Indetermine => "indéterminé",
            Self::Orphelin => "orphelin",
        }
    }
}

/// Échange stocké par le daemon et exposé aux clients de lecture.
///
/// Plage P31 : `protocol.rs:LedgerMessage` — réservée au lot
/// `fix/ledger-emission-avant-ack` (visibilité ledger ≠ accusé + marqueur
/// de projection `delivery_status`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerMessage {
    pub id: String,
    /// Horodatage d'émission (gravure au ledger), pas d'accusé — ne pas en
    /// déduire un délai de livraison.
    pub ts: i64,
    pub sender: String,
    pub target: String,
    pub body: String,
    /// Absent pour les entrées hors saga idempotente (Send legacy / antérieur).
    /// `en_vol` ⇔ phase SQL `dispatching` ⇔ dépôt CLI « en vol » / in_flight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_status: Option<LedgerDeliveryStatus>,
}

impl WrapperToDaemon {
    /// Vérifie la matrice fermée d'une connexion déjà négociée comme attach.
    /// Le handshake est volontairement exclu : il n'est admis qu'avant que le
    /// daemon n'enregistre le rôle de la connexion.
    pub fn attach_refusal(&self) -> Option<AttachRefusal> {
        match self {
            Self::Subscribe { .. } | Self::Unsubscribe { .. } | Self::Heartbeat => None,
            Self::Send(message) if !message.reply => None,
            Self::Send(_) => Some(AttachRefusal::ReplyNotAllowed),
            _ => Some(AttachRefusal::MessageOutsideAttachRole),
        }
    }
}

impl DaemonToWrapper {
    /// Vérifie la matrice de réception du client attach. Le daemon l'emploiera
    /// lors du fan-out : les livraisons réservées au wrapper ne traversent pas
    /// la frontière de rôle.
    pub fn allowed_for_attach(&self) -> bool {
        matches!(
            self,
            Self::RoleAccepted {
                role: ConnectionRole::Attach
            } | Self::Subscribed { .. }
                | Self::JournalFragment { .. }
                | Self::SnapshotCaughtUp { .. }
                | Self::Gap { .. }
                | Self::JournalReadError { .. }
                | Self::End { .. }
                | Self::AttachRejected { .. }
                | Self::Ack { .. }
                | Self::Nack { .. }
                | Self::DeliveryRejected { .. }
        )
    }
}

/// Sérialise un message en ligne JSON (newline-delimited JSON).
pub fn encode<T: Serialize>(msg: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(msg)
}

/// Désérialise une ligne JSON.
pub fn decode<T: for<'de> Deserialize<'de>>(line: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(line)
}

/// Espace libre attesté par le wrapper qui travaille sur ce volume.
///
/// C'est une photographie locale, pas une consigne de ramassage. Elle sert à
/// voir quel disque est réellement sollicité avant toute décision humaine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskSpaceFact {
    pub volume: String,
    pub free_bytes: u64,
    pub observed_at_unix: i64,
}

/// Information sur un agent connecté.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "AgentInfoWire")]
pub struct AgentInfo {
    pub name: String,
    pub agent_type: String,
    pub connection_id: String,
    pub host: String,
    /// Protocole d'agent réellement utilisé (`tmux`, `acp`,
    /// `codex_app_server`, `claude_stream_json`, ...).
    pub transport: String,
    /// Canal de connexion au daemon. Absent lorsqu'aucun wrapper ne l'a
    /// attesté ; il n'est jamais déduit du type d'agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Mode d'attelage attesté. `None` représente une présence historique
    /// dont le mode n'a jamais été annoncé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<PresenceMode>,
    /// Localisation interactive la plus précise attestée, jamais reconstruite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
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
    /// Faits de limite par fenêtre attestée. Vide = aucune observation.
    /// Chaque entrée est indépendante : un fait `five_hour` n'efface pas
    /// un fait `seven_day` déjà présent. Informational seulement.
    #[serde(default)]
    pub rate_limits: Vec<RateLimitFact>,
    /// Écart entre le modèle épinglé de la définition et le modèle attesté
    /// par le flux. Absent si le flux est muet ou si les deux étiquettes
    /// coïncident. Informational seulement : aucun refus ni bascule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_mismatch: Option<ModelMismatchFact>,
    /// Dernier espace libre attesté par cette machine. Informatif seulement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_space: Option<DiskSpaceFact>,
    /// L'agent survit-il au redémarrage du service ? `None` quand aucune
    /// entrée de flotte ne l'atteste — un agent lancé hors `bridget spawn`
    /// n'est pas drainé, la question ne se pose pas pour lui. Toujours
    /// sérialisé, `null` compris : une ronde doit pouvoir distinguer
    /// « indéterminable » d'un daemon trop ancien pour publier le champ.
    #[serde(default)]
    pub persistent: Option<bool>,
    /// Baseline fournisseur réduite, absente tant qu'elle n'est pas attestée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderUiProjection>,
    /// Projection durable de l'exécution en cours, absente tant que la bascule
    /// de double écriture n'est pas activée pour l'agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionUiProjection>,
    /// Relation durable entre cet agent, son parent et son mandat. Absente pour
    /// les agents qui ne proviennent pas d'un spawn propriétaire Bridget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_link: Option<AgentLinkUiProjection>,
}

/// Forme fil de lecture : accepte l'ancien champ mono-fenêtre `rate_limit`
/// et le nouveau `rate_limits`. Un fait ancien devient un Vec d'un élément
/// sans inventer de fenêtre.
#[derive(Debug, Deserialize)]
struct AgentInfoWire {
    name: String,
    agent_type: String,
    connection_id: String,
    host: String,
    transport: String,
    #[serde(default)]
    channel: Option<String>,
    #[serde(default)]
    mode: Option<PresenceMode>,
    #[serde(default)]
    location: Option<String>,
    #[serde(default = "unknown_os")]
    os: String,
    state: String,
    last_seen_secs: u64,
    reconnect_count: u32,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    effort: Option<String>,
    #[serde(default)]
    rate_limits: Vec<RateLimitFact>,
    /// Ancien instantané unique. Lu uniquement si `rate_limits` est vide.
    #[serde(default)]
    rate_limit: Option<RateLimitFact>,
    #[serde(default)]
    agent_link: Option<AgentLinkUiProjection>,
    #[serde(default)]
    model_mismatch: Option<ModelMismatchFact>,
    #[serde(default)]
    execution: Option<ExecutionUiProjection>,
    #[serde(default)]
    disk_space: Option<DiskSpaceFact>,
    #[serde(default)]
    persistent: Option<bool>,
    #[serde(default)]
    provider: Option<ProviderUiProjection>,
}

impl From<AgentInfoWire> for AgentInfo {
    fn from(wire: AgentInfoWire) -> Self {
        let rate_limits = if !wire.rate_limits.is_empty() {
            wire.rate_limits
        } else {
            wire.rate_limit.into_iter().collect()
        };
        Self {
            name: wire.name,
            agent_type: wire.agent_type,
            connection_id: wire.connection_id,
            host: wire.host,
            transport: wire.transport,
            channel: wire.channel,
            mode: wire.mode,
            location: wire.location,
            os: wire.os,
            state: wire.state,
            last_seen_secs: wire.last_seen_secs,
            reconnect_count: wire.reconnect_count,
            execution: wire.execution,
            domain: wire.domain,
            agent_link: wire.agent_link,
            model: wire.model,
            effort: wire.effort,
            rate_limits,
            model_mismatch: wire.model_mismatch,
            disk_space: wire.disk_space,
            persistent: wire.persistent,
            provider: wire.provider,
        }
    }
}
/// Vue compacte d'une exécution durable pour l'annuaire public.
/// Les absences restent des absences attestées : aucune activité fournisseur ne
/// se déduit de la seule présence réseau.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionUiProjection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_age_secs: Option<u64>,
    #[serde(default)]
    pub queue_depth: u64,
    /// Mode native, forked ou reconstructed, absent sans ascendance attestée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation_mode: Option<String>,
}

/// Projection publique d'un lien d'agent. Elle rend visibles l'ascendance et
/// le mandat sans déduire un état métier depuis la présence du processus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLinkUiProjection {
    pub link_id: String,
    pub parent_instance_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_execution_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    /// Référence projet reçue à l'admission du spawn. Le domaine historique
    /// reste volontairement un champ distinct de l'annuaire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectReference>,
    pub role: String,
    pub agent_path: String,
    pub state: String,
    /// Descendants directs dont le lien propriétaire reste ouvert.
    pub direct_descendants: u64,
    /// Descendants ouverts à toute profondeur, sans inférer de coût ou d'état.
    pub descendants: u64,
}

/// Fait de limite exposé dans l'annuaire. Les chaînes fournisseur restent
/// opaques ; `resets_at` ou `used_percent` manquants restent absents, jamais
/// inventés.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitFact {
    pub window: String,
    pub status: String,
    #[serde(default)]
    pub resets_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_percent: Option<u8>,
}

/// Écart attesté entre le modèle demandé et le modèle réellement servi.
/// Les deux chaînes restent opaques : aucune aliasisation n'est inventée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMismatchFact {
    pub pinned: String,
    pub served: String,
}

impl ModelMismatchFact {
    /// Produit un écart seulement si les deux étiquettes sont présentes et
    /// distinctes. Un flux muet ou un épinglage absent ne produit rien.
    pub fn observe(pinned: Option<&str>, served: Option<&str>) -> Option<Self> {
        match (pinned, served) {
            (Some(pinned), Some(served)) if pinned != served => Some(Self {
                pinned: pinned.to_string(),
                served: served.to_string(),
            }),
            _ => None,
        }
    }
}

fn unknown_os() -> String {
    "inconnu".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestInfo {
    pub id: String,
    pub sender: String,
    pub target: String,
    pub state: String,
    pub created_at: i64,
    pub deadline_at: i64,
    pub cancel_reason: Option<String>,
    #[serde(default)]
    pub deferred_reminder_level: Option<u8>,
    #[serde(default)]
    pub deferred_reminder_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
        "../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
    );
    const COORDINATION_EVENTS_FIXTURE: &str = include_str!(
        "../../../specs/016-coordination-active/contracts/fixtures/coordination-events-v1.jsonl"
    );
    const COORDINATION_STREAM_FIXTURE: &str = include_str!(
        "../../../specs/016-coordination-active/contracts/fixtures/coordination-stream-v2.jsonl"
    );

    #[test]
    fn contrat_revue_valide_ref_et_sha_fermes() {
        let valid = ReviewTarget {
            target_ref: "origin/fix/review/sub".to_string(),
            expected_head: "a".repeat(40),
        };
        assert_eq!(
            valid.remote_and_branch(),
            Some(("origin", "fix/review/sub"))
        );
        assert!(valid.is_valid());
        for target_ref in ["origin", "./main", "origin/../main", "-origin/main"] {
            assert!(
                !ReviewTarget {
                    target_ref: target_ref.to_string(),
                    expected_head: "a".repeat(40),
                }
                .is_valid(),
                "référence interdite acceptée : {target_ref}"
            );
        }
        assert!(
            !ReviewTarget {
                target_ref: "origin/main".to_string(),
                expected_head: "A".repeat(40),
            }
            .is_valid()
        );
    }

    #[test]
    fn delivery_report_ordinaire_conserve_ses_octets_sans_bloc_revue() {
        let payload = ServiceRequestPayload::DeliveryReport {
            objective_id: "objective-1".to_string(),
            delegation_id: "delegation-1".to_string(),
            delivery_hash: "0".repeat(64),
            in_reply_to: "message-1".to_string(),
            review_verdict: None,
        };
        assert_eq!(
            serde_json::to_string(&payload).unwrap(),
            format!(
                "{{\"objective_id\":\"objective-1\",\"delegation_id\":\"delegation-1\",\"delivery_hash\":\"{}\",\"in_reply_to\":\"message-1\"}}",
                "0".repeat(64)
            )
        );
    }

    #[test]
    fn delegation_historique_reste_v1_et_cible_de_revue_exige_v2() {
        let ordinary = ServiceRequestPayload::Delegate {
            goal: "relire le lot".to_string(),
            review_target: None,
            explicit_target: None,
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Courte,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
        };
        assert_eq!(
            ordinary.required_contract_version(),
            SERVICE_CONTRACT_VERSION
        );
        assert!(
            !serde_json::to_string(&ordinary)
                .unwrap()
                .contains("review_target")
        );

        let targeted = ServiceRequestPayload::Delegate {
            goal: "relire le lot".to_string(),
            review_target: Some(ReviewTarget {
                target_ref: "origin/session-047-verdict-tete-reecrite".to_string(),
                expected_head: "a".repeat(40),
            }),
            explicit_target: None,
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Courte,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
        };
        assert_eq!(
            targeted.required_contract_version(),
            REVIEW_DELEGATE_CONTRACT_VERSION
        );
        let wire = serde_json::to_string(&targeted).unwrap();
        assert!(wire.contains(r#""target_ref":"origin/session-047-verdict-tete-reecrite""#));
        assert_eq!(
            serde_json::from_str::<ServiceRequestPayload>(&wire).unwrap(),
            targeted
        );
    }

    #[test]
    fn service_negotiation_v1_emploie_la_fixture_canonique_partagee() {
        let lines = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 5, "la fixture couvre hello, welcome et refus");

        let role: WrapperToDaemon = decode(lines[0]).unwrap();
        assert_eq!(encode(&role).unwrap(), lines[0]);
        assert!(matches!(
            role,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Service
            }
        ));

        let accepted: DaemonToWrapper = decode(lines[1]).unwrap();
        assert_eq!(encode(&accepted).unwrap(), lines[1]);
        let hello: WrapperToDaemon = decode(lines[2]).unwrap();
        assert_eq!(encode(&hello).unwrap(), lines[2]);
        assert!(matches!(hello, WrapperToDaemon::ServiceHello { .. }));

        let welcome: DaemonToWrapper = decode(lines[3]).unwrap();
        assert_eq!(encode(&welcome).unwrap(), lines[3]);
        assert!(matches!(welcome, DaemonToWrapper::ServiceWelcome { .. }));

        let rejected: DaemonToWrapper = decode(lines[4]).unwrap();
        assert_eq!(encode(&rejected).unwrap(), lines[4]);
        assert!(matches!(
            rejected,
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            }
        ));
    }

    #[test]
    fn coordination_stream_v2_emploie_la_fixture_canonique_fermee() {
        let lines = COORDINATION_STREAM_FIXTURE.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 10);
        for index in [0, 2, 4] {
            let frame: WrapperToDaemon = decode(lines[index]).unwrap();
            assert_eq!(encode(&frame).unwrap(), lines[index]);
        }
        for index in [1, 3, 5, 6, 7, 8, 9] {
            let frame: DaemonToWrapper = decode(lines[index]).unwrap();
            assert_eq!(encode(&frame).unwrap(), lines[index]);
        }
        assert!(matches!(
            decode::<WrapperToDaemon>(lines[4]).unwrap(),
            WrapperToDaemon::CoordinationSubscribe {
                version: COORDINATION_STREAM_VERSION,
                after_cursor: None,
            }
        ));
        assert!(matches!(
            decode::<DaemonToWrapper>(lines[7]).unwrap(),
            DaemonToWrapper::CoordinationGap {
                from_cursor: 2,
                to_cursor: 3,
                ..
            }
        ));
    }

    #[test]
    fn coordination_events_v1_emploie_la_fixture_canonique_fermee() {
        let lines = COORDINATION_EVENTS_FIXTURE.lines().collect::<Vec<_>>();
        assert_eq!(lines.len(), 6, "négociation, fait attesté et refus");

        let hello: WrapperToDaemon = decode(lines[2]).unwrap();
        assert_eq!(encode(&hello).unwrap(), lines[2]);
        assert!(matches!(
            hello,
            WrapperToDaemon::ServiceHello { capabilities, .. }
                if capabilities == vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV1,
                ]
        ));

        let fact: DaemonToWrapper = decode(lines[4]).unwrap();
        assert_eq!(encode(&fact).unwrap(), lines[4]);
        assert!(matches!(
            fact,
            DaemonToWrapper::CoordinationEvent {
                kind: CoordinationEventKind::ReminderSent,
                generation: 1,
                observed_at: 1_787_500_003,
                ..
            }
        ));

        let rejected: DaemonToWrapper = decode(lines[5]).unwrap();
        assert!(matches!(
            rejected,
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::InvalidEnvelope
            }
        ));
    }

    #[test]
    fn test_encode_decode_register() {
        let msg = WrapperToDaemon::Register {
            agent_type: "codex".to_string(),
            name: None,
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            channel: ChannelReport::Known("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            os: Some("Linux".to_string()),
            instance_id: Some("instance-test".to_string()),
            domain: Some("bridget".to_string()),
            turn_in_progress: false,
            journal_available: Some(false),
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
                channel,
                mode,
                location,
                os,
                instance_id,
                domain,
                turn_in_progress,
                journal_available,
            } => {
                assert_eq!(agent_type, "codex");
                assert!(name.is_none());
                assert_eq!(host.as_deref(), Some("test-host"));
                assert_eq!(transport.as_deref(), Some("unix"));
                assert_eq!(channel.as_deref(), Some("unix"));
                assert_eq!(mode, Some(PresenceMode::Acp));
                assert_eq!(location, None);
                assert_eq!(os.as_deref(), Some("Linux"));
                assert_eq!(instance_id.as_deref(), Some("instance-test"));
                assert_eq!(domain.as_deref(), Some("bridget"));
                assert!(!turn_in_progress);
                assert_eq!(journal_available, Some(false));
            }
            _ => panic!("mauvais type"),
        }
    }

    #[test]
    fn register_historique_conserve_un_mode_inconnu() {
        let json =
            r#"{"type":"Register","agent_type":"codex","name":null,"turn_in_progress":false}"#;
        let decoded: WrapperToDaemon = decode(json).unwrap();
        assert!(matches!(
            decoded,
            WrapperToDaemon::Register {
                channel: ChannelReport::Omitted,
                mode: None,
                location: None,
                ..
            }
        ));
    }

    #[test]
    fn spec_024_register_distingue_omission_inconnu_et_attestation() {
        let omitted_json = r#"{"type":"Register","agent_type":"ui","name":"humain"}"#;
        let omitted: WrapperToDaemon = decode(omitted_json).unwrap();
        assert!(matches!(
            &omitted,
            WrapperToDaemon::Register {
                channel: ChannelReport::Omitted,
                ..
            }
        ));
        assert!(!encode(&omitted).unwrap().contains("\"channel\""));

        let unknown_json =
            r#"{"type":"Register","agent_type":"ui","name":"humain","channel":null}"#;
        let unknown: WrapperToDaemon = decode(unknown_json).unwrap();
        assert!(matches!(
            &unknown,
            WrapperToDaemon::Register {
                channel: ChannelReport::Unknown,
                ..
            }
        ));
        assert!(encode(&unknown).unwrap().contains("\"channel\":null"));

        let known_json =
            r#"{"type":"Register","agent_type":"ui","name":"humain","channel":"ssh-unix"}"#;
        let known: WrapperToDaemon = decode(known_json).unwrap();
        assert!(matches!(
            &known,
            WrapperToDaemon::Register {
                channel: ChannelReport::Known(value),
                ..
            } if value == "ssh-unix"
        ));
        assert!(encode(&known).unwrap().contains("\"channel\":\"ssh-unix\""));
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
    fn test_encode_decode_acp_delivery_lifecycle() {
        let cancel = DaemonToWrapper::CancelDelivery {
            id: "message-1".to_string(),
            reason: "échéance".to_string(),
        };
        assert!(matches!(
            decode(&encode(&cancel).unwrap()).unwrap(),
            DaemonToWrapper::CancelDelivery { id, reason } if id == "message-1" && reason == "échéance"
        ));
        let rejected = WrapperToDaemon::DeliveryRejected {
            id: "message-1".to_string(),
            reason: "file ACP pleine".to_string(),
        };
        assert!(matches!(
            decode(&encode(&rejected).unwrap()).unwrap(),
            WrapperToDaemon::DeliveryRejected { id, reason } if id == "message-1" && reason == "file ACP pleine"
        ));
        assert!(matches!(
            decode(&encode(&WrapperToDaemon::TurnState { in_progress: true }).unwrap()).unwrap(),
            WrapperToDaemon::TurnState { in_progress: true }
        ));
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
    fn runtime_source_claude_transcript_est_stable() {
        let encoded = encode(&RuntimeSource::ClaudeTranscript).unwrap();
        assert_eq!(encoded, "\"claude-transcript\"");
        assert_eq!(
            decode::<RuntimeSource>(&encoded).unwrap(),
            RuntimeSource::ClaudeTranscript
        );
        assert_eq!(
            RuntimeSource::ClaudeTranscript.to_string(),
            "claude-transcript"
        );
    }

    #[test]
    fn runtime_et_limite_codex_app_server_sont_fermes() {
        let runtime = encode(&RuntimeSource::CodexAppServer).unwrap();
        assert_eq!(runtime, "\"codex-app-server\"");
        assert_eq!(
            decode::<RuntimeSource>(&runtime).unwrap(),
            RuntimeSource::CodexAppServer
        );
        let limit = encode(&RateLimitSource::CodexAppServer).unwrap();
        assert_eq!(limit, "\"codex-app-server\"");
        assert_eq!(
            decode::<RateLimitSource>(&limit).unwrap(),
            RateLimitSource::CodexAppServer
        );
        assert!(decode::<RuntimeSource>("\"codex-app-server-futur\"").is_err());
        assert!(decode::<RateLimitSource>("\"codex-app-server-futur\"").is_err());
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
    fn test_encode_decode_rate_limit_et_absence_de_retour() {
        let message = WrapperToDaemon::RateLimit {
            agent: "claude-1".to_string(),
            window: "five_hour".to_string(),
            status: "rejected".to_string(),
            resets_at: Some(1_787_572_200),
            used_percent: Some(100),
            source: RateLimitSource::ClaudeStreamJson,
        };
        let encoded = encode(&message).unwrap();
        assert!(encoded.contains("\"type\":\"RateLimit\""));
        assert!(encoded.contains("\"source\":\"claude-stream-json\""));
        assert!(encoded.contains("\"used_percent\":100"));
        assert!(matches!(
            decode(&encoded).unwrap(),
            WrapperToDaemon::RateLimit {
                agent,
                window,
                status,
                resets_at: Some(1_787_572_200),
                used_percent: Some(100),
                source: RateLimitSource::ClaudeStreamJson,
            } if agent == "claude-1" && window == "five_hour" && status == "rejected"
        ));

        let without_reset = r#"{"type":"RateLimit","agent":"claude-1","window":"five_hour","status":"allowed","source":"claude-stream-json"}"#;
        assert!(matches!(
            decode(without_reset).unwrap(),
            WrapperToDaemon::RateLimit {
                resets_at: None,
                used_percent: None,
                ..
            }
        ));
    }

    #[test]
    fn test_encode_decode_usage_et_fenetre_inconnue() {
        let sample = WrapperToDaemon::Usage {
            agent: "claude-1".to_string(),
            input_tokens: 2,
            execution_id: Some("execution-1".to_string()),
            execution_generation: Some(1),
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
            source: UsageSource::ClaudeStreamJson,
        };
        let encoded = encode(&sample).unwrap();
        assert!(encoded.contains("\"type\":\"Usage\""));
        assert!(encoded.contains("\"source\":\"claude-stream-json\""));
        assert!(matches!(
            decode(&encoded).unwrap(),
            WrapperToDaemon::Usage {
                input_tokens: 2,
                execution_id: Some(execution_id),
                execution_generation: Some(1),
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
                source: UsageSource::ClaudeStreamJson,
                ..
            } if execution_id == "execution-1"
        ));

        let window = WrapperToDaemon::UsageWindow {
            agent: "claude-1".to_string(),
            from_secs: 10,
            to_secs: 20,
        };
        let encoded_window = encode(&window).unwrap();
        assert!(encoded_window.contains("\"type\":\"UsageWindow\""));

        let unknown = r#"{"type":"UsageWindowResult","agent":"tmux-1"}"#;
        assert!(matches!(
            decode(unknown).unwrap(),
            DaemonToWrapper::UsageWindowResult {
                aggregate: None,
                ..
            }
        ));

        let attested = DaemonToWrapper::UsageWindowResult {
            agent: "claude-1".to_string(),
            aggregate: Some(UsageAggregate {
                turns: 1,
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
                facturable_tokens: 40_981,
            }),
        };
        let encoded_attested = encode(&attested).unwrap();
        assert!(encoded_attested.contains("\"facturable_tokens\":40981"));
        assert!(!encoded_attested.contains("\"facturable_tokens\":0"));
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
        assert!(info.channel.is_none());
        assert!(info.rate_limits.is_empty());
        assert!(info.model_mismatch.is_none());
        assert!(info.disk_space.is_none());
    }

    #[test]
    fn fait_espace_disque_post_enregistrement_garde_sa_valeur_attestee() {
        let message = WrapperToDaemon::DiskSpace {
            fact: DiskSpaceFact {
                volume: "/".to_string(),
                free_bytes: 47_300_000_000,
                observed_at_unix: 1_788_000_000,
            },
        };
        let encoded = encode(&message).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(&encoded).unwrap(),
            WrapperToDaemon::DiskSpace { fact }
                if fact.volume == "/"
                    && fact.free_bytes == 47_300_000_000
                    && fact.observed_at_unix == 1_788_000_000
        ));
    }

    #[test]
    fn spec_024_agent_info_expose_protocole_et_canal_independants() {
        let json = r#"{"name":"lab-agent","agent_type":"codex","connection_id":"conn-1",
            "host":"lab-host","transport":"tmux","channel":"ssh-unix","mode":"tmux",
            "os":"Linux","state":"connected","last_seen_secs":0,"reconnect_count":0}"#;
        let info: AgentInfo = decode(json).unwrap();
        assert_eq!(info.transport, "tmux");
        assert_eq!(info.channel.as_deref(), Some("ssh-unix"));

        let encoded = encode(&info).unwrap();
        assert!(encoded.contains(r#""transport":"tmux""#));
        assert!(encoded.contains(r#""channel":"ssh-unix""#));
    }

    #[test]
    fn fait_mono_fenetre_ancien_se_relit_en_vec_sans_inventer() {
        // Ancien fil : un seul champ `rate_limit`. Doit devenir Vec d'1 élément
        // avec la fenêtre attestée telle quelle — aucune fenêtre inventée.
        let json = r#"{
            "name":"claude-1","agent_type":"claude","connection_id":"c1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0,
            "rate_limit":{"window":"five_hour","status":"rejected","resets_at":1787572200}
        }"#;
        let info: AgentInfo = decode(json).unwrap();
        assert_eq!(info.rate_limits.len(), 1);
        assert_eq!(info.rate_limits[0].window, "five_hour");
        assert_eq!(info.rate_limits[0].status, "rejected");
        assert_eq!(info.rate_limits[0].resets_at, Some(1_787_572_200));
        assert_eq!(info.rate_limits[0].used_percent, None);

        // Nouveau fil : `rate_limits` gagne ; l'ancien champ s'il coexiste est ignoré.
        let both = r#"{
            "name":"claude-1","agent_type":"claude","connection_id":"c1",
            "host":"h","transport":"unix","os":"macOS","state":"connected",
            "last_seen_secs":0,"reconnect_count":0,
            "rate_limits":[{"window":"seven_day","status":"allowed","used_percent":61}],
            "rate_limit":{"window":"five_hour","status":"rejected"}
        }"#;
        let prefer_new: AgentInfo = decode(both).unwrap();
        assert_eq!(prefer_new.rate_limits.len(), 1);
        assert_eq!(prefer_new.rate_limits[0].window, "seven_day");

        // Sérialisation neuve : uniquement `rate_limits`, jamais `rate_limit`.
        let encoded = encode(&prefer_new).unwrap();
        assert!(encoded.contains("\"rate_limits\""));
        assert!(!encoded.contains("\"rate_limit\":"));
    }

    #[test]
    fn test_encode_decode_served_model() {
        let message = WrapperToDaemon::ServedModel {
            agent: "claude-1".to_string(),
            model: "claude-opus-4-6".to_string(),
        };
        let encoded = encode(&message).unwrap();
        assert!(encoded.contains("\"type\":\"ServedModel\""));
        assert!(matches!(
            decode(&encoded).unwrap(),
            WrapperToDaemon::ServedModel { agent, model }
                if agent == "claude-1" && model == "claude-opus-4-6"
        ));
    }

    #[test]
    fn model_mismatch_observe_exige_les_deux_etiquettes_distinctes() {
        assert!(ModelMismatchFact::observe(None, Some("claude-opus-4-6")).is_none());
        assert!(ModelMismatchFact::observe(Some("claude-opus-5"), None).is_none());
        assert!(ModelMismatchFact::observe(Some("claude-opus-5"), Some("claude-opus-5")).is_none());
        let gap = ModelMismatchFact::observe(Some("claude-opus-5"), Some("claude-opus-4-6"))
            .expect("écart attesté");
        assert_eq!(gap.pinned, "claude-opus-5");
        assert_eq!(gap.served, "claude-opus-4-6");
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

    #[test]
    fn attach_client_messages_roundtrip() {
        let handshake = WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        };
        assert!(matches!(
            decode(&encode(&handshake).unwrap()).unwrap(),
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            }
        ));

        let subscribe = WrapperToDaemon::Subscribe {
            agent: "codex-1".to_string(),
            window: AttachWindow::Seq(42),
        };
        assert!(matches!(
            decode(&encode(&subscribe).unwrap()).unwrap(),
            WrapperToDaemon::Subscribe {
                agent,
                window: AttachWindow::Seq(42)
            } if agent == "codex-1"
        ));

        let unsubscribe = WrapperToDaemon::Unsubscribe {
            subscription_id: "sub-1".to_string(),
        };
        assert!(matches!(
            decode(&encode(&unsubscribe).unwrap()).unwrap(),
            WrapperToDaemon::Unsubscribe { subscription_id } if subscription_id == "sub-1"
        ));
    }

    #[test]
    fn service_guichet_messages_roundtrip_et_restent_hors_attach() {
        let hello = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![ServiceCapability::MaicieGuichet],
        };
        assert_eq!(
            encode(&hello).unwrap(),
            SERVICE_NEGOTIATION_FIXTURE.lines().nth(2).unwrap()
        );
        assert!(matches!(
            decode(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                capabilities,
                ..
            } if capabilities == vec![ServiceCapability::MaicieGuichet]
        ));

        let reply = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-1".to_string(),
            claim_generation: 3,
            claim_token: "claim-1".to_string(),
            response_message_id: "msg-1".to_string(),
            in_reply_to: "message-1".to_string(),
            outcome: GuichetOutcome::Accepted,
            payload: GuichetReplyPayload::DeliveryReport {
                objective_id: "objective-1".to_string(),
                delegation_id: "delegation-1".to_string(),
                delivery_hash: "0".repeat(64),
                review_verdict: None,
            },
        };
        assert!(matches!(
            decode(&encode(&reply).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                claim_generation: 3,
                claim_token,
                ..
            } if claim_token == "claim-1"
        ));
        assert_eq!(
            reply.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );

        let refused = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-refused".to_string(),
            claim_generation: 1,
            claim_token: "claim-refused".to_string(),
            response_message_id: "msg-refused".to_string(),
            in_reply_to: "message-refused".to_string(),
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::MissionStatus,
                reason: GuichetRefusalReason::DelegationMissing,
            },
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&refused).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                outcome: GuichetOutcome::Refused,
                payload: GuichetReplyPayload::Refused {
                    operation: ServiceRequestOperation::MissionStatus,
                    reason: GuichetRefusalReason::DelegationMissing,
                },
                ..
            }
        ));

        let status = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-status".to_string(),
            claim_generation: 4,
            claim_token: "claim-2".to_string(),
            response_message_id: "message-2".to_string(),
            in_reply_to: "request-status".to_string(),
            outcome: GuichetOutcome::Accepted,
            payload: GuichetReplyPayload::MissionStatus {
                delegation_id: "delegation-1".to_string(),
                objective_id: "objective-1".to_string(),
                coordination_state: GuichetCoordinationState::EnCoordination,
                local_delivery: GuichetLocalDelivery {
                    state: GuichetLocalDeliveryState::Accepted,
                    issue: Some("accepted".to_string()),
                    observed_at: Some(1_787_500_001),
                },
                transport_observation: Some(GuichetTransportObservation {
                    state: GuichetTransportState::Connected,
                    request_state: Some("answered".to_string()),
                    observed_at: 1_787_500_002,
                    source: "attach".to_string(),
                    subscription_id: Some("subscription-1".to_string()),
                    seq: Some(7),
                }),
                freshness: GuichetFreshness::Fresh,
            },
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&status).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                payload: GuichetReplyPayload::MissionStatus {
                    freshness: GuichetFreshness::Fresh,
                    transport_observation: Some(GuichetTransportObservation { seq: Some(7), .. }),
                    ..
                },
                ..
            }
        ));

        let rejected = DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired,
        };
        assert!(matches!(
            decode(&encode(&rejected).unwrap()).unwrap(),
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            }
        ));
        assert!(!rejected.allowed_for_attach());

        let lifecycle = DaemonToWrapper::RequestLifecycleEvent {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            event_id: "evt-1".to_string(),
            request_id: "request-1".to_string(),
            state: GuichetLifecycleState::Answered,
            observed_at: 1_787_500_000,
            in_reply_to: Some("message-1".to_string()),
            response_message_id: Some("response-1".to_string()),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&lifecycle).unwrap()).unwrap(),
            DaemonToWrapper::RequestLifecycleEvent {
                state: GuichetLifecycleState::Answered,
                in_reply_to: Some(in_reply_to),
                response_message_id: Some(response_message_id),
                ..
            } if in_reply_to == "message-1" && response_message_id == "response-1"
        ));
        assert!(!lifecycle.allowed_for_attach());

        let coordination = DaemonToWrapper::CoordinationEvent {
            version: COORDINATION_EVENTS_VERSION,
            event_id: "evt-reminder-1".to_string(),
            request_id: "request-1".to_string(),
            kind: CoordinationEventKind::ReminderSent,
            reminder_message_id: "message-reminder-1".to_string(),
            recipient: "codex-1".to_string(),
            generation: 1,
            observed_at: 1_787_500_003,
            cursor: None,
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&coordination).unwrap()).unwrap(),
            DaemonToWrapper::CoordinationEvent {
                version: COORDINATION_EVENTS_VERSION,
                kind: CoordinationEventKind::ReminderSent,
                generation: 1,
                observed_at: 1_787_500_003,
                ..
            }
        ));
        assert!(!coordination.allowed_for_attach());

        let hello_with_coordination = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![
                ServiceCapability::MaicieGuichet,
                ServiceCapability::CoordinationEventsV1,
            ],
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&hello_with_coordination).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello { capabilities, .. }
                if capabilities == vec![
                    ServiceCapability::MaicieGuichet,
                    ServiceCapability::CoordinationEventsV1,
                ]
        ));
        assert!(decode::<DaemonToWrapper>(
            r#"{"type":"coordination_event","v":1,"event_id":"evt","request_id":"req","kind":"unknown_fact","reminder_message_id":"msg","recipient":"codex","generation":1,"observed_at":1}"#
        )
        .is_err());
    }

    #[test]
    fn mutations_du_greffe_et_resultat_terminal_font_un_roundtrip_ferme() {
        let mutations = [
            (
                ServiceRequestOperation::RegistreAdd,
                ServiceRequestPayload::RegistreAdd {
                    line: "kind=add id=constat-1".to_string(),
                },
                "registre_add",
            ),
            (
                ServiceRequestOperation::ObjectiveClose,
                ServiceRequestPayload::ObjectiveClose {
                    objective_id: "objective-1".to_string(),
                    reason: "objectif atteint".to_string(),
                },
                "objective_close",
            ),
        ];
        for (operation, payload, wire_name) in mutations {
            let request = WrapperToDaemon::ServiceRequest {
                version: SERVICE_CONTRACT_VERSION,
                issuer_scope: "026_scope_0123456789abcdef0123456789abcdef".to_string(),
                request_id: format!("request-{wire_name}"),
                issued_at: 1_787_824_000,
                from: "jc2".to_string(),
                to: "maicie".to_string(),
                operation,
                payload: payload.clone(),
            };
            let wire = encode(&request).unwrap();
            assert!(wire.contains(&format!(r#""operation":"{wire_name}""#)));
            assert!(matches!(
                decode::<WrapperToDaemon>(&wire).unwrap(),
                WrapperToDaemon::ServiceRequest {
                    operation: decoded_operation,
                    payload: decoded_payload,
                    ..
                } if decoded_operation == operation && decoded_payload == payload
            ));
        }

        let result = DaemonToWrapper::GuichetResult {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "026_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "request-close".to_string(),
            issue: "accepted".to_string(),
            expires_at: 1_787_824_060,
            payload: Some(GuichetReplyPayload::ObjectiveClose {
                objective_id: "objective-1".to_string(),
                decision_id: "decision-1".to_string(),
                replayed: false,
            }),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&result).unwrap()).unwrap(),
            DaemonToWrapper::GuichetResult {
                issue,
                payload: Some(GuichetReplyPayload::ObjectiveClose {
                    objective_id,
                    decision_id,
                    replayed: false,
                }),
                ..
            } if issue == "accepted"
                && objective_id == "objective-1"
                && decision_id == "decision-1"
        ));

        let historical: DaemonToWrapper = decode(
            r#"{"type":"guichet_result","v":1,"issuer_scope":"026_scope_0123456789abcdef0123456789abcdef","request_id":"request-old","issue":"queued","expires_at":1787824060}"#,
        )
        .unwrap();
        assert!(matches!(
            historical,
            DaemonToWrapper::GuichetResult { payload: None, .. }
        ));
        assert!(matches!(
            decode::<DaemonToWrapper>(
                r#"{"type":"ServiceRejected","reason":{"kind":"greffe_authorization_denied"}}"#
            )
            .unwrap(),
            DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::GreffeAuthorizationDenied
            }
        ));
    }

    #[test]
    fn motif_compose_de_revue_fait_un_roundtrip_filaire_exact() {
        let reply = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-refused-compound".to_string(),
            claim_generation: 2,
            claim_token: "claim-refused-compound".to_string(),
            response_message_id: "msg-refused-compound".to_string(),
            in_reply_to: "message-refused-compound".to_string(),
            outcome: GuichetOutcome::Refused,
            payload: GuichetReplyPayload::Refused {
                operation: ServiceRequestOperation::DeliveryReport,
                reason: GuichetRefusalReason::TargetHeadMovedAndMeasuredHeadMismatch,
            },
        };

        let wire = encode(&reply).unwrap();
        assert_eq!(
            wire,
            r#"{"type":"guichet_reply","v":1,"issuer_scope":"015_scope_0123456789abcdef0123456789abcdef","request_id":"req-refused-compound","claim_generation":2,"claim_token":"claim-refused-compound","response_message_id":"msg-refused-compound","in_reply_to":"message-refused-compound","outcome":"refused","payload":{"kind":"refused","operation":"delivery_report","reason":"target_head_moved_and_measured_head_mismatch"}}"#
        );
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::GuichetReply {
                outcome: GuichetOutcome::Refused,
                payload: GuichetReplyPayload::Refused {
                    operation: ServiceRequestOperation::DeliveryReport,
                    reason: GuichetRefusalReason::TargetHeadMovedAndMeasuredHeadMismatch,
                },
                ..
            }
        ));
    }

    #[test]
    fn client_idempotency_messages_roundtrip_and_stay_outside_attach() {
        let hello = WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
            capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
        };
        assert!(matches!(
            decode(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope,
                capabilities,
            } if issuer_scope == "012_scope_aaaaaaaaaaaa"
                && capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
        ));
        assert_eq!(
            hello.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
        let welcome = DaemonToWrapper::ClientWelcome {
            version: CLIENT_CONTRACT_VERSION,
            build_id: "fixture-build".to_string(),
            horizon_secs: 60,
            issued_at_tolerance_secs: 5,
            capabilities: vec![ClientCapability::Lookup],
        };
        let decoded: DaemonToWrapper = decode(&encode(&welcome).unwrap()).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                build_id,
                capabilities,
                ..
            } if build_id == "fixture-build" && capabilities == vec![ClientCapability::Lookup]
        ));
        let result = DaemonToWrapper::IdempotencyResult {
            operation_kind: "send".to_string(),
            idempotency_key: "message-1".to_string(),
            issue: IdempotencyIssue::OutcomeUnknown {
                expires_at: 123,
                delivery_id: Some("delivery-1".to_string()),
            },
        };
        assert!(matches!(
            decode(&encode(&result).unwrap()).unwrap(),
            DaemonToWrapper::IdempotencyResult {
                operation_kind,
                idempotency_key,
                issue: IdempotencyIssue::OutcomeUnknown {
                    expires_at: 123,
                    delivery_id: Some(delivery_id),
                },
            } if operation_kind == "send" && idempotency_key == "message-1" && delivery_id == "delivery-1"
        ));
        assert!(!welcome.allowed_for_attach());
    }

    /// La machine, la base et l'instance du daemon voyagent par un message
    /// DÉDIÉ. Les trois valeurs doivent survivre au tour du fil : l'instance
    /// reste distincte du build et du chemin local.
    ///
    /// Mutant qui tue ce test : renvoyer `db_path` à la place de
    /// `instance_id` → la dernière assertion meurt.
    #[test]
    fn le_daemon_atteste_sa_machine_sa_base_et_son_instance() {
        let report = DaemonToWrapper::DaemonIdentityReport {
            host: "monordinateur".to_string(),
            db_path: "/Users/moi/.cache/bridget/bridget.db".to_string(),
            instance_id: "daemon-7f0c01d2".to_string(),
        };
        match decode::<DaemonToWrapper>(&encode(&report).unwrap()).unwrap() {
            DaemonToWrapper::DaemonIdentityReport {
                host,
                db_path,
                instance_id,
            } => {
                assert_eq!(host, "monordinateur");
                assert_eq!(db_path, "/Users/moi/.cache/bridget/bridget.db");
                assert_eq!(instance_id, "daemon-7f0c01d2");
            }
            other => panic!("variante inattendue: {other:?}"),
        }
    }

    /// Les deux hôtes du refus survivent au tour du fil ET restent distincts :
    /// un oracle qui vérifierait seulement leur présence passerait aussi si le
    /// producteur écrivait deux fois la même machine.
    ///
    /// Mutant qui tue ce test : sérialiser `requested_from` à partir de
    /// `searched_on` → l'assertion d'inégalité meurt.
    #[test]
    fn cwd_gone_transporte_les_deux_machines_et_les_distingue() {
        let refusal = SpawnRefusal::CwdGone {
            searched_on: "monordinateur".to_string(),
            requested_from: "cartae".to_string(),
        };
        let json = serde_json::to_string(&refusal).expect("sérialisable");
        let decoded: SpawnRefusal = serde_json::from_str(&json).expect("décodable");
        match decoded {
            SpawnRefusal::CwdGone {
                searched_on,
                requested_from,
            } => {
                assert_eq!(searched_on, "monordinateur");
                assert_eq!(requested_from, "cartae");
                assert_ne!(searched_on, requested_from);
            }
            other => panic!("variante inattendue: {other:?}"),
        }

        // Refus produit par un daemon antérieur : champs vides, jamais devinés.
        let ancien: SpawnRefusal =
            serde_json::from_str(r#"{"kind":"cwd_gone"}"#).expect("refus historique décodable");
        assert!(matches!(
            ancien,
            SpawnRefusal::CwdGone {
                searched_on,
                requested_from,
            } if searched_on.is_empty() && requested_from.is_empty()
        ));
    }

    #[test]
    fn client_welcome_historique_signale_un_build_id_inconnu() {
        let decoded: DaemonToWrapper = decode(
            r#"{"type":"ClientWelcome","version":1,"horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":[]}"#,
        )
        .unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::ClientWelcome { build_id, .. } if build_id == "unknown"
        ));
    }

    #[test]
    fn lifecycle_messages_roundtrip_and_stay_outside_attach() {
        let spawn = WrapperToDaemon::SpawnOrder {
            agent_type: "codex".to_string(),
            project: Some(ProjectReference {
                project_id: "project-1".to_string(),
                binding_generation: 2,
            }),
            ownership: Some(SpawnOwnership {
                parent_instance_id: "instance-parent".to_string(),
                parent_execution_id: Some("execution-parent".to_string()),
                objective_id: Some("objective-1".to_string()),
                delegation_id: Some("delegation-1".to_string()),
                project: Some(ProjectReference {
                    project_id: "project-1".to_string(),
                    binding_generation: 2,
                }),
                role: "verification".to_string(),
                max_children: Some(3),
                max_depth: Some(2),
            }),
            name: Some("codex-1".to_string()),
            cwd: "/tmp".to_string(),
            persistent: true,
            command_id: "command-1".to_string(),
            issued_at: 100,
            deadline_at: 110,
        };
        assert!(matches!(
            decode(&encode(&spawn).unwrap()).unwrap(),
            WrapperToDaemon::SpawnOrder {
                agent_type,
                name: Some(name),
                command_id,
                ownership: Some(ownership),
                ..
            } if agent_type == "codex"
                && name == "codex-1"
                && command_id == "command-1"
                && ownership.parent_instance_id == "instance-parent"
        ));
        assert_eq!(
            spawn.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
        let wait = WrapperToDaemon::WaitAgentLinks {
            after_cursor: Some(41),
            timeout_ms: 3_000,
        };
        assert!(matches!(
            decode(&encode(&wait).unwrap()).unwrap(),
            WrapperToDaemon::WaitAgentLinks {
                after_cursor: Some(41),
                timeout_ms: 3_000,
            }
        ));
        assert_eq!(
            wait.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
        let events = DaemonToWrapper::AgentLinkEvents {
            events: vec![AgentLinkEventFrame {
                cursor: 42,
                event_id: "link-1:2".to_string(),
                link_id: "link-1".to_string(),
                child_instance_id: "child-1".to_string(),
                state: "orphaned".to_string(),
                observed_at: 1_788_000_000,
                project: Some(ProjectReference {
                    project_id: "project-1".to_string(),
                    binding_generation: 2,
                }),
            }],
            through_cursor: Some(42),
            timed_out: false,
        };
        assert!(matches!(
            decode(&encode(&events).unwrap()).unwrap(),
            DaemonToWrapper::AgentLinkEvents {
                through_cursor: Some(42),
                timed_out: false,
                events,
            } if events.len() == 1 && events[0].state == "orphaned"
        ));
        let rejection = DaemonToWrapper::SpawnRejected {
            command_id: "command-1".to_string(),
            reason: SpawnRefusal::BillingGuard {
                variable: "OPENAI_API_KEY".to_string(),
            },
        };
        assert!(matches!(
            decode(&encode(&rejection).unwrap()).unwrap(),
            DaemonToWrapper::SpawnRejected {
                reason: SpawnRefusal::BillingGuard { variable },
                ..
            } if variable == "OPENAI_API_KEY"
        ));
        assert!(!rejection.allowed_for_attach());
        let stop = DaemonToWrapper::StopResult {
            command_id: "stop-1".to_string(),
            outcome: StopOutcome::StoppedForced {
                survivors_killed: 2,
            },
        };
        assert!(matches!(
            decode(&encode(&stop).unwrap()).unwrap(),
            DaemonToWrapper::StopResult {
                outcome: StopOutcome::StoppedForced {
                    survivors_killed: 2
                },
                ..
            }
        ));
        let relaunch = DaemonToWrapper::RelaunchResult {
            command_id: "relaunch-1".to_string(),
            outcome: RelaunchOutcome::Started {
                name: "codex-1".to_string(),
                generation: 12,
            },
        };
        assert!(matches!(
            decode(&encode(&relaunch).unwrap()).unwrap(),
            DaemonToWrapper::RelaunchResult {
                outcome: RelaunchOutcome::Started { generation: 12, .. },
                ..
            }
        ));
        let decommission = DaemonToWrapper::DecommissionResult {
            command_id: "decommission-1".to_string(),
            outcome: DecommissionOutcome::DecommissionedForced {
                survivors_killed: 3,
            },
        };
        assert!(matches!(
            decode(&encode(&decommission).unwrap()).unwrap(),
            DaemonToWrapper::DecommissionResult {
                outcome: DecommissionOutcome::DecommissionedForced {
                    survivors_killed: 3
                },
                ..
            }
        ));
        let adoption = DaemonToWrapper::AdoptStoppedResult {
            command_id: "adopt-1".to_string(),
            outcome: AdoptStoppedOutcome::Adopted { generation: 7 },
        };
        assert!(matches!(
            decode(&encode(&adoption).unwrap()).unwrap(),
            DaemonToWrapper::AdoptStoppedResult {
                outcome: AdoptStoppedOutcome::Adopted { generation: 7 },
                ..
            }
        ));
    }

    #[test]
    fn attach_relay_messages_roundtrip() {
        let messages = vec![
            WrapperToDaemon::Subscribed {
                subscription_id: "sub-1".to_string(),
            },
            WrapperToDaemon::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 7,
                offset: 0,
                final_fragment: true,
                bytes: b"{\"v\":1}\n".to_vec(),
            },
            WrapperToDaemon::LiveJournalFragment {
                seq: 8,
                offset: 0,
                final_fragment: true,
                bytes: b"{\"v\":1,\"seq\":8}".to_vec(),
            },
            WrapperToDaemon::SnapshotCaughtUp {
                subscription_id: "sub-1".to_string(),
                through_seq: Some(7),
            },
            WrapperToDaemon::Gap {
                subscription_id: "sub-1".to_string(),
                from_seq: 3,
                to_seq: 4,
                reason: Some("vue lente".to_string()),
            },
            WrapperToDaemon::End {
                subscription_id: "sub-1".to_string(),
                reason: "wrapper arrêté".to_string(),
            },
            WrapperToDaemon::AttachRejected {
                subscription_id: Some("sub-1".to_string()),
                reason: AttachRefusal::CommandQueueSaturated,
            },
            WrapperToDaemon::JournalReadError {
                subscription_id: "sub-1".to_string(),
                line: 3,
                offset: 42,
                reason: "ligne illisible".to_string(),
            },
        ];
        for message in messages {
            assert_eq!(
                encode(&message).unwrap(),
                encode(&decode::<WrapperToDaemon>(&encode(&message).unwrap()).unwrap()).unwrap()
            );
        }
        assert_eq!(MAX_ATTACH_FRAGMENT_BYTES, 190 * 1024);
        assert_eq!(MAX_ATTACH_SERIALIZED_FRAME_BYTES, 256 * 1024);
    }

    #[test]
    fn attach_daemon_messages_roundtrip() {
        let messages = vec![
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach,
            },
            DaemonToWrapper::Subscribe {
                subscription_id: "sub-1".to_string(),
                agent: "codex-1".to_string(),
                window: AttachWindow::Today,
            },
            DaemonToWrapper::Unsubscribe {
                subscription_id: "sub-1".to_string(),
            },
            DaemonToWrapper::Subscribed {
                subscription_id: "sub-1".to_string(),
            },
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 8,
                offset: 0,
                final_fragment: true,
                bytes: vec![0, 1, 2],
            },
            DaemonToWrapper::SnapshotCaughtUp {
                subscription_id: "sub-1".to_string(),
                through_seq: None,
            },
            DaemonToWrapper::Gap {
                subscription_id: "sub-1".to_string(),
                from_seq: 6,
                to_seq: 7,
                reason: None,
            },
            DaemonToWrapper::End {
                subscription_id: "sub-1".to_string(),
                reason: "désabonné".to_string(),
            },
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentStopped,
                mode: None,
                location: None,
            },
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentNotAcp,
                mode: Some(PresenceMode::Tmux),
                location: Some("bridget:4.2".to_string()),
            },
            DaemonToWrapper::JournalReadError {
                subscription_id: "sub-1".to_string(),
                line: 3,
                offset: 42,
                reason: "ligne illisible".to_string(),
            },
            DaemonToWrapper::DeliveryRejected {
                id: "message-1".to_string(),
                reason: "processus arrêté".to_string(),
            },
        ];
        for message in messages {
            let reaches_attach = !matches!(
                message,
                DaemonToWrapper::Subscribe { .. } | DaemonToWrapper::Unsubscribe { .. }
            );
            let json = encode(&message).unwrap();
            let decoded: DaemonToWrapper = decode(&json).unwrap();
            assert_eq!(json, encode(&decoded).unwrap());
            assert_eq!(decoded.allowed_for_attach(), reaches_attach);
        }
    }

    #[test]
    fn attach_rejected_historique_omet_les_details_de_presence() {
        let legacy = r#"{"type":"AttachRejected","reason":"agent_not_acp"}"#;
        let decoded: DaemonToWrapper = decode(legacy).unwrap();
        match decoded {
            DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::AgentNotAcp,
                mode: None,
                location: None,
            } => {}
            other => panic!("message historique inattendu : {other:?}"),
        }
    }

    #[test]
    fn journal_ready_est_un_signal_wrapper_hors_du_role_attach() {
        let signal = WrapperToDaemon::JournalReady;
        let json = encode(&signal).unwrap();
        assert_eq!(json, r#"{"type":"JournalReady"}"#);
        assert!(matches!(
            decode(&json).unwrap(),
            WrapperToDaemon::JournalReady
        ));
        assert_eq!(
            signal.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );
    }

    #[test]
    fn spawn_accepted_transporte_la_definition_resolue_complete() {
        let message = DaemonToWrapper::SpawnAccepted {
            command_id: "command-1".to_string(),
            name: "reviewer".to_string(),
            definition: Some(ResolvedAgentDefinition {
                command: "npx".to_string(),
                args: vec!["adapter@1.2.3".to_string()],
                protocol: "acp".to_string(),
                forbidden_env: vec!["API_KEY".to_string()],
                pass_env: vec!["HOME".to_string()],
                claude_config_dir: None,
                permissions: "allow".to_string(),
                queue_capacity: 32,
                notify_timeout_secs: 600,
                mcp: ResolvedMcpDefinition {
                    interactive: "codex".to_string(),
                    acp_session: true,
                },
                capabilities: AdapterCapabilities {
                    execution_paths: vec!["acp".to_string()],
                    models: BTreeMap::from([(
                        "gpt-5.6-terra".to_string(),
                        ModelCapabilities {
                            efforts: vec!["high".to_string()],
                        },
                    )]),
                    observed: None,
                },
                digest: "a".repeat(64),
            }),
        };
        let json = encode(&message).unwrap();
        let decoded = decode::<DaemonToWrapper>(&json).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::SpawnAccepted { definition: Some(definition), .. }
                if definition.command == "npx"
                    && definition.args == ["adapter@1.2.3"]
                    && definition.forbidden_env == ["API_KEY"]
                    && definition.capabilities.models["gpt-5.6-terra"].efforts == ["high"]
                    && definition.digest == "a".repeat(64)
        ));
        assert!(matches!(
            decode::<DaemonToWrapper>(
                r#"{"type":"SpawnAccepted","command_id":"legacy","name":"ancien"}"#
            )
            .unwrap(),
            DaemonToWrapper::SpawnAccepted {
                definition: None,
                ..
            }
        ));
    }

    #[test]
    fn fragment_base64_reste_sous_la_borne_filaire_pour_des_octets_hostiles() {
        for bytes in [
            vec![0; MAX_ATTACH_FRAGMENT_BYTES],
            vec![0xff; MAX_ATTACH_FRAGMENT_BYTES],
            "équipier".as_bytes().repeat(MAX_ATTACH_FRAGMENT_BYTES / 9),
        ] {
            let message = WrapperToDaemon::JournalFragment {
                subscription_id: "attach-0123456789abcdef".to_string(),
                seq: 7,
                offset: 0,
                final_fragment: true,
                bytes,
            };
            assert!(encode(&message).unwrap().len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES);
        }
    }

    #[test]
    fn role_attach_refuse_les_messages_wrapper_et_reply_suivi() {
        let wrapper_only = WrapperToDaemon::Runtime {
            agent: "codex-1".to_string(),
            model: "gpt-5.5".to_string(),
            effort: None,
            source: RuntimeSource::Declared,
        };
        assert_eq!(
            wrapper_only.attach_refusal(),
            Some(AttachRefusal::MessageOutsideAttachRole)
        );

        let mut tracked_send = BridgetMessage::new("forge", "codex-1", "réponds");
        tracked_send.reply = true;
        assert_eq!(
            WrapperToDaemon::Send(tracked_send).attach_refusal(),
            Some(AttachRefusal::ReplyNotAllowed)
        );
        assert!(
            WrapperToDaemon::Send(BridgetMessage::new("humain", "codex-1", "bonjour"))
                .attach_refusal()
                .is_none()
        );
        assert!(!DaemonToWrapper::Deliver(BridgetMessage::new("a", "b", "x")).allowed_for_attach());
    }

    #[test]
    fn issue_differee_attach_reste_correlee_parmi_le_flux() {
        let flux = vec![
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 11,
                offset: 0,
                final_fragment: true,
                bytes: b"premier".to_vec(),
            },
            DaemonToWrapper::DeliveryRejected {
                id: "message-humain".to_string(),
                reason: "file pleine".to_string(),
            },
            DaemonToWrapper::JournalFragment {
                subscription_id: "sub-1".to_string(),
                seq: 12,
                offset: 0,
                final_fragment: true,
                bytes: b"second".to_vec(),
            },
        ];
        let decoded = flux
            .into_iter()
            .map(|message| decode::<DaemonToWrapper>(&encode(&message).unwrap()).unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            decoded[0],
            DaemonToWrapper::JournalFragment { seq: 11, .. }
        ));
        assert!(
            matches!(decoded[1], DaemonToWrapper::DeliveryRejected { ref id, .. } if id == "message-humain")
        );
        assert!(matches!(
            decoded[2],
            DaemonToWrapper::JournalFragment { seq: 12, .. }
        ));
    }

    #[test]
    fn protocol_007_reste_compatible_sans_handshake() {
        let json =
            r#"{"type":"Register","agent_type":"codex","name":null,"turn_in_progress":false}"#;
        assert!(matches!(
            decode::<WrapperToDaemon>(json).unwrap(),
            WrapperToDaemon::Register { agent_type, .. } if agent_type == "codex"
        ));
    }

    #[test]
    fn refus_type_inconnu_historique_reste_lisible_apres_l_ajout_du_diagnostic() {
        let legacy: SpawnRefusal = serde_json::from_str(r#"{"kind":"unknown_type"}"#).unwrap();
        assert!(matches!(
            legacy,
            SpawnRefusal::UnknownType {
                requested_type,
                known_types,
                registry,
            } if requested_type.is_empty() && known_types.is_empty() && registry.is_empty()
        ));
    }

    #[test]
    fn test_encode_decode_bounded_ledger_projection() {
        let request = WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Both,
            limit: 20,
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&request).unwrap()).unwrap(),
            WrapperToDaemon::LedgerProjection {
                scope: LedgerScope::Both,
                limit: 20
            }
        ));

        let response = DaemonToWrapper::LedgerProjection {
            messages: vec![LedgerMessage {
                id: "m-1".to_string(),
                ts: 42,
                sender: "alice".to_string(),
                target: "bob".to_string(),
                body: "bonjour".to_string(),
                delivery_status: Some(LedgerDeliveryStatus::EnVol),
            }],
            requests: Vec::new(),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&response).unwrap()).unwrap(),
            DaemonToWrapper::LedgerProjection { messages, requests }
                if messages.len() == 1
                    && messages[0].id == "m-1"
                    && messages[0].delivery_status == Some(LedgerDeliveryStatus::EnVol)
                    && requests.is_empty()
        ));
    }

    /// Oracle : les phases nommées ont une correspondance. Meurt si l'on
    /// retire un arm (état SQL sans rendu lisible au ledger).
    #[test]
    fn from_phase_garde_indetermine_parmi_les_trois_etats() {
        assert_eq!(
            LedgerDeliveryStatus::from_phase("dispatching"),
            Some(LedgerDeliveryStatus::EnVol)
        );
        assert_eq!(
            LedgerDeliveryStatus::from_phase("acked"),
            Some(LedgerDeliveryStatus::Recu)
        );
        assert_eq!(
            LedgerDeliveryStatus::from_phase("indeterminate"),
            Some(LedgerDeliveryStatus::Indetermine),
            "retirer cette correspondance laisse la quarantaine muette au ledger"
        );
        assert_eq!(
            LedgerDeliveryStatus::from_phase("orphaned"),
            Some(LedgerDeliveryStatus::Orphelin),
            "orphelin doit se rendre au ledger, distinct de indéterminé"
        );
        assert_eq!(LedgerDeliveryStatus::Indetermine.label_fr(), "indéterminé");
        assert_eq!(LedgerDeliveryStatus::Orphelin.label_fr(), "orphelin");
        assert_eq!(LedgerDeliveryStatus::from_phase("autre"), None);
    }
    #[test]
    fn execution_control_contract_roundtrip_et_version_explicit() {
        let mut message = BridgetMessage::new("humain", "agent-fixture", "corrige ce point");
        message.id = "message-steer-fixture".to_string();
        message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        let command = ExecutionControlCommand {
            version: 1,
            command_id: "control-fixture".to_string(),
            execution_id: "execution-fixture".to_string(),
            generation: 7,
            revision: 3,
            operation: ExecutionControlOperation::SteerCurrent,
            message: Some(message),
        };
        let wire = encode(&command).unwrap();
        let decoded: ExecutionControlCommand = decode(&wire).unwrap();
        assert_eq!(decoded, command);
        assert!(wire.contains("\"version\":1"));
    }

    #[test]
    fn execution_control_refusal_reste_ferme_sur_le_fil() {
        let refusal = ExecutionControlRefusal::CapabilityUnavailable;
        assert_eq!(
            serde_json::to_string(&refusal).unwrap(),
            "\"capability_unavailable\""
        );
        assert!(serde_json::from_str::<ExecutionControlRefusal>("\"unknown\"").is_err());
    }
    #[test]
    fn spec_068_trames_incident_deleguees_sont_fermees_et_redacted() {
        let emitted = WrapperToDaemon::DelegatedRuntimeEvent {
            execution_id: "execution-child-42".to_string(),
            kind: DelegatedRuntimeEventKind::Warning,
            code: "unsupported_provider_request".to_string(),
            reference: "sha256:ab12".to_string(),
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&emitted).unwrap()).unwrap(),
            WrapperToDaemon::DelegatedRuntimeEvent {
                execution_id,
                kind: DelegatedRuntimeEventKind::Warning,
                code,
                reference,
            } if execution_id == "execution-child-42"
                && code == "unsupported_provider_request"
                && reference == "sha256:ab12"
        ));

        let delivery = DaemonToWrapper::DelegatedRuntimeEvent {
            event: DelegatedRuntimeEventFrame {
                cursor: 7,
                event_id: "runtime-link-1-execution-child-42-warning".to_string(),
                link_id: "link-1".to_string(),
                child_instance_id: "instance-child".to_string(),
                child_execution_id: "execution-child-42".to_string(),
                kind: DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".to_string(),
                reference: "sha256:ab12".to_string(),
                observed_at: 1_788_000_000,
                project: Some(ProjectReference {
                    project_id: "project-1".to_string(),
                    binding_generation: 2,
                }),
            },
        };
        let wire = encode(&delivery).unwrap();
        assert!(matches!(
            decode::<DaemonToWrapper>(&wire).unwrap(),
            DaemonToWrapper::DelegatedRuntimeEvent { event }
                if event.cursor == 7
                    && event.kind == DelegatedRuntimeEventKind::Warning
                    && event.code == "unsupported_provider_request"
                    && event.reference == "sha256:ab12"
        ));
        assert!(!wire.contains("params"));
        assert!(!wire.contains("secret"));

        let acknowledgement = WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
            event_id: "runtime-link-1-execution-child-42-warning".to_string(),
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&acknowledgement).unwrap()).unwrap(),
            WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id }
                if event_id == "runtime-link-1-execution-child-42-warning"
        ));
        assert!(serde_json::from_str::<DelegatedRuntimeEventKind>("\"other\"").is_err());
    }

    #[test]
    fn spec_065_registre_projet_est_ferme_directionnel_et_negocie() {
        let request = ProjectBindRequest {
            contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
            command_id: "project-command-1".to_string(),
            issued_at: 1_787_997_600,
            deadline_at: 1_787_998_200,
            project_id: "project-opaque-1".to_string(),
            requested_root: "/srv/projects/fixture".to_string(),
            backend: ProjectBackend::Host,
        };
        let message = WrapperToDaemon::ProjectRegistryRequest {
            request: request.clone(),
        };
        let wire = encode(&message).unwrap();
        assert!(wire.contains(r#""type":"project_registry_request""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ProjectRegistryRequest { request: decoded }
                if decoded == request
        ));
        assert!(serde_json::from_str::<ProjectBindRequest>(
            r#"{"contract_version":1,"command_id":"project-command-1","issued_at":1,"deadline_at":2,"project_id":"project-opaque-1","requested_root":"/srv/projects/fixture","backend":"host","unexpected":true}"#
        )
        .is_err());

        let hello = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "065_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![ServiceCapability::ProjectRegistryV1],
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello {
                service,
                capabilities,
                ..
            } if service == "maicie" && capabilities == vec![ServiceCapability::ProjectRegistryV1]
        ));

        let conflict = ProjectBindOutcome {
            contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
            command_id: "project-command-2".to_string(),
            project_id: "project-loser".to_string(),
            status: ProjectBindStatus::RegistrationConflict,
            binding_generation: None,
            backend: None,
            reason: Some(ProjectRegistryRefusal::RootAlreadyBound),
            existing_project_id: Some("project-winner".to_string()),
            existing_binding_generation: Some(4),
            observed_at: 1_787_997_601,
        };
        let outcome = DaemonToWrapper::ProjectRegistryOutcome {
            outcome: conflict.clone(),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&outcome).unwrap()).unwrap(),
            DaemonToWrapper::ProjectRegistryOutcome { outcome: decoded }
                if decoded == conflict
        ));

        let admin_request = ProjectAdminRequest {
            contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
            command_id: "project-rebind-1".to_string(),
            issued_at: 1_787_997_602,
            deadline_at: 1_787_998_202,
            operation: ProjectAdminOperation::Rebind,
            project_id: Some("project-winner".to_string()),
            requested_root: Some("/srv/projects/other".to_string()),
        };
        let admin_wire = encode(&WrapperToDaemon::ProjectRegistryAdminRequest {
            request: admin_request.clone(),
        })
        .unwrap();
        assert!(admin_wire.contains(r#""type":"project_registry_admin_request""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&admin_wire).unwrap(),
            WrapperToDaemon::ProjectRegistryAdminRequest { request }
                if request == admin_request
        ));
        let admin_outcome = ProjectAdminOutcome {
            contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
            command_id: admin_request.command_id.clone(),
            operation: ProjectAdminOperation::Rebind,
            bindings: vec![ProjectBindingProjection {
                project_id: "project-winner".to_string(),
                state: ProjectBindingStatus::Active,
                binding_generation: Some(2),
                backend: Some(ProjectBackend::Host),
                reason: None,
                observed_at: 1_787_997_603,
            }],
            reason: None,
            observed_at: 1_787_997_603,
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(
                &encode(&DaemonToWrapper::ProjectRegistryAdminOutcome {
                    outcome: admin_outcome.clone(),
                })
                .unwrap()
            )
            .unwrap(),
            DaemonToWrapper::ProjectRegistryAdminOutcome { outcome }
                if outcome == admin_outcome
        ));

        let version_refusal = ProjectRegistryRefusal::ProjectRegistryVersionUnsupported;
        assert_eq!(
            serde_json::to_string(&version_refusal).unwrap(),
            "\"project_registry_version_unsupported\""
        );
        assert_eq!(
            serde_json::to_string(&ProjectRegistryRefusal::PeerUidMismatch).unwrap(),
            "\"peer_uid_mismatch\""
        );
        assert!(serde_json::from_str::<ProjectRegistryRefusal>("\"unknown\"").is_err());
    }
}
