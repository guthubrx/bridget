//! Session 102 — règles métier des fils inter-agents.
//!
//! Le daemon est l'autorité : identité attestée en amont, appartenance et
//! rejeu décidés ici et dans `store::threads` sous transaction. Trois faits
//! distincts et jamais confondus : `posted` (contribution durable),
//! `dispatched` (alerte injectée chez un membre visé) et `acknowledged`
//! (page confirmée par un membre). Aucune lecture du corps pour y chercher des
//! mentions : seules les cibles structurées comptent.

use crate::store::threads::{
    AckOutcome, AddMembers, CloseThread, CreateThread, HistoryOutcome, NotifySpec, PostEntry,
    ReadOutcome, ReadRequest, ThreadLimits, ThreadRefusal, ThreadTxOutcome, WakeRow,
};
use crate::store::{Store, StoreError};
use bridget_transport::protocol::{
    ProjectWarning, THREAD_CONTRACT_VERSION, ThreadAction, ThreadEntryKind, ThreadNotify,
    ThreadRequest, ThreadResult,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

// Bornes V1 (contracts/thread-api.md). Constantes partagées, pas d'options.
pub(crate) const MIN_MEMBERS: usize = 2;
pub(crate) const MAX_MEMBERS: usize = 16;
pub(crate) const TITLE_MAX_CHARS: usize = 160;
pub(crate) const BODY_MAX_BYTES: usize = 16 * 1024;
pub(crate) const ACTION_MAX_BYTES: usize = 2048;
pub(crate) const PAGE_DEFAULT: u32 = 50;
pub(crate) const PAGE_MAX: u32 = 200;
/// JSON complet d'une page, reçu et métadonnées compris.
pub(crate) const PAGE_MAX_BYTES: usize = 60 * 1024;
/// Réserve pour les métadonnées bornées de page ; le reste va aux entrées.
pub(crate) const PAGE_METADATA_RESERVE_BYTES: usize = 12 * 1024;
/// Une entrée sérialisée doit tenir seule dans une page.
pub(crate) const ENTRY_MAX_JSON_BYTES: usize = PAGE_MAX_BYTES - PAGE_METADATA_RESERVE_BYTES;
pub(crate) const RECEIPT_TTL_SECS: i64 = 10 * 60;
pub(crate) const LIST_DEFAULT: u32 = 20;
pub(crate) const LIST_MAX: u32 = 100;
pub(crate) const LIMITS: ThreadLimits = ThreadLimits {
    max_threads: 256,
    max_open_per_creator: 32,
    max_entries_per_thread: 10_000,
    max_body_bytes_per_thread: 16 * 1024 * 1024,
    max_body_bytes_total: 128 * 1024 * 1024,
};
/// Sollicitations : départs par seconde, lot par tick, délai d'injection.
pub(crate) const WAKE_DEPARTURES_PER_SEC: u32 = 5;
pub(crate) const WAKE_BATCH: u32 = 16;
pub(crate) const WAKE_INJECTION_DEADLINE_SECS: i64 = 120;
pub(crate) const THREAD_NOTICE_VERSION: u16 = 1;

/// Faits d'annuaire fournis par le daemon pour `show` ; jamais des curseurs.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MemberFacts {
    pub connected: Option<bool>,
    pub thread_notice_version: Option<u16>,
}

/// Regard du daemon sur l'annuaire durable, injecté pour rester testable.
pub(crate) trait Directory {
    fn known_agent(&self, agent_id: &str) -> bool;
    fn member_facts(&self, agent_id: &str) -> MemberFacts;
    fn communication_scope(
        &self,
        actor: &str,
        members: &[String],
        reason: Option<&str>,
    ) -> Result<Vec<ProjectWarning>, Value> {
        let mut warnings = Vec::new();
        for member in members.iter().filter(|member| member.as_str() != actor) {
            warnings.extend(
                crate::communication::project_scope(None, None, member, reason)
                    .map_err(|detail| error("invalid_cross_project_reason", detail, false))?,
            );
        }
        Ok(warnings)
    }
}

pub(crate) fn error(code: &str, detail: impl Into<String>, retryable: bool) -> Value {
    json!({
        "status": "error",
        "code": code,
        "detail": detail.into(),
        "retryable": retryable,
    })
}

fn result(value: Value) -> ThreadResult {
    ThreadResult {
        version: THREAD_CONTRACT_VERSION,
        result: value,
    }
}

