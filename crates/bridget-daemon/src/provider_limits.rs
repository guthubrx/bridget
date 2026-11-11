//! Mémoire157 volatile par profil, avec faits de compte liés à leur instance.
use bridget_transport::protocol::{RateLimitFact, RateLimitSource};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Sans retour attesté, réévaluation possible après cinq minutes. Ce délai
// évite les relances immédiates et ne devient jamais une date de rétablissement.
const CLAUDE_UNKNOWN_REFUSAL_RETRY_SECS: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Seen {
    fact: RateLimitFact,
    source: RateLimitSource,
    observed_at: i64,
}

#[derive(Debug, Default)]
pub(crate) struct ProviderLimits {
    by_agent_type: BTreeMap<String, BTreeMap<String, Seen>>,
    account_refusals244: BTreeMap<String, BTreeMap<String, Seen>>,
    // Alias conservés aussi si le déplacement SQLite échoue : une réponse
    // positive ultérieure doit solder le fait sous ses deux clés.
    account_aliases245: BTreeMap<String, String>,
    // Au plus 4096 instances ; aucune capacité payante ne traverse cette clé.
    by_instance: BTreeMap<(String, String), ProviderLimits>,
    routing_bindings: BTreeMap<String, RoutingQuotaBinding>,
    codex_credits: BTreeMap<String, (Option<bridget_transport::protocol::CodexCredits>, i64)>,
}

