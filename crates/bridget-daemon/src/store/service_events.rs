//! Dépôt, claims et faits de cycle de vie du protocole public.
//! Aucun métier le service compagnon : bytes persistés, lease, curseur et transition atomique.
use super::{Store, StoreError, mark_answered_in_transaction};
use bridget_transport::greffe_authorization::GreffeAuthorizationAttestation;
use bridget_transport::protocol::{
    CoordinationEventKind, GuichetLifecycleState, GuichetOutcome, ServiceRequestOperation,
    ServiceRequestPayload,
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

const GUICHET_LEASE_SECS: i64 = 60;
pub(crate) const MAX_GUICHET_FRAME_BYTES: usize = 64 * 1024;

/// Requête de guichet validée par le daemon avant toute persistance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetDeposit {
    pub issuer_scope: String,
    pub request_id: String,
    pub issued_at: i64,
    pub from: String,
    pub operation: ServiceRequestOperation,
    pub payload: ServiceRequestPayload,
    pub canonical_bytes: Vec<u8>,
    /// Métadonnée produite par le daemon, jamais lue depuis la charge client.
    pub authorization_attestation: Option<GreffeAuthorizationAttestation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetClaim {
    pub deposited_sequence: i64,
    pub issuer_scope: String,
    pub request_id: String,
    pub canonical_request: Vec<u8>,
    pub authorization_attestation: Option<GreffeAuthorizationAttestation>,
    pub claimed_at: i64,
    pub claim_generation: u64,
    pub claim_token: String,
    pub claim_lease_expires_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuichetResult {
    Queued {
        expires_at: i64,
    },
    OutcomeUnknown {
        expires_at: i64,
    },
    Terminal {
        issue: String,
        expires_at: i64,
        newly_finalized: bool,
        /// Réponse canonique durable produite par le service compagnon, jamais reconstruite
        /// depuis le seul libellé terminal.
        reply_bytes: Vec<u8>,
    },
    CanonicalBytesMismatch,
    IdempotencyExpired,
    InvalidIssuedAt,
    ClaimStale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum GuichetNext {
    Claimed(GuichetClaim),
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetReplyInput<'a> {
    pub issuer_scope: &'a str,
    pub request_id: &'a str,
    pub generation: u64,
    pub token: &'a str,
    pub response_message_id: &'a str,
    pub reply_bytes: &'a [u8],
    pub in_reply_to: &'a str,
    pub outcome: GuichetOutcome,
}

/// Événement de cycle durable, relivable par une connexion de service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetLifecycleEvent {
    pub issuer_scope: String,
    pub event_id: String,
    pub request_id: String,
    pub state: GuichetLifecycleState,
    pub observed_at: i64,
    pub in_reply_to: Option<String>,
    pub response_message_id: Option<String>,
}

/// Fait de coordination v1 durable, produit par le transport après une
/// relance réellement écrite. Il reste séparé des terminaux 015 : plusieurs
/// relances peuvent appartenir à une même demande suivie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetCoordinationEvent {
    /// Curseur alloué par SQLite, stable même si deux rappels ont le même
    /// instant. Il est l'autorité de reprise publique de T1604.
    pub cursor: u64,
    pub event_id: String,
    pub request_id: String,
    pub kind: CoordinationEventKind,
    pub reminder_message_id: String,
    pub recipient: String,
    pub generation: u64,
    pub observed_at: i64,
}

/// Résultat brut de la relève cursée. Une lacune reste séparée de l'erreur de
/// lecture : le daemon la rendra respectivement `coordination_gap` ou
/// `coordination_unavailable` sans jamais l'aplatir en fait métier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuichetCoordinationReplay {
    pub events: Vec<GuichetCoordinationEvent>,
    pub through_cursor: Option<u64>,
    pub gap: Option<(u64, u64)>,
}

