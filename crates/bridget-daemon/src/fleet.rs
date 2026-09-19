//! Orchestration transactionnelle des équipiers gérés par le daemon.
//!
//! La machine d'états vit derrière un verrou propre au superviseur. Le socle
//! 012 reste l'unique autorité pour réserver, comparer et rejouer une clé ;
//! `fleet.rs` ne fait aucun lookup idempotent parallèle.

use crate::desired_state::{
    ContainerAgentExecution, DesiredAgentLink, DesiredEquipier, DesiredFleet,
    DesiredLifecycleState, DesiredStateError, DesiredStateStore,
};
pub use crate::idempotency::{
    AgentLinkEvent, AgentLinkRecord as AgentLink, AgentLinkState, DelegatedRuntimeEventInput,
    DelegatedRuntimeEventRecord,
};
use crate::idempotency::{
    HistoricalManagedSpawn, IdempotencyError, IdempotencyKey, IdempotencyStore, OperationKind,
    SpawnCommand, SpawnCommandIssue, SpawnCommandState, SpawnReservation,
};

/// Vue de propriété calculée depuis le lien durable et ses index. Elle ne
/// porte aucun coût et n'induit aucune transition de mission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLinkSummary {
    pub link: AgentLink,
    pub direct_descendants: u64,
    pub descendants: u64,
}

/// Résultat borné de l'attente d'un parent, rejouable depuis le dernier curseur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentLinkEventBatch {
    pub events: Vec<AgentLinkEvent>,
    pub through_cursor: Option<u64>,
}

use crate::execution_store::ExecutionBudgetFacts;
use crate::recovery_trace::{
    NamedRosterEntry, NamedRosterStore, RecoveryLossEntry, persist_report, report_path,
    resolved_domain, roster_path,
};
use bridget_core::router::validate_agent_id;
pub use bridget_transport::protocol::{ProjectReference, SpawnOwnership};
use bridget_transport::{ResolvedAgentDefinition, protocol::ExecutionBudgetOutcome};
use log::warn;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Variable d'environnement qui fixe le quota de flotte gérée au démarrage.
pub const FLEET_QUOTA_ENV: &str = "BRIDGET_FLEET_QUOTA";

/// Défaut relevé : la flotte légitime tourne autour de 10–12 équipiers.
pub const DEFAULT_FLEET_QUOTA: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct FleetConfig {
    pub quota: usize,
    pub persistent_horizon_secs: i64,
    pub ephemeral_horizon_secs: i64,
    pub issued_at_tolerance_secs: i64,
}

impl Default for FleetConfig {
    fn default() -> Self {
        Self {
            quota: DEFAULT_FLEET_QUOTA,
            persistent_horizon_secs: 7 * 24 * 60 * 60,
            ephemeral_horizon_secs: 24 * 60 * 60,
            issued_at_tolerance_secs: 30,
        }
    }
}

/// Bornes explicites de continuation. Elles sont définies près des quotas de
/// flotte existants afin de conserver une seule politique technique Bridget.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AutonomyBudgetPolicy {
    pub max_duration_secs: Option<u64>,
    pub max_facturable_tokens: Option<u64>,
    pub max_descendants: Option<u64>,
}

/// Précondition observée avant d évaluer une continuation. Une pause ou une
/// terminaison ne se confond jamais avec une limite de consommation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutonomyRuntimeState {
    Ready,
    Paused,
    Terminated,
}

/// Évalue les bornes dans un ordre fixe et documenté. L absence d usage
/// attesté ne devient jamais une limite d usage par défaut.
pub fn evaluate_autonomy_budget(
    policy: AutonomyBudgetPolicy,
    facts: &ExecutionBudgetFacts,
    runtime: AutonomyRuntimeState,
) -> Option<ExecutionBudgetOutcome> {
    match runtime {
        AutonomyRuntimeState::Paused => return Some(ExecutionBudgetOutcome::Paused),
        AutonomyRuntimeState::Terminated => return Some(ExecutionBudgetOutcome::Terminated),
        AutonomyRuntimeState::Ready => {}
    }
    if policy
        .max_descendants
        .is_some_and(|limit| facts.descendants >= limit)
    {
        return Some(ExecutionBudgetOutcome::Blocked);
    }
    if policy
        .max_duration_secs
        .is_some_and(|limit| facts.duration_secs >= limit)
    {
        return Some(ExecutionBudgetOutcome::BudgetLimit);
    }
    if policy.max_facturable_tokens.is_some_and(|limit| {
        facts
            .usage
            .as_ref()
            .is_some_and(|usage| usage.facturable_tokens >= limit)
    }) {
        return Some(ExecutionBudgetOutcome::UsageLimit);
    }
    None
}

impl FleetConfig {
    /// Lit `BRIDGET_FLEET_QUOTA` une fois au démarrage du daemon.
    /// Valeur invalide → défaut + warning. `0` conserve sa sémantique
    /// actuelle (`FleetSupervisor::open` refuse la configuration).
    pub fn from_env() -> Self {
        #[cfg(test)]
        {
            if let Some(quota) = test_fleet_quota_override() {
                return Self {
                    quota,
                    ..Self::default()
                };
            }
            Self::default()
        }
        #[cfg(not(test))]
        Self {
            quota: resolve_fleet_quota(std::env::var_os(FLEET_QUOTA_ENV)),
            ..Self::default()
        }
    }
}

