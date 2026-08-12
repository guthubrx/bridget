//! Disjoncteur — limite le nombre d'échanges par conversation dans une fenêtre glissante.
//!
//! Si une conversation dépasse N échanges dans W secondes, le transport est coupé.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Conversation key = paire triée (from, to) normalisée.
fn conv_key(from: &str, to: &str) -> String {
    let mut pair = [from, to];
    pair.sort_unstable();
    format!("{}|{}", pair[0], pair[1])
}

pub struct CircuitBreaker {
    /// Fenêtre glissante en secondes.
    window: Duration,
    /// Nombre maximum d'échanges dans la fenêtre.
    limit: usize,
    /// Historaire des timestamps par conversation.
    history: std::collections::HashMap<String, VecDeque<Instant>>,
}

impl CircuitBreaker {
    pub fn new(window_secs: u64, limit: usize) -> Self {
        CircuitBreaker {
            window: Duration::from_secs(window_secs),
            limit,
            history: std::collections::HashMap::new(),
        }
    }

    /// Vérifie si un nouvel échange serait autorisé SANS l'enregistrer.
    /// Retourne false si le disjoncteur est déclenché.
    pub fn check(&self, from: &str, to: &str) -> bool {
        let key = conv_key(from, to);
        let now = Instant::now();
        let cutoff = now - self.window;

        match self.history.get(&key) {
            None => true,
            Some(entries) => {
                let recent: usize = entries.iter().filter(|ts| **ts > cutoff).count();
                recent < self.limit
            }
        }
    }

    /// Enregistre un échange dans la fenêtre glissante.
    /// À appeler APRÈS avoir vérifié avec check().
    pub fn record(&mut self, from: &str, to: &str) {
        let key = conv_key(from, to);
        let now = Instant::now();
        let cutoff = now - self.window;

        let entries = self.history.entry(key).or_default();
        entries.push_back(now);

        // Nettoyer les entrées expirées
        while let Some(front) = entries.front() {
            if *front <= cutoff {
                entries.pop_front();
            } else {
                break;
            }
        }
    }

    /// Retourne le nombre d'échanges récents pour une conversation.
    pub fn count(&self, from: &str, to: &str) -> usize {
        let key = conv_key(from, to);
        let now = Instant::now();
        let cutoff = now - self.window;
        match self.history.get(&key) {
            None => 0,
            Some(entries) => entries.iter().filter(|ts| **ts > cutoff).count(),
        }
    }

    /// Limte configurée.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Fenêtre configurée en secondes.
    pub fn window_secs(&self) -> u64 {
        self.window.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_limit() {
        let mut cb = CircuitBreaker::new(60, 3);
        assert!(cb.check("a", "b"));
        cb.record("a", "b");
        assert!(cb.check("a", "b"));
        cb.record("a", "b");
        assert!(cb.check("a", "b"));
        cb.record("a", "b");
        // 3 échanges, limite atteinte
        assert!(!cb.check("a", "b"));
    }

    #[test]
    fn test_conv_key_symmetric() {
        let mut cb = CircuitBreaker::new(60, 1);
        cb.record("claude-1", "codex-1");
        // a->b et b->a sont la même conversation
        assert_eq!(cb.count("claude-1", "codex-1"), 1);
        assert_eq!(cb.count("codex-1", "claude-1"), 1);
    }

    #[test]
    fn test_different_conversations_independent() {
        let mut cb = CircuitBreaker::new(60, 2);
        cb.record("a", "b");
        cb.record("a", "b");
        assert!(!cb.check("a", "b")); // a-b saturé
        assert!(cb.check("a", "c")); // a-c vide
    }

    #[test]
    fn test_window_expiry() {
        let mut cb = CircuitBreaker::new(1, 2); // fenêtre 1 seconde
        cb.record("a", "b");
        cb.record("a", "b");
        assert!(!cb.check("a", "b"));
        std::thread::sleep(Duration::from_millis(1100));
        assert!(cb.check("a", "b")); // fenêtre expirée
    }
}
