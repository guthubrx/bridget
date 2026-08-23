use maicie::app::{route_direct_message, DirectBridgetMessage, DirectMessageRoute};
use maicie::MAICIE_IDENTITY;

#[test]
fn un_message_direct_hors_maicie_ne_peut_pas_creer_delegation_ni_objectif() {
    let message = DirectBridgetMessage {
        from: "humain",
        to: "prospective",
        body: "Peux-tu vérifier ce changement ?",
    };

    assert_eq!(
        route_direct_message(&message, MAICIE_IDENTITY),
        DirectMessageRoute::OutsideMaicie
    );
}
