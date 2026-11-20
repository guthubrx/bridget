//! Déduplicateur par contenu : empêche les doubles envois vers la même cible.
//!
//! Clé composite `(content_key, target)` avec éviction O(1) amorti en tête et capacité bornée.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const DEFAULT_MAX_DEDUP_ENTRIES: usize = 10_000;

pub struct Deduplicator {
    window: Duration,
    max_entries: usize,
    /// `(content_key, target)` -> dernier instant enregistré
    entries: HashMap<(String, String), Instant>,
    /// File chronologique des insertions pour éviction O(1) amorti
    order: VecDeque<(Instant, (String, String))>,
}

impl Deduplicator {
    pub fn new(window_secs: u64) -> Self {
        Self::with_capacity(window_secs, DEFAULT_MAX_DEDUP_ENTRIES)
    }

    pub fn with_capacity(window_secs: u64, max_entries: usize) -> Self {
        Deduplicator {
            window: Duration::from_secs(window_secs),
            max_entries: max_entries.max(1),
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    /// Vérifie si un contenu a déjà été envoyé vers cette cible récemment.
    /// Retourne true si le message DOIT être bloqué (doublon).
    pub fn is_duplicate(&mut self, content_key: &str, target: &str) -> bool {
        self.prune_at(Instant::now());
        self.entries
            .contains_key(&(content_key.to_string(), target.to_string()))
    }

    /// Enregistre un contenu comme envoyé vers une cible.
    pub fn mark_sent(&mut self, content_key: &str, target: &str) {
        let now = Instant::now();
        self.prune_at(now);
        let key = (content_key.to_string(), target.to_string());
        self.entries.insert(key.clone(), now);
        self.order.push_back((now, key));
        self.enforce_capacity();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn prune_at(&mut self, now: Instant) {
        if let Some(cutoff) = now.checked_sub(self.window) {
            while let Some((ts, _)) = self.order.front() {
                if *ts <= cutoff {
                    if let Some((evicted_ts, key)) = self.order.pop_front()
                        && self.entries.get(&key).copied() == Some(evicted_ts)
                    {
                        self.entries.remove(&key);
                    }
                } else {
                    break;
                }
            }
        }
        self.enforce_capacity();
    }

    fn enforce_capacity(&mut self) {
        while self.entries.len() > self.max_entries {
            let Some((evicted_ts, key)) = self.order.pop_front() else {
                break;
            };
            if self.entries.get(&key).copied() == Some(evicted_ts) {
                self.entries.remove(&key);
            }
        }
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
    fn test_different_target_not_duplicate_and_preserves_first_target() {
        let mut d = Deduplicator::new(60);
        d.mark_sent("key1", "codex-1");
        assert!(!d.is_duplicate("key1", "codex-2"));
        d.mark_sent("key1", "codex-2");
        // Les deux cibles restent dédupliquées indépendamment sans écrasement.
        assert!(d.is_duplicate("key1", "codex-1"));
        assert!(d.is_duplicate("key1", "codex-2"));
    }

    #[test]
    fn test_capacity_bound_evicts_oldest() {
        let mut d = Deduplicator::with_capacity(60, 2);
        d.mark_sent("k1", "a");
        d.mark_sent("k2", "a");
        d.mark_sent("k3", "a");
        assert_eq!(d.len(), 2);
        assert!(!d.is_duplicate("k1", "a"));
        assert!(d.is_duplicate("k2", "a"));
        assert!(d.is_duplicate("k3", "a"));
    }

    #[test]
    fn test_expiry() {
        let mut d = Deduplicator::new(1);
        d.mark_sent("key1", "codex-1");
        std::thread::sleep(Duration::from_millis(1100));
        assert!(!d.is_duplicate("key1", "codex-1"));
    }
}
