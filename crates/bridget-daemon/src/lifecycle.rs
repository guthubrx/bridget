//! Frontière des ordres de cycle de vie et préparation contrôlée du spawn.
//!
//! Ce module ne lance aucun processus : T906 consomme `PreparedSpawn` avec le
//! bootstrap supervisé. Il concentre déjà les gardes et leur ordre afin que
//! chaque refus terminal passe par la même saga idempotente que le succès.

use crate::fleet::{
    FleetError, FleetSupervisor, RecoveryCandidate, SpawnLease, SpawnOrder, SpawnSubmission,
    SpawnWaiter,
};
use crate::idempotency::SpawnCommandIssue;
use crate::registry::{
    AgentDefinition, AgentRegistry, allow_api_key_value, forbidden_environment_variable,
    validate_launch_capabilities,
};
use bridget_transport::{ResolvedAgentDefinition, SpawnRefusal};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

const BASELINE_ENV: &[&str] = &["HOME", "PATH", "USER", "LANG", "TMPDIR"];
const FALLBACK_PATH: &str = "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

pub type SourceEnvironment = BTreeMap<String, OsString>;

#[derive(Debug, Clone)]
pub struct PreparedSpawn {
    pub lease: SpawnLease,
    pub agent_type: String,
    pub command: String,
    pub args: Vec<String>,
    pub resolved_definition: Box<ResolvedAgentDefinition>,
    pub cwd: PathBuf,
    pub env: SourceEnvironment,
}

#[derive(Debug, Clone)]
pub enum SpawnDecision {
    Ready(PreparedSpawn),
    Await(SpawnWaiter),
    Accepted {
        name: String,
        definition: Option<ResolvedAgentDefinition>,
    },
    Rejected(SpawnRefusal),
    EnvelopeMismatch,
}

pub fn source_environment() -> SourceEnvironment {
    std::env::vars_os()
        .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
        .collect()
}

fn validate_provider_observation(
    agent_type: &str,
    definition: &AgentDefinition,
) -> Result<(), SpawnRefusal> {
    let Some(observed) = definition.capabilities.observed.as_ref() else {
        return Ok(());
    };
    let refusal = |capability: &str| SpawnRefusal::UnsupportedCapability {
        agent_type: agent_type.to_string(),
        model: "<non déclaré>".to_string(),
        capability: capability.to_string(),
    };
    if observed.binary_path.trim().is_empty()
        || !Path::new(&observed.binary_path).is_absolute()
        || observed.binary_path != definition.command
    {
        return Err(refusal("source du binaire fournisseur observée"));
    }
    if observed.binary_version.trim().is_empty() {
        return Err(refusal("version du binaire fournisseur observée"));
    }
    if observed.contract_version.trim().is_empty() {
        return Err(refusal("version du contrat fournisseur observée"));
    }
    if observed.binary_digest.len() != 64
        || !observed
            .binary_digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(refusal("empreinte du binaire fournisseur observé"));
    }
    let bytes = std::fs::read(&observed.binary_path)
        .map_err(|_| refusal("binaire fournisseur observé lisible"))?;
    let actual_digest = format!("{:x}", Sha256::digest(bytes));
    if actual_digest != observed.binary_digest {
        return Err(refusal("empreinte du binaire fournisseur observé"));
    }
    Ok(())
}

/// Les deux machines d'un ordre de lancement : celle dont le système de
/// fichiers est réellement interrogé, et celle qui a demandé.
///
/// Elles diffèrent dès qu'un agent fédéré demande un lancement : la commande
/// traverse le tunnel et le daemon **maître** valide le `cwd` chez LUI. Le refus
/// disait alors « répertoire de travail disparu » sans dire où il avait cherché.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnHosts {
    pub searched_on: String,
    pub requested_from: String,
}

impl SpawnHosts {
    /// Demandeur et exécutant confondus — chemin de reprise, où le daemon
    /// relance ses propres agents chez lui.
    pub fn local() -> Self {
        let host = crate::build_info::local_host();
        SpawnHosts {
            searched_on: host.clone(),
            requested_from: host,
        }
    }
}

/// Applique le lookup/rejeu idempotent avant toute garde mutable. Une
/// réservation neuve est ensuite soit préparée pour T906, soit terminée avec
/// l'un des onze motifs fermés du contrat.
pub fn submit_spawn(
    supervisor: &FleetSupervisor,
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    recovering: bool,
    hosts: &SpawnHosts,
) -> Result<SpawnDecision, FleetError> {
    submit_spawn_with_policy(
        supervisor, registry, source, order, now, recovering, hosts, false, false,
    )
}

