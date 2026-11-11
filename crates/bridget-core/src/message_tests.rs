#[test]
fn spec140_titre_facultatif_racine_et_ancien_lecteur_compatibles() {
    let old: super::BridgetMessage =
        serde_json::from_str(r#"{"id":"m","from":"a","to":"b","body":"x"}"#).unwrap();
    assert!(
        serde_json::to_value(&old)
            .unwrap()
            .get("thread_display_title")
            .is_none()
    );
    let mut wire = serde_json::to_value(&old).unwrap();
    wire["thread_display_title"] = serde_json::json!("Politique");
    let current: super::BridgetMessage = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(current).unwrap()["thread_display_title"],
        "Politique"
    );
    #[derive(serde::Deserialize)]
    struct OldRoot {
        id: String,
    }
    assert_eq!(
        serde_json::from_value::<OldRoot>(wire.clone()).unwrap().id,
        "m"
    );
    wire["thread_notice"] = serde_json::json!({"version":1,"thread_id":"f","through_seq":1,"generation":1,"title":"forgé"});
    assert!(serde_json::from_value::<super::BridgetMessage>(wire).is_err());
}

#[test]
fn spec134_libelle_associe_nom_humain_et_uuid_complet() {
    let agent_id = "cbd8d228-62ef-494b-80b7-bf08333591d6";
    let mut message = super::BridgetMessage::new(agent_id, "destinataire", "message");
    message.from_display_name = Some(" Regional ".into());

    assert_eq!(message.sender_label(), format!("Regional ({agent_id})"));
}

#[test]
fn spec134_libelle_replie_les_noms_inexploitables_sur_uuid() {
    let agent_id = "cbd8d228-62ef-494b-80b7-bf08333591d6";
    for display_name in [None, Some(""), Some("   "), Some(agent_id)] {
        let mut message = super::BridgetMessage::new(agent_id, "destinataire", "message");
        message.from_display_name = display_name.map(str::to_string);
        assert_eq!(message.sender_label(), agent_id, "cas {display_name:?}");
    }
}

#[test]
fn spec134_libelle_garde_la_provenance_apres_le_parent_nomme() {
    let agent_id = "cbd8d228-62ef-494b-80b7-bf08333591d6";
    let mut message = super::BridgetMessage::new(agent_id, "destinataire", "message");
    message.from_display_name = Some("Regional".into());
    message.delegated_origin = Some(super::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
    });

    assert_eq!(
        message.sender_label(),
        format!("Regional ({agent_id}) (via sous-agent codex 0123456789abcdef)")
    );
}

#[test]
fn spec133_provenance_deleguee_est_fermee_et_retrocompatible() {
    let old: super::BridgetMessage =
        serde_json::from_str(r#"{"id":"m","from":"a","to":"b","body":"x"}"#).unwrap();
    assert!(old.delegated_origin.is_none());
    assert!(
        serde_json::to_value(&old)
            .unwrap()
            .get("delegated_origin")
            .is_none()
    );

    let mut message = super::BridgetMessage::new("a", "b", "x");
    message.delegated_origin = Some(super::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
    });
    let encoded = serde_json::to_string(&message).unwrap();
    let decoded: super::BridgetMessage = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.delegated_origin, message.delegated_origin);
    assert_eq!(
        decoded.sender_label(),
        "a (via sous-agent codex 0123456789abcdef)"
    );
    assert!(
            serde_json::from_str::<super::BridgetMessage>(
                r#"{"id":"m","from":"a","to":"b","body":"x","delegated_origin":{"provider":"codex","child_ref":"0123456789abcdef","session":"secrete"}}"#
            )
            .is_err()
        );
}

