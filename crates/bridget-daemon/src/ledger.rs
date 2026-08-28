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