/// O(log T + P log E + M), P ≤200, M ≤16. Aucun reçu ni écriture.
pub(crate) fn human_view(
    store: &Store,
    actor: &str,
    action: &bridget_transport::protocol::HumanThreadViewAction,
) -> bridget_transport::protocol::HumanThreadViewResult {
    use bridget_transport::protocol::{
        HUMAN_THREAD_VIEW_MAX_BYTES, HUMAN_THREAD_VIEW_VERSION, HumanRecentThreadSummary,
        HumanThreadEntry, HumanThreadMember, HumanThreadState as S, HumanThreadSummary,
        HumanThreadViewAction as A, HumanThreadViewError as E, HumanThreadViewOutcome as O,
        HumanThreadViewResult as R,
    };
    const MAX_SEQ: u64 = 9_007_199_254_740_991;
    let outcome = (|| -> Result<R, E> {
        let invalid = || E::InvalidRequest;
        let limit = |value, default, max| page_limit(value, default, max).map_err(|_| invalid());
        let uuid = |value: &str| canonical_uuid(value).ok_or_else(invalid);
        let result = match action {
            A::ListRecent {
                limit: requested,
                after,
            } => {
                let limit = limit(*requested, LIST_DEFAULT, LIST_MAX)?;
                let cursor = after
                    .as_deref()
                    .map(parse_recent_cursor)
                    .transpose()
                    .map_err(|_| invalid())?;
                let (threads, next_after) = store
                    .thread_list_recent(
                        actor,
                        limit,
                        cursor.as_ref().map(|(date, id)| (*date, id.as_str())),
                    )
                    .map_err(|_| E::StorageUnavailable)?;
                let ids = threads
                    .iter()
                    .map(|t| t.summary.thread_id.clone())
                    .collect::<Vec<_>>();
                let mut members = store
                    .thread_members_with_names(actor, &ids)
                    .map_err(|_| E::StorageUnavailable)?;
                let threads = threads
                    .into_iter()
                    .map(|t| {
                        if t.last_activity_at < 0 || t.last_activity_at as u64 > MAX_SEQ {
                            return Err(E::StorageUnavailable);
                        }
                        Ok(HumanRecentThreadSummary {
                            members: members.remove(&t.summary.thread_id).unwrap_or_default(),
                            thread_id: t.summary.thread_id,
                            title: t.summary.title,
                            creator_id: t.summary.creator_id,
                            state: if t.summary.closed { S::Closed } else { S::Open },
                            last_seq: t.summary.last_seq,
                            last_activity_at: t.last_activity_at,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                O::ListedRecent {
                    threads,
                    next_after,
                }
            }
            A::List {
                limit: requested,
                after_thread_id,
            } => {
                let limit = limit(*requested, LIST_DEFAULT, LIST_MAX)?;
                let after = after_thread_id.as_deref().map(uuid).transpose()?;
                let (threads, next_after) = store
                    .thread_list(actor, limit, after.as_deref())
                    .map_err(|_| E::StorageUnavailable)?;
                let ids = threads
                    .iter()
                    .map(|t| t.thread_id.clone())
                    .collect::<Vec<_>>();
                let mut members = store
                    .thread_members_with_names(actor, &ids)
                    .map_err(|_| E::StorageUnavailable)?;
                O::Listed {
                    threads: threads
                        .into_iter()
                        .map(|t| HumanThreadSummary {
                            members: members.remove(&t.thread_id).unwrap_or_default(),
                            thread_id: t.thread_id,
                            title: t.title,
                            creator_id: t.creator_id,
                            state: if t.closed { S::Closed } else { S::Open },
                            last_seq: t.last_seq,
                        })
                        .collect(),
                    next_after,
                }
            }
            A::Show { thread_id } => {
                let thread_id = uuid(thread_id)?;
                let view = store
                    .thread_show(actor, &thread_id)
                    .map_err(|_| E::StorageUnavailable)?
                    .map_err(|_| E::ThreadUnavailable)?;
                let mut grouped = store
                    .thread_members_with_names(actor, std::slice::from_ref(&thread_id))
                    .map_err(|_| E::StorageUnavailable)?;
                O::Shown {
                    thread_id,
                    title: view.thread.title,
                    creator_id: view.thread.creator_id,
                    state: if view.thread.closed_at.is_some() {
                        S::Closed
                    } else {
                        S::Open
                    },
                    created_at: view.thread.created_at,
                    closed_at: view.thread.closed_at,
                    last_seq: view.thread.last_seq,
                    members: grouped.remove(&view.thread.thread_id).unwrap_or_default(),
                }
            }
            A::History {
                thread_id,
                limit: requested,
                ..
            }
            | A::HistoryRecent {
                thread_id,
                limit: requested,
                ..
            } => {
                let thread_id = uuid(thread_id)?;
                let limit = limit(*requested, PAGE_DEFAULT, PAGE_MAX)?;
                let (from, page) = match action {
                    A::History {
                        from_seq, to_seq, ..
                    } => {
                        let from = from_seq.unwrap_or(1);
                        if from == 0
                            || from > MAX_SEQ
                            || to_seq.is_some_and(|to| to > MAX_SEQ || to.saturating_add(1) < from)
                        {
                            return Err(invalid());
                        }
                        (
                            from,
                            store.thread_history(
                                actor,
                                &thread_id,
                                from,
                                *to_seq,
                                limit,
                                PAGE_MAX_BYTES - PAGE_METADATA_RESERVE_BYTES,
                            ),
                        )
                    }
                    A::HistoryRecent {
                        before_seq, to_seq, ..
                    } => {
                        if before_seq.is_some_and(|n| n > MAX_SEQ)
                            || to_seq.is_some_and(|n| n > MAX_SEQ)
                        {
                            return Err(invalid());
                        }
                        (
                            0,
                            store.thread_history_recent(
                                actor,
                                &thread_id,
                                *before_seq,
                                *to_seq,
                                limit,
                                PAGE_MAX_BYTES - PAGE_METADATA_RESERVE_BYTES,
                            ),
                        )
                    }
                    _ => unreachable!("variante historique"),
                };
                let page = match page.map_err(|_| E::StorageUnavailable)? {
                    HistoryOutcome::Page(page) => page,
                    HistoryOutcome::Refused(ThreadRefusal::RangeUnavailable) => {
                        return Err(E::InvalidRequest);
                    }
                    HistoryOutcome::Refused(_) => return Err(E::ThreadUnavailable),
                };
                let mut grouped = store
                    .thread_members_with_names(actor, std::slice::from_ref(&thread_id))
                    .map_err(|_| E::StorageUnavailable)?;
                let names = grouped
                    .remove(&thread_id)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|m| (m.agent_id, m.name))
                    .collect::<std::collections::HashMap<_, _>>();
                // Une table ≤16 noms ; aucune requête par entrée.
                let entries = page
                    .entries
                    .into_iter()
                    .map(|mut entry| {
                        entry["author_name"] = json!(
                            entry["author_id"]
                                .as_str()
                                .and_then(|id| names.get(id))
                                .cloned()
                                .flatten()
                        );
                        serde_json::from_value::<HumanThreadEntry>(entry)
                            .map_err(|_| E::StorageUnavailable)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if matches!(action, A::HistoryRecent { .. }) {
                    O::HistoryRecent {
                        thread_id,
                        through_seq: page.through_seq,
                        snapshot_seq: page.snapshot_seq,
                        has_more: page.has_more,
                        next_before_seq: if page.has_more {
                            entries.last().map(|e| e.seq - 1)
                        } else {
                            None
                        },
                        entries,
                    }
                } else {
                    O::History {
                        thread_id,
                        from_seq: from,
                        through_seq: page.through_seq,
                        snapshot_seq: page.snapshot_seq,
                        has_more: page.has_more,
                        next_from_seq: page.has_more.then_some(page.through_seq + 1),
                        entries,
                    }
                }
            }
        };
        let projection = R {
            version: HUMAN_THREAD_VIEW_VERSION,
            subject: Some(HumanThreadMember {
                agent_id: actor.into(),
                name: store
                    .agent_display_name(actor)
                    .map_err(|_| E::StorageUnavailable)?,
            }),
            result,
        };
        let bytes = serde_json::to_vec(&projection).map_err(|_| E::StorageUnavailable)?;
        if bytes.len() > HUMAN_THREAD_VIEW_MAX_BYTES {
            return Err(E::ResponseTooLarge);
        }
        Ok(projection)
    })();
    outcome.unwrap_or_else(R::error)
}

/// UUID canonique en minuscules ; toute autre forme est refusée.
pub(crate) fn canonical_uuid(value: &str) -> Option<String> {
    let parsed = uuid::Uuid::parse_str(value).ok()?;
    let canonical = parsed.hyphenated().to_string();
    (canonical == value).then_some(canonical)
}

/// O(128) au plus : format fermé partagé par le CLI et la projection.
pub(crate) fn parse_recent_cursor(value: &str) -> Result<(i64, String), ()> {
    if value.len() > 128 {
        return Err(());
    }
    let (time, id) = value.split_once(':').ok_or(())?;
    let timestamp = time.parse::<u64>().map_err(|_| ())?;
    if timestamp > 9_007_199_254_740_991 || timestamp.to_string() != time {
        return Err(());
    }
    Ok((timestamp as i64, canonical_uuid(id).ok_or(())?))
}

fn digest(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    format!("{:x}", hasher.finalize())
}

fn refusal_result(refusal: ThreadRefusal) -> Value {
    match refusal {
        ThreadRefusal::EnvelopeMismatch => error(
            "envelope_mismatch",
            "Clé déjà utilisée pour une autre opération ; l'opération d'origine est inchangée.",
            false,
        ),
        ThreadRefusal::ThreadUnavailable => error(
            "thread_unavailable",
            "Fil indisponible pour cette identité.",
            false,
        ),
        ThreadRefusal::ThreadClosed => {
            error("thread_closed", "Fil clos : consultation seulement.", false)
        }
        ThreadRefusal::CreatorRequired => error(
            "creator_required",
            "Opération réservée au créateur initial du fil.",
            false,
        ),
        ThreadRefusal::AudienceChanged => error(
            "audience_changed",
            "Les membres ont changé pendant la vérification ; revalider la demande.",
            true,
        ),
        ThreadRefusal::NotAMember(unknown) => error(
            "not_a_member",
            format!("Cibles hors du fil : {}", unknown.join(", ")),
            false,
        ),
        ThreadRefusal::InvalidReplyReference => error(
            "invalid_reply_reference",
            "reply_to_seq ne désigne aucune entrée de ce fil.",
            false,
        ),
        ThreadRefusal::InvalidSupersession => error(
            "invalid_supersession",
            "Remplacement refusé : cible actuelle, même auteur et mêmes destinataires requis.",
            false,
        ),
        ThreadRefusal::CapacityExceeded(kind) => error(
            "capacity_exceeded",
            format!("Plafond atteint : {kind} ; aucune écriture effectuée."),
            false,
        ),
        ThreadRefusal::ReceiptInvalid => error(
            "receipt_invalid",
            "Reçu inconnu pour ce fil et cette identité.",
            false,
        ),
        ThreadRefusal::ReceiptObsolete => error(
            "receipt_obsolete",
            "Reçu remplacé par une lecture plus récente ; aucun changement.",
            false,
        ),
        ThreadRefusal::CursorConflict => error(
            "cursor_conflict",
            "Le reçu ne part pas du repère confirmé courant.",
            false,
        ),
        ThreadRefusal::RangeUnavailable => error(
            "range_unavailable",
            "to_seq dépasse la dernière entrée du fil.",
            false,
        ),
    }
}

fn storage_error(error_value: StoreError) -> Value {
    log::error!("fils 102 : stockage indisponible : {error_value}");
    error(
        "storage_unavailable",
        "Stockage indisponible ; rejouer avec la même clé et la même enveloppe.",
        true,
    )
}

fn page_limit(limit: Option<u32>, default: u32, max: u32) -> Result<u32, Value> {
    match limit {
        None => Ok(default),
        Some(value) if (1..=max).contains(&value) => Ok(value),
        Some(_) => Err(error(
            "invalid_request",
            format!("limit doit être compris entre 1 et {max}."),
            false,
        )),
    }
}

fn validate_body(body: &str) -> Result<(), Value> {
    if body.is_empty() || body.len() > BODY_MAX_BYTES {
        return Err(error(
            "invalid_request",
            format!("body doit faire entre 1 et {BODY_MAX_BYTES} octets UTF-8."),
            false,
        ));
    }
    if body.chars().all(char::is_whitespace) {
        return Err(error(
            "invalid_request",
            "body ne peut pas être vide ou tout blanc.",
            false,
        ));
    }
    Ok(())
}

fn validate_title(title: &str) -> Result<String, Value> {
    let trimmed = title.trim();
    let chars = trimmed.chars().count();
    if chars == 0 || chars > TITLE_MAX_CHARS {
        return Err(error(
            "invalid_request",
            format!("title doit faire entre 1 et {TITLE_MAX_CHARS} caractères."),
            false,
        ));
    }
    Ok(trimmed.to_string())
}

pub(crate) fn require_uuid(value: &str, field: &str) -> Result<String, Value> {
    uuid::Uuid::parse_str(value)
        .ok()
        .map(|parsed| parsed.hyphenated().to_string())
        .filter(|canonical| value.len() == 36 && canonical.eq_ignore_ascii_case(value))
        .ok_or_else(|| {
            error(
                "invalid_request",
                format!("{field} doit être un UUID de 36 caractères avec tirets."),
                false,
            )
        })
}

/// Taille JSON qu'occupera l'entrée dans une page, bornée avant le dépôt.
fn entry_json_len(
    message_id: &str,
    actor: &str,
    body: &str,
    targets: &[String],
    reply_to: Option<u64>,
) -> usize {
    json!({
        "seq": u64::MAX,
        "message_id": message_id,
        "author_id": actor,
        "created_at": i64::MAX,
        "body": body,
        "notify": {"mode": "targets", "targets": targets},
        "reply_to_seq": reply_to,
    })
    .to_string()
    .len()
        + 1
}

fn page_json(
    status: &str,
    thread_id: &str,
    page: crate::store::threads::Page,
    extra: Value,
    notices: Vec<&str>,
) -> Value {
    let mut value = json!({
        "status": status,
        "thread_id": thread_id,
        "through_seq": page.through_seq,
        "snapshot_seq": page.snapshot_seq,
        "has_more": page.has_more,
        "notices": notices,
        "entries": page.entries,
    });
    if let (Value::Object(target), Value::Object(source)) = (&mut value, extra) {
        for (key, item) in source {
            target.insert(key, item);
        }
    }
    value
}

fn wake_json(wake: Option<WakeRow>) -> Value {
    wake.map(|row| row.to_json()).unwrap_or_else(WakeRow::none)
}

/// Point d'entrée unique CLI/MCP. `actor` est l'identité attestée par la
/// connexion ; aucun paramètre ne peut la remplacer.
#[cfg(test)]
pub(crate) fn handle(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    request: ThreadRequest,
    now: i64,
) -> ThreadResult {
    handle_with_change(store, directory, actor, request, now).0
}

/// Le marqueur éphémère reste distinct du reçu JSON : seul Done après commit
/// invalide une consultation humaine. Un rejeu conserve le reçu original.
pub(crate) fn handle_with_change(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    request: ThreadRequest,
    now: i64,
) -> (ThreadResult, Option<String>) {
    let mut changed = None;
    if request.version != THREAD_CONTRACT_VERSION {
        return (
            result(error(
                "unsupported_version",
                format!(
                    "Version {} inconnue ; version attendue {THREAD_CONTRACT_VERSION}.",
                    request.version
                ),
                false,
            )),
            None,
        );
    }
    let reason = match crate::communication::validate_cross_project_reason(
        request.cross_project_reason.as_deref(),
    ) {
        Ok(reason) => reason,
        Err(detail) => {
            return (
                result(error("invalid_cross_project_reason", detail, false)),
                None,
            );
        }
    };
    let outcome = match request.request {
        ThreadAction::AddMembers {
            thread_id,
            members,
            operation_id,
        } => add_members(
            store,
            directory,
            actor,
            &thread_id,
            &members,
            &operation_id,
            reason.as_deref(),
            now,
            &mut changed,
        ),
        ThreadAction::Create {
            title,
            members,
            operation_id,
        } => create(
            store,
            directory,
            actor,
            &title,
            &members,
            &operation_id,
            reason.as_deref(),
            now,
            &mut changed,
        ),
        ThreadAction::List {
            limit,
            after_thread_id,
        } => list(store, actor, limit, after_thread_id.as_deref()),
        ThreadAction::Show { thread_id } => show(store, directory, actor, &thread_id),
        ThreadAction::Post {
            thread_id,
            body,
            notify,
            operation_id,
            reply_to_seq,
            ack_receipt,
            kind,
            supersedes_seq,
        } => post(
            store,
            directory,
            actor,
            &thread_id,
            &body,
            &notify,
            &operation_id,
            reply_to_seq,
            ack_receipt.as_deref(),
            kind,
            supersedes_seq,
            reason.as_deref(),
            now,
            &mut changed,
        ),
        ThreadAction::Read { thread_id, limit } => read(store, actor, &thread_id, limit, now),
        ThreadAction::Ack { thread_id, receipt } => ack(store, actor, &thread_id, &receipt),
        ThreadAction::History {
            thread_id,
            from_seq,
            to_seq,
            limit,
        } => history(store, actor, &thread_id, from_seq, to_seq, limit),
        ThreadAction::Close {
            thread_id,
            operation_id,
        } => close(store, actor, &thread_id, &operation_id, now, &mut changed),
    };
    (result(outcome.unwrap_or_else(|refused| refused)), changed)
}

#[allow(clippy::too_many_arguments)]
fn create(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    title: &str,
    members: &[String],
    operation_id: &str,
    reason: Option<&str>,
    now: i64,
    changed: &mut Option<String>,
) -> Result<Value, Value> {
    let title = validate_title(title)?;
    let operation_id = require_uuid(operation_id, "operation_id")?;
    let mut set: Vec<String> = Vec::with_capacity(members.len() + 1);
    for member in members {
        set.push(require_uuid(member, "members[]")?);
    }
    set.push(actor.to_string());
    set.sort();
    set.dedup();
    if !(MIN_MEMBERS..=MAX_MEMBERS).contains(&set.len()) {
        return Err(error(
            "invalid_request",
            format!("Un fil compte de {MIN_MEMBERS} à {MAX_MEMBERS} membres, créateur inclus."),
            false,
        ));
    }
    let mut parts: Vec<&[u8]> = vec![b"create", b"v1", title.as_bytes()];
    for member in &set {
        parts.push(member.as_bytes());
    }
    if let Some(reason) = reason {
        parts.extend([b"communication-project-v1".as_slice(), reason.as_bytes()]);
    }
    let canonical_hash = digest(&parts);
    // Le reçu durable précède les faits vivants : un changement de projet ou
    // une déconnexion ne retire jamais une opération déjà acceptée.
    if let Some(replay) = store
        .thread_operation_replay(actor, &operation_id, &canonical_hash)
        .map_err(storage_error)?
    {
        return tx_result(replay, changed);
    }
    let unknown: Vec<&String> = set
        .iter()
        .filter(|member| member.as_str() != actor && !directory.known_agent(member))
        .collect();
    if !unknown.is_empty() {
        return Err(error(
            "unknown_member",
            format!(
                "Membres inconnus de l'annuaire : {}",
                unknown
                    .iter()
                    .map(|m| m.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            false,
        ));
    }
    let warnings = directory.communication_scope(actor, &set, reason)?;
    let thread_id = uuid::Uuid::new_v4().hyphenated().to_string();
    let outcome = store
        .thread_create(CreateThread {
            actor,
            operation_id: &operation_id,
            canonical_hash: &canonical_hash,
            thread_id: &thread_id,
            title: &title,
            members: &set,
            project_warnings: &warnings,
            now,
            limits: &LIMITS,
        })
        .map_err(storage_error)?;
    tx_result(outcome, changed)
}

fn tx_result(outcome: ThreadTxOutcome, changed: &mut Option<String>) -> Result<Value, Value> {
    match outcome {
        ThreadTxOutcome::Done(value) => {
            *changed = value["thread_id"].as_str().map(str::to_owned);
            Ok(value)
        }
        ThreadTxOutcome::Replayed(value) | ThreadTxOutcome::NoChange(value) => Ok(value),
        ThreadTxOutcome::Refused(refusal) => Err(refusal_result(refusal)),
    }
}

/// O(log T + M log M), M ≤16. Le reçu précède les faits vivants ; la
/// transaction refuse une union différente de celle autorisée ici.
#[allow(clippy::too_many_arguments)]
fn add_members(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    thread_id: &str,
    members: &[String],
    operation_id: &str,
    reason: Option<&str>,
    now: i64,
    changed: &mut Option<String>,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let operation_id = require_uuid(operation_id, "operation_id")?;
    let mut candidates = members
        .iter()
        .map(|member| require_uuid(member, "members[]"))
        .collect::<Result<Vec<_>, _>>()?;
    candidates.sort();
    candidates.dedup();
    if candidates.is_empty() || candidates.len() > MAX_MEMBERS {
        return Err(error(
            "invalid_request",
            "members exige de 1 à 16 UUID distincts.",
            false,
        ));
    }
    let mut parts: Vec<&[u8]> = vec![b"add_members", b"v1", thread_id.as_bytes()];
    parts.extend(candidates.iter().map(|member| member.as_bytes()));
    if let Some(reason) = reason {
        parts.extend([b"communication-project-v1".as_slice(), reason.as_bytes()]);
    }
    let canonical_hash = digest(&parts);
    if let Some(replay) = store
        .thread_operation_replay(actor, &operation_id, &canonical_hash)
        .map_err(storage_error)?
    {
        return tx_result(replay, changed);
    }
    let view = store
        .thread_show(actor, &thread_id)
        .map_err(storage_error)?
        .map_err(refusal_result)?;
    if view.thread.creator_id != actor {
        return Err(refusal_result(ThreadRefusal::CreatorRequired));
    }
    if view.thread.closed_at.is_some() {
        return Err(refusal_result(ThreadRefusal::ThreadClosed));
    }
    let mut union = view.members.clone();
    union.extend_from_slice(&candidates);
    union.sort();
    union.dedup();
    if union.len() > MAX_MEMBERS {
        return Err(refusal_result(ThreadRefusal::CapacityExceeded("members")));
    }
    let existing: std::collections::HashSet<_> = view.members.iter().collect();
    for member in candidates
        .iter()
        .filter(|member| !existing.contains(member))
    {
        if !directory.known_agent(member) {
            return Err(error(
                "unknown_member",
                "Membre candidat inconnu de l'annuaire.",
                false,
            ));
        }
    }
    let warnings = directory.communication_scope(actor, &union, reason)?;
    tx_result(
        store
            .thread_add_members(AddMembers {
                actor,
                operation_id: &operation_id,
                canonical_hash: &canonical_hash,
                thread_id: &thread_id,
                candidates: &candidates,
                expected_members: &union,
                project_warnings: &warnings,
                now,
            })
            .map_err(storage_error)?,
        changed,
    )
}

fn list(
    store: &Store,
    actor: &str,
    limit: Option<u32>,
    after: Option<&str>,
) -> Result<Value, Value> {
    let limit = page_limit(limit, LIST_DEFAULT, LIST_MAX)?;
    let after = match after {
        Some(value) => Some(require_uuid(value, "after_thread_id")?),
        None => None,
    };
    let (threads, next_after) = store
        .thread_list(actor, limit, after.as_deref())
        .map_err(storage_error)?;
    Ok(json!({
        "status": "listed",
        "threads": threads.iter().map(|thread| json!({
            "thread_id": thread.thread_id,
            "title": thread.title,
            "creator_id": thread.creator_id,
            "state": if thread.closed { "closed" } else { "open" },
            "last_seq": thread.last_seq,
        })).collect::<Vec<_>>(),
        "next_after": next_after,
    }))
}

fn show(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    thread_id: &str,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let view = store
        .thread_show(actor, &thread_id)
        .map_err(storage_error)?
        .map_err(refusal_result)?;
    let members: Vec<Value> = view
        .members
        .iter()
        .map(|member| {
            let facts = directory.member_facts(member);
            json!({
                "agent_id": member,
                "connected": facts.connected,
                "thread_notice_version": facts.thread_notice_version,
            })
        })
        .collect();
    Ok(json!({
        "status": "shown",
        "thread_id": view.thread.thread_id,
        "title": view.thread.title,
        "creator_id": view.thread.creator_id,
        "state": if view.thread.closed_at.is_some() { "closed" } else { "open" },
        "created_at": view.thread.created_at,
        "closed_at": view.thread.closed_at,
        "last_seq": view.thread.last_seq,
        "members": members,
        "own_acked_seq": view.own.acked_seq,
        "own_wake": wake_json(view.own_wake),
    }))
}

#[allow(clippy::too_many_arguments)]
fn post(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    thread_id: &str,
    body: &str,
    notify: &ThreadNotify,
    operation_id: &str,
    reply_to_seq: Option<u64>,
    ack_receipt: Option<&str>,
    kind: Option<ThreadEntryKind>,
    supersedes_seq: Option<u64>,
    reason: Option<&str>,
    now: i64,
    changed: &mut Option<String>,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let operation_id = require_uuid(operation_id, "operation_id")?;
    validate_body(body)?;
    if kind == Some(ThreadEntryKind::History) {
        if !matches!(notify, ThreadNotify::Targets(targets) if targets.is_empty())
            || supersedes_seq.is_some()
        {
            return Err(error(
                "invalid_request",
                "history exige notify:[] et aucun remplacement.",
                false,
            ));
        }
    } else if kind.is_some() && body.len() > ACTION_MAX_BYTES {
        return Err(error(
            "invalid_request",
            "Action, blocage ou décision limité à 2048 octets ; publier les preuves en history.",
            false,
        ));
    }
    if supersedes_seq.is_some() && kind.is_none() {
        return Err(error(
            "invalid_request",
            "supersedes_seq exige une classe déclarée.",
            false,
        ));
    }
    if reply_to_seq == Some(0) {
        return Err(error(
            "invalid_request",
            "reply_to_seq doit être strictement positif.",
            false,
        ));
    }
    let ack_receipt = match ack_receipt {
        Some(receipt) => Some(require_uuid(receipt, "ack_receipt")?),
        None => None,
    };
    let mut notices: Vec<&str> = Vec::new();
    let (spec, mode, canonical_targets): (NotifySpec, &str, Vec<String>) = match notify {
        ThreadNotify::All(_) => (NotifySpec::All, "all", Vec::new()),
        ThreadNotify::Targets(wanted) => {
            let mut targets = Vec::with_capacity(wanted.len());
            for target in wanted {
                let target = require_uuid(target, "notify[]")?;
                if target == actor {
                    if !notices.contains(&"self_mention_ignored") {
                        notices.push("self_mention_ignored");
                    }
                    continue;
                }
                targets.push(target);
            }
            targets.sort();
            targets.dedup();
            if targets.is_empty() {
                (NotifySpec::None, "none", Vec::new())
            } else {
                (NotifySpec::Targets(targets.clone()), "targets", targets)
            }
        }
    };
    let message_id = uuid::Uuid::new_v4().hyphenated().to_string();
    let entry_len = entry_json_len(&message_id, actor, body, &canonical_targets, reply_to_seq)
        + if kind.is_some() { 128 } else { 0 };
    if entry_len > ENTRY_MAX_JSON_BYTES {
        return Err(error(
            "entry_too_large",
            format!("Entrée sérialisée de {entry_len} octets ; maximum {ENTRY_MAX_JSON_BYTES}."),
            false,
        ));
    }
    let reply_bytes = reply_to_seq.map(|value| value.to_be_bytes());
    let mut parts: Vec<&[u8]> = vec![
        b"post",
        b"v1",
        thread_id.as_bytes(),
        body.as_bytes(),
        mode.as_bytes(),
    ];
    for target in &canonical_targets {
        parts.push(target.as_bytes());
    }
    parts.push(b"reply_to");
    if let Some(bytes) = reply_bytes.as_ref() {
        parts.push(bytes);
    }
    parts.push(b"ack");
    if let Some(receipt) = ack_receipt.as_deref() {
        parts.push(receipt.as_bytes());
    }
    let supersedes_bytes = supersedes_seq.map(u64::to_be_bytes);
    if let Some(kind) = kind {
        // Suffixe absent pour les anciens dépôts : leur empreinte reste exacte.
        parts.extend([b"entry-control-v1".as_slice(), kind.as_str().as_bytes()]);
        if let Some(bytes) = &supersedes_bytes {
            parts.push(bytes);
        }
    }
    if let Some(reason) = reason {
        parts.extend([b"communication-project-v1".as_slice(), reason.as_bytes()]);
    }
    let canonical_hash = digest(&parts);
    if let Some(replay) = store
        .thread_operation_replay(actor, &operation_id, &canonical_hash)
        .map_err(storage_error)?
    {
        return tx_result(replay, changed);
    }
    // Le fil conserve ses lecteurs, même quand notify est vide. Vérifier
    // l'accès avant la portée pour ne rien révéler à un non-membre.
    let view = store
        .thread_show(actor, &thread_id)
        .map_err(storage_error)?
        .map_err(refusal_result)?;
    let warnings = directory.communication_scope(actor, &view.members, reason)?;
    let outcome = store
        .thread_post(PostEntry {
            actor,
            operation_id: &operation_id,
            canonical_hash: &canonical_hash,
            thread_id: &thread_id,
            message_id: &message_id,
            body,
            notify: &spec,
            notices: &notices,
            project_warnings: &warnings,
            reply_to_seq,
            ack_receipt: ack_receipt.as_deref(),
            kind,
            supersedes_seq,
            now,
            limits: &LIMITS,
        })
        .map_err(storage_error)?;
    tx_result(outcome, changed)
}

fn read(
    store: &Store,
    actor: &str,
    thread_id: &str,
    limit: Option<u32>,
    now: i64,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let limit = page_limit(limit, PAGE_DEFAULT, PAGE_MAX)?;
    let receipt_id = uuid::Uuid::new_v4().hyphenated().to_string();
    let outcome = store
        .thread_read(&ReadRequest {
            actor,
            thread_id: &thread_id,
            limit,
            byte_budget: PAGE_MAX_BYTES - PAGE_METADATA_RESERVE_BYTES,
            receipt_id: &receipt_id,
            receipt_ttl_secs: RECEIPT_TTL_SECS,
            now,
        })
        .map_err(storage_error)?;
    match outcome {
        ReadOutcome::Refused(refusal) => Err(refusal_result(refusal)),
        ReadOutcome::Page {
            page,
            receipt,
            expires_at,
            requested_limit,
            replayed,
        } => {
            let notices = if replayed {
                vec!["pending_receipt_replayed"]
            } else {
                Vec::new()
            };
            let base_seq = page.base_seq;
            Ok(page_json(
                "read",
                &thread_id,
                page,
                json!({
                    "base_seq": base_seq,
                    "receipt": receipt,
                    "expires_at": expires_at,
                    "requested_limit": requested_limit,
                }),
                notices,
            ))
        }
    }
}

fn ack(store: &Store, actor: &str, thread_id: &str, receipt: &str) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let receipt = require_uuid(receipt, "receipt")
        .map_err(|_| refusal_result(ThreadRefusal::ReceiptInvalid))?;
    match store
        .thread_ack(actor, &thread_id, &receipt)
        .map_err(storage_error)?
    {
        AckOutcome::Acknowledged { acked_seq, wake } => Ok(json!({
            "status": "acknowledged",
            "thread_id": thread_id,
            "acked_seq": acked_seq,
            "own_wake": wake_json(wake),
        })),
        AckOutcome::AlreadyAcknowledged { acked_seq, wake } => Ok(json!({
            "status": "already_acknowledged",
            "thread_id": thread_id,
            "acked_seq": acked_seq,
            "own_wake": wake_json(wake),
        })),
        AckOutcome::Refused(refusal) => Err(refusal_result(refusal)),
    }
}

fn history(
    store: &Store,
    actor: &str,
    thread_id: &str,
    from_seq: Option<u64>,
    to_seq: Option<u64>,
    limit: Option<u32>,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let limit = page_limit(limit, PAGE_DEFAULT, PAGE_MAX)?;
    let from_seq = from_seq.unwrap_or(1);
    if from_seq == 0 {
        return Err(error(
            "invalid_request",
            "from_seq doit être strictement positif.",
            false,
        ));
    }
    if let Some(to_seq) = to_seq
        && to_seq + 1 < from_seq
    {
        return Err(error(
            "invalid_request",
            "to_seq ne peut pas précéder from_seq - 1.",
            false,
        ));
    }
    match store
        .thread_history(
            actor,
            &thread_id,
            from_seq,
            to_seq,
            limit,
            PAGE_MAX_BYTES - PAGE_METADATA_RESERVE_BYTES,
        )
        .map_err(storage_error)?
    {
        HistoryOutcome::Refused(refusal) => Err(refusal_result(refusal)),
        HistoryOutcome::Page(page) => {
            let next_from_seq = page.has_more.then_some(page.through_seq + 1);
            Ok(page_json(
                "history",
                &thread_id,
                page,
                json!({"from_seq": from_seq, "next_from_seq": next_from_seq}),
                Vec::new(),
            ))
        }
    }
}

fn close(
    store: &Store,
    actor: &str,
    thread_id: &str,
    operation_id: &str,
    now: i64,
    changed: &mut Option<String>,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let operation_id = require_uuid(operation_id, "operation_id")?;
    let canonical_hash = digest(&[b"close", b"v1", thread_id.as_bytes()]);
    let outcome = store
        .thread_close(CloseThread {
            actor,
            operation_id: &operation_id,
            canonical_hash: &canonical_hash,
            thread_id: &thread_id,
            now,
        })
        .map_err(storage_error)?;
    tx_result(outcome, changed)
}

/// Corps neutre de l'alerte : aucun texte de contribution, aucun titre.
pub(crate) fn notice_body(thread_id: &str, through_seq: u64) -> String {
    format!(
        "Sollicitation dans le fil {thread_id}, nouveautés jusqu'à {through_seq}. \
         Lis les nouveautés avec bridget_thread/read. Les références sans body \
         sont des preuves historiques ou des consignes remplacées : history permet \
         de les relire, jamais de les exécuter comme consignes actuelles. Confirme \
         chaque page reçue, puis contrôle has_more et les pages suivantes avant \
         d'agir. L'ACK n'accepte aucune mission. Publie preuves avec kind=history \
         et notify:[], consignes courtes avec kind=action/blocker/decision et \
         supersedes_seq si remplacement. Ne réponds pas par message direct à cette alerte."
    )
}

/// Identifiant déterministe d'une alerte : paire fil/membre/génération, distinct
/// des identifiants de contribution.
pub(crate) fn notice_message_id(thread_id: &str, agent_id: &str, generation: u64) -> String {
    format!("thread-notice:{thread_id}:{agent_id}:{generation}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID_A: &str = "14700000-aaaa-4aaa-8aaa-00000000000a";
    const UUID_B: &str = "14700000-bbbb-4bbb-8bbb-00000000000b";
    const UUID_T: &str = "14700000-cccc-4ccc-8ccc-00000000000c";
    const UUID_OP: &str = "14700000-dddd-4ddd-8ddd-00000000000d";

    struct UuidDirectory;
    impl Directory for UuidDirectory {
        fn known_agent(&self, id: &str) -> bool {
            [UUID_A, UUID_B].contains(&id)
        }
        fn member_facts(&self, _: &str) -> MemberFacts {
            MemberFacts::default()
        }
        fn communication_scope(
            &self,
            _: &str,
            _: &[String],
            _: Option<&str>,
        ) -> Result<Vec<ProjectWarning>, Value> {
            Ok(Vec::new())
        }
    }

    struct UuidFixture {
        store: Store,
        root: std::path::PathBuf,
    }
    impl UuidFixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("uuid147-{}", uuid::Uuid::new_v4().simple()));
            std::fs::create_dir(&root).unwrap();
            let store = Store::open(&root.join("db")).unwrap();
            assert!(matches!(
                store
                    .thread_create(CreateThread {
                        actor: UUID_A,
                        operation_id: "14700000-eeee-4eee-8eee-00000000000e",
                        canonical_hash: "seed",
                        thread_id: UUID_T,
                        title: "Titre É exact",
                        members: &[UUID_A.into(), UUID_B.into()],
                        project_warnings: &[],
                        now: 1,
                        limits: &LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
            Self { store, root }
        }
        fn run(&self, actor: &str, request: ThreadAction) -> (Value, Option<String>) {
            let (result, changed) = handle_with_change(
                &self.store,
                &UuidDirectory,
                actor,
                ThreadRequest {
                    version: 1,
                    cross_project_reason: Some("Motif É inchangé".into()),
                    request,
                },
                2,
            );
            (result.result, changed)
        }
        fn post(&self, upper: bool, notify: Vec<String>) -> (Value, Option<String>) {
            self.run(
                UUID_A,
                ThreadAction::Post {
                    thread_id: if upper {
                        UUID_T.to_uppercase()
                    } else {
                        UUID_T.into()
                    },
                    operation_id: if upper {
                        UUID_OP.to_uppercase()
                    } else {
                        UUID_OP.into()
                    },
                    body: "É 🦀\r\n  Exact\n\n".into(),
                    notify: ThreadNotify::Targets(notify),
                    reply_to_seq: None,
                    ack_receipt: None,
                    kind: None,
                    supersedes_seq: None,
                },
            )
        }
    }
    impl Drop for UuidFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn spec147_members_add_normalized_replay_no_change_and_cursor_zero() {
        let fixture = UuidFixture::new();
        struct Known;
        impl Directory for Known {
            fn known_agent(&self, _: &str) -> bool {
                true
            }
            fn member_facts(&self, _: &str) -> MemberFacts {
                MemberFacts::default()
            }
            fn communication_scope(
                &self,
                _: &str,
                _: &[String],
                _: Option<&str>,
            ) -> Result<Vec<ProjectWarning>, Value> {
                Ok(vec![])
            }
        }
        let c = "14700000-ffff-4fff-8fff-00000000000f";
        let run = |op: &str, members: Value| {
            let request = serde_json::from_value(json!({"version":1,"request":{"action":"add_members","thread_id":UUID_T.to_uppercase(),"members":members,"operation_id":op}})).unwrap();
            let (result, changed) = handle_with_change(&fixture.store, &Known, UUID_A, request, 9);
            (result.result, changed)
        };
        fixture.post(false, vec![]);
        let before = fixture.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
        let op = "14700000-eeee-4eee-8eee-00000000000f";
        let (added, changed) = run(&op.to_uppercase(), json!([c.to_uppercase(), c, UUID_B]));
        assert_eq!(added["status"], "members_added");
        assert_eq!(added["added_members"], json!([c]));
        assert_eq!(changed.as_deref(), Some(UUID_T));
        let (replayed, changed) = run(op, json!([UUID_B, c, c.to_uppercase()]));
        assert_eq!(replayed, added);
        assert!(changed.is_none());
        let (noop, changed) = run("14700000-eeee-4eee-8eee-000000000010", json!([c]));
        assert_eq!(noop["status"], "no_change");
        assert!(changed.is_none());
        let view = fixture.store.thread_show(c, UUID_T).unwrap().unwrap();
        assert_eq!(view.own.acked_seq, 0);
        assert_eq!(view.thread, before.thread);
        assert!(view.own_wake.is_none());
        let (bad, changed) = run(op, json!([UUID_B]));
        assert_eq!(bad["code"], "envelope_mismatch");
        assert!(changed.is_none());
    }

    #[test]
    fn spec147_members_refusals_and_project_guard_cover_whole_audience() {
        let f = UuidFixture::new();
        struct Checked(std::cell::RefCell<Vec<String>>);
        impl Directory for Checked {
            fn known_agent(&self, id: &str) -> bool {
                id != "14700000-ffff-4fff-8fff-000000000099"
            }
            fn member_facts(&self, _: &str) -> MemberFacts {
                MemberFacts::default()
            }
            fn communication_scope(
                &self,
                _: &str,
                members: &[String],
                reason: Option<&str>,
            ) -> Result<Vec<ProjectWarning>, Value> {
                *self.0.borrow_mut() = members.to_vec();
                if reason.is_none() {
                    return Err(error(
                        "cross_project_reason_required",
                        "motif requis",
                        false,
                    ));
                }
                Ok(vec![])
            }
        }
        let directory = Checked(std::cell::RefCell::new(vec![]));
        let c = "14700000-ffff-4fff-8fff-00000000000f";
        let run = |actor: &str, members: Value, reason: Option<&str>| {
            let request = ThreadRequest {version:1,cross_project_reason:reason.map(str::to_string),request:serde_json::from_value(json!({"action":"add_members","thread_id":UUID_T,"members":members,"operation_id":uuid::Uuid::new_v4().to_string()})).unwrap()};
            let (result, changed) = handle_with_change(&f.store, &directory, actor, request, 2);
            (result.result, changed)
        };
        let before = f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
        for (actor, members, code) in [
            (UUID_B, json!([c]), "creator_required"),
            (c, json!([c]), "thread_unavailable"),
            (UUID_A, json!([]), "invalid_request"),
            (UUID_A, json!(["bad"]), "invalid_request"),
            (
                UUID_A,
                json!(["14700000-ffff-4fff-8fff-000000000099"]),
                "unknown_member",
            ),
        ] {
            let (result, changed) = run(actor, members, Some("mandat"));
            assert_eq!(result["code"], code, "{result}");
            assert!(changed.is_none());
            assert_eq!(
                f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap(),
                before
            );
        }
        let (refused, changed) = run(UUID_A, json!([c]), None);
        assert_eq!(refused["code"], "cross_project_reason_required");
        assert!(changed.is_none());
        assert_eq!(
            &*directory.0.borrow(),
            &vec![UUID_A.to_string(), UUID_B.to_string(), c.to_string()]
        );
        let candidates: Vec<_> = (0..14)
            .map(|n| format!("14700000-ffff-4fff-8fff-{n:012x}"))
            .collect();
        assert_eq!(
            run(UUID_A, json!(candidates), Some("mandat")).0["status"],
            "members_added"
        );
        let view = f.store.thread_show(UUID_A, UUID_T).unwrap().unwrap();
        assert_eq!(view.members.len(), 16);
        assert_eq!(
            run(UUID_A, json!([c, c.to_uppercase()]), Some("mandat")).0["code"],
            "capacity_exceeded"
        );
        assert_eq!(
            run(
                UUID_A,
                json!([candidates[0].clone(), candidates[0].to_uppercase()]),
                Some("mandat")
            )
            .0["status"],
            "no_change"
        );
        f.run(
            UUID_A,
            ThreadAction::Close {
                thread_id: UUID_T.into(),
                operation_id: uuid::Uuid::new_v4().to_string(),
            },
        );
        assert_eq!(
            run(UUID_A, json!([candidates[0].clone()]), Some("mandat")).0["code"],
            "thread_closed"
        );
    }

    #[test]
    fn spec147_uuid_create_replays_across_case_without_second_commit() {
        for upper_first in [false, true] {
            let f = UuidFixture::new();
            let action = |upper: bool| ThreadAction::Create {
                title: "Titre É exact".into(),
                members: vec![if upper {
                    UUID_B.to_uppercase()
                } else {
                    UUID_B.into()
                }],
                operation_id: if upper {
                    UUID_OP.to_uppercase()
                } else {
                    UUID_OP.into()
                },
            };
            let (first, changed) = f.run(UUID_A, action(upper_first));
            assert_eq!(first["status"], "created", "{first}");
            assert_eq!(changed.as_deref(), first["thread_id"].as_str());
            let before = f.store.connection().total_changes();
            let (replay, changed) = f.run(UUID_A, action(!upper_first));
            assert_eq!(replay, first);
            assert!(changed.is_none());
            assert_eq!(f.store.connection().total_changes(), before);
        }
    }

    #[test]
    fn spec147_uuid_post_replays_across_case_and_keeps_exact_body() {
        for upper_first in [false, true] {
            let f = UuidFixture::new();
            let (first, changed) = f.post(
                upper_first,
                vec![if upper_first {
                    UUID_B.to_uppercase()
                } else {
                    UUID_B.into()
                }],
            );
            assert_eq!(first["status"], "posted", "{first}");
            assert_eq!(changed.as_deref(), Some(UUID_T));
            let before = f.store.connection().total_changes();
            let (replay, changed) = f.post(
                !upper_first,
                vec![if upper_first {
                    UUID_B.into()
                } else {
                    UUID_B.to_uppercase()
                }],
            );
            assert_eq!(replay, first);
            assert!(changed.is_none());
            assert_eq!(f.store.connection().total_changes(), before);
            let (history, _) = f.run(
                UUID_A,
                ThreadAction::History {
                    thread_id: UUID_T.into(),
                    from_seq: None,
                    to_seq: None,
                    limit: None,
                },
            );
            assert_eq!(history["entries"][0]["body"], "É 🦀\r\n  Exact\n\n");
        }
    }

    #[test]
    fn spec147_uuid_close_replays_across_case_without_second_commit() {
        for upper_first in [false, true] {
            let f = UuidFixture::new();
            let action = |upper: bool| ThreadAction::Close {
                thread_id: if upper {
                    UUID_T.to_uppercase()
                } else {
                    UUID_T.into()
                },
                operation_id: if upper {
                    UUID_OP.to_uppercase()
                } else {
                    UUID_OP.into()
                },
            };
            let (first, changed) = f.run(UUID_A, action(upper_first));
            assert_eq!(first["status"], "closed", "{first}");
            assert_eq!(changed.as_deref(), Some(UUID_T));
            let before = f.store.connection().total_changes();
            let (replay, changed) = f.run(UUID_A, action(!upper_first));
            assert_eq!(replay, first);
            assert!(changed.is_none());
            assert_eq!(f.store.connection().total_changes(), before);
        }
    }

    #[test]
    fn spec147_uuid_ack_receipt_is_case_insensitive_but_invalid_stays_typed() {
        let f = UuidFixture::new();
        assert_eq!(f.post(false, vec![]).0["status"], "posted");
        let (page, _) = f.run(
            UUID_B,
            ThreadAction::Read {
                thread_id: UUID_T.into(),
                limit: None,
            },
        );
        let receipt = page["receipt"].as_str().unwrap();
        let (ack, _) = f.run(
            UUID_B,
            ThreadAction::Ack {
                thread_id: UUID_T.to_uppercase(),
                receipt: receipt.to_uppercase(),
            },
        );
        assert_eq!(ack["status"], "acknowledged", "{ack}");
        let before = f.store.connection().total_changes();
        let (again, _) = f.run(
            UUID_B,
            ThreadAction::Ack {
                thread_id: UUID_T.into(),
                receipt: receipt.into(),
            },
        );
        assert_eq!(again["status"], "already_acknowledged");
        assert_eq!(f.store.connection().total_changes(), before);
        let (bad, _) = f.run(
            UUID_B,
            ThreadAction::Ack {
                thread_id: UUID_T.into(),
                receipt: "bad".into(),
            },
        );
        assert_eq!(bad["code"], "receipt_invalid");
        assert_eq!(f.store.connection().total_changes(), before);
        let g = UuidFixture::new();
        assert_eq!(g.post(false, vec![]).0["status"], "posted");
        let (page, _) = g.run(
            UUID_B,
            ThreadAction::Read {
                thread_id: UUID_T.into(),
                limit: None,
            },
        );
        let receipt = page["receipt"].as_str().unwrap();
        let action = |upper: bool| ThreadAction::Post {
            thread_id: if upper {
                UUID_T.to_uppercase()
            } else {
                UUID_T.into()
            },
            operation_id: if upper {
                UUID_OP.to_uppercase()
            } else {
                UUID_OP.into()
            },
            body: "Réponse exacte".into(),
            notify: ThreadNotify::Targets(vec![]),
            reply_to_seq: None,
            ack_receipt: Some(if upper {
                receipt.to_uppercase()
            } else {
                receipt.into()
            }),
            kind: None,
            supersedes_seq: None,
        };
        let (posted, changed) = g.run(UUID_B, action(true));
        assert_eq!(posted["status"], "posted", "{posted}");
        assert_eq!(changed.as_deref(), Some(UUID_T));
        let before = g.store.connection().total_changes();
        let (replayed, changed) = g.run(UUID_B, action(false));
        assert_eq!(replayed, posted);
        assert!(changed.is_none());
        assert_eq!(g.store.connection().total_changes(), before);
    }

    #[test]
    fn spec147_uuid_reads_show_history_use_the_same_thread() {
        let f = UuidFixture::new();
        assert_eq!(f.post(false, vec![]).0["status"], "posted");
        for action in [
            ThreadAction::Show {
                thread_id: UUID_T.to_uppercase(),
            },
            ThreadAction::History {
                thread_id: UUID_T.to_uppercase(),
                from_seq: None,
                to_seq: None,
                limit: None,
            },
            ThreadAction::Read {
                thread_id: UUID_T.to_uppercase(),
                limit: None,
            },
        ] {
            let (result, changed) = f.run(UUID_B, action);
            assert_ne!(result["status"], "error", "{result}");
            assert_eq!(result["thread_id"], UUID_T);
            assert!(changed.is_none());
        }
    }

    #[test]
    fn spec147_uuid_members_and_targets_deduplicate_before_hash_and_membership() {
        let f = UuidFixture::new();
        let mixed_b = "14700000-bBbB-4bBb-8BbB-00000000000B";
        let (first, _) = f.run(
            UUID_A,
            ThreadAction::Create {
                title: "Identités".into(),
                operation_id: "14700000-dDdD-4dDd-8DdD-00000000000D".into(),
                members: vec![UUID_A.to_uppercase(), mixed_b.into(), UUID_B.into()],
            },
        );
        assert_eq!(first["status"], "created", "{first}");
        let before = f.store.connection().total_changes();
        let (replay, changed) = f.run(
            UUID_A,
            ThreadAction::Create {
                title: "Identités".into(),
                operation_id: UUID_OP.into(),
                members: vec![UUID_B.into()],
            },
        );
        assert_eq!(replay, first);
        assert!(changed.is_none());
        assert_eq!(f.store.connection().total_changes(), before);
        // Une autre clé est requise pour Post : la clé Create ne change pas d'enveloppe.
        let g = UuidFixture::new();
        let (posted, _) = g.post(
            true,
            vec![UUID_A.to_uppercase(), mixed_b.into(), UUID_B.into()],
        );
        assert_eq!(posted["status"], "posted", "{posted}");
        let (replayed, changed) = g.post(false, vec![UUID_A.into(), UUID_B.into()]);
        assert_eq!(replayed, posted);
        assert!(changed.is_none());
    }

    #[test]
    fn spec147_uuid_invalid_forms_never_mutate_and_human_canonical_stays_strict() {
        let f = UuidFixture::new();
        let before = f.store.connection().total_changes();
        for invalid in [
            UUID_T.replace('-', ""),
            format!("{{{UUID_T}}}"),
            format!("urn:uuid:{UUID_T}"),
            format!(" {UUID_T}"),
            "invalid".into(),
        ] {
            let actions = [
                ThreadAction::Show {
                    thread_id: invalid.clone(),
                },
                ThreadAction::Create {
                    title: "Refus".into(),
                    members: vec![UUID_B.into()],
                    operation_id: invalid.clone(),
                },
                ThreadAction::Create {
                    title: "Refus membre".into(),
                    members: vec![invalid.clone()],
                    operation_id: UUID_OP.into(),
                },
                ThreadAction::Close {
                    thread_id: UUID_T.into(),
                    operation_id: invalid.clone(),
                },
                ThreadAction::Post {
                    thread_id: UUID_T.into(),
                    operation_id: invalid.clone(),
                    body: "Exact".into(),
                    notify: ThreadNotify::Targets(vec![]),
                    reply_to_seq: None,
                    ack_receipt: None,
                    kind: None,
                    supersedes_seq: None,
                },
                ThreadAction::Post {
                    thread_id: UUID_T.into(),
                    operation_id: UUID_OP.into(),
                    body: "Exact".into(),
                    notify: ThreadNotify::Targets(vec![invalid.clone()]),
                    reply_to_seq: None,
                    ack_receipt: None,
                    kind: None,
                    supersedes_seq: None,
                },
                ThreadAction::Post {
                    thread_id: UUID_T.into(),
                    operation_id: UUID_OP.into(),
                    body: "Exact".into(),
                    notify: ThreadNotify::Targets(vec![]),
                    reply_to_seq: None,
                    ack_receipt: Some(invalid.clone()),
                    kind: None,
                    supersedes_seq: None,
                },
            ];
            for action in actions {
                let (bad, changed) = f.run(UUID_A, action);
                assert_eq!(bad["code"], "invalid_request", "{bad}");
                assert!(changed.is_none());
            }
        }
        assert_eq!(f.store.connection().total_changes(), before);
        assert!(canonical_uuid(&UUID_T.to_uppercase()).is_none());
        assert_eq!(canonical_uuid(UUID_T).as_deref(), Some(UUID_T));
        assert!(parse_recent_cursor(&format!("2:{}", UUID_T.to_uppercase())).is_err());
        assert!(parse_recent_cursor(&format!("2:{UUID_T}")).is_ok());
    }

    #[test]
    fn spec147_done_keeps_a_change_marker_but_replay_keeps_only_the_receipt() {
        let value = json!({"status":"posted","thread_id":"14700000-0000-4000-8000-000000000001"});
        let mut changed = None;
        assert_eq!(
            tx_result(ThreadTxOutcome::Done(value.clone()), &mut changed).unwrap(),
            value
        );
        assert_eq!(changed.as_deref(), value["thread_id"].as_str());
        let mut replayed = None;
        assert_eq!(
            tx_result(ThreadTxOutcome::Replayed(value.clone()), &mut replayed).unwrap(),
            value
        );
        assert!(replayed.is_none());
    }

    #[test]
    fn spec146_real_projection_descending_pages_exact_bodies_and_fixed_snapshot() {
        use bridget_transport::protocol::HumanThreadViewAction as A;
        let root =
            std::env::temp_dir().join(format!("h146-export-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        let db = root.join("db");
        let store = Store::open(&db).unwrap();
        let mut profiles = crate::agent_profile::AgentProfileStore::open(&db).unwrap();
        let members = vec![
            "14600000-0000-4000-8000-000000000001".to_string(),
            "14600000-0000-4000-8000-000000000002".to_string(),
        ];
        profiles.ensure_agent_ids(members.iter().cloned()).unwrap();
        for (n, id) in members.iter().enumerate() {
            store.connection().execute("UPDATE agent_profiles SET display_name=?2,display_name_normalized=?2 WHERE agent_id=?1",rusqlite::params![id,format!("recette-{n}")]).unwrap();
        }
        let actor = &members[0];
        let thread = "14600000-0000-4001-8000-000000000001";
        for n in 1..=3 {
            let id = format!("14600000-0000-4001-8000-{n:012}");
            assert!(matches!(
                store
                    .thread_create(CreateThread {
                        actor,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: &id,
                        thread_id: &id,
                        title: &format!("Recette {n}"),
                        members: &members,
                        project_warnings: &[],
                        now: n,
                        limits: &LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
        }
        let mut bodies = serde_json::Map::new();
        for n in 1..=137 {
            let body = format!(
                "Message {n} é 🦀\r\n  espaces conservés\n\n{}",
                "0123456789 ".repeat(155)
            );
            bodies.insert(n.to_string(), json!(body));
            assert!(matches!(
                store
                    .thread_post(PostEntry {
                        project_warnings: &[],
                        actor,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: &format!("entry-{n}"),
                        thread_id: thread,
                        message_id: &uuid::Uuid::new_v4().to_string(),
                        body: &body,
                        notify: &NotifySpec::None,
                        notices: &[],
                        reply_to_seq: None,
                        ack_receipt: None,
                        kind: Some(if n == 1 || n == 137 {
                            ThreadEntryKind::Action
                        } else {
                            ThreadEntryKind::History
                        }),
                        supersedes_seq: (n == 137).then_some(1),
                        now: 100 + n,
                        limits: &LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
        }
        let before = store.connection().total_changes();
        let listed = human_view(
            &store,
            actor,
            &A::ListRecent {
                limit: Some(2),
                after: None,
            },
        );
        for action in [
            A::ListRecent {
                limit: Some(0),
                after: None,
            },
            A::ListRecent {
                limit: Some(101),
                after: None,
            },
            A::ListRecent {
                limit: Some(1),
                after: Some("not-a-cursor".into()),
            },
            A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: Some(9_007_199_254_740_992),
                to_seq: None,
                limit: Some(1),
            },
            A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: None,
                to_seq: Some(9_007_199_254_740_992),
                limit: Some(1),
            },
            A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: None,
                to_seq: None,
                limit: Some(201),
            },
        ] {
            let refused = serde_json::to_value(human_view(&store, actor, &action)).unwrap();
            assert_eq!(refused["result"]["code"], "invalid_request");
            assert!(refused["subject"].is_null());
        }
        let shown = human_view(
            &store,
            actor,
            &A::Show {
                thread_id: thread.into(),
            },
        );
        let mut pages = Vec::new();
        let mut cursor = None;
        let mut all = Vec::new();
        loop {
            let projected = human_view(
                &store,
                actor,
                &A::HistoryRecent {
                    thread_id: thread.into(),
                    before_seq: cursor,
                    to_seq: Some(137),
                    limit: Some(100),
                },
            );
            let value = serde_json::to_value(&projected).unwrap();
            let result = &value["result"];
            assert_eq!(result["status"], "history_recent");
            assert_eq!(result["snapshot_seq"], 137);
            for e in result["entries"].as_array().unwrap() {
                let seq = e["seq"].as_u64().unwrap();
                assert_eq!(e["body"], bodies[&seq.to_string()]);
                all.push(seq);
                if seq == 1 {
                    assert_eq!(e["superseded_by_seq"], 137);
                }
            }
            cursor = result["next_before_seq"].as_u64();
            pages.push(value);
            if cursor.is_none() {
                break;
            }
        }
        assert!(pages.len() > 3);
        assert_eq!(all, (1..=137).rev().collect::<Vec<_>>());
        assert_eq!(store.connection().total_changes(), before);
        let output = json!({"listed":listed,"shown":shown,"pages":pages,"body_by_seq":bodies});
        if let Ok(directory) = std::env::var("BRIDGET_SPEC146_EXPORT_DIR") {
            std::fs::write(
                std::path::Path::new(&directory).join("interop-146.json"),
                serde_json::to_vec_pretty(&output).unwrap(),
            )
            .unwrap();
        }
        // Publication après ouverture : la même borne ne voit pas138.
        assert!(matches!(
            store
                .thread_post(PostEntry {
                    project_warnings: &[],
                    actor,
                    operation_id: &uuid::Uuid::new_v4().to_string(),
                    canonical_hash: "after-snapshot",
                    thread_id: thread,
                    message_id: &uuid::Uuid::new_v4().to_string(),
                    body: "nouveau138",
                    notify: &NotifySpec::None,
                    notices: &[],
                    reply_to_seq: None,
                    ack_receipt: None,
                    kind: Some(ThreadEntryKind::History),
                    supersedes_seq: None,
                    now: 300,
                    limits: &LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
        let still = serde_json::to_value(human_view(
            &store,
            actor,
            &A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: None,
                to_seq: Some(137),
                limit: Some(1),
            },
        ))
        .unwrap();
        assert_eq!(still["result"]["entries"][0]["seq"], 137);
        let fresh = serde_json::to_value(human_view(
            &store,
            actor,
            &A::HistoryRecent {
                thread_id: thread.into(),
                before_seq: None,
                to_seq: None,
                limit: Some(1),
            },
        ))
        .unwrap();
        assert_eq!(fresh["result"]["entries"][0]["seq"], 138);
        drop(profiles);
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec146_recent_projection_limits_and_cursor_fail_closed() {
        for input in [
            "",
            "-1:14600000-0000-4000-8000-000000000001",
            "01:14600000-0000-4000-8000-000000000001",
            "+1:14600000-0000-4000-8000-000000000001",
            "9007199254740992:14600000-0000-4000-8000-000000000001",
            "1:14600000-0000-4000-8000-000000000001:extra",
        ] {
            assert!(parse_recent_cursor(input).is_err(), "{input}");
        }
        assert!(parse_recent_cursor("0:14600000-0000-4000-8000-000000000001").is_ok());
    }

    #[test]
    fn spec145_projection_bound_and_member_revocation() {
        use bridget_transport::protocol::HumanThreadViewAction as A;
        let root =
            std::env::temp_dir().join(format!("h145-projection-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        let db = root.join("db");
        let store = Store::open(&db).unwrap();
        let mut profiles = crate::agent_profile::AgentProfileStore::open(&db).unwrap();
        let mut members = (0..16)
            .map(|n| format!("14500000-0000-4000-8000-{n:012}"))
            .collect::<Vec<_>>();
        members.sort();
        profiles.ensure_agent_ids(members.iter().cloned()).unwrap();
        for (n, member) in members.iter().enumerate() {
            store.connection().execute("UPDATE agent_profiles SET display_name=?2,display_name_normalized=?3 WHERE agent_id=?1",
                rusqlite::params![member,format!("{}{}","🦀".repeat(78),n),format!("member-{n}")]).unwrap();
        }
        let actor = &members[0];
        for n in 0..24 {
            let thread = format!("14500000-0000-4001-8000-{n:012}");
            assert!(matches!(
                store
                    .thread_create(CreateThread {
                        actor,
                        operation_id: &uuid::Uuid::new_v4().to_string(),
                        canonical_hash: &thread,
                        thread_id: &thread,
                        title: "Projection bornée",
                        members: &members,
                        project_warnings: &[],
                        now: 1,
                        limits: &LIMITS
                    })
                    .unwrap(),
                ThreadTxOutcome::Done(_)
            ));
        }
        let before = store.connection().total_changes();
        let refused = serde_json::to_value(human_view(
            &store,
            actor,
            &A::List {
                limit: Some(100),
                after_thread_id: None,
            },
        ))
        .unwrap();
        assert_eq!(refused["result"]["code"], "response_too_large");
        assert!(refused["subject"].is_null());
        let recent_refused = serde_json::to_value(human_view(
            &store,
            actor,
            &A::ListRecent {
                limit: Some(100),
                after: None,
            },
        ))
        .unwrap();
        assert_eq!(recent_refused["result"]["code"], "response_too_large");
        assert!(recent_refused["subject"].is_null());
        assert_eq!(store.connection().total_changes(), before);
        let first = human_view(
            &store,
            actor,
            &A::List {
                limit: Some(10),
                after_thread_id: None,
            },
        );
        let first = serde_json::to_value(first).unwrap();
        assert_eq!(first["result"]["threads"].as_array().unwrap().len(), 10);
        let second = serde_json::to_value(human_view(
            &store,
            actor,
            &A::List {
                limit: Some(10),
                after_thread_id: Some(first["result"]["next_after"].as_str().unwrap().into()),
            },
        ))
        .unwrap();
        assert!(
            first["result"]["threads"]
                .as_array()
                .unwrap()
                .iter()
                .all(|a| second["result"]["threads"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|b| a["thread_id"] != b["thread_id"]))
        );
        let thread = "14500000-0000-4001-8000-000000000000";
        store
            .connection()
            .execute(
                "DELETE FROM discussion_members WHERE thread_id=?1 AND agent_id=?2",
                rusqlite::params![thread, actor],
            )
            .unwrap();
        let before = store.connection().total_changes();
        let refused = serde_json::to_value(human_view(
            &store,
            actor,
            &A::Show {
                thread_id: thread.into(),
            },
        ))
        .unwrap();
        assert_eq!(refused["result"]["code"], "thread_unavailable");
        assert_eq!(store.connection().total_changes(), before);
        drop(store);
        drop(profiles);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec138_thread_reason_absence_is_legacy_but_null_is_invalid() {
        let legacy =
            json!({"version":1,"request":{"action":"list","limit":null,"after_thread_id":null}});
        let parsed: ThreadRequest = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), legacy);
        let mut invalid = legacy;
        invalid["cross_project_reason"] = Value::Null;
        assert!(serde_json::from_value::<ThreadRequest>(invalid).is_err());
    }

    #[test]
    fn spec102_v05_uuid_canonique_et_bornes_de_page() {
        assert!(canonical_uuid("10200000-0000-4000-8000-00000000000a").is_some());
        assert!(
            canonical_uuid("10200000-0000-4000-8000-00000000000A").is_none(),
            "majuscules refusées"
        );
        assert!(canonical_uuid("102000000000400080000000000000a").is_none());
        assert!(canonical_uuid("nom-lisible").is_none());
        assert_eq!(page_limit(None, PAGE_DEFAULT, PAGE_MAX).unwrap(), 50);
        assert_eq!(page_limit(Some(200), PAGE_DEFAULT, PAGE_MAX).unwrap(), 200);
        assert!(page_limit(Some(201), PAGE_DEFAULT, PAGE_MAX).is_err());
        assert!(page_limit(Some(0), PAGE_DEFAULT, PAGE_MAX).is_err());
        assert!(page_limit(Some(101), LIST_DEFAULT, LIST_MAX).is_err());
    }

    #[test]
    fn spec102_v14_corps_et_titre_bornes_sans_coupe() {
        assert!(validate_body("").is_err());
        assert!(validate_body("   \n\t").is_err());
        assert!(validate_body(&"x".repeat(BODY_MAX_BYTES)).is_ok());
        assert!(validate_body(&"x".repeat(BODY_MAX_BYTES + 1)).is_err());
        assert!(validate_body("é".repeat(BODY_MAX_BYTES / 2).as_str()).is_ok());
        assert_eq!(
            validate_title("  Relecture sécurité  ").unwrap(),
            "Relecture sécurité"
        );
        assert!(validate_title(" ").is_err());
        assert!(validate_title(&"t".repeat(TITLE_MAX_CHARS)).is_ok());
        assert!(validate_title(&"t".repeat(TITLE_MAX_CHARS + 1)).is_err());
        // Une entrée pleine d'échappements JSON peut dépasser le budget d'entrée
        // alors que son corps respecte 16 Kio : le contrôle porte sur le JSON.
        let quoted = "\"\\".repeat(BODY_MAX_BYTES / 2);
        assert!(validate_body(&quoted).is_ok());
        assert!(entry_json_len("m", "a", &quoted, &[], None) > BODY_MAX_BYTES);
        assert!(
            entry_json_len("m", "a", &"x".repeat(BODY_MAX_BYTES), &[], None)
                <= ENTRY_MAX_JSON_BYTES
        );
    }

    #[test]
    fn spec102_v20_empreinte_stable_et_sensible() {
        let a = digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"b"]);
        assert_eq!(
            a,
            digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"b"])
        );
        assert_ne!(
            a,
            digest(&[b"post", b"v1", b"t", b"corps ", b"targets", b"b"]),
            "espace final compte"
        );
        assert_ne!(
            a,
            digest(&[b"post", b"v1", b"t", b"corps", b"targets", b"c"])
        );
        // Les frontières de champs sont encodées : "ab"+"c" ≠ "a"+"bc".
        assert_ne!(digest(&[b"ab", b"c"]), digest(&[b"a", b"bc"]));
    }

    #[test]
    fn spec102_v23_alerte_neutre_sans_corps_ni_titre() {
        let body = notice_body("33333333-3333-4333-8333-333333333333", 20);
        assert!(body.contains("33333333-3333-4333-8333-333333333333"));
        assert!(body.contains("jusqu'à 20"));
        assert!(body.contains("Ne réponds pas par message direct"));
        assert!(!body.contains("Bridget thread"));
        assert_eq!(notice_message_id("t", "b", 3), "thread-notice:t:b:3");
        assert_ne!(
            notice_message_id("t", "b", 3),
            notice_message_id("t", "b", 4)
        );
    }

    #[test]
    fn spec102_v22_version_inconnue_refusee_avant_toute_lecture() {
        struct NoDirectory;
        impl Directory for NoDirectory {
            fn known_agent(&self, _: &str) -> bool {
                false
            }
            fn member_facts(&self, _: &str) -> MemberFacts {
                MemberFacts::default()
            }
        }
        let root =
            std::env::temp_dir().join(format!("spec102-threads-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).unwrap();
        let store = Store::open(&root.join("db.sqlite")).unwrap();
        let outcome = handle(
            &store,
            &NoDirectory,
            "10200000-0000-4000-8000-00000000000a",
            ThreadRequest {
                cross_project_reason: None,
                version: 2,
                request: ThreadAction::List {
                    limit: None,
                    after_thread_id: None,
                },
            },
            1,
        );
        assert_eq!(outcome.result["code"], "unsupported_version");
        let outcome = handle(
            &store,
            &NoDirectory,
            "10200000-0000-4000-8000-00000000000a",
            ThreadRequest {
                cross_project_reason: None,
                version: 1,
                request: ThreadAction::Show {
                    thread_id: "pas-un-uuid".into(),
                },
            },
            1,
        );
        assert_eq!(outcome.result["code"], "invalid_request");
        let _ = std::fs::remove_dir_all(root);
    }
}
