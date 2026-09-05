//! Liens de propriété et observations de session : mêmes transactions qu'avant découpage.
use super::{
    AgentLinkEvent, AgentLinkRecord, AgentLinkState, DelegatedRuntimeEventInput,
    DelegatedRuntimeEventRecord, IdempotencyError, IdempotencyStore, to_sql_error,
};
use bridget_transport::protocol::{DelegatedRuntimeEventKind, ProjectReference};
use rusqlite::{OptionalExtension, TransactionBehavior, params};
use sha2::{Digest, Sha256};

impl IdempotencyStore {
    /// Crée le lien de propriété avant que la génération ne puisse démarrer.
    /// Le même enfant ne peut pas recevoir un second propriétaire ouvert.
    pub fn create_agent_link(&mut self, link: &AgentLinkRecord) -> Result<(), IdempotencyError> {
        if link.link_id.trim().is_empty()
            || link.parent_instance_id.trim().is_empty()
            || link.child_instance_id.trim().is_empty()
            || link.role.trim().is_empty()
            || link.agent_path.trim().is_empty()
            || link.created_at <= 0
            || link.revision != 0
            || link.state != AgentLinkState::Reserved
            || link.closed_at.is_some()
        {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let inserted = tx.execute(
            "INSERT INTO agent_links (
                link_id, parent_instance_id, child_instance_id, parent_execution_id,
                objective_id, delegation_id, role, agent_path, state, created_at,
                closed_at, revision, project_id, binding_generation
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'reserved', ?9, NULL, 0, ?10, ?11)
             ON CONFLICT(child_instance_id) DO NOTHING",
            params![
                link.link_id,
                link.parent_instance_id,
                link.child_instance_id,
                link.parent_execution_id,
                link.objective_id,
                link.delegation_id,
                link.role,
                link.agent_path,
                link.created_at,
                link.project.as_ref().map(|project| &project.project_id),
                link.project
                    .as_ref()
                    .map(|project| i64::try_from(project.binding_generation))
                    .transpose()
                    .map_err(|_| IdempotencyError::InvalidAgentLink)?,
            ],
        )?;
        if inserted == 0 {
            let existing = tx
                .query_row(
                    "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                            objective_id, delegation_id, role, agent_path, state, created_at,
                            closed_at, revision, project_id, binding_generation
                     FROM agent_links WHERE child_instance_id = ?1",
                    [&link.child_instance_id],
                    agent_link_from_row,
                )
                .optional()?;
            tx.commit()?;
            return match existing {
                Some(existing) if existing == *link => Ok(()),
                Some(_) => Err(IdempotencyError::AgentLinkOwned),
                None => Err(IdempotencyError::CorruptRecord(
                    "lien agent absent après conflit",
                )),
            };
        }
        record_agent_link_event(&tx, link, link.created_at)?;
        tx.commit()?;
        Ok(())
    }

