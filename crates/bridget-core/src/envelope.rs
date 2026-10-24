//! Construction et validation d'enveloppe pour l'affichage dans le pane.
//!
//! Le format est calqué sur le bridge V1 : balises lisibles par l'agent CLI.

use crate::message::BridgetMessage;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Génère le texte de l'enveloppe à injecter dans le pane de l'agent destinataire.
/// Format compact avec instruction de réponse explicite.
pub const HUMAN_REPLY_NOTICE: &str = "Instruction de l’utilisateur : réponds normalement dans ce fil, que l’utilisateur lit depuis Bridget. N’envoie pas de message au destinataire « human » : ce nom n’est pas routable. Pour une demande simple, réponds directement ; aucun outil Bridget n’est nécessaire pour transmettre ta réponse.";

/// Présentation seule : les clients humains historiques n'avaient pas origin ; les agents sont routés par UUID.
pub fn is_human_instruction(msg: &BridgetMessage) -> bool {
    msg.origin == Some(crate::MessageOrigin::Human)
        || (msg.origin.is_none() && matches!(msg.from.as_str(), "human" | "humain") && msg.delegated_origin.is_none())
}

pub fn no_reply_notice(msg: &BridgetMessage) -> &'static str {
    if is_human_instruction(msg) { HUMAN_REPLY_NOTICE } else { NO_REPLY_NOTICE }
}

pub fn wrap_envelope(msg: &BridgetMessage) -> String {
    if !msg.reply && is_human_instruction(msg) {
        return format!("💬 Instruction de l’utilisateur (id={})\n{}\n\n{HUMAN_REPLY_NOTICE}", msg.id, msg.body);
    }
    if msg.reply {
        // reply=yes : l'expéditeur attend une réponse.
        // On rend l'instruction impossible à ignorer.
        format!(
            "💬 {from} → {to} (reply=yes, id={id})\n{body}\n\n⚠ Tu DOIS répondre à {from} avec: bridget reply \"ta réponse\"\nNe réponds pas ici. Ne dis pas \"bien reçu\". Réponds avec du contenu utile.",
            from = msg.from,
            to = msg.to,
            body = msg.body,
            id = msg.id,
        )
    } else {
        // reply=no : notification simple, pas de réponse attendue.
        format!(
            "💬 {from} → {to} (reply=no, id={id})\n{body}\n\n{NO_REPLY_NOTICE}",
            from = msg.from,
            to = msg.to,
            body = msg.body,
            id = msg.id,
        )
    }
}

/// Consigne d'un message sans réponse attendue (reply=no), commune à tous
/// les pilotes. Session 127 : l'ancienne formule « Aucune réponse inter-agent
/// attendue » a été lue comme « rien à faire », et un défaut signalé est resté
/// sans traitement. Elle sépare désormais l'accusé de réception de l'action.
pub const NO_REPLY_NOTICE: &str = "Pas d'accusé de réception : n'envoie aucun « bien reçu », même pour annoncer ton silence. Ce n'est pas une absence de tâche : si ce message demande une action, fais-la. Ta réponse finale ne sera pas relayée ; pour transmettre un résultat à l'expéditeur, fais un nouvel envoi Bridget.";

/// Gardien de quarantaine pour les IDs de messages déjà relayés.
/// Empêche un message d'être retransmis (misroute, doublon réseau, etc.).
pub struct EnvelopeGuard {
    seen_ids: HashSet<(String, String)>, // (message_id, target)
    window: Duration,
    entries: Vec<(Instant, (String, String))>,
}

impl EnvelopeGuard {
    pub fn new(window: Duration) -> Self {
        EnvelopeGuard {
            seen_ids: HashSet::new(),
            window,
            entries: Vec::new(),
        }
    }

    /// Nettoie les entrées expirées.
    fn prune(&mut self) {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.entries.retain(|(ts, _)| *ts > cutoff);
        // Reconstruire le set depuis les entrées restantes
        self.seen_ids.clear();
        for (_, key) in &self.entries {
            self.seen_ids.insert(key.clone());
        }
    }

    /// Vérifie si ce message a déjà été relayé vers cette cible.
    /// Retourne true si le message DOIT être bloqué (déjà vu).
    pub fn is_quarantined(&mut self, msg_id: &str, target: &str) -> bool {
        self.prune();
        let key = (msg_id.to_string(), target.to_string());
        self.seen_ids.contains(&key)
    }

    /// Marque un message comme relayé vers une cible.
    pub fn mark_relayed(&mut self, msg_id: &str, target: &str) {
        let key = (msg_id.to_string(), target.to_string());
        if !self.seen_ids.contains(&key) {
            self.seen_ids.insert(key.clone());
            self.entries.push((Instant::now(), key));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn mobile177_human_instruction_replies_in_the_conversation_without_a_routable_human() {
        let mut message=BridgetMessage::new("human","agent177","Dis-moi coucou");
        for origin in [None,Some(crate::MessageOrigin::Human)] {
            message.origin=origin;
            let rendered=wrap_envelope(&message);
            assert!(rendered.contains("Dis-moi coucou"));
            assert!(rendered.contains(HUMAN_REPLY_NOTICE));
            assert!(!rendered.contains(NO_REPLY_NOTICE));
        }
        message.origin=Some(crate::MessageOrigin::Agent);
        assert!(!is_human_instruction(&message));
        message.from="agent177".into();message.origin=None;
        assert!(wrap_envelope(&message).contains(NO_REPLY_NOTICE));
    }

    #[test]
    fn test_envelope_format() {
        let mut msg = BridgetMessage::new("claude-1", "codex-1", "Analyse ce fichier");
        msg.id = "mcp-38210-6a8a7fc7-1".to_string();
        let env = wrap_envelope(&msg);
        assert!(env.contains("claude-1"));
        assert!(env.contains("codex-1"));
        assert!(env.contains("Analyse ce fichier"));
        assert!(env.contains("reply=no"));
        assert!(env.contains("id=mcp-38210-6a8a7fc7-1"));
        assert!(env.contains("💬"));
        assert!(env.contains("Pas d'accusé de réception"));
        assert!(env.contains("accusé"));
        assert!(!env.contains("Tu DOIS répondre"));
    }

    #[test]
    fn demande_suivie_affiche_l_identifiant_integral() {
        let mut msg = BridgetMessage::new("bridget", "codex-1", "Réponds précisément");
        msg.id = "mcp-38210-6a8a7fc7-1".to_string();
        msg.reply = true;

        let env = wrap_envelope(&msg);

        assert!(env.contains("(reply=yes, id=mcp-38210-6a8a7fc7-1)"));
        assert!(!env.contains("id=mcp-3821)"));
    }

    #[test]
    fn test_guard_quarantine() {
        let mut guard = EnvelopeGuard::new(Duration::from_secs(60));
        assert!(!guard.is_quarantined("abc123", "codex-1"));
        guard.mark_relayed("abc123", "codex-1");
        assert!(guard.is_quarantined("abc123", "codex-1"));
    }

    #[test]
    fn test_guard_different_target_not_quarantined() {
        let mut guard = EnvelopeGuard::new(Duration::from_secs(60));
        guard.mark_relayed("abc123", "codex-1");
        assert!(!guard.is_quarantined("abc123", "codex-2"));
    }

    #[test]
    fn test_guard_expiry() {
        let mut guard = EnvelopeGuard::new(Duration::from_millis(50));
        guard.mark_relayed("abc123", "codex-1");
        thread::sleep(Duration::from_millis(60));
        assert!(!guard.is_quarantined("abc123", "codex-1"));
    }
}