#[cfg(test)]
thread_local! {
    static TEST_FLEET_QUOTA: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn test_fleet_quota_override() -> Option<usize> {
    TEST_FLEET_QUOTA.with(|cell| cell.get())
}

/// Surcharge le quota lu par `FleetConfig::from_env` dans le fil courant.
/// Hors override, les tests restent hermétiques (défaut 16, pas l'env shell).
#[cfg(test)]
pub struct TestFleetQuotaGuard {
    previous: Option<usize>,
}

#[cfg(test)]
impl TestFleetQuotaGuard {
    pub fn set(quota: usize) -> Self {
        let previous = TEST_FLEET_QUOTA.with(|cell| {
            let previous = cell.get();
            cell.set(Some(quota));
            previous
        });
        Self { previous }
    }
}

#[cfg(test)]
impl Drop for TestFleetQuotaGuard {
    fn drop(&mut self) {
        TEST_FLEET_QUOTA.with(|cell| cell.set(self.previous));
    }
}

/// Résout le quota depuis une valeur d'environnement brute.
pub fn resolve_fleet_quota(raw: Option<OsString>) -> usize {
    let Some(raw) = raw else {
        return DEFAULT_FLEET_QUOTA;
    };
    let Some(text) = raw.to_str() else {
        warn!("{FLEET_QUOTA_ENV} non UTF-8 — défaut {DEFAULT_FLEET_QUOTA} appliqué");
        return DEFAULT_FLEET_QUOTA;
    };
    let trimmed = text.trim();
    if trimmed.is_empty() {
        warn!("{FLEET_QUOTA_ENV} vide — défaut {DEFAULT_FLEET_QUOTA} appliqué");
        return DEFAULT_FLEET_QUOTA;
    }
    match trimmed.parse::<usize>() {
        Ok(0) => 0,
        Ok(quota) => quota,
        Err(_) => {
            warn!("{FLEET_QUOTA_ENV}={trimmed:?} invalide — défaut {DEFAULT_FLEET_QUOTA} appliqué");
            DEFAULT_FLEET_QUOTA
        }
    }
}

/// Détail durable du refus `quota_exceeded` (idempotence / replay).
pub fn quota_exceeded_detail(quota: usize) -> String {
    format!(
        "quota de flotte atteint ({quota}) ; lever avec {FLEET_QUOTA_ENV} (ex. export {FLEET_QUOTA_ENV}={})",
        suggested_fleet_quota(quota)
    )
}

/// Message WARN visible quand une reprise est amputée pour quota.
/// La trace durable complète reste au périmètre D20 — ne pas la dupliquer ici.
pub fn resume_quota_refusal_message(agent: &str, quota: usize) -> String {
    format!(
        "reprise de {agent} refusée: quota de flotte atteint ({quota}) ; lever avec {FLEET_QUOTA_ENV} (ex. export {FLEET_QUOTA_ENV}={})",
        suggested_fleet_quota(quota)
    )
}

fn suggested_fleet_quota(quota: usize) -> usize {
    quota.saturating_add(8).max(DEFAULT_FLEET_QUOTA)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnOrder {
    pub posture: Option<bridget_transport::protocol::SpawnPosture>,
    pub agent_type: String,
    pub project: Option<ProjectReference>,
    pub requested_name: Option<String>,
    pub cwd: PathBuf,
    pub persistent: bool,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
    pub ownership: Option<SpawnOwnership>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnLease {
    pub command_id: String,
    pub name: String,
    pub instance_id: String,
    pub generation: u64,
    pub deadline_at: i64,
    pub persistent: bool,
    pub project: Option<ProjectReference>,
    pub link_id: Option<String>,
    pub ownership: Option<SpawnOwnership>,
    pub agent_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnWaiter {
    pub command_id: String,
    pub instance_id: String,
    pub generation: u64,
    pub deadline_at: i64,
}

/// Génération persistante restée en vol lors du crash précédent. T908 la
/// prépare à nouveau sans réserver une seconde clé ni une seconde génération.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCandidate {
    pub lease: SpawnLease,
    pub agent_type: String,
    pub cwd: PathBuf,
    /// Définition persistée au passage Reserved→Starting. Une ancienne ligne
    /// sans définition ne peut pas être reprise sans trahir la preuve publique.
    pub resolved_definition: Option<ResolvedAgentDefinition>,
    /// Métadonnée historique conservée pour refuser une reprise conteneur.
    /// Elle ne constitue jamais une autorisation d'exécution sur l'hôte.
    pub runtime_execution: Option<ContainerAgentExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum SpawnSubmission {
    Start(SpawnLease),
    Await(SpawnWaiter),
    Terminal(SpawnCommandIssue),
    EnvelopeMismatch,
    IdempotencyExpired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdoptStoppedResult {
    Adopted { generation: u64 },
    AlreadyManaged,
    NoManagedHistory,
}

#[derive(Debug)]
pub enum FleetError {
    InvalidOrder(&'static str),
    InvalidTransition {
        command_id: String,
        from: SpawnCommandState,
        expected: SpawnCommandState,
    },
    StaleGeneration,
    DeadlineElapsed,
    Ownership(&'static str),
    Idempotency(IdempotencyError),
    DesiredState(DesiredStateError),
    Observation(io::Error),
}

impl fmt::Display for FleetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOrder(reason) => write!(formatter, "ordre de spawn invalide: {reason}"),
            Self::InvalidTransition {
                command_id,
                from,
                expected,
            } => write!(
                formatter,
                "transition spawn interdite pour {command_id}: {from:?}, attendu {expected:?}"
            ),
            Self::StaleGeneration => write!(formatter, "génération de spawn obsolète"),
            Self::DeadlineElapsed => write!(formatter, "délai absolu du spawn dépassé"),
            Self::Ownership(reason) => write!(formatter, "propriété agent invalide: {reason}"),
            Self::Idempotency(source) => write!(formatter, "socle idempotent: {source}"),
            Self::DesiredState(source) => write!(formatter, "état désiré: {source}"),
            Self::Observation(source) => write!(formatter, "observation de frontière: {source}"),
        }
    }
}

impl std::error::Error for FleetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Idempotency(source) => Some(source),
            Self::DesiredState(source) => Some(source),
            Self::Observation(source) => Some(source),
            Self::InvalidOrder(_)
            | Self::InvalidTransition { .. }
            | Self::StaleGeneration
            | Self::DeadlineElapsed
            | Self::Ownership(_) => None,
        }
    }
}

impl From<IdempotencyError> for FleetError {
    fn from(source: IdempotencyError) -> Self {
        Self::Idempotency(source)
    }
}

impl From<DesiredStateError> for FleetError {
    fn from(source: DesiredStateError) -> Self {
        Self::DesiredState(source)
    }
}

#[derive(Debug, Clone)]
struct ActiveSpawn {
    command_id: String,
    name: String,
    instance_id: String,
    generation: u64,
    deadline_at: i64,
    persistent: bool,
    project: Option<ProjectReference>,
    agent_type: String,
    cwd: PathBuf,
    state: SpawnCommandState,
    resolved_definition: Option<ResolvedAgentDefinition>,
    runtime_execution: Option<ContainerAgentExecution>,
    link_id: Option<String>,
    ownership: Option<SpawnOwnership>,
    agent_path: Option<String>,
}
struct FleetInner {
    idempotency: IdempotencyStore,
    supervisor_scope: String,
    active_by_name: HashMap<String, String>,
    active_by_command: HashMap<String, ActiveSpawn>,
    completed: HashMap<String, SpawnCommandIssue>,
    next_generation: u64,
}

pub struct FleetSupervisor {
    inner: Mutex<FleetInner>,
    terminal_changed: Condvar,
    desired: DesiredStateStore,
    agent_link_changed: Condvar,
    roster: NamedRosterStore,
    config: FleetConfig,
}

impl FleetSupervisor {
    pub fn open(
        database_path: &Path,
        desired: DesiredStateStore,
        config: FleetConfig,
    ) -> Result<Self, FleetError> {
        if config.quota == 0
            || config.persistent_horizon_secs <= 0
            || config.ephemeral_horizon_secs <= 0
        {
            return Err(FleetError::InvalidOrder("configuration de flotte invalide"));
        }
        let mut idempotency = IdempotencyStore::open(database_path)?;
        let supervisor_scope = idempotency.supervisor_scope()?;
        desired.stop_non_persistent_running()?;
        let desired_fleet = desired.load_at_startup()?;
        let mut inner = FleetInner {
            idempotency,
            supervisor_scope,
            active_by_name: HashMap::new(),
            active_by_command: HashMap::new(),
            completed: HashMap::new(),
            next_generation: 1,
        };
        recover_commands(&mut inner, &desired_fleet)?;
        let roster = NamedRosterStore::at_path(roster_path(desired.path()));
        Ok(Self {
            agent_link_changed: Condvar::new(),
            inner: Mutex::new(inner),
            terminal_changed: Condvar::new(),
            desired,
            roster,
            config,
        })
    }

    /// Retourne la définition figée d'un processus géré sans relire le
    /// registre courant. Elle reste disponible après `Connected`, dans l'issue
    /// terminale de la saga de spawn.
    pub(crate) fn resolved_definition_for_command(
        &self,
        command_id: &str,
    ) -> Option<ResolvedAgentDefinition> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .active_by_command
            .get(command_id)
            .and_then(|active| active.resolved_definition.clone())
            .or_else(|| match inner.completed.get(command_id) {
                Some(SpawnCommandIssue::Connected { definition, .. }) => {
                    definition.as_deref().cloned()
                }
                _ => None,
            })
    }

    pub fn supervisor_scope(&self) -> String {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .supervisor_scope
            .clone()
    }

    /// Retourne uniquement le binding de projet de l'agent actuellement
    /// supervisé. Cette lecture mémoire est l'attestation qui lie une
    /// publication à son projet : aucun wrapper ne peut l'indiquer lui-même.
    pub(crate) fn project_for_agent(&self, agent_name: &str) -> Option<ProjectReference> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let command_id = inner.active_by_name.get(agent_name)?;
        inner.active_by_command.get(command_id)?.project.clone()
    }

    /// Consultation mémoire sans second lookup SQLite. Utilisée pendant la
    /// phase Recovering : une clé connue doit conserver son chemin de rejeu,
    /// tandis qu'une clé neuve est refusée sans créer d'état opérationnel.
    pub fn knows_command(&self, command_id: &str) -> bool {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner.active_by_command.contains_key(command_id) || inner.completed.contains_key(command_id)
    }

    pub fn quota(&self) -> usize {
        self.config.quota
    }

    pub fn active_count(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .active_by_command
            .len()
    }

    /// Instantané ordonné des générations en vol rattachées à `fleet.json`.
    /// Les entrées déjà terminales ne figurent pas ici : T908 leur réserve une
    /// nouvelle génération de reprise.
    pub fn recovery_candidates(&self) -> Vec<RecoveryCandidate> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut candidates = inner
            .active_by_command
            .values()
            .filter(|active| active.persistent)
            .map(|active| RecoveryCandidate {
                lease: lease_from(active),
                agent_type: active.agent_type.clone(),
                cwd: active.cwd.clone(),
                resolved_definition: active.resolved_definition.clone(),
                runtime_execution: active.runtime_execution.clone(),
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.lease.name.cmp(&right.lease.name));
        candidates
    }

    pub fn desired_fleet(&self) -> Result<DesiredFleet, FleetError> {
        self.desired.load().map_err(Into::into)
    }

    pub fn desired_entry(&self, name: &str) -> Result<Option<DesiredEquipier>, FleetError> {
        Ok(self.desired.load()?.equipiers.remove(name))
    }

    pub fn mark_stopped(&self, name: &str) -> Result<bool, FleetError> {
        Ok(self
            .desired
            .set_lifecycle_state(name, DesiredLifecycleState::Stopped)?
            .is_some())
    }

    pub fn decommission(&self, name: &str) -> Result<bool, FleetError> {
        let changed = self
            .desired
            .set_lifecycle_state(name, DesiredLifecycleState::Decommissioned)?
            .is_some();
        if changed {
            self.roster.forget(name);
        }
        Ok(changed)
    }

    /// Importe explicitement dans le registre v4 un ancien agent arrêté.
    /// Le nom seul ne suffit jamais : une saga connectée et une définition
    /// runtime complète doivent être présentes dans le store idempotent.
    pub fn adopt_stopped(&self, name: &str, now: i64) -> Result<AdoptStoppedResult, FleetError> {
        if self.desired.load()?.equipiers.contains_key(name) {
            return Ok(AdoptStoppedResult::AlreadyManaged);
        }
        let historical: Option<HistoricalManagedSpawn> = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .idempotency
            .latest_connected_spawn_by_name(name)?;
        let Some(historical) = historical else {
            return Ok(AdoptStoppedResult::NoManagedHistory);
        };
        self.desired.upsert(
            name.to_string(),
            DesiredEquipier {
                agent_type: historical.agent_type.clone(),
                cwd: historical.cwd.clone(),
                command_id: historical.command_id,
                generation: historical.generation,
                created: now.to_string(),
                persistent: historical.persistent,
                lifecycle_state: DesiredLifecycleState::Stopped,
                resolved_definition: Some(historical.resolved_definition),
                domain: resolved_domain(None, &historical.cwd),
                project: historical.project,
                agent_link: None,
                runtime_execution: None,
            },
        )?;
        self.roster.remember(
            name.to_string(),
            NamedRosterEntry {
                agent_type: historical.agent_type,
                persistent: historical.persistent,
                domain: resolved_domain(None, &historical.cwd),
            },
        );
        Ok(AdoptStoppedResult::Adopted {
            generation: historical.generation,
        })
    }

    pub fn remove_desired(&self, name: &str) -> Result<(), FleetError> {
        self.desired.remove(name)?;
        self.roster.forget(name);
        Ok(())
    }

    pub fn drain_legacy_non_persistent_named(
        &self,
    ) -> Result<Vec<(String, NamedRosterEntry)>, FleetError> {
        let desired = self.desired.load()?;
        let mut legacy = Vec::new();
        for (name, entry) in self.roster.drain_non_persistent() {
            if desired.equipiers.contains_key(&name) {
                self.roster.remember(name, entry);
            } else {
                legacy.push((name, entry));
            }
        }
        Ok(legacy)
    }

    pub fn persistent_named(&self) -> Vec<(String, NamedRosterEntry)> {
        self.roster.persistent_entries()
    }

    /// Persistance attestée par nom, pour publication à l'annuaire.
    ///
    /// Lue au roster nommé et non à l'historique `spawn_commands` : le roster
    /// ne garde qu'une entrée par nom, donc la dernière génération connectée,
    /// alors que la table conserve toutes les générations et rend une ligne
    /// arbitraire dès qu'on la groupe sans trier. C'est aussi la source que
    /// consulte `drain_legacy_non_persistent_named` : l'annuaire publie donc
    /// exactement ce qui décidera du drain.
    pub fn named_persistence(&self) -> BTreeMap<String, bool> {
        let mut persistence = self.roster.persistence_by_name();
        if let Ok(desired) = self.desired.load() {
            for (name, entry) in desired.equipiers {
                if entry.lifecycle_state != DesiredLifecycleState::Decommissioned {
                    persistence.insert(name, entry.persistent);
                }
            }
        }
        persistence
    }

    pub fn forget_named(&self, name: &str) {
        self.roster.forget(name);
    }

    pub fn persist_recovery_losses(
        &self,
        recorded_at: i64,
        absents: Vec<RecoveryLossEntry>,
    ) -> Result<(), FleetError> {
        persist_report(&report_path(self.desired.path()), recorded_at, absents)
            .map_err(FleetError::Observation)
    }

    pub fn recovery_losses_path(&self) -> PathBuf {
        report_path(self.desired.path())
    }

    pub fn desired_domain(&self, name: &str) -> Option<String> {
        self.desired
            .load()
            .ok()?
            .equipiers
            .get(name)?
            .domain
            .clone()
    }

    pub fn set_desired_domain(&self, name: &str, domain: Option<&str>) -> Result<(), FleetError> {
        self.desired.set_domain(name, domain)?;
        Ok(())
    }
    pub fn agent_link_for_child(
        &self,
        child_instance_id: &str,
    ) -> Result<Option<AgentLink>, FleetError> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .idempotency
            .agent_link_for_child(child_instance_id)
            .map_err(Into::into)
    }

    /// Persiste un fait runtime en déduisant le parent du lien enfant durable.
    /// Aucun champ de parent ne traverse cette façade.
    pub fn record_delegated_runtime_event(
        &self,
        input: DelegatedRuntimeEventInput,
    ) -> Result<DelegatedRuntimeEventRecord, FleetError> {
        let event = {
            let mut inner = self
                .inner
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            inner.idempotency.record_delegated_runtime_event(input)?
        };
        self.agent_link_changed.notify_all();
        Ok(event)
    }

    /// Relit les faits runtime non accusés sans balayer les équipiers.
    pub fn delegated_runtime_events_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<DelegatedRuntimeEventRecord>, FleetError> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .idempotency
            .delegated_runtime_events_for_parent(parent_instance_id)
            .map_err(Into::into)
    }

    /// Accuse le fait au nom du parent attesté par le daemon.
    pub fn acknowledge_delegated_runtime_event(
        &self,
        event_id: &str,
        parent_instance_id: &str,
    ) -> Result<bool, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .idempotency
            .acknowledge_delegated_runtime_event(event_id, parent_instance_id)
            .map_err(Into::into)
    }

    pub fn agent_link_summary_for_child(
        &self,
        child_instance_id: &str,
    ) -> Result<Option<AgentLinkSummary>, FleetError> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(link) = inner.idempotency.agent_link_for_child(child_instance_id)? else {
            return Ok(None);
        };
        let (direct_descendants, descendants) = inner
            .idempotency
            .open_agent_link_descendant_counts(child_instance_id)?;
        Ok(Some(AgentLinkSummary {
            link,
            direct_descendants,
            descendants,
        }))
    }

    pub fn agent_links_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<AgentLink>, FleetError> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        inner
            .idempotency
            .open_agent_links_for_parent(parent_instance_id)
            .map_err(Into::into)
    }
    /// Attend un changement durable de descendance sans polling. Une reprise
    /// peut fournir le dernier curseur reçu afin de relire uniquement le delta.
    pub fn wait_for_agent_link_events(
        &self,
        parent_instance_id: &str,
        after_cursor: Option<u64>,
        timeout: Duration,
    ) -> Result<AgentLinkEventBatch, FleetError> {
        let inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let events = inner
            .idempotency
            .agent_link_events_after(parent_instance_id, after_cursor)?;
        if !events.is_empty() {
            return Ok(AgentLinkEventBatch {
                through_cursor: events.last().map(|event| event.cursor),
                events,
            });
        }
        let (inner, _) = self
            .agent_link_changed
            .wait_timeout_while(inner, timeout, |state| {
                state
                    .idempotency
                    .agent_link_events_after(parent_instance_id, after_cursor)
                    .map(|events| events.is_empty())
                    .unwrap_or(false)
            })
            .unwrap_or_else(|poison| poison.into_inner());
        let events = inner
            .idempotency
            .agent_link_events_after(parent_instance_id, after_cursor)?;
        Ok(AgentLinkEventBatch {
            through_cursor: events.last().map(|event| event.cursor),
            events,
        })
    }

    pub fn orphan_agent_links_for_parent(
        &self,
        parent_instance_id: &str,
        now: i64,
    ) -> Result<usize, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let changed = inner
            .idempotency
            .orphan_agent_links_for_parent(parent_instance_id, now)?;
        drop(inner);
        if changed > 0 {
            self.agent_link_changed.notify_all();
        }
        Ok(changed)
    }

    pub fn transfer_agent_link(
        &self,
        link_id: &str,
        new_parent_instance_id: &str,
        now: i64,
    ) -> Result<AgentLink, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let current = inner
            .idempotency
            .agent_link_by_id(link_id)?
            .ok_or(FleetError::Ownership("lien de transfert introuvable"))?;
        let parent_path = inner
            .idempotency
            .agent_link_for_child(new_parent_instance_id)?
            .map(|link| link.agent_path)
            .unwrap_or_else(|| new_parent_instance_id.to_string());
        if new_parent_instance_id == current.child_instance_id
            || agent_path_contains(&parent_path, &current.child_instance_id)
        {
            return Err(FleetError::Ownership("cycle d'ascendance"));
        }
        let path = format!("{}/{}", parent_path, current.child_instance_id);
        let transferred =
            inner
                .idempotency
                .transfer_agent_link(link_id, new_parent_instance_id, &path, now)?;
        drop(inner);
        self.agent_link_changed.notify_all();
        Ok(transferred)
    }

    /// Réserve une clé puis applique, sous le même verrou métier, les gardes
    /// mutables de nom et de quota. Une clé rejouée ne traverse jamais ces
    /// gardes et ne peut donc pas lancer une seconde génération.
    pub fn request_spawn(
        &self,
        order: &SpawnOrder,
        now: i64,
    ) -> Result<SpawnSubmission, FleetError> {
        self.request_spawn_with_policy(order, now, None, false)
    }

    /// Réserve une nouvelle génération sous un nom arrêté déjà présent dans
    /// l'inventaire. Aucun autre état existant n'est contourné.
    pub fn request_relaunch(
        &self,
        order: &SpawnOrder,
        now: i64,
    ) -> Result<SpawnSubmission, FleetError> {
        let Some(name) = order.requested_name.as_deref() else {
            return Err(FleetError::InvalidOrder("nom de relance absent"));
        };
        let Some(entry) = self.desired_entry(name)? else {
            return Err(FleetError::InvalidOrder("agent de relance absent"));
        };
        if entry.lifecycle_state != DesiredLifecycleState::Stopped {
            return Err(FleetError::InvalidOrder("agent de relance non arrêté"));
        }
        self.request_spawn_with_policy(order, now, Some(DesiredLifecycleState::Stopped), true)
    }

    /// Réserve une nouvelle génération de reprise sous une entrée persistante
    /// encore déclarée running. Ce contournement n'est accessible qu'au
    /// chemin de reprise du daemon.
    pub fn request_recovery(
        &self,
        order: &SpawnOrder,
        now: i64,
    ) -> Result<SpawnSubmission, FleetError> {
        let Some(name) = order.requested_name.as_deref() else {
            return Err(FleetError::InvalidOrder("nom de reprise absent"));
        };
        let Some(entry) = self.desired_entry(name)? else {
            return Err(FleetError::InvalidOrder("agent de reprise absent"));
        };
        if entry.lifecycle_state != DesiredLifecycleState::Running || !entry.persistent {
            return Err(FleetError::InvalidOrder(
                "agent de reprise non persistant ou non actif",
            ));
        }
        self.request_spawn_with_policy(order, now, Some(DesiredLifecycleState::Running), true)
    }

    fn request_spawn_with_policy(
        &self,
        order: &SpawnOrder,
        now: i64,
        allowed_existing_state: Option<DesiredLifecycleState>,
        accept_legacy_durable_name: bool,
    ) -> Result<SpawnSubmission, FleetError> {
        validate_order(order, accept_legacy_durable_name)?;
        let canonical = canonical_order(order)?;
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let generation = inner.next_generation;
        let name = resolve_name(order, generation, &inner.active_by_name);
        let candidate_instance_id = uuid::Uuid::new_v4().to_string();
        let key = IdempotencyKey::new(
            inner.supervisor_scope.clone(),
            OperationKind::Spawn,
            order.command_id.clone(),
        )?;
        let horizon = if order.persistent {
            self.config.persistent_horizon_secs
        } else {
            self.config.ephemeral_horizon_secs
        };
        let reservation = inner.idempotency.reserve_spawn(
            &key,
            &canonical,
            order.issued_at,
            horizon,
            now,
            self.config.issued_at_tolerance_secs,
            &name,
            generation,
            order.persistent,
            order.deadline_at,
            &candidate_instance_id,
        )?;
        match reservation {
            SpawnReservation::Requested(command) => {
                inner.next_generation = generation.saturating_add(1).max(1);
                if now >= order.deadline_at {
                    return terminal_refusal(
                        &mut inner,
                        &key,
                        &command,
                        "spawn_timeout",
                        "délai absolu dépassé avant réservation",
                    );
                }
                if inner.active_by_name.contains_key(&command.name) {
                    return terminal_refusal(
                        &mut inner,
                        &key,
                        &command,
                        "name_active",
                        "nom déjà actif",
                    );
                }
                let existing = self.desired.load()?.equipiers.remove(&command.name);
                let allowed = allowed_existing_state.is_some_and(|allowed_state| {
                    existing
                        .as_ref()
                        .is_some_and(|entry| entry.lifecycle_state == allowed_state)
                });
                if existing.is_some() && !allowed {
                    return terminal_refusal(
                        &mut inner,
                        &key,
                        &command,
                        "name_reserved",
                        "nom réservé par un agent géré",
                    );
                }
                if inner.active_by_command.len() >= self.config.quota {
                    let detail = quota_exceeded_detail(self.config.quota);
                    return terminal_refusal(&mut inner, &key, &command, "quota_exceeded", &detail);
                }
                let link = match reserve_agent_link(
                    &mut inner,
                    &command,
                    order.ownership.as_ref(),
                    order.project.as_ref(),
                    now,
                ) {
                    Ok(link) => link,
                    Err(FleetError::Ownership("quota enfants atteint")) => {
                        return terminal_refusal(
                            &mut inner,
                            &key,
                            &command,
                            "children_quota_exceeded",
                            "nombre maximal d'enfants atteint",
                        );
                    }
                    Err(FleetError::Ownership("profondeur maximale atteinte")) => {
                        return terminal_refusal(
                            &mut inner,
                            &key,
                            &command,
                            "depth_exceeded",
                            "profondeur maximale atteinte",
                        );
                    }
                    Err(error) => return Err(error),
                };
                if let Err(error) = inner.idempotency.advance_spawn(
                    &key,
                    command.generation,
                    SpawnCommandState::Requested,
                    SpawnCommandState::Reserved,
                ) {
                    if let Some(link) = &link {
                        let _ = inner.idempotency.transition_agent_link(
                            &link.link_id,
                            &[AgentLinkState::Reserved],
                            AgentLinkState::Closed,
                            now,
                        );
                    }
                    return Err(error.into());
                }
                if link.is_some() {
                    self.agent_link_changed.notify_all();
                }
                let active = ActiveSpawn {
                    command_id: command.command_id.clone(),
                    name: command.name.clone(),
                    instance_id: command.instance_id.clone().ok_or(
                        IdempotencyError::CorruptRecord("instance spawn en vol absente"),
                    )?,
                    generation: command.generation,
                    deadline_at: command.deadline_at,
                    persistent: command.persistent,
                    project: order.project.clone(),
                    agent_type: order.agent_type.clone(),
                    cwd: order.cwd.clone(),
                    state: SpawnCommandState::Reserved,
                    link_id: link.as_ref().map(|link| link.link_id.clone()),
                    ownership: order.ownership.clone(),
                    agent_path: link.as_ref().map(|link| link.agent_path.clone()),
                    resolved_definition: command.resolved_definition,
                    runtime_execution: None,
                };
                inner
                    .active_by_name
                    .insert(active.name.clone(), active.command_id.clone());
                inner
                    .active_by_command
                    .insert(active.command_id.clone(), active.clone());
                Ok(SpawnSubmission::Start(lease_from(&active)))
            }
            SpawnReservation::InFlight(command) => {
                if !inner.active_by_command.contains_key(&command.command_id) {
                    let issue = SpawnCommandIssue::Failed {
                        category: "daemon_restarted".to_string(),
                        reason: "génération sans état désiré après redémarrage".to_string(),
                    };
                    inner
                        .idempotency
                        .finish_spawn(&key, command.generation, &issue)?;
                    inner.completed.insert(command.command_id, issue.clone());
                    self.terminal_changed.notify_all();
                    return Ok(SpawnSubmission::Terminal(issue));
                }
                Ok(SpawnSubmission::Await(SpawnWaiter {
                    command_id: command.command_id,
                    instance_id: command.instance_id.ok_or(IdempotencyError::CorruptRecord(
                        "instance spawn en vol absente",
                    ))?,
                    generation: command.generation,
                    deadline_at: command.deadline_at,
                }))
            }
            SpawnReservation::Replayed(issue) => Ok(SpawnSubmission::Terminal(issue)),
            SpawnReservation::EnvelopeMismatch => Ok(SpawnSubmission::EnvelopeMismatch),
            SpawnReservation::IdempotencyExpired => Ok(SpawnSubmission::IdempotencyExpired),
        }
    }

    pub fn mark_starting(
        &self,
        lease: &SpawnLease,
        now: i64,
        definition: &ResolvedAgentDefinition,
    ) -> Result<(), FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let active = active_for_lease(&inner, lease)?.clone();
        if now >= active.deadline_at {
            expire_locked(&mut inner, &self.desired, &self.roster, &active)?;
            self.agent_link_changed.notify_all();
            self.terminal_changed.notify_all();
            return Err(FleetError::DeadlineElapsed);
        }
        if active.state != SpawnCommandState::Reserved {
            return Err(FleetError::InvalidTransition {
                command_id: active.command_id,
                from: active.state,
                expected: SpawnCommandState::Reserved,
            });
        }
        let key = spawn_key(&inner, &active.command_id)?;
        inner.idempotency.advance_spawn_with_definition(
            &key,
            active.generation,
            SpawnCommandState::Reserved,
            SpawnCommandState::Starting,
            definition,
        )?;
        let active = inner
            .active_by_command
            .get_mut(&active.command_id)
            .expect("la génération a été validée sous le verrou");
        active.state = SpawnCommandState::Starting;
        active.resolved_definition = Some(definition.clone());
        Ok(())
    }

    pub fn register_connected(
        &self,
        lease: &SpawnLease,
        instance_id: &str,
        now: i64,
    ) -> Result<SpawnCommandIssue, FleetError> {
        self.register_connected_observed(lease, instance_id, now, || Ok(()))
    }

    /// Même chemin de production avec une barrière après `fleet.json` et
    /// avant l'issue SQLite, pour les crash-tests déterministes D-503.
    pub fn register_connected_observed(
        &self,
        lease: &SpawnLease,
        instance_id: &str,
        now: i64,
        after_fleet: impl FnOnce() -> io::Result<()>,
    ) -> Result<SpawnCommandIssue, FleetError> {
        if instance_id.is_empty() {
            return Err(FleetError::InvalidOrder("instance_id vide"));
        }
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let active = active_for_lease(&inner, lease)?.clone();
        if instance_id != active.instance_id {
            return Err(FleetError::StaleGeneration);
        }
        if now >= active.deadline_at {
            expire_locked(&mut inner, &self.desired, &self.roster, &active)?;
            self.agent_link_changed.notify_all();
            self.terminal_changed.notify_all();
            return Err(FleetError::DeadlineElapsed);
        }
        if active.state != SpawnCommandState::Starting {
            return Err(FleetError::InvalidTransition {
                command_id: active.command_id,
                from: active.state,
                expected: SpawnCommandState::Starting,
            });
        }
        let runtime_execution = active.runtime_execution.clone().map(|mut execution| {
            execution.state = crate::desired_state::ContainerAgentExecutionState::Running;
            execution
        });
        self.desired.upsert(
            active.name.clone(),
            DesiredEquipier {
                agent_type: active.agent_type.clone(),
                cwd: active.cwd.clone(),
                command_id: active.command_id.clone(),
                generation: active.generation,
                created: now.to_string(),
                persistent: active.persistent,
                lifecycle_state: DesiredLifecycleState::Running,
                agent_link: desired_agent_link(&active),
                resolved_definition: active.resolved_definition.clone(),
                domain: resolved_domain(None, &active.cwd),
                project: active.project.clone(),
                runtime_execution,
            },
        )?;
        self.roster.remember(
            active.name.clone(),
            NamedRosterEntry {
                agent_type: active.agent_type.clone(),
                persistent: active.persistent,
                domain: resolved_domain(None, &active.cwd),
            },
        );
        after_fleet().map_err(FleetError::Observation)?;
        if let Some(link_id) = &active.link_id {
            inner.idempotency.transition_agent_link(
                link_id,
                &[AgentLinkState::Reserved, AgentLinkState::Transferred],
                AgentLinkState::Open,
                now,
            )?;
        }
        let issue = SpawnCommandIssue::Connected {
            name: active.name.clone(),
            generation: active.generation,
            instance_id: instance_id.to_string(),
            definition: active.resolved_definition.clone().map(Box::new),
        };
        let key = spawn_key(&inner, &active.command_id)?;
        inner
            .idempotency
            .finish_spawn(&key, active.generation, &issue)?;
        complete_locked(&mut inner, &active, issue.clone());
        self.agent_link_changed.notify_all();
        self.terminal_changed.notify_all();
        Ok(issue)
    }

    pub fn fail(
        &self,
        lease: &SpawnLease,
        category: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<SpawnCommandIssue, FleetError> {
        let issue = SpawnCommandIssue::Failed {
            category: category.into(),
            reason: reason.into(),
        };
        self.finish_non_success(lease, issue)
    }

    pub fn cancel(
        &self,
        lease: &SpawnLease,
        reason: impl Into<String>,
    ) -> Result<SpawnCommandIssue, FleetError> {
        let issue = SpawnCommandIssue::Cancelled {
            reason: reason.into(),
        };
        self.finish_non_success(lease, issue)
    }

    /// Invalide la génération avant toute E/S d'arrêt. Une génération encore
    /// en lancement devient `Cancelled`; une génération déjà `Connected`
    /// conserve son issue de spawn mais passe durablement à `stopped` afin
    /// qu'aucun redémarrage ne puisse la ressusciter.
    pub fn invalidate_for_stop(&self, lease: &SpawnLease) -> Result<(), FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = inner.active_by_command.get(&lease.command_id).cloned() {
            if active.generation != lease.generation || active.name != lease.name {
                return Err(FleetError::StaleGeneration);
            }
            close_agent_link(&mut inner, &active, unix_now())?;
            self.persist_stopped_active(&active)?;
            let issue = SpawnCommandIssue::Cancelled {
                reason: "arrêt demandé".to_string(),
            };
            let key = spawn_key(&inner, &active.command_id)?;
            inner
                .idempotency
                .finish_spawn(&key, active.generation, &issue)?;
            complete_locked(&mut inner, &active, issue);
            self.agent_link_changed.notify_all();
            self.terminal_changed.notify_all();
            return Ok(());
        }
        let connected = matches!(
            inner.completed.get(&lease.command_id),
            Some(SpawnCommandIssue::Connected {
                name,
                generation,
                instance_id,
                ..
            }) if name == &lease.name
                && generation == &lease.generation
                && instance_id == &lease.instance_id
        );
        if !connected {
            return Err(FleetError::StaleGeneration);
        }
        self.desired.mark_stopped_if_generation(
            &lease.name,
            &lease.command_id,
            lease.generation,
        )?;
        self.roster.remember(
            lease.name.clone(),
            NamedRosterEntry {
                agent_type: self
                    .desired_entry(&lease.name)?
                    .map(|entry| entry.agent_type)
                    .unwrap_or_else(|| "unknown".to_string()),
                persistent: lease.persistent,
                domain: self
                    .desired_entry(&lease.name)?
                    .and_then(|entry| entry.domain),
            },
        );
        if let Some(link_id) = &lease.link_id {
            inner.idempotency.transition_agent_link(
                link_id,
                &[
                    AgentLinkState::Reserved,
                    AgentLinkState::Open,
                    AgentLinkState::Transferred,
                    AgentLinkState::Orphaned,
                ],
                AgentLinkState::Closed,
                unix_now(),
            )?;
        }
        self.agent_link_changed.notify_all();
        Ok(())
    }

    fn persist_stopped_active(&self, active: &ActiveSpawn) -> Result<(), FleetError> {
        let existing = self.desired_entry(&active.name)?;
        if let Some(entry) = existing {
            if entry.command_id == active.command_id && entry.generation == active.generation {
                self.desired.mark_stopped_if_generation(
                    &active.name,
                    &active.command_id,
                    active.generation,
                )?;
            } else {
                self.desired
                    .set_lifecycle_state(&active.name, DesiredLifecycleState::Stopped)?;
            }
            self.roster.remember(
                active.name.clone(),
                NamedRosterEntry {
                    agent_type: entry.agent_type,
                    persistent: entry.persistent,
                    domain: entry.domain,
                },
            );
            return Ok(());
        }
        let Some(resolved_definition) = active.resolved_definition.clone() else {
            self.roster.forget(&active.name);
            return Ok(());
        };
        let domain = resolved_domain(None, &active.cwd);
        let runtime_execution = active.runtime_execution.clone().map(|mut execution| {
            execution.state = crate::desired_state::ContainerAgentExecutionState::Terminal;
            execution
        });
        self.desired.upsert(
            active.name.clone(),
            DesiredEquipier {
                agent_type: active.agent_type.clone(),
                cwd: active.cwd.clone(),
                command_id: active.command_id.clone(),
                generation: active.generation,
                created: unix_now().to_string(),
                persistent: active.persistent,
                lifecycle_state: DesiredLifecycleState::Stopped,
                resolved_definition: Some(resolved_definition),
                domain: domain.clone(),
                project: active.project.clone(),
                agent_link: desired_agent_link(active),
                runtime_execution,
            },
        )?;
        self.roster.remember(
            active.name.clone(),
            NamedRosterEntry {
                agent_type: active.agent_type.clone(),
                persistent: active.persistent,
                domain,
            },
        );
        Ok(())
    }

    fn finish_non_success(
        &self,
        lease: &SpawnLease,
        issue: SpawnCommandIssue,
    ) -> Result<SpawnCommandIssue, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let active = active_for_lease(&inner, lease)?.clone();
        if self.desired_entry(&active.name)?.is_some() {
            self.persist_stopped_active(&active)?;
        } else {
            self.roster.forget(&active.name);
        }
        close_agent_link(&mut inner, &active, unix_now())?;
        let key = spawn_key(&inner, &active.command_id)?;
        inner
            .idempotency
            .finish_spawn(&key, active.generation, &issue)?;
        complete_locked(&mut inner, &active, issue.clone());
        self.agent_link_changed.notify_all();
        self.terminal_changed.notify_all();
        Ok(issue)
    }

    /// Expire le tour uniquement si l'échéance absolue est atteinte. Le
    /// `Register` ultérieur échoue ensuite comme génération obsolète.
    pub fn expire(&self, command_id: &str, generation: u64, now: i64) -> Result<bool, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let Some(active) = inner.active_by_command.get(command_id).cloned() else {
            return Ok(false);
        };
        if active.generation != generation {
            return Err(FleetError::StaleGeneration);
        }
        if now < active.deadline_at {
            return Ok(false);
        }
        expire_locked(&mut inner, &self.desired, &self.roster, &active)?;
        self.agent_link_changed.notify_all();
        self.terminal_changed.notify_all();
        Ok(true)
    }

    pub fn wait_for_terminal(
        &self,
        waiter: &SpawnWaiter,
        timeout: Duration,
    ) -> Result<Option<SpawnCommandIssue>, FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(issue) = inner.completed.get(&waiter.command_id) {
            return Ok(Some(issue.clone()));
        }
        if inner
            .active_by_command
            .get(&waiter.command_id)
            .is_some_and(|active| active.generation != waiter.generation)
        {
            return Err(FleetError::StaleGeneration);
        }
        let (guard, _) = self
            .terminal_changed
            .wait_timeout_while(inner, timeout, |state| {
                !state.completed.contains_key(&waiter.command_id)
            })
            .unwrap_or_else(|poison| poison.into_inner());
        inner = guard;
        Ok(inner.completed.get(&waiter.command_id).cloned())
    }
}

