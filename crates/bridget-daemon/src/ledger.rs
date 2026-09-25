//! Projection de lecture du ledger, indépendante des interfaces CLI et MCP.

use bridget_transport::protocol::{LedgerDeliveryStatus, LedgerMessage, LedgerScope, RequestInfo};

use crate::store::{Store, StoreError};

pub const MAX_LEDGER_PROJECTION: usize = 200;

#[derive(Debug, Clone)]
pub struct LedgerProjection {
    pub messages: Vec<LedgerMessage>,
    pub requests: Vec<RequestInfo>,
}

/// Lit une projection typée du store sans appliquer de formatage utilisateur.
pub fn read_projection(
    store: &Store,
    scope: LedgerScope,
    limit: usize,
) -> Result<LedgerProjection, StoreError> {
    let limit = limit.clamp(1, MAX_LEDGER_PROJECTION);
    let wants_messages = matches!(scope, LedgerScope::Messages | LedgerScope::Both);
    let wants_requests = matches!(scope, LedgerScope::Requests | LedgerScope::Both);

    let messages = if wants_messages {
        store
            .recent_messages(limit)?
            .into_iter()
            .map(|entry| LedgerMessage {
                id: entry.id,
                ts: entry.ts,
                sender: entry.sender,
                target: entry.target,
                body: entry.body,
                delivery_status: entry
                    .delivery_phase
                    .as_deref()
                    .and_then(LedgerDeliveryStatus::from_phase),
            })
            .collect()
    } else {
        Vec::new()
    };
    let requests = if wants_requests {
        store
            .open_requests()?
            .into_iter()
            .take(limit)
            .map(|request| {
                let deferred = store.latest_deferred_reminder(&request.id)?;
                Ok(RequestInfo {
                    id: request.id,
                    sender: request.sender,
                    target: request.target,
                    state: request.state,
                    created_at: request.created_at,
                    deadline_at: request.deadline_at,
                    cancel_reason: request.cancel_reason,
                    deferred_reminder_level: deferred.map(|event| event.0),
                    deferred_reminder_at: deferred.map(|event| event.1),
                })
            })
            .collect::<Result<Vec<_>, StoreError>>()?
    } else {
        Vec::new()
    };

    Ok(LedgerProjection { messages, requests })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridget_core::BridgetMessage;

    #[test]
    fn projection_messages_est_bornee_et_preserve_le_corps() {
        let path = std::env::temp_dir().join(format!(
            "bridget-ledger-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let store = Store::open(&path).unwrap();
        let mut message = BridgetMessage::new("alice", "bob", "ligne 1\n$VAR `intact`");
        message.id = "ledger-1".to_string();
        store.record_message(&message, "alice:bob").unwrap();

        let projection = read_projection(&store, LedgerScope::Messages, 1).unwrap();
        assert_eq!(projection.messages.len(), 1);
        assert_eq!(projection.messages[0].body, "ligne 1\n$VAR `intact`");
        assert!(projection.requests.is_empty());
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn projection_requests_est_globale_tous_emetteurs() {
        let path = std::env::temp_dir().join(format!(
            "bridget-ledger-req-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let store = Store::open(&path).unwrap();
        store
            .create_request("req-alice", "alice", "bob", 60)
            .unwrap();
        store
            .create_request("req-carol", "carol", "dave", 60)
            .unwrap();

        let projection = read_projection(&store, LedgerScope::Requests, 40).unwrap();
        let ids: Vec<_> = projection
            .requests
            .iter()
            .map(|request| request.id.as_str())
            .collect();
        assert!(
            ids.contains(&"req-alice") && ids.contains(&"req-carol"),
            "{ids:?}"
        );
        assert!(
            projection
                .requests
                .iter()
                .any(|request| request.sender == "alice" && request.target == "bob")
        );
        assert!(
            projection
                .requests
                .iter()
                .any(|request| request.sender == "carol" && request.target == "dave")
        );
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// Oracle : un message `dispatching` et un message `acked` ne se rendent
    /// pas pareil au ledger (CLI et projection typée).
    #[test]
    fn projection_distingue_en_vol_et_recu() {
        use crate::idempotency::{IdempotencyKey, IdempotencyStore, OperationKind, SendDelivery};

        const NOW: i64 = 1_700_000_000;
        const HORIZON: i64 = 3600;

        let path = std::env::temp_dir().join(format!(
            "bridget-ledger-phase-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        {
            let mut idem = IdempotencyStore::open(&path).unwrap();
            for (msg_id, delivery_id, ack) in [
                ("msg-en-vol", "delivery-en-vol", false),
                ("msg-recu", "delivery-recu", true),
            ] {
                let key =
                    IdempotencyKey::new("012_scope_aaaaaaaaaaaa", OperationKind::Send, msg_id)
                        .unwrap();
                let mut message = BridgetMessage::new("peer-a", "peer-b", "corps collège");
                message.id = msg_id.to_string();
                let bytes = serde_json::to_vec(&message).unwrap();
                idem.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
                idem.begin_send_delivery(
                    &key,
                    &SendDelivery {
                        delivery_id: delivery_id.to_string(),
                        recipient_instance_id: "instance-b".to_string(),
                        delivery_generation: 1,
                        expires_at: NOW + HORIZON,
                        message_bytes: bytes,
                    },
                )
                .unwrap();
                if ack {
                    idem.acknowledge_send_delivery(delivery_id, "instance-b", 1)
                        .unwrap();
                }
            }
        }

        let store = Store::open(&path).unwrap();
        let projection = read_projection(&store, LedgerScope::Messages, 10).unwrap();
        let en_vol = projection
            .messages
            .iter()
            .find(|message| message.id == "msg-en-vol")
            .expect("message en vol visible");
        let recu = projection
            .messages
            .iter()
            .find(|message| message.id == "msg-recu")
            .expect("message reçu visible");
        assert_eq!(en_vol.delivery_status, Some(LedgerDeliveryStatus::EnVol));
        assert_eq!(recu.delivery_status, Some(LedgerDeliveryStatus::Recu));

        let rendered_vol = crate::cli::render_ledger(std::slice::from_ref(en_vol));
        let rendered_recu = crate::cli::render_ledger(std::slice::from_ref(recu));
        assert_ne!(
            rendered_vol, rendered_recu,
            "dispatching et acked ne doivent pas se rendre pareil"
        );
        assert!(rendered_vol.contains("[en vol]"), "{rendered_vol}");
        assert!(rendered_recu.contains("[reçu]"), "{rendered_recu}");

        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// Oracle : une remise `indeterminate` se rend distinctement au ledger.
    /// Meurt si `from_phase("indeterminate")` perd sa correspondance — le
    /// troisième état fonctionnerait en base sans jamais s'afficher.
    #[test]
    fn projection_rend_indetermine_distinctement() {
        use crate::idempotency::{IdempotencyKey, IdempotencyStore, OperationKind, SendDelivery};

        const NOW: i64 = 1_700_000_000;
        const HORIZON: i64 = 3600;

        let path = std::env::temp_dir().join(format!(
            "bridget-ledger-indetermine-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        {
            let mut idem = IdempotencyStore::open(&path).unwrap();
            let key = IdempotencyKey::new(
                "012_scope_aaaaaaaaaaaa",
                OperationKind::Send,
                "msg-indetermine",
            )
            .unwrap();
            let mut message = BridgetMessage::new("peer-a", "peer-b", "corps quarantaine");
            message.id = "msg-indetermine".to_string();
            let bytes = serde_json::to_vec(&message).unwrap();
            idem.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
            idem.begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-indetermine".to_string(),
                    recipient_instance_id: "instance-b".to_string(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: bytes,
                },
            )
            .unwrap();
            idem.mark_delivery_indeterminate("delivery-indetermine", "instance-b", 1)
                .unwrap();
        }

        let store = Store::open(&path).unwrap();
        let projection = read_projection(&store, LedgerScope::Messages, 10).unwrap();
        let entry = projection
            .messages
            .iter()
            .find(|message| message.id == "msg-indetermine")
            .expect("message indéterminé visible");
        assert_eq!(
            entry.delivery_status,
            Some(LedgerDeliveryStatus::Indetermine)
        );
        let rendered = crate::cli::render_ledger(std::slice::from_ref(entry));
        assert!(
            rendered.contains("[indéterminé]"),
            "la quarantaine doit s'afficher, pas se taire: {rendered}"
        );
        assert!(!rendered.contains("[en vol]"));
        assert!(!rendered.contains("[reçu]"));

        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    /// Oracle : une remise `orphaned` se rend distinctement — pas en vol, pas
    /// indéterminé. Meurt si `from_phase("orphaned")` perd sa correspondance.
    #[test]
    fn projection_rend_orphelin_distinctement() {
        use crate::idempotency::{IdempotencyKey, IdempotencyStore, OperationKind, SendDelivery};

        const NOW: i64 = 1_700_000_000;
        const HORIZON: i64 = 3600;

        let path = std::env::temp_dir().join(format!(
            "bridget-ledger-orphelin-{}-{}.sqlite",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        {
            let mut idem = IdempotencyStore::open(&path).unwrap();
            let key = IdempotencyKey::new(
                "012_scope_aaaaaaaaaaaa",
                OperationKind::Send,
                "msg-orphelin",
            )
            .unwrap();
            let mut message = BridgetMessage::new("bridget", "relec-zombie", "mandat perdu");
            message.id = "msg-orphelin".to_string();
            let bytes = serde_json::to_vec(&message).unwrap();
            idem.reserve(&key, &bytes, NOW, HORIZON, NOW, 30).unwrap();
            idem.begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-orphelin".to_string(),
                    recipient_instance_id: "instance-morte".to_string(),
                    delivery_generation: 1,
                    expires_at: NOW + HORIZON,
                    message_bytes: bytes,
                },
            )
            .unwrap();
            idem.orphan_dispatching_for_instance(
                "instance-morte",
                "destinataire purgé — présence absente ; remise orpheline",
            )
            .unwrap();
        }

        let store = Store::open(&path).unwrap();
        let projection = read_projection(&store, LedgerScope::Messages, 10).unwrap();
        let entry = projection
            .messages
            .iter()
            .find(|message| message.id == "msg-orphelin")
            .expect("message orphelin visible");
        assert_eq!(entry.delivery_status, Some(LedgerDeliveryStatus::Orphelin));
        let rendered = crate::cli::render_ledger(std::slice::from_ref(entry));
        assert!(
            rendered.contains("[orphelin]"),
            "l'orphelin doit s'afficher, pas se taire: {rendered}"
        );
        assert!(!rendered.contains("[en vol]"));
        assert!(!rendered.contains("[indéterminé]"));

        drop(store);
        std::fs::remove_file(path).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Session 104 — recherche bornée et relecture exacte
// ---------------------------------------------------------------------------

pub mod search {
    //! Validation, curseur sans pouvoir, assemblage d'une page et relecture.
    //! Aucune écriture : ni table, ni reçu, ni sollicitation. Le curseur ne
    //! porte aucun droit : SQL et participation s'appliquent à chaque page.
    use crate::artifact_types::sha256_hex;
    use crate::store::ledger_search as sql;
    use crate::store::ledger_search::{DateWindow, MessageKey, SearchError};
    use bridget_transport::protocol::{
        LedgerReadFragment, LedgerReadOutcomeV1, LedgerReadRequest, LedgerSearchHit,
        LedgerSearchOutcomeV1, LedgerSearchPage, LedgerSearchRequest, LedgerSearchSource,
    };
    use rusqlite::Connection;
    use serde::{Deserialize, Serialize};
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub const MAX_QUERY_BYTES: usize = 256;
    pub const MAX_TERMS: usize = 8;
    pub const DEFAULT_LIMIT: u16 = 20;
    pub const MAX_LIMIT: u16 = 50;
    /// Objet `outcome` compact, curseur compris, avant enveloppe MCP.
    pub const RESPONSE_BUDGET_BYTES: usize = 61_440;
    pub const CURSOR_MAX_BYTES: usize = 16_384;
    pub const READ_FRAGMENT_BYTES: usize = 16_384;
    /// Recherches et relectures simultanées ; au-delà : `busy` immédiat.
    pub const MAX_CONCURRENT: usize = 2;

    // ------------------------------------------------------------- permis

    /// Compteur partagé des lectures 104 en cours (pas de file d'attente).
    #[derive(Clone, Default)]
    pub struct ReadPermits(Arc<AtomicUsize>);

    pub struct ReadPermit(Arc<AtomicUsize>);

    impl ReadPermits {
        pub fn try_acquire(&self) -> Option<ReadPermit> {
            let mut current = self.0.load(Ordering::Acquire);
            loop {
                if current >= MAX_CONCURRENT {
                    return None;
                }
                match self.0.compare_exchange_weak(
                    current,
                    current + 1,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => return Some(ReadPermit(Arc::clone(&self.0))),
                    Err(observed) => current = observed,
                }
            }
        }

        pub fn in_use(&self) -> usize {
            self.0.load(Ordering::Acquire)
        }
    }

    impl Drop for ReadPermit {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::AcqRel);
        }
    }

    // ------------------------------------------------------------- erreurs

    pub struct Refusal {
        pub code: &'static str,
        pub reason: String,
    }

    fn refuse(code: &'static str, reason: impl Into<String>) -> Refusal {
        Refusal {
            code,
            reason: reason.into(),
        }
    }

    impl From<SearchError> for Refusal {
        fn from(error: SearchError) -> Self {
            match error {
                SearchError::MetadataTooLarge => refuse(
                    "source_metadata_too_large",
                    "un identifiant hérité dépasse 256 octets dans la fenêtre demandée : restreindre les dates",
                ),
                SearchError::Storage => storage_unavailable(),
            }
        }
    }

    impl From<rusqlite::Error> for Refusal {
        fn from(error: rusqlite::Error) -> Self {
            // Diagnostic local sans corps, terme ni empreinte de requête.
            log::warn!("recherche 104 : SQLite indisponible : {error}");
            storage_unavailable()
        }
    }

    fn storage_unavailable() -> Refusal {
        refuse(
            "storage_unavailable",
            "lecture du ledger impossible pour l'instant : réessayer plus tard",
        )
    }

    pub fn search_error(code: &str, reason: &str) -> LedgerSearchOutcomeV1 {
        LedgerSearchOutcomeV1::Error {
            code: code.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn read_error(code: &str, reason: &str) -> LedgerReadOutcomeV1 {
        LedgerReadOutcomeV1::Error {
            code: code.to_string(),
            reason: reason.to_string(),
        }
    }

    // --------------------------------------------------------- validation

    struct Spec {
        source: LedgerSearchSource,
        terms: Vec<String>,
        author: Option<String>,
        peer: Option<String>,
        window: DateWindow,
        limit: usize,
        thread_id: Option<String>,
    }

    fn uuid_param(name: &str, value: &Option<String>) -> Result<Option<String>, Refusal> {
        match value {
            None => Ok(None),
            Some(value) => crate::threads::canonical_uuid(value)
                .map(Some)
                .ok_or_else(|| {
                    refuse(
                        "invalid_params",
                        format!("{name} doit être un UUID canonique"),
                    )
                }),
        }
    }

    fn validate(request: &LedgerSearchRequest) -> Result<Spec, Refusal> {
        let raw_query = request.query.as_str();
        if raw_query.trim().is_empty() {
            return Err(refuse("invalid_params", "query ne doit pas être vide"));
        }
        if raw_query.len() > MAX_QUERY_BYTES {
            return Err(refuse(
                "invalid_params",
                format!("query dépasse {MAX_QUERY_BYTES} octets"),
            ));
        }
        let terms: Vec<String> = raw_query
            .split_whitespace()
            .map(crate::store::fold_for_search)
            .collect();
        if terms.is_empty() || terms.len() > MAX_TERMS {
            return Err(refuse(
                "invalid_params",
                format!("query doit contenir de 1 à {MAX_TERMS} termes"),
            ));
        }
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(refuse(
                "invalid_params",
                format!("limit doit être compris entre 1 et {MAX_LIMIT}"),
            ));
        }
        let since = request.since.unwrap_or(0);
        let until = request.until.unwrap_or(i64::MAX);
        if since < 0 || until < 0 {
            return Err(refuse("invalid_params", "since et until doivent être ≥ 0"));
        }
        if since > until {
            return Err(refuse("invalid_params", "since doit être ≤ until"));
        }
        let author = uuid_param("author", &request.author)?;
        let peer = uuid_param("peer", &request.peer)?;
        let thread_id = uuid_param("thread_id", &request.thread_id)?;
        match request.source {
            LedgerSearchSource::Messages if thread_id.is_some() => {
                return Err(refuse("invalid_params", "thread_id exige source=thread"));
            }
            LedgerSearchSource::Thread if thread_id.is_none() => {
                return Err(refuse("invalid_params", "source=thread exige thread_id"));
            }
            LedgerSearchSource::Thread if peer.is_some() => {
                return Err(refuse(
                    "invalid_params",
                    "peer est interdit avec source=thread",
                ));
            }
            _ => {}
        }
        Ok(Spec {
            source: request.source,
            terms,
            author,
            peer,
            window: DateWindow { since, until },
            limit: usize::from(limit),
            thread_id,
        })
    }

    // ------------------------------------------------------------ curseur

    #[derive(Serialize)]
    struct Fingerprint<'a> {
        source: &'a str,
        query: &'a str,
        author: Option<&'a str>,
        peer: Option<&'a str>,
        since: Option<i64>,
        until: Option<i64>,
        thread_id: Option<&'a str>,
    }

    fn fingerprint(request: &LedgerSearchRequest) -> String {
        let bytes = serde_json::to_vec(&Fingerprint {
            source: request.source.name(),
            query: &request.query,
            author: request.author.as_deref(),
            peer: request.peer.as_deref(),
            since: request.since,
            until: request.until,
            thread_id: request.thread_id.as_deref(),
        })
        .expect("empreinte sérialisable");
        sha256_hex(&bytes)
    }

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    #[serde(untagged)]
    enum CursorKey {
        Message(MessageKey),
        Seq(u64),
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CursorV1 {
        v: u16,
        actor: String,
        fingerprint: String,
        source: LedgerSearchSource,
        upper: CursorKey,
        before: CursorKey,
    }

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn hex_decode(text: &str) -> Option<Vec<u8>> {
        if !text.len().is_multiple_of(2)
            || !text
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return None;
        }
        (0..text.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&text[index..index + 2], 16).ok())
            .collect()
    }

    fn encode_cursor(cursor: &CursorV1) -> Result<String, Refusal> {
        let encoded = hex_encode(&serde_json::to_vec(cursor).expect("curseur sérialisable"));
        if encoded.len() > CURSOR_MAX_BYTES {
            return Err(refuse(
                "source_metadata_too_large",
                "le curseur de reprise dépasserait la taille admise",
            ));
        }
        Ok(encoded)
    }

    fn invalid_cursor() -> Refusal {
        refuse(
            "invalid_cursor",
            "curseur invalide pour cette requête : répéter exactement query, source et filtres, ou recommencer sans cursor",
        )
    }

    fn key_admissible(key: &MessageKey) -> bool {
        !key.id.is_empty()
            && !key.target.is_empty()
            && key.id.len() <= sql::MAX_METADATA_BYTES
            && key.target.len() <= sql::MAX_METADATA_BYTES
    }

    fn decode_cursor(
        text: &str,
        actor: &str,
        request: &LedgerSearchRequest,
    ) -> Result<CursorV1, Refusal> {
        if text.is_empty() || text.len() > CURSOR_MAX_BYTES {
            return Err(invalid_cursor());
        }
        let bytes = hex_decode(text).ok_or_else(invalid_cursor)?;
        let cursor: CursorV1 = serde_json::from_slice(&bytes).map_err(|_| invalid_cursor())?;
        if cursor.v != 1
            || cursor.actor != actor
            || cursor.source != request.source
            || cursor.fingerprint != fingerprint(request)
        {
            return Err(invalid_cursor());
        }
        match (&cursor.upper, &cursor.before) {
            (CursorKey::Message(upper), CursorKey::Message(before))
                if request.source == LedgerSearchSource::Messages =>
            {
                if !key_admissible(upper)
                    || !key_admissible(before)
                    || before.page_cmp(upper) == std::cmp::Ordering::Less
                {
                    return Err(invalid_cursor());
                }
            }
            (CursorKey::Seq(upper), CursorKey::Seq(before))
                if request.source == LedgerSearchSource::Thread =>
            {
                if *upper == 0 || *before == 0 || before > upper {
                    return Err(invalid_cursor());
                }
            }
            _ => return Err(invalid_cursor()),
        }
        Ok(cursor)
    }

    // --------------------------------------------------------- assemblage

    const NOTICE_RETENTION: &str = "retention: seuls les échanges encore conservés par ce daemon sont parcourus ; une purge n'est pas signalée";
    const NOTICE_VISIBILITY: &str = "visibility: seuls vos propres échanges (émis ou reçus) ou les fils dont vous êtes membre sont visibles";
    const NOTICE_UNICODE: &str = "unicode: repli de la casse et des accents précomposés (é→e) seulement ; formes décomposées non assimilées";
    const NOTICE_LIVE: &str = "snapshot: corpus vivant borné par la première page ; recommencer sans cursor pour une vue fraîche";
    const NOTICE_THREAD: &str = "context: relire autour d'une entrée avec bridget_thread action=history from_seq/to_seq ; cette recherche ne confirme aucune lecture";

    struct Candidate<K> {
        key: K,
        author: String,
        peer: Option<String>,
        ts: i64,
        body_bytes: u64,
    }

    /// Prépare la sélection bornée : préfixe des candidats à consommer et
    /// clés dont le corps doit être chargé (budget d'octets, une seule ligne
    /// entière peut dépasser). Les lignes rejetées par filtre ou trop
    /// grandes restent consommables sans corps.
    fn select_prefix<K: Clone>(spec: &Spec, candidates: &[Candidate<K>]) -> (usize, Vec<K>, bool) {
        let mut selected_bytes = 0u64;
        let mut to_load = Vec::new();
        let mut byte_stop = false;
        let mut prefix = 0usize;
        for candidate in candidates {
            if admissible(spec, candidate) && candidate.body_bytes <= sql::MAX_BODY_BYTES {
                if selected_bytes >= sql::BYTE_BUDGET {
                    byte_stop = true;
                    break;
                }
                selected_bytes += candidate.body_bytes;
                to_load.push(candidate.key.clone());
            }
            prefix += 1;
        }
        (prefix, to_load, byte_stop)
    }

    fn admissible<K>(spec: &Spec, candidate: &Candidate<K>) -> bool {
        spec.author
            .as_deref()
            .is_none_or(|author| candidate.author == author)
            && spec
                .peer
                .as_deref()
                .is_none_or(|peer| candidate.peer.as_deref() == Some(peer))
    }

    struct PageBuilder<'a> {
        spec: &'a Spec,
        hits: Vec<LedgerSearchHit>,
        hits_bytes: usize,
        scanned_count: u32,
        scanned_bytes: u64,
        skipped_oversized: u32,
        consumed: usize,
        stop_reason: &'static str,
    }

    enum Step {
        Consumed,
        StopBefore(&'static str),
    }

    impl<'a> PageBuilder<'a> {
        fn new(spec: &'a Spec) -> Self {
            Self {
                spec,
                hits: Vec::new(),
                hits_bytes: 0,
                scanned_count: 0,
                scanned_bytes: 0,
                skipped_oversized: 0,
                consumed: 0,
                stop_reason: "exhausted",
            }
        }

        /// Traite un candidat du préfixe ; `body` absent = rejeté par filtre,
        /// trop grand, ou disparu entre la sélection et le chargement.
        fn step<K>(
            &mut self,
            candidate: &Candidate<K>,
            body: Option<&str>,
            make_hit: impl FnOnce(&str, u64, String, u64) -> LedgerSearchHit,
            skeleton_bytes: impl FnOnce() -> usize,
        ) -> Step {
            if self.hits.len() >= self.spec.limit {
                return Step::StopBefore("result_limit");
            }
            if candidate.body_bytes > sql::MAX_BODY_BYTES {
                self.skipped_oversized += 1;
            } else if let Some(body) = body {
                self.scanned_bytes += body.len() as u64;
                if let Some(offset) = sql::locate_terms(body, &self.spec.terms) {
                    let excerpt = sql::excerpt(body, offset, sql::EXCERPT_BYTES).to_string();
                    let hit = make_hit(
                        excerpt.as_str(),
                        offset as u64,
                        sha256_hex(body.as_bytes()),
                        body.len() as u64,
                    );
                    let hit_bytes = serde_json::to_vec(&hit).expect("hit sérialisable").len() + 1;
                    // Le premier résultat d'une page passe toujours : progrès garanti.
                    if !self.hits.is_empty()
                        && skeleton_bytes() + self.hits_bytes + hit_bytes > RESPONSE_BUDGET_BYTES
                    {
                        return Step::StopBefore("response_budget");
                    }
                    self.hits_bytes += hit_bytes;
                    self.hits.push(hit);
                }
            }
            self.scanned_count += 1;
            self.consumed += 1;
            Step::Consumed
        }

        fn finish(
            self,
            has_more: bool,
            next_cursor: Option<String>,
            consistency: &str,
            notices: Vec<&str>,
        ) -> LedgerSearchPage {
            let mut notices: Vec<String> = notices.into_iter().map(str::to_string).collect();
            if self.skipped_oversized > 0 {
                notices.push(format!(
                    "oversized_source: {} échange(s) de plus de 16 Mio ignoré(s) sans chargement",
                    self.skipped_oversized
                ));
            }
            LedgerSearchPage {
                source: self.spec.source,
                hits: self.hits,
                has_more,
                next_cursor,
                scanned_count: self.scanned_count,
                scanned_bytes: self.scanned_bytes,
                skipped_oversized: self.skipped_oversized,
                stop_reason: self.stop_reason.to_string(),
                consistency: consistency.to_string(),
                notices,
            }
        }
    }

    fn skeleton_size(spec: &Spec, cursor: &CursorV1, notices: &[&str]) -> usize {
        let page = LedgerSearchPage {
            source: spec.source,
            hits: Vec::new(),
            has_more: true,
            next_cursor: encode_cursor(cursor).ok(),
            scanned_count: u32::MAX,
            scanned_bytes: u64::MAX,
            skipped_oversized: u32::MAX,
            stop_reason: "response_budget".into(),
            consistency: "immutable_upper_bound".into(),
            notices: notices
                .iter()
                .map(|n| n.to_string())
                .chain(std::iter::once(
                    "oversized_source: 4294967295 échange(s) de plus de 16 Mio ignoré(s) sans chargement".to_string(),
                ))
                .collect(),
        };
        serde_json::to_vec(&LedgerSearchOutcomeV1::Ok(page))
            .expect("page sérialisable")
            .len()
    }

    fn search_messages(
        conn: &Connection,
        actor: &str,
        request: &LedgerSearchRequest,
        spec: &Spec,
    ) -> Result<LedgerSearchPage, Refusal> {
        let notices = [
            NOTICE_RETENTION,
            NOTICE_VISIBILITY,
            NOTICE_UNICODE,
            NOTICE_LIVE,
        ];
        let cursor = match request.cursor.as_deref() {
            Some(text) => Some(decode_cursor(text, actor, request)?),
            None => None,
        };
        // Transaction de lecture courte : clés puis corps, fermée avant tout
        // travail CPU (repli, empreintes, sérialisation).
        let tx = conn.unchecked_transaction()?;
        let (upper, before) = match cursor {
            Some(CursorV1 {
                upper: CursorKey::Message(upper),
                before: CursorKey::Message(before),
                ..
            }) => (upper, Some(before)),
            Some(_) => return Err(invalid_cursor()),
            None => match sql::message_upper_bound(&tx, actor, spec.window)? {
                Some(upper) => (upper, None),
                None => {
                    drop(tx);
                    let builder = PageBuilder::new(spec);
                    return Ok(builder.finish(false, None, "live_bounded", notices.to_vec()));
                }
            },
        };
        let rows = sql::message_candidates(&tx, actor, spec.window, &upper, before.as_ref())?;
        let witness = rows.len() > sql::CANDIDATES_PER_PAGE;
        let rows = &rows[..rows.len().min(sql::CANDIDATES_PER_PAGE)];
        let candidates: Vec<Candidate<MessageKey>> = rows
            .iter()
            .map(|row| {
                let peer = if row.sender == actor {
                    row.key.target.clone()
                } else {
                    row.sender.clone()
                };
                Candidate {
                    key: row.key.clone(),
                    author: row.sender.clone(),
                    peer: Some(peer),
                    ts: row.key.ts,
                    body_bytes: row.body_bytes,
                }
            })
            .collect();
        let (prefix, to_load, byte_stop) = select_prefix(spec, &candidates);
        let keys: Vec<&MessageKey> = to_load.iter().collect();
        let bodies = sql::message_bodies(&tx, actor, &keys)?;
        drop(tx);

        let mut builder = PageBuilder::new(spec);
        let mut last_key: Option<MessageKey> = before.clone();
        for candidate in &candidates[..prefix] {
            let loaded = bodies.get(&(candidate.key.id.clone(), candidate.key.target.clone()));
            let body = if admissible(spec, candidate) {
                loaded.map(|row| row.body.as_str())
            } else {
                None
            };
            let sender = loaded
                .map(|row| row.sender.clone())
                .unwrap_or_else(|| candidate.author.clone());
            let key = candidate.key.clone();
            let skeleton = || {
                skeleton_size(
                    spec,
                    &CursorV1 {
                        v: 1,
                        actor: actor.to_string(),
                        fingerprint: fingerprint(request),
                        source: spec.source,
                        upper: CursorKey::Message(upper.clone()),
                        before: CursorKey::Message(key.clone()),
                    },
                    &notices,
                )
            };
            let step = builder.step(
                candidate,
                body,
                |excerpt, offset, digest, bytes| LedgerSearchHit::Message {
                    id: candidate.key.id.clone(),
                    target: candidate.key.target.clone(),
                    sender,
                    ts: candidate.ts,
                    excerpt: excerpt.to_string(),
                    match_offset: offset,
                    body_digest: digest,
                    body_bytes: bytes,
                },
                skeleton,
            );
            match step {
                Step::Consumed => last_key = Some(candidate.key.clone()),
                Step::StopBefore(reason) => {
                    builder.stop_reason = reason;
                    break;
                }
            }
        }
        let all_prefix_consumed = builder.consumed == prefix;
        if all_prefix_consumed {
            if byte_stop {
                builder.stop_reason = "byte_budget";
            } else if witness {
                builder.stop_reason = "scan_budget";
            }
        }
        let has_more = builder.stop_reason != "exhausted";
        let next_cursor = if has_more {
            let before = last_key.unwrap_or_else(|| upper.clone());
            Some(encode_cursor(&CursorV1 {
                v: 1,
                actor: actor.to_string(),
                fingerprint: fingerprint(request),
                source: spec.source,
                upper: CursorKey::Message(upper),
                before: CursorKey::Message(before),
            })?)
        } else {
            None
        };
        Ok(builder.finish(has_more, next_cursor, "live_bounded", notices.to_vec()))
    }

    fn search_thread(
        conn: &Connection,
        actor: &str,
        request: &LedgerSearchRequest,
        spec: &Spec,
    ) -> Result<LedgerSearchPage, Refusal> {
        let notices = [
            NOTICE_RETENTION,
            NOTICE_VISIBILITY,
            NOTICE_UNICODE,
            NOTICE_THREAD,
        ];
        let thread_id = spec.thread_id.as_deref().expect("thread_id validé");
        let cursor = match request.cursor.as_deref() {
            Some(text) => Some(decode_cursor(text, actor, request)?),
            None => None,
        };
        let tx = conn.unchecked_transaction()?;
        if !crate::store::threads::schema_ready(&tx).map_err(|error| {
            log::warn!("recherche 104 : schéma des fils illisible : {error}");
            storage_unavailable()
        })? {
            return Err(refuse(
                "capability_unavailable",
                "les fils partagés ne sont pas disponibles sur ce daemon",
            ));
        }
        let thread = match crate::store::threads::load_thread_for_member(&tx, thread_id, actor)
            .map_err(|error| {
                log::warn!("recherche 104 : appartenance illisible : {error}");
                storage_unavailable()
            })? {
            Ok((thread, _member)) => thread,
            Err(_) => {
                return Err(refuse(
                    "not_found_or_forbidden",
                    "fil indisponible pour cette identité",
                ));
            }
        };
        let (upper, before) = match cursor {
            Some(CursorV1 {
                upper: CursorKey::Seq(upper),
                before: CursorKey::Seq(before),
                ..
            }) => (upper.min(thread.last_seq), Some(before)),
            Some(_) => return Err(invalid_cursor()),
            None => (thread.last_seq, None),
        };
        if upper == 0 {
            drop(tx);
            let builder = PageBuilder::new(spec);
            return Ok(builder.finish(false, None, "immutable_upper_bound", notices.to_vec()));
        }
        let rows = sql::thread_candidates(&tx, thread_id, spec.window, upper, before)?;
        let witness = rows.len() > sql::CANDIDATES_PER_PAGE;
        let rows = &rows[..rows.len().min(sql::CANDIDATES_PER_PAGE)];
        let candidates: Vec<Candidate<u64>> = rows
            .iter()
            .map(|row| Candidate {
                key: row.seq,
                author: row.author_id.clone(),
                peer: None,
                ts: row.created_at,
                body_bytes: row.body_bytes,
            })
            .collect();
        let (prefix, to_load, byte_stop) = select_prefix(spec, &candidates);
        let bodies = sql::thread_bodies(&tx, thread_id, &to_load)?;
        drop(tx);

        let mut builder = PageBuilder::new(spec);
        let mut last_seq: Option<u64> = before;
        for (candidate, row) in candidates[..prefix].iter().zip(rows) {
            let body = if admissible(spec, candidate) {
                bodies.get(&candidate.key).map(String::as_str)
            } else {
                None
            };
            let seq = candidate.key;
            let skeleton = || {
                skeleton_size(
                    spec,
                    &CursorV1 {
                        v: 1,
                        actor: actor.to_string(),
                        fingerprint: fingerprint(request),
                        source: spec.source,
                        upper: CursorKey::Seq(upper),
                        before: CursorKey::Seq(seq),
                    },
                    &notices,
                )
            };
            let step = builder.step(
                candidate,
                body,
                |excerpt, offset, digest, bytes| LedgerSearchHit::ThreadEntry {
                    thread_id: thread_id.to_string(),
                    seq,
                    message_id: row.message_id.clone(),
                    author_id: row.author_id.clone(),
                    ts: row.created_at,
                    excerpt: excerpt.to_string(),
                    match_offset: offset,
                    body_digest: digest,
                    body_bytes: bytes,
                },
                skeleton,
            );
            match step {
                Step::Consumed => last_seq = Some(seq),
                Step::StopBefore(reason) => {
                    builder.stop_reason = reason;
                    break;
                }
            }
        }
        if builder.consumed == prefix {
            if byte_stop {
                builder.stop_reason = "byte_budget";
            } else if witness {
                builder.stop_reason = "scan_budget";
            }
        }
        let has_more = builder.stop_reason != "exhausted";
        let next_cursor = if has_more {
            Some(encode_cursor(&CursorV1 {
                v: 1,
                actor: actor.to_string(),
                fingerprint: fingerprint(request),
                source: spec.source,
                upper: CursorKey::Seq(upper),
                before: CursorKey::Seq(last_seq.unwrap_or(upper)),
            })?)
        } else {
            None
        };
        Ok(builder.finish(
            has_more,
            next_cursor,
            "immutable_upper_bound",
            notices.to_vec(),
        ))
    }

    /// Recherche sur une connexion déjà ouverte (lecture seule côté daemon).
    pub fn search_on(
        conn: &Connection,
        actor: &str,
        request: &LedgerSearchRequest,
    ) -> LedgerSearchOutcomeV1 {
        let spec = match validate(request) {
            Ok(spec) => spec,
            Err(refusal) => return search_error(refusal.code, &refusal.reason),
        };
        let page = match spec.source {
            LedgerSearchSource::Messages => search_messages(conn, actor, request, &spec),
            LedgerSearchSource::Thread => search_thread(conn, actor, request, &spec),
        };
        match page {
            Ok(page) => LedgerSearchOutcomeV1::Ok(page),
            Err(refusal) => search_error(refusal.code, &refusal.reason),
        }
    }

    /// Recherche depuis le chemin de base configuré : validation AVANT toute
    /// ouverture, connexion lecture seule, aucune base de secours.
    pub fn search(
        db_path: &Path,
        actor: &str,
        request: &LedgerSearchRequest,
    ) -> LedgerSearchOutcomeV1 {
        if let Err(refusal) = validate(request) {
            return search_error(refusal.code, &refusal.reason);
        }
        match crate::store::Store::open_read_only(db_path) {
            Ok(conn) => search_on(&conn, actor, request),
            Err(error) => {
                log::warn!("recherche 104 : ouverture lecture seule impossible : {error}");
                let refusal = storage_unavailable();
                search_error(refusal.code, &refusal.reason)
            }
        }
    }

    // ---------------------------------------------------------- relecture

    fn validate_read(request: &LedgerReadRequest) -> Result<(), Refusal> {
        for (name, value) in [("id", &request.id), ("target", &request.target)] {
            if value.is_empty() || value.len() > sql::MAX_METADATA_BYTES {
                return Err(refuse(
                    "invalid_params",
                    format!(
                        "{name} doit contenir de 1 à {} octets",
                        sql::MAX_METADATA_BYTES
                    ),
                ));
            }
        }
        match &request.digest {
            Some(digest)
                if digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) =>
            {
                Err(refuse(
                    "invalid_params",
                    "digest doit être un SHA-256 hexadécimal minuscule",
                ))
            }
            None if request.offset > 0 => Err(refuse(
                "invalid_params",
                "digest est obligatoire dès que offset > 0",
            )),
            _ => Ok(()),
        }
    }

    fn read_fragment(
        conn: &Connection,
        actor: &str,
        request: &LedgerReadRequest,
    ) -> Result<LedgerReadFragment, Refusal> {
        let not_found = || {
            refuse(
                "not_found_or_forbidden",
                "message indisponible pour cette identité",
            )
        };
        let tx = conn.unchecked_transaction()?;
        let Some((sender, ts, body_bytes)) =
            sql::message_head(&tx, actor, &request.id, &request.target)?
        else {
            return Err(not_found());
        };
        if body_bytes > sql::MAX_BODY_BYTES {
            return Err(refuse(
                "source_too_large",
                "le corps dépasse 16 Mio : relecture refusée sans troncature",
            ));
        }
        let Some(body) = sql::message_body(&tx, actor, &request.id, &request.target)? else {
            return Err(not_found());
        };
        drop(tx);
        let digest = sha256_hex(body.as_bytes());
        if request
            .digest
            .as_deref()
            .is_some_and(|expected| expected != digest)
        {
            return Err(refuse(
                "content_changed",
                "le corps a changé depuis l'empreinte fournie : recommencer à offset 0 pour lire la version courante",
            ));
        }
        let offset = usize::try_from(request.offset)
            .map_err(|_| refuse("invalid_params", "offset hors du corps"))?;
        if offset > body.len() || !body.is_char_boundary(offset) {
            return Err(refuse(
                "invalid_params",
                "offset hors du corps ou hors d'une frontière UTF-8",
            ));
        }
        let end = sql::floor_char_boundary(
            &body,
            offset.saturating_add(READ_FRAGMENT_BYTES).min(body.len()),
        );
        Ok(LedgerReadFragment {
            id: request.id.clone(),
            target: request.target.clone(),
            sender,
            ts,
            body_bytes: body.len() as u64,
            digest,
            fragment: body[offset..end].to_string(),
            next_offset: (end < body.len()).then_some(end as u64),
        })
    }

    pub fn read_on(
        conn: &Connection,
        actor: &str,
        request: &LedgerReadRequest,
    ) -> LedgerReadOutcomeV1 {
        if let Err(refusal) = validate_read(request) {
            return read_error(refusal.code, &refusal.reason);
        }
        match read_fragment(conn, actor, request) {
            Ok(fragment) => LedgerReadOutcomeV1::Ok(fragment),
            Err(refusal) => read_error(refusal.code, &refusal.reason),
        }
    }

    pub fn read(db_path: &Path, actor: &str, request: &LedgerReadRequest) -> LedgerReadOutcomeV1 {
        if let Err(refusal) = validate_read(request) {
            return read_error(refusal.code, &refusal.reason);
        }
        match crate::store::Store::open_read_only(db_path) {
            Ok(conn) => read_on(&conn, actor, request),
            Err(error) => {
                log::warn!("relecture 104 : ouverture lecture seule impossible : {error}");
                let refusal = storage_unavailable();
                read_error(refusal.code, &refusal.reason)
            }
        }
    }
}

#[cfg(test)]
mod spec104_permit_tests {
    use super::search::{MAX_CONCURRENT, ReadPermits};

    #[test]
    fn spec104_deux_permis_puis_busy_et_liberation_raii() {
        let permits = ReadPermits::default();
        let first = permits.try_acquire().expect("premier permis");
        let second = permits.try_acquire().expect("second permis");
        assert_eq!(permits.in_use(), MAX_CONCURRENT);
        assert!(
            permits.try_acquire().is_none(),
            "troisième : busy, sans file d'attente"
        );
        drop(first);
        assert_eq!(permits.in_use(), 1);
        let third = permits.try_acquire().expect("permis rendu réutilisable");
        drop(second);
        drop(third);
        assert_eq!(permits.in_use(), 0);
    }
}
