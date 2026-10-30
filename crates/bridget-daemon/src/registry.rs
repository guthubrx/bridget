//! Registre déclaratif des types d'agents lancés par Bridget.

use bridget_transport::protocol::{
    IdentityCredential, RoutingCatalogueOptionV1, RoutingCatalogueV1, RoutingEntitlement,
};
use bridget_transport::{
    AdapterCapabilities, ModelCapabilities, ResolvedAgentDefinition, ResolvedMcpDefinition,
    SpawnRefusal,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const DEFAULT_QUEUE_CAPACITY: usize = 32;
/// Plafond par défaut d'un tour sans `deadline_at` absolue.
/// 600 s condamnait toute revue qui compile un workspace ; 2700 s couvre
/// aussi les types absents de `agents.json` (codex via définition native).
pub const DEFAULT_NOTIFY_TIMEOUT_SECS: u64 = 2700;
// launchd démarre le daemon avec un PATH minimal : les pilotes embarqués ne
// doivent pas dépendre de la configuration interactive de l'utilisateur.
const NATIVE_CODEX_COMMAND: &str = "/opt/homebrew/bin/codex";
const MAX_PASS_ENV_ENTRIES: usize = 64;
const MAX_ENV_NAME_BYTES: usize = 128;
const MAX_CAPABILITY_VALUE_CHARS: usize = 100;
const MAX_MACHINES: usize = 16;
const MAX_MACHINE_MAPPINGS: usize = 64;
const MAX_MACHINE_REVISION: u64 = (1 << 53) - 1;
const MAX_ADMISSION_PROFILES: usize = 8;
const MAX_ADMISSION_PREFERENCES: usize = 8;
const ROUTING_POLICY_VERSION: u16 = 2;
const MAX_ROUTING_CANDIDATES: usize = 128;
const DEFAULT_ROUTING_TELEMETRY_MAX_AGE_SECS: u64 = 30;
const DEFAULT_ROUTING_DECISION_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_ROUTING_SERVICE_TIMEOUT_MS: u64 = 2_000;
const MAX_DECISION_ENDPOINT_BYTES: usize = 2048;
const MAX_PROFILE_DESCRIPTION_BYTES: usize = 32;
const MAX_PROFILE_VALUE_BYTES: usize = 256;
const DEFAULT_DECISION_HEALTH_SECONDS: u64 = 30;
const MAX_DECISION_HEALTH_SECONDS: u64 = 300;
/// Les coordinateurs de découverte ne réemploient jamais une définition
/// d'agent ordinaire telle quelle : celle-ci peut volontairement avoir des
/// permissions larges pour un autre usage. Le préfixe rend la génération
/// durable lisible dans la flotte sans permettre de confondre les deux rôles.
const PROJECT_DISCOVERY_AGENT_PREFIX: &str = "project-discovery-";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct McpDefinition {
    #[serde(default = "default_interactive_mcp")]
    pub interactive: String,
    #[serde(default)]
    pub acp_session: bool,
}

impl Default for McpDefinition {
    fn default() -> Self {
        Self {
            interactive: default_interactive_mcp(),
            acp_session: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AgentDefinition {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_protocol")]
    pub protocol: String,
    #[serde(default)]
    pub forbidden_env: Vec<String>,
    /// Variables supplémentaires recopiées depuis l'environnement source du
    /// daemon après application de la garde de facturation.
    #[serde(default)]
    pub pass_env: Vec<String>,
    /// Répertoire de profil Claude Code, non secret, injecté par Bridget.
    /// Les secrets restent dans settings.json avec droits stricts hors du registre.
    #[serde(default)]
    pub claude_config_dir: Option<String>,
    #[serde(default = "default_permissions")]
    pub permissions: String,
    #[serde(default = "default_queue_capacity")]
    pub queue_capacity: usize,
    /// File retenue par le daemon, fermée pour tout registre antérieur.
    #[serde(default)]
    pub held_queue: bool,
    #[serde(default = "default_notify_timeout_secs")]
    pub notify_timeout_secs: u64,
    /// Branchement MCP éphémère, déclaré par type et jamais par une
    /// configuration utilisateur persistante.
    #[serde(default)]
    pub mcp: McpDefinition,
    /// Matrice déclarative qui autorise un lancement sans interroger le
    /// pilote. Son contenu est figé avec le reste de la définition résolue.
    #[serde(default = "default_capabilities")]
    pub capabilities: AdapterCapabilities,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AgentRegistryFile {
    #[serde(default)]
    pub agents: BTreeMap<String, AgentDefinition>,
    #[serde(default)]
    pub machines: BTreeMap<String, MachineProvision>,
    /// La racine reste volontairement tolérante pour les clés historiques.
    /// Ces deux nouvelles sections, elles, ont un schéma fermé.
    #[serde(default)]
    pub decision_services: BTreeMap<String, DecisionService>,
    #[serde(default)]
    pub admission_policy: Option<AdmissionPolicy>,
    /// Politique168 additive. Son absence conserve intégralement le parcours
    /// d'admission163 ; le registre ne démarre aucun moteur en la lisant.
    #[serde(default)]
    pub routing_policy: Option<RoutingPolicy>,
    /// Choix des futures admissions seulement ; validé au moment de la sélection.
    /// La racine tolérante permet aux anciens lecteurs de l'ignorer.
    #[serde(default)]
    pub project_preferences: serde_json::Value,
}

/// Déclaration opaque d'un environnement déjà authentifié. Les secrets et
/// chemins d'authentification restent exclusivement dans le lanceur existant.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingConnection {
    pub connection_id: String,
    pub machine_id: String,
    pub engine: String,
    pub agent_type: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Provenance explicitement déclarée d'un modèle. Elle décrit une observation
/// ou une déclaration, jamais une entitlement fournisseur.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingModelProvenance {
    Declared,
    CodexModelList,
    OperatorObservation,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingModel {
    pub model: String,
    #[serde(default)]
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
    pub provenance: RoutingModelProvenance,
    #[serde(default)]
    pub observed_at: Option<i64>,
}

/// Les cinq axes visibles restent indépendants. Les champs fixes ne sont
/// utilisés que lorsque l'axe correspondant est désactivé.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingAutomaticAxes {
    #[serde(default = "default_routing_automatic_axis")]
    pub engine: bool,
    #[serde(default = "default_routing_automatic_axis")]
    pub model: bool,
    #[serde(default = "default_routing_automatic_axis")]
    pub machine: bool,
    #[serde(default = "default_routing_automatic_axis")]
    pub connection: bool,
    #[serde(default = "default_routing_automatic_axis")]
    pub effort: bool,
}

impl Default for RoutingAutomaticAxes {
    fn default() -> Self {
        Self {
            engine: true,
            model: true,
            machine: true,
            connection: true,
            effort: true,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingFixedValues {
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
}

/// Poids168 bornés. La pénalité de saturation reste fixée à100 hors réglage.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct RoutingScoreWeights {
    pub cpu: u8,
    pub ram: u8,
    pub load: u8,
    pub continuity: u8,
    pub compilation: u8,
    pub quota: u8,
    pub reset: u8,
}

impl Default for RoutingScoreWeights {
    fn default() -> Self {
        Self {
            cpu: 20,
            ram: 20,
            load: 20,
            continuity: 20,
            compilation: 10,
            quota: 5,
            reset: 3,
        }
    }
}

impl RoutingScoreWeights {
    // Les anciens snapshots168 omettaient les poids par défaut. Les réécrire
    // changerait leur digest lors de la reprise ; seuls les écarts sont figés.
    fn is_default(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingTelemetryPolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_routing_telemetry_max_age_secs")]
    pub max_age_secs: u64,
    #[serde(default = "default_routing_cpu_saturation_percent")]
    pub cpu_saturation_percent: u8,
    #[serde(default = "default_routing_memory_min_percent")]
    pub memory_available_min_percent: u8,
    #[serde(default = "default_routing_load_saturation_percent")]
    pub normalized_load_saturation_percent: u8,
    #[serde(default, skip_serializing_if = "RoutingScoreWeights::is_default")]
    pub weights: RoutingScoreWeights,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingDecisionServicePolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub service_id: Option<String>,
    #[serde(default)]
    pub accessible: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingLightweightDecisionPolicy {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub accessible: bool,
    #[serde(default)]
    pub engine: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub effort: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingMachinePolicy {
    #[serde(default)]
    pub enabled: bool,
    /// Capacité de compilation déclarée par l'opérateur. `None` reste une
    /// inconnue : le registre ne déduit jamais une vitesse du matériel.
    #[serde(default)]
    pub compilation_capacity: Option<u8>,
    #[serde(default)]
    pub automatic_axes: RoutingAutomaticAxes,
    #[serde(default)]
    pub fixed_values: RoutingFixedValues,
    #[serde(default)]
    pub telemetry: Option<RoutingTelemetryPolicy>,
    #[serde(default)]
    pub decision_service: Option<RoutingDecisionServicePolicy>,
    #[serde(default)]
    pub lightweight_decision: Option<RoutingLightweightDecisionPolicy>,
    #[serde(default = "default_routing_decision_timeout_ms")]
    pub decision_timeout_ms: u64,
    #[serde(default = "default_routing_service_timeout_ms")]
    pub service_timeout_ms: u64,
}

/// Politique168 déclarative. Le catalogue est indexé par connexion afin de ne
/// jamais transférer un modèle ou une observation entre deux environnements.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingPolicy {
    pub version: u16,
    pub revision: u64,
    #[serde(default)]
    pub enabled: bool,
    pub connections: Vec<RoutingConnection>,
    pub model_catalog: BTreeMap<String, Vec<RoutingModel>>,
    pub machine_policies: BTreeMap<String, RoutingMachinePolicy>,
}

/// Produit déclaré connexion×modèle. Il est construit en mémoire depuis le
/// registre déjà lu ; aucune découverte, authentification ou entitlement ne
/// participe à cette projection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutingCandidate {
    pub machine_id: String,
    pub engine: String,
    pub connection_id: String,
    pub agent_type: String,
    pub enabled: bool,
    pub model: String,
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
    pub provenance: RoutingModelProvenance,
    pub observed_at: Option<i64>,
}

/// Contraintes demandées pour les cinq axes168. `None` laisse l'axe
/// automatique, sauf lorsqu'une politique le fixe explicitement.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoutingCandidateFilters {
    pub engine: Option<String>,
    pub model: Option<String>,
    pub machine_id: Option<String>,
    pub connection_id: Option<String>,
    pub effort: Option<String>,
}

/// Candidat après application des contraintes de demande et des axes fixes.
/// `imposed_effort` est distinct du défaut déclaré du modèle : `None` laisse
/// le consommateur employer ce défaut attesté, y compris le défaut natif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutingCandidateSelection {
    pub candidate: RoutingCandidate,
    pub imposed_effort: Option<String>,
}

/// Endpoint de décision explicitement administré. Son hébergement ne choisit
/// jamais une machine d'exécution et une mission ne peut pas le remplacer.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecisionService {
    #[serde(default)]
    pub enabled: bool,
    pub machine_id: String,
    pub protocol: String,
    /// Checkpoint de décision attendu. Il est envoyé au service et la réponse
    /// doit le réattester dans `routing.model`; le modèle racine est seulement
    /// informatif et n'autorise aucun choix.
    pub model: String,
    pub endpoint: String,
    #[serde(default = "default_decision_health_seconds")]
    pub health_ttl_seconds: u64,
    #[serde(default = "default_decision_health_seconds")]
    pub retry_after_seconds: u64,
}

/// Un profil borne le catalogue avant toute lecture de disponibilité : il ne
/// représente donc ni une autorisation ni un candidat déjà admis.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CandidateProfile {
    pub id: String,
    /// "local" ou l'identifiant d'une machine provisionnée. Cette référence
    /// est vérifiée au moment de l'admission, car le registre peut évoluer.
    pub machine_id: String,
    pub agent_type: String,
    pub model: String,
    #[serde(default)]
    pub effort: Option<String>,
    pub description: String,
}

/// Politique d'admission automatique, rechargée pour une nouvelle admission
/// puis figée dans son reçu. Absente, elle désactive tout automatisme.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdmissionPolicy {
    pub revision: u64,
    #[serde(default)]
    pub auto_engine: bool,
    #[serde(default)]
    pub auto_placement: bool,
    #[serde(default)]
    pub service_id: Option<String>,
    pub profiles: Vec<CandidateProfile>,
    #[serde(default)]
    pub fixed_profile: Option<String>,
    #[serde(default)]
    pub fallback_profile: Option<String>,
    #[serde(default)]
    pub preferences: Vec<String>,
}

/// Autorité machine locale au maître, jamais dans une définition fournisseur figée.
/// Les clés de `machines` et `mappings` portent machine_id et project_id.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MachineProvision {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub display_name: Option<String>,
    pub worker_id: String,
    pub provisioning_revision: u64,
    pub credential_fingerprint: String,
    #[serde(default)]
    pub allowed_agent_types: Vec<String>,
    #[serde(default)]
    pub mappings: BTreeMap<String, ProjectMapping>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProjectMapping {
    pub source_host: String,
    pub source_root: PathBuf,
    pub remote_root: PathBuf,
    pub expected_git_identity: String,
    pub mapping_revision: u64,
    #[serde(default)]
    pub additional_root: Option<AdditionalDirectoryMapping171>,
}

impl ProjectMapping {
    /// Une commande sur la machine cible utilise les racines déjà provisionnées
    /// comme origine. Cela ne crée ni nouvelle correspondance ni droit.
    pub(crate) fn for_origin(&self, machine: &str, origin: &str) -> Option<Self> {
        if self.source_host == origin { return Some(self.clone()); }
        if machine != origin { return None; }
        let mut mapping = self.clone();
        mapping.source_host = origin.to_owned();
        mapping.source_root = mapping.remote_root.clone();
        if let Some(extra) = &mut mapping.additional_root {
            extra.source_root = extra.remote_root.clone();
            extra.source_uid = extra.remote_uid;
            extra.source_mode = extra.remote_mode;
        }
        Some(mapping)
    }
}

/// One provisioned directory alias, not a content-transfer or permission grant.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AdditionalDirectoryMapping171 {
    pub source_root: PathBuf,
    pub remote_root: PathBuf,
    pub source_uid: u32,
    pub source_mode: u32,
    pub remote_uid: u32,
    pub remote_mode: u32,
    pub access: String,
}

pub(crate) fn validate_additional_mapping171(mapping: &ProjectMapping) -> Result<(), String> {
    let Some(extra) = &mapping.additional_root else { return Ok(()); };
    let overlaps = |a: &Path, b: &Path| a.starts_with(b) || b.starts_with(a);
    if !mapping_root(&extra.source_root) || !mapping_root(&extra.remote_root)
        || overlaps(&extra.source_root, &mapping.source_root)
        || overlaps(&extra.remote_root, &mapping.remote_root)
        || extra.access != "inherit" || extra.source_mode > 0o777 || extra.remote_mode != 0o700
        || extra.source_mode & 0o700 != 0o700
    { return Err("project_mapping_invalid".into()); }
    Ok(())
}

impl MachineProvision {
    pub fn permits_agent_type(&self, agent_type: &str) -> bool {
        self.enabled
            && self
                .allowed_agent_types
                .iter()
                .any(|value| value == agent_type)
    }

    /// Authentification seule : enabled contrôle les nouvelles admissions.
    /// Les commandes existantes restent contrôlables après désactivation.
    pub fn matches_credential(&self, credential: &IdentityCredential) -> bool {
        if credential.expose_for_attestation().is_empty() {
            return false;
        }
        let digest = Sha256::digest(credential.expose_for_attestation().as_bytes());
        self.credential_fingerprint == format!("{digest:x}")
    }
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: BTreeMap<String, AgentDefinition>,
    machines: BTreeMap<String, MachineProvision>,
    decision_services: BTreeMap<String, DecisionService>,
    admission_policy: Option<AdmissionPolicy>,
    routing_policy: Option<RoutingPolicy>,
    project_preferences: serde_json::Value,
    source: PathBuf,
}

fn default_protocol() -> String {
    "acp".to_string()
}
fn default_permissions() -> String {
    "allow".to_string()
}
fn default_queue_capacity() -> usize {
    DEFAULT_QUEUE_CAPACITY
}
fn default_notify_timeout_secs() -> u64 {
    DEFAULT_NOTIFY_TIMEOUT_SECS
}
fn default_interactive_mcp() -> String {
    "none".to_string()
}
fn default_decision_health_seconds() -> u64 {
    DEFAULT_DECISION_HEALTH_SECONDS
}
fn default_routing_automatic_axis() -> bool {
    true
}
fn default_routing_telemetry_max_age_secs() -> u64 {
    DEFAULT_ROUTING_TELEMETRY_MAX_AGE_SECS
}
fn default_routing_cpu_saturation_percent() -> u8 {
    90
}
fn default_routing_memory_min_percent() -> u8 {
    10
}
fn default_routing_load_saturation_percent() -> u8 {
    100
}
fn default_routing_decision_timeout_ms() -> u64 {
    DEFAULT_ROUTING_DECISION_TIMEOUT_MS
}
fn default_routing_service_timeout_ms() -> u64 {
    DEFAULT_ROUTING_SERVICE_TIMEOUT_MS
}

fn routing_model_provenance_name(provenance: RoutingModelProvenance) -> &'static str {
    match provenance {
        RoutingModelProvenance::Declared => "declared",
        RoutingModelProvenance::CodexModelList => "codex_model_list",
        RoutingModelProvenance::OperatorObservation => "operator_observation",
    }
}