impl Store {
    /// Dépose ou rejoue une demande de guichet. La clé et les octets restent
    /// dans l'unique transaction IMMEDIATE : deux producteurs concurrents ne
    /// peuvent ni créer deux lignes, ni remplacer le canon du premier.
    pub fn deposit_guichet(
        &mut self,
        deposit: &GuichetDeposit,
        horizon_secs: i64,
        issued_at_tolerance_secs: i64,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        if deposit.canonical_bytes.len().saturating_add(1) > MAX_GUICHET_FRAME_BYTES {
            return Err(StoreError::FrameTooLarge {
                max_frame_bytes: MAX_GUICHET_FRAME_BYTES,
            });
        }
        let authorization_attestation = deposit
            .authorization_attestation
            .as_ref()
            .map(serde_json::to_vec)
            .transpose()
            .map_err(|_| StoreError::Invariant("attestation serveur non sérialisable"))?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let existing = guichet_row_for_key(&tx, &deposit.issuer_scope, &deposit.request_id)?;
        if let Some(row) = existing {
            let result = if row.canonical_request != deposit.canonical_bytes {
                GuichetResult::CanonicalBytesMismatch
            } else {
                guichet_existing_result(&row, now)
            };
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(result);
        }
        if deposit.issued_at > now.saturating_add(issued_at_tolerance_secs) {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::InvalidIssuedAt);
        }
        let expires_at = deposit.issued_at.saturating_add(horizon_secs);
        if expires_at < now {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::IdempotencyExpired);
        }
        tx.execute(
            "INSERT INTO guichet_requests
                (issuer_scope, operation_kind, request_id, canonical_request,
                 authorization_attestation, issued_at, expires_at, sender,
                 linked_request_id, state)
             VALUES (?1, 'service_request', ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'queued')",
            params![
                deposit.issuer_scope,
                deposit.request_id,
                deposit.canonical_bytes,
                authorization_attestation,
                deposit.issued_at,
                expires_at,
                deposit.from,
                linked_request_id(&deposit.payload),
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetResult::Queued { expires_at })
    }

    /// Relève atomiquement le prochain dépôt FIFO. Une tête abandonnée est
    /// remise en file avant la sélection ; en v1, des crashes répétés sur la
    /// même tête peuvent donc la faire réapparaître jusqu'à son refus métier.
    pub fn claim_next_guichet(&mut self, owner: &str, now: i64) -> Result<GuichetNext, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        tx.execute(
            "UPDATE guichet_requests
             SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                 claim_lease_expires_at = NULL
             WHERE state = 'claimed' AND claim_lease_expires_at < ?1",
            params![now],
        )
        .map_err(StoreError::Sqlite)?;
        let Some(row) = guichet_next_queued(&tx, now)? else {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetNext::Empty);
        };
        let generation = row.claim_generation.saturating_add(1);
        let token = claim_token();
        let lease_expires_at = now.saturating_add(GUICHET_LEASE_SECS);
        let changed = tx
            .execute(
                "UPDATE guichet_requests
                 SET state = 'claimed', claim_owner = ?1, claim_generation = ?2,
                     claim_token = ?3, claim_lease_expires_at = ?4
                 WHERE deposited_sequence = ?5 AND state = 'queued'",
                params![
                    owner,
                    generation as i64,
                    token,
                    lease_expires_at,
                    row.deposited_sequence
                ],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            return Err(StoreError::Invariant("claim FIFO concurrent perdu"));
        }
        let claim = GuichetClaim {
            deposited_sequence: row.deposited_sequence,
            issuer_scope: row.issuer_scope,
            request_id: row.request_id,
            canonical_request: row.canonical_request,
            authorization_attestation: decode_authorization_attestation(
                row.authorization_attestation.as_deref(),
            )?,
            claimed_at: now,
            claim_generation: generation,
            claim_token: token,
            claim_lease_expires_at: lease_expires_at,
            expires_at: row.expires_at,
        };
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetNext::Claimed(claim))
    }

    /// Rejoue strictement le claim courant sans prolonger sa lease.
    pub fn claim_guichet(
        &self,
        owner: &str,
        issuer_scope: &str,
        request_id: &str,
        token: &str,
        now: i64,
    ) -> Result<Result<GuichetClaim, GuichetResult>, StoreError> {
        let Some(row) = guichet_row_for_key(&self.conn, issuer_scope, request_id)? else {
            return Ok(Err(GuichetResult::IdempotencyExpired));
        };
        if row.state != "claimed"
            || row.claim_owner.as_deref() != Some(owner)
            || row.claim_token.as_deref() != Some(token)
            || row
                .claim_lease_expires_at
                .is_none_or(|expires| expires < now)
        {
            return Ok(Err(GuichetResult::ClaimStale));
        }
        Ok(Ok(GuichetClaim {
            deposited_sequence: row.deposited_sequence,
            issuer_scope: row.issuer_scope,
            request_id: row.request_id,
            canonical_request: row.canonical_request,
            authorization_attestation: decode_authorization_attestation(
                row.authorization_attestation.as_deref(),
            )?,
            claimed_at: now,
            claim_generation: row.claim_generation,
            claim_token: row.claim_token.expect("claim courant sans token"),
            claim_lease_expires_at: row
                .claim_lease_expires_at
                .expect("claim courant sans lease"),
            expires_at: row.expires_at,
        }))
    }

    pub fn lookup_guichet(
        &self,
        issuer_scope: &str,
        request_id: &str,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        Ok(guichet_row_for_key(&self.conn, issuer_scope, request_id)?
            .map(|row| guichet_existing_result(&row, now))
            .unwrap_or(GuichetResult::IdempotencyExpired))
    }

    /// Consomme un claim seulement si le détenteur, sa génération, son token
    /// et sa lease sont encore courants. Le résultat entier est durable avant
    /// toute réponse socket, ce qui rend le retry reconstructible après crash.
    pub fn reply_guichet(
        &mut self,
        owner: &str,
        input: GuichetReplyInput<'_>,
        now: i64,
    ) -> Result<GuichetResult, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let Some(row) = guichet_row_for_key(&tx, input.issuer_scope, input.request_id)? else {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::IdempotencyExpired);
        };
        if matches!(row.state.as_str(), "replied" | "rejected") {
            let result = if row.reply_bytes.as_deref() == Some(input.reply_bytes) {
                guichet_existing_result(&row, now)
            } else {
                GuichetResult::CanonicalBytesMismatch
            };
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(result);
        }
        if row
            .linked_request_id
            .as_deref()
            .is_some_and(|linked| linked != input.in_reply_to)
        {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::CanonicalBytesMismatch);
        }
        let issue = guichet_outcome_name(input.outcome).to_string();
        let changed = tx
            .execute(
                "UPDATE guichet_requests
                 SET state = CASE WHEN ?1 = 'refused' THEN 'rejected' ELSE 'replied' END,
                     result_issue = ?1, reply_bytes = ?2
                 WHERE issuer_scope = ?3 AND operation_kind = 'service_request'
                   AND request_id = ?4 AND state = 'claimed'
                   AND claim_owner = ?5 AND claim_generation = ?6
                   AND claim_token = ?7 AND claim_lease_expires_at >= ?8",
                params![
                    issue,
                    input.reply_bytes,
                    input.issuer_scope,
                    input.request_id,
                    owner,
                    input.generation as i64,
                    input.token,
                    now
                ],
            )
            .map_err(StoreError::Sqlite)?;
        if changed != 1 {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(GuichetResult::ClaimStale);
        }
        if input.outcome == GuichetOutcome::Accepted
            && let Some(linked_request_id) = row.linked_request_id.as_deref()
        {
            // La clôture, lorsqu'elle est encore ouverte, est indissociable
            // du résultat guichet durable. Une demande déjà terminale relève
            // de D-208 : le rapport reste traçable sans la rouvrir.
            // La clé du service n'est pas l'identité de l'émetteur de la
            // demande suivie (désormais un UUID). L'autorité est le couple
            // durable de CETTE demande, pas le nom historique du service.
            // Le déposant doit en être le destinataire ; une référence vers
            // la demande d'un tiers ne confère jamais le droit de la clôturer.
            let recipient: Option<String> = tx
                .query_row(
                    "SELECT sender FROM tracked_requests WHERE id=?1 AND target=?2",
                    params![linked_request_id, row.sender],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::Sqlite)?;
            let answered = match recipient {
                Some(recipient) => {
                    mark_answered_in_transaction(&tx, linked_request_id, &row.sender, &recipient)
                        .map_err(StoreError::Sqlite)?
                }
                None => false,
            };
            if answered {
                record_lifecycle_event_in_transaction(
                    &tx,
                    &row.issuer_scope,
                    &row.request_id,
                    GuichetLifecycleState::Answered,
                    now,
                    Some(linked_request_id),
                    Some(input.response_message_id),
                )?;
            }
        }
        let expires_at = row.expires_at;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(GuichetResult::Terminal {
            issue,
            expires_at,
            newly_finalized: true,
            reply_bytes: input.reply_bytes.to_vec(),
        })
    }

    /// Lit les faits terminaux retenus par Bridget dans l'ordre d'observation.
    /// La réception est idempotente côté le service compagnon par `event_id`; une nouvelle
    /// connexion peut donc relever sans réinventer une transition.
    pub fn guichet_lifecycle_events(&self) -> Result<Vec<GuichetLifecycleEvent>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT issuer_scope, event_id, request_id, state, observed_at,
                        in_reply_to, response_message_id
                 FROM guichet_lifecycle_events
                 ORDER BY observed_at ASC, event_id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map([], lifecycle_event_from_row)
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    /// Atteste une relance après son écriture transport. La clé
    /// `(request_id, generation)` empêche qu'un même palier soit exposé deux
    /// fois après un rejeu local ; les clients relisent ensuite les mêmes
    /// bytes via l'`event_id` durable.
    pub fn record_reminder_sent(
        &mut self,
        request_id: &str,
        reminder_message_id: &str,
        recipient: &str,
        generation: u64,
        observed_at: i64,
    ) -> Result<GuichetCoordinationEvent, StoreError> {
        if request_id.is_empty()
            || reminder_message_id.is_empty()
            || recipient.is_empty()
            || generation == 0
        {
            return Err(StoreError::Invariant("rappel de coordination incomplet"));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sqlite)?;
        let existing = tx
            .query_row(
                "SELECT cursor, event_id, request_id, kind, reminder_message_id, recipient,
                        generation, observed_at
                 FROM guichet_coordination_events
                 WHERE request_id = ?1 AND generation = ?2",
                params![request_id, generation as i64],
                coordination_event_from_row,
            )
            .optional()
            .map_err(StoreError::Sqlite)?;
        if let Some(event) = existing {
            tx.commit().map_err(StoreError::Sqlite)?;
            return Ok(event);
        }
        tx.execute(
            "UPDATE guichet_coordination_stream_state
             SET high_watermark = high_watermark + 1 WHERE singleton = 1",
            [],
        )
        .map_err(StoreError::Sqlite)?;
        let cursor = tx
            .query_row(
                "SELECT high_watermark FROM guichet_coordination_stream_state WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(StoreError::Sqlite)? as u64;
        let event = GuichetCoordinationEvent {
            cursor,
            event_id: format!("evt-{}", Uuid::new_v4()),
            request_id: request_id.to_string(),
            kind: CoordinationEventKind::ReminderSent,
            reminder_message_id: reminder_message_id.to_string(),
            recipient: recipient.to_string(),
            generation,
            observed_at,
        };
        tx.execute(
            "INSERT INTO guichet_coordination_events
                 (cursor, event_id, request_id, kind, reminder_message_id, recipient,
                  generation, observed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.cursor as i64,
                event.event_id,
                event.request_id,
                coordination_event_kind_name(event.kind),
                event.reminder_message_id,
                event.recipient,
                event.generation as i64,
                event.observed_at,
            ],
        )
        .map_err(StoreError::Sqlite)?;
        tx.commit().map_err(StoreError::Sqlite)?;
        Ok(event)
    }

    /// Relève historique v1, conservée pour les clients qui n'ont pas négocié
    /// la relève cursée T1604.
    pub fn guichet_coordination_events(&self) -> Result<Vec<GuichetCoordinationEvent>, StoreError> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT cursor, event_id, request_id, kind, reminder_message_id, recipient,
                        generation, observed_at
                 FROM guichet_coordination_events
                 ORDER BY observed_at ASC, event_id ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map([], coordination_event_from_row)
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)
    }

    /// Relève bornée à partir d'un curseur public. Le watermark est monotone
    /// et persistant : une disparition d'octet entre deux curseurs devient une
    /// lacune attestée, jamais un `SnapshotCaughtUp` optimiste.
    pub fn guichet_coordination_events_after(
        &self,
        after_cursor: Option<u64>,
    ) -> Result<GuichetCoordinationReplay, StoreError> {
        let high_watermark = self
            .conn
            .query_row(
                "SELECT high_watermark FROM guichet_coordination_stream_state WHERE singleton = 1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(StoreError::Sqlite)? as u64;
        let after = after_cursor.unwrap_or(0);
        if after > high_watermark {
            return Ok(GuichetCoordinationReplay {
                events: Vec::new(),
                through_cursor: None,
                gap: Some((high_watermark.saturating_add(1), after)),
            });
        }
        let mut statement = self
            .conn
            .prepare(
                "SELECT cursor, event_id, request_id, kind, reminder_message_id, recipient,
                        generation, observed_at
                 FROM guichet_coordination_events
                 WHERE cursor > ?1
                 ORDER BY cursor ASC",
            )
            .map_err(StoreError::Sqlite)?;
        let events = statement
            .query_map(params![after as i64], coordination_event_from_row)
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)?;
        let mut expected = after.saturating_add(1);
        for event in &events {
            if event.cursor != expected {
                return Ok(GuichetCoordinationReplay {
                    events: Vec::new(),
                    through_cursor: None,
                    gap: Some((expected, event.cursor.saturating_sub(1))),
                });
            }
            expected = expected.saturating_add(1);
        }
        if expected <= high_watermark {
            return Ok(GuichetCoordinationReplay {
                events: Vec::new(),
                through_cursor: None,
                gap: Some((expected, high_watermark)),
            });
        }
        Ok(GuichetCoordinationReplay {
            through_cursor: events.last().map(|event| event.cursor).or(after_cursor),
            events,
            gap: None,
        })
    }

    /// La disparition d'une connexion ne laisse jamais son droit de claim
    /// actif : la prochaine relève récupère le même dépôt et une génération neuve.
    pub fn release_guichet_claims(&mut self, owner: &str) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE guichet_requests
                 SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                     claim_lease_expires_at = NULL
                 WHERE state = 'claimed' AND claim_owner = ?1",
                params![owner],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }

    /// Uniquement au bootstrap du daemon : toute lease d'un processus arrêté
    /// redevient FIFO. Ouvrir un second handle SQLite ne doit jamais voler le
    /// claim vivant du premier.
    pub fn recover_guichet_claims_after_restart(&mut self) -> Result<(), StoreError> {
        self.conn
            .execute(
                "UPDATE guichet_requests
                 SET state = 'queued', claim_owner = NULL, claim_token = NULL,
                     claim_lease_expires_at = NULL
                 WHERE state = 'claimed'",
                [],
            )
            .map_err(StoreError::Sqlite)?;
        Ok(())
    }
}

