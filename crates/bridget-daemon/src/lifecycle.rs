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
    pub(crate) specialist_authority: Option<SpecialistLaunchAuthority>,
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
    source: &SourceEnvironment,
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
    let actual_digest =
        crate::provider_program_compat::program_digest(Path::new(&observed.binary_path))
            .map_err(|_| refusal("binaire fournisseur observé lisible"))?;
    if actual_digest != observed.binary_digest {
        if definition.protocol != "codex_app_server" {
            return Err(refusal("compatibilité du programme modifié non vérifiée"));
        }
        if let Err(code) = crate::provider_program_compat::validate_upgrade(
            &definition.command,
            source,
            &actual_digest,
        ) {
            crate::provider_program_compat::record_failure(code);
            return Err(refusal(code));
        }
    }
    Ok(())
}

/// Préflight157 commun sur la définition épinglée et l'environnement capturé.
/// O(t+C+B+P+E log e) : t types (liste en cas de refus), C capacités/arguments, B binaire observé,
/// P entrées PATH, E variables déclarées, e variables source. Aucun cwd ni spawn.
pub(crate) fn launch_preflight(
    registry: &AgentRegistry,
    agent_type: &str,
    source: &SourceEnvironment,
) -> Result<(), SpawnRefusal> {
    let definition = registry
        .get(agent_type)
        .map_err(|_| SpawnRefusal::UnknownType {
            requested_type: agent_type.to_string(),
            known_types: registry.known_types(),
            registry: registry.source().display().to_string(),
        })?;
    validate_launch_capabilities(agent_type, definition)?;
    validate_provider_observation(agent_type, definition, source)?;
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
    let env = build_environment(definition, source)?;
    if !command_exists(&definition.command, &env) {
        return Err(SpawnRefusal::CommandMissing {
            command: definition.command.clone(),
            registry: registry.source().display().to_string(),
        });
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
    if order.remote_placement.is_some() {
        return Ok(SpawnDecision::Rejected(SpawnRefusal::NegotiationFailed {
            detail: "remote_placement_requires_worker".into(),
        }));
    }
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
    // La définition est épinglée par le caller. Une commande connue garde son rejeu
    // avant les gardes mutables ; une commande neuve refuse tout défaut de lancement.
    if !supervisor.knows_command(&order.command_id)
        && let Err(reason) = launch_preflight(registry, &order.agent_type, source)
    {
        return Ok(SpawnDecision::Rejected(reason));
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
    if candidate.lease.remote_placement.is_some() {
        return Err(SpawnRefusal::NegotiationFailed {
            detail: "remote_placement_requires_worker".into(),
        });
    }
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

pub(crate) fn prepare_spawn_parts(
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    agent_type: &str,
    cwd: &Path,
    lease: SpawnLease,
    hosts: &SpawnHosts,
) -> Result<PreparedSpawn, SpawnRefusal> {
    reject_removed_project(lease.project.as_ref())?;
    launch_preflight(registry, agent_type, source)?;
    let definition = registry
        .get(agent_type)
        .map_err(|_| SpawnRefusal::UnknownType {
            requested_type: agent_type.to_string(),
            known_types: registry.known_types(),
            registry: registry.source().display().to_string(),
        })?;
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
    Ok(PreparedSpawn {
        specialist_authority: None,
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

/// Sélection logique dans les seules racines du parent déjà attestées.
/// Aucune I/O : le maître ne doit pas ouvrir un chemin de la machine source.
/// Les appelants physiques vérifient canonicalisation, existence et protections.
pub(crate) fn source_root(
    parent: &bridget_transport::protocol::ProviderPermissions,
    cwd: &Path,
) -> Result<PathBuf, String> {
    let valid = |path: &Path| {
        path.is_absolute()
            && !path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
    };
    if !valid(cwd) {
        return Err("cwd_outside_parent_project".into());
    }
    let root = Path::new(&parent.cwd);
    if valid(root) && cwd.starts_with(root) {
        return Ok(root.to_path_buf());
    }
    let additional = [
        parent.provider_policy.get("additional_directories"),
        parent
            .provider_policy
            .pointer("/sandbox_policy/writableRoots"),
    ];
    additional
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_array())
        .flatten()
        .filter_map(|v| v.as_str())
        .map(Path::new)
        .filter(|root| valid(root) && cwd.starts_with(root))
        .max_by_key(|root| root.components().count())
        .map(Path::to_path_buf)
        .or_else(|| {
            crate::native_permissions::unrestricted_source_scope(parent).then(|| cwd.to_path_buf())
        })
        .ok_or_else(|| "cwd_outside_parent_project".into())
}

/// Une vue exporte la racine de A, pas un sous-arbre hérité d'un cwd.
/// La racine Git doit donc appartenir au périmètre du parent attesté.
/// Git est lu sur la machine source et hors mutex du maître.
pub(crate) fn validate_parent_source_base(
    parent: &bridget_transport::protocol::ProviderPermissions,
    request: &bridget_transport::protocol::WorkerSourceBase,
) -> Result<(), String> {
    let selected = source_root(parent, Path::new(&request.cwd))?;
    if selected != Path::new(&parent.cwd)
        && !crate::native_permissions::unrestricted_source_scope(parent)
        && std::fs::canonicalize(&selected).ok().as_deref() != Some(selected.as_path())
    {
        return Err("cwd_outside_parent_project".into());
    }
    if matches!(
        request.workspace,
        Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
    ) {
        let allowed = std::fs::canonicalize(source_root(parent, Path::new(&request.cwd))?)
            .map_err(|_| "workspace_preflight_technical")?;
        let cwd =
            std::fs::canonicalize(&request.cwd).map_err(|_| "workspace_preflight_technical")?;
        if !cwd.starts_with(&allowed) {
            return Err("cwd_outside_parent_project".into());
        }
        let out = crate::workspace_git::run_git(
            &cwd,
            &["rev-parse", "--show-toplevel"],
            crate::workspace_git::READ_TIMEOUT,
        )?;
        if !out.success() || out.truncated {
            return Err("workspace_git_required".into());
        }
        let top = std::fs::canonicalize(out.stdout.trim_end_matches(['\r', '\n']))
            .map_err(|_| "workspace_preflight_technical")?;
        if !top.starts_with(&allowed) {
            return Err("cwd_outside_parent_project".into());
        }
    }
    validate_source_base(request)
}

pub(crate) fn validate_source_base(
    request: &bridget_transport::protocol::WorkerSourceBase,
) -> Result<(), String> {
    if matches!(
        request.workspace,
        Some(bridget_transport::protocol::WorkspaceRequest::CommitView {})
    ) {
        // Fait de source seulement : les portes d'admission, de client et de
        // création worker restent indépendantes. Lire A ne qualifie pas HEAD B.
        let expected = request
            .expected_base_commit
            .as_ref()
            .ok_or("expected_base_commit_required")?;
        let actual =
            crate::workspace_git::resolve_source_commit(Path::new(&request.cwd), Some(expected))
                .map_err(|error| error.into_admission(true))?;
        if &actual != expected {
            return Err("expected_base_commit_mismatch".into());
        }
        return Ok(());
    }
    use bridget_transport::protocol::AutoDelegateWorkload;
    // La branche demandée est la source d'un worktree. Le checkout du parent
    // reste intact et peut être sur une autre branche. Les contraintes de
    // création sont vérifiées sur la machine qui crée réellement le worktree.
    if let Some(bridget_transport::protocol::WorkspaceRequest::Worktree { base_ref, branch }) =
        &request.workspace
    {
        let expected = request
            .expected_base_commit
            .as_ref()
            .ok_or("expected_base_commit_required")?;
        if let Some(branch) = branch {
            crate::workspace_git::validate_branch(branch)?;
        }
        let actual = crate::workspace_git::resolve_source_commit(
            Path::new(&request.cwd),
            base_ref.as_deref(),
        )
        .map_err(|error| error.into_admission(true))?;
        if &actual != expected {
            return Err("expected_base_commit_mismatch".into());
        }
        return Ok(());
    }
    let actual = crate::lifecycle::source_base_commit(Path::new(&request.cwd))?;
    match (&actual, &request.expected_base_commit) {
        (Some(_), None) => return Err("expected_base_commit_required".into()),
        (None, Some(_)) => return Err("expected_base_commit_outside_git".into()),
        (None, None)
            if request.workload == AutoDelegateWorkload::Compilation
                || matches!(
                    request.workspace,
                    Some(bridget_transport::protocol::WorkspaceRequest::Worktree { .. })
                ) =>
        {
            return Err("workspace_git_required".into());
        }
        (Some(actual), Some(expected)) if actual != expected => {
            return Err("expected_base_commit_mismatch".into());
        }
        _ => {}
    }

    Ok(())
}

/// Preuve168 de la source Git. O(d+B), d ancêtres du cwd, B sortie Git bornée.
/// Une absence Git exige des métadonnées absentes sur tous les ancêtres.
/// Une erreur E/S, un dépôt nu ou une sortie Git invalide reste technique.
pub(crate) fn source_base_commit(cwd: &Path) -> Result<Option<String>, String> {
    let cwd = std::fs::canonicalize(cwd)
        .map_err(|error| format!("workspace_base_preflight_failed: {error}"))?;
    if !cwd.is_dir() {
        return Err("workspace_base_preflight_failed".into());
    }
    let mut outside = true;
    for ancestor in cwd.ancestors() {
        let mut present = [false; 3];
        for (index, metadata) in [
            ancestor.join(".git"),
            ancestor.join("HEAD"),
            ancestor.join("objects"),
        ]
        .into_iter()
        .enumerate()
        {
            match std::fs::symlink_metadata(&metadata) {
                Ok(_) => present[index] = true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("workspace_base_preflight_failed: {error}")),
            }
        }
        outside &= !(present[0] || present[1] && present[2]);
    }
    if outside {
        return Ok(None);
    }
    let root = crate::workspace_git::run_git(
        &cwd,
        &["rev-parse", "--show-toplevel"],
        crate::workspace_git::READ_TIMEOUT,
    )
    .map_err(|error| format!("workspace_base_preflight_failed: {error}"))?;
    if root.truncated {
        return Err("workspace_base_preflight_failed".into());
    }
    if !root.success() {
        return Err("workspace_base_preflight_failed".into());
    }
    let head = crate::workspace_git::run_git(
        &cwd,
        &["rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"],
        crate::workspace_git::READ_TIMEOUT,
    )
    .map_err(|error| format!("workspace_base_preflight_failed: {error}"))?;
    let commit = head.stdout.trim();
    if !head.success()
        || head.truncated
        || commit.len() != 40
        || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("workspace_base_unavailable".into());
    }
    Ok(Some(commit.to_ascii_lowercase()))
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
        // Les credentials d'un montage MCP appartiennent à une session.
        // Même pass_env explicite ne transmet jamais l'identité du parent.
        if matches!(
            name.as_str(),
            "BRIDGET_T3_MCP_ENDPOINT" | "BRIDGET_T3_MCP_AUTHORIZATION"
        ) {
            continue;
        }
        if let Some(value) = source.get(name) {
            env.insert(name.clone(), value.clone());
        }
    }
    if let Some(profile) = definition.claude_config_dir.as_deref() {
        validate_provider_profile_directory(profile, "Claude")?;
        env.insert("CLAUDE_CONFIG_DIR".to_string(), OsString::from(profile));
        if definition.capabilities.account_profiles {
            let anthropic = Path::new(profile).join("anthropic");
            validate_provider_profile_directory(&anthropic.to_string_lossy(), "Anthropic")?;
            env.insert("ANTHROPIC_CONFIG_DIR".into(), anthropic.into_os_string());
        }
    }
    if let Some(profile) = definition.codex_home.as_deref() {
        validate_provider_profile_directory(profile, "Codex")?;
        env.insert("CODEX_HOME".to_string(), OsString::from(profile));
    }
    Ok(env)
}

pub(crate) fn validate_provider_profile_directory(
    profile: &str,
    provider: &str,
) -> Result<(), SpawnRefusal> {
    let path = Path::new(profile);
    let metadata = fs::symlink_metadata(path).map_err(|err| SpawnRefusal::EnvUnfit {
        detail: format!("profil {provider} indisponible {}: {err}", path.display()),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil {provider} invalide {}: répertoire réel requis",
                path.display()
            ),
        });
    }
    if metadata.uid() != bridget_transport::fsutil::effective_uid() {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil {provider} invalide {}: propriétaire inattendu",
                path.display()
            ),
        });
    }
    let mode = metadata.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        return Err(SpawnRefusal::EnvUnfit {
            detail: format!(
                "profil {provider} invalide {}: permissions {mode:04o}, attendu 0700 ou plus restrictif",
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

pub(crate) fn refusal_record(reason: &SpawnRefusal) -> (&'static str, String) {
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
#[path = "lifecycle_tests.rs"]
mod tests;

#[derive(Debug, Clone)]
pub(crate) struct SpecialistLaunchAuthority {
    pub db_path: PathBuf,
    pub agent_id: String,
    pub grant_id: String,
    pub grant_revision: u64,
    pub profile_revision: u64,
    pub profile_id: String,
    pub project_id: String,
    pub submission_id: String,
    pub human_approval_ref: Option<String>,
}
pub(crate) struct SpecialistReleaseGuard {
    connection: rusqlite::Connection,
}
impl SpecialistLaunchAuthority {
    /// Le verrou SQL est commun aux mutations humaines et au dernier RELEASE.
    /// Il est acquis après BootstrapReady ; aucun démarrage fournisseur sous une autorité périmée.
    pub(crate) fn lock(
        &self,
    ) -> Result<SpecialistReleaseGuard, crate::managed_process::ManagedProcessError> {
        use sha2::{Digest, Sha256};
        let invalid =
            |code: &str| crate::managed_process::ManagedProcessError::InvalidStatus(code.into());
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|_| invalid("specialist_authority_unavailable"))?;
        conn.busy_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| invalid("specialist_authority_unavailable"))?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| invalid("specialist_authority_unavailable"))?;
        let body: String = conn
            .query_row(
                "SELECT record_json FROM specialists WHERE agent_id=?1",
                [&self.agent_id],
                |r| r.get(0),
            )
            .map_err(|_| invalid("mandate_required"))?;
        let record: crate::agent_profile::specialists::SpecialistRecord =
            serde_json::from_str(&body).map_err(|_| invalid("specialist_authority_unavailable"))?;
        let grant = record
            .grant
            .as_ref()
            .ok_or_else(|| invalid("mandate_required"))?;
        let (canonical,role_official):(Vec<u8>,Option<bool>)=conn.query_row(
            "SELECT canonical_bytes,json_extract(profile_snapshot,'$.role_official') FROM specialist_requests WHERE native_submission_id=?1 AND selected_agent_id=?2",
            rusqlite::params![self.submission_id,self.agent_id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|_|invalid("specialist_authority_unavailable"))?;
        let canonical: serde_json::Value = serde_json::from_slice(&canonical)
            .map_err(|_| invalid("specialist_authority_unavailable"))?;
        let role = canonical["role_id"]
            .as_str()
            .ok_or_else(|| invalid("role_changed"))?;
        let (scope, official): (String, bool) = conn
            .query_row(
                "SELECT scope_json,official FROM specialist_roles WHERE agent_id=?1 AND role_id=?2",
                rusqlite::params![self.agent_id, role],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .map_err(|_| invalid("role_changed"))?;
        let scope: bridget_transport::specialists::SpecialistScope =
            serde_json::from_str(&scope).map_err(|_| invalid("role_changed"))?;
        if !scope.permits(&self.project_id) || (role_official == Some(true) && !official) {
            return Err(invalid("role_changed"));
        }

        let human_request = if let Some(expected) = &self.human_approval_ref {
            let (snapshot,canonical,selected,grant_revision):(String,Vec<u8>,String,u64)=conn.query_row(
                "SELECT profile_snapshot,canonical_bytes,selected_agent_id,grant_revision FROM specialist_requests WHERE native_submission_id=?1",
                [&self.submission_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).map_err(|_|invalid("specialist_authority_unavailable"))?;
            let snapshot: serde_json::Value = serde_json::from_str(&snapshot)
                .map_err(|_| invalid("specialist_authority_unavailable"))?;
            let hash = format!("{:x}", Sha256::digest(canonical));
            let parts = expected
                .rsplit_once(':')
                .ok_or_else(|| invalid("human_principal_required"))?;
            let native_question = parts
                .0
                .strip_prefix("user-input:")
                .and_then(|id| crate::user_input::get(&conn, id).ok().flatten());
            let question_approved = native_question.as_ref().is_some_and(|q| {
                matches!(q["state"].as_str(), Some("submitted" | "applied"))
                    && q["callback"]["kind"] == "specialist_approval"
                    && q["callback"]["digest"] == hash
                    && q["answers"]["specialist_approval"]["selected"]
                        == serde_json::json!(["approve"])
                    && q["expires_at"].as_i64().is_some_and(|deadline| {
                        deadline
                            > std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64
                    })
            });
            let known = crate::referent_control::human_principal_actor(parts.0).is_some()
                || parts
                    .0
                    .strip_prefix("mobile:")
                    .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
                || question_approved;
            known
                && parts.1 == hash
                && snapshot["human_approval_ref"] == expected.as_str()
                && selected == self.agent_id
                && grant_revision == self.grant_revision
        } else {
            false
        };
        if !record.enabled
            || (!record.auto_start && !human_request)
            || (!record.durable && !human_request)
            || record.profile_revision != self.profile_revision
            || record.profile_id.as_deref() != Some(self.profile_id.as_str())
            || grant.profile_id != record.profile_id
            || grant.profile_revision != record.profile_revision
            || grant.grant_id != self.grant_id
            || grant.revision != self.grant_revision
            || grant.revoked_at.is_some()
            || !grant.allowed_projects.permits(&self.project_id)
        {
            return Err(invalid("mandate_changed"));
        }
        let control = crate::referent_control::read(&conn)
            .map_err(|_| invalid("control_state_unavailable"))?;
        if control.paused {
            return Err(invalid("referent_paused"));
        }
        Ok(SpecialistReleaseGuard { connection: conn })
    }
}
impl SpecialistReleaseGuard {
    pub(crate) fn finish(self) -> Result<(), crate::managed_process::ManagedProcessError> {
        self.connection.execute_batch("COMMIT").map_err(|_| {
            crate::managed_process::ManagedProcessError::InvalidStatus(
                "specialist_authority_unavailable".into(),
            )
        })
    }
}

/// Nouvelle mission de spécialiste sous mandat courant : la relance historique
/// conserve son ancienne définition, cette frontière emploie la cible qualifiée courante.
pub(crate) fn submit_specialist_relaunch(
    supervisor: &FleetSupervisor,
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    hosts: &SpawnHosts,
) -> Result<SpawnDecision, FleetError> {
    if order.persistent
        || order
            .requested_name
            .as_ref()
            .is_none_or(|id| uuid::Uuid::parse_str(id).is_err())
        || !order.command_id.starts_with("specialist-")
        || order.posture.is_none()
    {
        return Err(FleetError::InvalidOrder(
            "specialist_launch_authority_required",
        ));
    }
    submit_spawn_with_policy(
        supervisor, registry, source, order, now, false, hosts, true, false,
    )
}