fn requested_or_fixed(
    requested: &Option<String>,
    automatic: bool,
    fixed: &Option<String>,
) -> Result<Option<String>, String> {
    if requested.is_some() {
        return Ok(requested.clone());
    }
    if automatic {
        return Ok(None);
    }
    fixed
        .clone()
        .map(Some)
        .ok_or("routing_fixed_axis_missing".into())
}

fn effective_routing_filters(
    requested: &RoutingCandidateFilters,
    policy: &RoutingMachinePolicy,
) -> Result<RoutingCandidateFilters, String> {
    Ok(RoutingCandidateFilters {
        engine: requested_or_fixed(
            &requested.engine,
            policy.automatic_axes.engine,
            &policy.fixed_values.engine,
        )?,
        model: requested_or_fixed(
            &requested.model,
            policy.automatic_axes.model,
            &policy.fixed_values.model,
        )?,
        machine_id: requested_or_fixed(
            &requested.machine_id,
            policy.automatic_axes.machine,
            &policy.fixed_values.machine_id,
        )?,
        connection_id: requested_or_fixed(
            &requested.connection_id,
            policy.automatic_axes.connection,
            &policy.fixed_values.connection_id,
        )?,
        effort: requested_or_fixed(
            &requested.effort,
            policy.automatic_axes.effort,
            &policy.fixed_values.effort,
        )?,
    })
}

fn default_capabilities() -> AdapterCapabilities {
    // Compatibilité des registres historiques : ACP était déjà le seul chemin
    // réellement lancé par ce daemon. Un modèle explicite, lui, reste refusé
    // tant qu'il n'est pas déclaré dans la matrice.
    AdapterCapabilities::default()
}

impl AgentRegistry {
    pub fn load() -> Result<Self, String> {
        Self::load_from_path(config_path()?)
    }

    pub(crate) fn load_from_path(source: PathBuf) -> Result<Self, String> {
        Self::load_from_path_observed(source, true)
    }
    pub(crate) fn load_from_path_quiet(source: PathBuf) -> Result<Self, String> {
        Self::load_from_path_observed(source, false)
    }
    fn load_from_path_observed(source: PathBuf, emit_warnings: bool) -> Result<Self, String> {
        let mut agents = default_agents()?;
        let mut machines = BTreeMap::new();
        let mut decision_services = BTreeMap::new();
        let mut admission_policy = None;
        let mut routing_policy = None;
        let mut project_preferences = serde_json::Value::Null;
        match std::fs::symlink_metadata(&source) {
            Ok(_) => {
                let content = read_private_registry(&source)?;
                if emit_warnings {
                    for warning in registry_warnings(&content, &source, &agents) {
                        eprintln!("avertissement: {warning}");
                    }
                }
                let user: AgentRegistryFile = serde_json::from_str(&content)
                    .map_err(|err| format!("registre invalide {}: {err}", source.display()))?;
                validate_registry(&user.agents, &source)?;
                validate_machines(&user.machines)?;
                validate_admission_config(&user.decision_services, user.admission_policy.as_ref())?;
                let mut validation_agents = default_agents()?;
                validation_agents.extend(user.agents.clone());
                validate_routing_policy(
                    user.routing_policy.as_ref(),
                    &validation_agents,
                    &user.machines,
                    &user.decision_services,
                )?;
                if !user.machines.is_empty()
                    || !user.decision_services.is_empty()
                    || user.admission_policy.is_some()
                    || user.routing_policy.is_some()
                {
                    validate_private_machine_registry_parent(&source)?;
                }
                agents.extend(user.agents);
                machines = user.machines;
                decision_services = user.decision_services;
                admission_policy = user.admission_policy;
                routing_policy = user.routing_policy;
                project_preferences = user.project_preferences;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "impossible d'inspecter {}: {error}",
                    source.display()
                ));
            }
        }
        add_project_discovery_definitions(&mut agents);
        Ok(Self {
            agents,
            machines,
            decision_services,
            admission_policy,
            routing_policy,
            project_preferences,
            source,
        })
    }

    pub fn from_json(content: &str, source: impl Into<PathBuf>) -> Result<Self, String> {
        let source = source.into();
        let user: AgentRegistryFile = serde_json::from_str(content)
            .map_err(|err| format!("registre invalide {}: {err}", source.display()))?;
        validate_registry(&user.agents, &source)?;
        validate_machines(&user.machines)?;
        validate_admission_config(&user.decision_services, user.admission_policy.as_ref())?;
        let mut validation_agents = default_agents()?;
        validation_agents.extend(user.agents.clone());
        validate_routing_policy(
            user.routing_policy.as_ref(),
            &validation_agents,
            &user.machines,
            &user.decision_services,
        )?;
        let mut agents = default_agents()?;
        agents.extend(user.agents);
        add_project_discovery_definitions(&mut agents);
        Ok(Self {
            agents,
            machines: user.machines,
            decision_services: user.decision_services,
            admission_policy: user.admission_policy,
            routing_policy: user.routing_policy,
            project_preferences: user.project_preferences,
            source,
        })
    }

    /// Les octets de from_json sont fournis par l'appelant ; seule load vérifie
    /// les permissions du fichier. Aucun loader fournisseur ne copie ce registre.
    pub fn machines(&self) -> &BTreeMap<String, MachineProvision> {
        &self.machines
    }

    pub fn decision_services(&self) -> &BTreeMap<String, DecisionService> {
        &self.decision_services
    }

    pub fn decision_service(&self, service_id: &str) -> Result<&DecisionService, String> {
        let service = self
            .decision_services
            .get(service_id)
            .ok_or("decision_service_unknown")?;
        if !service.enabled {
            return Err("decision_service_disabled".into());
        }
        Ok(service)
    }

    pub fn admission_policy(&self) -> Option<&AdmissionPolicy> {
        self.admission_policy.as_ref()
    }

    /// Politique168 optionnelle : `None` signifie chemin163 inchangé.
    /// Sa consultation ne lance jamais de préflight ni de découverte modèle.
    pub fn routing_policy(&self) -> Option<&RoutingPolicy> {
        self.routing_policy.as_ref()
    }

    /// Même connexion, lanceur, politique et worker ; le modèle de mission
    /// ne change pas le compte. Aucun secret ni fait fournisseur n'est créé.
    pub(crate) fn routing_connection_scope(&self, connection_id: &str) -> Option<String> {
        let policy = self.routing_policy()?;
        let connection = policy
            .connections
            .iter()
            .find(|item| item.enabled && item.connection_id == connection_id)?;
        let definition = self.resolved_definition(&connection.agent_type).ok()?;
        let worker = self.machines.get(&connection.machine_id).map(|item| {
            (
                &item.worker_id,
                item.provisioning_revision,
                &item.credential_fingerprint,
            )
        });
        let bytes =
            serde_json::to_vec(&(connection, definition.digest, policy.revision, worker)).ok()?;
        Some(format!("{:x}", Sha256::digest(bytes)))
    }

    /// Matrice complète déclarée avant les filtres de demande. Les connexions
    /// désactivées restent comptées ici afin qu'un registre129 ne puisse pas
    /// devenir valide en masquant arbitrairement une partie du catalogue.
    pub fn routing_candidates(&self) -> Option<Vec<RoutingCandidate>> {
        self.routing_policy.as_ref().map(|policy| {
            policy
                .connections
                .iter()
                .flat_map(|connection| {
                    policy
                        .model_catalog
                        .get(&connection.connection_id)
                        .into_iter()
                        .flatten()
                        .map(move |model| RoutingCandidate {
                            machine_id: connection.machine_id.clone(),
                            engine: connection.engine.clone(),
                            connection_id: connection.connection_id.clone(),
                            agent_type: connection.agent_type.clone(),
                            enabled: connection.enabled,
                            model: model.model.clone(),
                            supported_efforts: model.supported_efforts.clone(),
                            default_effort: model.default_effort.clone(),
                            provenance: model.provenance,
                            observed_at: model.observed_at,
                        })
                })
                .collect()
        })
    }

    /// Projection de lecture seule destinée au catalogue168. Elle ne révèle ni
    /// lanceur, ni chemin, ni variable d'environnement ; les observations
    /// sont celles déjà déclarées dans le registre, jamais rafraîchies ici.
    pub fn routing_catalogue(&self, observed_at: i64) -> Result<RoutingCatalogueV1, String> {
        let policy = self
            .routing_policy
            .as_ref()
            .ok_or("routing_policy_unavailable")?;
        if !policy.enabled {
            return Err("routing_policy_disabled".into());
        }
        let mut options = self
            .routing_candidates()
            .expect("routing_policy was checked above")
            .into_iter()
            .filter(|candidate| candidate.enabled)
            .map(|candidate| RoutingCatalogueOptionV1 {
                machine_id: candidate.machine_id,
                engine: candidate.engine,
                connection_id: candidate.connection_id,
                model: candidate.model,
                supported_efforts: candidate.supported_efforts,
                default_effort: candidate.default_effort,
                provenance: routing_model_provenance_name(candidate.provenance).into(),
                observed_at: candidate.observed_at,
                entitlement: RoutingEntitlement::Unknown,
            })
            .collect::<Vec<_>>();
        options.sort_by(|left, right| {
            (
                &left.machine_id,
                &left.engine,
                &left.connection_id,
                &left.model,
            )
                .cmp(&(
                    &right.machine_id,
                    &right.engine,
                    &right.connection_id,
                    &right.model,
                ))
        });
        Ok(RoutingCatalogueV1 {
            version: 1,
            observed_at,
            options,
        })
    }

    /// Applique uniquement les contraintes explicites et les axes fixes de
    /// la machine coordinatrice. Le score, les faits dynamiques et les droits
    /// restent hors de cette méthode pure et sont traités par l'admission.
    pub fn select_routing_candidates(
        &self,
        coordinator_machine_id: &str,
        requested: &RoutingCandidateFilters,
    ) -> Result<Vec<RoutingCandidateSelection>, String> {
        let policy = self
            .routing_policy
            .as_ref()
            .ok_or("routing_policy_unavailable")?;
        if !policy.enabled {
            return Err("routing_policy_disabled".into());
        }
        let machine_policy = policy
            .machine_policies
            .get(coordinator_machine_id)
            .ok_or("routing_machine_policy_unknown")?;
        if !machine_policy.enabled {
            return Err("routing_machine_policy_disabled".into());
        }
        let effective = effective_routing_filters(requested, machine_policy)?;
        let mut candidates = self
            .routing_candidates()
            .expect("routing_policy was checked above")
            .into_iter()
            .filter(|candidate| candidate.enabled)
            .filter(|candidate| {
                effective
                    .engine
                    .as_deref()
                    .is_none_or(|value| candidate.engine == value)
                    && effective
                        .model
                        .as_deref()
                        .is_none_or(|value| candidate.model == value)
                    && effective
                        .machine_id
                        .as_deref()
                        .is_none_or(|value| candidate.machine_id == value)
                    && effective
                        .connection_id
                        .as_deref()
                        .is_none_or(|value| candidate.connection_id == value)
                    && effective.effort.as_deref().is_none_or(|value| {
                        candidate
                            .supported_efforts
                            .iter()
                            .any(|effort| effort == value)
                    })
            })
            .map(|candidate| RoutingCandidateSelection {
                candidate,
                imposed_effort: effective.effort.clone(),
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            (
                &left.candidate.machine_id,
                &left.candidate.engine,
                &left.candidate.connection_id,
                &left.candidate.model,
            )
                .cmp(&(
                    &right.candidate.machine_id,
                    &right.candidate.engine,
                    &right.candidate.connection_id,
                    &right.candidate.model,
                ))
        });
        Ok(candidates)
    }

    pub fn machine(&self, machine_id: &str) -> Result<&MachineProvision, String> {
        let machine = self.machines.get(machine_id).ok_or("machine_unknown")?;
        if !machine.enabled {
            return Err("machine_disabled".into());
        }
        Ok(machine)
    }

    pub fn project_mapping(
        &self,
        machine_id: &str,
        project_id: &str,
    ) -> Result<&ProjectMapping, String> {
        self.machine(machine_id)?
            .mappings
            .get(project_id)
            .ok_or_else(|| "project_mapping_unknown".into())
    }

    /// O(M log P), M/P <=64. Résolution d'une admission neuve. Les tâches utilisent project_mapping,
    /// dont la clé et les révocations ne dépendent jamais de cette préférence.
    pub(crate) fn select_project_mapping(
        &self, machine: &str, requested: Option<&str>, origin: &str, cwd: &Path,
    ) -> Result<(String, ProjectMapping), String> {
        let provision = self.machine(machine)?;
        let resolve = |mapping: &ProjectMapping| mapping.for_origin(machine, origin)
            .filter(|mapping| cwd.starts_with(&mapping.source_root));
        if let Some(id) = requested {
            return resolve(provision.mappings.get(id).ok_or("project_mapping_unknown")?)
                .map(|mapping| (id.to_owned(), mapping))
                .ok_or_else(|| "project_mapping_mismatch".into());
        }
        let matches = provision.mappings.iter().filter_map(|(id,mapping)|
            resolve(mapping).map(|mapping|(id.clone(),mapping))).collect::<Vec<_>>();
        if matches.len()==1 {return Ok(matches.into_iter().next().unwrap());}
        if matches.is_empty() {return Err("project_mapping_unknown".into());}
        let mut preferred = std::collections::BTreeSet::new();
        if !self.project_preferences.is_null() {
            let invalid = || "project_mapping_preference_invalid".to_owned();
            let section = self.project_preferences.as_object().ok_or_else(invalid)?;
            if section.len()!=2 || section.get("version").and_then(|v|v.as_u64())!=Some(1) {
                return Err(invalid());
            }
            let machines = section.get("machines").and_then(|v|v.as_object()).ok_or_else(invalid)?;
            if machines.len()>MAX_MACHINES {return Err(invalid());}
            if let Some(value) = machines.get(machine) {
                let ids = value.as_array().ok_or_else(invalid)?;
                if ids.len()>MAX_MACHINE_MAPPINGS {return Err(invalid());}
                for value in ids {
                    let id = value.as_str().ok_or_else(invalid)?;
                    if !provision.mappings.contains_key(id) || !preferred.insert(id) {
                        return Err(invalid());
                    }
                }
            }
        }
        let choices = matches.iter().filter(|(id,_)|preferred.contains(id.as_str())).collect::<Vec<_>>();
        if choices.len()==1 {
            let (id, selected) = choices[0];
            if matches.iter().all(|(_,mapping)|mapping.source_host==selected.source_host
                && mapping.source_root==selected.source_root && mapping.additional_root==selected.additional_root) {
                return Ok((id.clone(),selected.clone()));
            }
        }
        Err("project_mapping_ambiguous".into())
    }

    pub fn get(&self, agent_type: &str) -> Result<&AgentDefinition, String> {
        self.agents.get(agent_type).ok_or_else(|| {
            let available = self.agents.keys().cloned().collect::<Vec<_>>().join(", ");
            format!(
                "type d'agent inconnu '{agent_type}' dans {}. Types disponibles : {available}",
                self.source.display()
            )
        })
    }

    pub fn resolved_definition(&self, agent_type: &str) -> Result<ResolvedAgentDefinition, String> {
        resolved_definition(self.get(agent_type)?)
    }

    /// Projette uniquement les définitions résolues et attestées du registre.
    /// Les secrets et les valeurs d environnement n y figurent jamais.
    pub fn resolved_definitions(&self) -> Result<Vec<(String, ResolvedAgentDefinition)>, String> {
        self.agents
            .iter()
            .map(|(agent_type, definition)| {
                resolved_definition(definition).map(|resolved| (agent_type.clone(), resolved))
            })
            .collect()
    }

    /// Retourne le type interne, réellement lecture seule, associé à un
    /// choix de coordinateur visible. Les protocoles dont nous ne savons pas
    /// prouver cette propriété ne sont volontairement pas proposés.
    pub fn project_discovery_agent_type(&self, agent_type: &str) -> Option<String> {
        let candidate = format!("{PROJECT_DISCOVERY_AGENT_PREFIX}{agent_type}");
        self.agents.contains_key(&candidate).then_some(candidate)
    }

    /// O(log n + a), n types et a arguments : le registre temporaire contient
    /// le seul type demandé. La saga fige sa définition et son digest.
    pub fn for_spawn_posture(
        &self,
        agent_type: &str,
        posture: bridget_transport::protocol::SpawnPosture,
    ) -> Result<Self, String> {
        use bridget_transport::protocol::SpawnPosture;
        let source = self.get(agent_type)?;
        let definition = match posture {
            SpawnPosture::Discovery => {
                let mut definition = project_discovery_definition(source)
                    .ok_or_else(|| "posture découverte non prise en charge".to_string())?;
                if definition.protocol == "codex_app_server" {
                    definition.args.extend(
                        [
                            "-c",
                            "sandbox_mode=\"read-only\"",
                            "-c",
                            "approval_policy=\"never\"",
                        ]
                        .map(str::to_string),
                    );
                }
                definition
            }
            SpawnPosture::Development => {
                if source.protocol != "codex_app_server" {
                    return Err("posture développement réservée à Codex app-server".to_string());
                }
                let mut definition = source.clone();
                // Les demandes d'extension sont refusées par le pilote. Le
                // droit d'écrire dans cwd vient uniquement du sandbox Codex.
                definition.permissions = "deny".to_string();
                strip_codex_discovery_unsafe_arguments(&mut definition.args);
                insert_before_subcommand(
                    &mut definition.args,
                    "app-server",
                    &[
                        "--sandbox",
                        "workspace-write",
                        "--ask-for-approval",
                        "never",
                    ],
                );
                // Placés après les arguments source, ces réglages gagnent
                // aussi contre un -c fourni après la sous-commande.
                definition.args.extend(
                    [
                        "-c",
                        "sandbox_mode=\"workspace-write\"",
                        "-c",
                        "approval_policy=\"never\"",
                        "-c",
                        "sandbox_workspace_write.network_access=false",
                        "-c",
                        "sandbox_workspace_write.writable_roots=[]",
                        "-c",
                        "sandbox_workspace_write.exclude_tmpdir_env_var=true",
                        "-c",
                        "sandbox_workspace_write.exclude_slash_tmp=true",
                    ]
                    .map(str::to_string),
                );
                definition
            }
        };
        Ok(Self {
            agents: BTreeMap::from([(agent_type.to_string(), definition)]),
            machines: BTreeMap::new(),
            decision_services: BTreeMap::new(),
            admission_policy: None,
            routing_policy: None,
            project_preferences: serde_json::Value::Null,
            source: self.source.clone(),
        })
    }

    /// Reconstruit un registre à une seule entrée depuis la définition figée
    /// par la saga. Le digest est revérifié avant tout lancement : le wrapper
    /// géré ne consulte donc jamais le registre utilisateur courant.
    pub fn from_resolved(
        agent_type: &str,
        resolved: &ResolvedAgentDefinition,
    ) -> Result<Self, String> {
        let mut definition = AgentDefinition {
            command: resolved.command.clone(),
            args: resolved.args.clone(),
            protocol: resolved.protocol.clone(),
            forbidden_env: resolved.forbidden_env.clone(),
            pass_env: resolved.pass_env.clone(),
            claude_config_dir: resolved.claude_config_dir.clone(),
            permissions: resolved.permissions.clone(),
            queue_capacity: resolved.queue_capacity,
            held_queue: false,
            notify_timeout_secs: resolved.notify_timeout_secs,
            mcp: McpDefinition {
                interactive: resolved.mcp.interactive.clone(),
                acp_session: resolved.mcp.acp_session,
            },
            capabilities: resolved.capabilities.clone(),
        };
        let expected = resolved_definition(&definition)?;
        let digest_before_profile = definition.claude_config_dir.is_none()
            && expected.digest != resolved.digest
            && pre_profile_resolved_digest(&definition)? == resolved.digest;
        let digest_before_capabilities = definition.claude_config_dir.is_none()
            && expected.digest != resolved.digest
            && legacy_resolved_digest(&definition)? == resolved.digest;
        if expected.digest != resolved.digest
            && !digest_before_profile
            && !digest_before_capabilities
        {
            return Err("digest de la définition figée invalide".to_string());
        }
        if digest_before_capabilities {
            definition.capabilities = legacy_capabilities_for(&definition);
        }
        let source = PathBuf::from("<définition-figée>");
        let agents = BTreeMap::from([(agent_type.to_string(), definition)]);
        validate_registry(&agents, &source)?;
        Ok(Self {
            agents,
            machines: BTreeMap::new(),
            decision_services: BTreeMap::new(),
            admission_policy: None,
            routing_policy: None,
            project_preferences: serde_json::Value::Null,
            source,
        })
    }

    pub fn source(&self) -> &Path {
        &self.source
    }

    /// Sélection native exacte, bornée par le catalogue et la posture.
    /// Le registre global et ses autres fournisseurs restent inchangés.
    pub(crate) fn for_delegation(
        &self,
        agent_type: &str,
        model: &str,
        effort: Option<&str>,
        posture: bridget_transport::protocol::SpawnPosture,
    ) -> Result<Self, String> {
        let source = self.get(agent_type)?;
        let capabilities = source
            .capabilities
            .models
            .get(model)
            .ok_or_else(|| "model_unavailable".to_string())?;
        if effort.is_some_and(|value| !capabilities.efforts.iter().any(|item| item == value)) {
            return Err("effort_unavailable".into());
        }
        let scoped = self.for_spawn_posture(agent_type, posture)?;
        scoped.pin_delegation_model(agent_type, model, effort)
    }

    /// Les droits sont construits au puits natif depuis un fait attesté.
    pub(crate) fn for_inherited_delegation(
        &self,
        agent_type: &str,
        model: &str,
        effort: Option<&str>,
        policy: &serde_json::Value,
    ) -> Result<Self, String> {
        let mut operation = crate::native_permissions::PermissionReadOperation::default();
        let result = self.for_inherited_delegation_in_operation(agent_type, model, effort, policy, &mut operation)?;
        operation.finish()?;
        Ok(result)
    }

    pub(crate) fn for_inherited_delegation_in_operation(
        &self,
        agent_type: &str,
        model: &str,
        effort: Option<&str>,
        policy: &serde_json::Value,
        operation: &mut crate::native_permissions::PermissionReadOperation,
    ) -> Result<Self, String> {
        let source = self.get(agent_type)?;
        let capabilities = source
            .capabilities
            .models
            .get(model)
            .ok_or("model_unavailable")?;
        if effort.is_some_and(|value| !capabilities.efforts.iter().any(|item| item == value)) {
            return Err("effort_unavailable".into());
        }
        let mut definition = source.clone();
        definition.permissions = "deny".into();
        crate::native_permissions::apply_child_arguments_in_operation(
            &definition.protocol, policy, &mut definition.args, operation,
        )?;
        Self {
            agents: BTreeMap::from([(agent_type.into(), definition)]),
            machines: BTreeMap::new(),
            decision_services: BTreeMap::new(),
            admission_policy: None,
            routing_policy: None,
            project_preferences: serde_json::Value::Null,
            source: self.source.clone(),
        }
        .pin_delegation_model(agent_type, model, effort)
    }

    /// O(log n+B+C) en temps et O(B) en mémoire : n types du registre, B octets
    /// des arguments/modèle/effort, C coût de validation des capacités déclarées.
    pub(crate) fn pin_delegation_model(
        mut self,
        agent_type: &str,
        model: &str,
        effort: Option<&str>,
    ) -> Result<Self, String> {
        self.machines.clear();
        self.decision_services.clear();
        self.admission_policy = None;
        self.routing_policy = None;
        let definition = self
            .agents
            .get_mut(agent_type)
            .ok_or("provider_unavailable")?;
        let mut args = Vec::new();
        let mut index = 0;
        while index < definition.args.len() {
            let arg = &definition.args[index];
            if matches!(
                arg.as_str(),
                "--model" | "--effort" | "--effort-level" | "-m"
            ) {
                index += 2;
                continue;
            }
            if matches!(arg.as_str(), "-c" | "--config")
                && definition.args.get(index + 1).is_some_and(|value| {
                    ["model=", "model_reasoning_effort=", "effort="]
                        .iter()
                        .any(|key| value.starts_with(key))
                })
            {
                index += 2;
                continue;
            }
            if [
                "model=",
                "model_reasoning_effort=",
                "effort=",
                "--model=",
                "--effort=",
                "--effort-level=",
                "--config=model=",
                "--config=model_reasoning_effort=",
                "--config=effort=",
            ]
            .iter()
            .any(|key| arg.starts_with(key))
            {
                index += 1;
                continue;
            }
            args.push(arg.clone());
            index += 1;
        }
        match definition.protocol.as_str() {
            "codex_app_server" => {
                args.extend([
                    "-c".into(),
                    format!(
                        "model={}",
                        serde_json::to_string(model).map_err(|error| error.to_string())?
                    ),
                ]);
                if let Some(effort) = effort {
                    args.extend([
                        "-c".into(),
                        format!(
                            "model_reasoning_effort={}",
                            serde_json::to_string(effort).map_err(|error| error.to_string())?
                        ),
                    ]);
                }
            }
            "claude_stream_json" => {
                args.extend(["--model".into(), model.into()]);
                if let Some(effort) = effort {
                    args.extend(["--effort".into(), effort.into()]);
                }
            }
            _ => return Err("delegation_protocol_unavailable".into()),
        }
        definition.args = args;
        validate_launch_capabilities(agent_type, definition)
            .map_err(|error| format!("{error:?}"))?;
        Ok(self)
    }

    /// Instantané ordonné des types effectivement chargés par le daemon.
    pub fn known_types(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    /// Alias des lanceurs interactifs historiques. Cette table ne vaut pas
    /// autorisation : `type_for_command` consulte toujours le registre.
    pub fn interactive_alias(command: &str) -> Option<&'static str> {
        match command {
            "codex" => Some("codex"),
            "claude" | "gclaude" | "claude-son" => Some("claude"),
            "gemini" => Some("gemini"),
            _ => None,
        }
    }

    /// Résout une commande du flux `bridget --` vers son type déclaré.
    ///
    /// Les alias interactifs historiques restent acceptés, puis les commandes
    /// du registre sont comparées par leur basename pour tolérer un chemin.
    pub fn type_for_command(&self, command: &str) -> Result<String, String> {
        let basename = command_basename(command);
        if let Some(agent_type) = Self::interactive_alias(basename) {
            self.get(agent_type)?;
            return Ok(agent_type.to_string());
        }
        let matching_types = self
            .agents
            .iter()
            // Les dérivations de politique ne sont pas des déclarations de
            // fournisseur supplémentaires. Leur sélection reste du ressort
            // du préflight, après résolution du type demandé.
            .filter(|(agent_type, _)| !agent_type.starts_with(PROJECT_DISCOVERY_AGENT_PREFIX))
            .filter(|(_, definition)| command_basename(&definition.command) == basename)
            .map(|(agent_type, _)| agent_type.as_str())
            .collect::<Vec<_>>();
        if matching_types.len() == 1 {
            return Ok(matching_types[0].to_string());
        }
        if matching_types.len() > 1 {
            return Err(format!(
                "commande ambiguë '{command}' dans {} : {}. Utilisez un type explicite.",
                self.source.display(),
                matching_types.join(", ")
            ));
        }

        let available = self
            .agents
            .iter()
            .map(|(agent_type, definition)| format!("{agent_type} ({})", definition.command))
            .collect::<Vec<_>>()
            .join(", ");
        Err(format!(
            "commande '{command}' non déclarée dans {}. Types/commandes disponibles : {available}",
            self.source.display()
        ))
    }
}

