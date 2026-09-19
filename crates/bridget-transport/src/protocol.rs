//! Définitions des messages JSON du protocole daemon/wrapper.
//!
//! Deux directions :
//! - WrapperToDaemon : ce que le wrapper envoie au daemon
//! - DaemonToWrapper : ce que le daemon envoie au wrapper

use bridget_core::BridgetMessage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[path = "project_profile_protocol.rs"]
pub mod project_profile;
pub use project_profile::{
    PROJECT_PROFILE_CONTRACT_VERSION, ProjectProfileAgent, ProjectProfileOutcome,
    ProjectProfileProposal, ProjectProfileRefusal, ProjectProfileRequest, ProjectResourceKind,
    ProjectResourceRef, ProjectRuntimeView, ResolvedProjectAgent, ResolvedProjectProfile,
    ResolvedProjectResource,
};
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

/// Changement de présentation uniquement : l'identité vient de la connexion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayNameRequest {
    pub version: u8,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayNameRefusal {
    InvalidRequest,
    IdentityUnavailable,
    NameConflict,
    RevisionConflict,
    StorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum DisplayNameOutcome {
    Applied {
        agent_id: String,
        display_name: String,
        revision: u64,
    },
    Rejected {
        reason: DisplayNameRefusal,
    },
}

/// Lecture durable du nom humain ; ne réserve ni ne prend une identité.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum DisplayNameResolution {
    Found { agent_id: String, active: bool },
    NotFound,
    Rejected { reason: DisplayNameRefusal },
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
/// Version du contrat de service du guichet le service compagnon.
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
/// Version du contrat d'état de contrôle du référent (SPEC-087) : pause de
/// l'autonomie et plafond d'objectifs auto-générés.
pub const CONTROL_STATE_CONTRACT_VERSION: u16 = 1;
/// Version du contrat de boîte de réception humaine (SPEC-087).
pub const HUMAN_INBOX_CONTRACT_VERSION: u16 = 1;

/// Capacité optionnelle du client idempotent. L'énumération fermée évite une
/// dégradation silencieuse lorsqu'un client demande une capacité inconnue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientCapability {
    SendIdempotent,
    Lookup,
    ExecutionControlV1,
    ProjectRoundPolicyV1,
    /// Lecture et mutation de l'état de contrôle du référent, lecture et
    /// résolution de la boîte de réception. La façade MCP ne la demande jamais.
    ControlStateV1,
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

/// Corrélation Bridget d'une remise aval idempotente.
///
/// Le champ reste optionnel dans `DeliverIdempotent` afin que les remises
/// historiques sans plan d'exécution conservent exactement leur comportement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionDeliveryContext {
    pub execution_id: String,
    pub generation: u64,
    pub revision: u64,
}

/// Capacité explicitement négociée par un service local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceCapability {
    GuichetV1,
    /// Registre local Bridget, exclusivement négocié par le service compagnon.
    ProjectRegistryV1,
    ProjectProfilesV1,
    CoordinationEventsV1,
    /// Relève bornée et cursée des faits 016. La v1 reste disponible pour les
    /// consommateurs qui n'ont besoin que du rejeu initial historique.
    CoordinationEventsV2,
    /// Dépôt d'items dans la boîte de réception humaine, relève et
    /// acquittement des décisions du référent (SPEC-087).
    HumanInboxV1,
}

/// Backend d'exécution admis par le registre de projets.
///
/// La v1 n'accepte qu'une racine validée directement sur l'hôte du daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectBackend {
    Host,
    Docker,
}

/// Opération locale sur un emplacement de projet attesté par le daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectPlacementOperation {
    Create,
    Import,
}

/// Commande versionnée de prévisualisation ou d'application d'emplacement.
/// Elle ne contient jamais un parent de création libre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPlacementRequestV2 {
    pub contract_version: u16,
    pub command_id: String,
    pub expected_catalog_generation: u64,
    pub location_id: String,
    pub operation: ProjectPlacementOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_root: Option<String>,
}

/// Politique hôte effectivement attestée pour une liaison Docker.
///
/// Les champs sont absents des projections historiques et ne transportent
/// aucune option Docker libre, aucun chemin hôte ni aucun secret.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimePolicyReference {
    pub policy_id: String,
    pub policy_version: u64,
    pub policy_digest: String,
    pub environment_epoch: u64,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<u64>,
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
    pub runtime_policy: Option<ProjectRuntimePolicyReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_binding_generation: Option<u64>,
    pub observed_at: i64,
}

/// Opération administrative locale du registre. Les mutations restent
/// strictement sur la connexion Service le service compagnon négociée; `List` et `Status`
/// sont des lectures explicites et ne créent aucune liaison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAdminOperation {
    List,
    Status,
    Rebind,
    Disable,
    Activate,
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

/// Synthèse non sensible du dernier audit durable d une liaison projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAuditOperationKind {
    Register,
    Rebind,
    Activate,
    Disable,
    ReviewProjectReconcile,
}

/// Issue fermée de la synthèse d audit publiée au relais administratif local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAuditOutcomeKind {
    Applied,
    Refused,
}

/// Dernier fait d audit d une liaison, sans command_id ni référence de racine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAuditProjection {
    pub operation: ProjectAuditOperationKind,
    pub outcome: ProjectAuditOutcomeKind,
    pub binding_generation: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    pub observed_at: i64,
}

/// Rôle fermé d'un projet dans un daemon Bridget. Seul le rôle système peut
/// participer au dogfooding expert ; aucun chemin ni libellé ne l'infère.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRole {
    #[default]
    Standard,
    BridgetSystem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectBindingProjection {
    pub project_id: String,
    /// Racine canonique exposée uniquement par les surfaces administratives locales.
    /// Elle ne transite jamais par MCP ni par une API réseau générale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_root: Option<String>,
    pub state: ProjectBindingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend: Option<ProjectBackend>,
    #[serde(default)]
    pub role: ProjectRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy: Option<ProjectRuntimePolicyReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRegistryRefusal>,
    /// Dernier audit durable, réservé à la projection administrative locale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_audit: Option<ProjectAuditProjection>,
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

/// Contrat réservé au seul projet système Bridget. Il ne réemploie pas les
/// mutations du registre standard : déclarer ou réconcilier ce rôle ne doit
/// jamais rendre un projet ordinaire administrable comme le système.
pub const PROJECT_SYSTEM_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSystemOperation {
    Status,
    Declare,
    Reconcile,
    DogfoodingPreview,
    DogfoodingApply,
}

/// Valeur fermée du réglage expert du seul projet système Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSystemDogfoodingMode {
    Disabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSystemRequest {
    pub contract_version: u16,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub operation: ProjectSystemOperation,
    pub project_id: String,
    pub expected_binding_generation: u64,
    /// Politique Docker déjà déclarée par le serveur. Obligatoire lors de la
    /// première réconciliation : le projet système ne possède pas de mode
    /// Host de repli.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_setting_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_mode: Option<ProjectSystemDogfoodingMode>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSystemRefusal {
    InvalidContract,
    InvalidRequest,
    IdempotencyExpired,
    PeerUidMismatch,
    ProjectNotFound,
    BindingGenerationMismatch,
    ProjectInactive,
    SystemLocationRequired,
    SystemProjectAlreadyDeclared,
    SystemProjectRequired,
    RuntimePolicyRequired,
    SettingGenerationMismatch,
    DogfoodingModeRequired,
    ActiveSystemAgent,
    DockerRequired,
    RecreateRequired,
    RecreateFailed,
    StoreUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSystemOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub operation: ProjectSystemOperation,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<ProjectRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub setting_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dogfooding_mode: Option<ProjectSystemDogfoodingMode>,
    /// Etat attesté du runtime Docker du projet système, sans détail de chemin
    /// ni d'identifiant de conteneur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectSystemRefusal>,
    pub observed_at: i64,
}

/// Version du contrat local de politique de ronde par projet.
pub const PROJECT_ROUND_POLICY_CONTRACT_VERSION: u16 = 1;

/// Opération fermée du contrôle de ronde. Les lectures ne modifient jamais la
/// politique et les mutations exigent la génération exacte de la liaison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRoundOperation {
    List,
    Status,
    Enable,
    Disable,
}

/// Requête locale corrélée. project_id est absent uniquement pour List et la
/// génération est obligatoire uniquement pour Enable et Disable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRoundRequest {
    pub contract_version: u16,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub operation: ProjectRoundOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
}

/// Refus fermé du contrôle de ronde, sans chemin hôte ni détail fournisseur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRoundRefusal {
    InvalidContract,
    PolicyDisabled,
    InvalidRequest,
    IdempotencyExpired,
    PeerUidMismatch,
    ProjectNotFound,
    ProjectInactive,
    BindingGenerationMismatch,
    EnvelopeMismatch,
    StoreUnavailable,
    /// La pause de l'autonomie est active : aucun réveil n'est émis (SPEC-087).
    ControlPaused,
}

/// Résultat opératoire fermé du dernier passage effectivement admis par la
/// politique. Il décrit la remise de la ronde, jamais la réponse d'un agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRoundDispatchState {
    Deposited,
    Refused,
    Indeterminate,
}

/// Projection effective. configured distingue une désactivation explicite de
/// l'état sûr par défaut, lui aussi disabled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRoundProjection {
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    pub active: bool,
    pub configured: bool,
    pub enabled: bool,
    pub revision: u64,
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_occurrence_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_dispatch_state: Option<ProjectRoundDispatchState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_dispatch_observed_at: Option<i64>,
}

/// Issue rejouable d'une commande de politique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRoundOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub operation: ProjectRoundOperation,
    pub policies: Vec<ProjectRoundProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRoundRefusal>,
    pub observed_at: i64,
}

/// Période canonique de la ronde globale. L'occurrence est calculée par le
/// client, puis contrôlée par le daemon avant toute remise.
pub const PROJECT_ROUND_INTERVAL_SECS: i64 = 7 * 60;

/// Demande interne d'émission d'une occurrence pour une cible déjà sélectionnée.
/// La référence projet est structurée et ne peut pas être déduite du texte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRoundDispatchRequest {
    pub contract_version: u16,
    pub occurrence_at: i64,
    pub project: ProjectReference,
}

/// Issue d'une occurrence projet. Le résultat de remise reste celui du socle
/// idempotent commun aux fournisseurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRoundDispatchOutcome {
    pub contract_version: u16,
    pub occurrence_at: i64,
    pub project: ProjectReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<IdempotencyIssue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRoundRefusal>,
    pub observed_at: i64,
}

/// Opération locale explicitement bornée sur l'environnement Docker d'un projet.
/// Elle n'accepte ni argument Docker, ni chemin hôte, ni image fournie par l'appelant.
pub const PROJECT_RUNTIME_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRuntimeOperation {
    ActivateDocker,
    Prepare,
    Status,
    Stop,
    Remove,
    Recreate,
    SwitchBackend,
}

/// Requête locale versionnée du pilote d'environnement. Seul le daemon lit la
/// liaison durable et la politique hôte fermée correspondante.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimeRequest {
    pub contract_version: u16,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub operation: ProjectRuntimeOperation,
    pub project_id: String,
    /// Verrou optimiste obligatoire pour `activate_docker`. Les opérations
    /// historiques conservent leur contrat et n'envoient pas ce champ.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_binding_generation: Option<u64>,
    /// Référence fermée vers une politique déjà présente sur le serveur.
    /// Aucune image, option Docker ou chemin n'est accepté ici.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<ResolvedProjectProfile>,
}

/// Refus fermé du pilote Docker, sans détail de commande ni chemin local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRuntimeRefusal {
    InvalidContract,
    InvalidProjectId,
    IdempotencyExpired,
    PeerUidMismatch,
    ProjectNotDocker,
    ProjectNotFound,
    PolicyUnavailable,
    BindingGenerationMismatch,
    ActivationFailed,
    EnvironmentBusy,
    PrepareFailed,
    RecreateFailed,
    StoreUnavailable,
}

