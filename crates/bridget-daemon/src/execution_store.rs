//! Persistance durable des soumissions et exécutions Bridget.

use bridget_transport::protocol::{
    ExecutionProviderContext, ProjectReference, UsageAggregate, UsageTokens,
};
use rusqlite::{
    Connection, OptionalExtension, Transaction, TransactionBehavior, params, types::Type,
};
use std::{collections::HashMap, path::Path};

pub struct ExecutionStore {
    conn: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionSnapshot {
    pub execution_id: String,
    /// Liaison figée à l'admission. `None` représente sans ambiguïté les
    /// exécutions historiques, sans tenter de la déduire du chemin ou du nom.
    pub project: Option<ProjectReference>,
    pub state: String,
    pub generation: u64,
    pub revision: u64,
    pub created_at: i64,
}

/// Cible persistée nécessaire à une commande de contrôle.
///
/// La résolution ne dépend jamais d'un nom fourni à nouveau par le client :
/// l'exécution admise reste la seule autorité de son destinataire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionControlTarget {
    pub snapshot: ExecutionSnapshot,
    pub target_agent: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionalTransition {
    Applied(ExecutionSnapshot),
    Rejected(ExecutionSnapshot),
    Missing,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructedExecution {
    pub parent_execution_id: String,
    pub snapshot: ExecutionSnapshot,
    pub message: bridget_core::BridgetMessage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionRecoveryOutcome {
    None,
    Reconstructed(Box<ReconstructedExecution>),
    PayloadUnavailable { parent_execution_id: String },
    Ambiguous { execution_ids: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedSubmission {
    pub submission_id: String,
    pub target_agent: String,
    pub priority: i64,
    pub enqueued_at: i64,
    pub message: Option<bridget_core::BridgetMessage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExecutionSummary {
    pub state: Option<String>,
    pub updated_at: Option<i64>,
    pub queue_depth: u64,
    pub continuation_mode: Option<String>,
}

/// Statut durable d'une commande de contrôle.
///
/// `Dispatched` est volontairement distinct de `Accepted` : il atteste une
/// écriture vers le wrapper, pas l'effet chez le fournisseur. Après une panne
/// à cette frontière, Bridget conserve une issue inconnue au lieu de rejouer
/// silencieusement une interruption ou une correction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlCommandStatus {
    Prepared,
    Dispatched,
    Accepted,
    Refused { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredControlCommand {
    pub execution_id: String,
    pub status: ControlCommandStatus,
    pub expires_at: i64,
}

/// Résultat fermé de l'enregistrement d'une référence fournisseur. L'absence
/// et l'incompatibilité ne sont jamais transformées en nouvelle exécution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderBindingOutcome {
    Applied,
    Missing,
    GenerationMismatch,
    Incompatible,
}

/// Mode de continuité publié par Bridget. Il est fermé afin qu'une nouvelle
/// stratégie ne puisse pas s'afficher comme une reprise native par défaut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationMode {
    Native,
    Forked,
    Reconstructed,
}

impl ContinuationMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Forked => "forked",
            Self::Reconstructed => "reconstructed",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "native" => Some(Self::Native),
            "forked" => Some(Self::Forked),
            "reconstructed" => Some(Self::Reconstructed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationRecord {
    pub execution_id: String,
    pub generation: u64,
    pub parent_execution_id: String,
    pub mode: ContinuationMode,
    pub reason: Option<String>,
    pub observed_at: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationOutcome {
    Applied,
    Missing,
    GenerationMismatch,
    ParentMissing,
    Incompatible,
}

/// Échantillon fournisseur attribué à une exécution et une génération exactes.
/// Les compteurs sont additionnés dans une seule ligne par source : la
/// cardinalité reste donc bornée par exécution et par fournisseur connu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionUsageSample {
    pub execution_id: String,
    pub generation: u64,
    pub tokens: UsageTokens,
    pub source: String,
    pub observed_at: i64,
}

/// Faits nécessaires à une politique d autonomie. Une consommation absente
/// reste `None` et ne peut jamais être interprétée comme zéro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionBudgetFacts {
    pub duration_secs: u64,
    pub descendants: u64,
    pub usage: Option<UsageAggregate>,
}

/// Résultat fermé de la réservation d une continuation. Il est séparé d une
/// transition runtime : réserver ne crée ni mission ni tour fournisseur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationReservation {
    Reserved,
    Replayed,
    AlreadyReserved,
    Missing,
    GenerationMismatch,
    RevisionMismatch,
    NotInactive,
    StaleProof,
    ConcurrentExecution,
}

/// Contexte attesté avant de réserver une continuation.
///
/// Une continuation ordinaire exige un parent déjà terminal. La reprise de
/// daemon est différente : le wrapper vient de se reconnecter et atteste qu'il
/// n'a plus de tour actif, tandis que SQLite porte encore son état historique.
/// Ce cas reste explicite pour ne pas assouplir la garde générale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContinuationReservationContext {
    InactiveParent,
    RecoveryAfterIdleWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlReservation {
    New,
    Replayed(StoredControlCommand),
    EnvelopeMismatch,
    Expired,
}

/// Priorité de file des soumissions issues d'un focus (SPEC-087).
pub const FOCUS_QUEUE_PRIORITY: i64 = 100;

impl ExecutionStore {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))?;
        Self::init_schema(&mut conn)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::init_schema(&mut conn)?;
        Ok(Self { conn })
    }

    /// Enregistre atomiquement l'acceptation durable et le démarrage d'une exécution.
    ///
    /// La reprise ne déduit jamais ce fait d'un événement fournisseur : elle relit
    /// exclusivement les états d'exécution que Bridget a lui-même persistés.
    pub fn record_starting(
        &self,
        submission_id: &str,
        execution_id: &str,
        target_agent: &str,
        observed_at: i64,
    ) -> rusqlite::Result<()> {
        self.record_starting_for_project(
            submission_id,
            execution_id,
            target_agent,
            None,
            observed_at,
        )
    }

    /// Variante d'admission qui reçoit la référence déjà validée par la
    /// frontière appelante. Elle ne la reconstruit jamais depuis le runtime.
    pub fn record_starting_for_project(
        &self,
        submission_id: &str,
        execution_id: &str,
        target_agent: &str,
        project: Option<&ProjectReference>,
        observed_at: i64,
    ) -> rusqlite::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO work_submissions (submission_id, message_id, target_agent, origin, intent, references_json, state, accepted_at) VALUES (?1, ?1, ?2, 'system', 'trigger_turn', '[]', 'accepted', ?3) ON CONFLICT(submission_id) DO NOTHING",
            params![submission_id, target_agent, observed_at],
        )?;
        tx.execute(
            "INSERT INTO executions (execution_id, submission_id, agent_instance_id, project_id, binding_generation, generation, state, revision, reason, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, ?4, 1, 'starting', 0, NULL, ?5, ?5) ON CONFLICT(execution_id) DO NOTHING",
            params![execution_id, submission_id, project.map(|value| &value.project_id), project.map(|value| value.binding_generation), observed_at],
        )?;
        tx.commit()
    }
    /// Admet puis démarre une remise sans l'inscrire dans la file.
    pub fn admit_starting(
        &self,
        submission_id: &str,
        execution_id: &str,
        target_agent: &str,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        self.admit_starting_for_project(
            submission_id,
            execution_id,
            target_agent,
            None,
            observed_at,
        )
    }

    /// Admet une remise et fixe sa référence projet optionnelle dans la même
    /// transaction que l'exécution, donc sans fenêtre de redéduction après un
    /// redémarrage.
    pub fn admit_starting_for_project(
        &self,
        submission_id: &str,
        execution_id: &str,
        target_agent: &str,
        project: Option<&ProjectReference>,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let inserted = tx.execute(
            "INSERT INTO work_submissions (submission_id, message_id, target_agent, origin, intent, references_json, state, accepted_at) VALUES (?1, ?1, ?2, 'system', 'trigger_turn', '[]', 'accepted', ?3) ON CONFLICT(submission_id) DO NOTHING",
            params![submission_id, target_agent, observed_at],
        )?;
        if inserted != 1 {
            tx.commit()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO executions (execution_id, submission_id, agent_instance_id, project_id, binding_generation, generation, state, revision, reason, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, ?4, 1, 'starting', 0, NULL, ?5, ?5)",
            params![execution_id, submission_id, project.map(|value| &value.project_id), project.map(|value| value.binding_generation), observed_at],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// Admet puis démarre une exécution en conservant l'enveloppe source.
    /// Après un crash, la réconciliation retrouve donc le même travail, pas
    /// une représentation incomplète du message initial.
    pub fn admit_starting_message(
        &self,
        message: &bridget_core::BridgetMessage,
        execution_id: &str,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        self.admit_starting_message_for_project(message, execution_id, None, observed_at)
    }

    /// Variante avec identité projet déjà portée par l'appel public. Le
    /// message conservé garde son contenu historique; seule l'exécution porte
    /// la référence structurée.
    pub fn admit_starting_message_for_project(
        &self,
        message: &bridget_core::BridgetMessage,
        execution_id: &str,
        project: Option<&ProjectReference>,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        let message_json = serde_json::to_string(message)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let references_json = serde_json::to_string(&message.references)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let origin = message.origin.map(|origin| match origin {
            bridget_core::MessageOrigin::Human => "human",
            bridget_core::MessageOrigin::Agent => "agent",
            bridget_core::MessageOrigin::Routine => "routine",
            bridget_core::MessageOrigin::System => "system",
        });
        let intent = message.intent.map(|intent| match intent {
            bridget_core::MessageIntent::QueueOnly => "queue_only",
            bridget_core::MessageIntent::TriggerTurn => "trigger_turn",
            bridget_core::MessageIntent::SteerCurrent => "steer_current",
            bridget_core::MessageIntent::InterruptAndStart => "interrupt_and_start",
            bridget_core::MessageIntent::ControlOnly => "control_only",
        });
        let tx = self.conn.unchecked_transaction()?;
        let inserted = tx.execute(
            "INSERT INTO work_submissions (submission_id, message_id, target_agent, origin, intent, references_json, message_json, state, accepted_at) VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6, 'accepted', ?7) ON CONFLICT(submission_id) DO NOTHING",
            params![message.id, message.to, origin, intent, references_json, message_json, observed_at],
        )?;
        if inserted != 1 {
            tx.commit()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO executions (execution_id, submission_id, agent_instance_id, project_id, binding_generation, generation, state, revision, reason, created_at, updated_at) VALUES (?1, ?2, NULL, ?3, ?4, 1, 'starting', 0, NULL, ?5, ?5)",
            params![execution_id, message.id, project.map(|value| &value.project_id), project.map(|value| value.binding_generation), observed_at],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// Admet une soumission une seule fois puis l'insère dans la file indexée.
    /// Cette forme historique ne porte pas le corps du message et reste lue
    /// pour compatibilité. Les nouvelles soumissions utilisent la variante
    /// qui persiste l'enveloppe complète ci-dessous.
    pub fn admit_submission(
        &self,
        submission_id: &str,
        target_agent: &str,
        priority: i64,
        enqueued_at: i64,
    ) -> rusqlite::Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let inserted = tx.execute(
            "INSERT INTO work_submissions (submission_id, message_id, target_agent, origin, intent, references_json, state, accepted_at) VALUES (?1, ?1, ?2, 'system', 'queue_only', '[]', 'accepted', ?3) ON CONFLICT(submission_id) DO NOTHING",
            params![submission_id, target_agent, enqueued_at],
        )?;
        if inserted != 1 {
            tx.commit()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO execution_queue (submission_id, target_agent, priority, enqueued_at) VALUES (?1, ?2, ?3, ?4)",
            params![submission_id, target_agent, priority, enqueued_at],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// Conserve le message exact d'une intention `QueueOnly` afin qu'un
    /// déclenchement ultérieur après redémarrage ne reconstruise jamais un
    /// prompt à partir d'un résumé ou d'une heuristique.
    /// SPEC-087 : une remise issue d'un focus du référent passe avant le
    /// travail ordinaire dans la file d'exécution. Le fait vient des
    /// références du message (`focus:<objective_id>`), posées par le service compagnon.
    pub fn queue_priority_for(message: &bridget_core::BridgetMessage, requested: i64) -> i64 {
        if message
            .references
            .iter()
            .any(|reference| reference.starts_with("focus:"))
        {
            requested.max(FOCUS_QUEUE_PRIORITY)
        } else {
            requested
        }
    }

    pub fn admit_message_submission(
        &self,
        message: &bridget_core::BridgetMessage,
        priority: i64,
        enqueued_at: i64,
    ) -> rusqlite::Result<bool> {
        let priority = Self::queue_priority_for(message, priority);
        let message_json = serde_json::to_string(message)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let references_json = serde_json::to_string(&message.references)
            .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?;
        let origin = message.origin.map(|origin| match origin {
            bridget_core::MessageOrigin::Human => "human",
            bridget_core::MessageOrigin::Agent => "agent",
            bridget_core::MessageOrigin::Routine => "routine",
            bridget_core::MessageOrigin::System => "system",
        });
        let intent = message.intent.map(|intent| match intent {
            bridget_core::MessageIntent::QueueOnly => "queue_only",
            bridget_core::MessageIntent::TriggerTurn => "trigger_turn",
            bridget_core::MessageIntent::SteerCurrent => "steer_current",
            bridget_core::MessageIntent::InterruptAndStart => "interrupt_and_start",
            bridget_core::MessageIntent::ControlOnly => "control_only",
        });
        let tx = self.conn.unchecked_transaction()?;
        let inserted = tx.execute(
            "INSERT INTO work_submissions (submission_id, message_id, target_agent, origin, intent, references_json, message_json, state, accepted_at) VALUES (?1, ?1, ?2, ?3, ?4, ?5, ?6, 'accepted', ?7) ON CONFLICT(submission_id) DO NOTHING",
            params![message.id, message.to, origin, intent, references_json, message_json, enqueued_at],
        )?;
        if inserted != 1 {
            tx.commit()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO execution_queue (submission_id, target_agent, priority, enqueued_at) VALUES (?1, ?2, ?3, ?4)",
            params![message.id, message.to, priority, enqueued_at],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// Retire atomiquement le prochain travail admissible de la file indexée.
    pub fn take_next_submission(
        &self,
        target_agent: &str,
    ) -> rusqlite::Result<Option<QueuedSubmission>> {
        let tx = self.conn.unchecked_transaction()?;
        let next = tx
            .query_row(
                "SELECT queue.submission_id, queue.target_agent, queue.priority, queue.enqueued_at, submission.message_json FROM execution_queue queue JOIN work_submissions submission ON submission.submission_id = queue.submission_id WHERE queue.target_agent = ?1 ORDER BY queue.priority DESC, queue.enqueued_at, queue.submission_id LIMIT 1",
                [target_agent],
                |row| {
                    let message_json: Option<String> = row.get(4)?;
                    let message = message_json
                        .map(|value| {
                            serde_json::from_str(&value).map_err(|error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    4,
                                    Type::Text,
                                    Box::new(error),
                                )
                            })
                        })
                        .transpose()?;
                    Ok(QueuedSubmission {
                        submission_id: row.get(0)?,
                        target_agent: row.get(1)?,
                        priority: row.get(2)?,
                        enqueued_at: row.get(3)?,
                        message,
                    })
                },
            )
            .optional()?;
        let Some(next) = next else {
            tx.commit()?;
            return Ok(None);
        };
        let deleted = tx.execute(
            "DELETE FROM execution_queue WHERE submission_id = ?1",
            [&next.submission_id],
        )?;
        if deleted != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        tx.commit()?;
        Ok(Some(next))
    }

    /// Projette file, exécution active et mode de continuité sans lire le corps des messages.
    pub fn agent_execution_summaries(
        &self,
    ) -> rusqlite::Result<HashMap<String, AgentExecutionSummary>> {
        let mut summaries = HashMap::new();
        let mut queue = self
            .conn
            .prepare("SELECT target_agent, COUNT(*) FROM execution_queue GROUP BY target_agent")?;
        for row in queue.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })? {
            let (target_agent, queue_depth) = row?;
            summaries.insert(
                target_agent,
                AgentExecutionSummary {
                    state: None,
                    updated_at: None,
                    queue_depth,
                    continuation_mode: None,
                },
            );
        }
        let mut active = self.conn.prepare(
            "SELECT submission.target_agent, execution.state, execution.updated_at, continuation.mode \
             FROM executions execution \
             JOIN work_submissions submission ON submission.submission_id = execution.submission_id \
             LEFT JOIN execution_continuations continuation ON continuation.execution_id = execution.execution_id \
             WHERE execution.state IN ('starting', 'running', 'waiting_approval', 'waiting_user_input', 'interrupting') \
             ORDER BY execution.updated_at DESC, execution.execution_id DESC",
        )?;
        for row in active.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })? {
            let (target_agent, state, updated_at, continuation_mode) = row?;
            let entry = summaries
                .entry(target_agent)
                .or_insert(AgentExecutionSummary {
                    state: None,
                    updated_at: None,
                    queue_depth: 0,
                    continuation_mode: None,
                });

            if entry.state.is_none() {
                entry.state = Some(state);
                entry.updated_at = Some(updated_at);
                entry.continuation_mode = continuation_mode;
            }
        }
        Ok(summaries)
    }
    /// Atteste la référence fournisseur d'une exécution sans autoriser une
    /// réécriture de génération, de thread ou de tour déjà connus.
    pub fn record_provider_context(
        &self,
        context: &ExecutionProviderContext,
    ) -> rusqlite::Result<ProviderBindingOutcome> {
        let tx = self.conn.unchecked_transaction()?;
        let generation = tx
            .query_row(
                "SELECT generation FROM executions WHERE execution_id = ?1",
                [&context.execution_id],
                |row| row.get::<_, u64>(0),
            )
            .optional()?;
        let Some(generation) = generation else {
            tx.commit()?;
            return Ok(ProviderBindingOutcome::Missing);
        };
        if generation != context.generation {
            tx.commit()?;
            return Ok(ProviderBindingOutcome::GenerationMismatch);
        }
        let existing = tx
            .query_row(
                "SELECT provider_kind, execution_path, binary_path, binary_version, binary_digest, contract_version, provider_session_id, provider_thread_id, active_turn_id, capabilities_revision FROM provider_bindings WHERE execution_id = ?1",
                [&context.execution_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                        row.get::<_, Option<String>>(8)?,
                        row.get::<_, Option<String>>(9)?,
                    ))
                },
            )
            .optional()?;
        let compatible = |known: &Option<String>, observed: &str| {
            known.as_ref().is_none_or(|known| known == observed)
        };
        let compatible_option = |known: &Option<String>, observed: &Option<String>| {
            known
                .as_ref()
                .zip(observed.as_ref())
                .is_none_or(|(known, observed)| known == observed)
        };
        if let Some((
            provider_kind,
            execution_path,
            binary_path,
            binary_version,
            binary_digest,
            contract_version,
            provider_session_id,
            provider_thread_id,
            provider_turn_id,
            capabilities_revision,
        )) = existing
        {
            let compatible = provider_kind == context.provider_kind
                && execution_path == context.execution_path
                && compatible(&binary_path, &context.observation.binary_path)
                && compatible(&binary_version, &context.observation.binary_version)
                && compatible(&binary_digest, &context.observation.binary_digest)
                && compatible(&contract_version, &context.observation.contract_version)
                && compatible(
                    &capabilities_revision,
                    &context.observation.contract_version,
                )
                && compatible_option(&provider_session_id, &context.provider_session_id)
                && compatible_option(&provider_thread_id, &context.provider_thread_id)
                && compatible_option(&provider_turn_id, &context.provider_turn_id);
            if !compatible {
                tx.commit()?;
                return Ok(ProviderBindingOutcome::Incompatible);
            }
            tx.execute(
                "UPDATE provider_bindings SET binary_path = COALESCE(binary_path, ?2), binary_version = COALESCE(binary_version, ?3), binary_digest = COALESCE(binary_digest, ?4), contract_version = COALESCE(contract_version, ?5), provider_session_id = COALESCE(provider_session_id, ?6), provider_thread_id = COALESCE(provider_thread_id, ?7), active_turn_id = COALESCE(active_turn_id, ?8), capabilities_revision = COALESCE(capabilities_revision, ?9), observed_at = MAX(observed_at, ?10) WHERE execution_id = ?1",
                params![
                    context.execution_id,
                    context.observation.binary_path,
                    context.observation.binary_version,
                    context.observation.binary_digest,
                    context.observation.contract_version,
                    context.provider_session_id,
                    context.provider_thread_id,
                    context.provider_turn_id,
                    context.observation.contract_version,
                    context.observed_at,
                ],
            )?;
        } else {
            tx.execute(
                "INSERT INTO provider_bindings (execution_id, provider_kind, execution_path, binary_path, binary_version, binary_digest, contract_version, provider_session_id, provider_thread_id, active_turn_id, capabilities_revision, observed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    context.execution_id,
                    context.provider_kind,
                    context.execution_path,
                    context.observation.binary_path,
                    context.observation.binary_version,
                    context.observation.binary_digest,
                    context.observation.contract_version,
                    context.provider_session_id,
                    context.provider_thread_id,
                    context.provider_turn_id,
                    context.observation.contract_version,
                    context.observed_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(ProviderBindingOutcome::Applied)
    }
    /// Conserve une ascendance d'exécution immuable. Une reprise ne modifie
    /// jamais le parent: elle crée ou réatteste seulement le fait du descendant.
    pub fn record_continuation(
        &self,
        execution_id: &str,
        generation: u64,
        parent_execution_id: &str,
        mode: ContinuationMode,
        reason: Option<&str>,
        observed_at: i64,
    ) -> rusqlite::Result<ContinuationOutcome> {
        let tx = self.conn.unchecked_transaction()?;
        let child_generation = tx
            .query_row(
                "SELECT generation FROM executions WHERE execution_id = ?1",
                [execution_id],
                |row| row.get::<_, u64>(0),
            )
            .optional()?;
        let Some(child_generation) = child_generation else {
            tx.commit()?;
            return Ok(ContinuationOutcome::Missing);
        };
        if child_generation != generation {
            tx.commit()?;
            return Ok(ContinuationOutcome::GenerationMismatch);
        }
        if execution_id == parent_execution_id
            || !tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM executions WHERE execution_id = ?1)",
                [parent_execution_id],
                |row| row.get::<_, bool>(0),
            )?
        {
            tx.commit()?;
            return Ok(ContinuationOutcome::ParentMissing);
        }
        let existing = tx.query_row(
            "SELECT generation, parent_execution_id, mode, reason FROM execution_continuations WHERE execution_id = ?1",
            [execution_id],
            |row| Ok((row.get::<_, u64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, Option<String>>(3)?)),
        ).optional()?;
        if let Some((known_generation, known_parent, known_mode, known_reason)) = existing {
            let compatible = known_generation == generation
                && known_parent == parent_execution_id
                && known_mode == mode.as_str()
                && known_reason.as_deref() == reason;
            if !compatible {
                tx.commit()?;
                return Ok(ContinuationOutcome::Incompatible);
            }
            tx.execute(
                "UPDATE execution_continuations SET observed_at = MAX(observed_at, ?2) WHERE execution_id = ?1",
                params![execution_id, observed_at],
            )?;
        } else {
            tx.execute(
                "INSERT INTO execution_continuations (execution_id, generation, parent_execution_id, mode, reason, observed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![execution_id, generation, parent_execution_id, mode.as_str(), reason, observed_at],
            )?;
        }
        tx.commit()?;
        Ok(ContinuationOutcome::Applied)
    }

    pub fn continuation_for(
        &self,
        execution_id: &str,
    ) -> rusqlite::Result<Option<ContinuationRecord>> {
        self.conn.query_row(
            "SELECT execution_id, generation, parent_execution_id, mode, reason, observed_at FROM execution_continuations WHERE execution_id = ?1",
            [execution_id],
            |row| {
                let mode: String = row.get(3)?;
                let mode = ContinuationMode::parse(&mode).ok_or_else(|| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, "mode de continuité inconnu".into()))?;
                Ok(ContinuationRecord {
                    execution_id: row.get(0)?,
                    generation: row.get(1)?,
                    parent_execution_id: row.get(2)?,
                    mode,
                    reason: row.get(4)?,
                    observed_at: row.get(5)?,
                })
            },
        ).optional()
    }

    /// Agrège un échantillon corrélé dans une ligne par source. Les échantillons
    /// sans exécution sont volontairement exclus de cette table de budget.
    pub fn record_execution_usage(&self, sample: &ExecutionUsageSample) -> rusqlite::Result<bool> {
        if sample.execution_id.trim().is_empty()
            || sample.generation == 0
            || sample.source.trim().is_empty()
            || sample.source.len() > 64
            || sample.observed_at <= 0
        {
            return Ok(false);
        }
        let input = sql_counter(sample.tokens.input_tokens)?;
        let output = sql_counter(sample.tokens.output_tokens)?;
        let cache_creation = sql_counter(sample.tokens.cache_creation_input_tokens)?;
        let cache_read = sql_counter(sample.tokens.cache_read_input_tokens)?;
        let tx = self.conn.unchecked_transaction()?;
        let generation = tx
            .query_row(
                "SELECT generation FROM executions WHERE execution_id = ?1",
                [&sample.execution_id],
                |row| row.get::<_, u64>(0),
            )
            .optional()?;
        if generation != Some(sample.generation) {
            tx.commit()?;
            return Ok(false);
        }
        tx.execute(
            "INSERT INTO execution_usage_totals (execution_id, generation, source, turns, input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens, observed_at) VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6, ?7, ?8) ON CONFLICT(execution_id, generation, source) DO UPDATE SET turns = MIN(9223372036854775807, execution_usage_totals.turns + 1), input_tokens = MIN(9223372036854775807, execution_usage_totals.input_tokens + excluded.input_tokens), output_tokens = MIN(9223372036854775807, execution_usage_totals.output_tokens + excluded.output_tokens), cache_creation_input_tokens = MIN(9223372036854775807, execution_usage_totals.cache_creation_input_tokens + excluded.cache_creation_input_tokens), cache_read_input_tokens = MIN(9223372036854775807, execution_usage_totals.cache_read_input_tokens + excluded.cache_read_input_tokens), observed_at = MAX(execution_usage_totals.observed_at, excluded.observed_at)",
            params![sample.execution_id, sample.generation, sample.source, input, output, cache_creation, cache_read, sample.observed_at],
        )?;
        tx.commit()?;
        Ok(true)
    }

    /// Agrège durée, usage et ascendance de continuité sans lire de contenu.
    /// Une arborescence sans échantillon retourne `usage: None`, jamais zéro.
    pub fn execution_budget_facts(
        &self,
        execution_id: &str,
        observed_at: i64,
    ) -> rusqlite::Result<Option<ExecutionBudgetFacts>> {
        let created_at: Option<i64> = self
            .conn
            .query_row(
                "SELECT created_at FROM executions WHERE execution_id = ?1",
                [execution_id],
                |row| row.get(0),
            )
            .optional()?;
        let Some(created_at) = created_at else {
            return Ok(None);
        };
        let lineage = "WITH RECURSIVE lineage(execution_id) AS (SELECT ?1 UNION SELECT child.execution_id FROM execution_continuations child JOIN lineage parent ON child.parent_execution_id = parent.execution_id) ";
        let descendants: u64 = self.conn.query_row(
            &(lineage.to_string() + "SELECT COUNT(*) - 1 FROM lineage"),
            [execution_id],
            |row| row.get(0),
        )?;
        let aggregate = self.conn.query_row(
            &(lineage.to_string() + "SELECT SUM(turns), SUM(input_tokens), SUM(output_tokens), SUM(cache_creation_input_tokens), SUM(cache_read_input_tokens) FROM execution_usage_totals WHERE execution_id IN (SELECT execution_id FROM lineage)"),
            [execution_id],
            |row| {
                Ok((
                    row.get::<_, Option<i64>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            },
        )?;
        let usage = match aggregate {
            (Some(turns), Some(input), Some(output), Some(cache_creation), Some(cache_read)) => {
                let turns = u64::try_from(turns).unwrap_or(u64::MAX);
                let input_tokens = u64::try_from(input).unwrap_or(u64::MAX);
                let output_tokens = u64::try_from(output).unwrap_or(u64::MAX);
                let cache_creation_input_tokens = u64::try_from(cache_creation).unwrap_or(u64::MAX);
                let cache_read_input_tokens = u64::try_from(cache_read).unwrap_or(u64::MAX);
                Some(UsageAggregate {
                    turns,
                    input_tokens,
                    output_tokens,
                    cache_creation_input_tokens,
                    cache_read_input_tokens,
                    facturable_tokens: input_tokens
                        .saturating_add(output_tokens)
                        .saturating_add(cache_creation_input_tokens),
                })
            }
            _ => None,
        };
        Ok(Some(ExecutionBudgetFacts {
            duration_secs: observed_at.saturating_sub(created_at).max(0) as u64,
            descendants,
            usage,
        }))
    }

    /// Réserve une seule continuation seulement si le parent est inactif et
    /// qu aucune autre exécution active ne vise le même agent. La transaction
    /// immédiate rend deux tentatives concurrentes déterministes.
    #[allow(clippy::too_many_arguments)]
    pub fn reserve_continuation_if_idle(
        &self,
        parent_execution_id: &str,
        expected_generation: u64,
        expected_revision: u64,
        continuation_id: &str,
        proof_idle_at: i64,
        observed_at: i64,
    ) -> rusqlite::Result<ContinuationReservation> {
        self.reserve_continuation_with_context(
            parent_execution_id,
            expected_generation,
            expected_revision,
            continuation_id,
            proof_idle_at,
            observed_at,
            ContinuationReservationContext::InactiveParent,
        )
    }

    /// Réserve une reprise après l'attestation d'un wrapper revenu sans tour
    /// actif. Ce contexte n'est construit que dans le chemin de reconnexion du
    /// daemon, jamais depuis une requête cliente.
    #[allow(clippy::too_many_arguments)]
    pub fn reserve_recovery_continuation_after_idle_wrapper(
        &self,
        parent_execution_id: &str,
        expected_generation: u64,
        expected_revision: u64,
        continuation_id: &str,
        proof_idle_at: i64,
        observed_at: i64,
    ) -> rusqlite::Result<ContinuationReservation> {
        self.reserve_continuation_with_context(
            parent_execution_id,
            expected_generation,
            expected_revision,
            continuation_id,
            proof_idle_at,
            observed_at,
            ContinuationReservationContext::RecoveryAfterIdleWrapper,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn reserve_continuation_with_context(
        &self,
        parent_execution_id: &str,
        expected_generation: u64,
        expected_revision: u64,
        continuation_id: &str,
        proof_idle_at: i64,
        observed_at: i64,
        context: ContinuationReservationContext,
    ) -> rusqlite::Result<ContinuationReservation> {
        if continuation_id.trim().is_empty() || proof_idle_at <= 0 || observed_at < proof_idle_at {
            return Ok(ContinuationReservation::StaleProof);
        }
        let tx = self.conn.unchecked_transaction()?;
        let parent = tx.query_row(
            "SELECT execution.state, execution.generation, execution.revision, execution.updated_at, submission.target_agent FROM executions execution JOIN work_submissions submission ON submission.submission_id = execution.submission_id WHERE execution.execution_id = ?1",
            [parent_execution_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?, row.get::<_, u64>(2)?, row.get::<_, i64>(3)?, row.get::<_, String>(4)?)),
        ).optional()?;
        let Some((state, generation, revision, updated_at, target_agent)) = parent else {
            tx.commit()?;
            return Ok(ContinuationReservation::Missing);
        };
        if generation != expected_generation {
            tx.commit()?;
            return Ok(ContinuationReservation::GenerationMismatch);
        }
        if revision != expected_revision {
            tx.commit()?;
            return Ok(ContinuationReservation::RevisionMismatch);
        }
        let parent_is_inactive = matches!(
            state.as_str(),
            "interrupted" | "completed" | "failed" | "unreachable" | "paused" | "blocked"
        );
        let parent_is_recoverable = matches!(
            state.as_str(),
            "queued"
                | "starting"
                | "running"
                | "waiting_approval"
                | "waiting_user_input"
                | "interrupting"
        );
        if !(parent_is_inactive
            || context == ContinuationReservationContext::RecoveryAfterIdleWrapper
                && parent_is_recoverable)
        {
            tx.commit()?;
            return Ok(ContinuationReservation::NotInactive);
        }
        if proof_idle_at < updated_at {
            tx.commit()?;
            return Ok(ContinuationReservation::StaleProof);
        }
        let existing: Option<String> = tx.query_row(
            "SELECT continuation_id FROM execution_continuation_reservations WHERE parent_execution_id = ?1",
            [parent_execution_id],
            |row| row.get(0),
        ).optional()?;
        if let Some(existing) = existing {
            tx.commit()?;
            return Ok(if existing == continuation_id {
                ContinuationReservation::Replayed
            } else {
                ContinuationReservation::AlreadyReserved
            });
        }
        let concurrent: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM executions execution JOIN work_submissions submission ON submission.submission_id = execution.submission_id WHERE submission.target_agent = ?1 AND execution.execution_id != ?2 AND execution.state IN (\"starting\", \"running\", \"waiting_approval\", \"waiting_user_input\", \"interrupting\"))",
            params![target_agent, parent_execution_id],
            |row| row.get(0),
        )?;
        if concurrent {
            tx.commit()?;
            return Ok(ContinuationReservation::ConcurrentExecution);
        }
        tx.execute(
            "INSERT INTO execution_continuation_reservations (parent_execution_id, continuation_id, generation, revision, proof_idle_at, reserved_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![parent_execution_id, continuation_id, expected_generation, expected_revision, proof_idle_at, observed_at],
        )?;
        tx.commit()?;
        Ok(ContinuationReservation::Reserved)
    }
    /// Lit l'état persistant sans l'inférer du runtime fournisseur.
    pub fn execution_snapshot(
        &self,
        execution_id: &str,
    ) -> rusqlite::Result<Option<ExecutionSnapshot>> {
        self.conn
            .query_row(
                "SELECT execution_id, project_id, binding_generation, state, generation, revision, created_at FROM executions WHERE execution_id = ?1",
                [execution_id],
                |row| {
                    Ok(ExecutionSnapshot {
                        execution_id: row.get(0)?,
                        project: project_reference_from_parts(row.get(1)?, row.get(2)?)?,
                        state: row.get(3)?,
                        generation: row.get(4)?,
                        revision: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .optional()
    }

    /// Retrouve l'enveloppe d'une exécution pour la reprise et la
    /// réconciliation, sans jamais en déduire une transition métier.
    pub fn execution_message(
        &self,
        execution_id: &str,
    ) -> rusqlite::Result<Option<bridget_core::BridgetMessage>> {
        let message_json: Option<String> = self
            .conn
            .query_row(
                "SELECT submission.message_json FROM executions execution JOIN work_submissions submission ON submission.submission_id = execution.submission_id WHERE execution.execution_id = ?1",
                [execution_id],
                |row| row.get(0),
            )
            .optional()?;
        message_json
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
                })
            })
            .transpose()
    }

    /// Lit la cible durable d'une exécution, pour router un contrôle vers son
    /// wrapper sans croire un destinataire déclaré par le client.
    pub fn execution_control_target(
        &self,
        execution_id: &str,
    ) -> rusqlite::Result<Option<ExecutionControlTarget>> {
        self.conn
            .query_row(
                "SELECT execution.execution_id, execution.project_id, execution.binding_generation, execution.state, execution.generation, execution.revision, execution.created_at, submission.target_agent FROM executions execution JOIN work_submissions submission ON submission.submission_id = execution.submission_id WHERE execution.execution_id = ?1",
                [execution_id],
                |row| {
                    Ok(ExecutionControlTarget {
                        snapshot: ExecutionSnapshot {
                            execution_id: row.get(0)?,
                            project: project_reference_from_parts(row.get(1)?, row.get(2)?)?,
                            state: row.get(3)?,
                            generation: row.get(4)?,
                            revision: row.get(5)?,
                            created_at: row.get(6)?,
                        },
                        target_agent: row.get(7)?,
                    })
                },
            )
            .optional()
    }

    /// Réserve une commande sous ses octets canoniques. Un rejeu aux mêmes
    /// octets ne redéclenche jamais une action locale pendant qu'une issue est
    /// encore inconnue.
    pub fn reserve_control_command(
        &self,
        issuer_scope: &str,
        command_id: &str,
        execution_id: &str,
        canonical_bytes: &[u8],
        observed_at: i64,
        expires_at: i64,
    ) -> rusqlite::Result<ControlReservation> {
        let tx = self.conn.unchecked_transaction()?;
        let existing = tx
            .query_row(
                "SELECT canonical_bytes, execution_id, state, refusal_reason, expires_at FROM execution_control_commands WHERE issuer_scope = ?1 AND command_id = ?2",
                params![issuer_scope, command_id],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, i64>(4)?,
                    ))
                },
            )
            .optional()?;
        let reservation = match existing {
            Some((canonical, execution_id, state, refusal_reason, expires_at))
                if expires_at <= observed_at =>
            {
                ControlReservation::Expired
            }
            Some((canonical, _, _, _, _)) if canonical != canonical_bytes => {
                ControlReservation::EnvelopeMismatch
            }
            Some((_, execution_id, state, refusal_reason, expires_at)) => {
                let status = match state.as_str() {
                    "prepared" => ControlCommandStatus::Prepared,
                    "dispatched" => ControlCommandStatus::Dispatched,
                    "accepted" => ControlCommandStatus::Accepted,
                    "refused" => ControlCommandStatus::Refused {
                        reason: refusal_reason.unwrap_or_else(|| "refus_sans_raison".to_string()),
                    },
                    _ => ControlCommandStatus::Refused {
                        reason: "etat_controle_inconnu".to_string(),
                    },
                };
                ControlReservation::Replayed(StoredControlCommand {
                    execution_id,
                    status,
                    expires_at,
                })
            }
            None => {
                tx.execute(
                    "INSERT INTO execution_control_commands (issuer_scope, command_id, canonical_bytes, execution_id, state, refusal_reason, created_at, updated_at, expires_at) VALUES (?1, ?2, ?3, ?4, 'prepared', NULL, ?5, ?5, ?6)",
                    params![issuer_scope, command_id, canonical_bytes, execution_id, observed_at, expires_at],
                )?;
                ControlReservation::New
            }
        };
        tx.commit()?;
        Ok(reservation)
    }

    /// Marque l'ordre comme écrit vers le wrapper avant l'E/S. Une panne après
    /// cette écriture reste donc `dispatched` et devient inconnue, jamais un
    /// second contrôle invisible au rejeu.
    pub fn mark_control_dispatched(
        &self,
        issuer_scope: &str,
        command_id: &str,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        Ok(self.conn.execute(
            "UPDATE execution_control_commands SET state = 'dispatched', updated_at = ?3 WHERE issuer_scope = ?1 AND command_id = ?2 AND state = 'prepared'",
            params![issuer_scope, command_id, observed_at],
        )? == 1)
    }

    pub fn refuse_control_command(
        &self,
        issuer_scope: &str,
        command_id: &str,
        refusal_reason: &str,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        Ok(self.conn.execute(
            "UPDATE execution_control_commands SET state = 'refused', refusal_reason = ?3, updated_at = ?4 WHERE issuer_scope = ?1 AND command_id = ?2 AND state = 'prepared'",
            params![issuer_scope, command_id, refusal_reason, observed_at],
        )? == 1)
    }

    /// Le wrapper clôt seulement son propre traitement. L'effet fournisseur
    /// reste porté par les transitions d'exécution corrélées.
    pub fn resolve_control_command(
        &self,
        issuer_scope: &str,
        command_id: &str,
        accepted: bool,
        refusal_reason: Option<&str>,
        observed_at: i64,
    ) -> rusqlite::Result<bool> {
        let state = if accepted { "accepted" } else { "refused" };
        Ok(self.conn.execute(
            "UPDATE execution_control_commands SET state = ?3, refusal_reason = ?4, updated_at = ?5 WHERE issuer_scope = ?1 AND command_id = ?2 AND state = 'dispatched'",
            params![issuer_scope, command_id, state, refusal_reason, observed_at],
        )? == 1)
    }

    pub fn lookup_control_command(
        &self,
        issuer_scope: &str,
        command_id: &str,
        observed_at: i64,
    ) -> rusqlite::Result<Option<StoredControlCommand>> {
        self.conn.query_row(
            "SELECT execution_id, state, refusal_reason, expires_at FROM execution_control_commands WHERE issuer_scope = ?1 AND command_id = ?2 AND expires_at > ?3",
            params![issuer_scope, command_id, observed_at],
            |row| {
                let state: String = row.get(1)?;
                let refusal_reason: Option<String> = row.get(2)?;
                let status = match state.as_str() {
                    "prepared" => ControlCommandStatus::Prepared,
                    "dispatched" => ControlCommandStatus::Dispatched,
                    "accepted" => ControlCommandStatus::Accepted,
                    "refused" => ControlCommandStatus::Refused {
                        reason: refusal_reason.unwrap_or_else(|| "refus_sans_raison".to_string()),
                    },
                    _ => ControlCommandStatus::Refused {
                        reason: "etat_controle_inconnu".to_string(),
                    },
                };
                Ok(StoredControlCommand {
                    execution_id: row.get(0)?,
                    status,
                    expires_at: row.get(3)?,
                })
            },
        ).optional()
    }

    /// Écrit une transition uniquement si l'événement concerne encore le même cycle.
    /// Les événements tardifs sont refusés sans réouvrir ni modifier l'exécution.
    #[allow(clippy::too_many_arguments)]
    pub fn transition_if_current(
        &self,
        execution_id: &str,
        expected_state: &str,
        expected_revision: u64,
        expected_generation: u64,
        next_state: &str,
        reason: &str,
        observed_at: i64,
    ) -> rusqlite::Result<ConditionalTransition> {
        let tx = self.conn.unchecked_transaction()?;
        let current = tx
            .query_row(
                "SELECT execution_id, project_id, binding_generation, state, generation, revision, created_at FROM executions WHERE execution_id = ?1",
                [execution_id],
                |row| {
                    Ok(ExecutionSnapshot {
                        execution_id: row.get(0)?,
                        project: project_reference_from_parts(row.get(1)?, row.get(2)?)?,
                        state: row.get(3)?,
                        generation: row.get(4)?,
                        revision: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .optional()?;
        let Some(current) = current else {
            tx.commit()?;
            return Ok(ConditionalTransition::Missing);
        };
        if current.state != expected_state
            || current.revision != expected_revision
            || current.generation != expected_generation
        {
            tx.commit()?;
            return Ok(ConditionalTransition::Rejected(current));
        }
        let updated = tx.execute(
            "UPDATE executions SET state = ?1, reason = ?2, revision = revision + 1, updated_at = ?3 WHERE execution_id = ?4 AND state = ?5 AND revision = ?6 AND generation = ?7",
            params![
                next_state,
                reason,
                observed_at,
                execution_id,
                expected_state,
                expected_revision,
                expected_generation,
            ],
        )?;
        if updated != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let applied = ExecutionSnapshot {
            execution_id: current.execution_id,
            project: current.project,
            state: next_state.to_string(),
            generation: current.generation,
            revision: expected_revision + 1,
            created_at: current.created_at,
        };
        tx.commit()?;
        Ok(ConditionalTransition::Applied(applied))
    }

    /// Reconstruit au plus une exécution active d'un agent après perte du
    /// processus fournisseur.
    ///
    /// Le parent, l'enfant et leur lignée partagent la transaction. Le message
    /// exact est décodé avant toute création d'enfant; une enveloppe absente ou
    /// corrompue ferme le parent sans inventer de prompt.
    pub fn reconstruct_active_for_agent(
        &self,
        target_agent: &str,
        agent_instance_id: &str,
        child_execution_id: &str,
        observed_at: i64,
    ) -> rusqlite::Result<ExecutionRecoveryOutcome> {
        if target_agent.trim().is_empty()
            || agent_instance_id.trim().is_empty()
            || child_execution_id.trim().is_empty()
            || observed_at < 0
        {
            return Err(rusqlite::Error::InvalidParameterName(
                "reconstruction d'exécution invalide".to_string(),
            ));
        }
        let tx = self.conn.unchecked_transaction()?;
        let candidates = {
            let mut statement = tx.prepare(
                "SELECT execution.execution_id, execution.submission_id,
                        execution.project_id, execution.binding_generation,
                        execution.generation, execution.revision,
                        submission.message_json
                 FROM executions execution
                 JOIN work_submissions submission
                   ON submission.submission_id = execution.submission_id
                 WHERE submission.target_agent = ?1
                   AND (execution.state IN (
                       'queued', 'starting', 'running', 'waiting_approval',
                       'waiting_user_input', 'interrupting'
                   ) OR execution.execution_id IN (SELECT execution_id FROM control_pause_interruptions WHERE resumed_at IS NULL))
                 ORDER BY execution.created_at, execution.execution_id
                 LIMIT 2",
            )?;
            statement
                .query_map([target_agent], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        project_reference_from_parts(row.get(2)?, row.get(3)?)?,
                        row.get::<_, u64>(4)?,
                        row.get::<_, u64>(5)?,
                        row.get::<_, Option<String>>(6)?,
                    ))
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        if candidates.is_empty() {
            tx.commit()?;
            return Ok(ExecutionRecoveryOutcome::None);
        }
        if candidates.len() > 1 {
            let execution_ids = candidates
                .into_iter()
                .map(|candidate| candidate.0)
                .collect();
            tx.commit()?;
            return Ok(ExecutionRecoveryOutcome::Ambiguous { execution_ids });
        }
        let (
            parent_execution_id,
            submission_id,
            project,
            parent_generation,
            parent_revision,
            message_json,
        ) = candidates.into_iter().next().expect("candidat vérifié");
        let message = message_json
            .and_then(|value| serde_json::from_str::<bridget_core::BridgetMessage>(&value).ok());
        let Some(message) = message else {
            tx.execute(
                "UPDATE executions
                 SET state = 'unreachable',
                     reason = 'recovery_payload_unavailable',
                     revision = revision + 1,
                     updated_at = ?2
                 WHERE execution_id = ?1
                   AND revision = ?3
                   AND state IN (
                       'queued', 'starting', 'running', 'waiting_approval',
                       'waiting_user_input', 'interrupting'
                   )",
                params![parent_execution_id, observed_at, parent_revision],
            )?;
            tx.commit()?;
            return Ok(ExecutionRecoveryOutcome::PayloadUnavailable {
                parent_execution_id,
            });
        };
        let closed = tx.execute(
            "UPDATE executions
             SET state = CASE WHEN state = 'interrupted' THEN state ELSE 'unreachable' END,
                 reason = CASE WHEN state = 'interrupted' THEN 'control_resume' ELSE 'daemon_restart' END,
                 revision = revision + 1, updated_at = ?2
             WHERE execution_id = ?1 AND revision = ?3
               AND (state IN (
                   'queued', 'starting', 'running', 'waiting_approval',
                   'waiting_user_input', 'interrupting'
               ) OR (state = 'interrupted' AND execution_id IN (SELECT execution_id FROM control_pause_interruptions WHERE resumed_at IS NULL)))",
            params![parent_execution_id, observed_at, parent_revision],
        )?;
        if closed != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        let child_generation = parent_generation
            .checked_add(1)
            .ok_or(rusqlite::Error::IntegralValueOutOfRange(4, i64::MAX))?;
        tx.execute(
            "INSERT INTO executions (
                execution_id, submission_id, agent_instance_id,
                project_id, binding_generation, generation, state,
                revision, reason, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'starting', 0,
                       'accepted', ?7, ?7)",
            params![
                child_execution_id,
                submission_id,
                agent_instance_id,
                project.as_ref().map(|value| &value.project_id),
                project.as_ref().map(|value| value.binding_generation),
                child_generation,
                observed_at,
            ],
        )?;
        tx.execute("UPDATE control_pause_interruptions SET resumed_at = ?2, resume_execution_id = ?3 WHERE execution_id = ?1 AND resumed_at IS NULL", params![parent_execution_id, observed_at, child_execution_id])?;
        tx.execute(
            "INSERT INTO execution_continuations (
                execution_id, generation, parent_execution_id,
                mode, reason, observed_at
             ) VALUES (?1, ?2, ?3, 'reconstructed', 'daemon_restart', ?4)",
            params![
                child_execution_id,
                child_generation,
                parent_execution_id,
                observed_at
            ],
        )?;
        let snapshot = ExecutionSnapshot {
            execution_id: child_execution_id.to_string(),
            project,
            state: "starting".to_string(),
            generation: child_generation,
            revision: 0,
            created_at: observed_at,
        };
        tx.commit()?;
        Ok(ExecutionRecoveryOutcome::Reconstructed(Box::new(
            ReconstructedExecution {
                parent_execution_id,
                snapshot,
                message,
            },
        )))
    }

    /// Sélection bornée utilisée avant une reconstruction. Deux identifiants
    /// suffisent à distinguer le cas nominal du cas ambigu sans charger tout
    /// l'historique d'un agent.
    pub fn recoverable_execution_ids_for_agent(
        &self,
        target_agent: &str,
    ) -> rusqlite::Result<Vec<String>> {
        let mut statement = self.conn.prepare(
            "SELECT execution.execution_id
             FROM executions execution
             JOIN work_submissions submission
               ON submission.submission_id = execution.submission_id
             WHERE submission.target_agent = ?1
               AND (execution.state IN ('queued', 'starting', 'running',
                   'waiting_approval', 'waiting_user_input', 'interrupting')
                    OR execution.execution_id IN (SELECT execution_id FROM control_pause_interruptions WHERE resumed_at IS NULL))
             ORDER BY execution.created_at, execution.execution_id
             LIMIT 2",
        )?;
        statement
            .query_map([target_agent], |row| row.get(0))?
            .collect()
    }

    /// Identifiants des exécutions qu'un redémarrage Bridget doit reprendre ou
    /// réconcilier avec le fournisseur avant toute nouvelle transition métier.
    pub fn recoverable_execution_ids(&self) -> rusqlite::Result<Vec<String>> {
        let mut statement = self.conn.prepare(
            "SELECT execution_id FROM executions WHERE state IN ('queued', 'starting', 'running', 'waiting_approval', 'waiting_user_input', 'interrupting') ORDER BY created_at, execution_id",
        )?;
        statement
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()
    }

    pub fn record_pause_interruption(
        &self,
        execution_id: &str,
        target_agent: &str,
        control_generation: u64,
        now: i64,
    ) -> rusqlite::Result<()> {
        self.conn.execute("INSERT OR IGNORE INTO control_pause_interruptions(execution_id, target_agent, control_generation, requested_at) VALUES (?1, ?2, ?3, ?4)", params![execution_id, target_agent, control_generation, now])?;
        Ok(())
    }

    pub fn pending_pause_interruptions(&self) -> rusqlite::Result<Vec<(String, String)>> {
        let mut statement = self.conn.prepare("SELECT execution_id, target_agent FROM control_pause_interruptions WHERE resumed_at IS NULL ORDER BY requested_at, execution_id")?;
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect()
    }

    pub fn schema_version(&self) -> rusqlite::Result<u32> {
        self.conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM execution_schema_migrations",
            [],
            |row| row.get(0),
        )
    }

    /// Borne les démarrages restés sans preuve de tour fournisseur.
    ///
    /// Cette écriture ne touche que l'état starting : un tour déjà observé doit
    /// être traité par sa propre politique de progrès, jamais par ce délai de
    /// remise initiale.
    pub fn expire_starting_before(
        &self,
        cutoff_at: i64,
        observed_at: i64,
    ) -> rusqlite::Result<usize> {
        self.conn.execute(
            "UPDATE executions SET state = 'unreachable', reason = 'provider_unavailable', revision = revision + 1, updated_at = ?1 WHERE state = 'starting' AND updated_at < ?2",
            params![observed_at, cutoff_at],
        )
    }

    fn init_schema(conn: &mut Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            r#"PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS execution_schema_migrations (version INTEGER PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS work_submissions (submission_id TEXT PRIMARY KEY, message_id TEXT NOT NULL UNIQUE, target_agent TEXT NOT NULL, origin TEXT, intent TEXT, references_json TEXT NOT NULL DEFAULT "[]", message_json TEXT, state TEXT NOT NULL CHECK (state IN ("prepared", "accepted", "cancelled")), accepted_at INTEGER NOT NULL);
            CREATE INDEX IF NOT EXISTS idx_work_submissions_target_state ON work_submissions(target_agent, state, accepted_at);
            CREATE TABLE IF NOT EXISTS executions (execution_id TEXT PRIMARY KEY, submission_id TEXT NOT NULL, agent_instance_id TEXT, project_id TEXT, binding_generation INTEGER, generation INTEGER NOT NULL CHECK (generation > 0), state TEXT NOT NULL, revision INTEGER NOT NULL CHECK (revision >= 0), reason TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, FOREIGN KEY (submission_id) REFERENCES work_submissions(submission_id));
            CREATE INDEX IF NOT EXISTS idx_executions_submission ON executions(submission_id);
            CREATE TABLE IF NOT EXISTS provider_bindings (execution_id TEXT PRIMARY KEY, provider_kind TEXT NOT NULL, execution_path TEXT NOT NULL, binary_path TEXT, binary_version TEXT, binary_digest TEXT, contract_version TEXT, provider_session_id TEXT, provider_thread_id TEXT, active_turn_id TEXT, capabilities_revision TEXT, observed_at INTEGER NOT NULL, FOREIGN KEY (execution_id) REFERENCES executions(execution_id));
            CREATE TABLE IF NOT EXISTS message_correlations (client_message_id TEXT PRIMARY KEY, submission_id TEXT NOT NULL, delivery_id TEXT, provider_item_id TEXT, provider_thread_id TEXT, provider_turn_id TEXT, visibility_event TEXT, visible_at INTEGER, FOREIGN KEY (submission_id) REFERENCES work_submissions(submission_id));
            CREATE INDEX IF NOT EXISTS idx_message_correlations_submission ON message_correlations(submission_id);"#,
        )?;
        conn.execute_batch(r#"CREATE TABLE IF NOT EXISTS execution_control_commands (issuer_scope TEXT NOT NULL, command_id TEXT NOT NULL, canonical_bytes BLOB NOT NULL, execution_id TEXT NOT NULL, state TEXT NOT NULL CHECK (state IN ("prepared", "dispatched", "accepted", "refused")), refusal_reason TEXT, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, expires_at INTEGER NOT NULL, PRIMARY KEY (issuer_scope, command_id), FOREIGN KEY (execution_id) REFERENCES executions(execution_id)); CREATE INDEX IF NOT EXISTS idx_execution_control_commands_expires ON execution_control_commands(expires_at);"#)?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (1)",
            [],
        )?;
        if !execution_column(&tx, "revision")? {
            tx.execute_batch("ALTER TABLE executions ADD COLUMN generation INTEGER NOT NULL DEFAULT 1; ALTER TABLE executions ADD COLUMN revision INTEGER NOT NULL DEFAULT 0; ALTER TABLE executions ADD COLUMN reason TEXT; ALTER TABLE executions ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0; ALTER TABLE executions ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0;")?;
        }
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (2)",
            [],
        )?;
        tx.execute("CREATE TABLE IF NOT EXISTS execution_queue (submission_id TEXT PRIMARY KEY, target_agent TEXT NOT NULL, priority INTEGER NOT NULL, enqueued_at INTEGER NOT NULL, FOREIGN KEY (submission_id) REFERENCES work_submissions(submission_id) ON DELETE CASCADE)", [])?;
        tx.execute("CREATE INDEX IF NOT EXISTS idx_execution_queue_next ON execution_queue(target_agent, priority DESC, enqueued_at, submission_id)", [])?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (3)",
            [],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (4)",
            [],
        )?;
        if !work_submission_column(&tx, "message_json")? {
            tx.execute_batch("ALTER TABLE work_submissions ADD COLUMN message_json TEXT;")?;
        }
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (5)",
            [],
        )?;
        if !provider_binding_column(&tx, "binary_path")? {
            tx.execute_batch("ALTER TABLE provider_bindings ADD COLUMN binary_path TEXT;")?;
        }
        if !provider_binding_column(&tx, "binary_version")? {
            tx.execute_batch("ALTER TABLE provider_bindings ADD COLUMN binary_version TEXT;")?;
        }
        if !provider_binding_column(&tx, "binary_digest")? {
            tx.execute_batch("ALTER TABLE provider_bindings ADD COLUMN binary_digest TEXT;")?;
        }
        if !provider_binding_column(&tx, "contract_version")? {
            tx.execute_batch("ALTER TABLE provider_bindings ADD COLUMN contract_version TEXT;")?;
        }
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (6)",
            [],
        )?;
        tx.execute_batch(r#"CREATE TABLE IF NOT EXISTS execution_continuations (execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id), generation INTEGER NOT NULL CHECK (generation > 0), parent_execution_id TEXT NOT NULL REFERENCES executions(execution_id), mode TEXT NOT NULL CHECK (mode IN ("native", "forked", "reconstructed")), reason TEXT, observed_at INTEGER NOT NULL CHECK (observed_at >= 0)); CREATE INDEX IF NOT EXISTS idx_execution_continuations_parent ON execution_continuations(parent_execution_id);"#)?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (7)",
            [],
        )?;
        tx.execute_batch(r#"CREATE TABLE IF NOT EXISTS execution_usage_totals (execution_id TEXT NOT NULL REFERENCES executions(execution_id), generation INTEGER NOT NULL CHECK (generation > 0), source TEXT NOT NULL CHECK (length(source) <= 64), turns INTEGER NOT NULL CHECK (turns >= 0), input_tokens INTEGER NOT NULL CHECK (input_tokens >= 0), output_tokens INTEGER NOT NULL CHECK (output_tokens >= 0), cache_creation_input_tokens INTEGER NOT NULL CHECK (cache_creation_input_tokens >= 0), cache_read_input_tokens INTEGER NOT NULL CHECK (cache_read_input_tokens >= 0), observed_at INTEGER NOT NULL CHECK (observed_at > 0), PRIMARY KEY (execution_id, generation, source)); CREATE INDEX IF NOT EXISTS idx_execution_usage_totals_execution ON execution_usage_totals(execution_id, generation); CREATE TABLE IF NOT EXISTS execution_continuation_reservations (parent_execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id), continuation_id TEXT NOT NULL UNIQUE, generation INTEGER NOT NULL CHECK (generation > 0), revision INTEGER NOT NULL CHECK (revision >= 0), proof_idle_at INTEGER NOT NULL CHECK (proof_idle_at > 0), reserved_at INTEGER NOT NULL CHECK (reserved_at > 0));"#)?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (8)",
            [],
        )?;
        if !execution_column(&tx, "project_id")? {
            tx.execute_batch("ALTER TABLE executions ADD COLUMN project_id TEXT; ALTER TABLE executions ADD COLUMN binding_generation INTEGER;")?;
        }
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (9)",
            [],
        )?;
        tx.execute_batch(r#"CREATE TABLE IF NOT EXISTS control_pause_interruptions (execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id), target_agent TEXT NOT NULL, control_generation INTEGER NOT NULL, requested_at INTEGER NOT NULL, resumed_at INTEGER, resume_execution_id TEXT); CREATE INDEX IF NOT EXISTS idx_control_pause_interruptions_pending ON control_pause_interruptions(target_agent) WHERE resumed_at IS NULL;"#)?;
        tx.execute(
            "INSERT OR IGNORE INTO execution_schema_migrations(version) VALUES (10)",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }
}

