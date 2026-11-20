//! Routes opt-in mobiles (spec 306) : recherche plein texte dans le ledger,
//! filtres de notifications par projet/urgence, relance encadrée d'agent arrêté
//! et lancement encadré d'un profil spécialiste approuvé.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::Path;

const FILTERS_FILE: &str = "mobile-notification-filters.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct NotificationFilterPrefs {
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default = "default_urgency")]
    pub min_urgency: String,
}

fn default_urgency() -> String {
    "all".to_string()
}

impl Default for NotificationFilterPrefs {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            min_urgency: default_urgency(),
        }
    }
}

impl NotificationFilterPrefs {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.min_urgency.as_str(), "all" | "normal" | "high") {
            return Err("invalid_request");
        }
        if self.projects.len() > 64
            || self
                .projects
                .iter()
                .any(|p| p.trim().is_empty() || p.len() > 128)
        {
            return Err("invalid_request");
        }
        Ok(())
    }
}

pub(super) fn load_notification_filters(root: &Path, device_id: &str) -> NotificationFilterPrefs {
    let path = root.join(FILTERS_FILE);
    let Ok(raw) = std::fs::read(&path) else {
        return NotificationFilterPrefs::default();
    };
    let Ok(map) =
        serde_json::from_slice::<std::collections::BTreeMap<String, NotificationFilterPrefs>>(&raw)
    else {
        return NotificationFilterPrefs::default();
    };
    map.get(device_id).cloned().unwrap_or_default()
}

pub(super) fn save_notification_filters(
    root: &Path,
    device_id: &str,
    prefs: NotificationFilterPrefs,
) -> Result<(), &'static str> {
    prefs.validate()?;
    let path = root.join(FILTERS_FILE);
    let mut map: std::collections::BTreeMap<String, NotificationFilterPrefs> = std::fs::read(&path)
        .ok()
        .and_then(|raw| serde_json::from_slice(&raw).ok())
        .unwrap_or_default();
    map.insert(device_id.to_string(), prefs);
    let bytes = serde_json::to_vec_pretty(&map).map_err(|_| "storage_unavailable")?;
    bridget_transport::fsutil::write_private_file_atomic(&path, &bytes)
        .map_err(|_| "storage_unavailable")
}

pub(super) fn opt_in_control_enabled() -> bool {
    std::env::var("BRIDGET_MOBILE_OPT_IN_CONTROL")
        .map(|v| matches!(v.trim(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

/// Recherche plein texte bornée dans la table `ledger` de `bridget.db` en lecture seule.
pub(super) fn search_ledger(
    root: &Path,
    term: &str,
    project: Option<&str>,
    limit: usize,
) -> Result<Value, &'static str> {
    let needle = term.trim();
    if needle.is_empty() || needle.len() > 256 {
        return Err("invalid_request");
    }
    let db_path = root.join("bridget.db");
    if !db_path.is_file() {
        return Ok(json!({"status": "ok", "items": [], "query": needle}));
    }
    let conn =
        rusqlite::Connection::open_with_flags(&db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "storage_unavailable")?;
    let capped = limit.clamp(1, 50) as i64;
    let pattern = format!("%{}%", needle.to_lowercase());
    let mut stmt = conn
        .prepare(
            "SELECT id, from_agent, to_agent, body, ts, conversation_key \
             FROM ledger \
             WHERE lower(body) LIKE ?1 OR lower(from_agent) LIKE ?1 OR lower(to_agent) LIKE ?1 \
             ORDER BY ts DESC LIMIT ?2",
        )
        .map_err(|_| "storage_unavailable")?;
    let rows = stmt
        .query_map(rusqlite::params![pattern, capped], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .map_err(|_| "storage_unavailable")?;

    let mut items = Vec::new();
    for row in rows {
        let Ok((id, from, to, body, ts, conv)) = row else {
            continue;
        };
        if let Some(proj) = project
            && !proj.trim().is_empty()
            && !conv.to_lowercase().contains(&proj.to_lowercase())
        {
            continue;
        }
        items.push(json!({
            "id": id,
            "agent_id": to,
            "from": from,
            "project": conv,
            "body": body,
            "ts": ts,
        }));
    }
    Ok(json!({"status": "ok", "query": needle, "items": items}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec306_mobile_notification_filters_and_ledger_search_roundtrip() {
        let dir = std::env::temp_dir().join(format!("bridget-mopt-{}", uuid::Uuid::new_v4()));
        bridget_transport::fsutil::create_private_dir(&dir).unwrap();

        let prefs = NotificationFilterPrefs {
            projects: vec!["bridget".into()],
            min_urgency: "high".into(),
        };
        save_notification_filters(&dir, "dev-1", prefs.clone()).unwrap();
        assert_eq!(load_notification_filters(&dir, "dev-1"), prefs);

        let bad = NotificationFilterPrefs {
            projects: vec![],
            min_urgency: "invalid".into(),
        };
        assert_eq!(bad.validate(), Err("invalid_request"));

        let conn = crate::store_schema::open_private_wal(&dir.join("bridget.db")).unwrap();
        conn.execute_batch(
            "CREATE TABLE ledger(id TEXT, from_agent TEXT, to_agent TEXT, body TEXT, ts INTEGER, conversation_key TEXT);
             INSERT INTO ledger VALUES('m1','codex','agent1','Audit securite OK',100,'bridget:a');
             INSERT INTO ledger VALUES('m2','claude','agent2','Autre sujet',200,'parix:b');",
        )
        .unwrap();
        let res = search_ledger(&dir, "securite", Some("bridget"), 10).unwrap();
        assert_eq!(res["items"].as_array().unwrap().len(), 1);
        assert_eq!(res["items"][0]["id"], "m1");

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
