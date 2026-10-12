//! Mémoire157 volatile par nom du registre, indépendante des présences.
use bridget_transport::protocol::{RateLimitFact, RateLimitSource};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug)]
struct Seen {
    fact: RateLimitFact,
    source: RateLimitSource,
    observed_at: i64,
}

#[derive(Debug, Default)]
pub(crate) struct ProviderLimits {
    by_agent_type: BTreeMap<String, BTreeMap<String, Seen>>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Quota {
    pub windows: Vec<serde_json::Value>,
    pub exhausted: bool,
    pub limit_reached: bool,
    pub resets_at: Option<i64>,
    pub observed_at: i64,
}

impl ProviderLimits {
    /// O(log p+B), p types du registre, B chaînes copiées ; au plus9 fenêtres
    /// parcourues : purge avant upsert, puis éviction du minimum (instant, clé).
    pub(crate) fn record(
        &mut self,
        agent_type: &str,
        fact: RateLimitFact,
        source: RateLimitSource,
        observed_at: i64,
    ) {
        let windows = self.by_agent_type.entry(agent_type.to_owned()).or_default();
        windows.retain(|_, seen| seen.fact.resets_at.is_none_or(|at| at > observed_at));
        if fact.resets_at.is_some_and(|at| at <= observed_at) {
            windows.remove(&fact.window);
            return;
        }
        windows.insert(
            fact.window.clone(),
            Seen {
                fact,
                source,
                observed_at,
            },
        );
        if windows.len() > 8 {
            let oldest = windows
                .iter()
                .min_by(|(left_key, left), (right_key, right)| {
                    left.observed_at
                        .cmp(&right.observed_at)
                        .then_with(|| left_key.cmp(right_key))
                })
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                windows.remove(&oldest);
            }
        }
    }

    /// O(K log p + F), K octets de clé, p types, F au plus8 fenêtres ; espace O(1).
    /// Lecture160 : chaque fenêtre est filtrée avant les deux maxima indépendants.
    /// `past_since` est le début du hold résolu ; aucun fait ni état n'est créé.
    pub(crate) fn attested_exhausted_since(
        &self,
        agent_type: &str,
        since: i64,
        now: i64,
        past_since: Option<i64>,
    ) -> Option<(i64, i64)> {
        let windows = self.by_agent_type.get(agent_type)?;
        let mut resets_at: Option<i64> = None;
        let mut observed_at: Option<i64> = None;
        for seen in windows.values() {
            if seen.observed_at <= since
                || past_since.is_some_and(|at| seen.observed_at > at.saturating_add(5))
            {
                continue;
            }
            let Some(reset) = seen.fact.resets_at else {
                continue;
            };
            if reset <= now && !past_since.is_some_and(|at| seen.observed_at >= at) {
                continue;
            }
            // Prédicat157 identique : petit doublon assumé pour ne pas extraire
            // un helper ni modifier quota(), dont l'admission reste inchangée.
            let exhausted = (seen.source == RateLimitSource::ClaudeStreamJson
                && seen.fact.status == "rejected")
                || seen.fact.used_percent.is_some_and(|percent| percent >= 100);
            if !exhausted {
                continue;
            }
            resets_at = resets_at.max(Some(reset));
            observed_at = observed_at.max(Some(seen.observed_at));
        }
        resets_at.zip(observed_at)
    }