fn recover_commands(inner: &mut FleetInner, desired: &DesiredFleet) -> Result<(), FleetError> {
    let commands = inner.idempotency.spawn_commands(&inner.supervisor_scope)?;
    for command in commands {
        inner.next_generation = inner
            .next_generation
            .max(command.generation.saturating_add(1));
        if let Some(issue) = command.issue.clone() {
            inner.completed.insert(command.command_id.clone(), issue);
            continue;
        }
        if let Some(equipier) = desired.equipiers.get(&command.name)
            && equipier.command_id == command.command_id
            && equipier.generation == command.generation
            && equipier.persistent
            && equipier.lifecycle_state == DesiredLifecycleState::Running
        {
            let active =
                ActiveSpawn {
                    command_id: command.command_id.clone(),
                    name: command.name.clone(),
                    instance_id: command.instance_id.clone().ok_or(
                        IdempotencyError::CorruptRecord("instance spawn reprise absente"),
                    )?,
                    generation: command.generation,
                    deadline_at: command.deadline_at,
                    persistent: equipier.persistent,
                    project: equipier.project.clone(),
                    link_id: equipier
                        .agent_link
                        .as_ref()
                        .map(|link| link.link_id.clone()),
                    ownership: ownership_from_desired(equipier),
                    agent_path: equipier
                        .agent_link
                        .as_ref()
                        .map(|link| link.agent_path.clone()),
                    agent_type: equipier.agent_type.clone(),
                    cwd: equipier.cwd.clone(),
                    state: SpawnCommandState::Starting,
                    resolved_definition: command.resolved_definition,
                    runtime_execution: equipier.runtime_execution.clone(),
                };
            inner
                .active_by_name
                .insert(active.name.clone(), active.command_id.clone());
            inner
                .active_by_command
                .insert(active.command_id.clone(), active);
            continue;
        }
        let issue = SpawnCommandIssue::Failed {
            category: "daemon_restarted".to_string(),
            reason: "ordre en vol sans état désiré durable".to_string(),
        };
        let key = spawn_key(inner, &command.command_id)?;
        inner
            .idempotency
            .finish_spawn(&key, command.generation, &issue)?;
        inner.completed.insert(command.command_id, issue);
    }
    Ok(())
}