#[test]
fn spec102_thread_notice_absente_par_defaut_et_omise() {
    let message = super::BridgetMessage::new("a", "b", "corps");
    assert!(message.thread_notice.is_none());
    let json = serde_json::to_string(&message).unwrap();
    assert!(!json.contains("thread_notice"));
    let old: super::BridgetMessage =
        serde_json::from_str(r#"{"id":"m","from":"a","to":"b","body":"x"}"#).unwrap();
    assert!(old.thread_notice.is_none());
    let typed: super::BridgetMessage = serde_json::from_str(
            r#"{"id":"m","from":"bridget","to":"b","body":"x","thread_notice":{"version":1,"thread_id":"t","through_seq":3,"generation":2}}"#,
        )
        .unwrap();
    assert_eq!(typed.thread_notice.as_ref().unwrap().through_seq, 3);
    assert!(
            serde_json::from_str::<super::BridgetMessage>(
                r#"{"id":"m","from":"a","to":"b","body":"x","thread_notice":{"version":1,"thread_id":"t","through_seq":3,"generation":2,"body":"forge"}}"#
            )
            .is_err(),
            "champ inconnu refusé dans la notice"
        );
}

use super::*;

#[test]
fn test_message_new_generates_id() {
    let msg = BridgetMessage::new("claude", "codex", "hello");
    assert!(!msg.id.is_empty());
    assert_eq!(msg.id.len(), 13);
    assert_eq!(msg.from, "claude");
    assert_eq!(msg.to, "codex");
    assert_eq!(msg.body, "hello");
    assert!(!msg.reply);
    assert_eq!(msg.hops, 4);
}

#[test]
fn content_key_keeps_distinct_tracked_replies_distinct() {
    let mut first = BridgetMessage::new("codex", "alice", "même réponse");
    first.in_reply_to = Some("request-1".to_string());
    let mut second = first.clone();
    second.in_reply_to = Some("request-2".to_string());
    assert_ne!(first.content_key(), second.content_key());
}

#[test]
fn test_decrement_hops() {
    let mut msg = BridgetMessage::new("a", "b", "x");
    assert!(msg.decrement_hops()); // 4 -> 3
    assert!(msg.decrement_hops()); // 3 -> 2
    assert!(msg.decrement_hops()); // 2 -> 1
    assert!(!msg.decrement_hops()); // 1 -> 0, budget épuisé
}

#[test]
fn test_content_key_stable() {
    let msg1 = BridgetMessage::new("a", "codex", "hello");
    let msg2 = BridgetMessage::new("b", "codex", "hello");
    // Même destinataire + même body = même clé, même si expéditeur différent
    assert_eq!(msg1.content_key(), msg2.content_key());
}

#[test]
fn test_content_key_differs_on_body() {
    let msg1 = BridgetMessage::new("a", "codex", "hello");
    let msg2 = BridgetMessage::new("a", "codex", "world");
    assert_ne!(msg1.content_key(), msg2.content_key());
}

#[test]
fn test_content_key_differs_on_target() {
    let msg1 = BridgetMessage::new("a", "codex", "hello");
    let msg2 = BridgetMessage::new("a", "claude", "hello");
    assert_ne!(msg1.content_key(), msg2.content_key());
}

#[test]
fn test_agent_type_display() {
    assert_eq!(AgentType::Codex.to_string(), "codex");
    assert_eq!(AgentType::Claude.to_string(), "claude");
}

#[test]
fn test_message_json_roundtrip() {
    let msg = BridgetMessage::new("claude-1", "codex-1", "test message");
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: BridgetMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(msg.id, decoded.id);
    assert_eq!(msg.from, decoded.from);
    assert_eq!(msg.body, decoded.body);
}

#[test]
fn message_historique_sans_champs_de_controle_reste_acceptable() {
    let historical = r#"{"id":"legacy","from":"a","to":"b","body":"x"}"#;
    let decoded: BridgetMessage = serde_json::from_str(historical).unwrap();
    assert_eq!(decoded.origin, None);
    assert_eq!(decoded.intent, None);
    assert!(decoded.references.is_empty());
    let reencoded = serde_json::to_value(decoded).unwrap();
    assert!(reencoded.get("origin").is_none());
    assert!(reencoded.get("intent").is_none());
    assert!(reencoded.get("references").is_none());
}

#[test]
fn message_controle_serialise_origine_intention_et_references() {
    let mut msg = BridgetMessage::new("humain", "agent", "travail");
    msg.origin = Some(MessageOrigin::Human);
    msg.intent = Some(MessageIntent::InterruptAndStart);
    msg.references = vec!["objective-1".to_string(), "delegation-2".to_string()];
    let json = serde_json::to_string(&msg).unwrap();
    let decoded: BridgetMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.origin, Some(MessageOrigin::Human));
    assert_eq!(decoded.intent, Some(MessageIntent::InterruptAndStart));
    assert_eq!(decoded.references, ["objective-1", "delegation-2"]);
}

#[test]
fn enums_de_controle_sont_stables_sur_le_fil() {
    assert_eq!(
        serde_json::to_string(&MessageIntent::QueueOnly).unwrap(),
        "\"queue_only\""
    );
    assert_eq!(
        serde_json::to_string(&MessageOrigin::Routine).unwrap(),
        "\"routine\""
    );
}
