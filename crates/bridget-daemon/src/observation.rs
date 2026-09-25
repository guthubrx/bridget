//! Faits locaux, filtres et risques de collision. Aucun workflow ni accès disque.
use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Component, Path};
use std::time::{Duration, Instant};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Subscription {
    id: String,
    owner: String,
    event: Kind,
    agent: Option<String>,
    file: Option<String>,
    once: bool,
    #[serde(skip, default = "Instant::now")]
    expires: Instant,
    expires_at: u64,
    #[serde(skip)]
    last_notification: Option<Instant>,
    #[serde(skip)]
    suppressed: u64,
    /// Repris d'un instantané au démarrage du daemon ; son propriétaire en est
    /// averti une fois à son retour (session 122).
    #[serde(skip)]
    restored: bool,
    #[serde(skip)]
    available: bool,
    #[serde(skip)]
    source_count: usize,
    #[serde(skip)]
    interruption_announced: bool,
    #[serde(skip)]
    facts_lost: u64,
    #[serde(skip)]
    last_gap_notice: Option<Instant>,
    #[serde(skip)]
    observation_gaps: u64,
    /// Couverture dont l'abonné a été averti ; un écart n'est annoncé qu'une
    /// fois stable (session 119).
    #[serde(skip)]
    announced_count: usize,
    #[serde(skip)]
    changed_at: Option<Instant>,
    #[serde(skip)]
    diverged_since: Option<Instant>,
    #[serde(skip)]
    flips: u32,
}

/// Stabilité exigée avant d'annoncer un changement d'état de source.
const SOURCE_NOTICE_GRACE: Duration = Duration::from_secs(30);
/// Borne d'attente : une source qui clignote reste annoncée, au plus une fois
/// par période.
const SOURCE_NOTICE_MAX_DELAY: Duration = Duration::from_secs(300);

#[derive(Clone, Debug)]
pub(crate) struct Fact {
    pub event: Kind,
    pub agent: String,
    pub host: String,
    pub file: Option<String>,
    pub other_agent: Option<String>,
}

pub(crate) struct Notification {
    pub owner: String,
    pub body: String,
}

#[derive(Clone)]
struct RecentWrite {
    agent: String,
    at: Instant,
    warned: Option<(String, String, Instant)>,
}

#[derive(Default, Clone)]
pub(crate) struct Observations {
    pub subscriptions: HashMap<String, Subscription>,
    writes: HashMap<(String, String), RecentWrite>,
    pub evicted_writes: u64,
    sources: HashMap<String, (String, Vec<Kind>)>,
    pub revision: u64,
    pub facts_lost: u64,
    pub observation_gaps: u64,
}

impl Observations {
    /// O(S*C), S<=128 abonnements, C sources primaires vivantes.
    /// Lacune signalée (possible si dropped=0), jamais une fin de tour ; once conservé.
    pub fn report_gap(&mut self, agent: &str, dropped: u64, now: Instant) -> Vec<Notification> {
        self.facts_lost = self.facts_lost.saturating_add(dropped);
        self.observation_gaps = self.observation_gaps.saturating_add(1);
        let mut notices = vec![];
        for sub in self.subscriptions.values_mut().filter(|s| s.expires > now) {
            if sub.agent.as_ref().is_some_and(|wanted| wanted != agent)
                || !self
                    .sources
                    .values()
                    .any(|(source, events)| source == agent && Self::compatible(events, sub.event))
            {
                continue;
            }
            sub.facts_lost = sub.facts_lost.saturating_add(dropped);
            sub.observation_gaps = sub.observation_gaps.saturating_add(1);
            if sub.last_gap_notice.is_some_and(|last| {
                now.saturating_duration_since(last) < Duration::from_millis(200)
            }) {
                sub.suppressed = sub.suppressed.saturating_add(1);
                continue;
            }
            sub.last_gap_notice = Some(now);
            notices.push(Notification { owner: sub.owner.clone(), body: format!("[Bridget observation] {}", json!({
                "subscription_id":sub.id,"event":"observation_gap","agent":agent,"facts_lost_total":sub.facts_lost,
                "observation_gaps":sub.observation_gaps,"dropped":if dropped > 0 {Some(dropped)} else {None},
                "notice":if dropped == 0 {"continuité d'observation non garantie ; quantité manquante inconnue ; surveillance poursuivie sans reconstruire la lacune"}
                    else {"faits perdus à la source ; surveillance poursuivie sans reconstruire la lacune ; types et chemins des faits perdus inconnus"}
            })) });
        }
        notices
    }

