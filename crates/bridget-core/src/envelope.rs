//! Construction et validation d'enveloppe pour l'affichage dans le pane.
//!
//! Le format est calqué sur le bridge V1 : balises lisibles par l'agent CLI.

use crate::message::BridgetMessage;
use std::collections::HashSet;
use std::time::{Duration, Instant};

/// Génère le texte de l'enveloppe à injecter dans le pane de l'agent destinataire.
/// Format compact avec instruction de réponse explicite.
pub fn wrap_envelope(msg: &BridgetMessage) -> String {
    let reply_str = if msg.reply { "yes" } else { "no" };
    let id_short = &msg.id[..msg.id.len().min(8)];

    if msg.reply {
        // reply=yes : l'expéditeur attend une réponse.
        // On rend l'instruction impossible à ignorer.
        format!(
            "💬 {from} → {to} (reply=yes, id={id})\n{body}\n\n⚠ Tu DOIS répondre à {from} avec: bridget send --to {from} \"ta réponse\"\nNe réponds pas ici. Ne dis pas \"bien reçu\". Réponds avec du contenu utile.",
            from = msg.from,
            to = msg.to,
            body = msg.body,
            id = id_short,
        )
    } else {
        // reply=no : notification simple, pas de réponse attendue.
        format!(
            "💬 {from} → {to} (reply=no, id={id})\n{body}",
            from = msg.from,
            to = msg.to,
            body = msg.body,
            id = id_short,
        )
    }
}

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
    fn test_envelope_format() {
        let msg = BridgetMessage::new("claude-1", "codex-1", "Analyse ce fichier");
        let env = wrap_envelope(&msg);
        assert!(env.contains("claude-1"));
        assert!(env.contains("codex-1"));
        assert!(env.contains("Analyse ce fichier"));
        assert!(env.contains("reply=no"));
        assert!(env.contains("💬"));
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