fn legacy_capabilities_for(definition: &AgentDefinition) -> AdapterCapabilities {
    let mut capabilities = AdapterCapabilities::default();
    if let Some((model, effort)) = runtime_model_and_effort(&definition.args) {
        capabilities.models.insert(
            model,
            ModelCapabilities {
                efforts: effort.into_iter().collect(),
            },
        );
    }
    capabilities
}

/// Paquets npm `@zed-industries/{codex,claude-code}-acp` : pont tiers figé
/// retiré en G10 (ADR 010). L'ACP générique (`cursor-agent acp`, `gemini
/// --acp`, fixtures ACP) reste autorisé.
///
/// Normalise avant comparaison : basename du chemin + retrait du suffixe
/// npm `@version`, pour couvrir aussi les chemins absolus vivants du projet
/// (`/opt/homebrew/bin/codex-acp`, `./node_modules/.bin/…`, `codex-acp@0.16.0`).
fn normalize_bridge_token(token: &str) -> String {
    let lowered = token.to_ascii_lowercase();
    let base = Path::new(&lowered)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(lowered.as_str());
    match base.rfind('@').filter(|&index| index > 0) {
        Some(index) => base[..index].to_string(),
        None => base.to_string(),
    }
}

fn zed_bridge_token(token: &str) -> Option<&'static str> {
    let normalized = normalize_bridge_token(token);
    let lowered = token.to_ascii_lowercase();
    if normalized == "codex-acp" || lowered.contains("@zed-industries/codex-acp") {
        Some("@zed-industries/codex-acp")
    } else if normalized == "claude-code-acp"
        || normalized == "claude-acp"
        || lowered.contains("@zed-industries/claude-code-acp")
    {
        Some("@zed-industries/claude-code-acp")
    } else {
        None
    }
}

/// Refuse un lancement qui ciblerait encore le pont Zed pour Codex ou Claude.
/// Ne touche pas le protocole ACP lui-même : seuls les paquets tiers figés
/// sont exclus.
pub(crate) fn reject_retired_zed_bridge(definition: &AgentDefinition) -> Result<(), SpawnRefusal> {
    let mut tokens = vec![definition.command.as_str()];
    tokens.extend(definition.args.iter().map(String::as_str));
    for token in tokens {
        if let Some(package) = zed_bridge_token(token) {
            return Err(SpawnRefusal::EnvUnfit {
                detail: format!(
                    "pont Zed retiré (G10, 2026-08-24) : le paquet {package} n'est plus un chemin de lancement ; utiliser le pilote natif (codex app-server / claude_stream_json) — ACP reste pour cursor et gemini"
                ),
            });
        }
    }
    Ok(())
}

/// Vérifie une demande de lancement contre la matrice persistable du registre.
/// Aucune sonde du pilote n'est exécutée : le même ordre rejoué conserve donc
/// la même décision après un crash ou une évolution externe.
pub(crate) fn validate_launch_capabilities(
    agent_type: &str,
    definition: &AgentDefinition,
) -> Result<(), SpawnRefusal> {
    reject_retired_zed_bridge(definition)?;
    let (model, effort) = runtime_labels(&definition.args);
    let model_label = model.clone().unwrap_or_else(|| "<non déclaré>".to_string());
    if !definition
        .capabilities
        .execution_paths
        .iter()
        .any(|path| path == &definition.protocol)
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: format!("chemin d'exécution '{}'", definition.protocol),
        });
    }
    if model
        .as_deref()
        .is_some_and(|value| !valid_capability_value(value))
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: "étiquette de modèle valide".to_string(),
        });
    }
    if effort
        .as_deref()
        .is_some_and(|value| !valid_capability_value(value))
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model: model_label,
            capability: "étiquette d'effort valide".to_string(),
        });
    }
    let Some(model) = model else {
        if effort.is_some() {
            return Err(SpawnRefusal::UnsupportedCapability {
                agent_type: agent_type.to_string(),
                model: model_label,
                capability: "modèle explicite requis pour l'effort".to_string(),
            });
        }
        return Ok(());
    };
    let Some(model_capabilities) = definition.capabilities.models.get(&model) else {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model,
            capability: "modèle pris en charge par l'adaptateur".to_string(),
        });
    };
    if let Some(effort) = effort
        && !model_capabilities
            .efforts
            .iter()
            .any(|declared| declared == &effort)
    {
        return Err(SpawnRefusal::UnsupportedCapability {
            agent_type: agent_type.to_string(),
            model,
            capability: format!("effort '{effort}' accepté par le modèle"),
        });
    }
    Ok(())
}

/// Extrait les étiquettes opaques du lancement sans les normaliser ni les
/// substituer. La projection de l'annuaire et la garde de lancement partagent
/// ainsi exactement la même lecture des arguments.
pub(crate) fn runtime_model_and_effort(args: &[String]) -> Option<(String, Option<String>)> {
    let (model, effort) = runtime_labels(args);
    let model = model.filter(|value| valid_capability_value(value))?;
    let effort = effort.filter(|value| valid_capability_value(value));
    Some((model, effort))
}

fn runtime_labels(args: &[String]) -> (Option<String>, Option<String>) {
    let mut model = None;
    let mut effort = None;
    let mut index = 0;
    while index < args.len() {
        let argument = &args[index];
        if matches!(argument.as_str(), "--model" | "--effort" | "--effort-level") {
            if let Some(value) = args.get(index + 1) {
                let value = unquote_runtime_value(value);
                if argument == "--model" {
                    model = Some(value);
                } else {
                    effort = Some(value);
                }
            }
            index += 2;
            continue;
        }
        if let Some(value) = runtime_assignment(argument, "model") {
            model = Some(unquote_runtime_value(value));
        }
        if let Some(value) = runtime_assignment(argument, "model_reasoning_effort")
            .or_else(|| runtime_assignment(argument, "effort"))
        {
            effort = Some(unquote_runtime_value(value));
        }
        index += 1;
    }
    (model, effort)
}

fn runtime_assignment<'a>(argument: &'a str, key: &str) -> Option<&'a str> {
    argument
        .strip_prefix(key)
        .and_then(|suffix| suffix.strip_prefix('='))
}

