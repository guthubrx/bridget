//! Ledger, demandes suivies et observations d'usage.
//! Les clôtures appellent l'écrivain d'événement avec LA transaction courante.
use super::service_events::record_lifecycle_for_linked_request_in_transaction;
use super::{Store, StoreError, now_secs};
use bridget_transport::protocol::GuichetLifecycleState;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};

/// Règle de repli d'UN caractère, partagée par la normalisation des corps et
/// la localisation des occurrences : accents précomposés du français → ASCII.
/// La casse est repliée ensuite par `to_lowercase`. Aucune normalisation
/// Unicode (NFC/NFD) : une forme décomposée n'est pas assimilée.
fn fold_char(ch: char) -> char {
    match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
        'Ý' | 'Ÿ' | 'ÿ' | 'ý' => 'y',
        'Ç' | 'ç' => 'c',
        'Ñ' | 'ñ' => 'n',
        other => other,
    }
}

/// Repli des accents français → ASCII, pour que « cafe » trouve « café ».
pub(crate) fn fold_for_search(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        for lower in fold_char(ch).to_lowercase() {
            out.push(lower);
        }
    }
    out
}

/// Longueur en octets du repli d'un caractère, sans allouer.
fn folded_len(ch: char) -> usize {
    fold_char(ch).to_lowercase().map(char::len_utf8).sum()
}

/// Agrégat d'usage affichable par le centre de contrôle. Les champs de
/// fournisseur et de modèle restent absents pour les échantillons hérités :
/// l'interface doit alors les annoncer comme inconnus, jamais les deviner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageDashboardRow {
    pub provider_kind: Option<String>,
    pub model: Option<String>,
    pub source: String,
    pub samples: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

impl UsageDashboardRow {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_add(self.cache_creation_input_tokens)
            .saturating_add(self.cache_read_input_tokens)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedRequest {
    pub id: String,
    pub sender: String,
    pub target: String,
    pub state: String,
    pub created_at: i64,
    pub deadline_at: i64,
    pub escalation_level: u8,
    pub cancel_reason: Option<String>,
    pub completed_at: Option<i64>,
}

#[derive(Debug)]
pub struct LedgerEntry {
    pub id: String,
    pub ts: i64,
    pub sender: String,
    pub target: String,
    pub body: String,
    /// Phase `send_deliveries` si une saga idempotente porte le même id.
    pub delivery_phase: Option<String>,
}

impl Store {
    /// Résout le nom visible depuis la source d'autorité locale. L'absence est
    /// un fait possible tant qu'une migration n'a pas encore créé le profil.
    pub fn agent_display_name(&self, agent_id: &str) -> Result<Option<String>, StoreError> {
        self.conn
            .query_row(
                "SELECT display_name FROM agent_profiles WHERE agent_id = ?1",
                [agent_id],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(StoreError::Sqlite)
    }

    /// Enregistre un échantillon de consommation attesté. Jamais de zéro inventé
    /// ici : l'appelant n'émet que des faits complets du pilote.
    pub fn record_usage_sample(
        &self,
        agent: &str,
        observed_at: i64,
        tokens: bridget_transport::protocol::UsageTokens,
        source: &str,
        provider_kind: Option<&str>,
        model: Option<&str>,
    ) -> Result<(), StoreError> {
        self.conn
            .execute(
                "INSERT INTO usage_samples (
                     agent, observed_at, input_tokens, output_tokens,
                     cache_creation_input_tokens, cache_read_input_tokens,
                     provider_kind, model, source
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    agent,
                    observed_at,
                    tokens.input_tokens as i64,
                    tokens.output_tokens as i64,
                    tokens.cache_creation_input_tokens as i64,
                    tokens.cache_read_input_tokens as i64,
                    provider_kind,
                    model,
                    source,
                ],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Agrège les faits d'usage du serveur dans une fenêtre fermée. Le coût
    /// n'est pas calculé ici : sans grille de prix versionnée, un montant
    /// serait une invention comptable.
    pub fn usage_dashboard_window(
        &self,
        from_secs: i64,
        to_secs: i64,
    ) -> Result<Vec<UsageDashboardRow>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT provider_kind, model, source, COUNT(*),
                        COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0),
                        COALESCE(SUM(cache_creation_input_tokens), 0),
                        COALESCE(SUM(cache_read_input_tokens), 0)
                 FROM usage_samples
                 WHERE observed_at >= ?1 AND observed_at <= ?2
                 GROUP BY provider_kind, model, source
                 ORDER BY provider_kind IS NULL, provider_kind, model IS NULL, model, source",
            )
            .map_err(StoreError::Sqlite)?;
        let rows = statement
            .query_map(params![from_secs, to_secs], |row| {
                Ok(UsageDashboardRow {
                    provider_kind: row.get(0)?,
                    model: row.get(1)?,
                    source: row.get(2)?,
                    samples: row.get::<_, i64>(3)? as u64,
                    input_tokens: row.get::<_, i64>(4)? as u64,
                    output_tokens: row.get::<_, i64>(5)? as u64,
                    cache_creation_input_tokens: row.get::<_, i64>(6)? as u64,
                    cache_read_input_tokens: row.get::<_, i64>(7)? as u64,
                })
            })
            .map_err(StoreError::Sqlite)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    /// Agrège les échantillons d'un agent dans `[from_secs, to_secs]`.
    /// `None` si aucun échantillon — le greffe rendra « inconnu », pas zéro.
    pub fn aggregate_usage_window(
        &self,
        agent: &str,
        from_secs: i64,
        to_secs: i64,
    ) -> Result<Option<bridget_transport::protocol::UsageAggregate>, StoreError> {
        let row: Option<(i64, i64, i64, i64, i64)> = self
            .conn
            .query_row(
                "SELECT COUNT(*),
                        COALESCE(SUM(input_tokens), 0),
                        COALESCE(SUM(output_tokens), 0),
                        COALESCE(SUM(cache_creation_input_tokens), 0),
                        COALESCE(SUM(cache_read_input_tokens), 0)
                 FROM usage_samples
                 WHERE agent = ?1 AND observed_at >= ?2 AND observed_at <= ?3",
                params![agent, from_secs, to_secs],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        let Some((turns, input, output, cache_create, cache_read)) = row else {
            return Ok(None);
        };
        if turns <= 0 {
            return Ok(None);
        }
        let input_tokens = input as u64;
        let output_tokens = output as u64;
        let cache_creation_input_tokens = cache_create as u64;
        let cache_read_input_tokens = cache_read as u64;
        Ok(Some(bridget_transport::protocol::UsageAggregate {
            turns: turns as u64,
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            facturable_tokens: input_tokens
                .saturating_add(output_tokens)
                .saturating_add(cache_creation_input_tokens),
        }))
    }

    pub fn create_request(
        &self,
        id: &str,
        sender: &str,
        target: &str,
        timeout_secs: u64,
    ) -> Result<TrackedRequest, StoreError> {
        let created_at = now_secs();
        let request = TrackedRequest {
            id: id.to_string(),
            sender: sender.to_string(),
            target: target.to_string(),
            state: "open".to_string(),
            created_at,
            deadline_at: created_at + timeout_secs as i64,
            escalation_level: 0,
            cancel_reason: None,
            completed_at: None,
        };
        self.conn.execute(
            "INSERT INTO tracked_requests (id, sender, target, state, created_at, deadline_at, escalation_level) VALUES (?1, ?2, ?3, 'open', ?4, ?5, 0)",
            rusqlite::params![request.id, request.sender, request.target, request.created_at, request.deadline_at],
        ).map_err(StoreError::Sqlite)?;
        Ok(request)
    }

    pub fn get_request(&self, id: &str) -> Result<Option<TrackedRequest>, StoreError> {
        let mut stmt = self.conn.prepare("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE id = ?1").map_err(StoreError::Sqlite)?;
        let mut rows = stmt
            .query(rusqlite::params![id])
            .map_err(StoreError::Sqlite)?;
        match rows.next().map_err(StoreError::Sqlite)? {
            Some(row) => Ok(Some(
                tracked_request_from_row(row).map_err(StoreError::Sqlite)?,
            )),
            None => Ok(None),
        }
    }

    pub fn requests_for_sender(&self, sender: &str) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE sender = ?1 ORDER BY created_at DESC", rusqlite::params![sender])
    }

