//! Construction et validation d'enveloppe pour l'affichage dans le pane.
//!
//! Le format est calqué sur le bridge V1 : balises lisibles par l'agent CLI.

use crate::message::BridgetMessage;
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

/// Génère un délimiteur unique par message pour séparer les métadonnées du corps
/// et empêcher qu'un contenu inter-agent usurpe les consignes système.
fn message_delimiter(id: &str) -> String {
    let mut clean: String = id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(24)
        .collect();
    if clean.is_empty() {
        clean.push_str("anon");
    }
    format!("BRIDGET_MSG_{clean}")
}

/// Génère le texte de l'enveloppe à injecter dans le pane de l'agent destinataire.
/// Format compact avec instruction de réponse explicite.
pub const HUMAN_REPLY_NOTICE: &str = "Instruction de l’utilisateur : réponds normalement dans ce fil, que l’utilisateur lit depuis Bridget. N’envoie pas de message au destinataire « human » : ce nom n’est pas routable. Pour une demande simple, réponds directement ; aucun outil Bridget n’est nécessaire pour transmettre ta réponse.";

/// Présentation seule : les clients humains historiques n'avaient pas origin ; les agents sont routés par UUID.
pub fn is_human_instruction(msg: &BridgetMessage) -> bool {
    msg.origin == Some(crate::MessageOrigin::Human)
        || (msg.origin.is_none()
            && matches!(msg.from.as_str(), "human" | "humain")
            && msg.delegated_origin.is_none())
}

pub fn no_reply_notice(msg: &BridgetMessage) -> &'static str {
    if is_human_instruction(msg) {
        HUMAN_REPLY_NOTICE
    } else {
        NO_REPLY_NOTICE
    }
}