fn validate_order(order: &SpawnOrder, accept_legacy_durable_name: bool) -> Result<(), FleetError> {
    if order.agent_type.trim().is_empty() {
        return Err(FleetError::InvalidOrder("type vide"));
    }
    if order.command_id.trim().is_empty() {
        return Err(FleetError::InvalidOrder("command_id vide"));
    }
    if !order.cwd.is_absolute() {
        return Err(FleetError::InvalidOrder("cwd non absolu"));
    }
    if order.deadline_at <= order.issued_at {
        return Err(FleetError::InvalidOrder(
            "échéance non postérieure à issued_at",
        ));
    }
    let Some(agent_id) = order.requested_name.as_deref() else {
        return Err(FleetError::InvalidOrder("agent_id de spawn absent"));
    };
    if validate_agent_id(agent_id).is_err() && !accept_legacy_durable_name {
        return Err(FleetError::InvalidOrder("agent_id de spawn invalide"));
    }
    if let Some(ownership) = &order.ownership
        && (ownership.parent_instance_id.trim().is_empty() || ownership.role.trim().is_empty())
    {
        return Err(FleetError::InvalidOrder("propriété parent ou rôle vide"));
    }
    if let Some(project) = &order.project
        && (project.project_id.trim().is_empty() || project.binding_generation == 0)
    {
        return Err(FleetError::InvalidOrder("référence projet invalide"));
    }
    Ok(())
}