    pub fn requests_for_participant(
        &self,
        participant: &str,
        limit: usize,
    ) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests(
            "SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE sender = ?1 OR target = ?1 ORDER BY created_at DESC LIMIT ?2",
            rusqlite::params![participant, limit.clamp(1, 200)],
        )
    }

    /// Projection bornée des demandes, destinée aux lecteurs neutres du
    /// daemon (CLI fédérée aujourd'hui, vue ledger demain).
    pub fn recent_requests(&self, limit: usize) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests(
            "SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests ORDER BY created_at DESC LIMIT ?1",
            rusqlite::params![limit as i64],
        )
    }

    pub fn open_requests(&self) -> Result<Vec<TrackedRequest>, StoreError> {
        self.query_requests("SELECT id, sender, target, state, created_at, deadline_at, escalation_level, cancel_reason, completed_at FROM tracked_requests WHERE state = 'open' ORDER BY deadline_at", [])
    }

    pub fn cancel_request(
        &mut self,
        id: &str,
        sender: &str,
        reason: Option<&str>,
    ) -> Result<Option<TrackedRequest>, StoreError> {
        let Some(request) = self.get_request(id)? else {
            return Ok(None);
        };
        if request.sender != sender || (request.state != "open" && request.state != "cancelled") {
            return Ok(Some(request));
        }
        if request.state == "open" {
            let completed_at = now_secs();
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(StoreError::Sqlite)?;
            let changed = tx
                .execute(
                    "UPDATE tracked_requests
                     SET state = 'cancelled', cancel_reason = ?1, completed_at = ?2
                     WHERE id = ?3 AND state = 'open'",
                    rusqlite::params![reason, completed_at, id],
                )
                .map_err(StoreError::Sqlite)?;
            if changed == 1 {
                record_lifecycle_for_linked_request_in_transaction(
                    &tx,
                    id,
                    GuichetLifecycleState::Cancelled,
                    completed_at,
                )?;
            }
            tx.commit().map_err(StoreError::Sqlite)?;
            return self.get_request(id);
        }
        Ok(Some(request))
    }

    pub fn mark_answered(
        &self,
        id: &str,
        responder: &str,
        recipient: &str,
    ) -> Result<bool, StoreError> {
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        let answered = mark_answered_in_transaction(&transaction, id, responder, recipient)
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
        Ok(answered)
    }

    pub fn mark_timed_out(&mut self, id: &str) -> Result<bool, StoreError> {
        let completed_at = now_secs();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let changed = tx
            .execute(
                "UPDATE tracked_requests SET state = 'timed_out', completed_at = ?1
                 WHERE id = ?2 AND state = 'open'",
                rusqlite::params![completed_at, id],
            )
            .map_err(StoreError::Sqlite)?;
        if changed == 1 {
            record_lifecycle_for_linked_request_in_transaction(
                &tx,
                id,
                GuichetLifecycleState::TimedOut,
                completed_at,
            )?;
        }
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(changed == 1)
    }

    pub fn set_escalation_level(&self, id: &str, level: u8) -> Result<(), StoreError> {
        self.conn.execute("UPDATE tracked_requests SET escalation_level = ?1 WHERE id = ?2 AND state = 'open'", rusqlite::params![level, id]).map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Événement de cycle de vie consultable par la vue ledger : une relance a
    /// été retenue parce que l'équipier avait un tour ACP en cours.
    pub fn record_deferred_reminder(&self, id: &str, level: u8) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO request_events (request_id, event_type, level, ts) VALUES (?1, 'reminder_deferred', ?2, ?3)",
            rusqlite::params![id, level, now_secs()],
        ).map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Dernier report différé, exposé par la vue publique des demandes.
    pub fn latest_deferred_reminder(&self, id: &str) -> Result<Option<(u8, i64)>, StoreError> {
        let mut statement = self.conn.prepare(
            "SELECT level, ts FROM request_events WHERE request_id = ?1 AND event_type = 'reminder_deferred' ORDER BY ts DESC, rowid DESC LIMIT 1",
        ).map_err(StoreError::Sqlite)?;
        let mut rows = statement
            .query(rusqlite::params![id])
            .map_err(StoreError::Sqlite)?;
        rows.next()
            .map_err(StoreError::Sqlite)?
            .map(|row| Ok((row.get(0)?, row.get(1)?)))
            .transpose()
            .map_err(StoreError::Sqlite)
    }

    fn query_requests<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<TrackedRequest>, StoreError> {
        let mut stmt = self.conn.prepare(sql).map_err(StoreError::Sqlite)?;
        Ok(stmt
            .query_map(params, tracked_request_from_row)
            .map_err(StoreError::Sqlite)?
            .filter_map(Result::ok)
            .collect())
    }

    /// Enregistre un message dans le ledger.
    pub fn record_message(
        &self,
        msg: &bridget_core::BridgetMessage,
        conversation_key: &str,
    ) -> Result<(), StoreError> {
        let transaction = self
            .conn
            .unchecked_transaction()
            .map_err(StoreError::Sqlite)?;
        record_message_in_transaction(&transaction, msg, conversation_key)
            .map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Compte les échanges dans une conversation pendant les N dernières secondes.
    pub fn count_recent(
        &self,
        conversation_key: &str,
        window_secs: u64,
    ) -> Result<i64, StoreError> {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - window_secs as i64;

        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE conversation_key = ?1 AND ts >= ?2",
                rusqlite::params![conversation_key, cutoff],
                |row| row.get(0),
            )
            .map_err(StoreError::Sqlite)?;
        Ok(count)
    }

    /// Récupère les échanges récents, enrichis de la phase de remise quand une
    /// saga `send_deliveries` existe pour le même identifiant de message.
    pub fn recent_messages(&self, limit: usize) -> Result<Vec<LedgerEntry>, StoreError> {
        let has_deliveries = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table' AND name = 'send_deliveries'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(StoreError::Sqlite)?
            > 0;

        if has_deliveries {
            // L'index idx_send_deliveries_kind_key est posé par le batch DDL
            // d'IdempotencyStore (même db_path au démarrage daemon). Pas de
            // CREATE INDEX ici : une lecture ne doit pas exiger l'écriture
            // (mode=ro → « attempt to write a readonly database »).
            let mut stmt = self
                .conn
                .prepare(
                    "SELECT l.id, l.ts, l.sender, l.target, l.body,
                            (SELECT d.phase FROM send_deliveries d
                             WHERE d.operation_kind = 'send' AND d.idempotency_key = l.id
                             ORDER BY CASE d.phase
                                 WHEN 'acked' THEN 0
                                 WHEN 'orphaned' THEN 1
                                 WHEN 'dispatching' THEN 2
                                 ELSE 3
                             END
                             LIMIT 1) AS delivery_phase
                     FROM ledger l
                     ORDER BY l.ts DESC
                     LIMIT ?1",
                )
                .map_err(StoreError::Sqlite)?;

            let entries = stmt
                .query_map(rusqlite::params![limit as i64], |row| {
                    Ok(LedgerEntry {
                        id: row.get(0)?,
                        ts: row.get(1)?,
                        sender: row.get(2)?,
                        target: row.get(3)?,
                        body: row.get(4)?,
                        delivery_phase: row.get(5)?,
                    })
                })
                .map_err(StoreError::Sqlite)?
                .filter_map(|r| r.ok())
                .collect();
            return Ok(entries);
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, ts, sender, target, body FROM ledger
                 ORDER BY ts DESC LIMIT ?1",
            )
            .map_err(StoreError::Sqlite)?;

        let entries = stmt
            .query_map(rusqlite::params![limit as i64], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    ts: row.get(1)?,
                    sender: row.get(2)?,
                    target: row.get(3)?,
                    body: row.get(4)?,
                    delivery_phase: None,
                })
            })
            .map_err(StoreError::Sqlite)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }

    /// Messages d'une conversation bilatérale : filtre sur le couple AVANT la borne.
    /// Contrairement à `recent_messages`, le trafic d'autres conversations ne peut
    /// pas éjecter ces lignes — `limit` borne uniquement ce fil.
    pub fn conversation_messages(
        &self,
        party_a: &str,
        party_b: &str,
        limit: usize,
    ) -> Result<Vec<LedgerEntry>, StoreError> {
        let limit = limit.max(1) as i64;
        let key_ab = format!("{party_a}|{party_b}");
        let key_ba = format!("{party_b}|{party_a}");
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, ts, sender, target, body FROM ledger
                 WHERE conversation_key IN (?1, ?2)
                 ORDER BY ts ASC, id ASC
                 LIMIT ?3",
            )
            .map_err(StoreError::Sqlite)?;
        let entries = stmt
            .query_map(rusqlite::params![key_ab, key_ba, limit], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    ts: row.get(1)?,
                    sender: row.get(2)?,
                    target: row.get(3)?,
                    body: row.get(4)?,
                    delivery_phase: None,
                })
            })
            .map_err(StoreError::Sqlite)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(entries)
    }

    /// Réécrit atomiquement les références d'agents du relais. Les couples
    /// de conversation sont recalculés après chaque changement de principal.
    pub fn migrate_agent_references(
        &mut self,
        mapping: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), StoreError> {
        let transaction = self.conn.transaction().map_err(StoreError::Sqlite)?;
        for (legacy, agent_id) in mapping {
            transaction
                .execute(
                    "UPDATE ledger SET sender = ?1 WHERE sender = ?2",
                    rusqlite::params![agent_id, legacy],
                )
                .map_err(StoreError::Sqlite)?;
            transaction
                .execute(
                    "UPDATE ledger SET target = ?1 WHERE target = ?2",
                    rusqlite::params![agent_id, legacy],
                )
                .map_err(StoreError::Sqlite)?;
            transaction
                .execute(
                    "UPDATE tracked_requests SET sender = ?1 WHERE sender = ?2",
                    rusqlite::params![agent_id, legacy],
                )
                .map_err(StoreError::Sqlite)?;
            transaction
                .execute(
                    "UPDATE tracked_requests SET target = ?1 WHERE target = ?2",
                    rusqlite::params![agent_id, legacy],
                )
                .map_err(StoreError::Sqlite)?;
        }
        transaction.execute(
            "UPDATE ledger SET conversation_key = CASE WHEN sender <= target THEN sender || ':' || target ELSE target || ':' || sender END",
            [],
        ).map_err(StoreError::Sqlite)?;
        transaction.commit().map_err(StoreError::Sqlite)
    }

    /// Parcourt le ledger (pas d'index FTS) : chaque mot doit apparaître dans
    /// le corps. Ordre chronologique. Les jokers LIKE du needle sont échappés.
    /// Accents repliés (cafe ↔ café). `truncated` si plus de hits que le plafond.
    /// Tous les messages où `party` est émetteur OU destinataire, quel que soit
    /// son correspondant. `conversation_messages` interroge la clé de couple :
    /// demandée pour le couple (humain, humain) elle ne peut rien rendre, ce
    /// qui vidait la propre entrée de l'humain dans la vue. Les plus RÉCENTS
    /// sont retenus puis rendus en ordre croissant : borner en ordre croissant
    /// masquerait justement les derniers échanges.
    pub fn participant_messages(
        &self,
        party: &str,
        limit: usize,
    ) -> Result<Vec<LedgerEntry>, StoreError> {
        let limit = limit.max(1) as i64;
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, ts, sender, target, body FROM (
                     SELECT id, ts, sender, target, body FROM ledger
                     WHERE sender = ?1 OR target = ?1
                     ORDER BY ts DESC, id DESC
                     LIMIT ?2
                 ) ORDER BY ts ASC, id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        let entries = stmt
            .query_map(rusqlite::params![party, limit], |row| {
                Ok(LedgerEntry {
                    id: row.get(0)?,
                    ts: row.get(1)?,
                    sender: row.get(2)?,
                    target: row.get(3)?,
                    body: row.get(4)?,
                    delivery_phase: None,
                })
            })
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)?;
        Ok(entries)
    }
}