fn unquote_runtime_value(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

fn valid_capability_value(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_CAPABILITY_VALUE_CHARS
        && !value.chars().any(bridget_core::is_disallowed_control)
}

fn command_basename(command: &str) -> &str {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
}

fn registry_warnings(
    content: &str,
    source: &Path,
    defaults: &BTreeMap<String, AgentDefinition>,
) -> Vec<String> {
    const KEYS: &[&str] = &[
        "command",
        "args",
        "protocol",
        "forbidden_env",
        "pass_env",
        "claude_config_dir",
        "permissions",
        "queue_capacity",
        "held_queue",
        "notify_timeout_secs",
        "mcp",
        "capabilities",
    ];
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return Vec::new();
    };
    let Some(agents) = value.get("agents").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    let mut warnings = Vec::new();
    for (agent_type, definition) in agents {
        let Some(definition) = definition.as_object() else {
            continue;
        };
        for key in definition
            .keys()
            .filter(|key| !KEYS.contains(&key.as_str()))
        {
            warnings.push(format!(
                "clé inconnue '{key}' pour '{agent_type}' dans {}",
                source.display()
            ));
        }
        if !definition.contains_key("forbidden_env")
            && defaults
                .get(agent_type)
                .is_some_and(|default| !default.forbidden_env.is_empty())
        {
            warnings.push(format!(
                "'{agent_type}' remplace un défaut protégé sans déclarer forbidden_env dans {}",
                source.display()
            ));
        }
    }
    warnings
}

fn read_private_registry(source: &Path) -> Result<String, String> {
    let link_metadata = std::fs::symlink_metadata(source)
        .map_err(|err| format!("impossible d'inspecter {}: {err}", source.display()))?;
    if link_metadata.file_type().is_symlink() {
        return Err(format!("registre refusé {}: les liens symboliques sont interdits",source.display()));
    }
    if !link_metadata.is_file() {
        return Err(format!("registre refusé {}: fichier régulier requis",source.display()));
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(source)
        .map_err(|err| {
            format!(
                "impossible d'ouvrir {} sans suivre de lien: {err}",
                source.display()
            )
        })?;
    let metadata = file
        .metadata()
        .map_err(|err| format!("impossible d'inspecter {}: {err}", source.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "registre refusé {}: fichier régulier requis",
            source.display()
        ));
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(format!(
            "registre refusé {}: propriétaire inattendu",
            source.display()
        ));
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(format!(
            "registre refusé {}: permissions {mode:04o}, attendu 0600 ou plus restrictif",
            source.display()
        ));
    }
    const MAX_REGISTRY_BYTES:u64=16*1024*1024;
    if metadata.len()>MAX_REGISTRY_BYTES {return Err("registre trop volumineux".into());}
    let mut content = String::new();
    std::io::Read::read_to_string(&mut std::io::Read::take(&mut file,MAX_REGISTRY_BYTES+1), &mut content)
        .map_err(|err| format!("impossible de lire {}: {err}", source.display()))?;
    if content.len() as u64>MAX_REGISTRY_BYTES {return Err("registre trop volumineux".into());}
    Ok(content)
}

#[derive(Serialize)]
struct CanonicalResolvedDefinition<'a> {
    command: &'a str,
    args: &'a [String],
    protocol: &'a str,
    forbidden_env: &'a [String],
    pass_env: &'a [String],
    claude_config_dir: Option<&'a str>,
    permissions: &'a str,
    queue_capacity: usize,
    notify_timeout_secs: u64,
    mcp: CanonicalResolvedMcpDefinition<'a>,
    capabilities: &'a AdapterCapabilities,
}

/// Forme de digest publiée avant l'ajout du profil Claude statique. Elle
/// conserve les capacités L1 et ne peut donc être admise que pour une
/// définition sans profil, déjà persistée par une version antérieure.
#[derive(Serialize)]
struct PreProfileCanonicalResolvedDefinition<'a> {
    command: &'a str,
    args: &'a [String],
    protocol: &'a str,
    forbidden_env: &'a [String],
    pass_env: &'a [String],
    permissions: &'a str,
    queue_capacity: usize,
    notify_timeout_secs: u64,
    mcp: CanonicalResolvedMcpDefinition<'a>,
    capabilities: &'a AdapterCapabilities,
}

#[derive(Serialize)]
struct CanonicalResolvedMcpDefinition<'a> {
    interactive: &'a str,
    acp_session: bool,
}

/// Forme de digest publiée avant la matrice L1. Elle n'est acceptée qu'à la
/// lecture d'une génération déjà persistée, puis la prochaine transition
/// réécrit la définition complète avec les capacités déclaratives.
#[derive(Serialize)]
struct LegacyCanonicalResolvedDefinition<'a> {
    command: &'a str,
    args: &'a [String],
    protocol: &'a str,
    forbidden_env: &'a [String],
    pass_env: &'a [String],
    permissions: &'a str,
    queue_capacity: usize,
    notify_timeout_secs: u64,
    mcp: CanonicalResolvedMcpDefinition<'a>,
}

fn legacy_resolved_digest(definition: &AgentDefinition) -> Result<String, String> {
    let canonical = LegacyCanonicalResolvedDefinition {
        command: &definition.command,
        args: &definition.args,
        protocol: &definition.protocol,
        forbidden_env: &definition.forbidden_env,
        pass_env: &definition.pass_env,
        permissions: &definition.permissions,
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: CanonicalResolvedMcpDefinition {
            interactive: &definition.mcp.interactive,
            acp_session: definition.mcp.acp_session,
        },
    };
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("définition historique impossible à sérialiser: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn pre_profile_resolved_digest(definition: &AgentDefinition) -> Result<String, String> {
    let canonical = PreProfileCanonicalResolvedDefinition {
        command: &definition.command,
        args: &definition.args,
        protocol: &definition.protocol,
        forbidden_env: &definition.forbidden_env,
        pass_env: &definition.pass_env,
        permissions: &definition.permissions,
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: CanonicalResolvedMcpDefinition {
            interactive: &definition.mcp.interactive,
            acp_session: definition.mcp.acp_session,
        },
        capabilities: &definition.capabilities,
    };
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("définition pré-profil impossible à sérialiser: {err}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn resolved_definition(definition: &AgentDefinition) -> Result<ResolvedAgentDefinition, String> {
    let canonical = CanonicalResolvedDefinition {
        command: &definition.command,
        args: &definition.args,
        protocol: &definition.protocol,
        forbidden_env: &definition.forbidden_env,
        pass_env: &definition.pass_env,
        claude_config_dir: definition.claude_config_dir.as_deref(),
        permissions: &definition.permissions,
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: CanonicalResolvedMcpDefinition {
            interactive: &definition.mcp.interactive,
            acp_session: definition.mcp.acp_session,
        },
        capabilities: &definition.capabilities,
    };
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|err| format!("définition résolue impossible à sérialiser: {err}"))?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    Ok(ResolvedAgentDefinition {
        command: definition.command.clone(),
        args: definition.args.clone(),
        protocol: definition.protocol.clone(),
        forbidden_env: definition.forbidden_env.clone(),
        pass_env: definition.pass_env.clone(),
        claude_config_dir: definition.claude_config_dir.clone(),
        permissions: definition.permissions.clone(),
        queue_capacity: definition.queue_capacity,
        notify_timeout_secs: definition.notify_timeout_secs,
        mcp: ResolvedMcpDefinition {
            interactive: definition.mcp.interactive.clone(),
            acp_session: definition.mcp.acp_session,
        },
        capabilities: definition.capabilities.clone(),
        digest,
    })
}

fn config_path() -> Result<PathBuf, String> {
    crate::environment::Namespace::from_environment()
        .map(|namespace| namespace.root.join("agents.json"))
}

fn machine_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn mapping_root(path: &Path) -> bool {
    // Validation lexicale seulement : la racine Linux ne se résout pas sur le Mac.
    path.to_str()
        .is_some_and(|value| value.len() <= 4096 && !value.chars().any(char::is_control))
        && path.is_absolute()
        && path.components().all(|component| {
            matches!(
                component,
                std::path::Component::RootDir | std::path::Component::Normal(_)
            )
        })
}

fn validate_machines(machines: &BTreeMap<String, MachineProvision>) -> Result<(), String> {
    if machines.len() > MAX_MACHINES {
        return Err("machine_registry_limit".into());
    }
    let mut workers = std::collections::BTreeSet::new();
    for (machine_id, machine) in machines {
        if !machine_label(machine_id)
            || !machine_label(&machine.worker_id)
            || machine
                .display_name
                .as_deref()
                .is_some_and(|name| !machine_label(name))
            || !workers.insert(&machine.worker_id)
            || !(1..=MAX_MACHINE_REVISION).contains(&machine.provisioning_revision)
            || machine.credential_fingerprint.len() != 64
            || !machine
                .credential_fingerprint
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("machine_provision_invalid".into());
        }
        if machine.mappings.len() > MAX_MACHINE_MAPPINGS
            || machine.allowed_agent_types.len() > MAX_MACHINE_MAPPINGS
        {
            return Err("machine_mapping_limit".into());
        }
        let mut types = std::collections::BTreeSet::new();
        if machine
            .allowed_agent_types
            .iter()
            .any(|value| !machine_label(value) || !types.insert(value))
        {
            return Err("machine_agent_types_invalid".into());
        }
        for (project_id, mapping) in &machine.mappings {
            validate_additional_mapping171(mapping)?;
            if !machine_label(project_id)
                || !machine_label(&mapping.source_host)
                || !mapping_root(&mapping.source_root)
                || !mapping_root(&mapping.remote_root)
                || mapping.expected_git_identity.is_empty()
                || mapping.expected_git_identity.len() > 2048
                || mapping.expected_git_identity.chars().any(char::is_control)
                || !(1..=MAX_MACHINE_REVISION).contains(&mapping.mapping_revision)
            {
                return Err("project_mapping_invalid".into());
            }
        }
    }
    Ok(())
}

fn decision_endpoint(value: &str) -> bool {
    let lexical = value.len() <= MAX_DECISION_ENDPOINT_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.contains('@')
        && !value.contains('#');
    if !lexical {
        return false;
    }
    if let Some(rest) = value.strip_prefix("https://") {
        return rest
            .split(['/', '?'])
            .next()
            .is_some_and(|authority| !authority.is_empty());
    }
    let Some(rest) = value.strip_prefix("http://") else {
        return false;
    };
    let authority = rest.split(['/', '?']).next().unwrap_or_default();
    matches!(authority, "localhost" | "127.0.0.1" | "[::1]")
        || authority
            .strip_prefix("localhost:")
            .or_else(|| authority.strip_prefix("127.0.0.1:"))
            .or_else(|| authority.strip_prefix("[::1]:"))
            .is_some_and(|port| port.parse::<u16>().is_ok_and(|port| port != 0))
}

fn profile_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PROFILE_VALUE_BYTES
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn validate_admission_config(
    services: &BTreeMap<String, DecisionService>,
    policy: Option<&AdmissionPolicy>,
) -> Result<(), String> {
    for (service_id, service) in services {
        if !machine_label(service_id)
            || !machine_label(&service.machine_id)
            || service.protocol != "laya_systemone_v1"
            || !profile_value(&service.model)
            || !decision_endpoint(&service.endpoint)
            || !(1..=MAX_DECISION_HEALTH_SECONDS).contains(&service.health_ttl_seconds)
            || !(1..=MAX_DECISION_HEALTH_SECONDS).contains(&service.retry_after_seconds)
        {
            return Err("decision_service_invalid".into());
        }
    }

    let Some(policy) = policy else {
        return Ok(());
    };
    if policy.revision == 0
        || !(1..=MAX_ADMISSION_PROFILES).contains(&policy.profiles.len())
        || policy.preferences.len() > MAX_ADMISSION_PREFERENCES
    {
        return Err("admission_policy_invalid".into());
    }
    if policy
        .service_id
        .as_deref()
        .is_some_and(|service_id| !services.contains_key(service_id))
    {
        return Err("admission_policy_service_unknown".into());
    }

    let mut profile_ids = std::collections::BTreeSet::new();
    for profile in &policy.profiles {
        if !machine_label(&profile.id)
            || !machine_label(&profile.machine_id)
            || !profile_value(&profile.agent_type)
            || !profile_value(&profile.model)
            || profile
                .effort
                .as_deref()
                .is_some_and(|effort| !profile_value(effort))
            || profile.description.as_bytes().len() > MAX_PROFILE_DESCRIPTION_BYTES
            || profile.description.chars().any(char::is_control)
            || !profile_ids.insert(&profile.id)
        {
            return Err("admission_profile_invalid".into());
        }
    }
    for reference in policy
        .fixed_profile
        .iter()
        .chain(policy.fallback_profile.iter())
        .chain(policy.preferences.iter())
    {
        if !profile_ids.contains(reference) {
            return Err("admission_profile_reference_unknown".into());
        }
    }
    Ok(())
}

fn valid_routing_engine(engine: &str, protocol: &str) -> bool {
    matches!(
        (engine, protocol),
        ("codex", "codex_app_server") | ("claude", "claude_stream_json")
    )
}

fn valid_routing_machine(machine_id: &str, machines: &BTreeMap<String, MachineProvision>) -> bool {
    machine_id == "local" || machines.contains_key(machine_id)
}

fn validate_routing_policy(
    policy: Option<&RoutingPolicy>,
    agents: &BTreeMap<String, AgentDefinition>,
    machines: &BTreeMap<String, MachineProvision>,
    services: &BTreeMap<String, DecisionService>,
) -> Result<(), String> {
    let Some(policy) = policy else {
        return Ok(());
    };
    if policy.version != ROUTING_POLICY_VERSION || policy.revision == 0 {
        return Err("routing_policy_invalid".into());
    }

    let mut connection_ids = std::collections::BTreeSet::new();
    let mut candidate_count = 0usize;
    for connection in &policy.connections {
        let Some(definition) = agents.get(&connection.agent_type) else {
            return Err("routing_connection_agent_unknown".into());
        };
        if !profile_value(&connection.connection_id)
            || (connection.enabled && !valid_routing_machine(&connection.machine_id, machines))
            || !valid_routing_engine(&connection.engine, &definition.protocol)
            || connection
                .display_name
                .as_deref()
                .is_some_and(|name| !profile_value(name))
            || !connection_ids.insert(connection.connection_id.as_str())
        {
            return Err("routing_connection_invalid".into());
        }
        let Some(models) = policy.model_catalog.get(&connection.connection_id) else {
            return Err("routing_catalogue_connection_missing".into());
        };
        if models.is_empty() {
            return Err("routing_catalogue_empty".into());
        }
        let mut names = std::collections::BTreeSet::new();
        for model in models {
            if !profile_value(&model.model)
                || !names.insert(&model.model)
                || model
                    .supported_efforts
                    .iter()
                    .any(|effort| !profile_value(effort))
                || model.supported_efforts.len()
                    != model
                        .supported_efforts
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                || (model.supported_efforts.is_empty() && model.default_effort.is_some())
                || (!model.supported_efforts.is_empty()
                    && !matches!(
                        &model.default_effort,
                        Some(effort) if model.supported_efforts.iter().any(|item| item == effort)
                    ))
            {
                return Err("routing_model_invalid".into());
            }
        }
        candidate_count = candidate_count.saturating_add(models.len());
    }
    if policy
        .model_catalog
        .keys()
        .any(|connection_id| !connection_ids.contains(connection_id.as_str()))
    {
        return Err("routing_catalogue_connection_unknown".into());
    }
    if candidate_count == 0 || candidate_count > MAX_ROUTING_CANDIDATES {
        return Err("candidate_limit".into());
    }

    for (machine_id, machine_policy) in &policy.machine_policies {
        if !valid_routing_machine(machine_id, machines)
            || !(1_000..=30_000).contains(&machine_policy.decision_timeout_ms)
            || !(1_000..=30_000).contains(&machine_policy.service_timeout_ms)
            || machine_policy.service_timeout_ms > machine_policy.decision_timeout_ms
            || machine_policy
                .compilation_capacity
                .is_some_and(|value| value > 10)
        {
            return Err("routing_machine_policy_invalid".into());
        }
        if let Some(telemetry) = &machine_policy.telemetry {
            if !(1..=300).contains(&telemetry.max_age_secs)
                || !(1..=100).contains(&telemetry.cpu_saturation_percent)
                || telemetry.memory_available_min_percent > 100
                || !(1..=100).contains(&telemetry.normalized_load_saturation_percent)
            {
                return Err("routing_telemetry_invalid".into());
            }
            let weights = &telemetry.weights;
            if weights.cpu > 20
                || weights.ram > 20
                || weights.load > 20
                || weights.continuity > 20
                || weights.compilation > 10
                || weights.quota > 5
                || weights.reset > 3
            {
                return Err("routing_score_weights_invalid".into());
            }
        }
        let fixed = &machine_policy.fixed_values;
        let automatic = &machine_policy.automatic_axes;
        if (!automatic.engine && !fixed.engine.as_deref().is_some_and(profile_value))
            || (!automatic.model && !fixed.model.as_deref().is_some_and(profile_value))
            || (!automatic.machine && !fixed.machine_id.as_deref().is_some_and(profile_value))
            || (!automatic.connection && !fixed.connection_id.as_deref().is_some_and(profile_value))
            || (!automatic.effort && !fixed.effort.as_deref().is_some_and(profile_value))
        {
            return Err("routing_fixed_axis_missing".into());
        }
        if fixed
            .machine_id
            .as_deref()
            .is_some_and(|value| !valid_routing_machine(value, machines))
            || fixed
                .connection_id
                .as_deref()
                .is_some_and(|value| !connection_ids.contains(value))
        {
            return Err("routing_fixed_axis_invalid".into());
        }
        if let Some(service) = &machine_policy.decision_service {
            if service.enabled
                && !service
                    .service_id
                    .as_deref()
                    .is_some_and(|id| services.contains_key(id))
            {
                return Err("routing_decision_service_invalid".into());
            }
        }
        if let Some(lightweight) = &machine_policy.lightweight_decision {
            if lightweight.enabled {
                let valid_connection = lightweight.connection_id.as_deref().and_then(|id| {
                    policy
                        .connections
                        .iter()
                        .find(|entry| entry.connection_id == id)
                });
                let valid_model = valid_connection.and_then(|connection| {
                    policy
                        .model_catalog
                        .get(&connection.connection_id)
                        .and_then(|models| {
                            models
                                .iter()
                                .find(|model| lightweight.model.as_deref() == Some(&model.model))
                        })
                });
                if lightweight.engine.as_deref() != Some("claude")
                    || valid_connection.is_none_or(|connection| {
                        !connection.enabled
                            || connection.machine_id != *machine_id
                            || connection.engine != "claude"
                    })
                    || valid_model.is_none_or(|model| {
                        lightweight.effort.as_deref().is_some_and(|effort| {
                            !model.supported_efforts.iter().any(|item| item == effort)
                        })
                    })
                {
                    return Err("routing_lightweight_decision_invalid".into());
                }
            }
        }
    }

    Ok(())
}

