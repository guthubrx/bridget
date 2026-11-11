//! Garde156 pure : faits attestés par les appelants, aucune lecture ni mutation.
use bridget_core::MessageOrigin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuidanceAction {
    Steer,
    Interrupt,
    MidTurnDelivery,
    QueueList,
    QueueCancel,
    QueueReorder,
    QueuePromote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuidancePath {
    SendIdempotent,
    Send,
    Control,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GuidanceActor {
    AttachedHuman,
    BareClient,
    Agent { id: String },
    Loop { id: String },
    InternalPause,
}

/// Les preuves sont des faits locaux, jamais des valeurs de from/origin clientes.
#[derive(Debug, Clone)]
pub(crate) struct GuidanceFacts {
    pub action: GuidanceAction,
    pub path: GuidancePath,
    pub actor: GuidanceActor,
    pub identity_attested: bool,
    pub trusted_origin: Option<MessageOrigin>,
    pub child_proof: bool,
    pub actor_is_target: bool,
    pub actor_is_parent: bool,
    pub actor_is_child: bool,
    pub actor_is_author: bool,
    pub message_editable: bool,
    pub recipient_turn_active: bool,
    pub carrier_queue_fallback: bool,
    pub cross_project: bool,
    pub paused: bool,
    pub target_has_active_native_task: bool,
    pub recent_interrupts_pair: u32,
    pub recent_interrupts_target: u32,
    pub now: i64,
    pub last_interrupt_pair_at: Option<i64>,
    pub third_latest_interrupt_target_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuidanceRefusal {
    IntentNotPermitted { reason: &'static str },
    ReferentPaused,
    MissionActive,
    NotEditable,
    NoActiveTurn,
    SteerUnavailable,
    InterruptRateLimited { retry_after_secs: u64 },
}

/// Sans I/O ; O(1), ou O(1+Bactor) pour les copies locales de QueuePromote.
/// Les timestamps et comptes proviennent de guidance_events.
/// Les fenêtres sont libérées exactement à at+30 ou at+600, compte SQL inclusif.
pub(crate) fn authorize(facts: &GuidanceFacts) -> Result<(), GuidanceRefusal> {
    use GuidanceAction::{Interrupt, MidTurnDelivery, QueueList};
    use GuidanceRefusal::IntentNotPermitted;
    if matches!(
        facts.action,
        GuidanceAction::QueueCancel | GuidanceAction::QueueReorder | GuidanceAction::QueuePromote
    ) && facts.actor == GuidanceActor::InternalPause
    {
        return Err(IntentNotPermitted {
            reason: "identity_unattested",
        });
    }
    if facts.action == GuidanceAction::QueuePromote {
        // Avant reçu : droits Steer réemployés, la pause est évaluée après le rejeu.
        let mut steer = facts.clone();
        steer.action = GuidanceAction::Steer;
        steer.paused = false;
        return authorize(&steer);
    }
    if facts.actor == GuidanceActor::InternalPause {
        return Ok(());
    }
    if facts.child_proof && facts.action != MidTurnDelivery {
        return Err(IntentNotPermitted {
            reason: "identity_unattested",
        });
    }
    if facts.action == QueueList {
        // Le caller filtre toute la file ou les seuls messages propres selon R5.
        return Ok(());
    }
    if facts.action == MidTurnDelivery && facts.trusted_origin == Some(MessageOrigin::System) {
        return Ok(());
    }
    if !facts.identity_attested {
        return Err(IntentNotPermitted {
            reason: "identity_unattested",
        });
    }
    if facts.actor == GuidanceActor::AttachedHuman {
        return Ok(());
    }
    if facts.actor == GuidanceActor::BareClient {
        return if facts.action == MidTurnDelivery
            && facts.trusted_origin == Some(MessageOrigin::Human)
        {
            Ok(())
        } else {
            Err(IntentNotPermitted {
                reason: "identity_unattested",
            })
        };
    }
    if facts.actor_is_target {
        return Err(IntentNotPermitted {
            reason: "self_target",
        });
    }
    if facts.cross_project {
        return Err(IntentNotPermitted {
            reason: "cross_project",
        });
    }
    if matches!(
        facts.action,
        GuidanceAction::QueueCancel | GuidanceAction::QueueReorder
    ) {
        return if facts.actor_is_parent
            || (facts.action == GuidanceAction::QueueCancel && facts.actor_is_author)
        {
            Ok(())
        } else {
            Err(IntentNotPermitted {
                reason: "not_parent",
            })
        };
    }
    if !(facts.actor_is_parent
        || facts.action == MidTurnDelivery && facts.actor_is_child
        || matches!(facts.actor, GuidanceActor::Loop { .. })
            && facts.path == GuidancePath::SendIdempotent
            && matches!(facts.action, GuidanceAction::Steer | Interrupt))
    {
        return Err(IntentNotPermitted {
            reason: "not_parent",
        });
    }
    if facts.paused {
        return Err(GuidanceRefusal::ReferentPaused);
    }
    if facts.action != Interrupt {
        return Ok(());
    }
    if facts.target_has_active_native_task {
        return Err(GuidanceRefusal::MissionActive);
    }
    let pair_retry = if facts.recent_interrupts_pair == 0 {
        0
    } else {
        facts.last_interrupt_pair_at.map_or(30, |at| {
            at.saturating_add(30).saturating_sub(facts.now).max(0) as u64
        })
    };
    let target_retry = if facts.recent_interrupts_target < 3 {
        0
    } else {
        facts.third_latest_interrupt_target_at.map_or(600, |at| {
            at.saturating_add(600).saturating_sub(facts.now).max(0) as u64
        })
    };
    let retry_after_secs = pair_retry.max(target_retry);
    if retry_after_secs > 0 {
        return Err(GuidanceRefusal::InterruptRateLimited { retry_after_secs });
    }
    Ok(())
}

/// Après le reçu : O(1+Bactor), mêmes droits156, puis phase et gardes de promotion.
/// Le caller rejoue un reçu connu avant ce contrôle, sans relancer son effet.
pub(crate) fn authorize_queue_edit(facts: &GuidanceFacts) -> Result<(), GuidanceRefusal> {
    authorize(facts)?;
    if !facts.message_editable {
        return Err(GuidanceRefusal::NotEditable);
    }
    if facts.action == GuidanceAction::QueuePromote {
        let mut steer = facts.clone();
        steer.action = GuidanceAction::Steer;
        // Cette fois, la garde Steer reçoit la vraie pause et conserve l'exception humaine.
        authorize(&steer)?;
        if !facts.recipient_turn_active {
            return Err(GuidanceRefusal::NoActiveTurn);
        }
        if !facts.carrier_queue_fallback {
            return Err(GuidanceRefusal::SteerUnavailable);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "guidance_tests.rs"]
mod tests;