/// Transition commune de résolution d'une demande, réutilisable lorsqu'une
/// opération adjacente doit être rendue atomique avec cette clôture.
pub(crate) fn mark_answered_in_transaction(
    transaction: &Transaction<'_>,
    id: &str,
    responder: &str,
    recipient: &str,
) -> Result<bool, rusqlite::Error> {
    let changed = transaction.execute(
        "UPDATE tracked_requests
         SET state = 'answered', completed_at = ?1
         WHERE id = ?2 AND sender = ?3 AND target = ?4 AND state = 'open'",
        rusqlite::params![now_secs(), id, recipient, responder],
    )?;
    Ok(changed == 1)
}
/// Insère le message livré dans le ledger sans quitter la transaction en
/// cours. La clé `(id, target)` rend une reprise du même message idempotente.
pub(crate) fn record_message_in_transaction(
    transaction: &Transaction<'_>,
    msg: &bridget_core::BridgetMessage,
    conversation_key: &str,
) -> Result<(), rusqlite::Error> {
    transaction.execute(
        "INSERT OR REPLACE INTO ledger (id, ts, sender, target, body, conversation_key)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            msg.id,
            now_secs(),
            msg.from,
            msg.to,
            msg.body,
            conversation_key,
        ],
    )?;
    Ok(())
}
fn tracked_request_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackedRequest> {
    Ok(TrackedRequest {
        id: row.get(0)?,
        sender: row.get(1)?,
        target: row.get(2)?,
        state: row.get(3)?,
        created_at: row.get(4)?,
        deadline_at: row.get(5)?,
        escalation_level: row.get(6)?,
        cancel_reason: row.get(7)?,
        completed_at: row.get(8)?,
    })
}