fn project_reference_from_parts(
    project_id: Option<String>,
    binding_generation: Option<i64>,
) -> rusqlite::Result<Option<ProjectReference>> {
    match (project_id, binding_generation) {
        (None, None) => Ok(None),
        (Some(project_id), Some(binding_generation)) if binding_generation > 0 => {
            Ok(Some(ProjectReference {
                project_id,
                binding_generation: binding_generation as u64,
            }))
        }
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            Type::Text,
            "référence projet d'exécution incomplète".into(),
        )),
    }
}

fn sql_counter(value: u64) -> rusqlite::Result<i64> {
    i64::try_from(value).map_err(|_| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "compteur fournisseur hors borne SQLite",
        )))
    })
}

fn provider_binding_column(tx: &Transaction<'_>, column: &str) -> rusqlite::Result<bool> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('provider_bindings') WHERE name = ?1)",
        [column],
        |row| row.get(0),
    )
}

fn execution_column(tx: &Transaction<'_>, column: &str) -> rusqlite::Result<bool> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('executions') WHERE name = ?1)",
        [column],
        |row| row.get(0),
    )
}

fn work_submission_column(tx: &Transaction<'_>, column: &str) -> rusqlite::Result<bool> {
    tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('work_submissions') WHERE name = ?1)",
        [column],
        |row| row.get(0),
    )
}

#[cfg(test)]
mod focus_priority_tests {
    use super::*;

    #[test]
    fn spec_087_une_remise_de_focus_passe_devant_le_travail_ordinaire() {
        let mut ordinary = bridget_core::BridgetMessage::new("a", "b", "x");
        ordinary.references = vec!["project:p@1".to_string()];
        assert_eq!(ExecutionStore::queue_priority_for(&ordinary, 0), 0);
        assert_eq!(ExecutionStore::queue_priority_for(&ordinary, 7), 7);
        let mut focus = bridget_core::BridgetMessage::new("a", "b", "x");
        focus.references = vec!["project:p@1".to_string(), "focus:o1".to_string()];
        assert_eq!(
            ExecutionStore::queue_priority_for(&focus, 0),
            FOCUS_QUEUE_PRIORITY
        );
        assert_eq!(
            ExecutionStore::queue_priority_for(&focus, FOCUS_QUEUE_PRIORITY + 1),
            FOCUS_QUEUE_PRIORITY + 1
        );
    }
}