    /// O(log p+B), au plus8 fenêtres et B octets de projection ; lecture seule.
    /// Le statut Codex est global ; seul un retour futur atteste une fenêtre épuisée.
    pub(crate) fn quota(&self, agent_type: &str, now: i64) -> Option<Quota> {
        let known = self.by_agent_type.get(agent_type)?;
        let mut windows = Vec::new();
        let mut exhausted = false;
        let mut limit_reached = false;
        let mut resets_at = None;
        let mut observed_at = None;
        for seen in known.values() {
            let fact = &seen.fact;
            if fact.resets_at.is_some_and(|at| at <= now) {
                continue;
            }
            let window_exhausted = fact.resets_at.is_some_and(|at| at > now)
                && ((seen.source == RateLimitSource::ClaudeStreamJson
                    && fact.status == "rejected")
                    || fact.used_percent.is_some_and(|percent| percent >= 100));
            exhausted |= window_exhausted;
            limit_reached |= window_exhausted
                || (seen.source == RateLimitSource::CodexAppServer && fact.status != "available");
            if window_exhausted {
                resets_at = resets_at.max(fact.resets_at);
            }
            observed_at = observed_at.max(Some(seen.observed_at));
            windows.push(serde_json::json!({
                "window": fact.window,
                "used_percent": fact.used_percent,
                "resets_at": fact.resets_at,
                "exhausted": window_exhausted,
            }));
        }
        Some(Quota {
            windows,
            exhausted,
            limit_reached,
            resets_at,
            observed_at: observed_at?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native157_limits_regle_epuisement() {
        use RateLimitSource::{ClaudeStatusLine, ClaudeStreamJson, CodexAppServer};
        for (source, status, percent, reset, expected_exhausted, expected_reached) in [
            (ClaudeStreamJson, "rejected", None, Some(500), true, true),
            (
                ClaudeStreamJson,
                "rejected",
                Some(50),
                Some(500),
                true,
                true,
            ),
            (
                ClaudeStreamJson,
                "allowed",
                Some(100),
                Some(500),
                true,
                true,
            ),
            (
                ClaudeStreamJson,
                "allowed_warning",
                Some(99),
                Some(500),
                false,
                false,
            ),
            (ClaudeStreamJson, "rejected", Some(100), None, false, false),
            (CodexAppServer, "rejected", Some(50), Some(500), false, true),
            (
                CodexAppServer,
                "available",
                Some(100),
                Some(500),
                true,
                true,
            ),
            (
                CodexAppServer,
                "limit_reached",
                None,
                Some(500),
                false,
                true,
            ),
            (
                CodexAppServer,
                "limit_reached",
                Some(100),
                None,
                false,
                true,
            ),
            (CodexAppServer, "available", Some(50), None, false, false),
            (
                ClaudeStatusLine,
                "unknown",
                Some(50),
                Some(500),
                false,
                false,
            ),
            (ClaudeStatusLine, "unknown", None, Some(500), false, false),
            (
                ClaudeStatusLine,
                "unknown",
                Some(100),
                Some(500),
                true,
                true,
            ),
            (ClaudeStatusLine, "unknown", Some(100), None, false, false),
        ] {
            let mut limits = ProviderLimits::default();
            limits.record(
                "registry-provider",
                RateLimitFact {
                    window: "five_hour".into(),
                    status: status.into(),
                    resets_at: reset,
                    used_percent: percent,
                },
                source,
                90,
            );
            let quota = limits.quota("registry-provider", 100).expect("fait connu");
            assert_eq!(
                quota.exhausted, expected_exhausted,
                "{source}/{status}/{percent:?}/{reset:?}"
            );
            assert_eq!(quota.limit_reached, expected_reached);
            assert_eq!(
                quota.resets_at,
                if expected_exhausted { reset } else { None }
            );
            assert_eq!(quota.observed_at, 90);
            assert_eq!(
                quota.windows,
                vec![serde_json::json!({
                    "window":"five_hour", "used_percent":percent,
                    "resets_at":reset, "exhausted":expected_exhausted,
                })]
            );
            let seen = &limits.by_agent_type["registry-provider"]["five_hour"];
            assert_eq!(seen.source, source);
        }
    }

    #[test]
    fn native157_limits_statut_codex_global() {
        let mut limits = ProviderLimits::default();
        for (window, percent, reset, observed) in [
            ("primary/300m", 100, 500, 90),
            ("secondary/10080m", 61, 600_000, 91),
        ] {
            limits.record(
                "codex-pro",
                RateLimitFact {
                    window: window.into(),
                    status: "limit_reached".into(),
                    resets_at: Some(reset),
                    used_percent: Some(percent),
                },
                RateLimitSource::CodexAppServer,
                observed,
            );
        }
        let quota = limits.quota("codex-pro", 100).expect("deux fenêtres");
        assert!(quota.limit_reached);
        assert!(quota.exhausted);
        assert_eq!(quota.resets_at, Some(500));
        assert_eq!(quota.observed_at, 91);
        assert_eq!(quota.windows[0]["exhausted"], true);
        assert_eq!(quota.windows[1]["exhausted"], false);
        limits.record(
            "codex-pro",
            RateLimitFact {
                window: "primary/300m".into(),
                status: "limit_reached".into(),
                resets_at: Some(500),
                used_percent: Some(99),
            },
            RateLimitSource::CodexAppServer,
            92,
        );
        let quota = limits.quota("codex-pro", 100).expect("statut global connu");
        assert!(quota.limit_reached);
        assert!(!quota.exhausted);
        assert_eq!(quota.resets_at, None);
        assert_eq!(quota.windows.len(), 2);
    }

    #[test]
    fn native157_limits_fenetre_perimee() {
        let mut limits = ProviderLimits::default();
        limits.record(
            "claude",
            RateLimitFact {
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(100),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            90,
        );
        assert!(
            limits.by_agent_type.contains_key("claude"),
            "record mémorise la fenêtre avant la lecture"
        );
        assert!(
            limits.quota("claude", 100).is_none(),
            "retour exact atteint"
        );
        assert!(
            limits.by_agent_type["claude"].contains_key("five_hour"),
            "lecture sans purge"
        );
        limits.record(
            "claude",
            RateLimitFact {
                window: "unknown".into(),
                status: "unknown".into(),
                resets_at: None,
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            100,
        );
        assert_eq!(limits.by_agent_type["claude"].len(), 1);
        assert!(!limits.by_agent_type["claude"].contains_key("five_hour"));
        limits.record(
            "claude",
            RateLimitFact {
                window: "already-past".into(),
                status: "rejected".into(),
                resets_at: Some(100),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            100,
        );
        assert_eq!(limits.by_agent_type["claude"].len(), 1);
        let quota = limits
            .quota("claude", 100)
            .expect("fait sans heure conservé");
        assert!(!quota.exhausted);
        assert_eq!(quota.resets_at, None);
        // La dernière observation passée retire aussi l'ancien fait de même fenêtre.
        limits.record(
            "claude",
            RateLimitFact {
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(500),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            100,
        );
        limits.record(
            "claude",
            RateLimitFact {
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(95),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            101,
        );
        assert!(!limits.by_agent_type["claude"].contains_key("five_hour"));
        assert!(
            !limits
                .quota("claude", 101)
                .expect("autre fenêtre conservée")
                .exhausted
        );
    }

    #[test]
    fn native157_limits_fenetres_independantes() {
        let mut limits = ProviderLimits::default();
        for (window, reset) in [("five_hour", 500), ("seven_day", 6000)] {
            limits.record(
                "claude",
                RateLimitFact {
                    window: window.into(),
                    status: "rejected".into(),
                    resets_at: Some(reset),
                    used_percent: Some(100),
                },
                RateLimitSource::ClaudeStreamJson,
                90,
            );
        }
        let quota = limits.quota("claude", 100).expect("fenêtres indépendantes");
        assert_eq!(quota.windows.len(), 2);
        assert_eq!(
            quota.resets_at,
            Some(6000),
            "retour le plus tardif des épuisées"
        );
        limits.record(
            "claude",
            RateLimitFact {
                window: "seven_day".into(),
                status: "unknown".into(),
                resets_at: Some(6000),
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            101,
        );
        let quota = limits.quota("claude", 102).expect("aucune fenêtre effacée");
        assert_eq!(quota.windows.len(), 2);
        assert_eq!(
            quota.resets_at,
            Some(500),
            "fenêtre non épuisée exclue du retour"
        );
        assert_eq!(quota.observed_at, 101);
        assert_eq!(
            limits.by_agent_type["claude"]["seven_day"].source,
            RateLimitSource::ClaudeStatusLine
        );
        assert_eq!(
            limits.by_agent_type["claude"]["five_hour"]
                .fact
                .used_percent,
            Some(100)
        );
    }

    #[test]
    fn native157_limits_cle_agent_type() {
        let mut limits = ProviderLimits::default();
        assert!(limits.quota("unknown-provider", 100).is_none());
        for (agent_type, percent, source) in [
            ("claude-child", 100, RateLimitSource::ClaudeStreamJson),
            ("glm", 50, RateLimitSource::ClaudeStatusLine),
            ("codex-pro", 100, RateLimitSource::CodexAppServer),
        ] {
            limits.record(
                agent_type,
                RateLimitFact {
                    window: "shared-window".into(),
                    status: "unknown".into(),
                    resets_at: Some(500),
                    used_percent: Some(percent),
                },
                source,
                90,
            );
        }
        assert!(limits.quota("claude-child", 100).unwrap().exhausted);
        assert!(!limits.quota("glm", 100).unwrap().exhausted);
        assert!(limits.quota("codex-pro", 100).unwrap().exhausted);
        assert!(limits.quota("claude-stream-json", 100).is_none());
        assert!(limits.quota("claude-statusline", 100).is_none());
        assert!(limits.quota("codex-app-server", 100).is_none());
        assert_eq!(limits.by_agent_type.len(), 3);
    }

    #[test]
    fn native157_limits_borne_huit_fenetres() {
        let mut limits = ProviderLimits::default();
        for index in 0..8 {
            limits.record(
                "claude",
                RateLimitFact {
                    window: format!("w{index}"),
                    status: "unknown".into(),
                    resets_at: None,
                    used_percent: Some(50),
                },
                RateLimitSource::ClaudeStatusLine,
                90 + index,
            );
        }
        limits.record(
            "claude",
            RateLimitFact {
                window: "w0".into(),
                status: "unknown".into(),
                resets_at: None,
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            100,
        );
        assert_eq!(limits.by_agent_type["claude"].len(), 8);
        limits.record(
            "claude",
            RateLimitFact {
                window: "w8".into(),
                status: "unknown".into(),
                resets_at: None,
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            101,
        );
        let windows = &limits.by_agent_type["claude"];
        assert_eq!(windows.len(), 8);
        assert!(
            !windows.contains_key("w1"),
            "éviction du plus ancien après l'upsert"
        );
        assert!(windows.contains_key("w0") && windows.contains_key("w8"));

        let mut tied = ProviderLimits::default();
        for window in ["c", "b", "a", "d", "e", "f", "g", "h", "z"] {
            tied.record(
                "claude",
                RateLimitFact {
                    window: window.into(),
                    status: "unknown".into(),
                    resets_at: None,
                    used_percent: Some(50),
                },
                RateLimitSource::ClaudeStatusLine,
                90,
            );
        }
        let windows = &tied.by_agent_type["claude"];
        assert_eq!(windows.len(), 8);
        assert!(
            !windows.contains_key("a"),
            "égalité départagée par clé lexicale"
        );
        assert!(windows.contains_key("z"));

        let mut purged = ProviderLimits::default();
        for index in 0..8 {
            purged.record(
                "claude",
                RateLimitFact {
                    window: format!("w{index}"),
                    status: "unknown".into(),
                    resets_at: Some(if matches!(index, 1 | 2) { 95 } else { 500 }),
                    used_percent: Some(50),
                },
                RateLimitSource::ClaudeStatusLine,
                90,
            );
        }
        purged.record(
            "claude",
            RateLimitFact {
                window: "w8".into(),
                status: "unknown".into(),
                resets_at: Some(500),
                used_percent: Some(50),
            },
            RateLimitSource::ClaudeStatusLine,
            100,
        );
        let windows = &purged.by_agent_type["claude"];
        assert_eq!(windows.len(), 7, "purge avant éviction à la neuvième clé");
        assert!(windows.contains_key("w0"));
        assert!(!windows.contains_key("w1") && !windows.contains_key("w2"));
    }
    #[test]
    fn native160_limits_fait_ancien_ignore() {
        for (observed, expected) in [(89, None), (90, None), (91, Some((500, 91)))] {
            let mut limits = ProviderLimits::default();
            limits.record(
                "claude-child",
                RateLimitFact {
                    window: "five_hour".into(),
                    status: "rejected".into(),
                    resets_at: Some(500),
                    used_percent: None,
                },
                RateLimitSource::ClaudeStreamJson,
                observed,
            );
            assert_eq!(
                limits.attested_exhausted_since("claude-child", 90, 100, None),
                expected
            );
            assert!(
                limits.quota("claude-child", 100).unwrap().exhausted,
                "l'admission157 garde rejected sans pourcentage"
            );
        }
    }

    #[test]
    fn native160_limits_codex_global_sans_fenetre_100_ignore() {
        let mut limits = ProviderLimits::default();
        limits.record(
            "codex-pro",
            RateLimitFact {
                window: "secondary".into(),
                status: "limit_reached".into(),
                resets_at: Some(600_000),
                used_percent: None,
            },
            RateLimitSource::CodexAppServer,
            99,
        );
        assert_eq!(
            limits.attested_exhausted_since("codex-pro", 80, 100, None),
            None
        );
        let quota = limits.quota("codex-pro", 100).unwrap();
        assert!(quota.limit_reached && !quota.exhausted);
        limits.record(
            "codex-pro",
            RateLimitFact {
                window: "primary".into(),
                status: "limit_reached".into(),
                resets_at: Some(500),
                used_percent: Some(100),
            },
            RateLimitSource::CodexAppServer,
            100,
        );
        assert_eq!(
            limits.attested_exhausted_since("codex-pro", 80, 101, None),
            Some((500, 100)),
            "la fenêtre globale non attestée n'allonge ni reset ni observation"
        );
    }

    #[test]
    fn native160_limits_ancienne_observation_perimee_ignore() {
        let mut limits = ProviderLimits::default();
        limits.record(
            "claude",
            RateLimitFact {
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(99),
                used_percent: Some(100),
            },
            RateLimitSource::ClaudeStreamJson,
            95,
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 104, None),
            None
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 104, Some(100)),
            None,
            "observation antérieure au hold : aucun Past inventé"
        );
        assert!(limits.quota("claude", 104).is_none());
        assert!(
            limits.by_agent_type["claude"].contains_key("five_hour"),
            "lecture sans purge"
        );
    }

    #[test]
    fn native160_limits_hold_fait_expire_admis() {
        let mut limits = ProviderLimits::default();
        limits.record(
            "claude",
            RateLimitFact {
                window: "five_hour".into(),
                status: "rejected".into(),
                resets_at: Some(103),
                used_percent: None,
            },
            RateLimitSource::ClaudeStreamJson,
            102,
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 104, None),
            None
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 104, Some(100)),
            Some((103, 102))
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 110, Some(100)),
            Some((103, 102)),
            "un tick retardé ne change pas l'instant d'observation admissible"
        );
        assert!(
            limits.quota("claude", 104).is_none(),
            "admission157 inchangée malgré Past160"
        );
    }

    #[test]
    fn native160_limits_hold_filtre_chaque_fenetre_avant_agregation() {
        let mut limits = ProviderLimits::default();
        for (window, reset, observed) in [("admissible", 130, 102), ("tardive", 200, 106)] {
            limits.record(
                "claude",
                RateLimitFact {
                    window: window.into(),
                    status: "unknown".into(),
                    resets_at: Some(reset),
                    used_percent: Some(100),
                },
                RateLimitSource::ClaudeStatusLine,
                observed,
            );
        }
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 110, Some(100)),
            Some((130, 102)),
            "la fenêtre tardive est écartée avant reset_max et observed_max"
        );
        for (window, reset, observed) in
            [("retour_maximal", 400, 101), ("borne_inclusive", 150, 105)]
        {
            limits.record(
                "claude",
                RateLimitFact {
                    window: window.into(),
                    status: "unknown".into(),
                    resets_at: Some(reset),
                    used_percent: Some(100),
                },
                RateLimitSource::ClaudeStatusLine,
                observed,
            );
        }
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 110, Some(100)),
            Some((400, 105)),
            "maxima indépendants sur les seules observations jusqu'à since_hold+5 inclus"
        );
        assert_eq!(
            limits.attested_exhausted_since("claude", 90, 110, None),
            Some((400, 106)),
            "hors hold, aucune borne de cinq secondes ajoutée à157"
        );
    }
}
