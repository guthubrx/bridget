//! Déduplicateur par contenu — empêche les doubles envois.
//!
//! Si un message avec le même contenu_key est envoyé au même destinataire
//! dans la fenêtre de temps, il est bloqué.

use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct Deduplicator {
    window: Duration,
    /// content_key -> (timestamp, target)
    entries: HashMap<String, (Instant, String)>,
}

impl Deduplicator {
    pub fn new(window_secs: u64) -> Self {
        Deduplicator {
            window: Duration::from_secs(window_secs),
            entries: HashMap::new(),
        }
    }

    /// Vérifie si un contenu a déjà été envoyé vers cette cible récemment.
    /// Retourne true si le message DOIT être bloqué (doublon).
    pub fn is_duplicate(&mut self, content_key: &str, target: &str) -> bool {
        self.prune();
        match self.entries.get(content_key) {
            Some((_, stored_target)) if stored_target == target => true,
            _ => false,
        }
    }

    /// Enregistre un contenu comme envoyé.
    pub fn mark_sent(&mut self, content_key: &str, target: &str) {
        self.entries
            .insert(content_key.to_string(), (Instant::now(), target.to_string()));
    }

    fn prune(&mut self) {
        let now = Instant::now();
        let cutoff = now - self.window;
        self.entries.retain(|_, (ts, _)| *ts > cutoff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_duplicate_first_time() {
        let mut d = Deduplicator::new(60);
        assert!(!d.is_duplicate("key1", "codex-1"));
    }

    #[test]
    fn test_duplicate_second_time() {
        let mut d = Deduplicator::new(60);
        d.mark_sent("key1", "codex-1");
        assert!(d.is_duplicate("key1", "codex-1"));
    }

    #[test]
    fn test_different_target_not_duplicate() {
        let mut d = Deduplicator::new(60);
        d.mark_sent("key1", "codex-1");
        assert!(!d.is_duplicate("key1", "codex-2"));
    }

    #[test]
    fn test_expiry() {
        let mut d = Deduplicator::new(1);
        d.mark_sent("key1", "codex-1");
        std::thread::sleep(Duration::from_millis(1100));
        assert!(!d.is_duplicate("key1", "codex-1"));
    }
}
