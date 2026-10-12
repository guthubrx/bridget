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
            actor_is_author: false,
            message_editable: true,
            recipient_turn_active: true,
            carrier_queue_fallback: true,
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

    #[test]
    fn edit159_rights_matrix() {
        for action in [
            GuidanceAction::QueueCancel,
            GuidanceAction::QueueReorder,
            GuidanceAction::QueuePromote,
        ] {
            let author_refusal = (action != GuidanceAction::QueueCancel).then_some("not_parent");
            let cases = [
                (
                    "parent",
                    GuidanceActor::Agent {
                        id: "parent159".into(),
                    },
                    true,
                    false,
                    false,
                    true,
                    false,
                    false,
                    None,
                ),
                (
                    "humain attaché",
                    GuidanceActor::AttachedHuman,
                    false,
                    false,
                    false,
                    true,
                    false,
                    false,
                    None,
                ),
                (
                    "auteur",
                    GuidanceActor::Agent {
                        id: "author159".into(),
                    },
                    false,
                    true,
                    false,
                    true,
                    false,
                    false,
                    author_refusal,
                ),
                (
                    "destinataire",
                    GuidanceActor::Agent {
                        id: "target159".into(),
                    },
                    false,
                    false,
                    true,
                    true,
                    false,
                    false,
                    Some("self_target"),
                ),
                (
                    "destinataire auteur",
                    GuidanceActor::Agent {
                        id: "target159".into(),
                    },
                    false,
                    true,
                    true,
                    true,
                    false,
                    false,
                    Some("self_target"),
                ),
                (
                    "pair",
                    GuidanceActor::Agent {
                        id: "peer159".into(),
                    },
                    false,
                    false,
                    false,
                    true,
                    false,
                    false,
                    Some("not_parent"),
                ),
                (
                    "client nu",
                    GuidanceActor::BareClient,
                    false,
                    true,
                    false,
                    true,
                    false,
                    false,
                    Some("identity_unattested"),
                ),
                (
                    "preuve enfant",
                    GuidanceActor::Agent {
                        id: "child159".into(),
                    },
                    true,
                    true,
                    false,
                    true,
                    true,
                    false,
                    Some("identity_unattested"),
                ),
                (
                    "identité non attestée",
                    GuidanceActor::Agent {
                        id: "parent159".into(),
                    },
                    true,
                    false,
                    false,
                    false,
                    false,
                    false,
                    Some("identity_unattested"),
                ),
                (
                    "autre projet",
                    GuidanceActor::Agent {
                        id: "author159".into(),
                    },
                    true,
                    true,
                    false,
                    true,
                    false,
                    true,
                    Some("cross_project"),
                ),
                (
                    "pause interne sans droit neuf",
                    GuidanceActor::InternalPause,
                    false,
                    false,
                    false,
                    true,
                    false,
                    false,
                    Some("identity_unattested"),
                ),
            ];
            for (label, actor, parent, author, target, attested, child, cross, refusal) in cases {
                let mut f = facts(action);
                f.actor = actor;
                f.actor_is_parent = parent;
                f.actor_is_author = author;
                f.actor_is_target = target;
                f.identity_attested = attested;
                f.child_proof = child;
                f.cross_project = cross;
                let expected = refusal.map_or(Ok(()), |reason| {
                    Err(GuidanceRefusal::IntentNotPermitted { reason })
                });
                assert_eq!(
                    authorize(&f),
                    expected,
                    "identité/droits {label}/{action:?}"
                );
                assert_eq!(
                    authorize_queue_edit(&f),
                    expected,
                    "édition {label}/{action:?}"
                );
            }
        }
    }

    #[test]
    fn edit159_rights_pause_et_gardes() {
        for action in [
            GuidanceAction::QueueCancel,
            GuidanceAction::QueueReorder,
            GuidanceAction::QueuePromote,
        ] {
            let mut f = facts(action);
            f.paused = true;
            assert_eq!(authorize(&f), Ok(()), "avant reçu: droits seuls/{action:?}");
            let expected = if action == GuidanceAction::QueuePromote {
                Err(GuidanceRefusal::ReferentPaused)
            } else {
                Ok(())
            };
            assert_eq!(authorize_queue_edit(&f), expected, "pause/{action:?}");
            f.message_editable = false;
            assert_eq!(
                authorize(&f),
                Ok(()),
                "reçu consultable après changement de phase"
            );
            assert_eq!(
                authorize_queue_edit(&f),
                Err(GuidanceRefusal::NotEditable),
                "phase avant pause"
            );
            f.actor = GuidanceActor::AttachedHuman;
            f.actor_is_parent = false;
            assert_eq!(authorize_queue_edit(&f), Err(GuidanceRefusal::NotEditable));
            f.message_editable = true;
            assert_eq!(
                authorize_queue_edit(&f),
                Ok(()),
                "exception humaine attestée/{action:?}"
            );
            assert!(f.paused, "la garde ne modifie pas les faits source");
        }
        let mut f = facts(GuidanceAction::QueuePromote);
        f.recipient_turn_active = false;
        f.carrier_queue_fallback = false;
        assert_eq!(
            authorize_queue_edit(&f),
            Err(GuidanceRefusal::NoActiveTurn),
            "tour avant fallback"
        );
        f.recipient_turn_active = true;
        assert_eq!(
            authorize_queue_edit(&f),
            Err(GuidanceRefusal::SteerUnavailable)
        );
        f.carrier_queue_fallback = true;
        assert_eq!(authorize_queue_edit(&f), Ok(()));
    }
}