#[derive(Debug, Clone)]
struct RoutingQuotaBinding {
    agent_type: String,
    instance: String,
    definition_digest: String,
    connection_scope: String,
    coordinator_epoch: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct Quota {
    pub windows: Vec<serde_json::Value>,
    pub exhausted: bool,
    pub launch_blocked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits: Option<bridget_transport::protocol::CodexCredits>,
    pub limit_reached: bool,
    pub resets_at: Option<i64>,
    pub observed_at: i64,
}

impl ProviderLimits {
    /// Cache de disponibilité uniquement : aucun droit ni crédit. Les refus
    /// survivent au redémarrage ; un horaire inconnu reste inconnu.
    pub(crate) fn restore_routing244(db: &rusqlite::Connection, now: i64) -> Self {
        let mut limits = Self::default();
        let result = (|| -> rusqlite::Result<()> {
            db.execute_batch("CREATE TABLE IF NOT EXISTS provider_refusals244 (scope TEXT NOT NULL, window TEXT NOT NULL, fact TEXT NOT NULL, expires_at INTEGER NOT NULL, PRIMARY KEY(scope,window))")?;
            db.execute(
                "DELETE FROM provider_refusals244 WHERE expires_at <= ?1",
                [now],
            )?;
            let mut stmt =
                db.prepare("SELECT scope, fact FROM provider_refusals244 WHERE expires_at > ?1")?;
            let rows = stmt.query_map([now], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
            for row in rows {
                let (scope, body) = row?;
                if scope.len() != 64 || body.len() > 4096 {
                    continue;
                }
                if let Ok(seen) = serde_json::from_str::<Seen>(&body)
                    && Self::shared_claude_window244(&seen.fact.window)
                    && seen.source == RateLimitSource::ClaudeStreamJson
                    && seen.fact.status == "rejected"
                    && seen.observed_at > 0
                    && seen.observed_at <= now
                    && Self::refusal_until244(&seen) > now
                {
                    limits
                        .account_refusals244
                        .entry(scope)
                        .or_default()
                        .insert(seen.fact.window.clone(), seen);
                }
            }
            Ok(())
        })();
        if result.is_err() {
            log::warn!("cache des refus fournisseur indisponible ; disponibilité inconnue");
        }
        limits
    }

    /// Changement de clé de cache uniquement : mêmes faits et mêmes dates.
    /// Le déplacement atomique empêche de ressusciter un ancien refus après
    /// une réponse positive reçue ensuite sous la clé canonique.
    pub(crate) fn migrate_account_scope245(
        &mut self,
        db: &rusqlite::Connection,
        old: &str,
        new: &str,
    ) -> rusqlite::Result<()> {
        if old == new {
            return Ok(());
        }
        let Some(previous) = self.account_refusals244.get(old).cloned() else {
            return Ok(());
        };
        self.account_aliases245.insert(old.into(), new.into());
        let mut merged = self
            .account_refusals244
            .get(new)
            .cloned()
            .unwrap_or_default();
        for (window, seen) in previous {
            if merged
                .get(&window)
                .is_none_or(|current| current.observed_at < seen.observed_at)
            {
                merged.insert(window, seen);
            }
        }
        // La mémoire garde le fait même si le cache disque est indisponible.
        self.account_refusals244.insert(new.into(), merged.clone());
        let tx = db.unchecked_transaction()?;
        for (window, seen) in merged {
            let body = serde_json::to_string(&seen).expect("fait de quota sérialisable");
            tx.execute("INSERT INTO provider_refusals244(scope,window,fact,expires_at) VALUES(?1,?2,?3,?4) ON CONFLICT(scope,window) DO UPDATE SET fact=excluded.fact,expires_at=excluded.expires_at",
                rusqlite::params![new,window,body,Self::refusal_until244(&seen)])?;
        }
        tx.execute("DELETE FROM provider_refusals244 WHERE scope=?1", [old])?;
        tx.commit()?;
        self.account_refusals244.remove(old);
        Ok(())
    }

    fn shared_claude_window244(window: &str) -> bool {
        // Les fenêtres spécifiques à Opus/Sonnet restent propres à la connexion.
        matches!(window, "unknown" | "five_hour" | "seven_day")
    }
    fn refusal_until244(seen: &Seen) -> i64 {
        seen.fact.resets_at.unwrap_or_else(|| {
            seen.observed_at
                .saturating_add(CLAUDE_UNKNOWN_REFUSAL_RETRY_SECS)
        })
    }
    pub(crate) fn record_account244(
        &mut self,
        db: &rusqlite::Connection,
        scope: &str,
        fact: &RateLimitFact,
        source: RateLimitSource,
        now: i64,
    ) {
        if source != RateLimitSource::ClaudeStreamJson
            || !Self::shared_claude_window244(&fact.window)
        {
            return;
        }
        let windows = self.account_refusals244.entry(scope.into()).or_default();
        if fact.status == "rejected" && fact.resets_at.is_none_or(|at| at > now) {
            let seen = Seen {
                fact: fact.clone(),
                source,
                observed_at: now,
            };
            if let Ok(body) = serde_json::to_string(&seen)
                && db.execute("INSERT INTO provider_refusals244(scope,window,fact,expires_at) VALUES(?1,?2,?3,?4) ON CONFLICT(scope,window) DO UPDATE SET fact=excluded.fact, expires_at=excluded.expires_at",
                    rusqlite::params![scope, fact.window, body, Self::refusal_until244(&seen)]).is_err() {
                    log::warn!("refus fournisseur conservé en mémoire ; cache durable indisponible");
                }
            windows.insert(fact.window.clone(), seen);
        } else if fact.status == "allowed" || fact.status == "available" {
            let aliases: Vec<_> = self
                .account_aliases245
                .iter()
                .filter(|(_, canonical)| canonical.as_str() == scope)
                .map(|(old, _)| old.clone())
                .collect();
            let cleared = (|| -> rusqlite::Result<()> {
                let tx = db.unchecked_transaction()?;
                for key in std::iter::once(scope).chain(aliases.iter().map(String::as_str)) {
                    tx.execute(
                        "DELETE FROM provider_refusals244 WHERE scope=?1 AND window=?2",
                        rusqlite::params![key, fact.window],
                    )?;
                }
                tx.commit()
            })();
            if cleared.is_ok() {
                windows.remove(&fact.window);
                for alias in aliases {
                    if let Some(old) = self.account_refusals244.get_mut(&alias) {
                        old.remove(&fact.window);
                    }
                }
            } else {
                log::warn!(
                    "réponse fournisseur positive observée ; effacement durable du refus indisponible"
                );
            }
        }
        if let Some(windows) = self.account_refusals244.get_mut(scope) {
            windows.retain(|_, seen| Self::refusal_until244(seen) > now);
        }
        let _ = db.execute(
            "DELETE FROM provider_refusals244 WHERE expires_at<=?1",
            [now],
        );
    }

    pub(crate) fn account_quota244(
        &self,
        quota: Option<Quota>,
        scope: Option<&str>,
        now: i64,
    ) -> Option<Quota> {
        let seen = scope
            .and_then(|scope| self.account_refusals244.get(scope))
            .into_iter()
            .flat_map(|windows| windows.values())
            .filter(|seen| seen.observed_at <= now && Self::refusal_until244(seen) > now)
            .max_by_key(|seen| seen.observed_at);
        let Some(seen) = seen else {
            return quota;
        };
        let mut quota = quota.unwrap_or(Quota {
            windows: Vec::new(),
            exhausted: false,
            launch_blocked: false,
            credits: None,
            limit_reached: false,
            resets_at: None,
            observed_at: seen.observed_at,
        });
        quota.launch_blocked = true;
        quota.limit_reached = true;
        quota.observed_at = quota.observed_at.max(seen.observed_at);
        quota.resets_at = quota.resets_at.max(seen.fact.resets_at);
        quota.windows.push(serde_json::json!({"window":seen.fact.window,"resets_at":seen.fact.resets_at,
            "exhausted":true,"scope":"same_claude_account","retry_after":Self::refusal_until244(seen)}));
        Some(quota)
    }

    /// O(log p+B), p types du registre, B chaînes copiées ; au plus9 fenêtres
    /// parcourues : purge avant upsert, puis éviction du minimum (instant, clé).
    pub(crate) fn record(
        &mut self,
        agent_type: &str,
        fact: RateLimitFact,
        source: RateLimitSource,
        observed_at: i64,
    ) {
        if source == RateLimitSource::CodexAppServer {
            self.codex_credits
                .insert(agent_type.to_owned(), (fact.credits.clone(), observed_at));
            if fact.window == "account/credits" {
                return;
            }
        }
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

    /// O(log I + F), I<=4096, F<=8 ; éviction O(I) seulement à saturation.
    pub(crate) fn record_scoped(
        &mut self,
        agent_type: &str,
        instance: &str,
        fact: RateLimitFact,
        source: RateLimitSource,
        observed_at: i64,
    ) {
        self.record(agent_type, fact.clone(), source, observed_at);
        let key = (agent_type.to_owned(), instance.to_owned());
        if !self.by_instance.contains_key(&key) && self.by_instance.len() >= 4096 {
            let oldest = self
                .by_instance
                .iter()
                .min_by_key(|(_, limits)| {
                    limits
                        .by_agent_type
                        .values()
                        .flat_map(|w| w.values())
                        .map(|seen| seen.observed_at)
                        .chain(limits.codex_credits.values().map(|(_, at)| *at))
                        .max()
                        .unwrap_or(0)
                })
                .map(|(key, _)| key.clone());
            if let Some(key) = oldest {
                self.by_instance.remove(&key);
            }
        }
        self.by_instance
            .entry(key)
            .or_default()
            .record(agent_type, fact, source, observed_at);
    }

    /// Lie une observation existante à une seule connexion attestée.
    /// O(log C), C<=128 ; aucune fusion par type ni entre comptes.
    pub(crate) fn bind_routing_instance(
        &mut self,
        connection_id: &str,
        agent_type: &str,
        instance: &str,
        definition_digest: &str,
        coordinator_epoch: i64,
        connection_scope: &str,
    ) {
        if self.routing_bindings.len() >= 128 && !self.routing_bindings.contains_key(connection_id)
        {
            return;
        }
        self.routing_bindings.insert(
            connection_id.to_owned(),
            RoutingQuotaBinding {
                agent_type: agent_type.to_owned(),
                instance: instance.to_owned(),
                definition_digest: definition_digest.to_owned(),
                connection_scope: connection_scope.to_owned(),
                coordinator_epoch,
            },
        );
    }

    /// O(log C + log I + F), F<=8. Un refus reste jusqu'au retour attesté
    /// ou à une réponse acceptée ; sans retour, réévaluation après cinq minutes.
    /// Les faits positifs gardent définition, présence et TTL ; aucune date inventée.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn routing_quota(
        &self,
        connection_id: &str,
        definition_digest: &str,
        coordinator_epoch: i64,
        connection_scope: &str,
        live_instances: &std::collections::HashSet<String>,
        now: i64,
        max_age_secs: u64,
    ) -> Option<Quota> {
        let binding = self.routing_bindings.get(connection_id)?;
        if binding.coordinator_epoch != coordinator_epoch
            || connection_scope.is_empty()
            || binding.connection_scope != connection_scope
            || !(1..=300).contains(&max_age_secs)
        {
            return None;
        }
        let instance_live = live_instances.contains(&binding.instance);
        let same_definition = binding.definition_digest == definition_digest;
        let scoped = self
            .by_instance
            .get(&(binding.agent_type.clone(), binding.instance.clone()))?;
        let fresh = |at: i64| at > 0 && at <= now && now.saturating_sub(at) <= max_age_secs as i64;
        let mut filtered = ProviderLimits::default();
        for seen in scoped
            .by_agent_type
            .get(&binding.agent_type)
            .into_iter()
            .flat_map(|v| v.values())
        {
            let rejection = seen.source == RateLimitSource::ClaudeStreamJson
                && seen.fact.status == "rejected"
                && seen.fact.resets_at.map_or_else(
                    || now.saturating_sub(seen.observed_at) <= CLAUDE_UNKNOWN_REFUSAL_RETRY_SECS,
                    |at| at > now,
                );
            if seen.observed_at > 0
                && seen.observed_at <= now
                && (rejection || (same_definition && instance_live && fresh(seen.observed_at)))
            {
                filtered.record(
                    &binding.agent_type,
                    seen.fact.clone(),
                    seen.source,
                    seen.observed_at,
                );
            }
        }
        filtered.codex_credits.remove(&binding.agent_type);
        if let Some(credits) = scoped
            .codex_credits
            .get(&binding.agent_type)
            .filter(|(_, at)| same_definition && instance_live && fresh(*at))
        {
            filtered
                .codex_credits
                .insert(binding.agent_type.clone(), credits.clone());
        }
        let claude_rejected = filtered
            .by_agent_type
            .get(&binding.agent_type)
            .into_iter()
            .flat_map(|windows| windows.values())
            .any(|seen| {
                seen.source == RateLimitSource::ClaudeStreamJson
                    && seen.fact.status == "rejected"
                    && seen.fact.resets_at.is_none_or(|at| at > now)
            });
        if !instance_live && !claude_rejected {
            return None;
        }
        let mut quota = filtered.quota_with_credits(&binding.agent_type, now, true)?;
        quota.limit_reached |= claude_rejected;
        quota.launch_blocked |= claude_rejected;
        Some(quota)
    }

    pub(crate) fn scoped_quota(&self, agent_type: &str, instance: &str, now: i64) -> Option<Quota> {
        self.by_instance
            .get(&(agent_type.to_owned(), instance.to_owned()))?
            .quota_with_credits(agent_type, now, true)
    }

    pub(crate) fn scoped_exhausted_since(
        &self,
        agent_type: &str,
        instance: &str,
        since: i64,
        now: i64,
        past_since: Option<i64>,
    ) -> Option<(i64, i64)> {
        self.by_instance
            .get(&(agent_type.to_owned(), instance.to_owned()))?
            .attested_exhausted_since(agent_type, since, now, past_since)
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
            if reset <= now && past_since.is_none_or(|at| seen.observed_at < at) {
                continue;
            }
            // La fenêtre reste épuisée ; seuls les crédits attestés dans cette
            // même instance retirent le blocage effectif du hold160.
            let exhausted = (seen.source == RateLimitSource::ClaudeStreamJson
                && seen.fact.status == "rejected")
                || seen.fact.used_percent.is_some_and(|percent| percent >= 100);
            if !exhausted
                || (seen.source == RateLimitSource::CodexAppServer
                    && self
                        .codex_credits
                        .get(agent_type)
                        .and_then(|(credits, _)| credits.as_ref())
                        .is_some_and(|credits| credits.usable()))
            {
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
        self.quota_with_credits(agent_type, now, false)
    }

    fn quota_with_credits(&self, agent_type: &str, now: i64, scoped: bool) -> Option<Quota> {
        let known = self.by_agent_type.get(agent_type);
        let current_credits = self.codex_credits.get(agent_type).filter(|_| scoped);
        let mut windows = Vec::new();
        let mut exhausted = false;
        let mut launch_blocked = current_credits
            .and_then(|(credits, _)| credits.as_ref())
            .is_some_and(|credits| credits.spend_control_reached);
        let mut limit_reached = false;
        let mut resets_at = None;
        let mut observed_at = current_credits.map(|(_, at)| *at);
        for seen in known.into_iter().flat_map(|windows| windows.values()) {
            let fact = &seen.fact;
            if fact.resets_at.is_some_and(|at| at <= now) {
                continue;
            }
            let window_exhausted = fact.resets_at.is_some_and(|at| at > now)
                && ((seen.source == RateLimitSource::ClaudeStreamJson
                    && fact.status == "rejected")
                    || fact.used_percent.is_some_and(|percent| percent >= 100));
            exhausted |= window_exhausted;
            let credits = self
                .codex_credits
                .get(agent_type)
                .and_then(|(credits, _)| credits.as_ref())
                .filter(|_| scoped && seen.source == RateLimitSource::CodexAppServer);
            launch_blocked |= credits.is_some_and(|credit| credit.spend_control_reached)
                || (window_exhausted && !credits.is_some_and(|credit| credit.usable()));
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
            launch_blocked,
            credits: current_credits.and_then(|(credits, _)| credits.clone()),
            limit_reached,
            resets_at,
            observed_at: observed_at?,
        })
    }
}

#[cfg(test)]
#[path = "provider_limits_tests.rs"]
mod tests;
