//! Faits locaux, filtres et risques de collision. Aucun workflow ni accès disque.
use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Component, Path};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Subscription {
    id: String,
    owner: String,
    event: Kind,
    agent: Option<String>,
    file: Option<String>,
    once: bool,
    expires: Instant,
    expires_at: u64,
    last_notification: Option<Instant>,
    suppressed: u64,
}

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

struct RecentWrite {
    agent: String,
    at: Instant,
    warned: Option<(String, String, Instant)>,
}

#[derive(Default)]
pub(crate) struct Observations {
    subscriptions: HashMap<String, Subscription>,
    writes: HashMap<(String, String), RecentWrite>,
    pub evicted_writes: u64,
}

impl Observations {
    /// O(S), liste triée O(S log S), S <= 128. Identité hors Request.
    pub fn request(&mut self, owner: &str, request: Request, now: Instant, wall: u64) -> Value {
        self.subscriptions.retain(|_, s| s.expires > now);
        match request {
            Request::Types {} => json!({"events":[
                {"event":"turn_ended","meaning":"fin d'un tour, pas succès ou fin de mission"},
                {"event":"permission_required","meaning":"demande de permission observée, éventuellement déjà traitée"},
                {"event":"file_written","meaning":"écriture structurée confirmée par une intégration"},
                {"event":"file_collision","meaning":"risque : deux auteurs, même hôte/chemin, 30 secondes"}
            ]}),
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
                };
                let result = json!({"status":"subscribed","subscription":subscription_json(&sub)});
                self.subscriptions.insert(id, sub);
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
        self.subscriptions.retain(|_, sub| {
            if sub.expires <= now { return false; }
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
        notifications
    }
}

fn subscription_json(s: &Subscription) -> Value {
    json!({"id":s.id,"event":s.event,"agent":s.agent,"file":s.file,
        "once":s.once,"expires_at":s.expires_at,"suppressed_total":s.suppressed})
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