    pub fn snapshot(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut subscriptions: Vec<_> = self.subscriptions.values().collect();
        subscriptions.sort_by(|a, b| a.id.cmp(&b.id));
        serde_json::to_vec(&subscriptions)
    }

    pub fn restore(bytes: &[u8], now: Instant, wall: u64) -> Result<Self, std::io::Error> {
        let invalid = || std::io::Error::other("invalid observation subscription snapshot");
        if bytes.len() > 262144 {
            return Err(invalid());
        }
        let subs: Vec<Subscription> = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        if subs.len() > 128 {
            return Err(invalid());
        }
        let mut state = Self::default();
        for mut sub in subs {
            if sub.id.is_empty()
                || sub.id.len() > 128
                || sub.owner.is_empty()
                || sub.owner.len() > 128
                || sub.agent.as_ref().is_some_and(|s| {
                    s.is_empty() || s.len() > 128 || s.chars().any(char::is_control)
                })
                || sub.file.as_ref().is_some_and(|s| {
                    s.is_empty() || s.len() > 256 || s.chars().any(char::is_control)
                })
                || (sub.file.is_some()
                    && !matches!(sub.event, Kind::FileWritten | Kind::FileCollision))
                || sub.expires_at > wall.saturating_add(604800)
                || state.subscriptions.contains_key(&sub.id)
                || state
                    .subscriptions
                    .values()
                    .filter(|s| s.owner == sub.owner)
                    .count()
                    >= 16
            {
                return Err(invalid());
            }
            if sub.expires_at <= wall {
                continue;
            }
            sub.expires = now + Duration::from_secs(sub.expires_at - wall);
            // Session 122 : repris sans nouvel abonnement. Aucune source n'est
            // encore déclarée ; son retour rend l'abonnement actif sans avis,
            // son absence prolongée est annoncée par `due_source_notices`.
            sub.restored = true;
            sub.announced_count = 1;
            sub.changed_at = Some(now);
            sub.diverged_since = Some(now);
            state.subscriptions.insert(sub.id.clone(), sub);
        }
        Ok(state)
    }

    fn compatible(events: &[Kind], event: Kind) -> bool {
        events.contains(&if event == Kind::FileCollision {
            Kind::FileWritten
        } else {
            event
        })
    }

    pub fn accepts(&self, connection: &str, event: Kind) -> bool {
        self.sources
            .get(connection)
            .is_some_and(|(_, events)| events.contains(&event))
    }

    pub fn catalogue(&self) -> Value {
        let mut sources: Vec<_> = self
            .sources
            .iter()
            .map(|(connection, (agent, events))| {
                let mut supported = events.clone();
                if events.contains(&Kind::FileWritten) {
                    supported.push(Kind::FileCollision);
                }
                json!({"connection":connection,"agent":agent,"events":supported})
            })
            .collect();
        sources.sort_by(|a, b| a["connection"].as_str().cmp(&b["connection"].as_str()));
        json!(sources)
    }

    pub fn set_source(&mut self, connection: &str, agent: &str, events: Vec<Kind>, now: Instant) {
        self.sources
            .insert(connection.into(), (agent.into(), events));
        self.refresh_sources(now);
    }

    pub fn remove_source(&mut self, connection: &str, now: Instant) {
        self.sources.remove(connection);
        self.refresh_sources(now);
    }