/// Le moteur projet a été retiré. Une référence historique n'est ni effacée
/// ni interprétée comme une autorisation de lancement sur l'hôte.
fn reject_removed_project(
    project: Option<&bridget_transport::protocol::ProjectReference>,
) -> Result<(), SpawnRefusal> {
    if let Some(project) = project {
        return Err(SpawnRefusal::DockerRuntimeUnavailable {
            project_id: project.project_id.clone(),
        });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn submit_spawn_with_policy(
    supervisor: &FleetSupervisor,
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    recovering: bool,
    hosts: &SpawnHosts,
    relaunch: bool,
    recovery: bool,
) -> Result<SpawnDecision, FleetError> {
    // Le refus précède toute réservation NEUVE. Une clé connue conserve son
    // canon et son issue durable : relire une réussite passée ne relance rien.
    if !supervisor.knows_command(&order.command_id)
        && let Err(reason) = reject_removed_project(order.project.as_ref())
    {
        return Ok(SpawnDecision::Rejected(reason));
    }
    if (relaunch || recovery)
        && !supervisor.knows_command(&order.command_id)
        && let Some(name) = order.requested_name.as_deref()
        && let Some(entry) = supervisor.desired_entry(name)?
    {
        // Une ancienne exécution conteneur peut avoir perdu son binding ou
        // sa référence projet. Son propre relevé reste une interdiction : le
        // demandeur ne peut pas contourner la garde en omettant order.project.
        if let Some(project_id) = entry
            .project
            .as_ref()
            .map(|p| &p.project_id)
            .or_else(|| entry.runtime_execution.as_ref().map(|e| &e.project_id))
        {
            return Ok(SpawnDecision::Rejected(
                SpawnRefusal::DockerRuntimeUnavailable {
                    project_id: project_id.clone(),
                },
            ));
        }
    }
    if recovering && !supervisor.knows_command(&order.command_id) {
        return Ok(SpawnDecision::Rejected(SpawnRefusal::DaemonRecovering));
    }
    if let Some(name) = order.requested_name.as_deref()
        && !order.persistent
        && !supervisor.knows_command(&order.command_id)
    {
        log::warn!(
            "{}",
            crate::recovery_trace::non_persistent_spawn_warning(name)
        );
    }
    // Une capacité absente est une propriété du registre déclaratif, pas une
    // issue de la saga. Pour une commande neuve, elle est donc refusée avant
    // toute réservation durable et, a fortiori, avant toute création de
    // processus. Une commande connue conserve son rejeu figé ci-dessous.
    if !supervisor.knows_command(&order.command_id)
        && let Ok(definition) = registry.get(&order.agent_type)
        && let Err(reason) = validate_launch_capabilities(&order.agent_type, definition)
            .and_then(|_| validate_provider_observation(&order.agent_type, definition))
    {
        return Ok(SpawnDecision::Rejected(reason));
    }
    // L'exécutable est toujours vérifié sur l'hôte : aucun chemin alternatif
    // ne peut désormais désactiver cette garde avant réservation.
    if !supervisor.knows_command(&order.command_id)
        && let Ok(definition) = registry.get(&order.agent_type)
        && let Ok(env) = build_environment(definition, source)
        && !command_exists(&definition.command, &env)
    {
        return Ok(SpawnDecision::Rejected(SpawnRefusal::CommandMissing {
            command: definition.command.clone(),
            registry: registry.source().display().to_string(),
        }));
    }
    let submission = if relaunch {
        supervisor.request_relaunch(order, now)?
    } else if recovery {
        supervisor.request_recovery(order, now)?
    } else {
        supervisor.request_spawn(order, now)?
    };
    match submission {
        SpawnSubmission::Start(lease) => {
            let prepared = match prepare_spawn(registry, source, order, lease.clone(), hosts) {
                Ok(prepared) => prepared,
                Err(reason) => {
                    let (category, detail) = refusal_record(&reason);
                    supervisor.fail(&lease, category, detail)?;
                    return Ok(SpawnDecision::Rejected(reason));
                }
            };
            supervisor.mark_starting(&lease, now, &prepared.resolved_definition)?;
            Ok(SpawnDecision::Ready(prepared))
        }
        SpawnSubmission::Await(waiter) => Ok(SpawnDecision::Await(waiter)),
        SpawnSubmission::Terminal(issue) => Ok(decision_from_issue(issue, supervisor.quota())),
        SpawnSubmission::EnvelopeMismatch => Ok(SpawnDecision::EnvelopeMismatch),
        SpawnSubmission::IdempotencyExpired => {
            Ok(SpawnDecision::Rejected(SpawnRefusal::IdempotencyExpired))
        }
    }
}

/// Variante réservée à la reprise d'une entrée persistante déjà connectée :
/// elle crée une nouvelle saga, mais sa préparation consomme la définition
/// durable de `fleet.json` plutôt que le registre courant.
pub fn submit_spawn_from_resolved(
    supervisor: &FleetSupervisor,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    resolved: &ResolvedAgentDefinition,
    hosts: &SpawnHosts,
) -> Result<SpawnDecision, FleetError> {
    let registry = AgentRegistry::from_resolved(&order.agent_type, resolved)
        .map_err(|_| FleetError::InvalidOrder("définition figée de reprise invalide"))?;
    submit_spawn_with_policy(
        supervisor, &registry, source, order, now, false, hosts, false, true,
    )
}

/// Relance explicite d'une entrée arrêtée à partir de sa définition figée. Le
/// chemin de préparation reste identique au spawn, mais la réservation admet
/// uniquement le nom stopped déjà présent dans l'inventaire.
pub fn submit_relaunch_from_resolved(
    supervisor: &FleetSupervisor,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    resolved: &ResolvedAgentDefinition,
    hosts: &SpawnHosts,
) -> Result<SpawnDecision, FleetError> {
    let registry = AgentRegistry::from_resolved(&order.agent_type, resolved)
        .map_err(|_| FleetError::InvalidOrder("définition figée de relance invalide"))?;
    submit_spawn_with_policy(
        supervisor, &registry, source, order, now, false, hosts, true, false,
    )
}

fn prepare_spawn(
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    lease: SpawnLease,
    hosts: &SpawnHosts,
) -> Result<PreparedSpawn, SpawnRefusal> {
    prepare_spawn_parts(
        registry,
        source,
        &order.agent_type,
        &order.cwd,
        lease,
        hosts,
    )
}

/// Reprépare une génération persistante restée en vol sans repasser par la
/// réservation idempotente ni relire le registre mutable. La définition
/// persistée est l'unique autorité de lancement de cette génération.
pub fn prepare_recovery(
    source: &SourceEnvironment,
    candidate: RecoveryCandidate,
) -> Result<PreparedSpawn, SpawnRefusal> {
    reject_removed_project(candidate.lease.project.as_ref())?;
    if let Some(execution) = candidate.runtime_execution.as_ref() {
        return Err(SpawnRefusal::DockerRuntimeUnavailable {
            project_id: execution.project_id.clone(),
        });
    }
    let resolved =
        candidate
            .resolved_definition
            .ok_or_else(|| SpawnRefusal::NegotiationFailed {
                detail: "définition figée absente de la génération à reprendre".to_string(),
            })?;
    let frozen_registry = AgentRegistry::from_resolved(&candidate.agent_type, &resolved)
        .map_err(|detail| SpawnRefusal::NegotiationFailed { detail })?;
    prepare_spawn_parts(
        &frozen_registry,
        source,
        &candidate.agent_type,
        &candidate.cwd,
        candidate.lease,
        &SpawnHosts::local(),
    )
}

fn prepare_spawn_parts(
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    agent_type: &str,
    cwd: &Path,
    lease: SpawnLease,
    hosts: &SpawnHosts,
) -> Result<PreparedSpawn, SpawnRefusal> {
    reject_removed_project(lease.project.as_ref())?;
    let definition = registry
        .get(agent_type)
        .map_err(|_| SpawnRefusal::UnknownType {
            requested_type: agent_type.to_string(),
            known_types: registry.known_types(),
            registry: registry.source().display().to_string(),
        })?;
    validate_launch_capabilities(agent_type, definition)?;
    validate_provider_observation(agent_type, definition)?;
    if let Some(variable) = forbidden_environment_variable(
        definition,
        allow_api_key_value(
            source
                .get("BRIDGET_ALLOW_API_KEY")
                .and_then(|value| value.to_str()),
        ),
        |variable| source.contains_key(variable),
    ) {
        return Err(SpawnRefusal::BillingGuard { variable });
    }
    if !matches!(
        definition.protocol.as_str(),
        "acp" | "claude_stream_json" | "codex_app_server"
    ) {
        return Err(SpawnRefusal::NegotiationFailed {
            detail: format!(
                "le protocole '{}' n'utilise pas une session gérée",
                definition.protocol
            ),
        });
    }
    if !cwd.is_dir() {
        return Err(SpawnRefusal::CwdGone {
            searched_on: hosts.searched_on.clone(),
            requested_from: hosts.requested_from.clone(),
        });
    }
    let mut env = build_environment(definition, source)?;
    if lease.persistent {
        env.insert(
            "BRIDGET_MANAGED_PERSISTENT".to_string(),
            OsString::from("1"),
        );
    }
    if let Ok(max) = std::env::var("BRIDGET_PROVIDER_RELAUNCH_MAX") {
        env.insert(
            "BRIDGET_PROVIDER_RELAUNCH_MAX".to_string(),
            OsString::from(max),
        );
    }
    if !command_exists(&definition.command, &env) {
        return Err(SpawnRefusal::CommandMissing {
            command: definition.command.clone(),
            registry: registry.source().display().to_string(),
        });
    }
    Ok(PreparedSpawn {
        lease,
        agent_type: agent_type.to_string(),
        command: definition.command.clone(),
        args: definition.args.clone(),
        resolved_definition: Box::new(
            registry
                .resolved_definition(agent_type)
                .map_err(|detail| SpawnRefusal::NegotiationFailed { detail })?,
        ),
        cwd: cwd.to_path_buf(),
        env,
    })
}

pub fn build_environment(
    definition: &AgentDefinition,
    source: &SourceEnvironment,
) -> Result<SourceEnvironment, SpawnRefusal> {
    let home = source.get("HOME").ok_or_else(|| SpawnRefusal::EnvUnfit {
        detail: "HOME absent de l'environnement du daemon".to_string(),
    })?;
    if !Path::new(home).is_absolute() {
        return Err(SpawnRefusal::EnvUnfit {
            detail: "HOME n'est pas absolu".to_string(),
        });
    }
    let mut env = SourceEnvironment::new();
    for name in BASELINE_ENV {
        if let Some(value) = source.get(*name) {
            env.insert((*name).to_string(), value.clone());
        }
    }
    // Configuration IPC opérateur, distincte des capacités et des secrets
    // fournisseur. Le bootstrap vide son environnement : ne pas perdre la
    // racine du daemon et rejoindre implicitement une autre instance.
    if source.contains_key("BRIDGET_HOME") || source.contains_key("BRIDGET_SOCKET") {
        let namespace = crate::environment::Namespace::resolve(
            source.get("BRIDGET_HOME").map(PathBuf::from),
            source.get("BRIDGET_SOCKET").map(PathBuf::from),
            Some(PathBuf::from(home)),
        )
        .map_err(|detail| SpawnRefusal::EnvUnfit { detail })?;
        for (name, value) in namespace.child_environment() {
            env.insert(name.to_string_lossy().into_owned(), value);
        }
    }
    env.entry("PATH".to_string())
        .or_insert_with(|| OsString::from(FALLBACK_PATH));
    if let Some(path) = env.get("PATH").cloned() {
        env.insert("PATH".to_string(), prepend_current_exe_dir(&path));
    }
    env.entry("TMPDIR".to_string())
        .or_insert_with(|| OsString::from("/tmp"));
    for name in &definition.pass_env {
        if let Some(value) = source.get(name) {
            env.insert(name.clone(), value.clone());
        }
    }
    if let Some(profile) = definition.claude_config_dir.as_deref() {
        validate_claude_profile_directory(profile)?;
        env.insert("CLAUDE_CONFIG_DIR".to_string(), OsString::from(profile));
    }
    Ok(env)
}

fn validate_claude_profile_directory(profile: &str) -> Result<(), SpawnRefusal> {
    let path = Path::new(profile);
    let metadata = fs::symlink_metadata(path).map_err(|err| SpawnRefusal::EnvUnfit {
        detail: format!("profil Claude indisponible {}: {err}", path.display()),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil Claude invalide {}: répertoire réel requis",
                path.display()
            ),
        });
    }
    if metadata.uid() != unsafe { libc::geteuid() } {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil Claude invalide {}: propriétaire inattendu",
                path.display()
            ),
        });
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil Claude invalide {}: permissions {mode:04o}, attendu 0700 ou plus restrictif",
                path.display()
            ),
        });
    }
    Ok(())
}

