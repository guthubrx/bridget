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
        self.check_at(from, to, Instant::now())
    }

    /// Enregistre un échange dans la fenêtre glissante.
    /// À appeler APRÈS avoir vérifié avec check().
    pub fn record(&mut self, from: &str, to: &str) {
        self.record_at(from, to, Instant::now())
    }

    /// Retourne le nombre d'échanges récents pour une conversation.
    pub fn count(&self, from: &str, to: &str) -> usize {
        self.count_at(from, to, Instant::now())
    }

    /// Variante à horloge injectée : sert aux oracles déterministes.
    pub fn check_at(&self, from: &str, to: &str, now: Instant) -> bool {
        let key = conv_key(from, to);
        let cutoff = now - self.window;

        match self.history.get(&key) {
            None => true,
            Some(entries) => {
                let recent: usize = entries.iter().filter(|ts| **ts > cutoff).count();
                recent < self.limit
            }
        }
    }

    /// Enregistre un échange à un instant donné (tests et horloge contrôlée).
    pub fn record_at(&mut self, from: &str, to: &str, now: Instant) {
        let key = conv_key(from, to);
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

    pub fn count_at(&self, from: &str, to: &str, now: Instant) -> usize {
        let key = conv_key(from, to);
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
        // Mutation : retirer le filtre `> cutoff` (ou le nettoyage) laisse le
        // disjoncteur fermé après l'avancée d'horloge — ce test échoue alors.
        let mut cb = CircuitBreaker::new(1, 2);
        let t0 = Instant::now();
        cb.record_at("a", "b", t0);
        cb.record_at("a", "b", t0);
        assert!(!cb.check_at("a", "b", t0));
        let after_window = t0 + Duration::from_secs(1) + Duration::from_millis(1);
        assert!(cb.check_at("a", "b", after_window));
        assert_eq!(cb.count_at("a", "b", after_window), 0);
    }
}