    /// L'état consultable change aussitôt ; l'avis à l'abonné est différé
    /// (`due_source_notices`).
    fn refresh_sources(&mut self, now: Instant) {
        for sub in self.subscriptions.values_mut().filter(|s| s.expires > now) {
            let source_count = self
                .sources
                .values()
                .filter(|(agent, events)| {
                    sub.agent.as_ref().is_none_or(|wanted| wanted == agent)
                        && Self::compatible(events, sub.event)
                })
                .count();
            if source_count != sub.source_count {
                sub.available = source_count > 0;
                sub.source_count = source_count;
                sub.changed_at = Some(now);
                sub.diverged_since.get_or_insert(now);
                sub.flips = sub.flips.saturating_add(1);
            }
        }
    }

    /// Annonce un changement d'état de source resté stable SOURCE_NOTICE_GRACE ;
    /// un aller-retour bref suivi de calme n'est jamais annoncé ; une source
    /// qui ne se stabilise pas est signalée instable au plus une fois par
    /// SOURCE_NOTICE_MAX_DELAY. Session 119 : une source qui clignotait a
    /// réveillé son abonné environ 200 fois en trois heures. O(S), S <= 128.
    pub fn due_source_notices(&mut self, now: Instant) -> Vec<Notification> {
        self.subscriptions
            .values_mut()
            .filter(|s| s.expires > now)
            .filter_map(|sub| {
                let since = sub.diverged_since?;
                let stable = sub
                    .changed_at
                    .is_some_and(|at| now.saturating_duration_since(at) >= SOURCE_NOTICE_GRACE);
                let unstable = !stable && now.saturating_duration_since(since) >= SOURCE_NOTICE_MAX_DELAY;
                if !stable && !unstable {
                    return None;
                }
                let flips = std::mem::take(&mut sub.flips);
                sub.diverged_since = None;
                if stable && sub.source_count == sub.announced_count {
                    return None;
                }
                sub.announced_count = sub.source_count;
                let notice = if unstable {
                    "source instable : disponibilité changeante, faits possiblement manqués"
                } else if sub.available {
                    "couverture modifiée ; seules les sources déclarées sont observées, lacunes non rejouées"
                } else {
                    "surveillance interrompue : source indisponible"
                };
                Some(Notification { owner: sub.owner.clone(), body: format!("[Bridget observation] {}", json!({
                    "subscription_id":sub.id,"state":if sub.available {"active"} else {"source_unavailable"},
                    "sources_compatible":sub.source_count,"changes":flips,"notice":notice
                })) })
            })
            .collect()
    }

    pub fn owner_returned(&mut self, owner: &str, now: Instant) -> Vec<Notification> {
        self.subscriptions.values_mut().filter(|s| s.owner == owner && s.restored && !s.interruption_announced && s.expires > now)
            .map(|s| {
                s.interruption_announced = true;
                Notification { owner: owner.into(), body: format!("[Bridget observation] {}", json!({"subscription_id":s.id,"state":if s.available {"active"} else {"source_unavailable"},"notice":"daemon redémarré : abonnement repris automatiquement ; faits survenus pendant la coupure perdus, non rejoués"})) }
            }).collect()
    }

