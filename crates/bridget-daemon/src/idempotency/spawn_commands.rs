//! Sagas de lancement : réservation, définition figée et issue durables.
use super::{
    HistoricalCanonicalSpawnOrder, HistoricalManagedSpawn, IdempotencyError, IdempotencyKey,
    IdempotencyStore, OperationKind, RecordState, SpawnCommand, SpawnCommandIssue,
    SpawnCommandState, SpawnReservation, load_record_from, to_sql_error, validate_canonical_bytes,
    validate_issuer_scope,
};
use bridget_transport::ResolvedAgentDefinition;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::path::PathBuf;

impl IdempotencyStore {
    /// Réserve atomiquement une clé `spawn` et sa ligne de saga. Cette API est
    /// l'unique point de décision du consommateur 009 : le code `fleet` ne
    /// refait ni lookup, ni comparaison canonique, ni logique de rejeu.
    #[allow(clippy::too_many_arguments)]
    pub fn reserve_spawn(
        &mut self,
        key: &IdempotencyKey,
        canonical_bytes: &[u8],
        issued_at: i64,
        horizon_secs: i64,
        now: i64,
        issued_at_tolerance_secs: i64,
        name: &str,
        generation: u64,
        persistent: bool,
        deadline_at: i64,
        instance_id: &str,
    ) -> Result<SpawnReservation, IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || name.is_empty()
            || generation == 0
            || deadline_at <= issued_at
            || instance_id.is_empty()
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        validate_canonical_bytes(canonical_bytes)?;
        if issued_at > now.saturating_add(issued_at_tolerance_secs.max(0)) {
            return Err(IdempotencyError::InvalidIssuedAt);
        }

        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = load_record_from(&tx, key)?;
        if let Some(record) = existing {
            let result = if record.expires_at <= now {
                SpawnReservation::IdempotencyExpired
            } else if record.canonical_bytes != canonical_bytes {
                SpawnReservation::EnvelopeMismatch
            } else {
                let command = load_spawn_command_from(&tx, key)?
                    .ok_or(IdempotencyError::CorruptRecord("saga spawn absente"))?;
                if record.state == RecordState::Terminal {
                    let issue = command
                        .issue
                        .clone()
                        .ok_or(IdempotencyError::CorruptRecord(
                            "issue spawn terminale absente",
                        ))?;
                    record.validate_spawn_issue(&issue)?;
                    SpawnReservation::Replayed(issue)
                } else if command.state.is_terminal() {
                    return Err(IdempotencyError::CorruptRecord(
                        "saga terminale avec socle non terminal",
                    ));
                } else {
                    SpawnReservation::InFlight(command)
                }
            };
            tx.commit()?;
            return Ok(result);
        }

        if horizon_secs <= 0 {
            return Err(IdempotencyError::InvalidHorizon);
        }
        let expires_at = issued_at
            .checked_add(horizon_secs)
            .ok_or(IdempotencyError::InvalidHorizon)?;
        if expires_at <= now {
            tx.commit()?;
            return Ok(SpawnReservation::IdempotencyExpired);
        }

