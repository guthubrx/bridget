//! Contrats de protocole du registre de projets et du projet système Bridget.

use serde::{Deserialize, Serialize};

/// Mode d'exécution d'une liaison de projet.
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
