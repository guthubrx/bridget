//! Orchestration transactionnelle des équipiers gérés par le daemon.
//!
//! La machine d'états vit derrière un verrou propre au superviseur. Le socle
//! 012 reste l'unique autorité pour réserver, comparer et rejouer une clé ;
//! `fleet.rs` ne fait aucun lookup idempotent parallèle.

use bridget_transport::poison::RecoverPoison;
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
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
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
    pub remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
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
    remote_placement: Option<bridget_transport::protocol::RemotePlacement>,
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
            .recover_poison();
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
            .recover_poison()
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
            .recover_poison();
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
            .recover_poison();
        inner.active_by_command.contains_key(command_id) || inner.completed.contains_key(command_id)
    }

    pub fn quota(&self) -> usize {
        self.config.quota
    }

    pub fn active_count(&self) -> usize {
        self.inner
            .lock()
            .recover_poison()
            .active_by_command
            .len()
    }

    /// Générations persistantes locales et réservations distantes à réconcilier.
    /// Une issue de lancement Connected ne prouve pas la fin du processus distant.
    pub fn recovery_candidates(&self) -> Vec<RecoveryCandidate> {
        let inner = self
            .inner
            .lock()
            .recover_poison();
        let mut candidates = inner
            .active_by_command
            .values()
            .filter(|active| active.persistent || active.remote_placement.is_some())
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

    #[cfg(test)]
    pub(crate) fn desired_write_count(&self) -> usize {
        self.desired.write_count()
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

    /// Décommissionne plusieurs entrées en une lecture et au plus une écriture de la flotte.
    /// Même effet que `decommission` pour chaque nom présent.
    pub fn decommission_all(&self, names: &[&str]) -> Result<usize, FleetError> {
        let present = self
            .desired
            .set_lifecycle_states(names, DesiredLifecycleState::Decommissioned)?;
        for name in &present {
            self.roster.forget(name);
        }
        Ok(present.len())
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
            .recover_poison()
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
                remote_placement: historical.remote_placement,
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
            .recover_poison();
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
                .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
            .recover_poison();
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
                    remote_placement: order.remote_placement.clone(),
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
            .recover_poison();
        let active = active_for_lease(&inner, lease)?.clone();
        if active
            .remote_placement
            .as_ref()
            .is_some_and(|placement| placement.definition_digest != definition.digest)
        {
            return Err(FleetError::InvalidOrder("définition distante divergente"));
        }
        if now >= active.deadline_at {
            if active.remote_placement.is_some() {
                return Err(FleetError::DeadlineElapsed);
            }
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
            .recover_poison();
        let active = active_for_lease(&inner, lease)?.clone();
        if instance_id != active.instance_id {
            return Err(FleetError::StaleGeneration);
        }
        if active.remote_placement.is_some() && active.state == SpawnCommandState::Connected {
            return inner
                .completed
                .get(&active.command_id)
                .cloned()
                .ok_or(FleetError::StaleGeneration);
        }
        if now >= active.deadline_at {
            if active.remote_placement.is_some() {
                return Err(FleetError::DeadlineElapsed);
            }
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
                remote_placement: active.remote_placement.clone(),
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
            .recover_poison();
        if let Some(active) = inner.active_by_command.get(&lease.command_id).cloned() {
            active_for_lease(&inner, lease)?;
            close_agent_link(&mut inner, &active, unix_now())?;
            self.persist_stopped_active(&active)?;
            if active.remote_placement.is_some() && active.state == SpawnCommandState::Connected {
                // La preuve terminale libère la réservation, sans réécrire l'issue du lancement.
                release_active_locked(&mut inner, &active);
                self.agent_link_changed.notify_all();
                self.terminal_changed.notify_all();
                return Ok(());
            }
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
                remote_placement: active.remote_placement.clone(),
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
            .recover_poison();
        let active = active_for_lease(&inner, lease)?.clone();
        if active.remote_placement.is_some() && active.state == SpawnCommandState::Connected {
            return Err(FleetError::InvalidTransition {
                command_id: active.command_id,
                from: active.state,
                expected: SpawnCommandState::Starting,
            });
        }
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
            .recover_poison();
        let Some(active) = inner.active_by_command.get(command_id).cloned() else {
            return Ok(false);
        };
        if active.generation != generation {
            return Err(FleetError::StaleGeneration);
        }
        // L'horloge du maître n'est pas une preuve d'arrêt sur la machine distante.
        if now < active.deadline_at || active.remote_placement.is_some() {
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
            .recover_poison();
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
            .recover_poison();
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
            if command.remote_placement.is_none() || command.state != SpawnCommandState::Connected {
                continue;
            }
        }
        if command.remote_placement.is_some() {
            let equipier = desired.equipiers.get(&command.name);
            // Une clôture durable ou une génération postérieure interdit une résurrection.
            if equipier.is_some_and(|entry| {
                entry.generation > command.generation
                    || (entry.command_id == command.command_id
                        && entry.generation == command.generation
                        && entry.lifecycle_state != DesiredLifecycleState::Running)
            }) {
                continue;
            }
            let key = spawn_key(inner, &command.command_id)?;
            let order = inner
                .idempotency
                .spawn_order(&key)?
                .ok_or(IdempotencyError::CorruptRecord("canon distant absent"))?;
            validate_order(&order, true)?;
            let instance_id = command
                .instance_id
                .clone()
                .ok_or(IdempotencyError::CorruptRecord("instance distante absente"))?;
            let link = inner.idempotency.agent_link_for_child(&instance_id)?;
            let mut ownership = order.ownership;
            if let (Some(owner), Some(link)) = (&mut ownership, &link) {
                owner.parent_instance_id = link.parent_instance_id.clone();
                owner.parent_execution_id = link.parent_execution_id.clone();
                owner.objective_id = link.objective_id.clone();
                owner.delegation_id = link.delegation_id.clone();
                owner.role = link.role.clone();
            }
            let active = ActiveSpawn {
                command_id: command.command_id.clone(),
                name: command.name.clone(),
                instance_id,
                generation: command.generation,
                deadline_at: command.deadline_at,
                persistent: order.persistent,
                project: order.project,
                agent_type: order.agent_type,
                cwd: order.cwd,
                state: command.state,
                resolved_definition: command.resolved_definition,
                runtime_execution: None,
                link_id: link.as_ref().map(|link| link.link_id.clone()),
                agent_path: link.as_ref().map(|link| link.agent_path.clone()),
                ownership,
                remote_placement: order.remote_placement,
            };
            if inner.active_by_name.contains_key(&active.name) {
                return Err(FleetError::InvalidOrder(
                    "réservations distantes de même nom",
                ));
            }
            inner
                .active_by_name
                .insert(active.name.clone(), active.command_id.clone());
            inner
                .active_by_command
                .insert(active.command_id.clone(), active);
            continue;
        }
        if let Some(equipier) = desired.equipiers.get(&command.name)
            && equipier.command_id == command.command_id
            && equipier.generation == command.generation
            && equipier.persistent
            && equipier.lifecycle_state == DesiredLifecycleState::Running
        {
            if equipier.remote_placement != command.remote_placement {
                return Err(FleetError::InvalidOrder("placement de reprise divergent"));
            }
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
                    remote_placement: equipier.remote_placement.clone(),
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
    if order.remote_placement.as_ref().is_some_and(|placement| {
        placement.validate().is_err() || order.cwd != Path::new(&placement.execution_cwd)
    }) {
        return Err(FleetError::InvalidOrder("placement distant invalide"));
    }
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

pub(crate) fn ownership_from_desired(equipier: &DesiredEquipier) -> Option<SpawnOwnership> {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    remote_placement: Option<&'a bridget_transport::protocol::RemotePlacement>,
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
        remote_placement: order.remote_placement.as_ref(),
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
        remote_placement: active.remote_placement.clone(),
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
    if active.generation != lease.generation
        || active.name != lease.name
        || active.remote_placement != lease.remote_placement
    {
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
    if active.remote_placement.is_some() && matches!(&issue, SpawnCommandIssue::Connected { .. }) {
        if let Some(retained) = inner.active_by_command.get_mut(&active.command_id) {
            retained.state = SpawnCommandState::Connected;
        }
    } else {
        release_active_locked(inner, active);
    }
    inner.completed.insert(active.command_id.clone(), issue);
}

fn release_active_locked(inner: &mut FleetInner, active: &ActiveSpawn) {
    inner.active_by_command.remove(&active.command_id);
    if inner
        .active_by_name
        .get(&active.name)
        .is_some_and(|command_id| command_id == &active.command_id)
    {
        inner.active_by_name.remove(&active.name);
    }
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
#[path = "fleet_tests.rs"]
mod tests;
