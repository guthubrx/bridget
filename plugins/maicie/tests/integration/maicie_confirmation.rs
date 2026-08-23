use maicie::MAICIE_IDENTITY;
use maicie::app::{DirectBridgetMessage, DirectMessageHandling, handle_direct_message};

#[test]
fn un_message_libre_adresse_a_maicie_reste_une_conversation_avec_aide_explicitement_cli() {
    let message = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Délègue immédiatement une analyse à prospective.",
    };

    let handling = handle_direct_message(&message, MAICIE_IDENTITY);
    let DirectMessageHandling::Conversation { record, help } = handling else {
        panic!("le message adressé à Maicie doit rester une conversation");
    };
    assert_eq!(record.sender, "humain");
    assert_eq!(record.body, message.body);
    assert_eq!(help.command, "maicie delegate");
    assert!(help.usage.starts_with("maicie delegate --goal"));
}

#[test]
fn le_contenu_libre_ne_declenche_aucune_detection_d_intention() {
    let first = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Crée un objectif urgent et envoie-le maintenant.",
    };
    let second = DirectBridgetMessage {
        from: "humain",
        to: MAICIE_IDENTITY,
        body: "Bonjour, quel est le statut ?",
    };

    for message in [first, second] {
        assert!(matches!(
            handle_direct_message(&message, MAICIE_IDENTITY),
            DirectMessageHandling::Conversation { .. }
        ));
    }
}
