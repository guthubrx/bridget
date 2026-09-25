//! Construction canonique commune : aucune dépendance aux façades CLI/MCP.
//!
//! Le scope identifie un namespace stable ; ce hash n'est pas une preuve
//! d'authentification. L'autorisation appartient à la connexion négociée.

pub(crate) mod client;

pub(crate) fn issuer_scope(identity: &str) -> String {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x9e3779b97f4a7c15_u64;
    for byte in identity.bytes() {
        first = (first ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        second = second.rotate_left(5) ^ u64::from(byte);
        second = second.wrapping_mul(0x9e3779b185ebca87);
    }
    format!("012_scope_{first:016x}{second:016x}")
}

fn canonical_field(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value);
}

fn canonical_option<T: ToString>(bytes: &mut Vec<u8>, value: Option<T>) {
    match value {
        Some(value) => {
            bytes.push(1);
            canonical_field(bytes, value.to_string().as_bytes());
        }
        None => bytes.push(0),
    }
}

fn canonical_message_control(bytes: &mut Vec<u8>, message: &bridget_core::BridgetMessage) {
    // Les anciens clients n'avaient pas ces champs. Ne rien ajouter pour leur
    // forme vide conserve leurs rejeux exacts ; une sémantique nouvelle ajoute
    // un suffixe distinct qui ne peut jamais réutiliser leur même identité.
    if message.origin.is_none() && message.intent.is_none() && message.references.is_empty() {
        return;
    }
    bytes.push(0xff);
    bytes.push(match message.origin {
        None => 0,
        Some(bridget_core::MessageOrigin::Human) => 1,
        Some(bridget_core::MessageOrigin::Agent) => 2,
        Some(bridget_core::MessageOrigin::Routine) => 3,
        Some(bridget_core::MessageOrigin::System) => 4,
    });
    bytes.push(match message.intent {
        None => 0,
        Some(bridget_core::MessageIntent::QueueOnly) => 1,
        Some(bridget_core::MessageIntent::TriggerTurn) => 2,
        Some(bridget_core::MessageIntent::SteerCurrent) => 3,
        Some(bridget_core::MessageIntent::InterruptAndStart) => 4,
        Some(bridget_core::MessageIntent::ControlOnly) => 5,
    });
    canonical_field(bytes, &(message.references.len() as u64).to_be_bytes());
    for reference in &message.references {
        canonical_field(bytes, reference.as_bytes());
    }
}
/// Sérialisation binaire fermée et sans ambiguïté de l'enveloppe publiée.
/// Elle ne dépend ni de l'ordre JSON ni des valeurs mutées lors du routage.
pub(crate) fn canonical_send(
    issuer_scope: &str,
    message_id: &str,
    message: &bridget_core::BridgetMessage,
    issued_at: i64,
) -> Vec<u8> {
    let mut bytes = b"bridget/client-send/v1\0".to_vec();
    canonical_field(&mut bytes, issuer_scope.as_bytes());
    canonical_field(&mut bytes, message_id.as_bytes());
    canonical_field(&mut bytes, message.to.as_bytes());
    canonical_field(&mut bytes, message.body.as_bytes());
    bytes.push(u8::from(message.reply));
    canonical_field(&mut bytes, &message.hops.to_be_bytes());
    canonical_option(&mut bytes, message.deadline_at);
    canonical_option(&mut bytes, message.in_reply_to.as_deref());
    canonical_message_control(&mut bytes, message);
    canonical_field(&mut bytes, &issued_at.to_be_bytes());
    canonical_thread_notice(&mut bytes, message);
    bytes
}

/// Session 102 : une alerte de fil ajoute un domaine distinct à la FIN du canon.
/// `None` conserve exactement les octets historiques ; chaque champ de la
/// notice modifie le canon, donc un rejeu avec une autre notice est refusé.
fn canonical_thread_notice(bytes: &mut Vec<u8>, message: &bridget_core::BridgetMessage) {
    let Some(notice) = &message.thread_notice else {
        return;
    };
    bytes.extend_from_slice(b"bridget/thread-notice/v1\0");
    canonical_field(bytes, &notice.version.to_be_bytes());
    canonical_field(bytes, notice.thread_id.as_bytes());
    canonical_field(bytes, &notice.through_seq.to_be_bytes());
    canonical_field(bytes, &notice.generation.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portee_stable_reste_identique_a_la_reference() {
        assert_eq!(
            issuer_scope("instance-stable-089"),
            "012_scope_37419e46a921ee300d273a45c16ebe1a"
        );
        assert_ne!(
            issuer_scope("instance-stable-089"),
            issuer_scope("instance-other")
        );
    }

    #[test]
    fn canon_historique_garde_ses_octets_et_ses_frontieres() {
        let mut message: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte\n  suite","reply":true,"hops":4,"deadline_at":123060,"in_reply_to":"request-0"}"#
        ).unwrap();
        let bytes = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000);
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            hex,
            "627269646765742f636c69656e742d73656e642f76310000000000000000163031325f73636f70655f61616161616161616161616100000000000000096d6573736167652d3100000000000000076167656e742d62000000000000000d74657874650a2020737569746501000000000000000400000004010000000000000006313233303630010000000000000009726571756573742d300000000000000008000000000001e078"
        );
        message.from_display_name = Some("Nouveau nom".into());
        assert_eq!(
            bytes,
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
        );
        message.in_reply_to = Some("request-other".into());
        assert_ne!(
            bytes,
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &message, 123000)
        );
    }

    #[test]
    fn spec102_v20_canon_sans_notice_inchange_et_chaque_champ_de_notice_compte() {
        let base: bridget_core::BridgetMessage = serde_json::from_str(
            r#"{"id":"message-1","from":"agent-a","to":"agent-b","body":"texte","reply":false,"hops":4}"#,
        )
        .unwrap();
        let reference = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &base, 123000);
        let mut same = base.clone();
        same.thread_notice = None;
        assert_eq!(
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &same, 123000),
            reference
        );
        let notice = bridget_core::ThreadNotice {
            version: 1,
            thread_id: "33333333-3333-4333-8333-333333333333".into(),
            through_seq: 20,
            generation: 2,
        };
        let mut with_notice = base.clone();
        with_notice.thread_notice = Some(notice.clone());
        let canon = canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &with_notice, 123000);
        assert_ne!(canon, reference);
        assert!(
            canon.starts_with(&reference),
            "le préfixe historique est conservé"
        );
        for variant in [
            bridget_core::ThreadNotice {
                version: 2,
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                thread_id: "44444444-4444-4444-8444-444444444444".into(),
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                through_seq: 21,
                ..notice.clone()
            },
            bridget_core::ThreadNotice {
                generation: 3,
                ..notice.clone()
            },
        ] {
            let mut other = base.clone();
            other.thread_notice = Some(variant);
            assert_ne!(
                canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &other, 123000),
                canon
            );
        }
    }

    #[test]
    fn les_facades_ne_sont_plus_une_dependance_du_noyau() {
        // Oracle d'architecture : remettre un appel via MCP recrée le cycle.
        for source in [
            include_str!("daemon.rs"),
            include_str!("store.rs"),
            include_str!("store/ledger_requests.rs"),
            include_str!("store/service_events.rs"),
            include_str!("store/project_compat.rs"),
            include_str!("idempotency.rs"),
            include_str!("idempotency/send_delivery.rs"),
            include_str!("idempotency/spawn_commands.rs"),
            include_str!("idempotency/agent_links.rs"),
            include_str!("referent_control.rs"),
            include_str!("cli.rs"),
        ] {
            assert!(!source.contains("crate::mcp::issuer_scope"));
        }
    }
}