/// Intention : le répertoire du binaire courant est le **premier** élément du
/// PATH, pour que `bridget` gagne la résolution même s'il figure déjà ailleurs.
/// Les entrées vides (POSIX = « répertoire courant ») sont retirées du reste :
/// un spawn géré ne doit pas résoudre via `.` implicite.
pub fn path_with_current_exe_dir_first(existing: &str) -> Option<String> {
    let directory = std::env::current_exe()
        .ok()?
        .parent()?
        .to_string_lossy()
        .into_owned();
    let rest: Vec<&str> = existing
        .split(':')
        .map(str::trim)
        .filter(|entry| !entry.is_empty() && *entry != directory.as_str())
        .collect();
    if rest.is_empty() {
        Some(directory)
    } else {
        Some(format!("{directory}:{}", rest.join(":")))
    }
}

fn prepend_current_exe_dir(path: &OsString) -> OsString {
    match path_with_current_exe_dir_first(&path.to_string_lossy()) {
        Some(prefixed) => OsString::from(prefixed),
        None => path.clone(),
    }
}

fn command_exists(command: &str, env: &SourceEnvironment) -> bool {
    let is_executable = |path: &Path| {
        path.metadata()
            .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    };
    if command.contains('/') {
        return is_executable(Path::new(command));
    }
    env.get("PATH")
        .and_then(|path| path.to_str())
        .is_some_and(|path| {
            path.split(':')
                .map(Path::new)
                .map(|dir| dir.join(command))
                .any(|candidate| is_executable(&candidate))
        })
}