    /// O(S), liste triée O(S log S), S <= 128. Identité hors Request.
    pub fn request(&mut self, owner: &str, request: Request, now: Instant, wall: u64) -> Value {
        let before = self.subscriptions.len();
        self.subscriptions.retain(|_, s| s.expires > now);
        if before != self.subscriptions.len() {
            self.revision += 1;
        }
        match request {
            Request::Types {} => {
                let events: Vec<_> = [
                    (Kind::TurnEnded, "fin d'un tour, pas succès ou fin de mission"),
                    (Kind::PermissionRequired, "demande de permission observée, éventuellement déjà traitée"),
                    (Kind::FileWritten, "écriture structurée confirmée par une intégration"),
                    (Kind::FileCollision, "risque : deux auteurs, même hôte/chemin, 30 secondes"),
                ].into_iter().map(|(kind, meaning)| {
                    let mut sources: Vec<_> = self.sources.values().filter(|(_, events)| Self::compatible(events, kind)).map(|(agent, _)| agent).collect();
                    sources.sort();
                    sources.dedup();
                    json!({"event":kind,"meaning":meaning,"available":!sources.is_empty(),"sources":sources})
                }).collect();
                json!({"events":events})
            }
            Request::Sub {
                event,
                agent,
                file,
                once,
                ttl_secs,
            } => {
                let ttl = ttl_secs.unwrap_or(3600);
                if !(1..=604800).contains(&ttl)
                    || agent.as_ref().is_some_and(|a| {
                        a.is_empty() || a.len() > 128 || a.chars().any(char::is_control)
                    })
                    || file.as_ref().is_some_and(|f| {
                        f.is_empty() || f.len() > 256 || f.chars().any(char::is_control)
                    })
                    || (file.is_some() && !matches!(event, Kind::FileWritten | Kind::FileCollision))
                {
                    return json!({"status":"rejected","reason":"invalid_filter_or_ttl"});
                }
                if self.subscriptions.len() >= 128
                    || self
                        .subscriptions
                        .values()
                        .filter(|s| s.owner == owner)
                        .count()
                        >= 16
                {
                    return json!({"status":"rejected","reason":"subscription_limit"});
                }
                let source_count = self
                    .sources
                    .values()
                    .filter(|(source, events)| {
                        agent.as_ref().is_none_or(|a| a == source)
                            && Self::compatible(events, event)
                    })
                    .count();
                if source_count == 0 {
                    let unavailable = agent.as_ref().is_some_and(|wanted| {
                        self.sources
                            .values()
                            .any(|(source, events)| source == wanted && events.is_empty())
                    });
                    return json!({"status":"rejected","reason":if unavailable {"source_unavailable"} else {"no_compatible_source"}});
                }
                let id = uuid::Uuid::new_v4().to_string();
                let sub = Subscription {
                    id: id.clone(),
                    owner: owner.into(),
                    event,
                    agent,
                    file,
                    once,
                    expires: now + Duration::from_secs(ttl),
                    expires_at: wall.saturating_add(ttl),
                    last_notification: None,
                    suppressed: 0,
                    restored: false,
                    available: true,
                    source_count,
                    announced_count: source_count,
                    changed_at: None,
                    diverged_since: None,
                    flips: 0,
                    interruption_announced: false,
                    facts_lost: 0,
                    last_gap_notice: None,
                    observation_gaps: 0,
                };
                let result = json!({"status":"subscribed","subscription":subscription_json(&sub)});
                self.subscriptions.insert(id, sub);
                self.revision += 1;
                result
            }
            Request::List {} => {
                let mut subs: Vec<_> = self
                    .subscriptions
                    .values()
                    .filter(|s| s.owner == owner)
                    .collect();
                subs.sort_by(|a, b| a.id.cmp(&b.id));
                json!({"subscriptions":subs.into_iter().map(subscription_json).collect::<Vec<_>>()})
            }
            Request::Unsub { id } => {
                if self.subscriptions.get(&id).is_none_or(|s| s.owner != owner) {
                    return json!({"status":"rejected","reason":"subscription_not_found"});
                }
                self.subscriptions.remove(&id);
                self.revision += 1;
                json!({"status":"unsubscribed","id":id})
            }
        }
    }

