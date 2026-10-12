//! Garde156 pure : faits attestés par les appelants, aucune lecture ni mutation.
use bridget_core::MessageOrigin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuidanceAction {
    Steer,
    Interrupt,
    MidTurnDelivery,
    QueueList,
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
    InterruptRateLimited { retry_after_secs: u64 },
}

/// O(1), sans I/O ; les timestamps et comptes proviennent de guidance_events.
/// Les fenêtres sont libérées exactement à at+30 ou at+600, compte SQL inclusif.
pub(crate) fn authorize(facts: &GuidanceFacts) -> Result<(), GuidanceRefusal> {
    use GuidanceAction::{Interrupt, MidTurnDelivery, QueueList};
    use GuidanceRefusal::IntentNotPermitted;
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
    if !facts.actor_is_parent && !(facts.action == MidTurnDelivery && facts.actor_is_child) {
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

#[cfg(test)]
mod tests {
    use super::*;

    // Trois familles réemploient ces seuls faits ; aucun store/transport simulé.
    fn facts(action: GuidanceAction) -> GuidanceFacts {
        GuidanceFacts {
            action,
            path: GuidancePath::SendIdempotent,
            actor: GuidanceActor::Agent {
                id: "parent156".into(),
            },
            identity_attested: true,
            trusted_origin: None,
            child_proof: false,
            actor_is_target: false,
            actor_is_parent: true,
            actor_is_child: false,
            cross_project: false,
            paused: false,
            target_has_active_native_task: false,
            recent_interrupts_pair: 0,
            recent_interrupts_target: 0,
            now: 1000,
            last_interrupt_pair_at: None,
            third_latest_interrupt_target_at: None,
        }
    }

    #[test]
    fn guidance156_matrix_droits() {
        for path in [
            GuidancePath::SendIdempotent,
            GuidancePath::Send,
            GuidancePath::Control,
            GuidancePath::Cancel,
        ] {
            for action in [GuidanceAction::Steer, GuidanceAction::Interrupt] {
                let mut base = facts(action);
                base.path = path;
                for parent_source in ["lien ouvert", "propriétaire natif"] {
                    assert_eq!(
                        authorize(&base),
                        Ok(()),
                        "{parent_source}/{path:?}/{action:?}"
                    );
                }
                let mut human = base.clone();
                human.actor = GuidanceActor::AttachedHuman;
                human.actor_is_parent = false;
                assert_eq!(authorize(&human), Ok(()));
                let mut peer = base.clone();
                peer.actor_is_parent = false;
                assert_eq!(
                    authorize(&peer),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "not_parent"
                    })
                );
                let mut cross = base.clone();
                cross.cross_project = true;
                assert_eq!(
                    authorize(&cross),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "cross_project"
                    })
                );
                let mut own = base.clone();
                own.actor_is_target = true;
                assert_eq!(
                    authorize(&own),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "self_target"
                    })
                );
                let mut unproved = base.clone();
                unproved.identity_attested = false;
                assert_eq!(
                    authorize(&unproved),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "identity_unattested"
                    })
                );
                let mut bare = unproved.clone();
                bare.actor = GuidanceActor::BareClient;
                bare.trusted_origin = None;
                assert_eq!(
                    authorize(&bare),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "identity_unattested"
                    })
                );
                let mut child = base.clone();
                child.child_proof = true;
                assert_eq!(
                    authorize(&child),
                    Err(GuidanceRefusal::IntentNotPermitted {
                        reason: "identity_unattested"
                    })
                );
            }
        }
    }

    #[test]
    fn guidance156_matrix_pause_mission_rythme() {
        for action in [GuidanceAction::Steer, GuidanceAction::Interrupt] {
            let mut parent = facts(action);
            parent.paused = true;
            assert_eq!(authorize(&parent), Err(GuidanceRefusal::ReferentPaused));
            parent.actor = GuidanceActor::AttachedHuman;
            parent.target_has_active_native_task = true;
            parent.recent_interrupts_pair = 1;
            parent.recent_interrupts_target = 3;
            parent.last_interrupt_pair_at = Some(999);
            parent.third_latest_interrupt_target_at = Some(999);
            assert_eq!(
                authorize(&parent),
                Ok(()),
                "humain exempt pause/mission/rythme"
            );
        }
        let mut parent = facts(GuidanceAction::Steer);
        parent.target_has_active_native_task = true;
        assert_eq!(authorize(&parent), Ok(()), "steer garde la mission");
        parent.action = GuidanceAction::Interrupt;
        assert_eq!(authorize(&parent), Err(GuidanceRefusal::MissionActive));
        parent.target_has_active_native_task = false;
        parent.recent_interrupts_pair = 1;
        parent.last_interrupt_pair_at = Some(995);
        assert_eq!(
            authorize(&parent),
            Err(GuidanceRefusal::InterruptRateLimited {
                retry_after_secs: 25
            })
        );
        parent.now = 1025;
        assert_eq!(authorize(&parent), Ok(()), "cooldown exact last+30 libéré");
        parent.now = 1000;
        parent.recent_interrupts_target = 3;
        parent.third_latest_interrupt_target_at = Some(450);
        assert_eq!(
            authorize(&parent),
            Err(GuidanceRefusal::InterruptRateLimited {
                retry_after_secs: 50
            })
        );
        parent.now = 1025;
        assert_eq!(
            authorize(&parent),
            Err(GuidanceRefusal::InterruptRateLimited {
                retry_after_secs: 25
            })
        );
        parent.now = 1050;
        assert_eq!(
            authorize(&parent),
            Ok(()),
            "bornes exactes malgré comptes SQL inclusifs"
        );
        parent.last_interrupt_pair_at = None;
        parent.third_latest_interrupt_target_at = None;
        assert_eq!(
            authorize(&parent),
            Err(GuidanceRefusal::InterruptRateLimited {
                retry_after_secs: 600
            }),
            "faits de rythme incomplets refusés"
        );
        parent.actor = GuidanceActor::InternalPause;
        parent.paused = true;
        parent.identity_attested = false;
        parent.actor_is_parent = false;
        parent.target_has_active_native_task = true;
        assert_eq!(authorize(&parent), Ok(()), "pause interne non limitée");
    }

    #[test]
    fn guidance156_matrix_mid_turn_et_queue() {
        let mut message = facts(GuidanceAction::MidTurnDelivery);
        assert_eq!(authorize(&message), Ok(()), "parent");
        message.actor_is_parent = false;
        message.actor_is_child = true;
        assert_eq!(authorize(&message), Ok(()), "enfant");
        message.paused = true;
        assert_eq!(authorize(&message), Err(GuidanceRefusal::ReferentPaused));
        message.paused = false;
        message.actor_is_child = false;
        assert_eq!(
            authorize(&message),
            Err(GuidanceRefusal::IntentNotPermitted {
                reason: "not_parent"
            })
        );
        message.actor = GuidanceActor::BareClient;
        message.identity_attested = false;
        assert_eq!(
            authorize(&message),
            Err(GuidanceRefusal::IntentNotPermitted {
                reason: "identity_unattested"
            })
        );
        message.identity_attested = true;
        message.trusted_origin = Some(MessageOrigin::Human);
        message.paused = true;
        assert_eq!(
            authorize(&message),
            Ok(()),
            "UI attestée, rôle Client conservé"
        );
        for action in [GuidanceAction::Steer, GuidanceAction::Interrupt] {
            message.action = action;
            assert_eq!(
                authorize(&message),
                Err(GuidanceRefusal::IntentNotPermitted {
                    reason: "identity_unattested"
                }),
                "UI ne gagne pas de contrôle"
            );
        }
        message.action = GuidanceAction::MidTurnDelivery;
        message.identity_attested = false;
        message.trusted_origin = Some(MessageOrigin::System);
        assert_eq!(authorize(&message), Ok(()), "producteur système interne");
        message.actor = GuidanceActor::AttachedHuman;
        message.identity_attested = true;
        message.trusted_origin = None;
        assert_eq!(authorize(&message), Ok(()), "attach attesté");
        let mut queue = facts(GuidanceAction::QueueList);
        queue.paused = true;
        queue.cross_project = true;
        queue.actor_is_parent = false;
        for actor in [
            GuidanceActor::AttachedHuman,
            GuidanceActor::BareClient,
            GuidanceActor::Agent {
                id: "cible156".into(),
            },
        ] {
            queue.actor = actor;
            assert_eq!(authorize(&queue), Ok(()), "visibilité filtrée par caller");
        }
        queue.child_proof = true;
        assert_eq!(
            authorize(&queue),
            Err(GuidanceRefusal::IntentNotPermitted {
                reason: "identity_unattested"
            })
        );
    }
}