fn refusal_record(reason: &SpawnRefusal) -> (&'static str, String) {
    let category = match reason {
        SpawnRefusal::UnknownType { .. } => "unknown_type",
        SpawnRefusal::CommandMissing { .. } => "command_missing",
        SpawnRefusal::UnsupportedCapability { .. } => "unsupported_capability",
        SpawnRefusal::BillingGuard { .. } => "billing_guard",
        SpawnRefusal::NameActive => "name_active",
        SpawnRefusal::EnvUnfit { .. } => "env_unfit",
        SpawnRefusal::CwdGone { .. } => "cwd_gone",
        SpawnRefusal::ProjectCwdMismatch { .. } => "project_cwd_mismatch",
        SpawnRefusal::DockerRuntimeUnavailable { .. } => "docker_runtime_unavailable",
        SpawnRefusal::NegotiationFailed { .. } => "negotiation_failed",
        SpawnRefusal::SpawnTimeout => "spawn_timeout",
        SpawnRefusal::QuotaExceeded { .. } => "quota_exceeded",
        SpawnRefusal::DaemonRecovering => "daemon_recovering",
        SpawnRefusal::IdempotencyExpired => "idempotency_expired",
    };
    (
        category,
        serde_json::to_string(reason).expect("SpawnRefusal est toujours sérialisable"),
    )
}