/// Session 104 — accès de recherche et de relecture bornés, sur une
/// connexion quelconque (lecture seule côté daemon, Store en test).
///
/// Complexité par page : deux plages indexées O(log N + 129) puis une fusion
/// O(258), un chargement groupé des corps admissibles (< 17 Mio) et un repli
/// linéaire O(K × B) sur les corps examinés (K ≤ 8 termes). Aucun OFFSET ni
/// rowid : la reprise se fait par clé `(ts, id, target)` décroissante.
pub(crate) mod search {
    use super::fold_for_search;
    use rusqlite::{Connection, OptionalExtension, Transaction, params};
    use std::cmp::Ordering;
    use std::collections::HashMap;

    /// Candidats consommables par page ; la 129e clé n'est qu'un témoin.
    pub(crate) const CANDIDATES_PER_PAGE: usize = 128;
    /// Corps cumulés avant arrêt (une ligne entière peut dépasser).
    pub(crate) const BYTE_BUDGET: u64 = 1 << 20;
    /// Corps au-delà : ignoré sans chargement (`skipped_oversized`).
    pub(crate) const MAX_BODY_BYTES: u64 = 16 << 20;
    /// Identifiants hérités au-delà : refus `source_metadata_too_large`.
    pub(crate) const MAX_METADATA_BYTES: usize = 256;
    /// Extrait rendu depuis l'occurrence (frontière UTF-8).
    pub(crate) const EXCERPT_BYTES: usize = 512;
    const FETCH_LIMIT: usize = CANDIDATES_PER_PAGE + 1;