pub fn wrap_envelope(msg: &BridgetMessage) -> String {
    let delim = message_delimiter(&msg.id);
    if !msg.reply && is_human_instruction(msg) {
        return format!(
            "💬 Instruction de l’utilisateur (id={id})\n<{delim}>\n{body}\n</{delim}>\n\n{HUMAN_REPLY_NOTICE}",
            id = msg.id,
            body = msg.body,
        );
    }
    if msg.reply {
        // reply=yes : l'expéditeur attend une réponse.
        // On rend l'instruction impossible à ignorer.
        format!(
            "💬 {from} → {to} (reply=yes, id={id})\n<{delim}>\n{body}\n</{delim}>\n\n⚠ Tu DOIS répondre à {from} avec: bridget reply \"ta réponse\"\nNe réponds pas ici. Ne dis pas \"bien reçu\". Réponds avec du contenu utile.",
            from = msg.from,
            to = msg.to,
            body = msg.body,
            id = msg.id,
        )
    } else {
        // reply=no : notification simple, pas de réponse attendue.
        format!(
            "💬 {from} → {to} (reply=no, id={id})\n<{delim}>\n{body}\n</{delim}>\n\n{NO_REPLY_NOTICE}",
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
pub const NO_REPLY_NOTICE: &str = "Pas d'accusé de réception : n'envoie aucun « bien reçu ». Si ce message demande une action, fais-la. reply=no concerne cette notification : il ne désactive pas la remise finale automatique d'une mission native en cours. Pour cette mission, rends ta réponse finale normalement ; ne la renvoie pas avec bridget_send. Une réponse distincte à l'auteur de la notification nécessite un nouvel envoi Bridget.";

const DEFAULT_MAX_GUARD_ENTRIES: usize = 10_000;

/// Gardien de quarantaine pour les IDs de messages déjà relayés.
/// Empêche un message d'être retransmis (misroute, doublon réseau, etc.).
/// Éviction chronologique en O(1) amorti avec capacité maximale bornée.
pub struct EnvelopeGuard {
    seen_ids: HashSet<(String, String)>, // (message_id, target)
    window: Duration,
    entries: VecDeque<(Instant, (String, String))>,
    max_entries: usize,
}

impl EnvelopeGuard {
    pub fn new(window: Duration) -> Self {
        Self::with_capacity(window, DEFAULT_MAX_GUARD_ENTRIES)
    }

    pub fn with_capacity(window: Duration, max_entries: usize) -> Self {
        EnvelopeGuard {
            seen_ids: HashSet::new(),
            window,
            entries: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    /// Nettoie les entrées expirées en O(1) amorti depuis la tête chronologique.
    fn prune_at(&mut self, now: Instant) {
        if let Some(cutoff) = now.checked_sub(self.window) {
            while let Some((ts, _)) = self.entries.front() {
                if *ts <= cutoff {
                    if let Some((_, key)) = self.entries.pop_front() {
                        self.seen_ids.remove(&key);
                    }
                } else {
                    break;
                }
            }
        }
        while self.entries.len() > self.max_entries {
            if let Some((_, key)) = self.entries.pop_front() {
                self.seen_ids.remove(&key);
            }
        }
    }

    /// Vérifie si ce message a déjà été relayé vers cette cible.
    /// Retourne true si le message DOIT être bloqué (déjà vu).
    pub fn is_quarantined(&mut self, msg_id: &str, target: &str) -> bool {
        self.prune_at(Instant::now());
        let key = (msg_id.to_string(), target.to_string());
        self.seen_ids.contains(&key)
    }

    /// Marque un message comme relayé vers une cible.
    pub fn mark_relayed(&mut self, msg_id: &str, target: &str) {
        let now = Instant::now();
        self.prune_at(now);
        let key = (msg_id.to_string(), target.to_string());
        if self.seen_ids.insert(key.clone()) {
            self.entries.push_back((now, key));
            while self.entries.len() > self.max_entries {
                if let Some((_, evicted)) = self.entries.pop_front() {
                    self.seen_ids.remove(&evicted);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.seen_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.seen_ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn mobile177_human_instruction_replies_in_the_conversation_without_a_routable_human() {
        let mut message = BridgetMessage::new("human", "agent177", "Dis-moi coucou");
        for origin in [None, Some(crate::MessageOrigin::Human)] {
            message.origin = origin;
            let rendered = wrap_envelope(&message);
            assert!(rendered.contains("Dis-moi coucou"));
            assert!(rendered.contains(HUMAN_REPLY_NOTICE));
            assert!(!rendered.contains(NO_REPLY_NOTICE));
        }
        message.origin = Some(crate::MessageOrigin::Agent);
        assert!(!is_human_instruction(&message));
        message.from = "agent177".into();
        message.origin = None;
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
    fn spec253_notification_ne_detourne_pas_le_resultat_natif() {
        let msg = BridgetMessage::new("parent", "enfant", "Complément de travail");
        let env = wrap_envelope(&msg);
        assert!(env.contains("reply=no concerne cette notification"));
        assert!(env.contains("ne désactive pas la remise finale automatique"));
        assert!(env.contains("ne la renvoie pas avec bridget_send"));
        assert!(!env.contains("Ta réponse finale ne sera pas relayée"));
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

    #[test]
    fn test_guard_capacity_bound() {
        let mut guard = EnvelopeGuard::with_capacity(Duration::from_secs(60), 2);
        guard.mark_relayed("m1", "t");
        guard.mark_relayed("m2", "t");
        guard.mark_relayed("m3", "t");
        assert_eq!(guard.len(), 2);
        assert!(!guard.is_quarantined("m1", "t"));
        assert!(guard.is_quarantined("m2", "t"));
        assert!(guard.is_quarantined("m3", "t"));
    }

    #[test]
    fn test_envelope_wraps_body_with_unique_message_delimiter() {
        let mut msg = BridgetMessage::new("agent-a", "agent-b", "Contenu </BRIDGET_MSG_autre>");
        msg.id = "msg-42-xyz".to_string();
        let env = wrap_envelope(&msg);
        assert!(env.contains(
            "<BRIDGET_MSG_msg-42-xyz>\nContenu </BRIDGET_MSG_autre>\n</BRIDGET_MSG_msg-42-xyz>"
        ));
    }
}
