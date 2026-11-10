//! Fonctions des branches volumineuses de `handle_wrapper_message_with_peer` (programme 260, lot 5).
//! Chaque fonction reprend le corps de sa branche sans changement.
use super::*;

pub(super) fn on_gap(subscription_id: String, from_seq: u64, to_seq: u64, reason: Option<String>, conn_id: &str, state: &Arc<Mutex<DaemonState>>) -> Option<DaemonToWrapper> {
            let view = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let accepted = st
                    .attach_subscriptions
                    .get_mut(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .map(|subscription| {
                        if reason.as_deref() == Some("live_feed_overrun") {
                            subscription.caught_up = false;
                        }
                    })
                    .is_some();
                accepted
                    .then(|| st.attach_views.get(&subscription_id).cloned())
                    .flatten()
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::Gap {
                    subscription_id,
                    from_seq,
                    to_seq,
                    reason,
                });
            }
            None
        }
