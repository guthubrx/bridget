//! Remise idempotente et accusé aval.
//!
//! begin_send_delivery garde ledger + suivi + remise dans une transaction.
//! acknowledge_send_delivery garde issue + ACK + answered dans une transaction.
//! Déplacement de code uniquement : visibilité ledger ne signifie pas ACK.
use super::{
    DeliveryExecutionLink, IdempotencyError, IdempotencyKey, IdempotencyStore, MAX_CANONICAL_BYTES,
    OperationKind, OrphanedDeliveryNotice, ReplyTracking, SendDelivery, StoredSendDelivery,
};
use rusqlite::{OptionalExtension, TransactionBehavior, params};

impl IdempotencyStore {
    /// O(log N + G + B), G réservations de l'agent, B enveloppe. Un seul candidat.
    /// Le caller garde State et la preuve de repos jusqu'au COMMIT ; aucune E/S.
    pub(crate) fn release_next_held(
        &mut self,
        agent: &str,
        now: i64,
    ) -> Result<(Option<SendDelivery>, Option<OrphanedDeliveryNotice>), IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let control = crate::referent_control::read(&tx)?;
        if matches!(
            crate::referent_control::admit_autonomous_effect(
                crate::referent_control::AutonomousEffect::HeldRelease,
                &control
            ),
            crate::referent_control::Admission::Deferred { .. }
        ) {
            return Ok((None, None));
        }
        let pending_pause: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM control_pause_interruptions
             WHERE target_agent=?1 AND resumed_at IS NULL)",
            [agent],
            |r| r.get(0),
        )?;
        if pending_pause {
            return Ok((None, None));
        }
        // Seul le caller au repos peut expirer une réservation accusée ; >30s.
        tx.execute(
            "UPDATE send_deliveries SET turn_gate=0
            WHERE recipient_agent=?1 AND turn_gate=1 AND phase='acked'
              AND acked_at IS NOT NULL AND acked_at<?2",
            params![agent, now.saturating_sub(30)],
        )?;
        let gated: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM send_deliveries
            WHERE recipient_agent=?1 AND turn_gate=1)",
            [agent],
            |r| r.get(0),
        )?;
        if gated {
            return Ok((None, None));
        }
        let row = tx
            .query_row(
                "SELECT d.delivery_id,d.recipient_instance_id,
            d.delivery_generation,d.expires_at,d.message_bytes,r.expires_at,
            EXISTS(SELECT 1 FROM send_delivery_execution_links l WHERE l.delivery_id=d.delivery_id)
            FROM send_deliveries d JOIN idempotency_records r
              ON r.issuer_scope=d.issuer_scope AND r.operation_kind=d.operation_kind
                AND r.idempotency_key=d.idempotency_key
            WHERE d.recipient_agent=?1 AND d.phase='held'
            ORDER BY d.queue_rank LIMIT 1",
                [agent],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, u64>(2)?,
                        r.get::<_, i64>(3)?,
                        r.get::<_, Vec<u8>>(4)?,
                        r.get::<_, i64>(5)?,
                        r.get::<_, bool>(6)?,
                    ))
                },
            )
            .optional()?;
        let Some(row) = row else {
            tx.commit()?;
            return Ok((None, None));
        };
        if row.6 {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let message: bridget_core::BridgetMessage = serde_json::from_slice(&row.4)
            .map_err(|_| IdempotencyError::CorruptRecord("enveloppe retenue illisible"))?;
        let reason = if row.3 <= now || row.5 <= now {
            Some("expired_in_queue")
        } else if message.reply {
            let open: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM tracked_requests
                    WHERE id=?1 AND sender=?2 AND target=?3 AND state='open' AND deadline_at>?4)",
                params![message.id, message.from, message.to, now],
                |r| r.get(0),
            )?;
            (!open).then_some("request_closed_in_queue")
        } else {
            None
        };
        if let Some(reason) = reason {
            let notice = settle_held_in_transaction(&tx, &row.0, reason, now)?;
            tx.commit()?;
            return Ok((None, notice));
        }
        if tx.execute(
            "UPDATE send_deliveries SET phase='dispatching',turn_gate=1
            WHERE delivery_id=?1 AND phase='held' AND turn_gate=0",
            [&row.0],
        )? != 1
        {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let delivery = SendDelivery {
            delivery_id: row.0,
            recipient_instance_id: row.1,
            delivery_generation: row.2,
            expires_at: row.3,
            message_bytes: row.4,
        };
        tx.commit()?;
        Ok((Some(delivery), None))
    }

    /// O(log N + B + L), L dépôts liés. Réemploi de la transaction de retrait.
    pub(crate) fn settle_held(
        &mut self,
        id: &str,
        reason: &str,
        now: i64,
    ) -> Result<Option<OrphanedDeliveryNotice>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let notice = settle_held_in_transaction(&tx, id, reason, now)?;
        tx.commit()?;
        Ok(notice)
    }

    /// O(log N + G), G lignes de l'agent ; seul TurnState true attesté l'appelle.
    pub(crate) fn clear_held_turn_gate(&self, agent: &str) -> Result<(), IdempotencyError> {
        self.conn.execute(
            "UPDATE send_deliveries SET turn_gate=0
            WHERE recipient_agent=?1 AND turn_gate=1",
            [agent],
        )?;
        Ok(())
    }

    /// Métadonnées figées dans l'enveloppe existante, toutes phases.
    pub(crate) fn stored_send_message(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<serde_json::Value>, IdempotencyError> {
        self.stored_send_delivery(key)?
            .and_then(|delivery| delivery.message_bytes)
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .map_err(|_| IdempotencyError::CorruptRecord("enveloppe de remise illisible"))
            })
            .transpose()
    }

    /// Index existant idx_send_deliveries_kind_key : deux lignes au maximum.
    pub(crate) fn correlated_project_reason(
        &self,
        request_id: &str,
        sender: &str,
        target: &str,
    ) -> Result<Option<String>, IdempotencyError> {
        let mut stmt = self.conn.prepare("SELECT message_bytes FROM send_deliveries WHERE operation_kind = 'send' AND idempotency_key = ?1 AND phase IN ('dispatching','acked') LIMIT 2")?;
        let mut rows = stmt.query([request_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let bytes: Option<Vec<u8>> = row.get(0)?;
        let message = bytes
            .as_deref()
            .and_then(|bytes| serde_json::from_slice::<bridget_core::BridgetMessage>(bytes).ok());
        if rows.next()?.is_some() {
            return Ok(None);
        }
        Ok(message
            .filter(|m| m.reply && m.from == sender && m.to == target)
            .and_then(|m| m.cross_project_reason))
    }
    /// Fige la remise d'un envoi. Le changement d'état du socle, l'entrée
    /// `send_deliveries` et la **visibilité ledger** (fait d'émission) partagent
    /// une transaction SQLite. L'accusé (`acked`) reste un fait distinct :
    /// `outcome_unknown` reste nominal tant que `DeliverAcked` n'est pas reçu.
    pub fn begin_send_delivery(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_with_guidance(key, delivery, None, None, None, None, None)
    }

    /// Prépare atomiquement la remise et le suivi d'une réponse attendue.
    pub fn begin_send_delivery_with_reply(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: &ReplyTracking,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_with_guidance(key, delivery, Some(reply), None, None, None, None)
    }

    /// Prépare la remise et son rattachement causal dans la même transaction.
    ///
    /// Une panne ne peut donc jamais laisser une remise injectable sans le
    /// contexte d'exécution que le wrapper doit publier.
    pub fn begin_send_delivery_with_execution(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: Option<&ReplyTracking>,
        link: &DeliveryExecutionLink,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_with_guidance(key, delivery, reply, Some(link), None, None, None)
    }

    /// O(B+log n), B enveloppe, n lignes des magasins. Transaction unique réemployée.
    /// guidance=(at,actor,target,action,path,message_id), admis SEULEMENT au COMMIT.
    /// reply/lien gardent leurs types existants ; None conserve les appels historiques.
    /// held_for_agent choisit Held et fixe l'identité résolue ; None garde Dispatching.
    /// held_capacity borne ce mode dans la même transaction ; None garde les fixtures.
    pub(crate) fn begin_send_delivery_with_guidance(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: Option<&ReplyTracking>,
        execution_link: Option<&DeliveryExecutionLink>,
        guidance: Option<(i64, &str, &str, &str, &str, Option<&str>)>,
        held_capacity: Option<usize>,
        held_for_agent: Option<&str>,
    ) -> Result<(), IdempotencyError> {
        if key.operation_kind != OperationKind::Send || delivery.delivery_id.is_empty() {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if delivery.message_bytes.is_empty() || delivery.message_bytes.len() > MAX_CANONICAL_BYTES {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if execution_link.is_some_and(|link| {
            link.delivery_id != delivery.delivery_id
                || link.submission_id.is_empty()
                || link.execution_id.is_empty()
        }) {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if held_for_agent.is_some_and(str::is_empty)
            || (held_for_agent.is_some() && execution_link.is_some())
        {
            return Err(IdempotencyError::InvalidDelivery);
        }
        // Jamais de .ok() silencieux : une remise sans enveloppe lisible
        // recréerait la classe « invisible au ledger » que la gravure à
        // begin_send_delivery existe pour supprimer.
        let emitted_message =
            serde_json::from_slice::<bridget_core::BridgetMessage>(&delivery.message_bytes)
                .map_err(|_| IdempotencyError::CorruptRecord("enveloppe de remise illisible"))?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let (Some(agent), Some(capacity)) = (held_for_agent, held_capacity) {
            let count: i64 = tx.query_row(
                "SELECT COUNT(*) FROM send_deliveries
                 WHERE recipient_agent = ?1 AND phase = 'held'",
                [agent],
                |row| row.get(0),
            )?;
            if count < 0 || count as u64 >= capacity as u64 {
                return Err(IdempotencyError::HeldQueueFull);
            }
        }
        let transitioned = tx.execute(
            "UPDATE idempotency_records SET state = 'dispatching'
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2
               AND state = 'prepared'",
            params![key.issuer_scope, key.idempotency_key],
        )?;
        if transitioned != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        // La reprise de pause précède les retenus, y compris avant son premier true.
        // Réservation dans CE COMMIT, sans effet sur les appels legacy sans retenus.
        let resumed_gate = if let Some(link) = execution_link {
            let held: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM send_deliveries
                WHERE recipient_agent=?1 AND phase='held')",
                [&emitted_message.to],
                |r| r.get(0),
            )?;
            if held {
                tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM control_pause_interruptions
                WHERE resume_execution_id=?1 AND target_agent=?2)",
                    params![link.execution_id, emitted_message.to],
                    |r| r.get::<_, bool>(0),
                )?
            } else {
                false
            }
        } else {
            false
        };
        tx.execute(
            "INSERT INTO send_deliveries (
                delivery_id, issuer_scope, operation_kind, idempotency_key,
                recipient_instance_id, delivery_generation, phase, expires_at, message_bytes,
                recipient_agent, turn_gate
             ) VALUES (?1, ?2, 'send', ?3, ?4, ?5, ?8, ?6, ?7, ?9, ?10)",
            params![
                delivery.delivery_id,
                key.issuer_scope,
                key.idempotency_key,
                delivery.recipient_instance_id,
                delivery.delivery_generation,
                delivery.expires_at,
                delivery.message_bytes,
                if held_for_agent.is_some() {
                    "held"
                } else {
                    "dispatching"
                },
                held_for_agent.unwrap_or(&emitted_message.to),
                resumed_gate,
            ],
        )?;
        if let Some(link) = execution_link {
            tx.execute(
                "INSERT INTO send_delivery_execution_links
                    (delivery_id, submission_id, execution_id)
                 VALUES (?1, ?2, ?3)",
                params![link.delivery_id, link.submission_id, link.execution_id],
            )?;
        }
        let conversation_key = format!("{}|{}", emitted_message.from, emitted_message.to);
        crate::store::record_message_in_transaction(&tx, &emitted_message, &conversation_key)?;
        if let Some(reply) = reply {
            tx.execute(
                "INSERT INTO tracked_requests (
                    id, sender, target, state, created_at, deadline_at, escalation_level
                 ) VALUES (?1, ?2, ?3, 'open', ?4, ?5, 0)",
                params![
                    reply.request_id,
                    reply.sender,
                    reply.target,
                    reply.created_at,
                    reply.deadline_at,
                ],
            )?;
        }
        if let Some((at, actor, target, action, path, message_id)) = guidance {
            // Pas de compteur anticipé : toute erreur de guidage annule aussi
            // remise, ledger, suivi et lien causal ; aucun effet avant ce COMMIT.
            tx.execute(
                "INSERT INTO guidance_events(at,actor,target,action,path,message_id,outcome)
                 VALUES (?1,?2,?3,?4,?5,?6,'admitted')",
                params![at, actor, target, action, path, message_id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Attache une remise déjà gravée à son contexte causal Bridget.
    /// La première écriture est définitive : un rejeu ne peut pas rerouter la remise.
    pub fn link_send_delivery(&self, link: &DeliveryExecutionLink) -> Result<(), IdempotencyError> {
        if link.delivery_id.is_empty()
            || link.submission_id.is_empty()
            || link.execution_id.is_empty()
        {
            return Err(IdempotencyError::InvalidDelivery);
        }

        self.conn.execute(
            "INSERT INTO send_delivery_execution_links (delivery_id, submission_id, execution_id) VALUES (?1, ?2, ?3) ON CONFLICT(delivery_id) DO NOTHING",
            params![link.delivery_id, link.submission_id, link.execution_id],
        )?;

        match self.delivery_execution_link(&link.delivery_id)? {
            Some(existing) if existing == *link => Ok(()),
            Some(_) => Err(IdempotencyError::InvalidDelivery),
            None => Err(IdempotencyError::CorruptRecord(
                "lien causal de remise absent",
            )),
        }
    }

    /// Lit le rattachement causal sans modifier la remise ni son état de rejeu.
    pub fn delivery_execution_link(
        &self,
        delivery_id: &str,
    ) -> Result<Option<DeliveryExecutionLink>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id, submission_id, execution_id FROM send_delivery_execution_links WHERE delivery_id = ?1",
                [delivery_id],
                |row| {
                    Ok(DeliveryExecutionLink {
                        delivery_id: row.get(0)?,
                        submission_id: row.get(1)?,
                        execution_id: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Réassigne les remises encore en vol d'une instance morte vers l'instance
    /// vivante du même équipier. Ne touche ni `acked` ni `indeterminate`.
    pub fn reassign_dispatching_deliveries(
        &mut self,
        from_instance_id: &str,
        to_instance_id: &str,
        now: i64,
    ) -> Result<usize, IdempotencyError> {
        if from_instance_id.is_empty()
            || to_instance_id.is_empty()
            || from_instance_id == to_instance_id
        {
            return Ok(0);
        }
        // Session 102 : une alerte de fil reste liée à l'instance figée dans
        // sa réservation ; elle n'est jamais réaffectée à une autre instance.
        // L'enveloppe durable typée est inspectée en SQL (JSON1), en un seul
        // UPDATE transactionnel, sans relecture ligne à ligne.
        let updated = self.conn.execute(
            "UPDATE send_deliveries
             SET recipient_instance_id = ?1
             WHERE recipient_instance_id = ?2
               AND phase IN ('dispatching','held')
               AND expires_at > ?3
               AND (message_bytes IS NULL
                    OR json_valid(CAST(message_bytes AS TEXT)) = 0
                    OR json_extract(CAST(message_bytes AS TEXT), '$.thread_notice') IS NULL)",
            params![to_instance_id, from_instance_id, now],
        )?;
        Ok(updated)
    }

    /// Après Register attesté, rattache les retenus et, si activé, les envois ordinaires.
    /// O(log n + h log n + B), h lignes de l'agent, B enveloppes inspectées en SQL.
    /// ID, génération, octets, phase et rang conservés ; aucune injection ici.
    pub(crate) fn adopt_held_for_agent(
        &mut self,
        agent_id: &str,
        instance_id: &str,
        recover_dispatching: bool,
    ) -> Result<usize, IdempotencyError> {
        if agent_id.is_empty() || instance_id.is_empty() {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .min(i64::MAX as u64) as i64;
        self.conn
            .execute(
                "UPDATE send_deliveries SET recipient_instance_id = ?2,
                    turn_gate = CASE WHEN phase = 'dispatching' THEN 1 ELSE turn_gate END
                 WHERE recipient_agent = ?1 AND (phase = 'held' OR
                    (?7 AND phase = 'dispatching' AND recipient_instance_id <> ?2 AND expires_at > ?3
                     AND issuer_scope NOT IN (?4, ?5, ?6)
                     AND (message_bytes IS NULL
                          OR json_valid(CAST(message_bytes AS TEXT)) = 0
                          OR json_extract(CAST(message_bytes AS TEXT), '$.thread_notice') IS NULL)))",
                params![agent_id, instance_id, now, crate::wake_schedules::CLI_WAKE_SCOPE,
                    crate::wake_schedules::SCHEDULED_WAKE_SCOPE, crate::wake_schedules::MISSED_SUMMARY_SCOPE,
                    recover_dispatching],
            )
            .map_err(Into::into)
    }

    /// Finalise un refus de clé neuve sans rendre l'état intermédiaire
    /// `Dispatching` observable à travers un crash ou une autre connexion.
    pub fn reject_prepared(
        &mut self,
        key: &IdempotencyKey,
        category: &str,
        reason: &str,
    ) -> Result<(), IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let dispatched = tx.execute(
            "UPDATE idempotency_records SET state = 'dispatching'
             WHERE issuer_scope = ?1 AND operation_kind = ?2 AND idempotency_key = ?3
               AND state = 'prepared'",
            params![
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
            ],
        )?;
        if dispatched != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let terminal = tx.execute(
            "UPDATE idempotency_records
             SET state = 'terminal', public_result_kind = 'rejected',
                 public_result_category = ?1, public_result_reason = ?2
             WHERE issuer_scope = ?3 AND operation_kind = ?4 AND idempotency_key = ?5
               AND state = 'dispatching'",
            params![
                category,
                reason,
                key.issuer_scope,
                key.operation_kind.as_str(),
                key.idempotency_key,
            ],
        )?;
        if terminal != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.commit()?;
        Ok(())
    }

    /// Remise ENCORE EN VOL pour cette clé, et elle seule.
    ///
    /// Le filtre de phase n'est pas cosmétique : une remise passée en
    /// `indeterminate` — par échec de reprise, par `DeliveryIndeterminate`, ou
    /// par la migration v2 qui écarte les enveloppes absentes — est un état
    /// ABSORBANT. Plus rien ne l'accusera jamais, et elle occupe pourtant sa
    /// clé jusqu'à l'expiration de l'horizon (7 jours). Sans ce filtre, elle
    /// remontait comme une remise en vol : l'appelant lisait « le dépôt a
    /// réussi, rejouez pour lire le sort », rc=0, sur un message qui ne
    /// partirait plus jamais. Le mensonge optimiste est pire que celui qu'on
    /// corrige — d'où `dispatching` seul.
    ///
    /// Complexité : O(log n) via l'index unique de la clé idempotente.
    /// `orphaned` est aussi absorbant : le destinataire a été purgé, le sort
    /// est connu. Ne pas le confondre avec une remise en vol.
    pub fn send_delivery(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        let Some(delivery) = self.stored_send_delivery(key)? else {
            return Ok(None);
        };
        if delivery.phase != "dispatching" {
            return Ok(None);
        }
        Ok(delivery.into_delivery())
    }

    /// Phase brute (`dispatching`, `acked`, `indeterminate`, `orphaned`) de la
    /// remise d'une clé, `None` si la ligne n'existe pas. Contrairement à
    /// `send_delivery`, aucune phase n'est masquée : la rétention de la
    /// délégation native (session 150) distingue « accusée » de « effacée ».
    ///
    /// Complexité : O(log n) via l'index unique de la clé idempotente.
    pub(crate) fn send_delivery_phase(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<String>, IdempotencyError> {
        Ok(self
            .stored_send_delivery(key)?
            .map(|delivery| delivery.phase))
    }

    pub(crate) fn stored_send_delivery(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<StoredSendDelivery>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at,
                        phase, message_bytes
                 FROM send_deliveries
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2",
                params![key.issuer_scope, key.idempotency_key],
                |row| {
                    Ok(StoredSendDelivery {
                        delivery_id: row.get(0)?,
                        recipient_instance_id: row.get(1)?,
                        delivery_generation: row.get(2)?,
                        expires_at: row.get(3)?,
                        phase: row.get(4)?,
                        message_bytes: row.get(5)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Mutant livré : retire uniquement la garde de phase, sans modifier la
    /// projection nullable. Hors chemin de production par construction.
    #[cfg(test)]
    pub(super) fn send_delivery_mutant_sans_filtre_de_phase(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        Ok(self
            .stored_send_delivery(key)?
            .and_then(StoredSendDelivery::into_delivery))
    }

    /// Identifiant de remise orpheline (phase `orphaned`), pour le rejeu honnête.
    pub fn orphaned_delivery_id(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<String>, IdempotencyError> {
        self.conn
            .query_row(
                "SELECT delivery_id FROM send_deliveries
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2
                   AND phase = 'orphaned'",
                params![key.issuer_scope, key.idempotency_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    /// O(log n+c log c+c log r+B), c candidats de cette instance, r reçus, B octets lus.
    /// Lecture seule par rang durable ; au plus 200 résultats, sans limiter les candidats examinés.
    /// Les files éphémères et le Send historique ne sont pas représentés ici.
    pub(crate) fn queued_deliveries_for_recipient(
        &self,
        recipient_instance_id: &str,
        now: i64,
        limit: usize,
        include_held: bool,
    ) -> Result<Vec<(SendDelivery, i64, bool)>, IdempotencyError> {
        let mut query = self.conn.prepare(
            "SELECT d.delivery_id,d.recipient_instance_id,d.delivery_generation,
                    d.expires_at,d.message_bytes,r.issued_at,d.phase='held'
             FROM send_deliveries d
             JOIN idempotency_records r
               ON r.issuer_scope=d.issuer_scope
              AND r.operation_kind=d.operation_kind
              AND r.idempotency_key=d.idempotency_key
             WHERE d.recipient_instance_id=?1
               AND (d.phase='dispatching' OR (?4 AND d.phase='held')) AND d.expires_at>?2
             ORDER BY d.queue_rank LIMIT ?3",
        )?;
        let rows = query
            .query_map(
                params![
                    recipient_instance_id,
                    now,
                    limit.min(200) as i64,
                    include_held
                ],
                |row| {
                    Ok((
                        SendDelivery {
                            delivery_id: row.get(0)?,
                            recipient_instance_id: row.get(1)?,
                            delivery_generation: row.get(2)?,
                            expires_at: row.get(3)?,
                            message_bytes: row.get(4)?,
                        },
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    /// Reprise bornée : seules les remises encore en cours pour l'instance
    /// exacte sont relivrées. Une remise Acked ou Indeterminate ne l'est pas.
    /// Toute ligne `dispatching` sans enveloppe est classée `indeterminate`
    /// dans la même transaction avant que les autres remises soient rendues.
    ///
    /// O(log N+K log K+B), K remises de cette instance, B enveloppes ; index156
    /// recipient/phase puis tri delivery_id, reclassement groupé.
    pub fn dispatching_deliveries_for_instance(
        &mut self,
        recipient_instance_id: &str,
        now: i64,
    ) -> Result<Vec<SendDelivery>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let reclassified = tx.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate', turn_gate = 0
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching'
               AND message_bytes IS NULL",
            params![recipient_instance_id],
        )?;
        let stored = {
            let mut statement = tx.prepare(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at, message_bytes
                 FROM send_deliveries
                 WHERE recipient_instance_id = ?1 AND phase = 'dispatching' AND expires_at > ?2
                 ORDER BY queue_rank",
            )?;
            statement
                .query_map(params![recipient_instance_id, now], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, u64>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, Option<Vec<u8>>>(4)?,
                    ))
                })?
                .collect::<Result<Vec<_>, _>>()?
        };
        let deliveries = stored
            .into_iter()
            .map(
                |(
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message_bytes,
                )| {
                    let message_bytes = message_bytes.ok_or(IdempotencyError::CorruptRecord(
                        "remise dispatching sans enveloppe après reclassement",
                    ))?;
                    Ok(SendDelivery {
                        delivery_id,
                        recipient_instance_id,
                        delivery_generation,
                        expires_at,
                        message_bytes,
                    })
                },
            )
            .collect::<Result<Vec<_>, IdempotencyError>>()?;
        tx.commit()?;
        if reclassified > 0 {
            log::warn!(
                "{reclassified} remise(s) de {recipient_instance_id} classée(s) indeterminate: enveloppe locale absente"
            );
        }
        Ok(deliveries)
    }

    /// Recherche une remise encore rejouable pour une exécution et une
    /// instance exactes. Une remise accusée ou expirée n'empêche pas la
    /// reconstruction d'une continuation.
    /// Recherche une remise liée même si le daemon a perdu la présence qui
    /// permettait de connaître l'ancien instance_id après un redémarrage.
    /// Deux remises actives pour la même exécution sont une corruption fermée.
    pub fn dispatching_delivery_for_execution_any_instance(
        &self,
        execution_id: &str,
        now: i64,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        if execution_id.trim().is_empty() {
            return Ok(None);
        }
        let mut statement = self.conn.prepare(
            "SELECT delivery.delivery_id, delivery.recipient_instance_id,
                    delivery.delivery_generation, delivery.expires_at,
                    delivery.message_bytes
             FROM send_deliveries delivery
             JOIN send_delivery_execution_links link
               ON link.delivery_id = delivery.delivery_id
             WHERE link.execution_id = ?1
               AND delivery.phase = 'dispatching'
               AND delivery.expires_at > ?2
             ORDER BY delivery.delivery_id
             LIMIT 2",
        )?;
        let deliveries = statement
            .query_map(params![execution_id, now], |row| {
                Ok(SendDelivery {
                    delivery_id: row.get(0)?,
                    recipient_instance_id: row.get(1)?,
                    delivery_generation: row.get(2)?,
                    expires_at: row.get(3)?,
                    message_bytes: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        match deliveries.as_slice() {
            [] => Ok(None),
            [delivery] => Ok(Some(delivery.clone())),
            _ => Err(IdempotencyError::CorruptRecord(
                "plusieurs remises actives pour une exécution",
            )),
        }
    }

    pub fn dispatching_delivery_for_execution(
        &self,
        execution_id: &str,
        recipient_instance_id: &str,
        now: i64,
    ) -> Result<Option<SendDelivery>, IdempotencyError> {
        if execution_id.trim().is_empty() || recipient_instance_id.trim().is_empty() {
            return Ok(None);
        }
        self.conn
            .query_row(
                "SELECT delivery.delivery_id, delivery.recipient_instance_id,
                        delivery.delivery_generation, delivery.expires_at,
                        delivery.message_bytes
                 FROM send_deliveries delivery
                 JOIN send_delivery_execution_links link
                   ON link.delivery_id = delivery.delivery_id
                 WHERE link.execution_id = ?1
                   AND delivery.recipient_instance_id = ?2
                   AND delivery.phase = 'dispatching'
                   AND delivery.expires_at > ?3",
                params![execution_id, recipient_instance_id, now],
                |row| {
                    Ok(SendDelivery {
                        delivery_id: row.get(0)?,
                        recipient_instance_id: row.get(1)?,
                        delivery_generation: row.get(2)?,
                        expires_at: row.get(3)?,
                        message_bytes: row.get(4)?,
                    })
                },
            )
            .optional()
            .map_err(Into::into)
    }

    /// Une marque `Seen` sans observable d'injection reste indéterminée : le
    /// daemon conserve OutcomeUnknown jusqu'à l'expiration et ne réinjecte pas.
    pub fn mark_delivery_indeterminate(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
    ) -> Result<(), IdempotencyError> {
        let updated = self.conn.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate', turn_gate = 0
             WHERE delivery_id = ?1 AND recipient_instance_id = ?2
               AND delivery_generation = ?3 AND phase = 'dispatching'",
            params![delivery_id, recipient_instance_id, delivery_generation],
        )?;
        if updated == 1 {
            Ok(())
        } else {
            Err(IdempotencyError::InvalidDelivery)
        }
    }

    /// Session 116 : une remise encore `dispatching` après l'expiration de sa
    /// saga ne sera plus jamais accusée — son destinataire a disparu sans que
    /// sa présence soit purgée. Son sort reste inconnu : `indeterminate` le dit,
    /// au lieu d'afficher indéfiniment « en vol » dans le journal des échanges.
    /// Aucune remise vivante n'est touchée : une saga expirée n'est plus rejouée.
    pub fn settle_expired_dispatching(&self, now: i64) -> Result<usize, IdempotencyError> {
        self.conn
            .execute(
                "UPDATE send_deliveries SET phase = 'indeterminate', turn_gate = 0
                 WHERE phase = 'dispatching' AND expires_at <= ?1",
                params![now],
            )
            .map_err(Into::into)
    }

    /// Session 116 : purge les envois dont la saga a expiré depuis plus de
    /// `grace` secondes ; leurs remises suivent en cascade.
    ///
    /// Seuls les envois sont visés. Un lancement d'équipier géré porte son
    /// historique dans `spawn_commands`, lui aussi supprimé en cascade avec son
    /// enregistrement : `purge_expired`, qui ne distingue pas les opérations,
    /// l'effacerait. La marge protège le contrat d'idempotence : pendant elle, un
    /// rejeu tardif reçoit encore « expiré » au lieu d'être réexpédié.
    pub fn purge_expired_sends(&self, now: i64, grace: i64) -> Result<usize, IdempotencyError> {
        self.conn
            .execute(
                "DELETE FROM idempotency_records
                 WHERE operation_kind = 'send' AND expires_at <= ?1",
                params![now.saturating_sub(grace)],
            )
            .map_err(Into::into)
    }

    /// Après purge d'une présence : les remises encore `dispatching` pour cette
    /// instance deviennent `orphaned` (sort CONNU) et le socle passe en
    /// terminal `orphaned`. Distinct de `indeterminate` (quarantaine d'injection)
    /// et de `outcome_unknown` (sort réellement inconnu).
    pub fn orphan_dispatching_for_instance(
        &mut self,
        recipient_instance_id: &str,
        reason: &str,
    ) -> Result<Vec<OrphanedDeliveryNotice>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut statement = tx.prepare(
            "SELECT delivery_id, issuer_scope, idempotency_key, message_bytes
             FROM send_deliveries
             WHERE recipient_instance_id = ?1 AND phase IN ('dispatching','held')",
        )?;
        let rows: Vec<(String, String, String, Vec<u8>)> = statement
            .query_map(params![recipient_instance_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?.unwrap_or_default(),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);

        let mut notices = Vec::with_capacity(rows.len());
        for (delivery_id, issuer_scope, idempotency_key, message_bytes) in rows {
            let message =
                serde_json::from_slice::<bridget_core::BridgetMessage>(&message_bytes).ok();
            let (message_id, sender, target) = match &message {
                Some(msg) => (msg.id.clone(), msg.from.clone(), msg.to.clone()),
                None => (
                    delivery_id.clone(),
                    "inconnu".to_string(),
                    "inconnu".to_string(),
                ),
            };
            let delivery = tx.execute(
                "UPDATE send_deliveries SET phase = 'orphaned', turn_gate = 0
                 WHERE delivery_id = ?1 AND phase IN ('dispatching','held')",
                params![delivery_id],
            )?;
            let record = tx.execute(
                "UPDATE idempotency_records
                 SET state = 'terminal', public_result_kind = 'orphaned',
                     public_result_category = NULL, public_result_reason = ?1
                 WHERE issuer_scope = ?2 AND operation_kind = 'send'
                   AND idempotency_key = ?3 AND state = 'dispatching'",
                params![reason, issuer_scope, idempotency_key],
            )?;
            if delivery != 1 || record != 1 {
                return Err(IdempotencyError::DispatchUnavailable);
            }
            // Conduite (pas seulement le constat) : `orphaned` est absorbant —
            // le réflexe REJEU_A_L_IDENTIQUE enseigné pour in_flight/outcome_unknown
            // ferait tourner l'émetteur en rond. Aligné sur mcp::CONDUITE_ORPHELIN.
            let body = format!(
                "ORPHELIN: le message {} destiné à {} n'a pas été livré — présence purgée (delivery {}). {}\n\
                 Le rejeu à l'identique ne sert à rien — cette clé est close. \
                 Change de destinataire, ou attends son retour avec une clé neuve.",
                message_id, target, delivery_id, reason
            );
            // Même transaction : le signal survit à un crash avant le Deliver.
            tx.execute(
                "INSERT INTO orphan_emitter_notices (delivery_id, sender, body, created_at, notified_at)
                 VALUES (?1, ?2, ?3, strftime('%s','now'), NULL)
                 ON CONFLICT(delivery_id) DO NOTHING",
                params![delivery_id, sender, body],
            )?;
            notices.push(OrphanedDeliveryNotice {
                delivery_id,
                message_id,
                sender,
                target,
                reason: reason.to_string(),
            });
        }
        tx.commit()?;
        Ok(notices)
    }

    /// Notices émetteur encore non poussées (crash entre orphan et Deliver).
    pub fn pending_orphan_emitter_notices(
        &self,
    ) -> Result<Vec<(String, String, String)>, IdempotencyError> {
        let mut statement = self.conn.prepare(
            "SELECT delivery_id, sender, body FROM orphan_emitter_notices
             WHERE notified_at IS NULL ORDER BY created_at, delivery_id",
        )?;
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Marque la notice comme émise sur la socket (`writeln!`+`flush` Ok).
    /// Limite : ce n'est pas une attestation de réception par le wrapper.
    pub fn mark_orphan_emitter_notified(
        &self,
        delivery_id: &str,
        now: i64,
    ) -> Result<(), IdempotencyError> {
        self.conn.execute(
            "UPDATE orphan_emitter_notices SET notified_at = ?1
             WHERE delivery_id = ?2 AND notified_at IS NULL",
            params![now, delivery_id],
        )?;
        Ok(())
    }

    /// Accusé aval : la remise et le résultat public deviennent terminaux dans
    /// une même transaction, après validation de l'instance et génération.
    /// Le ledger d'émission a déjà été gravé à `begin_send_delivery` — ici on
    /// ne fait que soldater l'accusé et le cycle de réponse, sans réécrire le
    /// fait d'émission (visibilité ≠ accusé).
    pub fn acknowledge_send_delivery(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
    ) -> Result<Option<String>, IdempotencyError> {
        self.acknowledge_send_delivery_at(
            delivery_id,
            recipient_instance_id,
            delivery_generation,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        )
    }

    pub(crate) fn acknowledge_send_delivery_at(
        &mut self,
        delivery_id: &str,
        recipient_instance_id: &str,
        delivery_generation: u64,
        now: i64,
    ) -> Result<Option<String>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let row = tx.query_row(
            "SELECT issuer_scope, idempotency_key, recipient_instance_id, delivery_generation, phase, message_bytes
             FROM send_deliveries WHERE delivery_id = ?1",
            params![delivery_id],
            |row| Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<Vec<u8>>>(5)?,
            )),
        ).optional()?.ok_or(IdempotencyError::InvalidDelivery)?;
        if row.2 != recipient_instance_id || row.3 != delivery_generation {
            return Err(IdempotencyError::InvalidDelivery);
        }
        if row.4 == "acked" {
            tx.commit()?;
            return Ok(None);
        }
        if row.4 != "dispatching" {
            return Err(IdempotencyError::InvalidDelivery);
        }
        let envelope_missing = row.5.is_none();
        let message = row
            .5
            .as_deref()
            .and_then(|bytes| serde_json::from_slice::<bridget_core::BridgetMessage>(bytes).ok());
        let delivery = tx.execute("UPDATE send_deliveries SET phase = 'acked', acked_at = COALESCE(acked_at, ?2) WHERE delivery_id = ?1 AND phase = 'dispatching'", params![delivery_id, now])?;
        let record = tx.execute(
            "UPDATE idempotency_records SET state = 'terminal', public_result_kind = 'accepted', public_result_category = NULL, public_result_reason = NULL
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2 AND state = 'dispatching'",
            params![row.0, row.1],
        )?;
        if delivery != 1 || record != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        let answered_request = if let Some(message) = message {
            if let Some(request_id) = message.in_reply_to.as_deref() {
                let reply_summary = crate::store::request_reply_summary(&message);
                let changed = crate::store::mark_answered_in_transaction(
                    &tx,
                    request_id,
                    &message.from,
                    &message.to,
                    Some(&reply_summary),
                )?;
                changed.then(|| request_id.to_string())
            } else {
                None
            }
        } else {
            None
        };
        tx.commit()?;
        if envelope_missing {
            log::warn!(
                "accusé idempotent {delivery_id} enregistré sans enveloppe locale: corrélation in_reply_to impossible"
            );
        }
        Ok(answered_request)
    }
}

/// FR006/FR024 : réutilisé par libération et balayage, jamais deux COMMIT.
/// O(log N+log R+B+L), R demandes, B enveloppe, L dépôts liés ; DTO/table existants.
fn settle_held_in_transaction(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    reason: &str,
    now: i64,
) -> Result<Option<OrphanedDeliveryNotice>, IdempotencyError> {
    if !matches!(
        reason,
        "expired_in_queue" | "request_closed_in_queue" | "cancelled_in_queue"
    ) {
        return Err(IdempotencyError::InvalidDelivery);
    }
    let row = tx
        .query_row(
            "SELECT d.issuer_scope,d.idempotency_key,d.message_bytes,
            d.expires_at,r.expires_at
        FROM send_deliveries d JOIN idempotency_records r
          ON r.issuer_scope=d.issuer_scope AND r.operation_kind=d.operation_kind
            AND r.idempotency_key=d.idempotency_key
        WHERE d.delivery_id=?1 AND d.phase='held'",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Vec<u8>>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            },
        )
        .optional()?;
    let Some((scope, key, bytes, delivery_expires, record_expires)) = row else {
        return Ok(None);
    };
    if reason == "expired_in_queue" && delivery_expires > now && record_expires > now {
        return Ok(None);
    }
    let message: bridget_core::BridgetMessage = serde_json::from_slice(&bytes)
        .map_err(|_| IdempotencyError::CorruptRecord("enveloppe retenue illisible"))?;
    if reason == "request_closed_in_queue" {
        if !message.reply {
            return Ok(None);
        }
        let open: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM tracked_requests
             WHERE id=?1 AND sender=?2 AND target=?3 AND state='open' AND deadline_at>?4)",
            params![message.id, message.from, message.to, now],
            |row| row.get(0),
        )?;
        if open {
            return Ok(None);
        }
    }
    if tx.execute(
        "UPDATE send_deliveries SET phase='cancelled',turn_gate=0
        WHERE delivery_id=?1 AND phase='held'",
        [id],
    )? != 1
    {
        return Err(IdempotencyError::DispatchUnavailable);
    }
    let body = format!(
        "RETIRÉ DE LA FILE : votre message {} pour {} a été retiré avant livraison ({}). Il ne sera pas livré.",
        message.id, message.to, reason
    );
    if tx.execute("UPDATE idempotency_records SET state='terminal',public_result_kind='rejected',
        public_result_category=?1,public_result_reason=?2
        WHERE issuer_scope=?3 AND operation_kind='send' AND idempotency_key=?4 AND state='dispatching'",
        params![reason,body,scope,key])? != 1 { return Err(IdempotencyError::DispatchUnavailable); }
    if message.reply {
        let expired: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM tracked_requests
             WHERE id=?1 AND sender=?2 AND target=?3 AND state='open' AND deadline_at<=?4)",
            params![message.id, message.from, message.to, now],
            |row| row.get(0),
        )?;
        let request_state = if expired { "timed_out" } else { "cancelled" };
        let lifecycle = if expired {
            bridget_transport::protocol::GuichetLifecycleState::TimedOut
        } else {
            bridget_transport::protocol::GuichetLifecycleState::Cancelled
        };
        let changed = tx.execute(
            "UPDATE tracked_requests SET state=?1,cancel_reason=?2,completed_at=?3
            WHERE id=?4 AND sender=?5 AND target=?6 AND state='open'",
            params![
                request_state,
                if expired { None } else { Some(reason) },
                now,
                message.id,
                message.from,
                message.to
            ],
        )?;
        if changed == 1 {
            crate::store::record_lifecycle_for_linked_request_in_transaction(
                tx,
                &message.id,
                lifecycle,
                now,
            )
            .map_err(|e| match e {
                crate::store::StoreError::Sqlite(e) => IdempotencyError::Sqlite(e),
                _ => IdempotencyError::CorruptRecord("cycle de demande retenue illisible"),
            })?;
        }
    }
    tx.execute(
        "INSERT INTO orphan_emitter_notices(delivery_id,sender,body,created_at,notified_at)
        VALUES(?1,?2,?3,?4,NULL) ON CONFLICT(delivery_id) DO NOTHING",
        params![id, message.from, body, now],
    )?;
    Ok(Some(OrphanedDeliveryNotice {
        delivery_id: id.into(),
        message_id: message.id,
        sender: message.from,
        target: message.to,
        reason: reason.into(),
    }))
}

#[cfg(test)]
#[test]
fn held159_rank_monotone_sous_concurrence() {
    let path = std::env::temp_dir().join(format!("held159-rank-{}.db", uuid::Uuid::new_v4()));
    let store = IdempotencyStore::open(&path).unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
    let workers=(0..4).map(|index| {
        let path=path.clone(); let barrier=barrier.clone();
        std::thread::spawn(move || {
            let mut store=IdempotencyStore::open(&path).unwrap();
            let id=format!("held159-concurrent-{index}");
            let key=IdempotencyKey::new("012_scope_held159_aaaaaaaa",OperationKind::Send,&id).unwrap();
            let message=bridget_core::BridgetMessage::new("author159",format!("agent159-{index}"),&id);
            let bytes=serde_json::to_vec(&message).unwrap();
            store.reserve(&key,&bytes,1_000_000,3600,1_000_000,30).unwrap();
            barrier.wait();
            let delivery=SendDelivery { delivery_id:id.clone(),recipient_instance_id:format!("instance159-{index}"),delivery_generation:1,expires_at:1_003_600,message_bytes:bytes };
            store.begin_send_delivery_with_guidance(&key,&delivery,None,None,None, None,Some(&message.to)).unwrap();
            let (rank,phase,recipient):(i64,String,String)=store.conn.query_row("SELECT queue_rank,phase,recipient_agent FROM send_deliveries WHERE delivery_id=?1",[&id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
            assert_eq!(phase,"held"); assert_eq!(recipient,message.to);
            assert!(store.delivery_execution_link(&id).unwrap().is_none()); rank
        })
    }).collect::<Vec<_>>();
    let mut ranks = workers
        .into_iter()
        .map(|w| w.join().unwrap())
        .collect::<Vec<_>>();
    ranks.sort();
    assert_eq!(ranks.len(), 4);
    assert!(ranks.windows(2).all(|w| w[0] < w[1]));
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT queue_rank) FROM send_deliveries",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        4
    );
    drop(store);
    std::fs::remove_file(&path).unwrap();
}

#[cfg(test)]
#[test]
fn held159_rank_survit_a_la_reaffectation() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let entries = [
        ("z-held", "old159", "held"),
        ("a-live", "old159", "dispatching"),
    ];
    for (index, (id, instance, phase)) in entries.iter().enumerate() {
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, *id).unwrap();
        let mut message = bridget_core::BridgetMessage::new("author159", "agent159", *id);
        message.id = (*id).into();
        let bytes = serde_json::to_vec(&message).unwrap();
        store
            .reserve(&key, &bytes, 1_000_000, 3600, 1_000_000, 30)
            .unwrap();
        store
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        store.conn.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?3,?4,?5,?6,1003600,?7,'agent159')",
            params![id,key.issuer_scope,key.idempotency_key,instance,index as i64+1,phase,bytes]).unwrap();
    }
    let before = store.conn.prepare("SELECT delivery_id,queue_rank,recipient_instance_id,phase,message_bytes FROM send_deliveries ORDER BY queue_rank").unwrap().query_map([],|row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,Vec<u8>>(4)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(
        store
            .reassign_dispatching_deliveries("old159", "new159", 1_000_000)
            .unwrap(),
        2
    );
    let after = store.conn.prepare("SELECT delivery_id,queue_rank,recipient_instance_id,phase,message_bytes FROM send_deliveries ORDER BY queue_rank").unwrap().query_map([],|row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,Vec<u8>>(4)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    for (before, after) in before.iter().zip(&after) {
        assert_eq!(
            (&before.0, before.1, &before.3, &before.4),
            (&after.0, after.1, &after.3, &after.4)
        );
        assert_eq!(after.2, "new159");
    }
}

#[cfg(test)]
#[test]
fn held159_rank_reprise_dans_l_ordre() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let entries = [
        ("z-first", "same159", "dispatching"),
        ("a-second", "same159", "dispatching"),
    ];
    for (index, (id, instance, phase)) in entries.iter().enumerate() {
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, *id).unwrap();
        let mut message = bridget_core::BridgetMessage::new("author159", "agent159", *id);
        message.id = (*id).into();
        let bytes = serde_json::to_vec(&message).unwrap();
        store
            .reserve(&key, &bytes, 1_000_000, 3600, 1_000_000, 30)
            .unwrap();
        store
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        store.conn.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?3,?4,?5,?6,1003600,?7,'agent159')",
            params![id,key.issuer_scope,key.idempotency_key,instance,index as i64+1,phase,bytes]).unwrap();
    }
    let recovered = store
        .dispatching_deliveries_for_instance("same159", 1_000_000)
        .unwrap();
    assert_eq!(
        recovered
            .iter()
            .map(|d| d.delivery_id.as_str())
            .collect::<Vec<_>>(),
        vec!["z-first", "a-second"]
    );
    for delivery in recovered {
        let original: Vec<u8> = store
            .conn
            .query_row(
                "SELECT message_bytes FROM send_deliveries WHERE delivery_id=?1",
                [&delivery.delivery_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(delivery.message_bytes, original);
    }
}

#[cfg(test)]
#[test]
fn held159_rank_sans_collision_entre_instances() {
    let store = IdempotencyStore::open_in_memory().unwrap();
    let entries = [
        ("z-one", "instance159a", "held"),
        ("a-two", "instance159b", "held"),
        ("m-three", "instance159a", "dispatching"),
    ];
    for (index, (id, instance, phase)) in entries.iter().enumerate() {
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, *id).unwrap();
        let mut message = bridget_core::BridgetMessage::new("author159", "agent159", *id);
        message.id = (*id).into();
        let bytes = serde_json::to_vec(&message).unwrap();
        store
            .reserve(&key, &bytes, 1_000_000, 3600, 1_000_000, 30)
            .unwrap();
        store
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        store.conn.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?3,?4,?5,?6,1003600,?7,'agent159')",
            params![id,key.issuer_scope,key.idempotency_key,instance,index as i64+1,phase,bytes]).unwrap();
    }
    let rows=store.conn.prepare("SELECT delivery_id,queue_rank,recipient_instance_id,phase,message_bytes FROM send_deliveries ORDER BY queue_rank").unwrap().query_map([],|row| Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,Vec<u8>>(4)?))).unwrap().collect::<Result<Vec<_>,_>>().unwrap();
    assert_eq!(
        rows.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        vec!["z-one", "a-two", "m-three"]
    );
    assert!(rows.windows(2).all(|w| w[0].1 < w[1].1));
    assert_eq!(
        rows.iter()
            .map(|r| r.1)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3
    );
}

#[cfg(test)]
#[test]
fn held159_rank_orphelinage_couvre_les_retenus() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let entries = [
        ("held-orphan159", "gone159", "held"),
        ("live-orphan159", "gone159", "dispatching"),
    ];
    for (index, (id, instance, phase)) in entries.iter().enumerate() {
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, *id).unwrap();
        let mut message = bridget_core::BridgetMessage::new("author159", "agent159", *id);
        message.id = (*id).into();
        let bytes = serde_json::to_vec(&message).unwrap();
        store
            .reserve(&key, &bytes, 1_000_000, 3600, 1_000_000, 30)
            .unwrap();
        store
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        store.conn.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?3,?4,?5,?6,1003600,?7,'agent159')",
            params![id,key.issuer_scope,key.idempotency_key,instance,index as i64+1,phase,bytes]).unwrap();
    }
    let notices = store
        .orphan_dispatching_for_instance("gone159", "instance disparue159")
        .unwrap();
    assert_eq!(notices.len(), 2);
    let durable = store.pending_orphan_emitter_notices().unwrap();
    assert_eq!(durable.len(), 2, "chaque auteur reçoit sa notice durable");
    for (id, _, _) in entries {
        let phase: String = store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
                [id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(phase, "orphaned");
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, id).unwrap();
        assert!(matches!(
            store.lookup(&key, 1_000_000).unwrap(),
            crate::idempotency::LookupResult::Orphaned { .. }
        ));
        assert!(
            durable
                .iter()
                .any(|notice| notice.0 == id && notice.1 == "author159" && notice.2.contains(id))
        );
    }
}

