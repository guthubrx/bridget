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
    /// Fige la remise d'un envoi. Le changement d'état du socle, l'entrée
    /// `send_deliveries` et la **visibilité ledger** (fait d'émission) partagent
    /// une transaction SQLite. L'accusé (`acked`) reste un fait distinct :
    /// `outcome_unknown` reste nominal tant que `DeliverAcked` n'est pas reçu.
    pub fn begin_send_delivery(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_inner(key, delivery, None, None)
    }

    /// Prépare atomiquement la remise et le suivi d'une réponse attendue.
    pub fn begin_send_delivery_with_reply(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: &ReplyTracking,
    ) -> Result<(), IdempotencyError> {
        self.begin_send_delivery_inner(key, delivery, Some(reply), None)
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
        self.begin_send_delivery_inner(key, delivery, reply, Some(link))
    }

    fn begin_send_delivery_inner(
        &mut self,
        key: &IdempotencyKey,
        delivery: &SendDelivery,
        reply: Option<&ReplyTracking>,
        execution_link: Option<&DeliveryExecutionLink>,
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
        // Jamais de .ok() silencieux : une remise sans enveloppe lisible
        // recréerait la classe « invisible au ledger » que la gravure à
        // begin_send_delivery existe pour supprimer.
        let emitted_message =
            serde_json::from_slice::<bridget_core::BridgetMessage>(&delivery.message_bytes)
                .map_err(|_| IdempotencyError::CorruptRecord("enveloppe de remise illisible"))?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let transitioned = tx.execute(
            "UPDATE idempotency_records SET state = 'dispatching'
             WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2
               AND state = 'prepared'",
            params![key.issuer_scope, key.idempotency_key],
        )?;
        if transitioned != 1 {
            return Err(IdempotencyError::DispatchUnavailable);
        }
        tx.execute(
            "INSERT INTO send_deliveries (
                delivery_id, issuer_scope, operation_kind, idempotency_key,
                recipient_instance_id, delivery_generation, phase, expires_at, message_bytes
             ) VALUES (?1, ?2, 'send', ?3, ?4, ?5, 'dispatching', ?6, ?7)",
            params![
                delivery.delivery_id,
                key.issuer_scope,
                key.idempotency_key,
                delivery.recipient_instance_id,
                delivery.delivery_generation,
                delivery.expires_at,
                delivery.message_bytes,
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
               AND phase = 'dispatching'
               AND expires_at > ?3
               AND (message_bytes IS NULL
                    OR json_valid(CAST(message_bytes AS TEXT)) = 0
                    OR json_extract(CAST(message_bytes AS TEXT), '$.thread_notice') IS NULL)",
            params![to_instance_id, from_instance_id, now],
        )?;
        Ok(updated)
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

    fn stored_send_delivery(
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

    /// Reprise bornée : seules les remises encore en cours pour l'instance
    /// exacte sont relivrées. Une remise Acked ou Indeterminate ne l'est pas.
    /// Toute ligne `dispatching` sans enveloppe est classée `indeterminate`
    /// dans la même transaction avant que les autres remises soient rendues.
    ///
    /// Complexité : O(N), N étant le nombre total de remises faute d'index sur
    /// l'instance ; le reclassement est groupé pour éviter N écritures.
    pub fn dispatching_deliveries_for_instance(
        &mut self,
        recipient_instance_id: &str,
        now: i64,
    ) -> Result<Vec<SendDelivery>, IdempotencyError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let reclassified = tx.execute(
            "UPDATE send_deliveries SET phase = 'indeterminate'
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching'
               AND message_bytes IS NULL",
            params![recipient_instance_id],
        )?;
        let stored = {
            let mut statement = tx.prepare(
                "SELECT delivery_id, recipient_instance_id, delivery_generation, expires_at, message_bytes
                 FROM send_deliveries
                 WHERE recipient_instance_id = ?1 AND phase = 'dispatching' AND expires_at > ?2
                 ORDER BY delivery_id",
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
            "UPDATE send_deliveries SET phase = 'indeterminate'
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
                "UPDATE send_deliveries SET phase = 'indeterminate'
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
             WHERE recipient_instance_id = ?1 AND phase = 'dispatching'",
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
                "UPDATE send_deliveries SET phase = 'orphaned'
                 WHERE delivery_id = ?1 AND phase = 'dispatching'",
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
        let delivery = tx.execute("UPDATE send_deliveries SET phase = 'acked' WHERE delivery_id = ?1 AND phase = 'dispatching'", params![delivery_id])?;
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
                let changed = crate::store::mark_answered_in_transaction(
                    &tx,
                    request_id,
                    &message.from,
                    &message.to,
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