fn validate_private_machine_registry_parent(source: &Path) -> Result<(), String> {
    let parent = source.parent().ok_or("machine_registry_parent_invalid")?;
    let metadata =
        std::fs::symlink_metadata(parent).map_err(|_| "machine_registry_parent_unavailable")?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err("machine_registry_parent_not_private".into());
    }
    Ok(())
}

/// À appeler sur l'hôte qui possède ces chemins, à Prepare puis avant RELEASE.
/// Les droits globaux d'un même UID restent hors de cette garde de mapping.
pub(crate) fn validate_unprotected_local_path(
    candidate: &Path,
    protected: &[PathBuf],
) -> Result<PathBuf, String> {
    if !mapping_root(candidate) || protected.is_empty() {
        return Err("machine_path_unverifiable".into());
    }
    let candidate = std::fs::canonicalize(candidate).map_err(|_| "machine_path_unverifiable")?;
    for path in protected {
        if !mapping_root(path) {
            return Err("machine_path_unverifiable".into());
        }
        let protected = std::fs::canonicalize(path).map_err(|_| "machine_path_unverifiable")?;
        if candidate.starts_with(&protected) || protected.starts_with(&candidate) {
            return Err("machine_path_protected".into());
        }
    }
    Ok(candidate)
}

fn validate_registry(
    agents: &BTreeMap<String, AgentDefinition>,
    source: &Path,
) -> Result<(), String> {
    for (name, definition) in agents {
        if definition.command.trim().is_empty() {
            return Err(format!(
                "registre invalide {}: command vide pour '{name}'",
                source.display()
            ));
        }
        if !matches!(
            definition.protocol.as_str(),
            "acp" | "claude_stream_json" | "codex_app_server" | "tmux"
        ) {
            return Err(format!(
                "registre invalide {}: protocol invalide pour '{name}'",
                source.display()
            ));
        }
        if matches!(name.as_str(), "glm" | "deepseek") && definition.claude_config_dir.is_none() {
            return Err(format!(
                "registre invalide {}: claude_config_dir requis pour '{name}'",
                source.display()
            ));
        }
        if let Some(profile) = definition.claude_config_dir.as_deref() {
            if definition.protocol != "claude_stream_json" {
                return Err(format!(
                    "registre invalide {}: claude_config_dir exige claude_stream_json pour '{name}'",
                    source.display()
                ));
            }
            if !Path::new(profile).is_absolute() {
                return Err(format!(
                    "registre invalide {}: claude_config_dir absolu requis pour '{name}'",
                    source.display()
                ));
            }
        }
        validate_capabilities(name, &definition.capabilities, source)?;
        if !matches!(definition.permissions.as_str(), "allow" | "deny") {
            return Err(format!(
                "registre invalide {}: permissions invalides pour '{name}'",
                source.display()
            ));
        }
        if definition.queue_capacity == 0 {
            return Err(format!(
                "registre invalide {}: queue_capacity nul pour '{name}'",
                source.display()
            ));
        }
        if !matches!(
            definition.mcp.interactive.as_str(),
            "none" | "claude" | "codex" | "unsupported"
        ) {
            return Err(format!(
                "registre invalide {}: mcp.interactive invalide pour '{name}'",
                source.display()
            ));
        }
        if definition.pass_env.len() > MAX_PASS_ENV_ENTRIES {
            return Err(format!(
                "registre invalide {}: pass_env dépasse {MAX_PASS_ENV_ENTRIES} entrées pour '{name}'",
                source.display()
            ));
        }
        if definition.claude_config_dir.is_some()
            && definition
                .pass_env
                .iter()
                .any(|name| name == "CLAUDE_CONFIG_DIR")
        {
            return Err(format!(
                "registre invalide {}: CLAUDE_CONFIG_DIR ne peut pas être hérité pour '{name}'",
                source.display()
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for variable in &definition.pass_env {
            if !valid_env_name(variable) {
                return Err(format!(
                    "registre invalide {}: variable pass_env invalide '{variable}' pour '{name}'",
                    source.display()
                ));
            }
            if !seen.insert(variable) {
                return Err(format!(
                    "registre invalide {}: variable pass_env dupliquée '{variable}' pour '{name}'",
                    source.display()
                ));
            }
            if definition.forbidden_env.contains(variable) {
                return Err(format!(
                    "registre invalide {}: '{variable}' est à la fois dans pass_env et forbidden_env pour '{name}'",
                    source.display()
                ));
            }
        }
    }
    Ok(())
}

fn validate_capabilities(
    name: &str,
    capabilities: &AdapterCapabilities,
    source: &Path,
) -> Result<(), String> {
    let mut paths = std::collections::HashSet::new();
    for path in &capabilities.execution_paths {
        if !valid_capability_value(path) {
            return Err(format!(
                "registre invalide {}: chemin de capacité invalide pour '{name}'",
                source.display()
            ));
        }
        if !paths.insert(path) {
            return Err(format!(
                "registre invalide {}: chemin de capacité dupliqué '{path}' pour '{name}'",
                source.display()
            ));
        }
    }
    for (model, model_capabilities) in &capabilities.models {
        if !valid_capability_value(model) {
            return Err(format!(
                "registre invalide {}: modèle de capacité invalide pour '{name}'",
                source.display()
            ));
        }
        let mut efforts = std::collections::HashSet::new();
        for effort in &model_capabilities.efforts {
            if !valid_capability_value(effort) {
                return Err(format!(
                    "registre invalide {}: effort de capacité invalide pour '{name}'",
                    source.display()
                ));
            }
            if !efforts.insert(effort) {
                return Err(format!(
                    "registre invalide {}: effort de capacité dupliqué '{effort}' pour '{name}'",
                    source.display()
                ));
            }
        }
    }
    Ok(())
}

fn valid_env_name(variable: &str) -> bool {
    let mut bytes = variable.bytes();
    variable.len() <= MAX_ENV_NAME_BYTES
        && bytes
            .next()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

pub(crate) fn allow_api_key_value(value: Option<&str>) -> bool {
    value == Some("1")
}

/// Retourne la première variable interdite présente dans la source. Cette
/// fonction est l'unique garde partagée par le wrapper 007 et le spawn 009.
pub(crate) fn forbidden_environment_variable(
    definition: &AgentDefinition,
    allow_api_key: bool,
    is_present: impl Fn(&str) -> bool,
) -> Option<String> {
    if allow_api_key {
        return None;
    }
    definition
        .forbidden_env
        .iter()
        .find(|variable| is_present(variable))
        .cloned()
}

fn definition(
    command: &str,
    args: &[&str],
    forbidden_env: &[&str],
    pass_env: &[&str],
    mcp_interactive: &str,
) -> AgentDefinition {
    AgentDefinition {
        command: command.to_string(),
        args: args.iter().map(ToString::to_string).collect(),
        protocol: "acp".to_string(),
        forbidden_env: forbidden_env.iter().map(ToString::to_string).collect(),
        pass_env: pass_env.iter().map(ToString::to_string).collect(),
        claude_config_dir: None,
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        held_queue: false,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: mcp_interactive.to_string(),
            acp_session: true,
        },
        capabilities: default_capabilities(),
    }
}

fn native_claude_definition() -> Result<AgentDefinition, String> {
    Ok(AgentDefinition {
        command: native_claude_command()?,
        // Même contrat que le wrapper interactif (wrapper.rs) : sans ces
        // flags le flux stream-json émet des demandes d'outil auxquelles le
        // pilote géré ne répond pas — tours clos, zéro outil.
        args: vec![
            "--model".to_string(),
            "claude-opus-5".to_string(),
            "--dangerously-skip-permissions".to_string(),
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ],
        protocol: "claude_stream_json".to_string(),
        forbidden_env: vec!["ANTHROPIC_API_KEY".to_string()],
        pass_env: [
            "CLAUDE_CONFIG_DIR",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "SSH_AUTH_SOCK",
            "HTTPS_PROXY",
            "HTTP_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        claude_config_dir: None,
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        held_queue: false,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "claude".to_string(),
            acp_session: false,
        },
        capabilities: AdapterCapabilities {
            execution_paths: vec!["claude_stream_json".to_string()],
            models: BTreeMap::from([("claude-opus-5".to_string(), ModelCapabilities::default())]),
            observed: None,
        },
    })
}

/// Résout le CLI Claude depuis le HOME du daemon afin de conserver un chemin
/// absolu sans publier le répertoire personnel d'une machine particulière.
fn native_claude_command() -> Result<String, String> {
    native_claude_command_from(std::env::var_os("HOME").map(PathBuf::from))
}

fn native_claude_command_from(home: Option<PathBuf>) -> Result<String, String> {
    let home = home.ok_or_else(|| "HOME absent pour résoudre le CLI Claude".to_string())?;
    if !home.is_absolute() {
        return Err("HOME doit être absolu pour résoudre le CLI Claude".to_string());
    }
    Ok(home
        .join(".local/bin/claude")
        .to_string_lossy()
        .into_owned())
}

fn native_codex_definition() -> AgentDefinition {
    AgentDefinition {
        command: NATIVE_CODEX_COMMAND.to_string(),
        // Même contrat que le wrapper interactif : sans ce drapeau, app-server
        // émet des requestApproval auxquelles un managed sans réponse mourait.
        // Le chemin managed applique aussi la politique via
        // `apply_managed_permission_policy` (ne pas se fier au seul registre).
        args: vec![
            "-c".to_string(),
            "model=\"gpt-5.6-terra\"".to_string(),
            "--dangerously-bypass-approvals-and-sandbox".to_string(),
            "app-server".to_string(),
        ],
        protocol: "codex_app_server".to_string(),
        forbidden_env: vec!["OPENAI_API_KEY".to_string(), "CODEX_API_KEY".to_string()],
        pass_env: [
            "CODEX_HOME",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "SSH_AUTH_SOCK",
            "NPM_CONFIG_CACHE",
            "HTTPS_PROXY",
            "HTTP_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
            "SSL_CERT_DIR",
        ]
        .into_iter()
        .map(str::to_string)
        .collect(),
        claude_config_dir: None,
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        held_queue: false,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "codex".to_string(),
            acp_session: false,
        },
        capabilities: AdapterCapabilities {
            execution_paths: vec!["codex_app_server".to_string()],
            observed: None,
            models: BTreeMap::from([("gpt-5.6-terra".to_string(), ModelCapabilities::default())]),
        },
    }
}

fn native_cursor_definition() -> AgentDefinition {
    // Le CLI Cursor choisit le modèle en interne : on affiche l'étiquette
    // honnête `auto`, jamais un nom inventé. Sans `--model`, who reste vide.
    AgentDefinition {
        command: "cursor-agent".to_string(),
        args: vec!["--model".to_string(), "auto".to_string(), "acp".to_string()],
        protocol: "acp".to_string(),
        forbidden_env: vec![
            "CURSOR_API_KEY".to_string(),
            "OPENAI_API_KEY".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
        ],
        pass_env: Vec::new(),
        claude_config_dir: None,
        permissions: "allow".to_string(),
        queue_capacity: DEFAULT_QUEUE_CAPACITY,
        held_queue: false,
        notify_timeout_secs: DEFAULT_NOTIFY_TIMEOUT_SECS,
        mcp: McpDefinition {
            interactive: "none".to_string(),
            acp_session: true,
        },
        capabilities: AdapterCapabilities {
            observed: None,
            execution_paths: vec!["acp".to_string()],
            models: BTreeMap::from([("auto".to_string(), ModelCapabilities::default())]),
        },
    }
}

/// Ajoute des définitions internes de découverte à partir des définitions
/// déclarées. Elles n'écrasent jamais le type normal et ne sont créées que
/// pour les pilotes dont l'isolation lecture seule est attestée ici.
fn add_project_discovery_definitions(agents: &mut BTreeMap<String, AgentDefinition>) {
    let sources = agents
        .iter()
        .filter(|(name, _)| !name.starts_with(PROJECT_DISCOVERY_AGENT_PREFIX))
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect::<Vec<_>>();
    for (agent_type, definition) in sources {
        let Some(discovery) = project_discovery_definition(&definition) else {
            continue;
        };
        agents.insert(
            format!("{PROJECT_DISCOVERY_AGENT_PREFIX}{agent_type}"),
            discovery,
        );
    }
}

/// Produit une définition strictement moins permissive que sa source.
///
/// Codex reçoit un bac à sable lecture seule explicite et aucun bypass.
/// Claude reçoit son mode restreint et le mode plan, qui écarte les outils
/// d'exécution et toute écriture sans décision humaine. Les autres protocoles
/// sont refusés faute de preuve équivalente.
fn project_discovery_definition(source: &AgentDefinition) -> Option<AgentDefinition> {
    let mut definition = source.clone();
    definition.permissions = "deny".to_string();
    match definition.protocol.as_str() {
        "codex_app_server" => {
            strip_codex_discovery_unsafe_arguments(&mut definition.args);
            insert_before_subcommand(
                &mut definition.args,
                "app-server",
                &["--sandbox", "read-only", "--ask-for-approval", "never"],
            );
        }
        "claude_stream_json" => {
            strip_claude_discovery_unsafe_arguments(&mut definition.args);
            definition.args.extend([
                "--restricted".to_string(),
                "--permission-mode".to_string(),
                "plan".to_string(),
            ]);
        }
        _ => return None,
    }
    Some(definition)
}

fn insert_before_subcommand(args: &mut Vec<String>, subcommand: &str, values: &[&str]) {
    let position = args
        .iter()
        .position(|argument| argument == subcommand)
        .unwrap_or(args.len());
    args.splice(
        position..position,
        values.iter().map(|value| (*value).to_string()),
    );
}

fn strip_codex_discovery_unsafe_arguments(args: &mut Vec<String>) {
    let mut cleaned = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        if matches!(
            args[index].as_str(),
            "--yolo"
                | "--dangerously-bypass-approvals-and-sandbox"
                | "--full-auto"
                | "--approve-for-me"
        ) || args[index].starts_with("--sandbox=")
            || args[index].starts_with("--ask-for-approval=")
            || args[index].starts_with("--add-dir=")
        {
            index += 1;
            continue;
        }
        if matches!(
            args[index].as_str(),
            "--sandbox" | "-s" | "--ask-for-approval" | "-a" | "--add-dir"
        ) {
            index += 2;
            continue;
        }
        cleaned.push(args[index].clone());
        index += 1;
    }
    *args = cleaned;
}

fn strip_claude_discovery_unsafe_arguments(args: &mut Vec<String>) {
    let mut cleaned = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        if args[index].contains("dangerously-skip-permissions") {
            index += 1;
            continue;
        }
        if args[index] == "--permission-mode" {
            index += 2;
            continue;
        }
        if args[index] == "bypassPermissions" {
            index += 1;
            continue;
        }
        if args[index] == "--restricted" {
            index += 1;
            continue;
        }
        cleaned.push(args[index].clone());
        index += 1;
    }
    *args = cleaned;
}