    #[derive(Debug)]
    pub(crate) enum SearchError {
        /// Toute erreur SQLite (busy, corruption, décodage de ligne) ; le
        /// détail n'est pas transmis au client.
        Storage,
        /// id/sender/target > 256 octets : aucune réponse partielle.
        MetadataTooLarge,
    }

    impl From<rusqlite::Error> for SearchError {
        fn from(error: rusqlite::Error) -> Self {
            log::warn!("recherche 104 : SQLite indisponible : {error}");
            Self::Storage
        }
    }

    /// Clé physique d'un échange, ordonnée comme la pagination :
    /// `(ts, id, target)` décroissants, texte comparé en binaire.
    #[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(crate) struct MessageKey {
        pub ts: i64,
        pub id: String,
        pub target: String,
    }

    impl MessageKey {
        /// Ordre de parcours : `Less` = vient AVANT dans la page (plus récent).
        pub(crate) fn page_cmp(&self, other: &Self) -> Ordering {
            other
                .ts
                .cmp(&self.ts)
                .then_with(|| other.id.as_bytes().cmp(self.id.as_bytes()))
                .then_with(|| other.target.as_bytes().cmp(self.target.as_bytes()))
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) struct MessageCandidate {
        pub key: MessageKey,
        pub sender: String,
        pub body_bytes: u64,
    }

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct DateWindow {
        pub since: i64,
        pub until: i64,
    }

    #[derive(Debug, Clone)]
    pub(crate) struct LoadedMessage {
        pub key: MessageKey,
        pub sender: String,
        pub body: String,
    }

    const META_TOO_LARGE: &str =
        "(octet_length(id) > 256 OR octet_length(target) > 256 OR octet_length(sender) > 256)";

    fn candidate_select(branch: &str, with_before: bool) -> String {
        let before = if with_before {
            " AND (ts, id, target) < (?7, ?8, ?9)"
        } else {
            ""
        };
        format!(
            "SELECT ts,
                    CASE WHEN {META_TOO_LARGE} THEN 1 ELSE 0 END,
                    CASE WHEN {META_TOO_LARGE} THEN '' ELSE id END,
                    CASE WHEN {META_TOO_LARGE} THEN '' ELSE target END,
                    CASE WHEN {META_TOO_LARGE} THEN '' ELSE sender END,
                    octet_length(body)
             FROM ledger
             WHERE {branch} AND ts >= ?2 AND ts <= ?3
               AND (ts, id, target) <= (?4, ?5, ?6){before}
             ORDER BY ts DESC, id DESC, target DESC
             LIMIT {FETCH_LIMIT}"
        )
    }

    pub(crate) const SENDER_BRANCH: &str = "sender = ?1";
    pub(crate) const TARGET_BRANCH: &str = "target = ?1 AND sender <> ?1";

    /// SQL des plages, exposé au test d'EXPLAIN (S23).
    #[cfg(test)]
    pub(crate) fn candidate_sql(branch: &str, with_before: bool) -> String {
        candidate_select(branch, with_before)
    }

    fn one_branch(
        conn: &Connection,
        branch: &str,
        actor: &str,
        window: DateWindow,
        upper: &MessageKey,
        before: Option<&MessageKey>,
    ) -> Result<Vec<MessageCandidate>, SearchError> {
        let sql = candidate_select(branch, before.is_some());
        let mut statement = conn.prepare(&sql)?;
        let map = |row: &rusqlite::Row<'_>| -> rusqlite::Result<(bool, MessageCandidate)> {
            Ok((
                row.get::<_, i64>(1)? == 1,
                MessageCandidate {
                    key: MessageKey {
                        ts: row.get(0)?,
                        id: row.get(2)?,
                        target: row.get(3)?,
                    },
                    sender: row.get(4)?,
                    body_bytes: row.get::<_, i64>(5)?.max(0) as u64,
                },
            ))
        };
        let rows: Vec<(bool, MessageCandidate)> = match before {
            Some(before) => statement
                .query_map(
                    params![
                        actor,
                        window.since,
                        window.until,
                        upper.ts,
                        upper.id,
                        upper.target,
                        before.ts,
                        before.id,
                        before.target
                    ],
                    map,
                )?
                .collect::<Result<_, _>>()?,
            None => statement
                .query_map(
                    params![
                        actor,
                        window.since,
                        window.until,
                        upper.ts,
                        upper.id,
                        upper.target
                    ],
                    map,
                )?
                .collect::<Result<_, _>>()?,
        };
        let mut out = Vec::with_capacity(rows.len());
        for (too_large, candidate) in rows {
            if too_large {
                return Err(SearchError::MetadataTooLarge);
            }
            out.push(candidate);
        }
        Ok(out)
    }

    /// Plus grande clé autorisée dans la fenêtre : borne haute de la première
    /// page. `None` = aucun échange visible.
    pub(crate) fn message_upper_bound(
        conn: &Connection,
        actor: &str,
        window: DateWindow,
    ) -> Result<Option<MessageKey>, SearchError> {
        let mut best: Option<MessageKey> = None;
        for branch in [SENDER_BRANCH, TARGET_BRANCH] {
            let sql = format!(
                "SELECT ts,
                        CASE WHEN {META_TOO_LARGE} THEN 1 ELSE 0 END,
                        CASE WHEN {META_TOO_LARGE} THEN '' ELSE id END,
                        CASE WHEN {META_TOO_LARGE} THEN '' ELSE target END
                 FROM ledger WHERE {branch} AND ts >= ?2 AND ts <= ?3
                 ORDER BY ts DESC, id DESC, target DESC LIMIT 1"
            );
            let found = conn
                .query_row(&sql, params![actor, window.since, window.until], |row| {
                    Ok((
                        row.get::<_, i64>(1)? == 1,
                        MessageKey {
                            ts: row.get(0)?,
                            id: row.get(2)?,
                            target: row.get(3)?,
                        },
                    ))
                })
                .optional()?;
            if let Some((too_large, key)) = found {
                if too_large {
                    return Err(SearchError::MetadataTooLarge);
                }
                if best
                    .as_ref()
                    .is_none_or(|current| key.page_cmp(current) == Ordering::Less)
                {
                    best = Some(key);
                }
            }
        }
        Ok(best)
    }

    /// Au plus 129 clés autorisées, fusion des deux plages (≤ 258), sans corps.
    pub(crate) fn message_candidates(
        conn: &Connection,
        actor: &str,
        window: DateWindow,
        upper: &MessageKey,
        before: Option<&MessageKey>,
    ) -> Result<Vec<MessageCandidate>, SearchError> {
        let sent = one_branch(conn, SENDER_BRANCH, actor, window, upper, before)?;
        let received = one_branch(conn, TARGET_BRANCH, actor, window, upper, before)?;
        let mut merged = Vec::with_capacity(sent.len() + received.len());
        let (mut a, mut b) = (sent.into_iter().peekable(), received.into_iter().peekable());
        while merged.len() < FETCH_LIMIT {
            match (a.peek(), b.peek()) {
                (None, None) => break,
                (Some(_), None) => merged.push(a.next().expect("pic présent")),
                (None, Some(_)) => merged.push(b.next().expect("pic présent")),
                (Some(x), Some(y)) => {
                    if x.key.page_cmp(&y.key) != Ordering::Greater {
                        merged.push(a.next().expect("pic présent"));
                    } else {
                        merged.push(b.next().expect("pic présent"));
                    }
                }
            }
        }
        Ok(merged)
    }

    /// Charge, en UNE requête, les corps des clés choisies (≤ 16 Mio chacune),
    /// en revérifiant la participation de l'acteur. Une clé disparue entre
    /// temps est simplement absente du résultat.
    pub(crate) fn message_bodies(
        conn: &Connection,
        actor: &str,
        keys: &[&MessageKey],
    ) -> Result<HashMap<(String, String), LoadedMessage>, SearchError> {
        let mut out = HashMap::with_capacity(keys.len());
        if keys.is_empty() {
            return Ok(out);
        }
        let values = std::iter::repeat_n("(?, ?)", keys.len())
            .collect::<Vec<_>>()
            .join(", ");
        // Jointure pilotée par la liste de clés : chaque clé fait une
        // recherche par clé primaire ; `IN (VALUES …)` laissait le planificateur
        // préférer un parcours des index participant (borné, mais linéaire).
        let sql = format!(
            "WITH keys(k_id, k_target) AS (VALUES {values})
             SELECT l.id, l.target, l.sender, l.ts, l.body
             FROM keys JOIN ledger AS l ON l.id = keys.k_id AND l.target = keys.k_target
             WHERE (l.sender = ?{actor_index} OR l.target = ?{actor_index})
               AND octet_length(l.body) <= {MAX_BODY_BYTES}",
            actor_index = keys.len() * 2 + 1
        );
        let mut binds: Vec<rusqlite::types::Value> = Vec::with_capacity(keys.len() * 2 + 1);
        for key in keys {
            binds.push(rusqlite::types::Value::Text(key.id.clone()));
            binds.push(rusqlite::types::Value::Text(key.target.clone()));
        }
        binds.push(rusqlite::types::Value::Text(actor.to_string()));
        let mut statement = conn.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(binds), |row| {
            Ok(LoadedMessage {
                key: MessageKey {
                    ts: row.get(3)?,
                    id: row.get(0)?,
                    target: row.get(1)?,
                },
                sender: row.get(2)?,
                body: row.get(4)?,
            })
        })?;
        for row in rows {
            let loaded = row?;
            out.insert((loaded.key.id.clone(), loaded.key.target.clone()), loaded);
        }
        Ok(out)
    }

    /// Métadonnées d'un message exact pour l'acteur participant :
    /// `(sender, ts, body_bytes)`, sans charger le corps.
    pub(crate) fn message_head(
        conn: &Connection,
        actor: &str,
        id: &str,
        target: &str,
    ) -> Result<Option<(String, i64, u64)>, rusqlite::Error> {
        conn.query_row(
            "SELECT sender, ts, octet_length(body) FROM ledger
             WHERE id = ?1 AND target = ?2 AND (sender = ?3 OR target = ?3)",
            params![id, target, actor],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, i64>(2)?.max(0) as u64,
                ))
            },
        )
        .optional()
    }

    pub(crate) fn message_body(
        conn: &Connection,
        actor: &str,
        id: &str,
        target: &str,
    ) -> Result<Option<String>, rusqlite::Error> {
        conn.query_row(
            "SELECT body FROM ledger
             WHERE id = ?1 AND target = ?2 AND (sender = ?3 OR target = ?3)",
            params![id, target, actor],
            |row| row.get(0),
        )
        .optional()
    }

    // ---------------------------------------------------------------- fils

    #[derive(Debug, Clone)]
    pub(crate) struct ThreadCandidate {
        pub seq: u64,
        pub message_id: String,
        pub author_id: String,
        pub created_at: i64,
        pub body_bytes: u64,
    }

    /// Au plus 129 entrées d'un fil, `seq` décroissant sous `upper`
    /// (inclus) et `before` (exclu). L'appartenance est vérifiée par
    /// l'appelant dans la MÊME transaction (`load_thread_for_member`).
    pub(crate) fn thread_candidates(
        tx: &Transaction<'_>,
        thread_id: &str,
        window: DateWindow,
        upper_seq: u64,
        before_seq: Option<u64>,
    ) -> Result<Vec<ThreadCandidate>, SearchError> {
        let before = before_seq.map(|seq| seq as i64).unwrap_or(i64::MAX);
        let mut statement = tx.prepare(
            "SELECT seq, message_id, author_id, created_at, octet_length(body)
             FROM discussion_entries
             WHERE thread_id = ?1 AND seq <= ?2 AND seq < ?3
               AND created_at >= ?4 AND created_at <= ?5
             ORDER BY seq DESC LIMIT ?6",
        )?;
        let rows = statement.query_map(
            params![
                thread_id,
                upper_seq as i64,
                before,
                window.since,
                window.until,
                FETCH_LIMIT as i64
            ],
            |row| {
                Ok(ThreadCandidate {
                    seq: row.get::<_, i64>(0)?.max(0) as u64,
                    message_id: row.get(1)?,
                    author_id: row.get(2)?,
                    created_at: row.get(3)?,
                    body_bytes: row.get::<_, i64>(4)?.max(0) as u64,
                })
            },
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub(crate) fn thread_bodies(
        tx: &Transaction<'_>,
        thread_id: &str,
        seqs: &[u64],
    ) -> Result<HashMap<u64, String>, SearchError> {
        let mut out = HashMap::with_capacity(seqs.len());
        if seqs.is_empty() {
            return Ok(out);
        }
        let marks = std::iter::repeat_n("?", seqs.len())
            .collect::<Vec<_>>()
            .join(", ");
        // `?1` explicite puis marqueurs anonymes : les anonymes continuent la
        // numérotation après le plus grand index déjà utilisé.
        let sql = format!(
            "SELECT seq, body FROM discussion_entries
             WHERE thread_id = ?1 AND seq IN ({marks})"
        );
        let mut binds: Vec<rusqlite::types::Value> = Vec::with_capacity(seqs.len() + 1);
        binds.push(rusqlite::types::Value::Text(thread_id.to_string()));
        binds.extend(
            seqs.iter()
                .map(|seq| rusqlite::types::Value::Integer(*seq as i64)),
        );
        let mut statement = tx.prepare(&sql)?;
        let rows = statement.query_map(rusqlite::params_from_iter(binds), |row| {
            Ok((
                row.get::<_, i64>(0)?.max(0) as u64,
                row.get::<_, String>(1)?,
            ))
        })?;
        for row in rows {
            let (seq, body) = row?;
            out.insert(seq, body);
        }
        Ok(out)
    }

    // ---------------------------------------------------------------- texte

    /// Termes déjà repliés (`fold_for_search`), tous requis. Le corps est
    /// replié UNE fois ; l'offset du premier terme trouvé est reconverti en
    /// offset du corps ORIGINAL par un seul parcours des caractères, sans
    /// tableau d'offsets. `None` si un terme manque.
    pub(crate) fn locate_terms(body: &str, folded_terms: &[String]) -> Option<usize> {
        let folded = fold_for_search(body);
        let mut first: Option<usize> = None;
        for term in folded_terms {
            let at = folded.find(term.as_str())?;
            first = Some(first.map_or(at, |current| current.min(at)));
        }
        let target = first?;
        let mut folded_offset = 0usize;
        for (original_offset, ch) in body.char_indices() {
            if folded_offset >= target {
                return Some(original_offset);
            }
            folded_offset += super::folded_len(ch);
        }
        // L'occurrence débute dans le repli du dernier caractère : son début
        // original est ce dernier caractère.
        body.char_indices().last().map(|(offset, _)| offset)
    }

    /// Extrait ≤ `max_bytes` depuis `offset` (frontières UTF-8 garanties).
    pub(crate) fn excerpt(body: &str, offset: usize, max_bytes: usize) -> &str {
        let start = offset.min(body.len());
        let start = floor_char_boundary(body, start);
        let end = floor_char_boundary(body, start.saturating_add(max_bytes).min(body.len()));
        &body[start..end]
    }

    pub(crate) fn floor_char_boundary(text: &str, index: usize) -> usize {
        let mut index = index.min(text.len());
        while !text.is_char_boundary(index) {
            index -= 1;
        }
        index
    }

    #[cfg(test)]
    mod spec104_text_tests {
        use super::*;

        #[test]
        fn spec104_repli_unique_et_offset_original_exact() {
            // Ÿ (2 octets) se replie en y (1 octet) : l'offset replié diverge
            // de l'offset original ; la reconversion doit rendre l'original.
            let body = "ŸŸŸ décision café";
            let terms = vec![fold_for_search("CAFE")];
            let offset = locate_terms(body, &terms).unwrap();
            assert_eq!(&body[offset..], "café");
            assert_eq!(offset, body.find("café").unwrap());
            // Plusieurs termes : le premier trouvé dans le corps localise.
            let terms = vec![fold_for_search("café"), fold_for_search("décision")];
            let offset = locate_terms(body, &terms).unwrap();
            assert_eq!(&body[offset..offset + "décision".len()], "décision");
            // Terme absent → None, même si les autres sont présents.
            assert!(locate_terms(body, &[fold_for_search("cafe"), "zzz".into()]).is_none());
            // Forme décomposée (e + U+0301) non assimilée au repli précomposé :
            // un terme décomposé ne trouve pas « café » ; limite documentée.
            assert!(locate_terms("café", &[fold_for_search("cafe\u{301}")]).is_none());
            assert!(locate_terms("café", &[fold_for_search("CAFE")]).is_some());
            // Jokers LIKE et guillemets littéraux, sans échappement.
            assert!(
                locate_terms(
                    "100% sûr_",
                    &[fold_for_search("%"), fold_for_search("sur_")]
                )
                .is_some()
            );
            assert!(locate_terms("dit \"bonjour\"", &[fold_for_search("\"bonjour\"")]).is_some());
        }

        #[test]
        fn spec104_extrait_respecte_les_frontieres_utf8() {
            let body = "ab😀cd";
            assert_eq!(excerpt(body, 0, 3), "ab");
            assert_eq!(excerpt(body, 2, 4), "😀");
            // Offset au milieu d'un caractère : ramené à sa frontière.
            assert_eq!(excerpt(body, 3, 10), "😀cd");
            assert_eq!(excerpt(body, 100, 10), "");
        }

        #[test]
        fn spec104_cle_de_page_ordonnee_en_binaire_decroissant() {
            let a = MessageKey {
                ts: 10,
                id: "b".into(),
                target: "x".into(),
            };
            let b = MessageKey {
                ts: 10,
                id: "a".into(),
                target: "z".into(),
            };
            let c = MessageKey {
                ts: 9,
                id: "z".into(),
                target: "z".into(),
            };
            assert_eq!(a.page_cmp(&b), Ordering::Less);
            assert_eq!(b.page_cmp(&c), Ordering::Less);
            assert_eq!(a.page_cmp(&a), Ordering::Equal);
        }
    }
}

#[cfg(test)]
mod spec104_index_tests {
    use super::search::{SENDER_BRANCH, TARGET_BRANCH};
    use rusqlite::Connection;

    fn fixture() -> (std::path::PathBuf, Connection) {
        let path = std::env::temp_dir().join(format!(
            "bridget-spec104-index-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let store = crate::store::Store::open(&path).unwrap();
        drop(store);
        (path.clone(), Connection::open(&path).unwrap())
    }

    fn plan(conn: &Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) -> String {
        let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows = statement
            .query_map(params, |row| row.get::<_, String>(3))
            .unwrap()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        rows.join(" / ")
    }

    /// T004 / S23 : les deux plages passent par les index participant, sans
    /// balayage de table ni tri temporaire ; réouverture idempotente.
    #[test]
    fn spec104_plages_indexees_et_migration_idempotente() {
        let (path, conn) = fixture();
        conn.execute_batch(
            "INSERT INTO ledger VALUES ('a', 10, 's', 't', 'x', 'k'), ('b', 9, 't', 's', 'y', 'k')",
        )
        .unwrap();
        let names: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='ledger' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(
            names,
            [
                "idx_ledger_conv",
                "idx_ledger_sender_page",
                "idx_ledger_target_page",
                "idx_ledger_ts",
                "sqlite_autoindex_ledger_1"
            ],
            "index existants conservés, deux index 104 ajoutés"
        );
        for (branch, index) in [
            (SENDER_BRANCH, "idx_ledger_sender_page"),
            (TARGET_BRANCH, "idx_ledger_target_page"),
        ] {
            for with_before in [false, true] {
                let sql = super::search::candidate_sql(branch, with_before);
                let plan = if with_before {
                    plan(
                        &conn,
                        &sql,
                        &[
                            &"s",
                            &0i64,
                            &i64::MAX,
                            &10i64,
                            &"a",
                            &"t",
                            &9i64,
                            &"b",
                            &"s",
                        ],
                    )
                } else {
                    plan(&conn, &sql, &[&"s", &0i64, &i64::MAX, &10i64, &"a", &"t"])
                };
                assert!(
                    plan.contains(index),
                    "{branch} before={with_before} : {plan}"
                );
                assert!(!plan.contains("SCAN ledger"), "{plan}");
                assert!(!plan.contains("TEMP B-TREE"), "{plan}");
            }
        }
        // Chargement des corps par clé primaire.
        let plan = plan(
            &conn,
            "WITH keys(k_id, k_target) AS (VALUES (?, ?)) SELECT l.id FROM keys JOIN ledger AS l ON l.id = keys.k_id AND l.target = keys.k_target WHERE (l.sender = ?3 OR l.target = ?3)",
            &[&"a", &"t", &"s"],
        );
        assert!(plan.contains("sqlite_autoindex_ledger_1"), "{plan}");
        assert!(
            !plan.contains("SCAN ledger") && !plan.contains("SCAN l "),
            "{plan}"
        );
        drop(conn);
        // Réouverture deux fois : aucune perte, aucune erreur de migration.
        for _ in 0..2 {
            let store = crate::store::Store::open(&path).unwrap();
            let count: i64 = store
                .connection()
                .query_row("SELECT count(*) FROM ledger", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 2);
        }
        let _ = std::fs::remove_file(&path);
    }
}