    /// O(F + S) borné (4096 chemins, 128 abonnements), aucune E/S.
    pub fn observe(&mut self, mut fact: Fact, now: Instant) -> Vec<Notification> {
        if let Some(path) = &fact.file {
            let Some(normalized) = normalized_absolute(path) else {
                return vec![];
            };
            fact.file = Some(normalized);
        }
        let mut facts = vec![fact.clone()];
        if fact.event == Kind::FileWritten {
            self.writes
                .retain(|_, w| now.saturating_duration_since(w.at) < Duration::from_secs(30));
            let Some(path) = fact.file.clone() else {
                return vec![];
            };
            let key = (fact.host.clone(), path);
            if !self.writes.contains_key(&key) && self.writes.len() >= 4096 {
                // Un cache saturé repart explicitement à froid, sans prétendre
                // couvrir une fenêtre dont les témoins n'ont pas été retenus.
                self.evicted_writes += self.writes.len() as u64;
                self.writes.clear();
            }
            let mut warned = None;
            if let Some(previous) = self.writes.get(&key) {
                warned = previous.warned.clone();
                if previous.agent != fact.agent {
                    let mut pair = [previous.agent.clone(), fact.agent.clone()];
                    pair.sort();
                    let duplicate = warned.as_ref().is_some_and(|(a, b, at)| {
                        a == &pair[0]
                            && b == &pair[1]
                            && now.saturating_duration_since(*at) < Duration::from_secs(30)
                    });
                    if !duplicate {
                        let mut collision = fact.clone();
                        collision.event = Kind::FileCollision;
                        collision.other_agent = Some(previous.agent.clone());
                        facts.push(collision);
                        warned = Some((pair[0].clone(), pair[1].clone(), now));
                    }
                }
            }
            self.writes.insert(
                key,
                RecentWrite {
                    agent: fact.agent,
                    at: now,
                    warned,
                },
            );
        }
        let mut notifications = vec![];
        let before = self.subscriptions.len();
        self.subscriptions.retain(|_, sub| {
            if sub.expires <= now { return false; }
            if !sub.available { return true; }
            for fact in &facts {
                if sub.event != fact.event
                    || sub.agent.as_ref().is_some_and(|a| a!=&fact.agent && fact.other_agent.as_ref()!=Some(a))
                    || sub.file.as_ref().is_some_and(|pattern| !fact.file.as_ref().is_some_and(|path| path_matches(pattern,path))) { continue; }
                if sub.last_notification.is_some_and(|last| now.saturating_duration_since(last)<Duration::from_millis(200)) {
                    sub.suppressed = sub.suppressed.saturating_add(1);
                    continue;
                }
                sub.last_notification = Some(now);
                let body = json!({"subscription_id":sub.id,"event":fact.event,
                    "agent":fact.agent,"host":fact.host,"file":fact.file,"other_agent":fact.other_agent,
                    "suppressed_total":sub.suppressed,
                    "notice":"fait observé, pas validation de mission ; aucune action obligatoire"});
                notifications.push(Notification { owner:sub.owner.clone(), body:format!("[Bridget observation] {body}") });
                if sub.once { return false; }
            }
            true
        });
        if before != self.subscriptions.len() {
            self.revision += 1;
        }
        notifications
    }
}

fn subscription_json(s: &Subscription) -> Value {
    json!({"id":s.id,"event":s.event,"agent":s.agent,"file":s.file,
        "once":s.once,"expires_at":s.expires_at,"suppressed_total":s.suppressed,
        "sources_compatible":s.source_count,
        "facts_lost_total":s.facts_lost,
        "observation_gaps":s.observation_gaps,
        "state":if s.available {"active"} else {"source_unavailable"}})
}

/// Comparaison lexicale Unix, sans I/O ni résolution des symlinks.
fn normalized_absolute(value: &str) -> Option<String> {
    if value.len() > 4096 || value.chars().any(char::is_control) || !Path::new(value).is_absolute()
    {
        return None;
    }
    let mut parts = Vec::new();
    for component in Path::new(value).components() {
        match component {
            Component::Normal(s) => parts.push(s.to_str()?),
            Component::ParentDir => {
                parts.pop()?;
            }
            Component::RootDir | Component::CurDir => {}
            _ => return None,
        }
    }
    (!parts.is_empty()).then(|| format!("/{}", parts.join("/")))
}

