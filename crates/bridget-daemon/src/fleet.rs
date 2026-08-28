//! Orchestration transactionnelle des équipiers gérés par le daemon.
//!
//! La machine d'états vit derrière un verrou propre au superviseur. Le socle
//! 012 reste l'unique autorité pour réserver, comparer et rejouer une clé ;
//! `fleet.rs` ne fait aucun lookup idempotent parallèle.

use crate::desired_state::{DesiredEquipier, DesiredFleet, DesiredStateError, DesiredStateStore};
use crate::idempotency::{
    IdempotencyError, IdempotencyKey, IdempotencyStore, OperationKind, SpawnCommand,
    SpawnCommandIssue, SpawnCommandState, SpawnReservation,
};
use crate::recovery_trace::{
    NamedRosterEntry, NamedRosterStore, RecoveryLossEntry, persist_report, report_path,
    resolved_domain, roster_path,
};
use bridget_transport::ResolvedAgentDefinition;
use log::warn;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

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
    pub agent_type: String,
    pub requested_name: Option<String>,
    pub cwd: PathBuf,
    pub persistent: bool,
    pub command_id: String,
    pub issued_at: i64,
    pub deadline_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnLease {
    pub command_id: String,
    pub name: String,
    pub instance_id: String,
    pub generation: u64,
    pub deadline_at: i64,
    pub persistent: bool,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnSubmission {
    Start(SpawnLease),
    Await(SpawnWaiter),
    Terminal(SpawnCommandIssue),
    EnvelopeMismatch,
    IdempotencyExpired,
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
            | Self::DeadlineElapsed => None,
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
    agent_type: String,
    cwd: PathBuf,
    state: SpawnCommandState,
    resolved_definition: Option<ResolvedAgentDefinition>,
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
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.lease.name.cmp(&right.lease.name));
        candidates
    }

    pub fn desired_fleet(&self) -> Result<DesiredFleet, FleetError> {
        self.desired.load().map_err(Into::into)
    }

    pub fn remove_desired(&self, name: &str) -> Result<(), FleetError> {
        self.desired.remove(name)?;
        self.roster.forget(name);
        Ok(())
    }

    pub fn drain_non_persistent_named(&self) -> Vec<(String, NamedRosterEntry)> {
        self.roster.drain_non_persistent()
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
    /// consulte `drain_non_persistent_named` : l'annuaire publie donc
    /// exactement ce qui décidera du drain.
    pub fn named_persistence(&self) -> BTreeMap<String, bool> {
        self.roster.persistence_by_name()
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

    /// Réserve une clé puis applique, sous le même verrou métier, les gardes
    /// mutables de nom et de quota. Une clé rejouée ne traverse jamais ces
    /// gardes et ne peut donc pas lancer une seconde génération.
    pub fn request_spawn(
        &self,
        order: &SpawnOrder,
        now: i64,
    ) -> Result<SpawnSubmission, FleetError> {
        validate_order(order)?;
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
                if inner.active_by_command.len() >= self.config.quota {
                    let detail = quota_exceeded_detail(self.config.quota);
                    return terminal_refusal(&mut inner, &key, &command, "quota_exceeded", &detail);
                }
                inner.idempotency.advance_spawn(
                    &key,
                    command.generation,
                    SpawnCommandState::Requested,
                    SpawnCommandState::Reserved,
                )?;
                let active = ActiveSpawn {
                    command_id: command.command_id.clone(),
                    name: command.name.clone(),
                    instance_id: command.instance_id.clone().ok_or(
                        IdempotencyError::CorruptRecord("instance spawn en vol absente"),
                    )?,
                    generation: command.generation,
                    deadline_at: command.deadline_at,
                    persistent: command.persistent,
                    agent_type: order.agent_type.clone(),
                    cwd: order.cwd.clone(),
                    state: SpawnCommandState::Reserved,
                    resolved_definition: command.resolved_definition,
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
        if active.persistent {
            self.desired.upsert(
                active.name.clone(),
                DesiredEquipier {
                    agent_type: active.agent_type.clone(),
                    cwd: active.cwd.clone(),
                    command_id: active.command_id.clone(),
                    generation: active.generation,
                    created: now.to_string(),
                    resolved_definition: active.resolved_definition.clone(),
                    domain: resolved_domain(None, &active.cwd),
                },
            )?;
        }
        self.roster.remember(
            active.name.clone(),
            NamedRosterEntry {
                agent_type: active.agent_type.clone(),
                persistent: active.persistent,
                domain: resolved_domain(None, &active.cwd),
            },
        );
        after_fleet().map_err(FleetError::Observation)?;
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
    /// conserve son issue de spawn mais est retirée durablement de l'état
    /// désiré afin qu'aucun redémarrage ne puisse la ressusciter.
    pub fn invalidate_for_stop(&self, lease: &SpawnLease) -> Result<(), FleetError> {
        let mut inner = self
            .inner
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = inner.active_by_command.get(&lease.command_id).cloned() {
            if active.generation != lease.generation || active.name != lease.name {
                return Err(FleetError::StaleGeneration);
            }
            if active.persistent {
                self.desired.remove(&active.name)?;
            }
            self.roster.forget(&active.name);
            let issue = SpawnCommandIssue::Cancelled {
                reason: "arrêt demandé".to_string(),
            };
            let key = spawn_key(&inner, &active.command_id)?;
            inner
                .idempotency
                .finish_spawn(&key, active.generation, &issue)?;
            complete_locked(&mut inner, &active, issue);
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
        if lease.persistent {
            self.desired.remove(&lease.name)?;
        }
        self.roster.forget(&lease.name);
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
        if active.persistent {
            self.desired.remove(&active.name)?;
        }
        self.roster.forget(&active.name);
        let key = spawn_key(&inner, &active.command_id)?;
        inner
            .idempotency
            .finish_spawn(&key, active.generation, &issue)?;
        complete_locked(&mut inner, &active, issue.clone());
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
                    persistent: true,
                    agent_type: equipier.agent_type.clone(),
                    cwd: equipier.cwd.clone(),
                    state: SpawnCommandState::Starting,
                    resolved_definition: command.resolved_definition,
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

fn validate_order(order: &SpawnOrder) -> Result<(), FleetError> {
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
    if order
        .requested_name
        .as_ref()
        .is_some_and(|name| name.trim().is_empty())
    {
        return Err(FleetError::InvalidOrder("nom explicite vide"));
    }
    Ok(())
}

#[derive(Serialize)]
struct CanonicalSpawnOrder<'a> {
    command_id: &'a str,
    agent_type: &'a str,
    requested_name: &'a Option<String>,
    cwd: &'a str,
    persistent: bool,
    issued_at: i64,
    deadline_at: i64,
}

fn canonical_order(order: &SpawnOrder) -> Result<Vec<u8>, FleetError> {
    let cwd = order
        .cwd
        .to_str()
        .ok_or(FleetError::InvalidOrder("cwd non UTF-8"))?;
    serde_json::to_vec(&CanonicalSpawnOrder {
        command_id: &order.command_id,
        agent_type: &order.agent_type,
        requested_name: &order.requested_name,
        cwd,
        persistent: order.persistent,
        issued_at: order.issued_at,
        deadline_at: order.deadline_at,
    })
    .map_err(|_| FleetError::InvalidOrder("canon non sérialisable"))
}

fn resolve_name(order: &SpawnOrder, generation: u64, active: &HashMap<String, String>) -> String {
    if let Some(name) = &order.requested_name {
        return name.clone();
    }
    let base = format!("{}-{generation}", order.agent_type);
    if !active.contains_key(&base) {
        return base;
    }
    let mut suffix = generation.saturating_add(1);
    loop {
        let candidate = format!("{}-{suffix}", order.agent_type);
        if !active.contains_key(&candidate) {
            return candidate;
        }
        suffix = suffix.saturating_add(1);
    }
}

fn lease_from(active: &ActiveSpawn) -> SpawnLease {
    SpawnLease {
        command_id: active.command_id.clone(),
        name: active.name.clone(),
        instance_id: active.instance_id.clone(),
        generation: active.generation,
        deadline_at: active.deadline_at,
        persistent: active.persistent,
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

fn expire_locked(
    inner: &mut FleetInner,
    desired: &DesiredStateStore,
    roster: &NamedRosterStore,
    active: &ActiveSpawn,
) -> Result<(), FleetError> {
    if active.persistent {
        desired.remove(&active.name)?;
    }
    roster.forget(&active.name);
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
            agent_type: "codex".to_string(),
            requested_name: name.map(str::to_string),
            cwd: PathBuf::from("/tmp"),
            persistent,
            command_id: command_id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 60,
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
        let mut spawn = order("command-domain", Some("cursor6"), true);
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
            fleet.equipiers["cursor6"].domain.as_deref(),
            Some("bridget")
        );
        let raw = fs::read_to_string(root.join("fleet.json")).unwrap();
        assert!(raw.contains("\"domain\": \"bridget\""), "{raw}");
        supervisor
            .set_desired_domain("cursor6", Some("nouveau-projet"))
            .unwrap();
        assert_eq!(
            supervisor.desired_fleet().unwrap().equipiers["cursor6"]
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
        let spawn = order("command-scope", Some("codex-scope"), true);
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
    fn reprise_expose_les_generations_en_vol_dans_l_ordre_des_noms() {
        let root = test_root("recovery-candidates");
        let supervisor = open(&root);
        for (command_id, name) in [("command-z", "zeta"), ("command-a", "alpha")] {
            let spawn = order(command_id, Some(name), true);
            let lease = start(&supervisor, &spawn);
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
                        resolved_definition: Some(resolved_test_definition()),
                        domain: None,
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
            ["alpha", "zeta"]
        );
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.lease.persistent)
        );
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
                let order = order(command_id, Some("codex-unique"), false);
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
        let spawn = order("command-wait", Some("codex-wait"), true);
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
        let first = order("command-quota-a", Some("codex-a"), false);
        let second = order("command-quota-b", Some("codex-b"), false);
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
        let spawn = order("command-guard", Some("codex-guard"), false);
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
            ..order("unused", Some("codex-expired"), false)
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
        let spawn = order("command-stop-starting", Some("codex-stop"), true);
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
        assert!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_connecte_preserve_l_issue_spawn_mais_retire_l_etat_desire() {
        let root = test_root("stop-connected");
        let supervisor = open(&root);
        let spawn = order("command-stop-connected", Some("codex-stop"), true);
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
        assert!(
            DesiredStateStore::at_path(root.join("fleet.json"))
                .load()
                .unwrap()
                .equipiers
                .is_empty()
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn spawn_crash_child(root: &Path, stage: &str) -> (Child, UnixStream) {
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
        command
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
                if libc::dup2(child_barrier.as_raw_fd(), CRASH_BARRIER_FD) < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        (command.spawn().unwrap(), parent_barrier)
    }

    fn wait_barrier(barrier: &mut UnixStream) {
        let mut byte = [0_u8; 1];
        barrier.read_exact(&mut byte).unwrap();
        assert_eq!(byte[0], b'B');
    }

    fn terminate_child(child: &mut Child) {
        assert_eq!(
            unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) },
            0
        );
        assert!(!child.wait().unwrap().success());
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
        let spawn = order("command-crash", Some("codex-crash"), true);
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
            let spawn = order("command-crash", Some("codex-crash"), true);
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
                        name: "codex-crash".to_string(),
                        instance_id: waiter.instance_id,
                        generation: waiter.generation,
                        deadline_at: waiter.deadline_at,
                        persistent: true,
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