fn reserve_agent_link(
    inner: &mut FleetInner,
    command: &SpawnCommand,
    ownership: Option<&SpawnOwnership>,
    project: Option<&ProjectReference>,
    now: i64,
) -> Result<Option<AgentLink>, FleetError> {
    let Some(ownership) = ownership else {
        return Ok(None);
    };
    let current_children = inner
        .idempotency
        .open_agent_links_for_parent(&ownership.parent_instance_id)?;
    if ownership
        .max_children
        .is_some_and(|limit| current_children.len() >= limit)
    {
        return Err(FleetError::Ownership("quota enfants atteint"));
    }
    let parent_path = inner
        .idempotency
        .agent_link_for_child(&ownership.parent_instance_id)?
        .map(|link| link.agent_path)
        .unwrap_or_else(|| ownership.parent_instance_id.clone());
    let child_depth = parent_path.split('/').count();
    if ownership.max_depth.is_some_and(|limit| child_depth > limit) {
        return Err(FleetError::Ownership("profondeur maximale atteinte"));
    }
    let child_instance_id = command
        .instance_id
        .clone()
        .ok_or(IdempotencyError::CorruptRecord(
            "instance spawn en vol absente",
        ))?;
    let link = AgentLink {
        link_id: uuid::Uuid::new_v4().to_string(),
        parent_instance_id: ownership.parent_instance_id.clone(),
        child_instance_id: child_instance_id.clone(),
        parent_execution_id: ownership.parent_execution_id.clone(),
        objective_id: ownership.objective_id.clone(),
        delegation_id: ownership.delegation_id.clone(),
        project: project.cloned(),
        role: ownership.role.clone(),
        agent_path: format!("{parent_path}/{child_instance_id}"),
        state: AgentLinkState::Reserved,
        created_at: now,
        closed_at: None,
        revision: 0,
    };
    inner.idempotency.create_agent_link(&link)?;
    Ok(Some(link))
}