#[cfg(test)]
#[test]
fn held159_rank_reprise_ne_rend_pas_les_retenus() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let entries = [
        ("a-held159", "same159", "held"),
        ("z-dispatch159", "same159", "dispatching"),
    ];
    for (index, (id, instance, phase)) in entries.iter().enumerate() {
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, *id).unwrap();
        let mut message = bridget_core::BridgetMessage::new("author159", "agent159", *id);
        message.id = (*id).into();
        let bytes = serde_json::to_vec(&message).unwrap();
        store
            .reserve(&key, &bytes, 1_000_000, 3600, 1_000_000, 30)
            .unwrap();
        store
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        store.conn.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?3,?4,?5,?6,1003600,?7,'agent159')",
            params![id,key.issuer_scope,key.idempotency_key,instance,index as i64+1,phase,bytes]).unwrap();
    }
    let recovered = store
        .dispatching_deliveries_for_instance("same159", 1_000_000)
        .unwrap();
    assert_eq!(
        recovered
            .iter()
            .map(|d| d.delivery_id.as_str())
            .collect::<Vec<_>>(),
        vec!["z-dispatch159"]
    );
    assert_eq!(
        store
            .conn
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id='a-held159'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "held"
    );
}

#[cfg(test)]
#[test]
fn held159_acked_at_premier_ack_et_rejeu() {
    let mut store = IdempotencyStore::open_in_memory().unwrap();
    let key = IdempotencyKey::new(
        "012_scope_held159_release",
        OperationKind::Send,
        "ack-clock",
    )
    .unwrap();
    let now = 100_000;
    let message = bridget_core::BridgetMessage::new("peer-a", "peer-b", "premier ACK durable");
    let bytes = serde_json::to_vec(&message).unwrap();
    store.reserve(&key, &bytes, now, 3600, now, 300).unwrap();
    store
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "ack-clock".into(),
                recipient_instance_id: "instance-ack".into(),
                delivery_generation: 1,
                expires_at: now + 3600,
                message_bytes: bytes,
            },
        )
        .unwrap();
    // Précondition de l'oracle ACK : gate de la remise déjà libérée.
    store
        .conn
        .execute(
            "UPDATE send_deliveries SET turn_gate=1 WHERE delivery_id='ack-clock'",
            [],
        )
        .unwrap();
    let first_ack = now - 31;
    store
        .acknowledge_send_delivery_at("ack-clock", "instance-ack", 1, first_ack)
        .unwrap();
    let first: Option<i64> = store
        .conn
        .query_row(
            "SELECT acked_at FROM send_deliveries WHERE delivery_id='ack-clock'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(first, Some(first_ack));
    store
        .acknowledge_send_delivery_at("ack-clock", "instance-ack", 1, now)
        .unwrap();
    let replay: Option<i64> = store
        .conn
        .query_row(
            "SELECT acked_at FROM send_deliveries WHERE delivery_id='ack-clock'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(replay, first, "le rejeu conserve la première date");
}