// Recherche de segments successifs ; entrées bornées à 256/4096 octets.
fn path_matches(pattern: &str, path: &str) -> bool {
    let mut rest = path;
    let segments: Vec<_> = pattern.split('*').collect();
    if segments.len() == 1 {
        return pattern == path;
    }
    if let Some(stripped) = rest.strip_prefix(segments[0]) {
        rest = stripped;
    } else {
        return false;
    }
    for segment in &segments[1..segments.len() - 1] {
        let Some(index) = rest.find(segment) else {
            return false;
        };
        rest = &rest[index + segment.len()..];
    }
    rest.ends_with(segments[segments.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec101_gap_is_counted_filtered_and_notification_rate_bounded() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        state.set_source("b", "b", vec![Kind::TurnEnded], now);
        let for_agent = |agent: &str| Request::Sub {
            event: Kind::TurnEnded,
            agent: Some(agent.into()),
            file: None,
            once: true,
            ttl_secs: Some(60),
        };
        state.request("owner-a", for_agent("a"), now, 100);
        state.request("owner-b", for_agent("b"), now, 100);
        let notices = state.report_gap("a", 44, now);
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].owner, "owner-a");
        assert!(notices[0].body.contains("observation_gap"));
        assert!(state.report_gap("a", 2, now).is_empty());
        assert_eq!(state.facts_lost, 46);
        let list = state.request("owner-a", Request::List {}, now, 100);
        assert_eq!(list["subscriptions"][0]["facts_lost_total"], 46);
        assert_eq!(list["subscriptions"][0]["state"], "active");
        assert_eq!(
            state.subscriptions.len(),
            2,
            "la lacune ne consomme jamais once"
        );
    }
    #[test]
    fn spec101_sources_are_required_and_loss_is_explicit() {
        let now = Instant::now();
        let mut state = Observations::default();
        assert_eq!(
            state.request("owner", sub(Kind::TurnEnded, false), now, 100)["reason"],
            "no_compatible_source"
        );
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        assert!(state.due_source_notices(now).is_empty());
        let catalogue = state.request("owner", Request::Types {}, now, 100);
        assert_eq!(catalogue["events"][0]["available"], true);
        assert_eq!(catalogue["events"][0]["sources"], json!(["a"]));
        assert_eq!(catalogue["events"][2]["available"], false);
        assert_eq!(
            state.request("owner", sub(Kind::FileWritten, false), now, 100)["reason"],
            "no_compatible_source"
        );
        state.request("owner", long_sub(Kind::TurnEnded), now, 100);
        state.remove_source("conn", now);
        assert_eq!(
            state.request("owner", Request::List {}, now, 100)["subscriptions"][0]["state"],
            "source_unavailable"
        );
        let later = now + SOURCE_NOTICE_GRACE;
        assert_eq!(state.due_source_notices(later).len(), 1);
        state.set_source("new", "a", vec![Kind::TurnEnded], later);
        assert_eq!(
            state.request("owner", Request::List {}, later, 100)["subscriptions"][0]["state"],
            "active"
        );
        assert_eq!(
            state.due_source_notices(later + SOURCE_NOTICE_GRACE).len(),
            1
        );
    }

    #[test]
    fn spec119_aller_retour_bref_de_source_sans_avis() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        state.request("owner", long_sub(Kind::TurnEnded), now, 100);
        state.remove_source("conn", now);
        state.set_source(
            "conn2",
            "a",
            vec![Kind::TurnEnded],
            now + Duration::from_secs(3),
        );
        assert!(
            state
                .due_source_notices(now + SOURCE_NOTICE_MAX_DELAY * 2)
                .is_empty(),
            "revenu à l'état annoncé : rien à dire"
        );
        // Une interruption qui dure est annoncée, une seule fois.
        state.remove_source("conn2", now);
        let bodies: Vec<_> = (0..120)
            .flat_map(|s| state.due_source_notices(now + Duration::from_secs(s)))
            .map(|n| n.body)
            .collect();
        assert_eq!(bodies.len(), 1);
        assert!(bodies[0].contains("source_unavailable"));
    }

    #[test]
    fn spec119_source_qui_clignote_avis_borne_par_periode() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        state.request("owner", long_sub(Kind::TurnEnded), now, 100);
        // Le cas du 25/09 : indisponible puis disponible toutes les 3 s, 10 min.
        let mut notices = Vec::new();
        for tick in 0..600u64 {
            let at = now + Duration::from_secs(tick);
            match tick % 6 {
                0 => state.remove_source("conn", at),
                3 => state.set_source("conn", "a", vec![Kind::TurnEnded], at),
                _ => {}
            }
            notices.extend(state.due_source_notices(at).into_iter().map(|n| n.body));
        }
        assert_eq!(
            notices.len(),
            1,
            "un avis par période de 5 min, pas un par bascule"
        );
        assert!(notices[0].contains("source instable"));
    }

    #[test]
    fn spec122_restore_resumes_and_absolute_ttl_is_preserved() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("conn", "a", vec![Kind::TurnEnded], now);
        state.request("owner", sub(Kind::TurnEnded, true), now, 100);
        let bytes = state.snapshot().unwrap();
        // Source pas encore revenue : indisponible, annoncée seulement si cela dure.
        let mut absent = Observations::restore(&bytes, now, 110).unwrap();
        assert_eq!(
            absent.request("owner", Request::List {}, now, 110)["subscriptions"][0]["state"],
            "source_unavailable"
        );
        let later = now + SOURCE_NOTICE_GRACE;
        assert!(
            absent.due_source_notices(later)[0]
                .body
                .contains("source indisponible")
        );
        // Source revenue : reprise sans nouvel abonnement, un seul avis au propriétaire.
        let mut restored = Observations::restore(&bytes, now, 110).unwrap();
        restored.set_source("conn", "a", vec![Kind::TurnEnded], now);
        assert_eq!(
            restored.request("owner", Request::List {}, now, 110)["subscriptions"][0]["state"],
            "active"
        );
        let notices = restored.owner_returned("owner", now);
        assert_eq!(notices.len(), 1);
        assert!(notices[0].body.contains("repris automatiquement"));
        assert!(restored.owner_returned("owner", now).is_empty());
        assert!(
            restored.due_source_notices(later).is_empty(),
            "retour à l'état connu"
        );
        assert_eq!(
            restored
                .observe(
                    Fact {
                        event: Kind::TurnEnded,
                        agent: "a".into(),
                        host: "h".into(),
                        file: None,
                        other_agent: None
                    },
                    now
                )
                .len(),
            1,
            "les faits futurs sont remis"
        );
        assert!(
            Observations::restore(&bytes, now, 161)
                .unwrap()
                .subscriptions
                .is_empty()
        );
        assert!(Observations::restore(b"invalid", now, 110).is_err());
    }
    fn long_sub(event: Kind) -> Request {
        Request::Sub {
            event,
            agent: None,
            file: None,
            once: false,
            ttl_secs: Some(3600),
        }
    }

    fn sub(event: Kind, once: bool) -> Request {
        Request::Sub {
            event,
            agent: None,
            file: None,
            once,
            ttl_secs: Some(60),
        }
    }
    fn file(agent: &str, host: &str, path: &str) -> Fact {
        Fact {
            event: Kind::FileWritten,
            agent: agent.into(),
            host: host.into(),
            file: Some(path.into()),
            other_agent: None,
        }
    }
    #[test]
    fn spec100_once_owner_expiry_and_filters() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        let receipt = state.request("owner", sub(Kind::TurnEnded, true), now, 100);
        let id = receipt["subscription"]["id"].as_str().unwrap().to_string();
        assert_eq!(
            state.request("other", Request::Unsub { id }, now, 100)["status"],
            "rejected"
        );
        let fact = Fact {
            event: Kind::TurnEnded,
            agent: "a".into(),
            host: "h".into(),
            file: None,
            other_agent: None,
        };
        assert_eq!(state.observe(fact.clone(), now).len(), 1);
        assert!(state.observe(fact.clone(), now).is_empty());
        state.request("owner", sub(Kind::TurnEnded, false), now, 100);
        assert!(
            state
                .observe(fact, now + Duration::from_secs(61))
                .is_empty()
        );
        assert!(path_matches("*.rs", "/project/src/lib.rs"));
        assert!(!path_matches("/other/*", "/project/src/lib.rs"));
    }
    #[test]
    fn spec100_collision_is_scoped_and_deduplicated() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::FileWritten], now);
        state.request("owner", sub(Kind::FileCollision, false), now, 100);
        assert!(state.observe(file("a", "h", "/p/x"), now).is_empty());
        assert!(state.observe(file("a", "h", "/p/./x"), now).is_empty());
        assert!(state.observe(file("b", "other", "/p/x"), now).is_empty());
        assert!(state.observe(file("b", "h", "/p/y"), now).is_empty());
        assert_eq!(state.observe(file("b", "h", "/p/x"), now).len(), 1);
        assert!(state.observe(file("a", "h", "/p/x"), now).is_empty());
        assert!(
            state
                .observe(file("c", "h", "/p/x"), now + Duration::from_secs(31))
                .is_empty()
        );
        assert_eq!(normalized_absolute("/a/../p/x"), Some("/p/x".into()));
        assert_eq!(normalized_absolute("relative"), None);
    }
    #[test]
    fn spec100_subscriptions_are_bounded_and_unknown_fields_refused() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::TurnEnded], now);
        for _ in 0..16 {
            assert_eq!(
                state.request("owner", sub(Kind::TurnEnded, false), now, 0)["status"],
                "subscribed"
            );
        }
        assert_eq!(
            state.request("owner", sub(Kind::TurnEnded, false), now, 0)["reason"],
            "subscription_limit"
        );
        assert!(
            serde_json::from_value::<Request>(json!({"action":"list","owner":"victim"})).is_err()
        );
        for owner in 1..8 {
            for _ in 0..16 {
                state.request(
                    &format!("owner{owner}"),
                    sub(Kind::TurnEnded, false),
                    now,
                    0,
                );
            }
        }
        assert_eq!(
            state.request("new-owner", sub(Kind::TurnEnded, false), now, 0)["reason"],
            "subscription_limit"
        );
        let list = state.request("owner", Request::List {}, now, 0);
        let id = list["subscriptions"][0]["id"].as_str().unwrap();
        assert_eq!(
            state.request("owner", Request::Unsub { id: id.into() }, now, 0)["status"],
            "unsubscribed"
        );
        assert_eq!(
            state.request("owner", Request::List {}, now, 0)["subscriptions"]
                .as_array()
                .unwrap()
                .len(),
            15
        );
    }

    #[test]
    fn spec100_rate_cache_and_combined_filters_are_bounded() {
        let now = Instant::now();
        let mut state = Observations::default();
        state.set_source("a", "a", vec![Kind::FileWritten], now);
        state.request(
            "owner",
            Request::Sub {
                event: Kind::FileWritten,
                agent: Some("a".into()),
                file: Some("/p/*.rs".into()),
                once: false,
                ttl_secs: Some(60),
            },
            now,
            0,
        );
        assert!(state.observe(file("b", "h", "/p/x.rs"), now).is_empty());
        assert!(state.observe(file("a", "h", "/p/x.txt"), now).is_empty());
        assert_eq!(state.observe(file("a", "h", "/p/x.rs"), now).len(), 1);
        assert!(state.observe(file("a", "h", "/p/y.rs"), now).is_empty());
        assert_eq!(
            state.request("owner", Request::List {}, now, 0)["subscriptions"][0]["suppressed_total"],
            1
        );
        assert_eq!(
            state
                .observe(file("a", "h", "/p/z.rs"), now + Duration::from_millis(201))
                .len(),
            1
        );
        for i in 0..4097 {
            state.observe(file("b", "h", &format!("/cache/{i}")), now);
        }
        assert!(state.writes.len() <= 4096);
        assert!(state.evicted_writes >= 4096);
    }
}