fn desired_agent_link(active: &ActiveSpawn) -> Option<DesiredAgentLink> {
    let (Some(link_id), Some(ownership), Some(agent_path)) = (
        active.link_id.as_ref(),
        active.ownership.as_ref(),
        active.agent_path.as_ref(),
    ) else {
        return None;
    };
    Some(DesiredAgentLink {
        link_id: link_id.clone(),
        parent_instance_id: ownership.parent_instance_id.clone(),
        parent_execution_id: ownership.parent_execution_id.clone(),
        objective_id: ownership.objective_id.clone(),
        delegation_id: ownership.delegation_id.clone(),
        project: active.project.clone(),
        role: ownership.role.clone(),
        agent_path: agent_path.clone(),
    })
}

fn agent_path_contains(path: &str, instance_id: &str) -> bool {
    path.split('/').any(|segment| segment == instance_id)
}

fn ownership_from_desired(equipier: &DesiredEquipier) -> Option<SpawnOwnership> {
    let link = equipier.agent_link.as_ref()?;
    Some(SpawnOwnership {
        parent_instance_id: link.parent_instance_id.clone(),
        parent_execution_id: link.parent_execution_id.clone(),
        objective_id: link.objective_id.clone(),
        delegation_id: link.delegation_id.clone(),
        project: equipier.project.clone(),
        role: link.role.clone(),
        max_children: None,
        max_depth: None,
    })
}
#[derive(Serialize)]
struct CanonicalSpawnOrder<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    posture: Option<bridget_transport::protocol::SpawnPosture>,
    command_id: &'a str,
    agent_type: &'a str,
    requested_name: &'a Option<String>,
    cwd: &'a str,
    persistent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<&'a ProjectReference>,
    issued_at: i64,
    deadline_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    ownership: Option<&'a SpawnOwnership>,
}

fn canonical_order(order: &SpawnOrder) -> Result<Vec<u8>, FleetError> {
    let cwd = order
        .cwd
        .to_str()
        .ok_or(FleetError::InvalidOrder("cwd non UTF-8"))?;
    serde_json::to_vec(&CanonicalSpawnOrder {
        posture: order.posture,
        command_id: &order.command_id,
        agent_type: &order.agent_type,
        requested_name: &order.requested_name,
        cwd,
        persistent: order.persistent,
        project: order.project.as_ref(),
        ownership: order.ownership.as_ref(),
        issued_at: order.issued_at,
        deadline_at: order.deadline_at,
    })
    .map_err(|_| FleetError::InvalidOrder("canon non sérialisable"))
}

fn resolve_name(order: &SpawnOrder, _generation: u64, _active: &HashMap<String, String>) -> String {
    // `validate_order` garantit l'existence et le format de l'agent_id.
    order
        .requested_name
        .clone()
        .expect("validate_order exige un agent_id de spawn")
}

fn lease_from(active: &ActiveSpawn) -> SpawnLease {
    SpawnLease {
        command_id: active.command_id.clone(),
        name: active.name.clone(),
        instance_id: active.instance_id.clone(),
        generation: active.generation,
        deadline_at: active.deadline_at,
        persistent: active.persistent,
        project: active.project.clone(),
        link_id: active.link_id.clone(),
        ownership: active.ownership.clone(),
        agent_path: active.agent_path.clone(),
    }
}

fn active_for_lease<'a>(
    inner: &'a FleetInner,
    lease: &SpawnLease,
) -> Result<&'a ActiveSpawn, FleetError> {
    let active = inner
        .active_by_command
        .get(&lease.command_id)
        .ok_or(FleetError::StaleGeneration)?;
    if active.generation != lease.generation || active.name != lease.name {
        return Err(FleetError::StaleGeneration);
    }
    Ok(active)
}

fn spawn_key(inner: &FleetInner, command_id: &str) -> Result<IdempotencyKey, FleetError> {
    IdempotencyKey::new(
        inner.supervisor_scope.clone(),
        OperationKind::Spawn,
        command_id.to_string(),
    )
    .map_err(Into::into)
}

fn terminal_refusal(
    inner: &mut FleetInner,
    key: &IdempotencyKey,
    command: &SpawnCommand,
    category: &str,
    reason: &str,
) -> Result<SpawnSubmission, FleetError> {
    let issue = SpawnCommandIssue::Failed {
        category: category.to_string(),
        reason: reason.to_string(),
    };
    inner
        .idempotency
        .finish_spawn(key, command.generation, &issue)?;
    inner
        .completed
        .insert(command.command_id.clone(), issue.clone());
    Ok(SpawnSubmission::Terminal(issue))
}