fn decision_from_issue(issue: SpawnCommandIssue, quota: usize) -> SpawnDecision {
    match issue {
        SpawnCommandIssue::Connected {
            name, definition, ..
        } => SpawnDecision::Accepted {
            name,
            definition: definition.map(|value| *value),
        },
        SpawnCommandIssue::Cancelled { reason } if reason == "spawn_timeout" => {
            SpawnDecision::Rejected(SpawnRefusal::SpawnTimeout)
        }
        SpawnCommandIssue::Cancelled { reason } => {
            SpawnDecision::Rejected(SpawnRefusal::NegotiationFailed { detail: reason })
        }
        SpawnCommandIssue::Failed { category, reason } => {
            if let Ok(refusal) = serde_json::from_str::<SpawnRefusal>(&reason) {
                return SpawnDecision::Rejected(refusal);
            }
            let refusal = match category.as_str() {
                "unknown_type" => SpawnRefusal::UnknownType {
                    requested_type: String::new(),
                    known_types: Vec::new(),
                    registry: "registre de l'issue initiale".to_string(),
                },
                "command_missing" => SpawnRefusal::CommandMissing {
                    command: reason,
                    registry: "registre de l'issue initiale".to_string(),
                },
                "unsupported_capability" => serde_json::from_str(&reason)
                    .unwrap_or(SpawnRefusal::NegotiationFailed { detail: reason }),
                "billing_guard" => SpawnRefusal::BillingGuard { variable: reason },
                "name_active" => SpawnRefusal::NameActive,
                "env_unfit" => SpawnRefusal::EnvUnfit { detail: reason },
                // Reconstruction depuis une issue ancienne : les hôtes ne sont
                // pas dans la catégorie. On ne les invente pas.
                "cwd_gone" => SpawnRefusal::CwdGone {
                    searched_on: String::new(),
                    requested_from: String::new(),
                },
                "project_cwd_mismatch" => SpawnRefusal::ProjectCwdMismatch {
                    project_id: String::new(),
                },
                "spawn_timeout" => SpawnRefusal::SpawnTimeout,
                "quota_exceeded" => SpawnRefusal::QuotaExceeded { limit: quota },
                "daemon_recovering" => SpawnRefusal::DaemonRecovering,
                "idempotency_expired" => SpawnRefusal::IdempotencyExpired,
                _ => SpawnRefusal::NegotiationFailed { detail: reason },
            };
            SpawnDecision::Rejected(refusal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desired_state::DesiredStateStore;
    use crate::fleet::FleetConfig;
    use std::fs;

    const NOW: i64 = 2_000_000;

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t905-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn supervisor(root: &Path, quota: usize) -> FleetSupervisor {
        fs::create_dir_all(root).unwrap();
        FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota,
                persistent_horizon_secs: 3600,
                ephemeral_horizon_secs: 300,
                issued_at_tolerance_secs: 30,
            },
        )
        .unwrap()
    }

    fn registry(command: &str, protocol: &str, forbidden: &[&str]) -> AgentRegistry {
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": command,
                        "protocol": protocol,
                        "forbidden_env": forbidden,
                        "pass_env": ["SPECIAL_AUTH"]
                    }
                }
            })
            .to_string(),
            "/tmp/t905-agents.json",
        )
        .unwrap()
    }

    fn registry_with_capabilities(args: &[&str], capabilities: serde_json::Value) -> AgentRegistry {
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": "/bin/sh",
                        "args": args,
                        "protocol": "acp",
                        "forbidden_env": [],
                        "pass_env": ["SPECIAL_AUTH"],
                        "capabilities": capabilities
                    }
                }
            })
            .to_string(),
            "/tmp/t905-capabilities.json",
        )
        .unwrap()
    }

    fn source(home: &Path) -> SourceEnvironment {
        BTreeMap::from([
            ("HOME".to_string(), home.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("fr_FR.UTF-8")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
            ("SPECIAL_AUTH".to_string(), OsString::from("présent")),
        ])
    }

    /// Deux machines DISTINCTES : c'est la situation fédérée réelle, et c'est
    /// la seule où l'oracle du point 5 peut distinguer les deux hôtes.
    fn hosts_fixture() -> SpawnHosts {
        SpawnHosts {
            searched_on: "poste-alpha".to_string(),
            requested_from: "poste-beta".to_string(),
        }
    }

    fn order(root: &Path, id: &str, name: &str) -> SpawnOrder {
        SpawnOrder {
            posture: None,
            agent_type: "fixture".to_string(),
            requested_name: Some(name.to_string()),
            cwd: root.to_path_buf(),
            persistent: false,
            command_id: id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 10,
            ownership: None,
            project: None,
        }
    }

    fn rejection(decision: SpawnDecision) -> SpawnRefusal {
        match decision {
            SpawnDecision::Rejected(reason) => reason,
            other => panic!("refus attendu, obtenu: {other:?}"),
        }
    }

    #[test]
    fn environnement_construit_ne_copie_que_baseline_et_pass_env() {
        let root = root("env");
        fs::create_dir_all(&root).unwrap();
        let definition = registry("/bin/sh", "acp", &[])
            .get("fixture")
            .unwrap()
            .clone();
        let mut source = source(&root);
        source.insert("SECRET_INATTENDU".to_string(), OsString::from("non"));
        let env = build_environment(&definition, &source).unwrap();
        assert_eq!(env.get("SPECIAL_AUTH"), Some(&OsString::from("présent")));
        assert!(!env.contains_key("SECRET_INATTENDU"));
        let directory = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let path = env.get("PATH").unwrap().to_string_lossy();
        // Intention : le binaire courant doit gagner la résolution (premier élément).
        assert_eq!(
            path.split(':').map(str::trim).next(),
            Some(directory.as_str()),
            "PATH géré sans préfixe du binaire courant: {path}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn profil_claude_prive_ecrase_toute_valeur_ambiante_et_refuse_les_droits_larges() {
        let root = root("claude-profile");
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        fs::set_permissions(&profile, fs::Permissions::from_mode(0o700)).unwrap();
        let mut definition = registry("/bin/sh", "claude_stream_json", &[])
            .get("fixture")
            .unwrap()
            .clone();
        definition.claude_config_dir = Some(profile.to_string_lossy().into_owned());
        let mut source = source(&root);
        source.insert("CLAUDE_CONFIG_DIR".to_string(), OsString::from("/ambient"));
        let env = build_environment(&definition, &source).unwrap();
        assert_eq!(
            env.get("CLAUDE_CONFIG_DIR"),
            Some(&profile.as_os_str().to_owned())
        );

        fs::set_permissions(&profile, fs::Permissions::from_mode(0o755)).unwrap();
        let refusal = build_environment(&definition, &source).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("attendu 0700")
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn session_claude_native_est_preparable_comme_equipier_gere() {
        let root = root("claude-stream-json");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": "/bin/sh",
                        "args": ["--model", "claude-opus-5"],
                        "protocol": "claude_stream_json",
                        "forbidden_env": [],
                        "pass_env": ["SPECIAL_AUTH"],
                        "capabilities": {
                            "execution_paths": ["claude_stream_json"],
                            "models": {"claude-opus-5": {"efforts": []}}
                        }
                    }
                }
            })
            .to_string(),
            "/tmp/t905-claude-native.json",
        )
        .unwrap();
        let decision = submit_spawn(
            &supervisor,
            &registry,
            &source(&root),
            &order(
                &root,
                "claude-native",
                "89000000-0000-4000-8000-000000000201",
            ),
            NOW,
            false,
            &hosts_fixture(),
        )
        .unwrap();
        assert!(matches!(decision, SpawnDecision::Ready(_)));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capacite_absente_refuse_avant_reservation_et_sans_processus() {
        let root = root("capability-preflight");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry_with_capabilities(
            &["--model", "gpt-5.6-terra"],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {"gpt-5.5": {"efforts": ["low"]}}
            }),
        );
        let order = order(
            &root,
            "command-unsupported-model",
            "89000000-0000-4000-8000-000000000202",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &order,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(
            refusal,
            SpawnRefusal::UnsupportedCapability {
                agent_type: "fixture".to_string(),
                model: "gpt-5.6-terra".to_string(),
                capability: "modèle pris en charge par l'adaptateur".to_string(),
            }
        );
        assert!(!supervisor.knows_command(&order.command_id));
        assert_eq!(supervisor.active_count(), 0);
        // Mutation discriminante : déplacer la garde après request_spawn rend
        // cette commande connue et transformerait ce refus sans processus en
        // état durable résiduel.
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn observation_fournisseur_incoherente_refuse_avant_reservation() {
        let root = root("provider-observation-preflight");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry_with_capabilities(
            &[],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {},
                "observed": {
                    "binary_path": "/bin/sh",
                    "binary_version": "fixture-1",
                    "binary_digest": "0000000000000000000000000000000000000000000000000000000000000000",
                    "contract_version": "acp-v1",
                    "operations": []
                }
            }),
        );
        let order = order(
            &root,
            "command-provider-observation",
            "89000000-0000-4000-8000-000000000202",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &order,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert!(matches!(
            refusal,
            SpawnRefusal::UnsupportedCapability { ref capability, .. }
                if capability == "empreinte du binaire fournisseur observé"
        ));
        assert!(!supervisor.knows_command(&order.command_id));
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn effort_non_declare_est_refuse_et_reprise_revalidee_sur_definition_figee() {
        let root = root("capability-effort");
        fs::create_dir_all(&root).unwrap();
        let registry = registry_with_capabilities(
            &["model=gpt-5.6-terra", "effort=high"],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {"gpt-5.6-terra": {"efforts": ["low"]}}
            }),
        );
        let supervisor = supervisor(&root, 1);
        let request = order(
            &root,
            "command-unsupported-effort",
            "89000000-0000-4000-8000-000000000202",
        );
        assert!(matches!(
            rejection(
                submit_spawn(&supervisor, &registry, &source(&root), &request, NOW, false, &hosts_fixture())
                    .unwrap()
            ),
            SpawnRefusal::UnsupportedCapability { ref capability, .. }
                if capability == "effort 'high' accepté par le modèle"
        ));

        let resolved = registry.resolved_definition("fixture").unwrap();
        let candidate = RecoveryCandidate {
            lease: SpawnLease {
                command_id: "recovery-unsupported-effort".to_string(),
                name: "89000000-0000-4000-8000-000000000202".to_string(),
                instance_id: "instance-recovery".to_string(),
                generation: 1,
                deadline_at: NOW + 10,
                persistent: true,
                project: None,
                link_id: None,
                ownership: None,
                agent_path: None,
            },
            agent_type: "fixture".to_string(),
            cwd: root.clone(),
            resolved_definition: Some(resolved),
            runtime_execution: None,
        };
        assert!(matches!(
            prepare_recovery(&source(&root), candidate),
            Err(SpawnRefusal::UnsupportedCapability { ref capability, .. })
                if capability == "effort 'high' accepté par le modèle"
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel() {
        // Huit refus indépendants ; NameActive et QuotaExceeded nécessitent
        // une génération témoin active et sont exercés plus bas.
        for (label, expected) in [
            (
                "unknown",
                SpawnRefusal::UnknownType {
                    requested_type: "absent".to_string(),
                    known_types: Vec::new(),
                    registry: "/tmp/t905-agents.json".to_string(),
                },
            ),
            (
                "missing",
                SpawnRefusal::CommandMissing {
                    command: "/commande/introuvable".to_string(),
                    registry: "/tmp/t905-agents.json".to_string(),
                },
            ),
            (
                "billing",
                SpawnRefusal::BillingGuard {
                    variable: "API_KEY".to_string(),
                },
            ),
            (
                "env",
                SpawnRefusal::EnvUnfit {
                    detail: "HOME absent de l'environnement du daemon".to_string(),
                },
            ),
            (
                "cwd",
                SpawnRefusal::CwdGone {
                    searched_on: "poste-alpha".to_string(),
                    requested_from: "poste-beta".to_string(),
                },
            ),
            (
                "negotiation",
                SpawnRefusal::UnsupportedCapability {
                    agent_type: "fixture".to_string(),
                    model: "<non déclaré>".to_string(),
                    capability: "chemin d'exécution 'tmux'".to_string(),
                },
            ),
            ("timeout", SpawnRefusal::SpawnTimeout),
            ("expired", SpawnRefusal::IdempotencyExpired),
        ] {
            let root = root(label);
            fs::create_dir_all(&root).unwrap();
            let supervisor = supervisor(&root, 2);
            let mut request = order(
                &root,
                &format!("command-{label}"),
                "89000000-0000-4000-8000-000000000203",
            );
            let mut env = source(&root);
            let registry = match label {
                "unknown" => {
                    request.agent_type = "absent".to_string();
                    registry("/bin/sh", "acp", &[])
                }
                "missing" => registry("/commande/introuvable", "acp", &[]),
                "billing" => {
                    env.insert("API_KEY".to_string(), OsString::from("secret"));
                    registry("/bin/sh", "acp", &["API_KEY"])
                }
                "env" => {
                    env.remove("HOME");
                    registry("/bin/sh", "acp", &[])
                }
                "cwd" => {
                    request.cwd = root.join("disparu");
                    registry("/bin/sh", "acp", &[])
                }
                "negotiation" => registry("/bin/sh", "tmux", &[]),
                "timeout" => {
                    request.issued_at = NOW - 2;
                    request.deadline_at = NOW - 1;
                    registry("/bin/sh", "acp", &[])
                }
                "expired" => {
                    request.issued_at = NOW - 400;
                    request.deadline_at = NOW - 390;
                    registry("/bin/sh", "acp", &[])
                }
                _ => unreachable!(),
            };
            let mut expected = expected;
            if let SpawnRefusal::UnknownType { known_types, .. } = &mut expected {
                // La composition du catalogue appartient au registre. Cette
                // matrice vérifie que lifecycle relaie son instantané exact,
                // sans recopier une liste native qui deviendrait périmée.
                *known_types = registry.known_types();
            }
            let actual = rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &env,
                    &request,
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap(),
            );
            assert_eq!(actual, expected, "famille {label}");
            let replay = rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &env,
                    &request,
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap(),
            );
            assert_eq!(replay, expected, "rejeu divergent pour {label}");
            assert_eq!(supervisor.active_count(), 0, "résidu actif pour {label}");
            let _ = fs::remove_dir_all(root);
        }

        let nq_root = root("name-quota");
        fs::create_dir_all(&nq_root).unwrap();
        let nq_supervisor = supervisor(&nq_root, 1);
        let nq_registry = registry("/bin/sh", "acp", &[]);
        let env = source(&nq_root);
        assert!(matches!(
            submit_spawn(
                &nq_supervisor,
                &nq_registry,
                &env,
                &order(
                    &nq_root,
                    "command-first",
                    "89000000-0000-4000-8000-000000000203"
                ),
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(
                        &nq_root,
                        "command-name",
                        "89000000-0000-4000-8000-000000000203"
                    ),
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::NameActive
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(
                        &nq_root,
                        "command-quota",
                        "89000000-0000-4000-8000-000000000204"
                    ),
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::QuotaExceeded { limit: 1 }
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        let _ = fs::remove_dir_all(nq_root);

        let root = root("recovering");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry("/bin/sh", "acp", &[]);
        assert_eq!(
            rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &source(&root),
                    &order(
                        &root,
                        "command-recovering",
                        "89000000-0000-4000-8000-000000000205"
                    ),
                    NOW,
                    true,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::DaemonRecovering
        );
        assert_eq!(supervisor.active_count(), 0);
        assert!(!supervisor.knows_command("command-recovering"));

        let known = order(
            &root,
            "command-known-before-recovery",
            "89000000-0000-4000-8000-000000000206",
        );
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &known,
                NOW,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &known,
                NOW,
                true,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Await(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    /// POINT 5 — le refus doit dire OÙ le daemon a cherché et QUI a demandé.
    ///
    /// Contrôle positif d'abord : un `cwd` existant est accepté, donc l'absence
    /// de refus ci-dessous n'est pas le silence d'une garde morte.
    ///
    /// Mutant qui tue ce test : recopier `hosts.searched_on` dans les deux
    /// champs → l'assertion sur `requested_from == "poste-beta"` meurt. Un oracle
    /// qui vérifierait seulement que les champs sont non vides survivrait.
    #[test]
    fn le_refus_de_cwd_nomme_la_machine_cherchee_et_la_machine_demandeuse() {
        let root = root("cwd-attribution");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 4);
        let registry = registry("/bin/sh", "acp", &[]);
        let env = source(&root);

        // Contrôle positif : le répertoire existe, la garde laisse passer.
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &order(&root, "cwd-present", "89000000-0000-4000-8000-000000000207"),
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));

        // Répertoire absent : le refus porte les DEUX machines.
        let mut absent = order(&root, "cwd-absent", "89000000-0000-4000-8000-000000000208");
        absent.cwd = root.join("repertoire-qui-n-existe-pas");
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &absent,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        match refusal {
            SpawnRefusal::CwdGone {
                searched_on,
                requested_from,
            } => {
                assert_eq!(searched_on, "poste-alpha", "machine où l'on a cherché");
                assert_eq!(requested_from, "poste-beta", "machine qui a demandé");
            }
            other => panic!("refus attendu CwdGone, obtenu {other:?}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn core_089_projet_refuse_avant_reservation_et_reparable_au_meme_id() {
        let root = root("removed-project");
        let supervisor = supervisor(&root, 2);
        let registry = registry("/bin/sh", "acp", &[]);
        let mut spawn = order(
            &root,
            "core-089-project",
            "00000089-0000-4000-8000-000000000001",
        );
        spawn.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "historical-project".to_string(),
            binding_generation: 7,
        });
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(
            refusal,
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "historical-project".to_string(),
            }
        );
        // Mutation : une garde après request_spawn laisse ici une clé connue.
        assert!(!supervisor.knows_command(&spawn.command_id));
        assert_eq!(supervisor.active_count(), 0);
        assert!(supervisor.desired_fleet().unwrap().equipiers.is_empty());

        // Changement EXPLICITE du demandeur, pas une suppression automatique.
        spawn.project = None;
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn core_089_reprise_refuse_le_runtime_meme_sans_reference_projet() {
        use crate::desired_state::{ContainerAgentExecution, ContainerAgentExecutionState};
        let root = root("removed-recovery");
        let registry = registry("/bin/sh", "acp", &[]);
        let mut candidate = RecoveryCandidate {
            lease: SpawnLease {
                command_id: "historical-recovery".to_string(),
                name: "00000089-0000-4000-8000-000000000002".to_string(),
                instance_id: "historical-instance".to_string(),
                generation: 7,
                deadline_at: NOW + 10,
                persistent: true,
                project: None,
                link_id: None,
                ownership: None,
                agent_path: None,
            },
            agent_type: "fixture".to_string(),
            cwd: root.clone(),
            resolved_definition: Some(registry.resolved_definition("fixture").unwrap()),
            runtime_execution: Some(ContainerAgentExecution {
                agent_instance_id: "historical-instance".to_string(),
                generation: 7,
                project_id: "orphaned-project".to_string(),
                binding_generation: 4,
                environment_epoch: 1,
                container_id: "a".repeat(64),
                exec_id: uuid::Uuid::new_v4().to_string(),
                cwd: root.clone(),
                state: ContainerAgentExecutionState::Running,
                provider_identity: "fixture".to_string(),
            }),
        };
        // Le cwd n'existe même pas : la métadonnée interdit le lanceur hôte
        // avant toute tentative de récupération de l'environnement.
        assert_eq!(
            prepare_recovery(&source(&root), candidate.clone()).unwrap_err(),
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "orphaned-project".to_string()
            }
        );
        candidate.runtime_execution = None;
        candidate.lease.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "project-only".to_string(),
            binding_generation: 5,
        });
        assert_eq!(
            prepare_recovery(&source(&root), candidate).unwrap_err(),
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "project-only".to_string()
            }
        );
        assert!(!root.exists());
    }

    #[test]
    fn core_089_rejeu_terminal_historique_ne_relance_ni_ne_reecrit_le_resultat() {
        let root = root("historical-replay");
        let supervisor = supervisor(&root, 2);
        let registry = registry("/bin/sh", "acp", &[]);
        let mut spawn = order(
            &root,
            "historical-project-terminal",
            "00000089-0000-4000-8000-000000000003",
        );
        spawn.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "archived-project".to_string(),
            binding_generation: 3,
        });
        // Reconstitution d'une issue écrite par l'ancien produit, au niveau
        // du store/superviseur. Aucun moteur ni processus n'est lancé.
        let SpawnSubmission::Start(lease) = supervisor.request_spawn(&spawn, NOW).unwrap() else {
            panic!("réservation historique attendue");
        };
        let definition = registry.resolved_definition("fixture").unwrap();
        supervisor.mark_starting(&lease, NOW, &definition).unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW)
            .unwrap();
        drop(supervisor);
        let supervisor = super::tests::supervisor(&root, 2);
        let before = fs::read(root.join("fleet.json")).unwrap();
        assert!(
            matches!(submit_spawn(&supervisor, &registry, &source(&root),
            &spawn, NOW + 1, false, &hosts_fixture()).unwrap(),
            SpawnDecision::Accepted { definition: Some(ref saved), .. } if saved == &definition)
        );
        assert_eq!(supervisor.active_count(), 0);
        assert_eq!(fs::read(root.join("fleet.json")).unwrap(), before);
        // Même clé, canon changé : ne contourne ni le rejet ni l'idempotence.
        spawn.project = None;
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW + 1,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::EnvelopeMismatch
        ));
        assert_eq!(fs::read(root.join("fleet.json")).unwrap(), before);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }
}