        tx.execute(
            "INSERT INTO idempotency_records (
                issuer_scope, operation_kind, idempotency_key, canonical_bytes,
                state, issued_at, expires_at
             ) VALUES (?1, 'spawn', ?2, ?3, 'prepared', ?4, ?5)",
            params![
                key.issuer_scope,
                key.idempotency_key,
                canonical_bytes,
                issued_at,
                expires_at,
            ],
        )?;
        tx.execute(
            "INSERT INTO spawn_commands (
                issuer_scope, operation_kind, command_id, name, generation,
                persistent, state, instance_id, deadline_at, expires_at
             ) VALUES (?1, 'spawn', ?2, ?3, ?4, ?5, 'requested', ?6, ?7, ?8)",
            params![
                key.issuer_scope,
                key.idempotency_key,
                name,
                generation,
                persistent,
                instance_id,
                deadline_at,
                expires_at,
            ],
        )?;
        tx.commit()?;
        Ok(SpawnReservation::Requested(SpawnCommand {
            command_id: key.idempotency_key.clone(),
            name: name.to_string(),
            generation,
            persistent,
            state: SpawnCommandState::Requested,
            instance_id: Some(instance_id.to_string()),
            deadline_at,
            expires_at,
            issue: None,
            resolved_definition: None,
        }))
    }

    /// Avance la saga en vol. Le passage `Requested→Reserved` rend le socle
    /// `Dispatching` dans la même transaction que la réservation métier.
    pub fn advance_spawn(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        from: SpawnCommandState,
        to: SpawnCommandState,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || !matches!(
                (from, to),
                (SpawnCommandState::Requested, SpawnCommandState::Reserved)
                    | (SpawnCommandState::Reserved, SpawnCommandState::Starting)
            )
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if from == SpawnCommandState::Requested {
            let core = tx.execute(
                "UPDATE idempotency_records SET state = 'dispatching'
                 WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
                   AND idempotency_key = ?2 AND state = 'prepared'",
                params![key.issuer_scope, key.idempotency_key],
            )?;
            if core != 1 {
                return Err(IdempotencyError::DispatchUnavailable);
            }
        }
        let saga = tx.execute(
            "UPDATE spawn_commands SET state = ?1
             WHERE issuer_scope = ?2 AND operation_kind = 'spawn'
               AND command_id = ?3 AND generation = ?4 AND state = ?5",
            params![
                to.as_str(),
                key.issuer_scope,
                key.idempotency_key,
                generation,
                from.as_str(),
            ],
        )?;
        if saga != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    pub fn advance_spawn_with_definition(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        from: SpawnCommandState,
        to: SpawnCommandState,
        definition: &ResolvedAgentDefinition,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn
            || (from, to) != (SpawnCommandState::Reserved, SpawnCommandState::Starting)
        {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let definition_json =
            serde_json::to_string(definition).map_err(|_| IdempotencyError::InvalidSpawnCommand)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let updated = tx.execute(
            "UPDATE spawn_commands
             SET state = 'starting', resolved_definition_json = ?1
             WHERE issuer_scope = ?2 AND operation_kind = 'spawn'
               AND command_id = ?3 AND generation = ?4 AND state = 'reserved'",
            params![
                definition_json,
                key.issuer_scope,
                key.idempotency_key,
                generation
            ],
        )?;
        if updated != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    /// Finalise ensemble la saga et le résultat public du socle.
    pub fn finish_spawn(
        &mut self,
        key: &IdempotencyKey,
        generation: u64,
        issue: &SpawnCommandIssue,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Spawn || generation == 0 {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let (state, issue_kind, instance_id, category, reason, public_kind) = match issue {
            SpawnCommandIssue::Connected {
                name: _,
                generation: issue_generation,
                instance_id,
                definition: _,
            } if *issue_generation == generation && !instance_id.is_empty() => (
                SpawnCommandState::Connected,
                "connected",
                Some(instance_id.as_str()),
                None,
                None,
                "accepted",
            ),
            SpawnCommandIssue::Failed { category, reason }
                if !category.is_empty() && !reason.is_empty() =>
            {
                (
                    SpawnCommandState::Failed,
                    "failed",
                    None,
                    Some(category.as_str()),
                    Some(reason.as_str()),
                    "rejected",
                )
            }
            SpawnCommandIssue::Cancelled { reason } if !reason.is_empty() => (
                SpawnCommandState::Cancelled,
                "cancelled",
                None,
                Some("cancelled"),
                Some(reason.as_str()),
                "rejected",
            ),
            _ => return Err(IdempotencyError::InvalidSpawnCommand),
        };
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let saga = tx.execute(
            "UPDATE spawn_commands
             SET state = ?1, instance_id = COALESCE(?2, instance_id), issue_kind = ?3,
                 issue_category = ?4, issue_reason = ?5
             WHERE issuer_scope = ?6 AND operation_kind = 'spawn'
               AND command_id = ?7 AND generation = ?8
               AND state IN ('requested', 'reserved', 'starting')
               AND (?3 != 'connected' OR instance_id = ?2)",
            params![
                state.as_str(),
                instance_id,
                issue_kind,
                category,
                reason,
                key.issuer_scope,
                key.idempotency_key,
                generation,
            ],
        )?;
        if saga != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let core = tx.execute(
            "UPDATE idempotency_records
             SET state = 'terminal', public_result_kind = ?1,
                 public_result_category = ?2, public_result_reason = ?3
             WHERE issuer_scope = ?4 AND operation_kind = 'spawn'
               AND idempotency_key = ?5 AND state IN ('prepared', 'dispatching')",
            params![
                public_kind,
                category,
                reason,
                key.issuer_scope,
                key.idempotency_key,
            ],
        )?;
        if core != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    /// Charge les sagas d'un superviseur pour la réconciliation au démarrage.
    pub fn spawn_commands(
        &self,
        issuer_scope: &str,
    ) -> Result<Vec<SpawnCommand>, IdempotencyError> {
        validate_issuer_scope(issuer_scope)?;
        let mut statement = self.conn.prepare(
            "SELECT command_id, name, generation, persistent, state,
                    instance_id, deadline_at, expires_at,
                    issue_kind, issue_category, issue_reason, resolved_definition_json
             FROM spawn_commands WHERE issuer_scope = ?1 AND operation_kind = 'spawn'
             ORDER BY generation, command_id",
        )?;
        let rows = statement.query_map(params![issuer_scope], spawn_command_from_row)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Retrouve la dernière génération réellement connectée pour un nom,
    /// indépendamment du superviseur qui l'avait lancée. Le canon original
    /// fournit le cwd, le type et le projet ; la définition résolue prouve que
    /// le runtime avait été figé. Une ligne incomplète est refusée.
    pub fn latest_connected_spawn_by_name(
        &self,
        name: &str,
    ) -> Result<Option<HistoricalManagedSpawn>, IdempotencyError> {
        if name.trim().is_empty() {
            return Err(IdempotencyError::InvalidSpawnCommand);
        }
        let mut statement = self.conn.prepare(
            "SELECT sc.command_id, sc.name, sc.generation, sc.persistent, sc.state,
                    sc.instance_id, sc.deadline_at, sc.expires_at,
                    sc.issue_kind, sc.issue_category, sc.issue_reason,
                    sc.resolved_definition_json, ir.canonical_bytes
             FROM spawn_commands sc
             JOIN idempotency_records ir
               ON ir.issuer_scope = sc.issuer_scope
              AND ir.operation_kind = sc.operation_kind
              AND ir.idempotency_key = sc.command_id
             WHERE sc.operation_kind = 'spawn'
               AND sc.name = ?1
               AND sc.state = 'connected'
               AND sc.issue_kind = 'connected'
             ORDER BY sc.generation DESC, sc.command_id DESC
             LIMIT 1",
        )?;
        let found = statement
            .query_row(params![name], |row| {
                let command = spawn_command_from_row(row)?;
                let canonical_bytes: Vec<u8> = row.get(12)?;
                Ok((command, canonical_bytes))
            })
            .optional()?;
        let Some((command, canonical_bytes)) = found else {
            return Ok(None);
        };
        let canonical: HistoricalCanonicalSpawnOrder = serde_json::from_slice(&canonical_bytes)
            .map_err(|_| IdempotencyError::CorruptRecord("canon historique du spawn invalide"))?;
        if canonical.command_id != command.command_id
            || canonical.agent_type.trim().is_empty()
            || canonical.cwd.trim().is_empty()
            || canonical.persistent != command.persistent
        {
            return Err(IdempotencyError::CorruptRecord(
                "canon historique du spawn incohérent",
            ));
        }
        let resolved_definition =
            command
                .resolved_definition
                .ok_or(IdempotencyError::CorruptRecord(
                    "définition historique du spawn absente",
                ))?;
        Ok(Some(HistoricalManagedSpawn {
            agent_type: canonical.agent_type,
            cwd: PathBuf::from(canonical.cwd),
            command_id: command.command_id,
            generation: command.generation,
            persistent: command.persistent,
            project: canonical.project,
            resolved_definition,
        }))
    }
}

fn load_spawn_command_from(
    conn: &Connection,
    key: &IdempotencyKey,
) -> Result<Option<SpawnCommand>, IdempotencyError> {
    conn.query_row(
        "SELECT command_id, name, generation, persistent, state,
                instance_id, deadline_at, expires_at,
                issue_kind, issue_category, issue_reason, resolved_definition_json
         FROM spawn_commands
         WHERE issuer_scope = ?1 AND operation_kind = 'spawn' AND command_id = ?2",
        params![key.issuer_scope, key.idempotency_key],
        spawn_command_from_row,
    )
    .optional()
    .map_err(Into::into)
}
fn spawn_command_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpawnCommand> {
    let command_id = row.get(0)?;
    let name: String = row.get(1)?;
    let generation: u64 = row.get(2)?;
    let persistent: bool = row.get(3)?;
    let state = SpawnCommandState::from_str(&row.get::<_, String>(4)?).map_err(to_sql_error)?;
    let instance_id: Option<String> = row.get(5)?;
    let deadline_at = row.get(6)?;
    let expires_at = row.get(7)?;
    let issue_kind: Option<String> = row.get(8)?;
    let issue_category: Option<String> = row.get(9)?;
    let issue_reason: Option<String> = row.get(10)?;
    let resolved_definition = row
        .get::<_, Option<String>>(11)?
        .map(|json| {
            serde_json::from_str::<ResolvedAgentDefinition>(&json).map_err(|_| {
                to_sql_error(IdempotencyError::CorruptRecord(
                    "définition résolue du spawn invalide",
                ))
            })
        })
        .transpose()?;
    let issue = match issue_kind.as_deref() {
        None if !state.is_terminal() => None,
        Some("connected") if state == SpawnCommandState::Connected => {
            Some(SpawnCommandIssue::Connected {
                name: name.clone(),
                generation,
                instance_id: instance_id.clone().ok_or_else(|| {
                    to_sql_error(IdempotencyError::CorruptRecord(
                        "instance spawn connectée absente",
                    ))
                })?,
                definition: resolved_definition.clone().map(Box::new),
            })
        }
        Some("failed") if state == SpawnCommandState::Failed => Some(SpawnCommandIssue::Failed {
            category: issue_category.ok_or_else(|| {
                to_sql_error(IdempotencyError::CorruptRecord("catégorie spawn absente"))
            })?,
            reason: issue_reason.ok_or_else(|| {
                to_sql_error(IdempotencyError::CorruptRecord("motif spawn absent"))
            })?,
        }),
        Some("cancelled") if state == SpawnCommandState::Cancelled => {
            Some(SpawnCommandIssue::Cancelled {
                reason: issue_reason.ok_or_else(|| {
                    to_sql_error(IdempotencyError::CorruptRecord(
                        "motif annulation spawn absent",
                    ))
                })?,
            })
        }
        _ => {
            return Err(to_sql_error(IdempotencyError::CorruptRecord(
                "issue spawn incohérente",
            )));
        }
    };
    Ok(SpawnCommand {
        command_id,
        name,
        generation,
        persistent,
        state,
        instance_id,
        deadline_at,
        expires_at,
        issue,
        resolved_definition,
    })
}
