//! Projection de lecture du ledger, indépendante des interfaces CLI et MCP.

use bridget_transport::protocol::{LedgerMessage, LedgerScope, RequestInfo};

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
}
