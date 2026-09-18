//! Session 102 — règles métier des fils inter-agents.
//!
//! Le daemon est l'autorité : identité attestée en amont, appartenance et
//! rejeu décidés ici et dans `store::threads` sous transaction. Trois faits
//! distincts et jamais confondus : `posted` (contribution durable),
//! `dispatched` (alerte injectée chez un membre visé) et `acknowledged`
//! (page confirmée par un membre). Aucune lecture du corps pour y chercher des
//! mentions : seules les cibles structurées comptent.

use crate::store::threads::{
    AckOutcome, CloseThread, CreateThread, HistoryOutcome, NotifySpec, PostEntry, ReadOutcome,
    ReadRequest, ThreadLimits, ThreadRefusal, ThreadTxOutcome, WakeRow,
};
use crate::store::{Store, StoreError};
use bridget_transport::protocol::{
    THREAD_CONTRACT_VERSION, ThreadAction, ThreadNotify, ThreadRequest, ThreadResult,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

// Bornes V1 (contracts/thread-api.md). Constantes partagées, pas d'options.
pub(crate) const MIN_MEMBERS: usize = 2;
pub(crate) const MAX_MEMBERS: usize = 16;
pub(crate) const TITLE_MAX_CHARS: usize = 160;
pub(crate) const BODY_MAX_BYTES: usize = 16 * 1024;
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

/// UUID canonique en minuscules ; toute autre forme est refusée.
pub(crate) fn canonical_uuid(value: &str) -> Option<String> {
    let parsed = uuid::Uuid::parse_str(value).ok()?;
    let canonical = parsed.hyphenated().to_string();
    (canonical == value).then_some(canonical)
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
            "Seul le créateur peut clore ce fil.",
            false,
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

fn require_uuid(value: &str, field: &str) -> Result<String, Value> {
    canonical_uuid(value).ok_or_else(|| {
        error(
            "invalid_request",
            format!("{field} doit être un UUID canonique en minuscules."),
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
pub(crate) fn handle(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    request: ThreadRequest,
    now: i64,
) -> ThreadResult {
    if request.version != THREAD_CONTRACT_VERSION {
        return result(error(
            "unsupported_version",
            format!(
                "Version {} inconnue ; version attendue {THREAD_CONTRACT_VERSION}.",
                request.version
            ),
            false,
        ));
    }
    let outcome = match request.request {
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
            now,
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
        } => post(
            store,
            actor,
            &thread_id,
            &body,
            &notify,
            &operation_id,
            reply_to_seq,
            ack_receipt.as_deref(),
            now,
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
        } => close(store, actor, &thread_id, &operation_id, now),
    };
    result(outcome.unwrap_or_else(|refused| refused))
}

fn create(
    store: &Store,
    directory: &dyn Directory,
    actor: &str,
    title: &str,
    members: &[String],
    operation_id: &str,
    now: i64,
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
    let mut parts: Vec<&[u8]> = vec![b"create", b"v1", title.as_bytes()];
    for member in &set {
        parts.push(member.as_bytes());
    }
    let canonical_hash = digest(&parts);
    let thread_id = uuid::Uuid::new_v4().hyphenated().to_string();
    let outcome = store
        .thread_create(CreateThread {
            actor,
            operation_id: &operation_id,
            canonical_hash: &canonical_hash,
            thread_id: &thread_id,
            title: &title,
            members: &set,
            now,
            limits: &LIMITS,
        })
        .map_err(storage_error)?;
    tx_result(outcome)
}

fn tx_result(outcome: ThreadTxOutcome) -> Result<Value, Value> {
    match outcome {
        ThreadTxOutcome::Done(value) | ThreadTxOutcome::Replayed(value) => Ok(value),
        ThreadTxOutcome::Refused(refusal) => Err(refusal_result(refusal)),
    }
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
    actor: &str,
    thread_id: &str,
    body: &str,
    notify: &ThreadNotify,
    operation_id: &str,
    reply_to_seq: Option<u64>,
    ack_receipt: Option<&str>,
    now: i64,
) -> Result<Value, Value> {
    let thread_id = require_uuid(thread_id, "thread_id")?;
    let operation_id = require_uuid(operation_id, "operation_id")?;
    validate_body(body)?;
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
    let entry_len = entry_json_len(&message_id, actor, body, &canonical_targets, reply_to_seq);
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
    let canonical_hash = digest(&parts);
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
            reply_to_seq,
            ack_receipt: ack_receipt.as_deref(),
            now,
            limits: &LIMITS,
        })
        .map_err(storage_error)?;
    tx_result(outcome)
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
    if canonical_uuid(receipt).is_none() {
        return Err(refusal_result(ThreadRefusal::ReceiptInvalid));
    }
    match store
        .thread_ack(actor, &thread_id, receipt)
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
    tx_result(outcome)
}

/// Corps neutre de l'alerte : aucun texte de contribution, aucun titre.
pub(crate) fn notice_body(thread_id: &str, through_seq: u64) -> String {
    format!(
        "Sollicitation dans le fil {thread_id}, nouveautés jusqu'à {through_seq}. \
         Lis les nouveautés avec bridget_thread/read. Confirme la plage reçue, puis publie \
         dans le fil si utile. Ne réponds pas par message direct à cette alerte."
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