fn complete_locked(inner: &mut FleetInner, active: &ActiveSpawn, issue: SpawnCommandIssue) {
    inner.active_by_command.remove(&active.command_id);
    if inner
        .active_by_name
        .get(&active.name)
        .is_some_and(|command_id| command_id == &active.command_id)
    {
        inner.active_by_name.remove(&active.name);
    }
    inner.completed.insert(active.command_id.clone(), issue);
}
fn close_agent_link(
    inner: &mut FleetInner,
    active: &ActiveSpawn,
    changed_at: i64,
) -> Result<(), FleetError> {
    if let Some(link_id) = &active.link_id {
        inner.idempotency.transition_agent_link(
            link_id,
            &[
                AgentLinkState::Reserved,
                AgentLinkState::Open,
                AgentLinkState::Transferred,
                AgentLinkState::Orphaned,
            ],
            AgentLinkState::Closed,
            changed_at,
        )?;
    }
    Ok(())
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn expire_locked(
    inner: &mut FleetInner,
    desired: &DesiredStateStore,
    roster: &NamedRosterStore,
    active: &ActiveSpawn,
) -> Result<(), FleetError> {
    let retained =
        desired.mark_stopped_if_generation(&active.name, &active.command_id, active.generation)?;
    if !retained {
        roster.forget(&active.name);
    }
    close_agent_link(inner, active, unix_now())?;
    let issue = SpawnCommandIssue::Cancelled {
        reason: "spawn_timeout".to_string(),
    };
    let key = spawn_key(inner, &active.command_id)?;
    inner
        .idempotency
        .finish_spawn(&key, active.generation, &issue)?;
    complete_locked(inner, active, issue);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;
    use std::os::fd::{AsRawFd, RawFd};
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::CommandExt;
    use std::process::{Child, Command, Stdio};
    use std::sync::{Arc, Barrier};
    use std::thread;

    // Identités v4 stables : le même agent reste identique après rejeu/redémarrage.
    const AGENT_DOMAIN: &str = "00000001-0000-4000-8000-000000000001";
    const AGENT_SCOPE: &str = "00000002-0000-4000-8000-000000000001";
    const AGENT_ADOPT: &str = "00000003-0000-4000-8000-000000000001";
    const AGENT_RELAUNCH: &str = "00000004-0000-4000-8000-000000000001";
    const AGENT_ALPHA: &str = "00000005-0000-4000-8000-000000000001";
    const AGENT_ZETA: &str = "00000006-0000-4000-8000-000000000001";
    const AGENT_UNIQUE: &str = "00000007-0000-4000-8000-000000000001";
    const AGENT_WAIT: &str = "00000008-0000-4000-8000-000000000001";
    const AGENT_QUOTA_A: &str = "00000009-0000-4000-8000-000000000001";
    const AGENT_QUOTA_B: &str = "00000010-0000-4000-8000-000000000001";
    const AGENT_GUARD: &str = "00000011-0000-4000-8000-000000000001";
    const AGENT_EXPIRED: &str = "00000012-0000-4000-8000-000000000001";
    const AGENT_STOP: &str = "00000013-0000-4000-8000-000000000001";
    const AGENT_CRASH: &str = "00000014-0000-4000-8000-000000000001";

    const NOW: i64 = 1_000_000;
    const CRASH_CHILD_ENV: &str = "BRIDGET_T904_CRASH_CHILD";
    const CRASH_ROOT_ENV: &str = "BRIDGET_T904_CRASH_ROOT";
    const CRASH_STAGE_ENV: &str = "BRIDGET_T904_CRASH_STAGE";
    const CRASH_BARRIER_FD: RawFd = 112;

    fn test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t904-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn config() -> FleetConfig {
        FleetConfig {
            quota: 2,
            persistent_horizon_secs: 3600,
            ephemeral_horizon_secs: 300,
            issued_at_tolerance_secs: 30,
        }
    }

    fn open(root: &Path) -> FleetSupervisor {
        fs::create_dir_all(root).unwrap();
        FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            config(),
        )
        .unwrap()
    }

    fn order(command_id: &str, name: Option<&str>, persistent: bool) -> SpawnOrder {
        SpawnOrder {
            posture: None,
            agent_type: "codex".to_string(),
            requested_name: name.map(str::to_string),
            cwd: PathBuf::from("/tmp"),
            persistent,
            command_id: command_id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 60,
            project: None,
            ownership: None,
        }
    }

    fn start(supervisor: &FleetSupervisor, order: &SpawnOrder) -> SpawnLease {
        match supervisor.request_spawn(order, NOW).unwrap() {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("nouvelle commande non démarrée: {other:?}"),
        }
    }

    #[test]
    fn domain_est_persiste_dans_fleet_json_apres_connexion() {
        let root = test_root("domain-connect");
        let supervisor = open(&root);
        let mut spawn = order("command-domain", Some(AGENT_DOMAIN), true);
        spawn.cwd = PathBuf::from("/tmp/bridget");
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        let fleet = supervisor.desired_fleet().unwrap();
        assert_eq!(
            fleet.equipiers[AGENT_DOMAIN].domain.as_deref(),
            Some("bridget")
        );
        let raw = fs::read_to_string(root.join("fleet.json")).unwrap();
        assert!(raw.contains("\"domain\": \"bridget\""), "{raw}");
        supervisor
            .set_desired_domain(AGENT_DOMAIN, Some("nouveau-projet"))
            .unwrap();
        assert_eq!(
            supervisor.desired_fleet().unwrap().equipiers[AGENT_DOMAIN]
                .domain
                .as_deref(),
            Some("nouveau-projet")
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn resolved_test_definition() -> ResolvedAgentDefinition {
        ResolvedAgentDefinition {
            command: "npx".to_string(),
            args: vec!["fixture-acp".to_string()],
            protocol: "acp".to_string(),
            forbidden_env: vec!["API_KEY".to_string()],
            pass_env: Vec::new(),
            claude_config_dir: None,
            permissions: "allow".to_string(),
            queue_capacity: 32,
            notify_timeout_secs: 600,
            mcp: bridget_transport::ResolvedMcpDefinition {
                interactive: "none".to_string(),
                acp_session: false,
            },
            capabilities: bridget_transport::AdapterCapabilities::default(),
            digest: "fixture-digest".to_string(),
        }
    }

    #[test]
    fn scope_superviseur_et_issue_persistante_survivent_au_redemarrage() {
        let root = test_root("scope");
        let supervisor = open(&root);
        let scope = supervisor.supervisor_scope();
        let spawn = order("command-scope", Some(AGENT_SCOPE), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let issue = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        assert!(matches!(
            &issue,
            SpawnCommandIssue::Connected { definition: Some(definition), .. }
                if definition.as_ref() == &resolved_test_definition()
        ));
        drop(supervisor);

        let reopened = open(&root);
        assert_eq!(reopened.supervisor_scope(), scope);
        assert_eq!(
            reopened.request_spawn(&spawn, NOW + 2).unwrap(),
            SpawnSubmission::Terminal(issue)
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adoption_exige_une_generation_connectee_et_reconstruit_un_stopped_complet() {
        let root = test_root("adopt-stopped");
        let supervisor = open(&root);
        let spawn = order("command-adopt", Some(AGENT_ADOPT), false);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();
        supervisor.remove_desired(AGENT_ADOPT).unwrap();

        assert_eq!(
            supervisor.adopt_stopped("inconnu", NOW + 2).unwrap(),
            AdoptStoppedResult::NoManagedHistory
        );
        assert_eq!(
            supervisor.adopt_stopped(AGENT_ADOPT, NOW + 2).unwrap(),
            AdoptStoppedResult::Adopted {
                generation: lease.generation
            }
        );
        let adopted = supervisor
            .desired_entry(AGENT_ADOPT)
            .unwrap()
            .expect("entrée adoptée");
        assert_eq!(adopted.lifecycle_state, DesiredLifecycleState::Stopped);
        assert!(!adopted.persistent);
        assert_eq!(adopted.cwd, PathBuf::from("/tmp"));
        assert_eq!(
            adopted.resolved_definition,
            Some(resolved_test_definition())
        );
        assert_eq!(
            supervisor.adopt_stopped(AGENT_ADOPT, NOW + 3).unwrap(),
            AdoptStoppedResult::AlreadyManaged
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relance_change_de_generation_sans_perdre_le_stopped_et_decommission_reserve_le_nom() {
        let root = test_root("relaunch-decommission");
        let supervisor = open(&root);
        let initial = order("command-initial", Some(AGENT_RELAUNCH), true);
        let initial_lease = start(&supervisor, &initial);
        supervisor
            .mark_starting(&initial_lease, NOW, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&initial_lease, &initial_lease.instance_id, NOW + 1)
            .unwrap();
        assert!(supervisor.mark_stopped(AGENT_RELAUNCH).unwrap());
        drop(supervisor);
        let supervisor = open(&root);
        assert_eq!(
            supervisor
                .desired_entry(AGENT_RELAUNCH)
                .unwrap()
                .unwrap()
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );

        let failed_order = order("command-relaunch-failed", Some(AGENT_RELAUNCH), true);
        let failed_lease = match supervisor.request_relaunch(&failed_order, NOW + 2).unwrap() {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("relance attendue: {other:?}"),
        };
        supervisor
            .mark_starting(&failed_lease, NOW + 2, &resolved_test_definition())
            .unwrap();
        supervisor
            .fail(&failed_lease, "startup_failed", "fixture")
            .unwrap();
        let retained = supervisor
            .desired_entry(AGENT_RELAUNCH)
            .unwrap()
            .expect("stopped conservé après échec");
        assert_eq!(retained.lifecycle_state, DesiredLifecycleState::Stopped);
        assert_eq!(retained.command_id, initial.command_id);

        let success_order = order("command-relaunch-ok", Some(AGENT_RELAUNCH), true);
        let success_lease = match supervisor
            .request_relaunch(&success_order, NOW + 3)
            .unwrap()
        {
            SpawnSubmission::Start(lease) => lease,
            other => panic!("nouvelle relance attendue: {other:?}"),
        };
        assert!(success_lease.generation > initial_lease.generation);
        supervisor
            .mark_starting(&success_lease, NOW + 3, &resolved_test_definition())
            .unwrap();
        supervisor
            .register_connected(&success_lease, &success_lease.instance_id, NOW + 4)
            .unwrap();
        let running = supervisor
            .desired_entry(AGENT_RELAUNCH)
            .unwrap()
            .expect("relance connectée");
        assert_eq!(running.lifecycle_state, DesiredLifecycleState::Running);
        assert_eq!(running.command_id, success_order.command_id);

        assert!(supervisor.mark_stopped(AGENT_RELAUNCH).unwrap());
        assert!(supervisor.decommission(AGENT_RELAUNCH).unwrap());
        drop(supervisor);
        let supervisor = open(&root);
        assert_eq!(
            supervisor
                .desired_entry(AGENT_RELAUNCH)
                .unwrap()
                .unwrap()
                .lifecycle_state,
            DesiredLifecycleState::Decommissioned
        );
        let historical = supervisor
            .inner
            .lock()
            .unwrap()
            .idempotency
            .latest_connected_spawn_by_name(AGENT_RELAUNCH)
            .unwrap()
            .expect("historique de la génération relancée conservé");
        assert_eq!(historical.generation, success_lease.generation);
        let reserved = order("command-reuse", Some(AGENT_RELAUNCH), true);
        assert!(matches!(
            supervisor.request_spawn(&reserved, NOW + 5).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. })
                if category == "name_reserved"
        ));
        assert!(!supervisor.named_persistence().contains_key(AGENT_RELAUNCH));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_expose_les_generations_en_vol_dans_l_ordre_des_noms() {
        let root = test_root("recovery-candidates");
        let supervisor = open(&root);
        for (command_id, name) in [("command-z", AGENT_ZETA), ("command-a", AGENT_ALPHA)] {
            let project = (name == AGENT_ALPHA).then(|| ProjectReference {
                project_id: "project-alpha".to_string(),
                binding_generation: 2,
            });
            let mut spawn = order(command_id, Some(name), true);
            spawn.project = project.clone();
            let lease = start(&supervisor, &spawn);
            assert_eq!(lease.project, project);
            supervisor
                .mark_starting(&lease, NOW, &resolved_test_definition())
                .unwrap();
            supervisor
                .desired
                .upsert(
                    name.to_string(),
                    DesiredEquipier {
                        agent_type: "codex".to_string(),
                        cwd: PathBuf::from("/tmp"),
                        command_id: command_id.to_string(),
                        generation: lease.generation,
                        created: NOW.to_string(),
                        persistent: true,
                        lifecycle_state: DesiredLifecycleState::Running,
                        resolved_definition: Some(resolved_test_definition()),
                        domain: None,
                        project: project.clone(),
                        agent_link: None,
                        runtime_execution: None,
                    },
                )
                .unwrap();
        }
        drop(supervisor);

        let reopened = open(&root);
        let candidates = reopened.recovery_candidates();
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.lease.name.as_str())
                .collect::<Vec<_>>(),
            [AGENT_ALPHA, AGENT_ZETA]
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.lease.persistent)
        );
        assert!(candidates.iter().any(|candidate| {
            candidate.lease.name == AGENT_ALPHA
                && candidate.lease.project.as_ref().is_some_and(|project| {
                    project.project_id == "project-alpha" && project.binding_generation == 2
                })
        }));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn deux_spawns_simultanes_du_meme_nom_reservent_un_seul_slot() {
        let root = test_root("same-name");
        let supervisor = Arc::new(open(&root));
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for command_id in ["command-a", "command-b"] {
            let supervisor = Arc::clone(&supervisor);
            let barrier = Arc::clone(&barrier);
            handles.push(thread::spawn(move || {
                let order = order(command_id, Some(AGENT_UNIQUE), false);
                barrier.wait();
                supervisor.request_spawn(&order, NOW).unwrap()
            }));
        }
        barrier.wait();
        let outcomes: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, SpawnSubmission::Start(_)))
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, .. })
                        if category == "name_active"
                ))
                .count(),
            1
        );
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn retry_en_vol_attend_le_vrai_register_sans_connected_synthetique() {
        let root = test_root("wait");
        let supervisor = Arc::new(open(&root));
        let spawn = order("command-wait", Some(AGENT_WAIT), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let waiter = match supervisor.request_spawn(&spawn, NOW + 1).unwrap() {
            SpawnSubmission::Await(waiter) => waiter,
            other => panic!("retry non rattaché: {other:?}"),
        };
        let waiting = Arc::clone(&supervisor);
        let wait_handle = thread::spawn(move || {
            waiting
                .wait_for_terminal(&waiter, Duration::from_secs(2))
                .unwrap()
        });
        let issue = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 2)
            .unwrap();
        assert_eq!(wait_handle.join().unwrap(), Some(issue));
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn quota_est_reserve_atomiquement_et_le_refus_est_rejouable() {
        let root = test_root("quota");
        fs::create_dir_all(&root).unwrap();
        let supervisor = FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota: 1,
                ..config()
            },
        )
        .unwrap();
        let first = order("command-quota-a", Some(AGENT_QUOTA_A), false);
        let second = order("command-quota-b", Some(AGENT_QUOTA_B), false);
        assert!(matches!(
            supervisor.request_spawn(&first, NOW).unwrap(),
            SpawnSubmission::Start(_)
        ));
        let refusal = supervisor.request_spawn(&second, NOW).unwrap();
        match &refusal {
            SpawnSubmission::Terminal(SpawnCommandIssue::Failed { category, reason }) => {
                assert_eq!(category, "quota_exceeded");
                assert_eq!(reason, &quota_exceeded_detail(1));
                assert!(reason.contains("BRIDGET_FLEET_QUOTA"));
                assert!(reason.contains("quota de flotte atteint (1)"));
            }
            other => panic!("refus quota attendu, obtenu {other:?}"),
        }
        assert_eq!(supervisor.request_spawn(&second, NOW + 1).unwrap(), refusal);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolve_fleet_quota_lit_env_invalide_et_zero() {
        assert_eq!(resolve_fleet_quota(None), DEFAULT_FLEET_QUOTA);
        assert_eq!(resolve_fleet_quota(Some(OsString::from("24"))), 24);
        assert_eq!(resolve_fleet_quota(Some(OsString::from("0"))), 0);
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from("abc"))),
            DEFAULT_FLEET_QUOTA
        );
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from(""))),
            DEFAULT_FLEET_QUOTA
        );
        assert_eq!(
            resolve_fleet_quota(Some(OsString::from("-3"))),
            DEFAULT_FLEET_QUOTA
        );
        let root = test_root("quota-zero");
        fs::create_dir_all(&root).unwrap();
        let err = FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota: 0,
                ..config()
            },
        );
        assert!(matches!(
            err,
            Err(FleetError::InvalidOrder("configuration de flotte invalide"))
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn from_env_posee_releve_le_quota_consomme_par_le_daemon() {
        let _guard = TestFleetQuotaGuard::set(24);
        assert_eq!(FleetConfig::from_env().quota, 24);
    }

    #[test]
    fn message_refus_reprise_quota_nomme_agent_quota_et_levier() {
        let message = resume_quota_refusal_message("cursor8", 8);
        assert!(message.contains("cursor8"), "{message}");
        assert!(message.contains("quota de flotte atteint (8)"), "{message}");
        assert!(message.contains("BRIDGET_FLEET_QUOTA"), "{message}");
        assert!(message.contains("export BRIDGET_FLEET_QUOTA="), "{message}");
    }

    #[test]
    fn mismatch_expiration_et_register_tardif_sont_terminaux() {
        let root = test_root("guards");
        let supervisor = open(&root);
        let spawn = order("command-guard", Some(AGENT_GUARD), false);
        let lease = start(&supervisor, &spawn);
        let mut divergent = spawn.clone();
        divergent.cwd = PathBuf::from("/var/tmp");
        assert_eq!(
            supervisor.request_spawn(&divergent, NOW).unwrap(),
            SpawnSubmission::EnvelopeMismatch
        );
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        assert!(matches!(
            supervisor.register_connected(&lease, "instance-obsolete", NOW + 1),
            Err(FleetError::StaleGeneration)
        ));
        assert!(
            supervisor
                .expire(&lease.command_id, lease.generation, spawn.deadline_at)
                .unwrap()
        );
        assert!(matches!(
            supervisor.register_connected(&lease, &lease.instance_id, NOW + 61),
            Err(FleetError::StaleGeneration)
        ));
        assert!(matches!(
            supervisor.request_spawn(&spawn, NOW + 61).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Cancelled { .. })
        ));

        let expired = SpawnOrder {
            command_id: "command-expired".to_string(),
            issued_at: NOW - 400,
            deadline_at: NOW + 1,
            ..order("unused", Some(AGENT_EXPIRED), false)
        };
        assert_eq!(
            supervisor.request_spawn(&expired, NOW).unwrap(),
            SpawnSubmission::IdempotencyExpired
        );
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_invalide_un_lancement_avant_toute_io_et_rejoue_cancelled() {
        let root = test_root("stop-starting");
        let supervisor = open(&root);
        let spawn = order("command-stop-starting", Some(AGENT_STOP), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();

        supervisor.invalidate_for_stop(&lease).unwrap();

        assert!(matches!(
            supervisor.request_spawn(&spawn, NOW + 1).unwrap(),
            SpawnSubmission::Terminal(SpawnCommandIssue::Cancelled { ref reason })
                if reason == "arrêt demandé"
        ));
        assert_eq!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers[AGENT_STOP]
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_connecte_preserve_l_issue_spawn_et_marque_l_agent_arrete() {
        let root = test_root("stop-connected");
        let supervisor = open(&root);
        let spawn = order("command-stop-connected", Some(AGENT_STOP), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        let connected = supervisor
            .register_connected(&lease, &lease.instance_id, NOW + 1)
            .unwrap();

        supervisor.invalidate_for_stop(&lease).unwrap();

        assert_eq!(
            supervisor.request_spawn(&spawn, NOW + 2).unwrap(),
            SpawnSubmission::Terminal(connected)
        );
        assert_eq!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers[AGENT_STOP]
                .lifecycle_state,
            DesiredLifecycleState::Stopped
        );
        fs::remove_dir_all(root).unwrap();
    }

    struct CrashChild(Child);

    impl CrashChild {
        fn terminate(&mut self) {
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            let pid = self.0.id();
            let executable = std::env::current_exe().unwrap();
            let observed = Command::new("/bin/ps")
                .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
                .output()
                .unwrap();
            let command = String::from_utf8_lossy(&observed.stdout);
            let owned = command
                .split_whitespace()
                .next()
                .and_then(|value| value.parse::<u32>().ok())
                == Some(std::process::id())
                && command.contains(executable.to_string_lossy().as_ref())
                && !command.to_ascii_lowercase().contains("firefox")
                && unsafe { libc::getpgid(pid as i32) } == pid as i32;
            if matches!(self.0.try_wait(), Ok(Some(_))) {
                return;
            }
            assert!(
                owned,
                "le crash doit cibler notre enfant de test et son groupe dédié"
            );
            assert_eq!(unsafe { libc::kill(pid as i32, libc::SIGKILL) }, 0);
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            loop {
                if let Some(status) = self.0.try_wait().unwrap() {
                    assert!(!status.success());
                    return;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "enfant de crash non récolté"
                );
                thread::sleep(Duration::from_millis(5));
            }
        }
    }

    impl Drop for CrashChild {
        fn drop(&mut self) {
            // Une barrière EOF/timeout ne doit pas abandonner un enfant suspendu.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.terminate()));
        }
    }

    fn spawn_crash_child(root: &Path, stage: &str) -> (CrashChild, UnixStream) {
        let executable = std::env::current_exe().unwrap();
        assert!(
            executable
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("bridget_daemon"),
            "le sous-processus de crash doit être le binaire de test daemon, jamais Firefox"
        );
        let (parent_barrier, child_barrier) = UnixStream::pair().unwrap();
        parent_barrier
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut command = Command::new(executable);
        for directory in ["home", "state", "tmp"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        command
            .env_clear()
            .env("HOME", root.join("home"))
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            .env("TMPDIR", root.join("tmp"))
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .arg("fleet::tests::crash_child")
            .arg("--exact")
            .arg("--ignored")
            .arg("--nocapture")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env(CRASH_CHILD_ENV, "1")
            .env(CRASH_ROOT_ENV, root)
            .env(CRASH_STAGE_ENV, stage);
        unsafe {
            command.pre_exec(move || {
                if libc::setpgid(0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                if libc::dup2(child_barrier.as_raw_fd(), CRASH_BARRIER_FD) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        (CrashChild(command.spawn().unwrap()), parent_barrier)
    }

    fn wait_barrier(barrier: &mut UnixStream) {
        let mut byte = [0_u8; 1];
        barrier.read_exact(&mut byte).unwrap();
        assert_eq!(byte[0], b'B');
    }

    fn terminate_child(child: &mut CrashChild) {
        child.terminate();
    }

    fn signal_and_park() {
        let byte = b'B';
        assert_eq!(
            unsafe { libc::write(CRASH_BARRIER_FD, (&byte as *const u8).cast(), 1) },
            1
        );
        loop {
            thread::park();
        }
    }

    #[test]
    #[ignore = "sous-processus contrôlé par crash_reel_aux_frontieres_d503"]
    fn crash_child() {
        if std::env::var(CRASH_CHILD_ENV).as_deref() != Ok("1") {
            return;
        }
        let root = PathBuf::from(std::env::var_os(CRASH_ROOT_ENV).unwrap());
        let stage = std::env::var(CRASH_STAGE_ENV).unwrap();
        let supervisor = open(&root);
        let spawn = order("command-crash", Some(AGENT_CRASH), true);
        let lease = start(&supervisor, &spawn);
        supervisor
            .mark_starting(&lease, NOW, &resolved_test_definition())
            .unwrap();
        match stage.as_str() {
            "before_fleet" => signal_and_park(),
            "after_fleet" => {
                let _ = supervisor.register_connected_observed(
                    &lease,
                    &lease.instance_id,
                    NOW + 1,
                    || {
                        signal_and_park();
                        #[allow(unreachable_code)]
                        Ok(())
                    },
                );
                unreachable!();
            }
            "after_issue" => {
                supervisor
                    .register_connected(&lease, &lease.instance_id, NOW + 1)
                    .unwrap();
                signal_and_park();
            }
            _ => panic!("frontière de crash inconnue"),
        }
    }

    #[test]
    fn crash_reel_aux_frontieres_d503_rejoue_une_issue_unique() {
        for stage in ["before_fleet", "after_fleet", "after_issue"] {
            let root = test_root(stage);
            fs::create_dir_all(&root).unwrap();
            let (mut child, mut barrier) = spawn_crash_child(&root, stage);
            wait_barrier(&mut barrier);
            terminate_child(&mut child);

            let reopened = open(&root);
            let spawn = order("command-crash", Some(AGENT_CRASH), true);
            let outcome = reopened.request_spawn(&spawn, NOW + 2).unwrap();
            match stage {
                "before_fleet" => assert!(matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Failed { .. })
                )),
                "after_fleet" => {
                    let waiter = match outcome {
                        SpawnSubmission::Await(waiter) => waiter,
                        other => panic!("reprise non rattachée: {other:?}"),
                    };
                    let lease = SpawnLease {
                        command_id: waiter.command_id,
                        name: AGENT_CRASH.to_string(),
                        instance_id: waiter.instance_id,
                        generation: waiter.generation,
                        deadline_at: waiter.deadline_at,
                        persistent: true,
                        project: None,
                        link_id: None,
                        ownership: None,
                        agent_path: None,
                    };
                    let connected = reopened
                        .register_connected(&lease, &lease.instance_id, NOW + 3)
                        .unwrap();
                    assert!(matches!(connected, SpawnCommandIssue::Connected { .. }));
                }
                "after_issue" => assert!(matches!(
                    outcome,
                    SpawnSubmission::Terminal(SpawnCommandIssue::Connected { .. })
                )),
                _ => unreachable!(),
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}