/// Projection d'exploitation d'un environnement. Les références de racine,
/// les identifiants complets de conteneur et les détails Docker restent privés
/// au daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectRuntimeOutcome {
    pub contract_version: u16,
    pub command_id: String,
    pub project_id: String,
    pub operation: ProjectRuntimeOperation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_generation: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy: Option<ProjectRuntimePolicyReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<ProjectRuntimeRefusal>,
    pub observed_at: i64,
}

/// Refus structurés de la frontière réservée aux services.
/// Version fermée du handshake du socket privé d'un environnement Docker.
pub const RUNTIME_INGRESS_CONTRACT_VERSION: u16 = 1;

/// Preuve déclarée par le wrapper avant toute inscription sur l'ingress privé.
/// Le daemon compare chaque champ à l'environnement durable et à la réservation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIngressHandshake {
    pub contract_version: u16,
    pub project_id: String,
    pub binding_generation: u64,
    pub container_id: String,
    pub environment_epoch: u64,
    pub agent_generation: u64,
    pub instance_id: String,
}

/// Refus fermé de l'ingress. Aucune information de chemin hôte ou Docker n'est
/// rendue au conteneur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeIngressRefusal {
    InvalidContract,
    IdentityMismatch,
    GenerationMismatch,
    EnvironmentEpochStale,
    ReservationMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ServiceRefusal {
    RoleHandshakeRequired,
    ServiceRoleRequired,
    NegotiationRequired,
    AlreadyNegotiated,
    UnsupportedVersion {
        supported_versions: Vec<u16>,
    },
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
    /// Une origine humaine ou un focus a été fourni par un émetteur qui n'est
    /// pas le principal humain. Seul le daemon fabrique cette origine.
    HumanOriginForbidden,
}

/// Opérations fermées que Bridget peut déposer dans le guichet le service compagnon.
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
/// la greffe le service compagnon décide s'il correspond au mandat durable.
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

/// Message humain tel que le daemon l'a observé au ledger avant de fabriquer
/// l'attestation d'origine (SPEC-087). Ce n'est pas une preuve d'identité :
/// c'est le fait causal scellé, pour qu'il ne soit ni inventé ni rejoué.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedHumanMessageFrame {
    pub message_id: String,
    pub ts: i64,
    pub sender: String,
    pub target: String,
    pub body: String,
}

/// SHA-256 en hexadécimal minuscule.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Scellé du contenu d'un message humain observé. Même algorithme que
/// `service compagnon : sceau de contenu de message humain` (préfixe, champs préfixés par
/// leur longueur, horodatage en big-endian) : le daemon scelle, service compagnon
/// vérifie, et les deux doivent produire le même octet.
pub fn human_message_content_seal(observed: &ObservedHumanMessageFrame) -> String {
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(b"bridget/human-origin-seal/v1");
    for field in [
        observed.message_id.as_bytes(),
        observed.sender.as_bytes(),
        observed.target.as_bytes(),
        observed.body.as_bytes(),
    ] {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field);
    }
    digest.update(observed.ts.to_be_bytes());
    digest
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Attestation d'origine humaine portée par un dépôt de délégation. Elle est
/// fabriquée uniquement par le daemon et rejouée par le service compagnon contre ses cinq
/// vérifications (`ObjectiveOpeningPermit::human_request`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanOriginAttestationFrame {
    pub version: u16,
    pub issuer_scope: String,
    pub canonical_request_sha256: String,
    pub signature: String,
}

/// Provenance fermée d'un dépôt de délégation. Absente : ouverture automatique.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DelegateOrigin {
    Human {
        message_id: String,
        observed: ObservedHumanMessageFrame,
        attestation: HumanOriginAttestationFrame,
    },
}

/// Conduite quand un focus existe déjà au moment d'en demander un autre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FocusConflictPolicy {
    Replace,
    Queue,
}

/// Demande de focus : l'objectif ouvert devient prioritaire pour ce projet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelegateFocus {
    pub project_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_conflict: Option<FocusConflictPolicy>,
}

/// État de contrôle du référent, projeté par le daemon (SPEC-087).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlStateFrame {
    pub version: u16,
    pub generation: u64,
    pub paused: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paused_since: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paused_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause_reason: Option<String>,
    pub auto_objectives_cap: u32,
    /// SPEC-088 : posture de lancement des agents gérés. Additif : un daemon
    /// antérieur ne l'émet pas ; `None` = inconnu, jamais une valeur permissive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_posture: Option<AgentPosture>,
    /// SPEC-088 : réassignation automatique admise. `None` = inconnu ⇒ différé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_reassignment: Option<bool>,
    pub updated_at: i64,
}

/// Postures de lancement que Bridget sait produire (SPEC-088, FR-016) :
/// définition normale du registre, ou définition de découverte en lecture
/// seule. Pas de troisième valeur tant qu'aucune n'est attestée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentPosture {
    Discovery,
    Complete,
}

impl AgentPosture {
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::Complete => "complete",
        }
    }

    pub fn from_sql(raw: &str) -> Option<Self> {
        match raw {
            "discovery" => Some(Self::Discovery),
            "complete" => Some(Self::Complete),
            _ => None,
        }
    }
}

/// Projection minimale du focus courant, publiée par le service compagnon dans Bridget.
/// Le daemon la conserve et la relit, mais ne consulte jamais la base le service compagnon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlFocusFrame {
    pub objective_id: String,
    pub goal: String,
    pub project_id: String,
    pub updated_at: i64,
}

/// Ligne du journal des mutations de contrôle, relisible par la CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlEventFrame {
    pub at: i64,
    pub actor: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub generation_after: u64,
}

/// Refus fermé d'une mutation de l'état de contrôle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ControlStateRefusal {
    HumanPrincipalRequired,
    GenerationMismatch { current: u64 },
    BudgetOutOfRange { min: u32, max: u32 },
    NothingToChange,
    StoreUnavailable,
    UnsupportedVersion,
    CapabilityRequired,
}

/// Type fermé d'un item de la boîte de réception humaine.
///
/// Le libellé SQL est dérivé du nom de fil par serde : une seule source pour
/// le `CHECK (kind IN (…))` de la table et pour la trame. `ALL` reste la seule
/// liste littérale ; `as_sql` est exhaustif par construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanInboxKind {
    InterventionRequired,
    ChainExhausted,
    ReviewVerdictPending,
    ActivationApproval,
    BudgetReached,
    ReplyDebt,
    FocusWaitingAgent,
    ObjectVanished,
    HumanRouteReplaced,
}

impl HumanInboxKind {
    pub const ALL: [Self; 9] = [
        Self::InterventionRequired,
        Self::ChainExhausted,
        Self::ReviewVerdictPending,
        Self::ActivationApproval,
        Self::BudgetReached,
        Self::ReplyDebt,
        Self::FocusWaitingAgent,
        Self::ObjectVanished,
        Self::HumanRouteReplaced,
    ];

    /// Libellé de fil et de colonne SQL, identique par construction.
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::InterventionRequired => "intervention_required",
            Self::ChainExhausted => "chain_exhausted",
            Self::ReviewVerdictPending => "review_verdict_pending",
            Self::ActivationApproval => "activation_approval",
            Self::BudgetReached => "budget_reached",
            Self::ReplyDebt => "reply_debt",
            Self::FocusWaitingAgent => "focus_waiting_agent",
            Self::ObjectVanished => "object_vanished",
            Self::HumanRouteReplaced => "human_route_replaced",
        }
    }

    /// Clause `IN ('…', …)` prête pour un `CHECK`, dérivée de `ALL`.
    pub fn sql_in_clause() -> String {
        let quoted: Vec<String> = Self::ALL
            .iter()
            .map(|kind| format!("'{}'", kind.as_sql()))
            .collect();
        format!("IN ({})", quoted.join(", "))
    }

    pub fn from_sql(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.as_sql() == value)
    }
}

/// État fermé d'un item de boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanInboxState {
    Open,
    Resolved,
    ClosedSelf,
}

/// Producteur d'un item : celui qui relèvera la décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanInboxProducer {
    Daemon,
    Guichet,
}

/// Référence vers l'objet concerné par un item. Tous les champs sont
/// optionnels : un item de reprise de route n'a ni objectif ni délégation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInboxSubject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

/// Décision prise par le référent sur un item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInboxDecision {
    pub decision_id: String,
    pub choice: String,
    pub actor: String,
    pub at: i64,
}

/// Item de boîte projeté vers un client humain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInboxItemFrame {
    pub id: String,
    pub dedup_key: String,
    pub kind: HumanInboxKind,
    pub subject: HumanInboxSubject,
    /// Résumé lisible et faits utiles, texte JSON borné par le daemon.
    pub context: String,
    pub options: Vec<String>,
    pub state: HumanInboxState,
    pub producer: HumanInboxProducer,
    pub created_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<i64>,
    pub occurrences: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<HumanInboxDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acked_at: Option<i64>,
}

/// Décision non encore acquittée par son producteur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanInboxPendingDecision {
    pub decision_id: String,
    pub item_id: String,
    pub dedup_key: String,
    pub kind: HumanInboxKind,
    pub subject: HumanInboxSubject,
    pub choice: String,
    pub at: i64,
}

/// Filtre de liste de la boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanInboxListFilter {
    Open,
    All,
}

/// Refus fermé d'une opération de boîte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HumanInboxRefusal {
    HumanPrincipalRequired,
    CapabilityRequired,
    UnsupportedVersion,
    InvalidRequest,
    UnknownItem,
    AlreadyResolved,
    ChoiceNotOffered,
    StoreUnavailable,
}

/// Charge canonique d'un dépôt de guichet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
// `Delegate` porte l'attestation humaine et le focus dans la trame publique.
// Les boxer modifierait l'API Rust sans changer les octets sur le fil ; ce coût
// n'est pas justifié pour une charge rare, contrôlée et durable.
#[allow(clippy::large_enum_variant)]
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
        /// Fabriquée par le daemon pour le principal humain ; refusée sinon.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        origin: Option<DelegateOrigin>,
        /// Demande de focus, valide uniquement avec une origine humaine.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focus: Option<DelegateFocus>,
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
            }
            | Self::Delegate {
                origin: Some(_), ..
            }
            | Self::Delegate { focus: Some(_), .. } => REVIEW_DELEGATE_CONTRACT_VERSION,
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

/// Issue fermée qu'un service compagnon atteste au guichet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetOutcome {
    Accepted,
    RequestAlreadyTerminal,
    RecipientUnavailable,
    Refused,
}

