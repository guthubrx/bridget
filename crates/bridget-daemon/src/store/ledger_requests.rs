//! Ledger, demandes suivies et observations d'usage.
//! Les clôtures appellent l'écrivain d'événement avec LA transaction courante.
use super::service_events::record_lifecycle_for_linked_request_in_transaction;
use super::{Store, StoreError, now_secs};
use bridget_transport::protocol::GuichetLifecycleState;
use rusqlite::{OptionalExtension, Transaction, TransactionBehavior, params};

const MAX_LEDGER_SEARCH: usize = 100;

/// Échappe les jokers LIKE pour une recherche littérale.
pub(crate) fn escape_like_needle(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        match ch {
            '\\' | '%' | '_' => {
                out.push('\\');
                out.push(ch);
            }
            other => out.push(other),
        }
    }
    out
}

/// Repli des accents français → ASCII, pour que « cafe » trouve « café ».
pub(crate) fn fold_for_search(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let mapped = match ch {
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => {
                'a'
            }
            'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'e',
            'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'i',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'u',
            'Ý' | 'ÿ' | 'ý' => 'y',
            'Ç' | 'ç' => 'c',
            'Ñ' | 'ñ' => 'n',
            other => other,
        };
        for lower in mapped.to_lowercase() {
            out.push(lower);
        }
    }
    out
}

/// Expression SQL qui replie le corps avant LIKE (miroir de `fold_for_search`).
fn folded_body_sql() -> String {
    let mut expr = "body".to_string();
    for (from, to) in [
        ("É", "e"),
        ("È", "e"),
        ("Ê", "e"),
        ("Ë", "e"),
        ("é", "e"),
        ("è", "e"),
        ("ê", "e"),
        ("ë", "e"),
        ("À", "a"),
        ("Â", "a"),
        ("Ä", "a"),
        ("à", "a"),
        ("â", "a"),
        ("ä", "a"),
        ("Î", "i"),
        ("Ï", "i"),
        ("î", "i"),
        ("ï", "i"),
        ("Ô", "o"),
        ("Ö", "o"),
        ("ô", "o"),
        ("ö", "o"),
        ("Ù", "u"),
        ("Û", "u"),
        ("Ü", "u"),
        ("ù", "u"),
        ("û", "u"),
        ("ü", "u"),
        ("Ç", "c"),
        ("ç", "c"),
        ("Ñ", "n"),
        ("ñ", "n"),
        ("Ÿ", "y"),
        ("ÿ", "y"),
    ] {
        expr = format!("REPLACE({expr}, '{from}', '{to}')");
    }
    format!("LOWER({expr})")
}

#[derive(Debug)]
pub struct LedgerSearchOutcome {
    pub hits: Vec<LedgerEntry>,
    pub truncated: bool,
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

    pub fn search_messages(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<LedgerSearchOutcome, StoreError> {
        let limit = limit.clamp(1, MAX_LEDGER_SEARCH);
        let needles: Vec<String> = query
            .split_whitespace()
            .filter(|token| !token.is_empty())
            .map(|token| escape_like_needle(&fold_for_search(token)))
            .collect();
        if needles.is_empty() {
            return Ok(LedgerSearchOutcome {
                hits: Vec::new(),
                truncated: false,
            });
        }

        let folded = folded_body_sql();
        let mut sql = String::from("SELECT id, ts, sender, target, body FROM ledger WHERE 1=1");
        for _ in &needles {
            sql.push_str(&format!(" AND {folded} LIKE ? ESCAPE '\\'"));
        }
        // limit+1 pour détecter la troncature sans mensonge par omission.
        sql.push_str(" ORDER BY ts ASC, id ASC LIMIT ?");

        let mut stmt = self.conn.prepare(&sql).map_err(StoreError::Sqlite)?;
        let mut binds: Vec<rusqlite::types::Value> = needles
            .iter()
            .map(|needle| rusqlite::types::Value::Text(format!("%{needle}%")))
            .collect();
        binds.push(rusqlite::types::Value::Integer((limit as i64) + 1));
        let mut entries: Vec<LedgerEntry> = stmt
            .query_map(rusqlite::params_from_iter(binds), |row| {
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
            .filter_map(|row| row.ok())
            .collect();
        let truncated = entries.len() > limit;
        if truncated {
            entries.truncate(limit);
        }
        Ok(LedgerSearchOutcome {
            hits: entries,
            truncated,
        })
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