fn default_agents() -> Result<BTreeMap<String, AgentDefinition>, String> {
    Ok(BTreeMap::from([
        ("codex".to_string(), native_codex_definition()),
        ("claude".to_string(), native_claude_definition()?),
        ("cursor".to_string(), native_cursor_definition()),
        (
            "gemini".to_string(),
            definition(
                "gemini",
                &["--acp"],
                &["GEMINI_API_KEY", "GOOGLE_API_KEY"],
                &[
                    "XDG_CONFIG_HOME",
                    "XDG_CACHE_HOME",
                    "HTTPS_PROXY",
                    "HTTP_PROXY",
                    "NO_PROXY",
                ],
                "unsupported",
            ),
        ),
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_091_development_is_scoped_and_denies_extension() {
        use bridget_transport::protocol::SpawnPosture;
        let registry =
            AgentRegistry::from_json(r#"{"agents":{}}"#, "/tmp/registry-091.json").unwrap();
        let original = registry.resolved_definition("codex").unwrap();
        let scoped = registry
            .for_spawn_posture("codex", SpawnPosture::Development)
            .unwrap();
        let definition = scoped.get("codex").unwrap();
        assert_eq!(definition.permissions, "deny");
        assert!(
            definition
                .args
                .windows(2)
                .any(|pair| pair == ["--sandbox", "workspace-write"])
        );
        assert!(
            definition
                .args
                .iter()
                .any(|arg| arg == "sandbox_workspace_write.network_access=false")
        );
        assert!(!definition.args.iter().any(|arg| arg.contains("bypass")));
        assert_eq!(registry.resolved_definition("codex").unwrap(), original);
        assert!(
            registry
                .for_spawn_posture("claude", SpawnPosture::Development)
                .is_err()
        );
        let frozen = scoped.resolved_definition("codex").unwrap();
        assert_eq!(
            AgentRegistry::from_resolved("codex", &frozen)
                .unwrap()
                .resolved_definition("codex")
                .unwrap(),
            frozen
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    #[ignore = "recette avec le bac à sable du binaire Codex installé"]
    fn spec_091_native_development_writes_only_cwd_without_network() {
        use bridget_transport::protocol::SpawnPosture;
        let root =
            std::env::temp_dir().join(format!("bridget-sandbox-091-{}", uuid::Uuid::new_v4()));
        let cwd = root.join("work");
        let codex_home = root.join("codex");
        std::fs::create_dir_all(&cwd).unwrap();
        std::fs::create_dir_all(&codex_home).unwrap();
        // Un profil utilisateur permissif ne doit pas agrandir l'ordre.
        std::fs::write(codex_home.join("config.toml"), "sandbox_mode = \"danger-full-access\"\napproval_policy = \"never\"\n[sandbox_workspace_write]\nnetwork_access = true\nwritable_roots = [\"/tmp\"]\n").unwrap();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let registry =
            AgentRegistry::from_json(r#"{"agents":{}}"#, root.join("agents.json")).unwrap();
        let scoped = registry
            .for_spawn_posture("codex", SpawnPosture::Development)
            .unwrap();
        let definition = scoped.get("codex").unwrap();
        let mut args = definition.args.clone();
        args.retain(|arg| arg != "app-server");
        let script = format!(
            "touch inside && if touch ../outside; then exit 91; fi; if /usr/bin/curl --noproxy '*' --connect-timeout 2 --max-time 2 http://{}/; then exit 92; fi",
            listener.local_addr().unwrap()
        );
        let output = std::process::Command::new(&definition.command)
            .args(args)
            .args(["sandbox", "/bin/sh", "-c", &script])
            .current_dir(&cwd)
            .env("CODEX_HOME", &codex_home)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(cwd.join("inside").exists());
        assert!(!root.join("outside").exists());
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    use std::os::unix::fs::symlink;

    fn test_root(label: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("bridget-registry-{label}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn write_registry(path: &Path, mode: u32) {
        std::fs::write(path, "{\"agents\":{}}").unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    fn remote162_registry_value() -> serde_json::Value {
        serde_json::json!({"machines":{"cartae":{
            "worker_id":"worker-cartae", "provisioning_revision":1,
            "credential_fingerprint":format!("{:x}", Sha256::digest(b"private-worker-credential")),
            "allowed_agent_types":["codex"],
            "mappings":{"cartae":{
                "source_host":"mac", "source_root":"/Users/moi/projects/cartae",
                "remote_root":"/home/moi/20.cartae", "expected_git_identity":"guthubrx/cartae",
                "mapping_revision":1
            }}
        }}})
    }

    fn admission163_registry_value() -> serde_json::Value {
        serde_json::json!({
            "historical_future_key": {"preserved": true},
            "decision_services": {
                "laya": {
                    "machine_id": "decision-host",
                    "protocol": "laya_systemone_v1",
                    "model": "typed-decisions",
                    "endpoint": "https://decision.internal/v1",
                }
            },
            "admission_policy": {
                "revision": 1,
                "profiles": [{
                    "id": "local-codex",
                    "machine_id": "local",
                    "agent_type": "codex",
                    "model": "gpt-5.6",
                    "effort": "high",
                    "description": "Revue locale"
                }]
            }
        })
    }

    #[test]
    fn spec163_configuration_admission_est_stricte_et_desactivee_par_defaut() {
        let value = admission163_registry_value();
        let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
        let policy = registry.admission_policy().unwrap();
        assert!(!policy.auto_engine);
        assert!(!policy.auto_placement);
        assert_eq!(policy.profiles.len(), 1);
        assert_eq!(
            registry.decision_service("laya").unwrap_err(),
            "decision_service_disabled"
        );
        assert_eq!(registry.decision_services()["laya"].health_ttl_seconds, 30);

        let mut unknown = value.clone();
        unknown["decision_services"]["laya"]["secret_ref"] = "forbidden".into();
        assert!(AgentRegistry::from_json(&unknown.to_string(), "/tmp/agents.json").is_err());

        let mut profile_limit = value.clone();
        let profile = profile_limit["admission_policy"]["profiles"][0].clone();
        profile_limit["admission_policy"]["profiles"] =
            serde_json::Value::Array(vec![profile; MAX_ADMISSION_PROFILES + 1]);
        assert!(AgentRegistry::from_json(&profile_limit.to_string(), "/tmp/agents.json").is_err());

        let mut reference = value;
        reference["admission_policy"]["fixed_profile"] = "missing".into();
        assert_eq!(
            AgentRegistry::from_json(&reference.to_string(), "/tmp/agents.json").unwrap_err(),
            "admission_profile_reference_unknown"
        );
    }

    fn routing168_registry_value() -> serde_json::Value {
        let mut value = remote162_registry_value();
        value["machines"]["cartae"]["enabled"] = true.into();
        value["agents"] = serde_json::json!({
            "codex-personal": {"command":"codex", "protocol":"codex_app_server"},
            "codex-work": {"command":"codex", "protocol":"codex_app_server"},
            "claude": {"command":"claude", "protocol":"claude_stream_json"}
        });
        value["routing_policy"] = serde_json::json!({
            "version": 2,
            "revision": 1,
            "enabled": true,
            "connections": [
                {"connection_id":"codex-personal", "machine_id":"local", "engine":"codex", "agent_type":"codex-personal", "enabled":true},
                {"connection_id":"codex-work", "machine_id":"cartae", "engine":"codex", "agent_type":"codex-work", "enabled":true},
                {"connection_id":"claude-local", "machine_id":"local", "engine":"claude", "agent_type":"claude", "enabled":true},
                {"connection_id":"claude-cartae", "machine_id":"cartae", "engine":"claude", "agent_type":"claude", "enabled":true}
            ],
            "model_catalog": {
                "codex-personal": [
                    {"model":"gpt-6.1-sol", "supported_efforts":["low","high"], "default_effort":"high", "provenance":"operator_observation"},
                    {"model":"gpt-6.1-luna", "supported_efforts":[], "default_effort":null, "provenance":"codex_model_list"},
                    {"model":"gpt-6.1-terra", "supported_efforts":["medium"], "default_effort":"medium", "provenance":"codex_model_list"}
                ],
                "codex-work": [
                    {"model":"gpt-6.1-sol", "supported_efforts":["low","high"], "default_effort":"high", "provenance":"operator_observation"},
                    {"model":"gpt-6.1-luna", "supported_efforts":[], "default_effort":null, "provenance":"codex_model_list"},
                    {"model":"gpt-6.1-terra", "supported_efforts":["medium"], "default_effort":"medium", "provenance":"codex_model_list"}
                ],
                "claude-local": [
                    {"model":"claude-sonnet-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                    {"model":"claude-opus-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                    {"model":"claude-haiku-4-5", "supported_efforts":[], "default_effort":null, "provenance":"declared"}
                ],
                "claude-cartae": [
                    {"model":"claude-sonnet-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                    {"model":"claude-opus-4-5", "supported_efforts":["high"], "default_effort":"high", "provenance":"declared"},
                    {"model":"claude-haiku-4-5", "supported_efforts":[], "default_effort":null, "provenance":"declared"}
                ]
            },
            "machine_policies": {
                "local": {"enabled":true, "decision_timeout_ms":10000, "service_timeout_ms":2000},
                "cartae": {"enabled":true, "decision_timeout_ms":10000, "service_timeout_ms":2000}
            }
        });
        value
    }

    #[test]
    fn routing168_catalogue_12_preserve_connexions_efforts_et_defaut_natif() {
        let registry = AgentRegistry::from_json(
            &routing168_registry_value().to_string(),
            "/tmp/agents-routing168.json",
        )
        .unwrap();
        let catalogue = registry.routing_catalogue(1_700_000_000).unwrap();
        assert_eq!(catalogue.version, 1);
        assert_eq!(catalogue.options.len(), 12);
        assert_eq!(
            registry.routing_policy().unwrap().machine_policies["local"].compilation_capacity,
            None,
            "absence de déclaration reste inconnue ; aucune vitesse n'est déduite"
        );
        assert!(catalogue.options.iter().any(|option| {
            option.connection_id == "codex-personal"
                && option.model == "gpt-6.1-sol"
                && option
                    .supported_efforts
                    .iter()
                    .map(String::as_str)
                    .eq(["low", "high"])
                && option.default_effort.as_deref() == Some("high")
        }));
        assert!(catalogue.options.iter().any(|option| {
            option.connection_id == "codex-work"
                && option.model == "gpt-6.1-luna"
                && option.supported_efforts.is_empty()
                && option.default_effort.is_none()
        }));
        assert!(catalogue.options.iter().any(|option| {
            option.connection_id == "claude-local"
                && option.model == "claude-sonnet-4-5"
                && option.entitlement == RoutingEntitlement::Unknown
        }));
    }

    #[test]
    fn routing168_129_meme_desactive_refuse_sans_catalogue_partiel() {
        let mut value = routing168_registry_value();
        value["routing_policy"]["connections"].as_array_mut().unwrap().push(
            serde_json::json!({"connection_id":"overflow-disabled", "machine_id":"cartae", "engine":"codex", "agent_type":"codex-work", "enabled":false}),
        );
        value["routing_policy"]["model_catalog"]["overflow-disabled"] = serde_json::Value::Array(
            (0..129)
                .map(|index| {
                    serde_json::json!({
                        "model": format!("declared-overflow-{index}"),
                        "supported_efforts": [],
                        "default_effort": null,
                        "provenance": "declared"
                    })
                })
                .collect(),
        );
        assert_eq!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json")
                .unwrap_err(),
            "candidate_limit"
        );
    }

    #[test]
    fn routing168_axes_independants_fixes_et_contradiction_restent_des_filtres() {
        let mut value = routing168_registry_value();
        value["routing_policy"]["machine_policies"]["local"]["automatic_axes"] = serde_json::json!({"engine":false, "model":true, "machine":true, "connection":true, "effort":true});
        value["routing_policy"]["machine_policies"]["local"]["fixed_values"] =
            serde_json::json!({"engine":"codex"});
        let registry =
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").unwrap();
        let selected = registry
            .select_routing_candidates(
                "local",
                &RoutingCandidateFilters {
                    model: Some("gpt-6.1-sol".into()),
                    ..RoutingCandidateFilters::default()
                },
            )
            .unwrap();
        assert_eq!(selected.len(), 2, "un modèle conserve les deux machines");
        assert_eq!(selected[0].candidate.machine_id, "cartae");
        assert_eq!(selected[1].candidate.machine_id, "local");
        assert!(
            registry
                .select_routing_candidates(
                    "local",
                    &RoutingCandidateFilters {
                        connection_id: Some("claude-local".into()),
                        ..RoutingCandidateFilters::default()
                    },
                )
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn routing168_decideurs_inaccessibles_sont_valides_et_claude_distant_refuse() {
        let mut value = routing168_registry_value();
        value["decision_services"] = serde_json::json!({"laya":{"machine_id":"local", "protocol":"laya_systemone_v1", "model":"routing", "endpoint":"https://decision.internal/v1"}});
        value["routing_policy"]["machine_policies"]["local"]["decision_service"] =
            serde_json::json!({"enabled":true, "accessible":false, "service_id":"laya"});
        value["routing_policy"]["machine_policies"]["local"]["lightweight_decision"] = serde_json::json!({"enabled":true, "accessible":false, "engine":"claude", "connection_id":"claude-local", "model":"claude-sonnet-4-5", "effort":"high"});
        assert!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").is_ok()
        );

        value["routing_policy"]["machine_policies"]["local"]["lightweight_decision"]["connection_id"] =
            "claude-cartae".into();
        assert_eq!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json")
                .unwrap_err(),
            "routing_lightweight_decision_invalid"
        );
    }

    #[test]
    fn spec168_telemetrie_cpu_charge_seuils_1_100_fermes() {
        let mut value = routing168_registry_value();
        value["routing_policy"]["machine_policies"]["local"]["telemetry"] =
            serde_json::json!({"enabled":true});
        let defaults = RoutingScoreWeights::default();
        let registry =
            AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-defaults.json").unwrap();
        assert_eq!(
            registry.routing_policy().unwrap().machine_policies["local"]
                .telemetry
                .as_ref()
                .unwrap()
                .weights,
            defaults
        );
        let default_policy = serde_json::to_vec(registry.routing_policy().unwrap()).unwrap();
        let old_snapshot: serde_json::Value = serde_json::from_slice(&default_policy).unwrap();
        assert!(
            old_snapshot["machine_policies"]["local"]["telemetry"]
                .get("weights")
                .is_none(),
            "le JSON historique sans poids reste sans ce champ"
        );
        let restored: RoutingPolicy = serde_json::from_slice(&default_policy).unwrap();
        assert_eq!(
            serde_json::to_vec(&restored).unwrap(),
            default_policy,
            "ancien snapshot sans weights : roundtrip byte-identique et digest conservé"
        );
        value["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
            serde_json::to_value(&defaults).unwrap();
        let explicit =
            AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-defaults.json").unwrap();
        let explicit_policy = serde_json::to_vec(explicit.routing_policy().unwrap()).unwrap();
        assert_eq!(
            default_policy, explicit_policy,
            "poids omis et défauts explicites figent les mêmes octets de politique/digest"
        );
        let default_digest = Sha256::digest(&default_policy);

        value["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
            serde_json::json!({"cpu":7});
        let partial =
            AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-weights.json").unwrap();
        assert_eq!(
            partial.routing_policy().unwrap().machine_policies["local"]
                .telemetry
                .as_ref()
                .unwrap()
                .weights,
            RoutingScoreWeights {
                cpu: 7,
                ..defaults.clone()
            },
            "les champs d'un objet partiel gardent leurs défauts"
        );
        let changed_policy = serde_json::to_vec(partial.routing_policy().unwrap()).unwrap();
        let changed_snapshot: serde_json::Value = serde_json::from_slice(&changed_policy).unwrap();
        assert_eq!(
            changed_snapshot["machine_policies"]["local"]["telemetry"]["weights"]["cpu"], 7,
            "les poids non par défaut restent explicitement figés"
        );
        assert_ne!(
            Sha256::digest(&changed_policy),
            default_digest,
            "les poids configurés sont portés par la politique sérialisée du snapshot/digest"
        );

        for (field, cap) in [
            ("cpu", 20),
            ("ram", 20),
            ("load", 20),
            ("continuity", 20),
            ("compilation", 10),
            ("quota", 5),
            ("reset", 3),
        ] {
            for valid in [0, cap] {
                let mut configured = value.clone();
                configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
                    serde_json::json!({});
                configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]
                    [field] = serde_json::json!(valid);
                assert!(
                    AgentRegistry::from_json(
                        &configured.to_string(),
                        "/tmp/routing168-weights.json"
                    )
                    .is_ok(),
                    "{field}={valid}"
                );
            }
            for invalid in [cap + 1, 255] {
                let mut configured = value.clone();
                configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] =
                    serde_json::json!({});
                configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]
                    [field] = serde_json::json!(invalid);
                assert_eq!(
                    AgentRegistry::from_json(
                        &configured.to_string(),
                        "/tmp/routing168-weights.json"
                    )
                    .unwrap_err(),
                    "routing_score_weights_invalid",
                    "{field}={invalid}"
                );
            }
        }
        let mut zero = value.clone();
        zero["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"] = serde_json::json!({"cpu":0,"ram":0,"load":0,"continuity":0,"compilation":0,"quota":0,"reset":0});
        assert!(
            AgentRegistry::from_json(&zero.to_string(), "/tmp/routing168-weights.json").is_ok()
        );
        zero["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]["saturation"] =
            serde_json::json!(0);
        assert!(
            AgentRegistry::from_json(&zero.to_string(), "/tmp/routing168-weights.json").is_err(),
            "la pénalité100 n'est jamais réglable, même quand les autres poids valent0"
        );
        for invalid in [
            serde_json::json!(-1),
            serde_json::json!(256),
            serde_json::json!(1.5),
            serde_json::json!("7"),
            serde_json::Value::Null,
        ] {
            let mut configured = value.clone();
            configured["routing_policy"]["machine_policies"]["local"]["telemetry"]["weights"]["cpu"] =
                invalid;
            assert!(
                AgentRegistry::from_json(&configured.to_string(), "/tmp/routing168-weights.json")
                    .is_err(),
                "chaque poids exige un entier borné"
            );
        }

        for field in [
            "cpu_saturation_percent",
            "normalized_load_saturation_percent",
        ] {
            for invalid in [0, 101, 255] {
                let mut value = routing168_registry_value();
                value["routing_policy"]["machine_policies"]["local"]["telemetry"] =
                    serde_json::json!({"enabled":true});
                value["routing_policy"]["machine_policies"]["local"]["telemetry"][field] =
                    serde_json::json!(invalid);
                assert_eq!(
                    AgentRegistry::from_json(&value.to_string(), "/tmp/routing168-invalid.json")
                        .unwrap_err(),
                    "routing_telemetry_invalid"
                );
            }
        }
    }

    #[test]
    fn routing168_capacite_compilation_est_declarative_et_bornee() {
        let mut value = routing168_registry_value();
        value["routing_policy"]["machine_policies"]["local"]["compilation_capacity"] = 10.into();
        assert!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json").is_ok()
        );
        value["routing_policy"]["machine_policies"]["local"]["compilation_capacity"] = 11.into();
        assert_eq!(
            AgentRegistry::from_json(&value.to_string(), "/tmp/agents-routing168.json")
                .unwrap_err(),
            "routing_machine_policy_invalid"
        );
    }

    #[test]
    fn remote162_absence_et_defaut_desactive_ne_creent_aucune_route() {
        let old = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        assert!(old.machines().is_empty());
        assert_eq!(old.machine("cartae").unwrap_err(), "machine_unknown");
        let value = remote162_registry_value();
        let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
        assert_eq!(registry.machine("cartae").unwrap_err(), "machine_disabled");
        let machine = &registry.machines()["cartae"];
        assert!(!machine.permits_agent_type("codex"));
        assert!(
            machine
                .matches_credential(&IdentityCredential::new("private-worker-credential".into()))
        );
        assert!(!machine.matches_credential(&IdentityCredential::new("another-credential".into())));
        assert!(registry.project_mapping("cartae", "cartae").is_err());
    }

    #[test]
    fn remote162_route_explicitement_activee_authentifie_sans_secret_dans_registre() {
        let mut value = remote162_registry_value();
        value["machines"]["cartae"]["enabled"] = true.into();
        value["machines"]["cartae"]["display_name"] = "Serveur Cartae".into();
        let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
        let machine = registry.machine("cartae").unwrap();
        assert_eq!(machine.display_name.as_deref(), Some("Serveur Cartae"));
        assert!(machine.permits_agent_type("codex"));
        assert!(!machine.permits_agent_type("claude"));
        assert!(
            machine
                .matches_credential(&IdentityCredential::new("private-worker-credential".into()))
        );
        assert!(!machine.matches_credential(&IdentityCredential::new("another-credential".into())));
        assert!(!format!("{registry:?}").contains("private-worker-credential"));
        assert_eq!(
            registry
                .project_mapping("cartae", "cartae")
                .unwrap()
                .remote_root,
            Path::new("/home/moi/20.cartae")
        );
        assert_eq!(
            registry.project_mapping("cartae", "missing").unwrap_err(),
            "project_mapping_unknown"
        );
        // Aucune résolution de la racine Linux n'est tentée sur le maître.
    }

    #[test]
    fn remote162_definitions_fournisseur_figees_excluent_autorite_machine() {
        let mut value = remote162_registry_value();
        value["machines"]["cartae"]["enabled"] = true.into();
        let registry = AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").unwrap();
        let resolved = registry.resolved_definition("codex").unwrap();
        assert!(
            AgentRegistry::from_resolved("codex", &resolved)
                .unwrap()
                .machines()
                .is_empty()
        );
        assert!(
            registry
                .for_spawn_posture(
                    "codex",
                    bridget_transport::protocol::SpawnPosture::Development
                )
                .unwrap()
                .machines()
                .is_empty()
        );
        assert!(registry.machine("cartae").is_ok());
    }

    #[test]
    fn remote162_configuration_invalide_refusee_atomiquement() {
        for (field, bad) in [
            ("provisioning_revision", serde_json::json!(0)),
            (
                "provisioning_revision",
                serde_json::json!(MAX_MACHINE_REVISION + 1),
            ),
            ("credential_fingerprint", serde_json::json!("not-a-digest")),
            ("worker_id", serde_json::json!("worker\nforged")),
            ("display_name", serde_json::json!("machine\nforged")),
            ("allowed_agent_types", serde_json::json!(["codex", "codex"])),
            ("credential", serde_json::json!("must-not-be-stored")),
        ] {
            let mut value = remote162_registry_value();
            value["machines"]["cartae"][field] = bad;
            assert!(
                AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err(),
                "{field}"
            );
        }
        for root in ["relative", "/home/moi/../private", "/home/moi/\u{0}"] {
            let mut value = remote162_registry_value();
            value["machines"]["cartae"]["mappings"]["cartae"]["remote_root"] = root.into();
            assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
        }
        assert!(AgentRegistry::from_json("{invalid", "/tmp/agents.json").is_err());
    }

    #[test]
    fn remote162_plafonds_et_identite_worker_unique() {
        let mut value = remote162_registry_value();
        let machine = value["machines"]["cartae"].clone();
        value["machines"]["duplicate"] = machine.clone();
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
        value["machines"] = serde_json::json!({});
        for index in 0..MAX_MACHINES {
            let mut item = machine.clone();
            item["worker_id"] = format!("worker-{index}").into();
            value["machines"][format!("machine-{index}")] = item;
        }
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_ok());
        value["machines"]["extra"] = machine;
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
        let mut value = remote162_registry_value();
        let mapping = value["machines"]["cartae"]["mappings"]["cartae"].clone();
        value["machines"]["cartae"]["mappings"] = serde_json::json!({});
        for index in 0..MAX_MACHINE_MAPPINGS {
            value["machines"]["cartae"]["mappings"][format!("project-{index}")] = mapping.clone();
        }
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_ok());
        value["machines"]["cartae"]["mappings"]["extra"] = mapping;
        assert!(AgentRegistry::from_json(&value.to_string(), "/tmp/agents.json").is_err());
    }

    #[test]
    fn remote162_registre_sur_disque_exige_fichier_et_parent_prives() {
        let root = test_root("remote162-private");
        let path = root.join("agents.json");
        std::fs::write(&path, remote162_registry_value().to_string()).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            AgentRegistry::load_from_path(path.clone()).unwrap_err(),
            "machine_registry_parent_not_private"
        );
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            AgentRegistry::load_from_path(path.clone())
                .unwrap()
                .machines()
                .len(),
            1
        );
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(AgentRegistry::load_from_path(path.clone()).is_err());
        let link = root.join("link.json");
        symlink(&path, &link).unwrap();
        assert!(AgentRegistry::load_from_path(link).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let parent_link = root.join("parent-link");
        symlink(&root, &parent_link).unwrap();
        assert_eq!(
            AgentRegistry::load_from_path(parent_link.join("agents.json")).unwrap_err(),
            "machine_registry_parent_not_private"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remote162_chemins_proteges_resolus_sur_hote_refusent_tout_chevauchement() {
        let root = test_root("remote162-paths");
        let private = root.join("worker");
        let project = root.join("project");
        std::fs::create_dir_all(private.join("slot")).unwrap();
        std::fs::create_dir(&project).unwrap();
        let protected = vec![private.clone()];
        for path in [&root, &private, &private.join("slot")] {
            assert_eq!(
                validate_unprotected_local_path(path, &protected).unwrap_err(),
                "machine_path_protected"
            );
        }
        let link = root.join("disguised-project");
        symlink(&private, &link).unwrap();
        assert_eq!(
            validate_unprotected_local_path(&link, &protected).unwrap_err(),
            "machine_path_protected"
        );
        assert!(validate_unprotected_local_path(&project, &protected).is_ok());
        assert!(validate_unprotected_local_path(&root.join("absent"), &protected).is_err());
        assert!(
            validate_unprotected_local_path(&project, &[root.join("unknown-private")]).is_err()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn defaults_cover_the_three_priorities() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let codex = registry.get("codex").unwrap();
        assert_eq!(codex.command, NATIVE_CODEX_COMMAND);
        assert!(Path::new(&codex.command).is_absolute());
        assert_eq!(
            codex.args,
            vec![
                "-c",
                "model=\"gpt-5.6-terra\"",
                "--dangerously-bypass-approvals-and-sandbox",
                "app-server",
            ]
        );
        assert_eq!(codex.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
        assert_eq!(codex.protocol, "codex_app_server");
        assert_eq!(codex.permissions, "allow");
        assert_eq!(codex.queue_capacity, 32);
        assert_eq!(codex.notify_timeout_secs, DEFAULT_NOTIFY_TIMEOUT_SECS);
        assert!(codex.pass_env.contains(&"CODEX_HOME".to_string()));
        assert!(!codex.pass_env.contains(&"OPENAI_API_KEY".to_string()));
        assert_eq!(registry.get("gemini").unwrap().args, vec!["--acp"]);
        assert_eq!(codex.mcp.interactive, "codex");
        assert!(!codex.mcp.acp_session);
        assert_eq!(codex.capabilities.execution_paths, vec!["codex_app_server"]);
        assert_eq!(
            codex.capabilities.models.get("gpt-5.6-terra"),
            Some(&ModelCapabilities::default())
        );
        let claude = registry.get("claude").unwrap();
        let expected_claude_command = native_claude_command().unwrap();
        assert_eq!(claude.command, expected_claude_command);
        assert!(Path::new(&claude.command).is_absolute());
        assert_eq!(claude.protocol, "claude_stream_json");
        assert_eq!(
            claude.args,
            [
                "--model",
                "claude-opus-5",
                "--dangerously-skip-permissions",
                "--permission-mode",
                "bypassPermissions",
            ]
        );
        assert!(!claude.mcp.acp_session);
        assert_eq!(
            claude.capabilities.execution_paths,
            vec!["claude_stream_json"]
        );
        assert_eq!(
            claude.capabilities.models.get("claude-opus-5"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(
            registry.get("gemini").unwrap().mcp.interactive,
            "unsupported"
        );
        let cursor = registry.get("cursor").unwrap();
        assert_eq!(cursor.command, "cursor-agent");
        assert_eq!(cursor.args, vec!["--model", "auto", "acp"]);
        assert_eq!(cursor.protocol, "acp");
        assert_eq!(
            cursor.capabilities.models.get("auto"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(
            runtime_model_and_effort(&cursor.args),
            Some(("auto".to_string(), None))
        );
    }

    #[test]
    fn spec_076_coordinateurs_decouverte_sont_derives_et_restreints() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let original_codex = registry.get("codex").unwrap();
        let discovery_codex = registry.get("project-discovery-codex").unwrap();
        assert_eq!(original_codex.permissions, "allow");
        assert_eq!(discovery_codex.permissions, "deny");
        assert!(
            discovery_codex
                .args
                .windows(2)
                .any(|arguments| arguments == ["--sandbox", "read-only"]),
            "le coordinateur Codex doit être enfermé en lecture seule"
        );
        assert!(
            discovery_codex
                .args
                .iter()
                .all(|argument| argument != "--dangerously-bypass-approvals-and-sandbox"),
            "aucun bypass Codex ne doit survivre"
        );

        let discovery_claude = registry.get("project-discovery-claude").unwrap();
        assert_eq!(discovery_claude.permissions, "deny");
        assert!(
            discovery_claude
                .args
                .iter()
                .any(|argument| argument == "--restricted")
        );
        assert!(
            discovery_claude
                .args
                .iter()
                .any(|argument| argument == "plan")
        );
        assert!(
            discovery_claude
                .args
                .iter()
                .all(|argument| !argument.contains("dangerously-skip-permissions")),
            "aucun bypass Claude ne doit survivre"
        );
        assert!(
            registry.project_discovery_agent_type("cursor").is_none(),
            "Cursor n'est pas proposé sans preuve de lecture seule"
        );
    }

    #[test]
    fn commande_claude_refuse_home_absent_au_point_de_derivation() {
        assert_eq!(
            native_claude_command_from(None).unwrap_err(),
            "HOME absent pour résoudre le CLI Claude"
        );
        assert_eq!(
            native_claude_command_from(Some(PathBuf::from("home-relatif"))).unwrap_err(),
            "HOME doit être absolu pour résoudre le CLI Claude"
        );
        assert_eq!(
            native_claude_command_from(Some(PathBuf::from("/srv/bridget-test"))).unwrap(),
            "/srv/bridget-test/.local/bin/claude"
        );
    }

    #[test]
    fn etiquette_modele_auto_lue_depuis_les_args_cursor() {
        // Sans --model, who reste vide — c'est le trou constaté sur les
        // anciennes définitions. Avec --model auto, l'étiquette est honnête.
        let observed = runtime_model_and_effort(&[
            "--model".to_string(),
            "auto".to_string(),
            "acp".to_string(),
        ]);
        assert_eq!(observed, Some(("auto".to_string(), None)));
        assert!(
            runtime_model_and_effort(&["acp".to_string()]).is_none(),
            "sans --model, who reste vide"
        );
    }

    #[test]
    fn bypass_permissions_change_le_digest_de_la_definition_claude() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let with_bypass = registry.resolved_definition("claude").unwrap();
        let mut without = registry.get("claude").unwrap().clone();
        without.args = vec!["--model".to_string(), "claude-opus-5".to_string()];
        let old = resolved_definition(&without).unwrap();
        assert_ne!(
            with_bypass.digest, old.digest,
            "ajouter les flags bypass doit changer le digest figé"
        );
        assert!(
            with_bypass
                .args
                .iter()
                .any(|a| a == "--dangerously-skip-permissions")
        );
        assert!(with_bypass.args.iter().any(|a| a == "bypassPermissions"));
    }

    #[test]
    fn user_entry_replaces_its_default() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"custom","protocol":"tmux"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        assert_eq!(registry.get("codex").unwrap().command, "custom");
        assert!(registry.get("codex").unwrap().forbidden_env.is_empty());
    }

    #[test]
    fn registre_sans_bascule_charge_les_definitions_par_defaut() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        assert!(registry.get("codex").is_ok());
    }

    #[test]
    fn remplacement_sans_forbidden_env_signale_la_perte_de_garde() {
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"custom"}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents().unwrap(),
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("codex"));
        assert!(warnings[0].contains("sans déclarer forbidden_env"));

        let explicit = registry_warnings(
            r#"{"agents":{"codex":{"command":"custom","forbidden_env":[]}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents().unwrap(),
        );
        assert!(explicit.is_empty());
    }

    #[test]
    fn definition_resolue_est_complete_et_son_digest_est_deterministe() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let first = registry.resolved_definition("codex").unwrap();
        let second = registry.resolved_definition("codex").unwrap();
        assert_eq!(first, second);
        assert_eq!(first.command, NATIVE_CODEX_COMMAND);
        assert_eq!(
            first.args,
            vec![
                "-c",
                "model=\"gpt-5.6-terra\"",
                "--dangerously-bypass-approvals-and-sandbox",
                "app-server",
            ]
        );
        assert_eq!(first.protocol, "codex_app_server");
        assert_eq!(first.forbidden_env, vec!["OPENAI_API_KEY", "CODEX_API_KEY"]);
        assert!(first.pass_env.contains(&"CODEX_HOME".to_string()));
        assert_eq!(first.permissions, "allow");
        assert_eq!(first.queue_capacity, 32);
        assert_eq!(first.notify_timeout_secs, DEFAULT_NOTIFY_TIMEOUT_SECS);
        assert_eq!(first.mcp.interactive, "codex");
        assert!(!first.mcp.acp_session);
        assert_eq!(
            first.capabilities.models.get("gpt-5.6-terra"),
            Some(&ModelCapabilities::default())
        );
        assert_eq!(first.digest.len(), 64);

        let baseline = default_agents().unwrap().remove("codex").unwrap();
        let mut mutations = Vec::new();
        let mut changed = baseline.clone();
        changed.command = "other".to_string();
        mutations.push(("command", changed));
        let mut changed = baseline.clone();
        changed.args.push("other".to_string());
        mutations.push(("args", changed));
        let mut changed = baseline.clone();
        changed.protocol = "tmux".to_string();
        mutations.push(("protocol", changed));
        let mut changed = baseline.clone();
        changed.forbidden_env.push("OTHER_KEY".to_string());
        mutations.push(("forbidden_env", changed));
        let mut changed = baseline.clone();
        changed.pass_env.push("OTHER_HOME".to_string());
        mutations.push(("pass_env", changed));
        let mut changed = baseline.clone();
        changed.permissions = "deny".to_string();
        mutations.push(("permissions", changed));
        let mut changed = baseline.clone();
        changed.queue_capacity += 1;
        mutations.push(("queue_capacity", changed));
        let mut changed = baseline.clone();
        changed.notify_timeout_secs += 1;
        mutations.push(("notify_timeout_secs", changed));
        let mut changed = baseline.clone();
        changed.mcp.interactive = "claude".to_string();
        mutations.push(("mcp.interactive", changed));
        let mut changed = baseline.clone();
        changed.mcp.acp_session = !changed.mcp.acp_session;
        mutations.push(("mcp.acp_session", changed));
        let mut changed = baseline.clone();
        changed.capabilities.models.insert(
            "gpt-5.6-terra".to_string(),
            ModelCapabilities {
                efforts: vec!["high".to_string()],
            },
        );
        mutations.push(("capabilities.models", changed));
        for (field, changed) in mutations {
            assert_ne!(
                first.digest,
                resolved_definition(&changed).unwrap().digest,
                "le digest doit changer avec {field}"
            );
        }
        assert!(AgentRegistry::from_resolved("codex", &first).is_ok());
        let mut forged = first;
        forged.queue_capacity += 1;
        assert!(AgentRegistry::from_resolved("codex", &forged).is_err());
    }

    #[test]
    fn profil_claude_est_absolu_reserve_au_transport_claude_et_fige_dans_le_digest() {
        let root = test_root("claude-config-dir");
        let profile = root.join("profile");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::set_permissions(&profile, std::fs::Permissions::from_mode(0o700)).unwrap();
        let profile = profile.to_string_lossy().into_owned();
        let registry = AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "glm": {
                        "command": "claude",
                        "protocol": "claude_stream_json",
                        "claude_config_dir": profile
                    }
                }
            })
            .to_string(),
            "/tmp/agents.json",
        )
        .unwrap();
        let resolved = registry.resolved_definition("glm").unwrap();
        assert_eq!(
            resolved.claude_config_dir.as_deref(),
            Some(profile.as_str())
        );

        let without_profile = AgentRegistry::from_json(
            r#"{"agents":{"anthropic":{"command":"claude","protocol":"claude_stream_json"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap()
        .resolved_definition("anthropic")
        .unwrap();
        assert_ne!(resolved.digest, without_profile.digest);

        let missing_profile = AgentRegistry::from_json(
            r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
        assert!(missing_profile.contains("claude_config_dir requis"));

        let relative = AgentRegistry::from_json(
            r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json","claude_config_dir":"relative"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
        assert!(relative.contains("claude_config_dir absolu"));

        let wrong_transport = AgentRegistry::from_json(
            r#"{"agents":{"cursor":{"command":"cursor-agent","protocol":"acp","claude_config_dir":"/tmp/profile"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
        assert!(wrong_transport.contains("claude_stream_json"));

        let inherited = AgentRegistry::from_json(
            r#"{"agents":{"glm":{"command":"claude","protocol":"claude_stream_json","claude_config_dir":"/tmp/profile","pass_env":["CLAUDE_CONFIG_DIR"]}}}"#,
            "/tmp/agents.json",
        )
        .unwrap_err();
        assert!(inherited.contains("CLAUDE_CONFIG_DIR"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn definition_figee_historique_reste_reprise_avec_la_capacite_acp_de_migration() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let definition = registry.get("codex").unwrap().clone();
        let mut legacy = resolved_definition(&definition).unwrap();
        legacy.digest = legacy_resolved_digest(&definition).unwrap();
        let mut value = serde_json::to_value(&legacy).unwrap();
        value.as_object_mut().unwrap().remove("capabilities");
        let decoded: ResolvedAgentDefinition = serde_json::from_value(value).unwrap();
        assert_eq!(decoded.capabilities, AdapterCapabilities::default());
        assert!(AgentRegistry::from_resolved("codex", &decoded).is_ok());
    }

    #[test]
    fn definition_figee_avant_profil_conserve_ses_capacites_a_la_reprise() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let definition = registry.get("codex").unwrap().clone();
        let mut before_profile = resolved_definition(&definition).unwrap();
        before_profile.digest = pre_profile_resolved_digest(&definition).unwrap();
        let mut value = serde_json::to_value(&before_profile).unwrap();
        value.as_object_mut().unwrap().remove("claude_config_dir");
        let decoded: ResolvedAgentDefinition = serde_json::from_value(value).unwrap();

        let restored = AgentRegistry::from_resolved("codex", &decoded).unwrap();
        assert_eq!(
            restored.get("codex").unwrap().capabilities,
            definition.capabilities
        );
    }

    #[test]
    fn lecture_privee_refuse_permissions_larges_et_accepte_0600() {
        let root = test_root("permissions");
        let path = root.join("agents.json");
        write_registry(&path, 0o644);
        let error = AgentRegistry::load_from_path(path.clone()).unwrap_err();
        assert!(error.contains("permissions 0644"));
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(AgentRegistry::load_from_path(path).is_ok());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lecture_privee_refuse_un_lien_symbolique() {
        let root = test_root("symlink");
        let target = root.join("target.json");
        let link = root.join("agents.json");
        write_registry(&target, 0o600);
        symlink(&target, &link).unwrap();
        let error = AgentRegistry::load_from_path(link).unwrap_err();
        assert!(error.contains("liens symboliques sont interdits"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn empty_command_is_rejected() {
        let result =
            AgentRegistry::from_json(r#"{"agents":{"codex":{"command":""}}}"#, "/tmp/agents.json");
        assert!(result.unwrap_err().contains("command vide"));
    }

    #[test]
    fn invalid_protocol_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","protocol":"bad"}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("protocol invalide"));
    }

    #[test]
    fn invalid_mcp_interactive_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","mcp":{"interactive":"global-file"}}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("mcp.interactive invalide"));
    }

    #[test]
    fn matrice_de_capacites_refuse_les_champs_inconnus_et_les_doublons() {
        let unknown = AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp"],"future":true}}}}"#,
            "/tmp/agents.json",
        );
        assert!(unknown.unwrap_err().contains("unknown field `future`"));

        let duplicate = AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","capabilities":{"execution_paths":["acp","acp"]}}}}"#,
            "/tmp/agents.json",
        );
        assert!(
            duplicate
                .unwrap_err()
                .contains("chemin de capacité dupliqué")
        );
    }

    #[test]
    fn invalid_permissions_are_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","permissions":"ask"}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("permissions invalides"));
    }

    #[test]
    fn zero_queue_capacity_is_rejected() {
        let result = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"codex","queue_capacity":0}}}"#,
            "/tmp/agents.json",
        );
        assert!(result.unwrap_err().contains("queue_capacity nul"));
    }

    #[test]
    fn unknown_type_names_its_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.get("inconnu").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains("codex"));
    }

    #[test]
    fn unknown_entry_key_is_reported() {
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"codex","forbidden_env":[],"surprise":true}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents().unwrap(),
        );
        assert_eq!(
            warnings,
            vec!["clé inconnue 'surprise' pour 'codex' dans /tmp/agents.json"]
        );
    }

    #[test]
    fn valid_entry_keys_produce_no_warning() {
        let warnings = registry_warnings(
            r#"{"agents":{"codex":{"command":"codex","args":[],"protocol":"acp","forbidden_env":[],"pass_env":["CODEX_HOME"],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600}}}"#,
            Path::new("/tmp/agents.json"),
            &default_agents().unwrap(),
        );
        assert!(warnings.is_empty());
    }

    #[test]
    fn pass_env_est_borne_et_valide_atomiquement() {
        for (json, detail) in [
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["INVALIDE-TIRET"]}}}"#,
                "variable pass_env invalide",
            ),
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["HOME","HOME"]}}}"#,
                "pass_env dupliquée",
            ),
            (
                r#"{"agents":{"x":{"command":"x","pass_env":["API_KEY"],"forbidden_env":["API_KEY"]}}}"#,
                "à la fois dans pass_env et forbidden_env",
            ),
        ] {
            let error = AgentRegistry::from_json(json, "/tmp/agents.json").unwrap_err();
            assert!(error.contains(detail), "erreur inattendue: {error}");
        }
        let entries = (0..=MAX_PASS_ENV_ENTRIES)
            .map(|index| format!("VAR_{index}"))
            .collect::<Vec<_>>();
        let json = serde_json::json!({"agents":{"x":{"command":"x","pass_env":entries}}});
        let error = AgentRegistry::from_json(&json.to_string(), "/tmp/agents.json").unwrap_err();
        assert!(error.contains("pass_env dépasse"));
    }

    #[test]
    fn generic_codex_command_resolves_through_the_registry() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        assert_eq!(registry.type_for_command("codex").unwrap(), "codex");
    }

    #[test]
    fn native_custom_command_resolves_without_its_internal_discovery_variant() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"provider":{"command":"/tmp/custom-provider","protocol":"claude_stream_json","args":["--model","fixture"]}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        // Le type restreint reste disponible pour la politique de lancement,
        // mais ne constitue pas un deuxième fournisseur déclaré par l'humain.
        assert_eq!(
            registry
                .get("project-discovery-provider")
                .unwrap()
                .permissions,
            "deny"
        );
        assert_eq!(
            registry.type_for_command("/tmp/custom-provider").unwrap(),
            "provider"
        );
        assert_eq!(
            registry.type_for_command("custom-provider").unwrap(),
            "provider"
        );
    }

    #[test]
    fn historic_interactive_aliases_remain_available() {
        for (command, agent_type) in [
            ("codex", "codex"),
            ("claude", "claude"),
            ("gemini", "gemini"),
            ("gclaude", "claude"),
            ("claude-son", "claude"),
        ] {
            assert_eq!(AgentRegistry::interactive_alias(command), Some(agent_type));
        }
    }

    #[test]
    fn ambiguous_command_is_refused_without_map_order_fallback() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"one":{"command":"bridge"},"two":{"command":"bridge"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let error = registry.type_for_command("bridge").unwrap_err();
        assert!(error.contains("commande ambiguë 'bridge'"));
        assert!(error.contains("one"));
        assert!(error.contains("two"));
    }

    #[test]
    fn undeclared_generic_command_names_source_and_choices() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        let error = registry.type_for_command("not-declared").unwrap_err();
        assert!(error.contains("/tmp/agents.json"));
        assert!(error.contains(&format!("codex ({NATIVE_CODEX_COMMAND})")));
    }

    #[test]
    fn pont_zed_codex_est_refuse_avant_lancement() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"legacy":{"command":"npx","args":["@zed-industries/codex-acp@0.16.0","-c","model=\"gpt-5.6-terra\""],"protocol":"acp"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let definition = registry.get("legacy").unwrap();
        let refusal = reject_retired_zed_bridge(definition).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/codex-acp")
        ));
        assert!(matches!(
            validate_launch_capabilities("legacy", definition),
            Err(SpawnRefusal::EnvUnfit { .. })
        ));
    }

    #[test]
    fn pont_zed_refuse_les_quatre_formes_de_chemin_vivantes() {
        // C1 revue G10 : basename + retrait @version — les formes qui
        // échappaient à la comparaison naïve `== "codex-acp"`.
        let forms = [
            (
                "/opt/homebrew/bin/codex-acp",
                Vec::<&str>::new(),
                "@zed-industries/codex-acp",
            ),
            (
                "/opt/bridget-fixtures/bin/claude-code-acp",
                vec![],
                "@zed-industries/claude-code-acp",
            ),
            (
                "./node_modules/.bin/codex-acp",
                vec![],
                "@zed-industries/codex-acp",
            ),
            ("npx", vec!["codex-acp@0.16.0"], "@zed-industries/codex-acp"),
        ];
        for (command, args, package) in forms {
            let args_json = serde_json::to_string(&args).unwrap();
            let json = format!(
                r#"{{"agents":{{"legacy":{{"command":"{command}","args":{args_json},"protocol":"acp"}}}}}}"#
            );
            let registry = AgentRegistry::from_json(&json, "/tmp/agents.json").unwrap();
            let definition = registry.get("legacy").unwrap();
            let refusal = reject_retired_zed_bridge(definition).unwrap_err();
            assert!(
                matches!(
                    &refusal,
                    SpawnRefusal::EnvUnfit { detail } if detail.contains(package)
                ),
                "forme non refusée: command={command} args={args:?} → {refusal:?}"
            );
        }
    }

    #[test]
    fn pont_zed_claude_est_refuse_avant_lancement() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{"legacy":{"command":"claude-code-acp","protocol":"acp"}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let definition = registry.get("legacy").unwrap();
        let refusal = reject_retired_zed_bridge(definition).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("@zed-industries/claude-code-acp")
        ));
    }

    #[test]
    fn acp_generique_cursor_et_gemini_restent_admis() {
        let registry = AgentRegistry::from_json(
            r#"{"agents":{
                "cursor":{"command":"cursor-agent","args":["--model","auto","acp"],"protocol":"acp","capabilities":{"execution_paths":["acp"],"models":{"auto":{"efforts":[]}}}},
                "gemini":{"command":"gemini","args":["--acp"],"protocol":"acp"}
            }}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        reject_retired_zed_bridge(registry.get("cursor").unwrap()).unwrap();
        reject_retired_zed_bridge(registry.get("gemini").unwrap()).unwrap();
        validate_launch_capabilities("cursor", registry.get("cursor").unwrap()).unwrap();
        // gemini sans modèle explicite : la matrice accepte le chemin seul
        validate_launch_capabilities("gemini", registry.get("gemini").unwrap()).unwrap();
    }

    #[test]
    fn defauts_natifs_codex_et_claude_ne_passent_pas_par_zed() {
        let registry = AgentRegistry::from_json("{}", "/tmp/agents.json").unwrap();
        for name in ["codex", "claude"] {
            let definition = registry.get(name).unwrap();
            reject_retired_zed_bridge(definition).unwrap();
            assert!(!definition.command.contains("npx"));
            assert!(
                !definition
                    .args
                    .iter()
                    .any(|arg| zed_bridge_token(arg).is_some())
            );
            assert_ne!(definition.protocol, "acp");
        }
    }

    /// Contrôle positif Cc d'abord. Puis chaîne portant un vrai bidi.
    /// Preuve que le test meurt sans la garde Cf : U+202E n'est pas `is_control`.
    #[test]
    fn oracle_capability_refuse_une_valeur_bidi_reelle() {
        assert!(
            !valid_capability_value("model\u{0007}x"),
            "PROMESSE — un Cc (BEL) DOIT être refusé"
        );
        assert!(
            valid_capability_value("gpt-5.6-terra"),
            "PROMESSE — une valeur légitime DOIT passer"
        );
        let bidi = "model\u{202e}ledom";
        assert!(
            !bidi.chars().any(char::is_control),
            "preuve d'aveuglement Cc : sans is_format_character cette chaîne passerait"
        );
        assert!(
            !valid_capability_value(bidi),
            "oracle — une capacité portant U+202E DOIT être refusée"
        );
        assert!(
            !valid_capability_value("claude\u{2066}opus"),
            "oracle — isolat U+2066 (Trojan Source) DOIT être refusé"
        );
    }
}