/// Motif fermé d'un refus déterministe rendu par le service compagnon après la relève.
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
    /// L'attestation d'origine humaine n'a pas passé les vérifications de
    /// le service compagnon, ou un focus a été demandé sans origine humaine valide. Code
    /// public unique : la garde précise reste dans le journal le service compagnon.
    HumanOriginInvalid,
    /// Le plafond de création automatique est atteint. Les deux valeurs sont
    /// attestées par le service compagnon au moment du refus, elles ne sont jamais déduites
    /// par Bridget lors de l'affichage ou d'un rejeu.
    BudgetReached {
        cap: u32,
        open: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuichetDelegateMutationStatus {
    Created,
    WaitingForAgent,
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
/// Cette énumération est volontairement fermée : le service compagnon ne déduit jamais une
/// relance d'un texte ou d'une échéance locale, et une valeur future exige une
/// capacité/version explicitement négociée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinationEventKind {
    ReminderSent,
}

/// Charge canonique d'une réponse le service compagnon. L'ordre de déclaration est l'ordre
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

/// Projection fermée d'une coordination le service compagnon : Bridget la transporte sans
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
    /// Refus historique conservé sur le fil : dans le cœur communication,
    /// toute nouvelle exécution exigeant le runtime projet est indisponible.
    /// Une ancienne référence n'est jamais convertie en lancement hôte.
    DockerRuntimeUnavailable {
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
    Started { agent_id: String, generation: u64 },
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

/// Choix local à un ordre, distinct de la posture globale du référent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpawnPosture {
    Discovery,
    Development,
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

    pub fn parse(value: &str) -> Option<Self> {
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

pub const ARTIFACT_READ_VERSION: u8 = 1;
pub const MAX_ARTIFACT_READ_BYTES: u32 = 16 * 1024;
pub const MAX_ARTIFACT_REF_BYTES: usize = 256;

/// Lecture de contenu seulement : aucune identité ni aucun chemin n'est accepté.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReadRequest {
    pub version: u8,
    pub artifact_ref: String,
    pub version_ref: String,
    pub kind: ArtifactReadKind,
    pub offset: u64,
    pub limit: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArtifactReadKind {
    Manifest,
    Blob { digest: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArtifactReadOutcome {
    Chunk {
        bytes: Vec<u8>,
        total_len: u64,
        /// SHA-256 du contenu entier, jamais du seul fragment.
        digest: String,
        offset: u64,
        next_offset: Option<u64>,
    },
    Rejected {
        reason: ArtifactReadRefusal,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactReadRefusal {
    UnsupportedVersion,
    InvalidRequest,
    IdentityUnavailable,
    ScopeUnavailable,
    NotFound,
    BlobNotLinked,
    OffsetOutOfRange,
    CorruptContent,
    ContentUnavailable,
    StorageUnavailable,
}

/// Preuve privée de rattachement. La sérialisation transporte le secret,
/// mais aucune projection Debug (y compris des trames) ne le révèle.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct IdentityCredential(String);

impl IdentityCredential {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for IdentityCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IdentityCredential([REDACTED])")
    }
}

/// Messages envoyés par le wrapper vers le daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationKind {
    TurnEnded,
    PermissionRequired,
    FileWritten,
    FileCollision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ObservationRequest {
    Types {},
    Sub {
        event: ObservationKind,
        agent: Option<String>,
        file: Option<String>,
        #[serde(default)]
        once: bool,
        ttl_secs: Option<u64>,
    },
    List {},
    Unsub {
        id: String,
    },
}

/// Session 102 — fils inter-agents. Version fixée par le client, refusée si
/// inconnue ; chaque variante rejette les champs inconnus.
pub const THREAD_CONTRACT_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadRequest {
    pub version: u16,
    pub request: ThreadAction,
}

/// Cibles d'un dépôt : `[]` silence explicite, liste d'UUID, ou `"all"`.
/// Le champ est obligatoire : la discipline de sollicitation n'est jamais
/// laissée au hasard d'une omission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ThreadNotify {
    All(ThreadNotifyAll),
    Targets(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreadNotifyAll {
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ThreadAction {
    Create {
        title: String,
        members: Vec<String>,
        operation_id: String,
    },
    List {
        #[serde(default)]
        limit: Option<u32>,
        #[serde(default)]
        after_thread_id: Option<String>,
    },
    Show {
        thread_id: String,
    },
    Post {
        thread_id: String,
        body: String,
        notify: ThreadNotify,
        operation_id: String,
        #[serde(default)]
        reply_to_seq: Option<u64>,
        #[serde(default)]
        ack_receipt: Option<String>,
    },
    Read {
        thread_id: String,
        #[serde(default)]
        limit: Option<u32>,
    },
    Ack {
        thread_id: String,
        receipt: String,
    },
    History {
        thread_id: String,
        #[serde(default)]
        from_seq: Option<u64>,
        #[serde(default)]
        to_seq: Option<u64>,
        #[serde(default)]
        limit: Option<u32>,
    },
    Close {
        thread_id: String,
        operation_id: String,
    },
}

impl ThreadAction {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Create { .. } => "create",
            Self::List { .. } => "list",
            Self::Show { .. } => "show",
            Self::Post { .. } => "post",
            Self::Read { .. } => "read",
            Self::Ack { .. } => "ack",
            Self::History { .. } => "history",
            Self::Close { .. } => "close",
        }
    }
}

/// Résultat versionné : `result` porte un discriminant fermé `status`
/// (created/listed/shown/posted/read/history/acknowledged/
/// already_acknowledged/closed/error), identique pour CLI et MCP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreadResult {
    pub version: u16,
    pub result: serde_json::Value,
}

/// Messages envoyés par le wrapper vers le daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum WrapperToDaemon {
    /// Session 102 : opération sur un fil, réservée à une identité attestée.
    ThreadRequest {
        request: ThreadRequest,
    },
    /// Session 102 : versions d'alerte de fil que ce wrapper sait recevoir,
    /// annoncées après `Registered` comme les autres faits de connexion ;
    /// vide = aucune. La sélection est liée à la connexion et disparaît avec elle.
    ThreadNoticeCapability {
        versions: Vec<u16>,
    },
    /// Lacune de faits structurés signalée par le seul producteur primaire.
    ObservationGap {
        /// Zéro : continuité non garantie, quantité inconnue ; positif : pertes comptées.
        dropped: u64,
    },
    /// Capacités réelles du producteur primaire vivant ; vide = indisponible.
    ObservationCapabilities {
        events: Vec<ObservationKind>,
    },
    ObservationRequest {
        request: ObservationRequest,
    },
    /// Fait minimal tiré du journal vivant par son seul wrapper propriétaire.
    ObservedActivity {
        seq: u64,
        event: ObservationKind,
        file: Option<String>,
    },
    /// Atteste une connexion auxiliaire sans remplacer la route propriétaire.
    RegisterAuxiliary {
        agent_id: String,
        instance_id: String,
        credential: IdentityCredential,
    },
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
    #[serde(rename = "artifact_read")]
    ArtifactRead {
        request: ArtifactReadRequest,
    },
    /// Publication d'un artefact déjà sérialisé canoniquement. Le daemon
    /// déduit le principal, le projet et le contexte de conversation de la
    /// connexion wrapper observée, jamais de cette charge utile.
    #[serde(rename = "artifact_publish")]
    ArtifactPublish {
        #[serde(rename = "v")]
        contract_version: u8,
        #[serde(with = "base64_bytes")]
        canonical_publication: Vec<u8>,
    },
    /// Négocie le contrat du service compagnon, uniquement après RoleAccepted(Service).
    ServiceHello {
        version: u16,
        service: String,
        issuer_scope: String,
        capabilities: Vec<ServiceCapability>,
    },
    /// Demande locale service compagnon vers Bridget. Elle ne reprend pas le sens inverse
    /// de `ServiceRequest`, qui reste un dépôt Bridget vers le guichet le service compagnon.
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
    /// Déclaration et relève du projet système, surface séparée du registre
    /// standard et admise seulement par le daemon local.
    #[serde(rename = "project_system_request")]
    ProjectSystemRequest {
        request: ProjectSystemRequest,
    },
    /// Lecture ou mutation locale de la ronde par projet.
    #[serde(rename = "project_round_request")]
    ProjectRoundRequest {
        request: ProjectRoundRequest,
    },
    /// Émission interne d'une occurrence déjà filtrée par le registre projet.
    #[serde(rename = "project_round_dispatch")]
    ProjectRoundDispatch {
        request: ProjectRoundDispatchRequest,
    },
    #[serde(rename = "project_profile_request")]
    ProjectProfileRequest {
        request: ProjectProfileRequest,
    },
    /// Commande locale du pilote Docker. L'authentification reste fondée sur
    /// le pair Unix observé par le daemon, jamais sur un champ JSON.
    #[serde(rename = "project_runtime_request")]
    ProjectRuntimeRequest {
        request: ProjectRuntimeRequest,
    },
    /// Handshake obligatoire avant toute inscription via le socket privé d'un
    /// environnement Docker. Il n'est jamais envoyé sur le socket utilisateur.
    #[serde(rename = "runtime_ingress_hello")]
    RuntimeIngressHello {
        hello: RuntimeIngressHandshake,
    },
    // Verification non consommatrice avant lecture locale dun secret process-env.
    #[serde(rename = "runtime_ingress_preflight")]
    RuntimeIngressPreflight {
        hello: RuntimeIngressHandshake,
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
    /// Dépôt durable produit par un wrapper enregistré vers le guichet le service compagnon.
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        posture: Option<SpawnPosture>,
        agent_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        project: Option<ProjectReference>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ownership: Option<SpawnOwnership>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_id: Option<String>,
        cwd: String,
        persistent: bool,
        command_id: String,
        issued_at: i64,
        deadline_at: i64,
    },
    /// Ordre corrélé d'arrêt d'un équipier supervisé.
    StopOrder {
        agent_id: String,
        command_id: String,
    },
    /// Relance corrélée d'un agent géré durablement arrêté.
    RelaunchOrder {
        agent_id: String,
        command_id: String,
    },
    /// Retrait corrélé de la flotte visible, sans purge d'historique.
    DecommissionOrder {
        agent_id: String,
        command_id: String,
    },
    /// Migration explicite d'un agent historique arrêté vers le registre v4.
    AdoptStoppedOrder {
        agent_id: String,
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
        /// Version du contrat d'identité. La version 2 interdit tout nom de
        /// routage historique et évite qu'un wrapper ancien soit admis comme
        /// une nouvelle identité.
        identity_version: u8,
        agent_type: String,
        /// Identifiant opaque créé par Bridget et stable sur les reconnexions.
        agent_id: String,
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
    /// La session native a démarré dans un terminal qui possède son cycle de
    /// vie. Fait lié à la connexion, à réannoncer après chaque reconnexion.
    /// Ne se déduit ni du protocole fournisseur ni de la présence d'un journal.
    TerminalSessionReady,
    /// Fait fournisseur corrélé à une exécution. Les versions anciennes ne
    /// l'émettent pas, ce qui laisse le contexte explicitement absent.
    ExecutionProviderObserved {
        context: ExecutionProviderContext,
    },
    /// Se désenregistrer.
    Unregister,
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
    /// Session 104 : recherche bornée et reprenable dans les échanges de
    /// l'appelant (messages) ou d'un fil dont il est membre. L'identité vient
    /// de la connexion attestée, jamais de la requête.
    #[serde(rename = "ledger_search")]
    LedgerSearch {
        request: LedgerSearchRequest,
    },
    /// Session 104 : relecture exacte d'un message par sa clé (id, target).
    #[serde(rename = "ledger_read")]
    LedgerRead {
        request: LedgerReadRequest,
    },
    /// Signal de vie (périodique).
    Heartbeat,
    /// Demander la liste des agents connectés.
    ListAgents,
    /// Extension 089 v1 : aucune mutation de route, de scope ou d'instructions.
    #[serde(rename = "display_name_set")]
    DisplayNameSet {
        request: DisplayNameRequest,
    },
    #[serde(rename = "display_name_resolve")]
    DisplayNameResolve {
        request: DisplayNameRequest,
    },
    /// Demande au daemon ce qu'il atteste de LUI-MÊME : sa machine et sa base.
    ///
    /// Un client fédéré ne peut pas les déduire — il affichait jusqu'ici SES
    /// chemins à côté de chiffres venus d'ici. Message dédié plutôt qu'un champ
    /// de plus sur `ClientWelcome` : aucune construction existante à modifier,
    /// donc aucun fichier tiers touché.
    DaemonIdentityRequest,
    /// Contrôle humain de session ; ne devient jamais un message au modèle.
    SelectRuntime {
        agent: String,
        selection: RuntimeSelection,
    },
    /// Réponse corrélée au wrapper réellement ciblé, jamais une déclaration libre.
    RuntimeSelectionReported {
        token: String,
        outcome: RuntimeSelectionOutcome,
    },
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
        /// Fournisseur choisi par le wrapper au moment de l'échantillon. Les
        /// wrappers historiques ne le transmettent pas : l'absence reste
        /// explicite et ne doit jamais être remplacée par une inférence UI.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        provider_kind: Option<String>,
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
    /// Lire l'état de contrôle du référent (rôle client, SPEC-087).
    #[serde(rename = "control_state_read")]
    ControlStateRead {
        version: u16,
    },
    /// Lire la projection passive du focus publiée par le service compagnon (SPEC-087).
    #[serde(rename = "control_focus_read")]
    ControlFocusRead {
        version: u16,
    },
    /// Relire le journal des mutations de contrôle, du plus récent au plus
    /// ancien.
    #[serde(rename = "control_history")]
    ControlHistory {
        version: u16,
        limit: u32,
    },
    /// Muter l'état de contrôle. Réservé au principal humain ; au moins un
    /// champ non nul, génération attendue obligatoire.
    #[serde(rename = "control_state_set")]
    ControlStateSet {
        version: u16,
        command_id: String,
        expected_generation: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        paused: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auto_objectives_cap: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
        /// SPEC-088 : droits du référent, sous la même génération.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_posture: Option<AgentPosture>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auto_reassignment: Option<bool>,
    },
    /// Déposer un item dans la boîte de réception humaine (rôle service).
    #[serde(rename = "human_inbox_deposit")]
    HumanInboxDeposit {
        version: u16,
        dedup_key: String,
        kind: HumanInboxKind,
        subject: HumanInboxSubject,
        context: String,
        options: Vec<String>,
    },
    /// Publier la projection passive du focus. Réservé au service compagnon,
    /// négocié avec la même capacité que la boîte humaine.
    #[serde(rename = "control_focus_publish")]
    ControlFocusPublish {
        version: u16,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focus: Option<ControlFocusFrame>,
    },
    /// Lister la boîte (principal humain).
    #[serde(rename = "human_inbox_list")]
    HumanInboxList {
        version: u16,
        state: HumanInboxListFilter,
        limit: u32,
    },
    /// Trancher un item (principal humain).
    #[serde(rename = "human_inbox_resolve")]
    HumanInboxResolve {
        version: u16,
        command_id: String,
        item_id: String,
        choice: String,
    },
    /// Relever les décisions non acquittées du producteur (rôle service).
    /// La relève ne modifie rien.
    #[serde(rename = "human_inbox_decisions")]
    HumanInboxDecisions {
        version: u16,
        limit: u32,
    },
    /// Acquitter une décision après l'avoir appliquée durablement.
    #[serde(rename = "human_inbox_ack")]
    HumanInboxAck {
        version: u16,
        decision_id: String,
    },
    /// Fermer un item ouvert dont l'objet source a disparu (rôle service).
    #[serde(rename = "human_inbox_close")]
    HumanInboxClose {
        version: u16,
        item_id: String,
        reason: String,
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
    /// Session 102 : résultat d'une opération de fil.
    ThreadResult {
        result: ThreadResult,
    },
    ObservationResult {
        result: serde_json::Value,
    },
    #[serde(rename = "runtime_ingress_accepted")]
    RuntimeIngressAccepted {
        project_id: String,
        binding_generation: u64,
        environment_epoch: u64,
    },
    #[serde(rename = "runtime_ingress_rejected")]
    RuntimeIngressRejected {
        reason: RuntimeIngressRefusal,
    },
    /// Le rôle demandé est accepté pour cette connexion.
    RoleAccepted {
        role: ConnectionRole,
    },
    #[serde(rename = "artifact_read_result")]
    ArtifactReadResult {
        version: u8,
        artifact_ref: String,
        version_ref: String,
        outcome: ArtifactReadOutcome,
    },
    /// Issue terminale et attestée d'une publication d'artefact. Les détails
    /// techniques restent volontairement bornés : un appelant reçoit soit le
    /// reçu signé par Bridget, soit un code et un message sûrs à afficher.
    #[serde(rename = "artifact_publication_result")]
    ArtifactPublicationResult {
        #[serde(rename = "v")]
        contract_version: u8,
        replayed: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        receipt_json: Option<Vec<u8>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refusal_code: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refusal_message: Option<String>,
    },
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
    ClientRejected {
        reason: ClientRefusal,
    },
    /// Contrat et capacité réellement négociés avec un service compagnon.
    ServiceWelcome {
        version: u16,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        capabilities: Vec<ServiceCapability>,
    },
    /// Refus motivé de la négociation ou de la matrice de service.
    ServiceRejected {
        reason: ServiceRefusal,
    },
    /// Issue terminale du registre local, corrélée à la commandu service compagnon.
    #[serde(rename = "project_registry_outcome")]
    ProjectRegistryOutcome {
        outcome: ProjectBindOutcome,
    },
    /// Issue corrélée d'une lecture ou mutation administrative du registre.
    #[serde(rename = "project_registry_admin_outcome")]
    ProjectRegistryAdminOutcome {
        outcome: ProjectAdminOutcome,
    },
    #[serde(rename = "project_system_outcome")]
    ProjectSystemOutcome {
        outcome: ProjectSystemOutcome,
    },
    /// Issue locale de la politique de ronde par projet.
    #[serde(rename = "project_round_outcome")]
    ProjectRoundOutcome {
        outcome: ProjectRoundOutcome,
    },
    #[serde(rename = "project_round_dispatch_outcome")]
    ProjectRoundDispatchOutcome {
        outcome: ProjectRoundDispatchOutcome,
    },
    #[serde(rename = "project_profile_outcome")]
    ProjectProfileOutcome {
        outcome: ProjectProfileOutcome,
    },
    /// Issue bornée d'une opération locale d'environnement Docker.
    #[serde(rename = "project_runtime_outcome")]
    ProjectRuntimeOutcome {
        outcome: ProjectRuntimeOutcome,
    },
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
    /// le service compagnon après la transition SQLite correspondante.
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
    /// négocié `coordination_events_v1` en plus de `guichet_v1`.
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
    DelegatedRuntimeEvent {
        event: DelegatedRuntimeEventFrame,
    },
    ///
    /// Cette trame ne franchit jamais la frontière client publique.
    ControlExecutionDispatch {
        issuer_scope: String,
        command: ExecutionControlCommand,
    },
    SelectRuntime {
        token: String,
        selection: RuntimeSelection,
    },
    RuntimeSelectionResult {
        outcome: RuntimeSelectionOutcome,
    },
    /// Succès d'un spawn, émis seulement après le `Register` réel.
    SpawnAccepted {
        command_id: String,
        agent_id: String,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        execution: Option<ExecutionDeliveryContext>,
    },
    /// Souscription du daemon vers le wrapper lecteur du journal.
    Subscribe {
        subscription_id: String,
        agent: String,
        window: AttachWindow,
    },
    /// Désabonnement relayé au wrapper.
    Unsubscribe {
        subscription_id: String,
    },
    /// Confirmation d'abonnement envoyée à la vue après acceptation wrapper.
    Subscribed {
        subscription_id: String,
    },
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
    /// Confirmation d'enregistrement avec l'identifiant stable.
    Registered {
        agent_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        credential: Option<IdentityCredential>,
    },
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
    CancelDelivery {
        id: String,
        reason: String,
    },
    /// Acquittement d'un envoi.
    Ack {
        id: String,
    },
    /// Refus d'un envoi avec raison.
    Nack {
        id: String,
        reason: String,
    },
    /// Échec terminal différé d'un envoi attach déjà acquitté.
    DeliveryRejected {
        id: String,
        reason: String,
    },
    /// Le daemon s'éteint.
    Disconnect,
    /// Réponse à ListAgents.
    AgentList {
        agents: Vec<AgentInfo>,
    },
    #[serde(rename = "display_name_result")]
    DisplayNameResult {
        outcome: DisplayNameOutcome,
    },
    #[serde(rename = "display_name_resolution")]
    DisplayNameResolved {
        outcome: DisplayNameResolution,
    },
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
    RequestCancelled {
        id: String,
        state: String,
    },
    /// Liste des demandes suivies accessibles à l'agent courant.
    RequestList {
        requests: Vec<RequestInfo>,
    },
    /// Projection bornée du ledger, indépendante de tout rendu CLI.
    LedgerProjection {
        messages: Vec<LedgerMessage>,
        requests: Vec<RequestInfo>,
    },
    /// Session 104 : page de recherche ou refus typé.
    #[serde(rename = "ledger_search_result")]
    LedgerSearchResult {
        outcome: LedgerSearchOutcomeV1,
    },
    /// Session 104 : fragment relu ou refus typé.
    #[serde(rename = "ledger_read_result")]
    LedgerReadResult {
        outcome: LedgerReadOutcomeV1,
    },
    /// État de contrôle courant (SPEC-087), avec le nombre d'items ouverts de
    /// la boîte humaine : un compte, jamais leur contenu.
    #[serde(rename = "control_state")]
    ControlState {
        state: ControlStateFrame,
        #[serde(default)]
        inbox_open_count: u32,
    },
    #[serde(rename = "control_focus")]
    ControlFocus {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        focus: Option<ControlFocusFrame>,
    },
    #[serde(rename = "control_history")]
    ControlHistory {
        events: Vec<ControlEventFrame>,
    },
    #[serde(rename = "control_state_rejected")]
    ControlStateRejected {
        reason: ControlStateRefusal,
    },
    /// Reçu d'un dépôt dans la boîte : `created` distingue un item neuf d'une
    /// occurrence rattachée à un item déjà ouvert.
    #[serde(rename = "human_inbox_deposited")]
    HumanInboxDeposited {
        item_id: String,
        created: bool,
        occurrences: u32,
    },
    #[serde(rename = "human_inbox")]
    HumanInbox {
        items: Vec<HumanInboxItemFrame>,
        open_count: u32,
    },
    #[serde(rename = "human_inbox_rejected")]
    HumanInboxRejected {
        reason: HumanInboxRefusal,
    },
    #[serde(rename = "human_inbox_decisions_batch")]
    HumanInboxDecisionsBatch {
        decisions: Vec<HumanInboxPendingDecision>,
    },
    #[serde(rename = "human_inbox_acked")]
    HumanInboxAcked {
        decision_id: String,
        acked_at: i64,
    },
    #[serde(rename = "human_inbox_closed")]
    HumanInboxClosed {
        item_id: String,
        closed: bool,
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

// ---------------------------------------------------------------------------
// Session 104 — recherche et relecture des échanges
// ---------------------------------------------------------------------------

/// Source d'une recherche 104 : les messages de l'appelant ou un fil 102.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LedgerSearchSource {
    #[default]
    Messages,
    Thread,
}

impl LedgerSearchSource {
    pub fn name(self) -> &'static str {
        match self {
            Self::Messages => "messages",
            Self::Thread => "thread",
        }
    }
}

/// Requête de recherche 104. Aucun champ n'identifie l'appelant, son
/// instance ni un chemin de base : la portée vient de la connexion.
/// Les bornes (query ≤ 256 octets, 1..8 termes, limit 1..50, since ≤ until,
/// UUID canoniques) sont contrôlées par le daemon, pas par serde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerSearchRequest {
    #[serde(default)]
    pub source: LedgerSearchSource,
    pub query: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Correspondant (source messages seulement).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    /// Bornes Unix inclusives, en secondes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    /// Curseur opaque rendu par la page précédente (hex minuscule).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Exigé pour `source = thread`, interdit sinon.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
}

/// Résultat de recherche 104 : un message (clé physique `(id, target)`) ou
/// une entrée de fil (clé `(thread_id, seq)`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LedgerSearchHit {
    Message {
        id: String,
        target: String,
        sender: String,
        ts: i64,
        excerpt: String,
        /// Offset en octets du corps original où commence le premier terme
        /// trouvé (frontière UTF-8) ; à réutiliser tel quel pour `read`.
        match_offset: u64,
        /// SHA-256 hexadécimal du corps entier.
        body_digest: String,
        body_bytes: u64,
    },
    ThreadEntry {
        thread_id: String,
        seq: u64,
        message_id: String,
        author_id: String,
        ts: i64,
        excerpt: String,
        match_offset: u64,
        body_digest: String,
        body_bytes: u64,
    },
}

/// Page de recherche : compteurs locaux à cette page, jamais un total.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSearchPage {
    pub source: LedgerSearchSource,
    pub hits: Vec<LedgerSearchHit>,
    /// Des candidats restent à explorer ; pas forcément des occurrences.
    pub has_more: bool,
    pub next_cursor: Option<String>,
    /// Candidats autorisés consommés dans cette page, filtres compris.
    pub scanned_count: u32,
    /// Octets de corps réellement examinés.
    pub scanned_bytes: u64,
    pub skipped_oversized: u32,
    /// exhausted | result_limit | scan_budget | byte_budget | response_budget
    pub stop_reason: String,
    /// live_bounded (messages) | immutable_upper_bound (fil)
    pub consistency: String,
    pub notices: Vec<String>,
}

/// Réponse de recherche : succès typé ou refus `{code, reason}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LedgerSearchOutcomeV1 {
    Ok(LedgerSearchPage),
    Error { code: String, reason: String },
}

/// Relecture d'un message exact : `id` et `target` désignent la clé
/// physique ; `digest` est facultatif seulement à `offset = 0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerReadRequest {
    pub id: String,
    pub target: String,
    #[serde(default)]
    pub offset: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

/// Fragment relu (≤ 16 384 octets UTF-8) et empreinte du corps entier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerReadFragment {
    pub id: String,
    pub target: String,
    pub sender: String,
    pub ts: i64,
    pub body_bytes: u64,
    pub digest: String,
    pub fragment: String,
    pub next_offset: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LedgerReadOutcomeV1 {
    Ok(LedgerReadFragment),
    Error { code: String, reason: String },
}

/// Réglages opaques : aucune substitution ni modification de permissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSelection {
    pub model: String,
    pub effort: String,
}
impl RuntimeSelection {
    pub fn valid(&self) -> bool {
        [&self.model, &self.effort].iter().all(|s| {
            !s.is_empty()
                && s.len() <= 128
                && s.chars().all(|c| !c.is_control() && !c.is_whitespace())
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSelectionRefusal {
    Unsupported,
    InvalidSelection,
    ModelUnavailable,
    EffortUnavailable,
    CatalogueUnavailable,
    TargetUnavailable,
    AttachRequired,
    Busy,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeSelectionOutcome {
    /// Réglage accepté par le fil natif pour ses prochains tours, PAS une inférence attestée.
    Selected {
        selection: RuntimeSelection,
    },
    Refused {
        reason: RuntimeSelectionRefusal,
    },
    OutcomeUnknown {},
}

impl WrapperToDaemon {
    /// Vérifie la matrice fermée d'une connexion déjà négociée comme attach.
    /// Le handshake est volontairement exclu : il n'est admis qu'avant que le
    /// daemon n'enregistre le rôle de la connexion.
    pub fn attach_refusal(&self) -> Option<AttachRefusal> {
        match self {
            Self::Subscribe { .. }
            | Self::Unsubscribe { .. }
            | Self::Heartbeat
            | Self::SelectRuntime { .. }
            | Self::ListAgents => None,
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
                | Self::AgentList { .. }
                | Self::RuntimeSelectionResult { .. }
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
    /// Identifiant opaque utilisé pour les opérations techniques.
    pub agent_id: String,
    /// Nom humain projeté par le daemon. Les clients ne reconstruisent jamais cette valeur.
    pub display_name: String,
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
    agent_id: String,
    display_name: String,
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
            agent_id: wire.agent_id,
            display_name: wire.display_name,
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
    #[test]
    fn spec099_credential_optionnel_compatible_et_masque_dans_debug() {
        use super::{DaemonToWrapper, IdentityCredential, WrapperToDaemon};
        let old: DaemonToWrapper =
            serde_json::from_str(r#"{"type":"Registered","agent_id":"agent"}"#).unwrap();
        assert!(matches!(
            old,
            DaemonToWrapper::Registered {
                credential: None,
                ..
            }
        ));
        let secret = "ne-doit-jamais-apparaitre-dans-les-logs";
        let credential = IdentityCredential::new(secret.into());
        let registration = WrapperToDaemon::RegisterAuxiliary {
            agent_id: "agent".into(),
            instance_id: "instance".into(),
            credential: credential.clone(),
        };
        let response = DaemonToWrapper::Registered {
            agent_id: "agent".into(),
            credential: Some(credential),
        };
        assert!(!format!("{registration:?} {response:?}").contains(secret));
        assert!(
            serde_json::to_string(&registration)
                .unwrap()
                .contains(secret)
        );
    }

    #[test]
    fn spec091_selection_fermee_bornee_et_sans_permissions() {
        use super::{RuntimeSelection, RuntimeSelectionOutcome};
        let source = r#"{"model":"gpt-5.6-terra","effort":"medium"}"#;
        let selection: RuntimeSelection = serde_json::from_str(source).unwrap();
        assert!(selection.valid());
        assert_eq!(serde_json::to_string(&selection).unwrap(), source);
        for source in [
            r#"{"model":"x","effort":"high","approval":"never"}"#,
            r#"{"model":"x"}"#,
        ] {
            assert!(serde_json::from_str::<RuntimeSelection>(source).is_err());
        }
        for model in [
            String::new(),
            "x".repeat(129),
            "x\u{1b}[2J".into(),
            "x y".into(),
        ] {
            assert!(
                !RuntimeSelection {
                    model,
                    effort: "high".into()
                }
                .valid()
            );
        }
        assert!(
            serde_json::from_str::<RuntimeSelectionOutcome>(
                r#"{"status":"refused","reason":"unknown"}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RuntimeSelectionOutcome>(
                r#"{"status":"outcome_unknown","applied":true}"#
            )
            .is_err()
        );
    }
    use super::*;

    #[test]
    fn terminal_session_ready_contrat_filaire() {
        let bytes = r#"{"type":"TerminalSessionReady"}"#;
        let decoded: WrapperToDaemon = decode(bytes).unwrap();
        assert!(matches!(decoded, WrapperToDaemon::TerminalSessionReady));
        assert_eq!(encode(&decoded).unwrap(), bytes);
    }

    const SERVICE_NEGOTIATION_FIXTURE: &str =
        include_str!("../../../fixtures/service-negotiation-v1.jsonl");
    const COORDINATION_EVENTS_FIXTURE: &str =
        include_str!("../../../fixtures/coordination-events-v1.jsonl");
    const COORDINATION_STREAM_FIXTURE: &str =
        include_str!("../../../fixtures/coordination-stream-v2.jsonl");

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
            origin: None,
            focus: None,
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
            origin: None,
            focus: None,
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
                    ServiceCapability::GuichetV1,
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
            identity_version: 2,
            agent_type: "codex".to_string(),
            agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
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
                identity_version,
                agent_type,
                agent_id,
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
                assert_eq!(identity_version, 2);
                assert_eq!(agent_type, "codex");
                assert_eq!(agent_id, "550e8400-e29b-41d4-a716-446655440000");
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
    fn register_historique_est_refuse_sans_contrat_identite_v2() {
        let json = r#"{\"type\":\"Register\",\"agent_type\":\"codex\",\"name\":null}"#;
        assert!(decode::<WrapperToDaemon>(json).is_err());
    }

    #[test]
    fn register_v2_conserve_la_distinction_de_canal() {
        let base = "\"type\":\"Register\",\"identity_version\":2,\"agent_type\":\"ui\",\"agent_id\":\"550e8400-e29b-41d4-a716-446655440000\"";
        let omitted: WrapperToDaemon = decode(&format!("{{{base}}}")).unwrap();
        assert!(matches!(
            omitted,
            WrapperToDaemon::Register {
                channel: ChannelReport::Omitted,
                ..
            }
        ));
        let unknown: WrapperToDaemon = decode(&format!("{{{base},\"channel\":null}}")).unwrap();
        assert!(matches!(
            unknown,
            WrapperToDaemon::Register {
                channel: ChannelReport::Unknown,
                ..
            }
        ));
        let known: WrapperToDaemon =
            decode(&format!("{{{base},\"channel\":\"ssh-unix\"}}")).unwrap();
        assert!(
            matches!(known, WrapperToDaemon::Register { channel: ChannelReport::Known(value), .. } if value == "ssh-unix")
        );
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
    fn spec_079_deliver_idempotent_projette_le_contexte_execution_optionnel() {
        let message = BridgetMessage::new("humain", "codex-1", "reprends ce travail");
        let frame = DaemonToWrapper::DeliverIdempotent {
            delivery_id: "delivery-079".to_string(),
            recipient_instance_id: "instance-079".to_string(),
            delivery_generation: 7,
            expires_at: 1_900_000_000,
            message,
            execution: Some(ExecutionDeliveryContext {
                execution_id: "execution-079".to_string(),
                generation: 2,
                revision: 0,
            }),
        };

        let encoded = encode(&frame).unwrap();
        let decoded: DaemonToWrapper = decode(&encoded).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::DeliverIdempotent {
                execution: Some(ExecutionDeliveryContext {
                    execution_id,
                    generation: 2,
                    revision: 0,
                }),
                ..
            } if execution_id == "execution-079"
        ));
    }

    #[test]
    fn spec_079_deliver_idempotent_historique_sans_contexte_reste_decodable() {
        let json = r#"{"type":"DeliverIdempotent","delivery_id":"delivery-old","recipient_instance_id":"instance-old","delivery_generation":1,"expires_at":1900000000,"message":{"id":"message-old","from":"humain","to":"codex-1","body":"historique","reply":false,"references":[],"timestamp":"2026-08-31T00:00:00Z"}}"#;

        let decoded: DaemonToWrapper = decode(json).unwrap();
        assert!(matches!(
            decoded,
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                execution: None,
                ..
            } if delivery_id == "delivery-old"
        ));
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
            provider_kind: Some("claude".to_string()),
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
        let json = r#"{"agent_id":"550e8400-e29b-41d4-a716-446655440002","display_name":"Agent 2","agent_type":"claude","connection_id":"conn-1",
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
        let json = r#"{"agent_id":"550e8400-e29b-41d4-a716-446655440003","display_name":"Lab Agent","agent_type":"codex","connection_id":"conn-1",
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
            "agent_id":"550e8400-e29b-41d4-a716-446655440004","display_name":"Claude","agent_type":"claude","connection_id":"c1",
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
            "agent_id":"550e8400-e29b-41d4-a716-446655440004","display_name":"Claude","agent_type":"claude","connection_id":"c1",
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
            service: "guichet".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![ServiceCapability::GuichetV1],
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
            } if capabilities == vec![ServiceCapability::GuichetV1]
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

        let waiting = WrapperToDaemon::GuichetReply {
            version: SERVICE_CONTRACT_VERSION,
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            request_id: "req-waiting".to_string(),
            claim_generation: 3,
            claim_token: "claim-waiting".to_string(),
            response_message_id: "msg-waiting".to_string(),
            in_reply_to: "message-waiting".to_string(),
            outcome: GuichetOutcome::Accepted,
            payload: GuichetReplyPayload::Delegate {
                status: GuichetDelegateMutationStatus::WaitingForAgent,
                objective_id: Some("objective-waiting".to_string()),
                delegation_id: None,
                message_id: None,
                participant: None,
                candidates: Vec::new(),
                waiting_on_prerequisites: true,
                replayed: false,
            },
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&waiting).unwrap()).unwrap(),
            WrapperToDaemon::GuichetReply {
                payload: GuichetReplyPayload::Delegate {
                    status: GuichetDelegateMutationStatus::WaitingForAgent,
                    objective_id: Some(objective_id),
                    delegation_id: None,
                    waiting_on_prerequisites: true,
                    ..
                },
                ..
            } if objective_id == "objective-waiting"
        ));

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
            service: "guichet".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![
                ServiceCapability::GuichetV1,
                ServiceCapability::CoordinationEventsV1,
            ],
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&hello_with_coordination).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello { capabilities, .. }
                if capabilities == vec![
                    ServiceCapability::GuichetV1,
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
                to: "guichet".to_string(),
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
            host: "poste-alpha".to_string(),
            db_path: "/Users/user/.cache/bridget/bridget.db".to_string(),
            instance_id: "daemon-7f0c01d2".to_string(),
        };
        match decode::<DaemonToWrapper>(&encode(&report).unwrap()).unwrap() {
            DaemonToWrapper::DaemonIdentityReport {
                host,
                db_path,
                instance_id,
            } => {
                assert_eq!(host, "poste-alpha");
                assert_eq!(db_path, "/Users/user/.cache/bridget/bridget.db");
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
            searched_on: "poste-alpha".to_string(),
            requested_from: "poste-beta".to_string(),
        };
        let json = serde_json::to_string(&refusal).expect("sérialisable");
        let decoded: SpawnRefusal = serde_json::from_str(&json).expect("décodable");
        match decoded {
            SpawnRefusal::CwdGone {
                searched_on,
                requested_from,
            } => {
                assert_eq!(searched_on, "poste-alpha");
                assert_eq!(requested_from, "poste-beta");
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
            posture: None,
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
            agent_id: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
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
                agent_id: Some(agent_id),
                command_id,
                ownership: Some(ownership),
                ..
            } if agent_type == "codex"
                && agent_id == "550e8400-e29b-41d4-a716-446655440000"
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
                agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
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
            agent_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
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
                r#"{"type":"SpawnAccepted","command_id":"legacy","agent_id":"550e8400-e29b-41d4-a716-446655440000"}"#
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
        assert!(WrapperToDaemon::ListAgents.attach_refusal().is_none());
        assert!(DaemonToWrapper::AgentList { agents: vec![] }.allowed_for_attach());
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
    fn protocol_007_register_legacy_est_refuse() {
        let json = r#"{\"type\":\"Register\",\"agent_type\":\"codex\",\"name\":null}"#;
        assert!(decode::<WrapperToDaemon>(json).is_err());
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
            policy_id: None,
            policy_version: None,
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
            service: "guichet".to_string(),
            issuer_scope: "065_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: vec![ServiceCapability::ProjectRegistryV1],
        };
        assert!(matches!(
            decode::<WrapperToDaemon>(&encode(&hello).unwrap()).unwrap(),
            WrapperToDaemon::ServiceHello {
                service,
                capabilities,
                ..
            } if service == "guichet" && capabilities == vec![ServiceCapability::ProjectRegistryV1]
        ));

        let conflict = ProjectBindOutcome {
            contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
            command_id: "project-command-2".to_string(),
            project_id: "project-loser".to_string(),
            status: ProjectBindStatus::RegistrationConflict,
            binding_generation: None,
            backend: None,
            runtime_policy: None,
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
                canonical_root: None,
                state: ProjectBindingStatus::Active,
                binding_generation: Some(2),
                backend: Some(ProjectBackend::Host),
                role: ProjectRole::Standard,
                runtime_policy: None,
                reason: None,
                last_audit: None,
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
    #[test]
    fn spec_066_runtime_local_est_versionne_ferme_et_sans_chemin_hote() {
        let request = ProjectRuntimeRequest {
            contract_version: 1,
            command_id: "runtime-command-1".to_string(),
            issued_at: 1_788_000_000,
            deadline_at: 1_788_000_060,
            operation: ProjectRuntimeOperation::Prepare,
            project_id: "project-066".to_string(),
            expected_binding_generation: None,
            policy_id: None,
            policy_version: None,
            profile: None,
        };
        let wire = encode(&WrapperToDaemon::ProjectRuntimeRequest {
            request: request.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"project_runtime_request""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ProjectRuntimeRequest { request: decoded } if decoded == request
        ));
        assert!(serde_json::from_str::<ProjectRuntimeRequest>(
            r#"{"contract_version":1,"command_id":"runtime-command-1","issued_at":1,"deadline_at":2,"operation":"prepare","project_id":"project-066","path":"/srv/private"}"#
        )
        .is_err());

        let outcome = ProjectRuntimeOutcome {
            contract_version: 1,
            command_id: request.command_id.clone(),
            project_id: request.project_id.clone(),
            operation: request.operation,
            binding_generation: Some(2),
            state: Some("ready".to_string()),
            runtime_policy: Some(ProjectRuntimePolicyReference {
                policy_id: "fixture-local".to_string(),
                policy_version: 1,
                policy_digest: format!("sha256:{}", "a".repeat(64)),
                environment_epoch: 3,
            }),
            last_reason: Some("runtime_exec_lost".to_string()),
            reason: None,
            observed_at: 1_788_000_001,
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(
                &encode(&DaemonToWrapper::ProjectRuntimeOutcome {
                    outcome: outcome.clone(),
                })
                .unwrap()
            )
            .unwrap(),
            DaemonToWrapper::ProjectRuntimeOutcome { outcome: decoded } if decoded == outcome
        ));
    }

    #[test]
    fn spec_085_activate_docker_exige_une_reference_fermee_et_reste_versionne() {
        let request = ProjectRuntimeRequest {
            contract_version: PROJECT_RUNTIME_CONTRACT_VERSION,
            command_id: "activate-docker-085".to_string(),
            issued_at: 1,
            deadline_at: 2,
            operation: ProjectRuntimeOperation::ActivateDocker,
            project_id: "project-085".to_string(),
            expected_binding_generation: Some(7),
            policy_id: Some("production-linux-amd64".to_string()),
            policy_version: Some(1),
            profile: None,
        };
        let wire = serde_json::to_string(&request).unwrap();
        assert!(wire.contains("\"operation\":\"activate_docker\""));
        assert!(wire.contains("\"expected_binding_generation\":7"));
        assert!(wire.contains("\"policy_id\":\"production-linux-amd64\""));
        assert!(serde_json::from_str::<ProjectRuntimeRequest>(
            r#"{"contract_version":1,"command_id":"x","issued_at":1,"deadline_at":2,"operation":"activate_docker","project_id":"p","image":"latest"}"#
        ).is_err());
    }
    #[test]
    fn spec_066_handshake_ingress_est_ferme_et_versionne() {
        let hello = RuntimeIngressHandshake {
            contract_version: RUNTIME_INGRESS_CONTRACT_VERSION,
            project_id: "project-066".to_string(),
            binding_generation: 2,
            container_id: "a".repeat(64),
            environment_epoch: 3,
            agent_generation: 4,
            instance_id: "00000000-0000-4000-8000-000000000066".to_string(),
        };
        let wire = encode(&WrapperToDaemon::RuntimeIngressHello {
            hello: hello.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"runtime_ingress_hello""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::RuntimeIngressHello { hello: decoded } if decoded == hello
        ));
        assert!(serde_json::from_str::<RuntimeIngressHandshake>(
            r#"{"contract_version":1,"project_id":"project-066","binding_generation":2,"container_id":"abc","environment_epoch":3,"agent_generation":4,"host_path":"/srv/private"}"#
        )
        .is_err());
        assert_eq!(
            serde_json::to_string(&RuntimeIngressRefusal::EnvironmentEpochStale).unwrap(),
            "\"environment_epoch_stale\""
        );
    }
    #[test]
    fn spec_079_contrat_ronde_projet_est_ferme_versionne_et_rejouable() {
        let request = ProjectRoundRequest {
            contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
            command_id: "round-enable-1".to_string(),
            issued_at: 1_788_000_000,
            deadline_at: 1_788_000_060,
            operation: ProjectRoundOperation::Enable,
            project_id: Some("project-079".to_string()),
            binding_generation: Some(4),
        };
        let wire = encode(&WrapperToDaemon::ProjectRoundRequest {
            request: request.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"project_round_request""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ProjectRoundRequest { request: decoded } if decoded == request
        ));
        assert!(serde_json::from_str::<ProjectRoundRequest>(
            r#"{"contract_version":1,"command_id":"round-enable-1","issued_at":1,"deadline_at":2,"operation":"enable","project_id":"project-079","binding_generation":4,"provider":"codex"}"#
        )
        .is_err());

        let legacy_projection: ProjectRoundProjection = serde_json::from_str(
            r#"{"project_id":"project-079","binding_generation":4,"active":true,"configured":true,"enabled":true,"revision":1,"updated_at":1788000000}"#,
        )
        .unwrap();
        assert_eq!(legacy_projection.last_occurrence_at, None);
        assert_eq!(legacy_projection.last_dispatch_state, None);
        assert_eq!(legacy_projection.last_dispatch_observed_at, None);

        let observed_projection = ProjectRoundProjection {
            last_occurrence_at: Some(1_788_000_000),
            last_dispatch_state: Some(ProjectRoundDispatchState::Deposited),
            last_dispatch_observed_at: Some(1_788_000_001),
            ..legacy_projection
        };
        let projection_wire = serde_json::to_string(&observed_projection).unwrap();
        assert!(projection_wire.contains(r#""last_dispatch_state":"deposited""#));
        assert_eq!(
            serde_json::from_str::<ProjectRoundProjection>(&projection_wire).unwrap(),
            observed_projection
        );

        let dispatch = ProjectRoundDispatchRequest {
            contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
            occurrence_at: 1_788_000_000,
            project: ProjectReference {
                project_id: "project-079".to_string(),
                binding_generation: 4,
            },
        };
        let wire = encode(&WrapperToDaemon::ProjectRoundDispatch {
            request: dispatch.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"project_round_dispatch""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ProjectRoundDispatch { request: decoded } if decoded == dispatch
        ));
        assert!(serde_json::from_str::<ProjectRoundDispatchRequest>(
            r#"{"contract_version":1,"occurrence_at":1788000000,"project":{"project_id":"project-079","binding_generation":4},"cwd":"/srv/private"}"#
        )
        .is_err());

        let outcome = ProjectRoundDispatchOutcome {
            contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
            occurrence_at: dispatch.occurrence_at,
            project: dispatch.project,
            issue: Some(IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_788_604_800,
                delivery_id: Some("delivery-079".to_string()),
            }),
            reason: None,
            observed_at: 1_788_000_001,
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(
                &encode(&DaemonToWrapper::ProjectRoundDispatchOutcome {
                    outcome: outcome.clone(),
                })
                .unwrap()
            )
            .unwrap(),
            DaemonToWrapper::ProjectRoundDispatchOutcome { outcome: decoded }
                if decoded == outcome
        ));
    }

    #[test]
    fn spec_086_role_projet_est_ferme_et_ancien_projection_reste_standard() {
        let legacy: ProjectBindingProjection =
            serde_json::from_str(r#"{"project_id":"legacy","state":"active","observed_at":1}"#)
                .unwrap();
        assert_eq!(legacy.role, ProjectRole::Standard);
        let system = ProjectBindingProjection {
            project_id: "bridget-system".to_string(),
            canonical_root: None,
            state: ProjectBindingStatus::Active,
            binding_generation: Some(1),
            backend: Some(ProjectBackend::Docker),
            role: ProjectRole::BridgetSystem,
            runtime_policy: None,
            reason: None,
            last_audit: None,
            observed_at: 2,
        };
        let wire = serde_json::to_string(&system).unwrap();
        assert!(wire.contains(r#""role":"bridget_system""#));
        assert_eq!(
            serde_json::from_str::<ProjectBindingProjection>(&wire).unwrap(),
            system
        );
    }

    #[test]
    fn spec_086_contrat_systeme_est_separe_du_registre_standard() {
        let request = ProjectSystemRequest {
            contract_version: PROJECT_SYSTEM_CONTRACT_VERSION,
            command_id: "system-declare-086".to_string(),
            issued_at: 1,
            deadline_at: 2,
            operation: ProjectSystemOperation::Declare,
            project_id: "bridget-system".to_string(),
            expected_binding_generation: 3,
            runtime_policy_id: None,
            runtime_policy_version: None,
            expected_setting_generation: None,
            requested_mode: None,
        };
        let wire = encode(&WrapperToDaemon::ProjectSystemRequest {
            request: request.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"project_system_request""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ProjectSystemRequest { request: decoded } if decoded == request
        ));
        assert!(serde_json::from_str::<ProjectSystemRequest>(
            r#"{"contract_version":1,"command_id":"x","issued_at":1,"deadline_at":2,"operation":"declare","project_id":"project","expected_binding_generation":1,"root":"/tmp"}"#,
        )
        .is_err());
    }
}

#[cfg(test)]
mod control_and_inbox_contract_tests {
    use super::*;

    fn control_state_frame() -> ControlStateFrame {
        ControlStateFrame {
            version: CONTROL_STATE_CONTRACT_VERSION,
            generation: 13,
            paused: true,
            paused_since: Some(1_788_400_000),
            paused_by: Some("humain".to_string()),
            pause_reason: Some("revue en cours".to_string()),
            auto_objectives_cap: 5,
            agent_posture: Some(AgentPosture::Complete),
            auto_reassignment: Some(true),
            updated_at: 1_788_400_000,
        }
    }

    #[test]
    fn spec_087_trames_etat_de_controle_font_l_aller_retour() {
        let read = WrapperToDaemon::ControlStateRead {
            version: CONTROL_STATE_CONTRACT_VERSION,
        };
        let encoded = encode(&read).unwrap();
        assert!(
            encoded.contains(r#""type":"control_state_read""#),
            "{encoded}"
        );
        assert!(matches!(
            decode::<WrapperToDaemon>(&encoded).unwrap(),
            WrapperToDaemon::ControlStateRead { version: 1 }
        ));
        let focus_read = WrapperToDaemon::ControlFocusRead {
            version: CONTROL_STATE_CONTRACT_VERSION,
        };
        let encoded = encode(&focus_read).unwrap();
        assert!(
            encoded.contains(r#""type":"control_focus_read""#),
            "{encoded}"
        );
        assert!(matches!(
            decode::<WrapperToDaemon>(&encoded).unwrap(),
            WrapperToDaemon::ControlFocusRead { version: 1 }
        ));
        let focus = ControlFocusFrame {
            objective_id: "objective-1".to_string(),
            goal: "Vérifier le focus".to_string(),
            project_id: "projet-1".to_string(),
            updated_at: 42,
        };
        let publish = WrapperToDaemon::ControlFocusPublish {
            version: CONTROL_STATE_CONTRACT_VERSION,
            focus: Some(focus.clone()),
        };
        let encoded = encode(&publish).unwrap();
        assert!(
            encoded.contains(r#""type":"control_focus_publish""#),
            "{encoded}"
        );
        assert!(matches!(
            decode::<WrapperToDaemon>(&encoded).unwrap(),
            WrapperToDaemon::ControlFocusPublish { focus: Some(decoded), .. } if decoded == focus
        ));
        let set = WrapperToDaemon::ControlStateSet {
            version: CONTROL_STATE_CONTRACT_VERSION,
            command_id: "control-1".to_string(),
            expected_generation: 12,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: Some("revue".to_string()),
            agent_posture: None,
            auto_reassignment: None,
        };
        let encoded = encode(&set).unwrap();
        assert!(
            !encoded.contains("auto_objectives_cap"),
            "champ nul omis : {encoded}"
        );
        let WrapperToDaemon::ControlStateSet { paused, reason, .. } = decode(&encoded).unwrap()
        else {
            panic!("variante attendue");
        };
        assert_eq!(paused, Some(true));
        assert_eq!(reason.as_deref(), Some("revue"));
        let state = DaemonToWrapper::ControlState {
            state: control_state_frame(),
            inbox_open_count: 3,
        };
        let encoded = encode(&state).unwrap();
        assert!(encoded.contains(r#""type":"control_state""#), "{encoded}");
        let DaemonToWrapper::ControlState {
            state,
            inbox_open_count,
        } = decode(&encoded).unwrap()
        else {
            panic!("variante attendue");
        };
        assert_eq!(state, control_state_frame());
        assert_eq!(inbox_open_count, 3);
        let projected = DaemonToWrapper::ControlFocus {
            focus: Some(focus.clone()),
        };
        assert!(matches!(
            decode::<DaemonToWrapper>(&encode(&projected).unwrap()).unwrap(),
            DaemonToWrapper::ControlFocus { focus: Some(decoded) } if decoded == focus
        ));
        let history = DaemonToWrapper::ControlHistory {
            events: vec![ControlEventFrame {
                at: 1,
                actor: "humain".to_string(),
                kind: "pause_on".to_string(),
                reason: None,
                generation_after: 1,
            }],
        };
        assert!(decode::<DaemonToWrapper>(&encode(&history).unwrap()).is_ok());
        assert!(
            decode::<WrapperToDaemon>(
                &encode(&WrapperToDaemon::ControlHistory {
                    version: 1,
                    limit: 20
                })
                .unwrap()
            )
            .is_ok()
        );
        let rejected = DaemonToWrapper::ControlStateRejected {
            reason: ControlStateRefusal::GenerationMismatch { current: 13 },
        };
        let encoded = encode(&rejected).unwrap();
        assert!(
            encoded.contains(r#""kind":"generation_mismatch""#),
            "{encoded}"
        );
    }

    #[test]
    fn spec_087_trames_boite_humaine_font_l_aller_retour() {
        let deposit = WrapperToDaemon::HumanInboxDeposit {
            version: HUMAN_INBOX_CONTRACT_VERSION,
            dedup_key: "chain-exhausted:d1".to_string(),
            kind: HumanInboxKind::ChainExhausted,
            subject: HumanInboxSubject {
                objective_id: Some("o1".to_string()),
                delegation_id: Some("d1".to_string()),
                ..HumanInboxSubject::default()
            },
            context: r#"{"summary":"chaîne épuisée"}"#.to_string(),
            options: vec!["reassign:a1".to_string(), "cancel".to_string()],
        };
        let encoded = encode(&deposit).unwrap();
        assert!(
            encoded.contains(r#""type":"human_inbox_deposit""#),
            "{encoded}"
        );
        assert!(encoded.contains(r#""kind":"chain_exhausted""#), "{encoded}");
        let WrapperToDaemon::HumanInboxDeposit { kind, subject, .. } = decode(&encoded).unwrap()
        else {
            panic!("variante attendue");
        };
        assert_eq!(kind, HumanInboxKind::ChainExhausted);
        assert_eq!(subject.message_id, None);
        let item = HumanInboxItemFrame {
            id: "i1".to_string(),
            dedup_key: "chain-exhausted:d1".to_string(),
            kind: HumanInboxKind::ChainExhausted,
            subject: HumanInboxSubject::default(),
            context: "{}".to_string(),
            options: vec!["ack".to_string()],
            state: HumanInboxState::Resolved,
            producer: HumanInboxProducer::Guichet,
            created_at: 1,
            resolved_at: Some(2),
            occurrences: 2,
            decision: Some(HumanInboxDecision {
                decision_id: "dec-1".to_string(),
                choice: "ack".to_string(),
                actor: "humain".to_string(),
                at: 2,
            }),
            acked_at: None,
        };
        let frame = DaemonToWrapper::HumanInbox {
            items: vec![item.clone()],
            open_count: 0,
        };
        let DaemonToWrapper::HumanInbox { items, open_count } =
            decode(&encode(&frame).unwrap()).unwrap()
        else {
            panic!("variante attendue");
        };
        assert_eq!(items, vec![item]);
        assert_eq!(open_count, 0);
        for frame in [
            WrapperToDaemon::HumanInboxList {
                version: 1,
                state: HumanInboxListFilter::Open,
                limit: 50,
            },
            WrapperToDaemon::HumanInboxResolve {
                version: 1,
                command_id: "c".to_string(),
                item_id: "i1".to_string(),
                choice: "ack".to_string(),
            },
            WrapperToDaemon::HumanInboxDecisions {
                version: 1,
                limit: 20,
            },
            WrapperToDaemon::HumanInboxAck {
                version: 1,
                decision_id: "dec-1".to_string(),
            },
            WrapperToDaemon::HumanInboxClose {
                version: 1,
                item_id: "i1".to_string(),
                reason: "object_vanished".to_string(),
            },
        ] {
            let encoded = encode(&frame).unwrap();
            assert!(decode::<WrapperToDaemon>(&encoded).is_ok(), "{encoded}");
        }
        let batch = DaemonToWrapper::HumanInboxDecisionsBatch {
            decisions: vec![HumanInboxPendingDecision {
                decision_id: "dec-1".to_string(),
                item_id: "i1".to_string(),
                dedup_key: "k".to_string(),
                kind: HumanInboxKind::BudgetReached,
                subject: HumanInboxSubject::default(),
                choice: "raise_budget".to_string(),
                at: 3,
            }],
        };
        assert!(decode::<DaemonToWrapper>(&encode(&batch).unwrap()).is_ok());
        let rejected = DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::ChoiceNotOffered,
        };
        assert!(encode(&rejected).unwrap().contains("choice_not_offered"));
    }

    #[test]
    fn spec_087_kind_sql_derive_d_une_seule_source() {
        // Chaque variante de ALL rend un libellé unique, égal à son nom de fil.
        let mut seen = std::collections::BTreeSet::new();
        for kind in HumanInboxKind::ALL {
            let wire = serde_json::to_string(&kind).unwrap();
            assert_eq!(
                wire,
                format!("\"{}\"", kind.as_sql()),
                "fil ≠ SQL pour {kind:?}"
            );
            assert!(seen.insert(kind.as_sql()), "doublon {kind:?}");
            assert_eq!(HumanInboxKind::from_sql(kind.as_sql()), Some(kind));
        }
        assert_eq!(
            HumanInboxKind::sql_in_clause(),
            "IN ('intervention_required', 'chain_exhausted', 'review_verdict_pending', 'activation_approval', 'budget_reached', 'reply_debt', 'focus_waiting_agent', 'object_vanished', 'human_route_replaced')"
        );
        assert_eq!(HumanInboxKind::from_sql("inconnu"), None);
    }

    #[test]
    fn spec_087_delegate_avec_origine_ou_focus_exige_la_version_revue() {
        let base = ServiceRequestPayload::Delegate {
            goal: "but".to_string(),
            review_target: None,
            explicit_target: None,
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Normale,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
            origin: None,
            focus: None,
        };
        assert_eq!(base.required_contract_version(), SERVICE_CONTRACT_VERSION);
        let encoded = serde_json::to_string(&base).unwrap();
        assert!(
            !encoded.contains("origin") && !encoded.contains("focus"),
            "{encoded}"
        );
        let legacy: ServiceRequestPayload = serde_json::from_str(
            r#"{"goal":"but","duration":"normale","suite":{"kind":"aucune"}}"#,
        )
        .unwrap();
        assert_eq!(legacy, base);
        let origin = Some(DelegateOrigin::Human {
            message_id: "m1".to_string(),
            observed: ObservedHumanMessageFrame {
                message_id: "m1".to_string(),
                ts: 1,
                sender: "humain".to_string(),
                target: "guichet".to_string(),
                body: "Travaille sur X".to_string(),
            },
            attestation: HumanOriginAttestationFrame {
                version: 1,
                issuer_scope: "bridget-ui".to_string(),
                canonical_request_sha256: "0".repeat(64),
                signature: "1".repeat(64),
            },
        });
        let focus = Some(DelegateFocus {
            project_id: "p1".to_string(),
            on_conflict: Some(FocusConflictPolicy::Queue),
        });
        let with_origin = ServiceRequestPayload::Delegate {
            goal: "but".to_string(),
            review_target: None,
            explicit_target: None,
            required_tags: Vec::new(),
            duration: GuichetDurationClass::Normale,
            suite: ServiceSuiteDeclaration::Aucune,
            depends_on: Vec::new(),
            references: Vec::new(),
            origin,
            focus,
        };
        assert_eq!(
            with_origin.required_contract_version(),
            REVIEW_DELEGATE_CONTRACT_VERSION
        );
        let encoded = serde_json::to_string(&with_origin).unwrap();
        assert!(encoded.contains(r#""kind":"human""#), "{encoded}");
        assert!(encoded.contains(r#""on_conflict":"queue""#), "{encoded}");
        let decoded: ServiceRequestPayload = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, with_origin);
    }
}

#[cfg(test)]
mod spec102_thread_contract_tests {
    use super::{
        DaemonToWrapper, THREAD_CONTRACT_VERSION, ThreadAction, ThreadNotify, ThreadNotifyAll,
        ThreadRequest, ThreadResult, WrapperToDaemon, decode, encode,
    };

    fn action(json: &str) -> Result<ThreadAction, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn spec102_v33_huit_actions_fermees_et_champs_inconnus_refuses() {
        let create =
            action(r#"{"action":"create","title":"T","members":["a","b"],"operation_id":"op"}"#)
                .unwrap();
        assert_eq!(create.name(), "create");
        assert_eq!(action(r#"{"action":"list"}"#).unwrap().name(), "list");
        assert_eq!(
            action(r#"{"action":"show","thread_id":"t"}"#)
                .unwrap()
                .name(),
            "show"
        );
        assert_eq!(
            action(r#"{"action":"read","thread_id":"t","limit":5}"#)
                .unwrap()
                .name(),
            "read"
        );
        assert_eq!(
            action(r#"{"action":"ack","thread_id":"t","receipt":"r"}"#)
                .unwrap()
                .name(),
            "ack"
        );
        assert_eq!(
            action(r#"{"action":"history","thread_id":"t","from_seq":1,"to_seq":9}"#)
                .unwrap()
                .name(),
            "history"
        );
        assert_eq!(
            action(r#"{"action":"close","thread_id":"t","operation_id":"op"}"#)
                .unwrap()
                .name(),
            "close"
        );
        let post = action(
            r#"{"action":"post","thread_id":"t","body":"b","notify":[],"operation_id":"op"}"#,
        )
        .unwrap();
        assert!(
            matches!(post, ThreadAction::Post { notify: ThreadNotify::Targets(ref t), .. } if t.is_empty())
        );
        // Champ inconnu, action inconnue, notify manquant, paramètre d'acteur : refusés avant toute mutation.
        assert!(action(r#"{"action":"read","thread_id":"t","actor":"x"}"#).is_err());
        assert!(action(r#"{"action":"summary","thread_id":"t"}"#).is_err());
        assert!(
            action(r#"{"action":"post","thread_id":"t","body":"b","operation_id":"op"}"#).is_err()
        );
        assert!(action(r#"{"action":"post","thread_id":"t","body":"b","notify":"everyone","operation_id":"op"}"#).is_err());
    }

    #[test]
    fn spec102_v05_notify_all_est_une_chaine_explicite() {
        let all = action(
            r#"{"action":"post","thread_id":"t","body":"b","notify":"all","operation_id":"op"}"#,
        )
        .unwrap();
        assert!(matches!(
            all,
            ThreadAction::Post {
                notify: ThreadNotify::All(ThreadNotifyAll::All),
                ..
            }
        ));
        let targets = action(r#"{"action":"post","thread_id":"t","body":"@all","notify":["b","b"],"operation_id":"op"}"#).unwrap();
        assert!(
            matches!(targets, ThreadAction::Post { notify: ThreadNotify::Targets(ref t), .. } if t.len() == 2)
        );
        assert_eq!(
            serde_json::to_value(ThreadNotify::All(ThreadNotifyAll::All)).unwrap(),
            "all"
        );
    }

    #[test]
    fn spec102_v22_enveloppe_versionnee_et_variantes_de_protocole() {
        let request = ThreadRequest {
            version: THREAD_CONTRACT_VERSION,
            request: ThreadAction::Show {
                thread_id: "t".into(),
            },
        };
        let wire = encode(&WrapperToDaemon::ThreadRequest {
            request: request.clone(),
        })
        .unwrap();
        assert!(wire.contains(r#""type":"ThreadRequest""#));
        assert!(matches!(
            decode::<WrapperToDaemon>(&wire).unwrap(),
            WrapperToDaemon::ThreadRequest { request: decoded } if decoded == request
        ));
        assert!(
            serde_json::from_str::<ThreadRequest>(
                r#"{"version":1,"request":{"action":"show","thread_id":"t"},"actor":"x"}"#
            )
            .is_err()
        );
        let result = DaemonToWrapper::ThreadResult {
            result: ThreadResult {
                version: 1,
                result: serde_json::json!({"status":"error","code":"thread_unavailable"}),
            },
        };
        let wire = encode(&result).unwrap();
        assert!(matches!(
            decode::<DaemonToWrapper>(&wire).unwrap(),
            DaemonToWrapper::ThreadResult { result } if result.result["status"] == "error"
        ));
    }

    #[test]
    fn spec102_v27_capacite_d_alerte_annoncee_apres_enregistrement() {
        // Ancien wrapper : aucune annonce, donc aucune alerte ; le daemon ancien
        // qui ignorerait la variante ne casse pas la sérialisation des DM.
        let announced =
            encode(&WrapperToDaemon::ThreadNoticeCapability { versions: vec![1] }).unwrap();
        assert!(matches!(
            decode::<WrapperToDaemon>(&announced).unwrap(),
            WrapperToDaemon::ThreadNoticeCapability { versions } if versions == vec![1]
        ));
        let legacy_register = r#"{"type":"Register","identity_version":2,"agent_type":"codex","agent_id":"10200000-0000-4000-8000-000000000001"}"#;
        assert!(matches!(
            decode::<WrapperToDaemon>(legacy_register).unwrap(),
            WrapperToDaemon::Register { .. }
        ));
        let legacy_registered = r#"{"type":"Registered","agent_id":"a"}"#;
        assert!(matches!(
            decode::<DaemonToWrapper>(legacy_registered).unwrap(),
            DaemonToWrapper::Registered { .. }
        ));
        assert!(
            WrapperToDaemon::ThreadRequest {
                request: ThreadRequest {
                    version: 1,
                    request: ThreadAction::List {
                        limit: None,
                        after_thread_id: None
                    }
                }
            }
            .attach_refusal()
            .is_some(),
            "le rôle attach ne peut pas manipuler les fils"
        );
    }
}

#[cfg(test)]
mod spec104_search_contract_tests {
    use super::*;

    #[test]
    fn spec104_requete_de_recherche_stricte_et_sans_identite() {
        let request: LedgerSearchRequest = serde_json::from_value(serde_json::json!({
            "query": "pagination erreur",
            "peer": "11111111-1111-4111-8111-111111111111",
            "limit": 20
        }))
        .unwrap();
        assert_eq!(request.source, LedgerSearchSource::Messages);
        assert_eq!(request.limit, Some(20));
        assert!(request.cursor.is_none());
        // Aucun champ d'identité, d'instance ou de chemin n'est accepté.
        for forbidden in [
            "agent_id",
            "acting_agent",
            "instance_id",
            "db_path",
            "scope",
            "project_id",
        ] {
            let value = serde_json::json!({"query": "x", forbidden: "y"});
            assert!(
                serde_json::from_value::<LedgerSearchRequest>(value).is_err(),
                "champ {forbidden} accepté"
            );
        }
        // Types stricts : booléen ou flottant refusés pour since/limit.
        assert!(
            serde_json::from_value::<LedgerSearchRequest>(
                serde_json::json!({"query":"x","since":true})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<LedgerSearchRequest>(
                serde_json::json!({"query":"x","limit":1.5})
            )
            .is_err()
        );
    }

    #[test]
    fn spec104_trames_serialisees_par_nom_et_resultats_types() {
        let frame = WrapperToDaemon::LedgerSearch {
            request: LedgerSearchRequest {
                source: LedgerSearchSource::Thread,
                query: "décision".into(),
                author: None,
                peer: None,
                since: None,
                until: None,
                limit: None,
                cursor: None,
                thread_id: Some("22222222-2222-4222-8222-222222222222".into()),
            },
        };
        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["type"], "ledger_search");
        assert_eq!(json["request"]["source"], "thread");
        assert!(json["request"].get("peer").is_none());
        let back: WrapperToDaemon = serde_json::from_value(json).unwrap();
        assert!(matches!(back, WrapperToDaemon::LedgerSearch { .. }));

        let outcome = LedgerSearchOutcomeV1::Ok(LedgerSearchPage {
            source: LedgerSearchSource::Messages,
            hits: vec![LedgerSearchHit::Message {
                id: "m1".into(),
                target: "t".into(),
                sender: "s".into(),
                ts: 7,
                excerpt: "…".into(),
                match_offset: 3,
                body_digest: "ab".into(),
                body_bytes: 12,
            }],
            has_more: true,
            next_cursor: Some("00".into()),
            scanned_count: 1,
            scanned_bytes: 12,
            skipped_oversized: 0,
            stop_reason: "result_limit".into(),
            consistency: "live_bounded".into(),
            notices: vec![],
        });
        let json = serde_json::to_value(DaemonToWrapper::LedgerSearchResult { outcome }).unwrap();
        assert_eq!(json["type"], "ledger_search_result");
        assert_eq!(json["outcome"]["status"], "ok");
        assert_eq!(json["outcome"]["hits"][0]["kind"], "message");
        assert_eq!(json["outcome"]["hits"][0]["match_offset"], 3);

        let refus = serde_json::to_value(LedgerReadOutcomeV1::Error {
            code: "not_found_or_forbidden".into(),
            reason: "source indisponible".into(),
        })
        .unwrap();
        assert_eq!(
            refus,
            serde_json::json!({"status":"error","code":"not_found_or_forbidden","reason":"source indisponible"})
        );

        let read: LedgerReadRequest =
            serde_json::from_value(serde_json::json!({"id":"handoff-pagination-01","target":"t"}))
                .unwrap();
        assert_eq!(read.offset, 0);
        assert!(read.digest.is_none());
        assert!(
            serde_json::from_value::<LedgerReadRequest>(
                serde_json::json!({"id":"a","target":"b","offset":-1})
            )
            .is_err()
        );
    }

    #[test]
    fn spec104_ancien_contrat_ledger_inchange() {
        // Un client ou un daemon antérieur continue d'échanger la projection
        // historique : trame et champs identiques à la version précédente.
        let request = serde_json::to_value(WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Messages,
            limit: 20,
        })
        .unwrap();
        assert_eq!(
            request,
            serde_json::json!({"type":"LedgerProjection","scope":"Messages","limit":20})
        );
        // Un ancien daemon ne connaît pas la trame 104 : le client doit obtenir
        // une erreur explicite, jamais un repli. Ici : la trame reste
        // syntaxiquement distincte de toute trame historique.
        let unknown: Result<WrapperToDaemon, _> =
            serde_json::from_str(r#"{"type":"ledger_search_v2","request":{"query":"x"}}"#);
        assert!(unknown.is_err());
    }
}