    pub fn agent_link_for_child(
        &self,
        child_instance_id: &str,
    ) -> Result<Option<AgentLinkRecord>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE child_instance_id = ?1",
                [child_instance_id],
                agent_link_from_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn agent_link_by_id(
        &self,
        link_id: &str,
    ) -> Result<Option<AgentLinkRecord>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()
            .map_err(Into::into)
    }
    /// Enfants qui ont encore un propriétaire actif. Les liens fermés ou
    /// orphelins restent consultables par enfant mais ne consomment plus quota.
    pub fn open_agent_links_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<AgentLinkRecord>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')
             ORDER BY created_at, link_id",
        )?;
        statement
            .query_map([parent_instance_id], agent_link_from_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Compte les descendants encore rattachés à un propriétaire. Le décompte
    /// direct exploite l'index parent-état ; le total est réservé à la
    /// projection UI et suit le chemin stable, sans modifier les liens.
    pub fn open_agent_link_descendant_counts(
        &self,
        instance_id: &str,
    ) -> Result<(u64, u64), IdempotencyError> {
        let direct: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agent_links
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')",
            [instance_id],
            |row| row.get(0),
        )?;
        let path = self
            .agent_link_for_child(instance_id)?
            .map(|link| link.agent_path)
            .unwrap_or_else(|| instance_id.to_string());
        let escaped = path
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let descendants: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM agent_links
             WHERE agent_path LIKE ?1 ESCAPE '\\'
               AND state IN ('reserved', 'open', 'transferred')",
            [format!("{escaped}/%")],
            |row| row.get(0),
        )?;
        Ok((
            u64::try_from(direct).unwrap_or(0),
            u64::try_from(descendants).unwrap_or(0),
        ))
    }
    /// Relit les changements durables après un curseur pour reprendre une
    /// attente interrompue sans redemander l'état courant à un fournisseur.
    pub fn agent_link_events_after(
        &self,
        parent_instance_id: &str,
        after_cursor: Option<u64>,
    ) -> Result<Vec<AgentLinkEvent>, IdempotencyError> {
        let after_cursor = i64::try_from(after_cursor.unwrap_or_default()).unwrap_or(i64::MAX);
        let mut statement = self.conn.prepare(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id, state, observed_at,
                    project_id, binding_generation
             FROM agent_link_events
             WHERE parent_instance_id = ?1 AND cursor > ?2
             ORDER BY cursor
             LIMIT 128",
        )?;
        statement
            .query_map(
                params![parent_instance_id, after_cursor],
                agent_link_event_from_row,
            )?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    /// Persiste un fait runtime corrélé à un enfant déjà lié. L'identifiant est
    /// déterministe sur les seuls champs redacted afin que le rejeu ne duplique
    /// jamais le même diagnostic ou le même terminal.
    pub fn record_delegated_runtime_event(
        &mut self,
        input: DelegatedRuntimeEventInput,
    ) -> Result<DelegatedRuntimeEventRecord, IdempotencyError> {
        validate_delegated_runtime_event_input(&input)?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let link = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE child_instance_id = ?1",
                [&input.child_instance_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::InvalidDelegatedRuntimeEvent)?;
        let event_id = delegated_runtime_event_id(&link, &input);
        tx.execute(
            "INSERT INTO delegated_runtime_events (
                event_id, link_id, parent_instance_id, child_instance_id,
                child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                project_id, binding_generation
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, NULL, ?10, ?11)
             ON CONFLICT(event_id) DO NOTHING",
            params![
                &event_id,
                &link.link_id,
                &link.parent_instance_id,
                &link.child_instance_id,
                &input.child_execution_id,
                input.kind.as_str(),
                &input.code,
                &input.reference,
                input.observed_at,
                link.project.as_ref().map(|project| &project.project_id),
                link.project
                    .as_ref()
                    .map(|project| i64::try_from(project.binding_generation))
                    .transpose()
                    .map_err(|_| IdempotencyError::InvalidDelegatedRuntimeEvent)?,
            ],
        )?;
        let event = tx.query_row(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id,
                    child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                    project_id, binding_generation
             FROM delegated_runtime_events WHERE event_id = ?1",
            [&event_id],
            delegated_runtime_event_from_row,
        )?;
        tx.commit()?;
        Ok(event)
    }

    /// Relit dans l'ordre les faits qui n'ont pas encore franchi une frontière
    /// observable du transport du parent.
    pub fn delegated_runtime_events_for_parent(
        &self,
        parent_instance_id: &str,
    ) -> Result<Vec<DelegatedRuntimeEventRecord>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT cursor, event_id, link_id, parent_instance_id, child_instance_id,
                    child_execution_id, kind, code, reference, observed_at, acknowledged_at,
                    project_id, binding_generation
             FROM delegated_runtime_events
             WHERE parent_instance_id = ?1 AND acknowledged_at IS NULL
             ORDER BY cursor
             LIMIT 128",
        )?;
        statement
            .query_map([parent_instance_id], delegated_runtime_event_from_row)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Accuse un fait seulement si le wrapper qui parle est son parent attesté.
    /// Un accusé répétitif du même parent est idempotent; toute autre identité
    /// est un refus sans écriture.
    pub fn acknowledge_delegated_runtime_event(
        &mut self,
        event_id: &str,
        parent_instance_id: &str,
    ) -> Result<bool, IdempotencyError> {
        if event_id.trim().is_empty() || parent_instance_id.trim().is_empty() {
            return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (recorded_parent, acknowledged_at): (String, Option<i64>) = tx
            .query_row(
                "SELECT parent_instance_id, acknowledged_at
                 FROM delegated_runtime_events WHERE event_id = ?1",
                [event_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(IdempotencyError::InvalidDelegatedRuntimeEvent)?;
        if recorded_parent != parent_instance_id {
            return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
        }
        if acknowledged_at.is_some() {
            tx.commit()?;
            return Ok(false);
        }
        let acknowledged_at = unix_now_secs();
        let changed = tx.execute(
            "UPDATE delegated_runtime_events
             SET acknowledged_at = ?1
             WHERE event_id = ?2 AND parent_instance_id = ?3 AND acknowledged_at IS NULL",
            params![acknowledged_at, event_id, parent_instance_id],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(true)
    }

    pub fn transition_agent_link(
        &mut self,
        link_id: &str,
        allowed: &[AgentLinkState],
        next: AgentLinkState,
        changed_at: i64,
    ) -> Result<AgentLinkRecord, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::CorruptRecord("lien agent absent"))?;
        if current.state == next {
            tx.commit()?;
            return Ok(current);
        }
        if !allowed.contains(&current.state) {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let changed_at = changed_at.max(current.created_at);
        let closed_at = if next == AgentLinkState::Closed {
            Some(changed_at)
        } else {
            None
        };
        let changed = tx.execute(
            "UPDATE agent_links
             SET state = ?1, closed_at = ?2, revision = revision + 1
             WHERE link_id = ?3 AND revision = ?4",
            params![next.as_str(), closed_at, link_id, current.revision],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let updated = tx.query_row(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links WHERE link_id = ?1",
            [link_id],
            agent_link_from_row,
        )?;
        record_agent_link_event(&tx, &updated, changed_at)?;
        tx.commit()?;
        Ok(updated)
    }

    pub fn orphan_agent_links_for_parent(
        &mut self,
        parent_instance_id: &str,
        observed_at: i64,
    ) -> Result<usize, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let active = tx
            .prepare(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links
                 WHERE parent_instance_id = ?1
                   AND state IN ('reserved', 'open', 'transferred')",
            )?
            .query_map([parent_instance_id], agent_link_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        let changed = tx.execute(
            "UPDATE agent_links
             SET state = 'orphaned', revision = revision + 1
             WHERE parent_instance_id = ?1
               AND state IN ('reserved', 'open', 'transferred')",
            [parent_instance_id],
        )?;
        for link in active {
            let updated = AgentLinkRecord {
                state: AgentLinkState::Orphaned,
                revision: link.revision + 1,
                closed_at: None,
                ..link
            };
            record_agent_link_event(&tx, &updated, observed_at)?;
        }
        tx.commit()?;
        Ok(changed)
    }

    pub fn transfer_agent_link(
        &mut self,
        link_id: &str,
        parent_instance_id: &str,
        agent_path: &str,
        observed_at: i64,
    ) -> Result<AgentLinkRecord, IdempotencyError> {
        if parent_instance_id.trim().is_empty() || agent_path.trim().is_empty() {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = tx
            .query_row(
                "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                        objective_id, delegation_id, role, agent_path, state, created_at,
                        closed_at, revision, project_id, binding_generation
                 FROM agent_links WHERE link_id = ?1",
                [link_id],
                agent_link_from_row,
            )
            .optional()?
            .ok_or(IdempotencyError::CorruptRecord("lien agent absent"))?;
        if !matches!(
            current.state,
            AgentLinkState::Reserved
                | AgentLinkState::Open
                | AgentLinkState::Transferred
                | AgentLinkState::Orphaned
        ) {
            return Err(IdempotencyError::InvalidAgentLink);
        }
        let changed = tx.execute(
            "UPDATE agent_links
             SET parent_instance_id = ?1, agent_path = ?2, state = 'transferred',
                 closed_at = NULL, revision = revision + 1
             WHERE link_id = ?3 AND revision = ?4",
            params![parent_instance_id, agent_path, link_id, current.revision],
        )?;
        if changed != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let updated = tx.query_row(
            "SELECT link_id, parent_instance_id, child_instance_id, parent_execution_id,
                    objective_id, delegation_id, role, agent_path, state, created_at,
                    closed_at, revision, project_id, binding_generation
             FROM agent_links WHERE link_id = ?1",
            [link_id],
            agent_link_from_row,
        )?;
        record_agent_link_event(&tx, &updated, observed_at)?;
        tx.commit()?;
        Ok(updated)
    }
}

fn agent_link_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentLinkRecord> {
    Ok(AgentLinkRecord {
        link_id: row.get(0)?,
        parent_instance_id: row.get(1)?,
        child_instance_id: row.get(2)?,
        parent_execution_id: row.get(3)?,
        objective_id: row.get(4)?,
        delegation_id: row.get(5)?,
        role: row.get(6)?,
        agent_path: row.get(7)?,
        state: AgentLinkState::from_str(&row.get::<_, String>(8)?).map_err(to_sql_error)?,
        created_at: row.get(9)?,
        closed_at: row.get(10)?,
        revision: row.get(11)?,
        project: project_reference_from_parts(row.get(12)?, row.get(13)?)?,
    })
}

fn agent_link_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AgentLinkEvent> {
    Ok(AgentLinkEvent {
        cursor: row.get(0)?,
        event_id: row.get(1)?,
        link_id: row.get(2)?,
        parent_instance_id: row.get(3)?,
        child_instance_id: row.get(4)?,
        state: AgentLinkState::from_str(&row.get::<_, String>(5)?).map_err(to_sql_error)?,
        observed_at: row.get(6)?,
        project: project_reference_from_parts(row.get(7)?, row.get(8)?)?,
    })
}

fn record_agent_link_event(
    tx: &rusqlite::Transaction<'_>,
    link: &AgentLinkRecord,
    observed_at: i64,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO agent_link_events (
            event_id, link_id, parent_instance_id, child_instance_id, state, observed_at,
            project_id, binding_generation
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            format!("{}:{}", link.link_id, link.revision),
            link.link_id,
            link.parent_instance_id,
            link.child_instance_id,
            link.state.as_str(),
            observed_at.max(link.created_at),
            link.project.as_ref().map(|project| &project.project_id),
            link.project
                .as_ref()
                .map(|project| i64::try_from(project.binding_generation))
                .transpose()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(7, i64::MAX))?,
        ],
    )?;
    Ok(())
}

fn delegated_runtime_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<DelegatedRuntimeEventRecord> {
    let kind = row.get::<_, String>(6)?;
    Ok(DelegatedRuntimeEventRecord {
        cursor: row.get(0)?,
        event_id: row.get(1)?,
        link_id: row.get(2)?,
        parent_instance_id: row.get(3)?,
        child_instance_id: row.get(4)?,
        child_execution_id: row.get(5)?,
        kind: DelegatedRuntimeEventKind::parse(&kind)
            .ok_or_else(|| to_sql_error(IdempotencyError::InvalidDelegatedRuntimeEvent))?,
        code: row.get(7)?,
        reference: row.get(8)?,
        observed_at: row.get(9)?,
        acknowledged_at: row.get(10)?,
        project: project_reference_from_parts(row.get(11)?, row.get(12)?)?,
    })
}

fn project_reference_from_parts(
    project_id: Option<String>,
    binding_generation: Option<i64>,
) -> rusqlite::Result<Option<ProjectReference>> {
    match (project_id, binding_generation) {
        (None, None) => Ok(None),
        (Some(project_id), Some(binding_generation))
            if !project_id.trim().is_empty() && binding_generation > 0 =>
        {
            Ok(Some(ProjectReference {
                project_id,
                binding_generation: u64::try_from(binding_generation)
                    .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, binding_generation))?,
            }))
        }
        _ => Err(to_sql_error(IdempotencyError::InvalidAgentLink)),
    }
}

fn validate_delegated_runtime_event_input(
    input: &DelegatedRuntimeEventInput,
) -> Result<(), IdempotencyError> {
    let token = |value: &str, limit: usize| {
        !value.is_empty()
            && value.len() <= limit
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    };
    let reference = input.reference.strip_prefix("sha256:").unwrap_or_default();
    if !token(&input.child_instance_id, 256)
        || !token(&input.child_execution_id, 256)
        || !token(&input.code, 128)
        || reference.len() != 64
        || !reference.bytes().all(|byte| byte.is_ascii_hexdigit())
        || input.observed_at <= 0
    {
        return Err(IdempotencyError::InvalidDelegatedRuntimeEvent);
    }
    Ok(())
}

fn delegated_runtime_event_id(
    link: &AgentLinkRecord,
    input: &DelegatedRuntimeEventInput,
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"bridget/delegated-runtime-event/v1\0");
    let kind = input.kind.as_str();
    let parts: [&[u8]; 5] = [
        link.link_id.as_bytes(),
        input.child_execution_id.as_bytes(),
        kind.as_bytes(),
        input.code.as_bytes(),
        input.reference.as_bytes(),
    ];
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    format!("delegated-runtime:{:x}", digest.finalize())
}

fn unix_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}
