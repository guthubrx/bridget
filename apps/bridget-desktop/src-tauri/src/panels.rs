//! Registre des panneaux distants, sans mélange d'origines.

use std::collections::HashMap;

pub const MAXIMUM_OPEN_PANELS: usize = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panel {
    pub label: String,
    pub profile_id: String,
    pub url: String,
}

/// Enfant WebView opérateur. Il ne compte pas comme un second panneau de
/// relais : le registre reste propriétaire de la régie visuelle unique.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserPanel {
    pub label: String,
    pub target: String,
}

#[derive(Default)]
pub struct PanelRegistry {
    panels: HashMap<String, Panel>,
    browser: Option<BrowserPanel>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PanelError {
    LimitReached,
    DuplicateProfile,
    InvalidRelayUrl,
    InvalidBrowserUrl,
}

impl std::fmt::Display for PanelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LimitReached => "Un seul panneau peut être affiché à la fois.",
            Self::DuplicateProfile => "Ce profil est déjà affiché.",
            Self::InvalidRelayUrl => "Un panneau doit viser uniquement son relais local.",
            Self::InvalidBrowserUrl => {
                "Le Browser accepte seulement HTTPS ou une publication locale Bridget."
            }
        })
    }
}
impl std::error::Error for PanelError {}

impl PanelRegistry {
    pub fn open(
        &mut self,
        profile_id: impl Into<String>,
        url: impl Into<String>,
    ) -> Result<Panel, PanelError> {
        let profile_id = profile_id.into();
        let url = url.into();
        if self
            .panels
            .values()
            .any(|panel| panel.profile_id == profile_id)
        {
            return Err(PanelError::DuplicateProfile);
        }
        if self.panels.len() >= MAXIMUM_OPEN_PANELS {
            return Err(PanelError::LimitReached);
        }
        if !is_loopback_relay_url(&url) {
            return Err(PanelError::InvalidRelayUrl);
        }
        let label = format!(
            "panel-{}",
            profile_id.replace(|character: char| !character.is_ascii_alphanumeric(), "-")
        );
        let panel = Panel {
            label: label.clone(),
            profile_id,
            url,
        };
        self.panels.insert(label, panel.clone());
        Ok(panel)
    }
    pub fn close(&mut self, label: &str) -> Option<Panel> {
        self.panels.remove(label)
    }

    /// Met à jour l'URL d'un panneau conservé après recréation de son tunnel.
    /// L'URL reste soumise à la même règle loopback que lors de l'ouverture.
    pub fn replace_relay_url(&mut self, label: &str, url: impl Into<String>) -> bool {
        let url = url.into();
        if !is_loopback_relay_url(&url) {
            return false;
        }
        let Some(panel) = self.panels.get_mut(label) else {
            return false;
        };
        panel.url = url;
        true
    }
    pub fn panels(&self) -> impl Iterator<Item = &Panel> {
        self.panels.values()
    }

    pub fn browser(&self) -> Option<&BrowserPanel> {
        self.browser.as_ref()
    }

    pub fn open_browser(&mut self, target: impl Into<String>) -> Result<BrowserPanel, PanelError> {
        let target = target.into();
        if !is_browser_target(&target) {
            return Err(PanelError::InvalidBrowserUrl);
        }
        let browser = BrowserPanel {
            label: "browser-primary".to_owned(),
            target,
        };
        self.browser = Some(browser.clone());
        Ok(browser)
    }

    pub fn clear_browser(&mut self) -> Option<BrowserPanel> {
        self.browser.take()
    }
}

pub fn is_loopback_relay_url(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("http://127.0.0.1:") else {
        return false;
    };
    let Some((port, path)) = rest.split_once('/') else {
        return false;
    };
    port.parse::<u16>().is_ok_and(|port| port != 0) && path.starts_with("?token=") && path.len() > 7
}

pub fn is_browser_target(value: &str) -> bool {
    if value == "bridget://browser-home" {
        return true;
    }
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    match url.scheme() {
        "https" => {
            url.host_str().is_some() && url.username().is_empty() && url.password().is_none()
        }
        "http" => url.host_str() == Some("127.0.0.1"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAXIMUM_OPEN_PANELS, PanelError, PanelRegistry, is_browser_target, is_loopback_relay_url,
    };
    #[test]
    fn un_seul_panneau_est_autorise_et_url_hors_loopback_refusee() {
        let mut registry = PanelRegistry::default();
        let first = registry
            .open("cartae.app", "http://127.0.0.1:39001/?token=fixture")
            .unwrap();
        assert!(first.label.starts_with("panel-"));
        assert!(is_loopback_relay_url(
            "http://127.0.0.1:39001/?token=fixture&view=settings"
        ));
        assert!(matches!(
            registry.open("cartae.app", "http://127.0.0.1:39002/?token=fixture"),
            Err(PanelError::DuplicateProfile)
        ));
        assert!(matches!(
            registry.open("local", "http://127.0.0.1:39002/?token=fixture"),
            Err(PanelError::LimitReached)
        ));
        assert_eq!(registry.panels().count(), MAXIMUM_OPEN_PANELS);
        assert!(registry.replace_relay_url(&first.label, "http://127.0.0.1:39003/?token=fixture"));
        assert_eq!(
            registry.panels().next().unwrap().url,
            "http://127.0.0.1:39003/?token=fixture"
        );
        assert!(!registry.replace_relay_url(&first.label, "https://example.com"));
        assert!(matches!(
            PanelRegistry::default().open("bad", "https://example.com/?token=fixture"),
            Err(PanelError::InvalidRelayUrl)
        ));
    }

    #[test]
    fn browser_unique_reste_dans_la_meme_regie_et_refuse_file() {
        let mut registry = PanelRegistry::default();
        assert!(is_browser_target("https://example.com/page"));
        assert!(is_browser_target("http://127.0.0.1:39001/artifact"));
        assert!(!is_browser_target("file:///tmp/secret"));
        assert!(matches!(
            registry.open_browser("file:///tmp/secret"),
            Err(PanelError::InvalidBrowserUrl)
        ));
        let browser = registry.open_browser("https://example.com").unwrap();
        assert_eq!(browser.label, "browser-primary");
        assert_eq!(registry.browser().unwrap().target, "https://example.com");
        assert_eq!(registry.clear_browser().unwrap().label, "browser-primary");
    }
}