#[cfg(test)]
#[test]
fn held159_hold_ferme_identique() {
    let omitted: AgentDefinition = serde_json::from_str(
        r#"{"command":"/bin/sh","args":["-c","printf raw159"],"queue_capacity":7}"#,
    )
    .unwrap();
    let closed: AgentDefinition=serde_json::from_str(r#"{"command":"/bin/sh","args":["-c","printf raw159"],"queue_capacity":7,"held_queue":false}"#).unwrap();
    let enabled: AgentDefinition=serde_json::from_str(r#"{"command":"/bin/sh","args":["-c","printf raw159"],"queue_capacity":7,"held_queue":true}"#).unwrap();
    assert_eq!(omitted, closed, "omission signifie réglage fermé");
    assert_eq!(closed.args, vec!["-c", "printf raw159"]);
    assert_eq!(closed.queue_capacity, 7);
    assert_ne!(
        enabled, closed,
        "le réglage ouvert doit être conservé par le registre"
    );
    // Le chargement réel lit un fichier privé ; le réglage reste propre au daemon.
    let root = std::env::temp_dir().join(format!("h159-reg-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("agents.json");
    let mut frozen = Vec::new();
    for (flag, expected) in [(None, false), (Some(false), false), (Some(true), true)] {
        let mut value = serde_json::json!({"agents":{"held159":{"command":"/bin/sh",
            "args":["-c","printf raw159"],"queue_capacity":7}}});
        if let Some(flag) = flag {
            value["agents"]["held159"]["held_queue"] = serde_json::json!(flag);
        }
        let content = value.to_string();
        std::fs::write(&path, &content).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let loaded = AgentRegistry::load_from_path(path.clone()).unwrap();
        let definition = loaded.get("held159").unwrap();
        assert_eq!(
            definition.held_queue, expected,
            "champ conservé par le vrai chargement"
        );
        assert_eq!(definition.args, closed.args);
        assert_eq!(definition.queue_capacity, closed.queue_capacity);
        frozen.push(loaded.resolved_definition("held159").unwrap());
        assert!(
            registry_warnings(&content, &path, &default_agents().unwrap()).is_empty(),
            "held_queue est une clé connue du registre"
        );
    }
    assert_eq!(
        frozen[0], frozen[1],
        "définition fournisseur inchangée quand fermé"
    );
    assert_eq!(
        frozen[0], frozen[2],
        "réglage daemon absent de la définition et du digest fournisseur"
    );
    std::fs::remove_dir_all(root).unwrap();
}