fn record_lifecycle_event_in_transaction(
    transaction: &Transaction<'_>,
    issuer_scope: &str,
    request_id: &str,
    state: GuichetLifecycleState,
    observed_at: i64,
    in_reply_to: Option<&str>,
    response_message_id: Option<&str>,
) -> Result<GuichetLifecycleEvent, StoreError> {
    let existing = transaction
        .query_row(
            "SELECT issuer_scope, event_id, request_id, state, observed_at,
                    in_reply_to, response_message_id
             FROM guichet_lifecycle_events
             WHERE issuer_scope = ?1 AND request_id = ?2",
            params![issuer_scope, request_id],
            lifecycle_event_from_row,
        )
        .optional()
        .map_err(StoreError::Sqlite)?;
    if let Some(existing) = existing {
        return Ok(existing);
    }
    let event = GuichetLifecycleEvent {
        issuer_scope: issuer_scope.to_string(),
        event_id: format!("evt-{}", Uuid::new_v4()),
        request_id: request_id.to_string(),
        state,
        observed_at,
        in_reply_to: in_reply_to.map(str::to_string),
        response_message_id: response_message_id.map(str::to_string),
    };
    transaction
        .execute(
            "INSERT INTO guichet_lifecycle_events
                 (issuer_scope, request_id, event_id, state, observed_at,
                  in_reply_to, response_message_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.issuer_scope,
                event.request_id,
                event.event_id,
                lifecycle_state_name(event.state),
                event.observed_at,
                event.in_reply_to,
                event.response_message_id,
            ],
        )
        .map_err(StoreError::Sqlite)?;
    Ok(event)
}

pub(super) fn record_lifecycle_for_linked_request_in_transaction(
    transaction: &Transaction<'_>,
    linked_request_id: &str,
    state: GuichetLifecycleState,
    observed_at: i64,
) -> Result<(), StoreError> {
    let deposits = {
        let mut statement = transaction
            .prepare(
                "SELECT issuer_scope, request_id
                 FROM guichet_requests
                 WHERE linked_request_id = ?1
                 ORDER BY deposited_sequence ASC",
            )
            .map_err(StoreError::Sqlite)?;
        statement
            .query_map(params![linked_request_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(StoreError::Sqlite)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sqlite)?
    };
    for (issuer_scope, request_id) in deposits {
        let _ = record_lifecycle_event_in_transaction(
            transaction,
            &issuer_scope,
            &request_id,
            state,
            observed_at,
            Some(linked_request_id),
            None,
        )?;
    }
    Ok(())
}

fn lifecycle_event_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GuichetLifecycleEvent> {
    let state = match row.get::<_, String>(3)?.as_str() {
        "answered" => GuichetLifecycleState::Answered,
        "cancelled" => GuichetLifecycleState::Cancelled,
        "timed_out" => GuichetLifecycleState::TimedOut,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(GuichetLifecycleEvent {
        issuer_scope: row.get(0)?,
        event_id: row.get(1)?,
        request_id: row.get(2)?,
        state,
        observed_at: row.get(4)?,
        in_reply_to: row.get(5)?,
        response_message_id: row.get(6)?,
    })
}

fn lifecycle_state_name(state: GuichetLifecycleState) -> &'static str {
    match state {
        GuichetLifecycleState::Answered => "answered",
        GuichetLifecycleState::Cancelled => "cancelled",
        GuichetLifecycleState::TimedOut => "timed_out",
    }
}

fn coordination_event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<GuichetCoordinationEvent> {
    let kind = match row.get::<_, String>(3)?.as_str() {
        "reminder_sent" => CoordinationEventKind::ReminderSent,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(GuichetCoordinationEvent {
        cursor: row.get::<_, i64>(0)? as u64,
        event_id: row.get(1)?,
        request_id: row.get(2)?,
        kind,
        reminder_message_id: row.get(4)?,
        recipient: row.get(5)?,
        generation: row.get::<_, i64>(6)? as u64,
        observed_at: row.get(7)?,
    })
}

fn coordination_event_kind_name(kind: CoordinationEventKind) -> &'static str {
    match kind {
        CoordinationEventKind::ReminderSent => "reminder_sent",
    }
}

#[derive(Debug)]
struct GuichetRow {
    deposited_sequence: i64,
    issuer_scope: String,
    request_id: String,
    canonical_request: Vec<u8>,
    authorization_attestation: Option<Vec<u8>>,
    sender: String,
    expires_at: i64,
    state: String,
    claim_owner: Option<String>,
    claim_generation: u64,
    claim_token: Option<String>,
    claim_lease_expires_at: Option<i64>,
    result_issue: Option<String>,
    reply_bytes: Option<Vec<u8>>,
    linked_request_id: Option<String>,
}

fn guichet_row_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<GuichetRow> {
    Ok(GuichetRow {
        deposited_sequence: row.get(0)?,
        issuer_scope: row.get(1)?,
        request_id: row.get(2)?,
        canonical_request: row.get(3)?,
        sender: row.get(4)?,
        expires_at: row.get(5)?,
        state: row.get(6)?,
        claim_owner: row.get(7)?,
        claim_generation: row.get::<_, i64>(8)? as u64,
        claim_token: row.get(9)?,
        claim_lease_expires_at: row.get(10)?,
        result_issue: row.get(11)?,
        reply_bytes: row.get(12)?,
        linked_request_id: row.get(13)?,
        authorization_attestation: row.get(14)?,
    })
}

fn decode_authorization_attestation(
    bytes: Option<&[u8]>,
) -> Result<Option<GreffeAuthorizationAttestation>, StoreError> {
    bytes
        .map(serde_json::from_slice)
        .transpose()
        .map_err(|_| StoreError::Invariant("attestation serveur durable corrompue"))
}

fn guichet_row_for_key(
    conn: &Connection,
    issuer_scope: &str,
    request_id: &str,
) -> Result<Option<GuichetRow>, StoreError> {
    conn.query_row(
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request, sender,
                expires_at, state, claim_owner, claim_generation, claim_token,
                claim_lease_expires_at, result_issue, reply_bytes, linked_request_id,
                authorization_attestation
         FROM guichet_requests
         WHERE issuer_scope = ?1 AND operation_kind = 'service_request' AND request_id = ?2",
        params![issuer_scope, request_id],
        guichet_row_from_row,
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn guichet_next_queued(conn: &Connection, now: i64) -> Result<Option<GuichetRow>, StoreError> {
    conn.query_row(
        "SELECT deposited_sequence, issuer_scope, request_id, canonical_request, sender,
                expires_at, state, claim_owner, claim_generation, claim_token,
                claim_lease_expires_at, result_issue, reply_bytes, linked_request_id,
                authorization_attestation
         FROM guichet_requests
         WHERE state = 'queued' AND expires_at >= ?1
         ORDER BY deposited_sequence ASC LIMIT 1",
        params![now],
        guichet_row_from_row,
    )
    .optional()
    .map_err(StoreError::Sqlite)
}

fn guichet_existing_result(row: &GuichetRow, now: i64) -> GuichetResult {
    if row.expires_at < now {
        return GuichetResult::IdempotencyExpired;
    }
    match row.state.as_str() {
        "queued" | "claimed" => GuichetResult::OutcomeUnknown {
            expires_at: row.expires_at,
        },
        "replied" | "rejected" => GuichetResult::Terminal {
            issue: row
                .result_issue
                .clone()
                .unwrap_or_else(|| "refused".to_string()),
            expires_at: row.expires_at,
            newly_finalized: false,
            reply_bytes: row.reply_bytes.clone().unwrap_or_default(),
        },
        _ => GuichetResult::IdempotencyExpired,
    }
}

fn guichet_outcome_name(outcome: GuichetOutcome) -> &'static str {
    match outcome {
        GuichetOutcome::Accepted => "accepted",
        GuichetOutcome::RequestAlreadyTerminal => "request_already_terminal",
        GuichetOutcome::RecipientUnavailable => "recipient_unavailable",
        GuichetOutcome::Refused => "refused",
    }
}

fn linked_request_id(payload: &ServiceRequestPayload) -> Option<&str> {
    match payload {
        ServiceRequestPayload::DeliveryReport { in_reply_to, .. } => Some(in_reply_to),
        ServiceRequestPayload::Delegation { .. }
        | ServiceRequestPayload::Delegate { .. }
        | ServiceRequestPayload::RegistreAdd { .. }
        | ServiceRequestPayload::ObjectiveClose { .. } => None,
    }
}

fn claim_token() -> String {
    // UUID v4 fournit 128 bits tirés par l'OS. Le contrat fixe une base64url
    // sans padding : 16 octets deviennent exactement 22 caractères opaques.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = Uuid::new_v4().into_bytes();
    let mut encoded = String::with_capacity(22);
    for chunk in bytes.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(ALPHABET[(a >> 2) as usize] as char);
        encoded.push(ALPHABET[((a & 0x03) << 4 | b >> 4) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(ALPHABET[((b & 0x0f) << 2 | c >> 6) as usize] as char);
        }
        if chunk.len() > 2 {
            encoded.push(ALPHABET[(c & 0x3f) as usize] as char);
        }
    }
    encoded
}
