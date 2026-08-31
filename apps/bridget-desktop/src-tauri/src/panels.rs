//! Registre des panneaux distants, sans mélange d'origines.

use std::collections::HashMap;

pub const MAXIMUM_OPEN_PANELS: usize = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panel {
    pub label: String,
    pub profile_id: String,
    pub url: String,
}

#[derive(Default)]
pub struct PanelRegistry {
    panels: HashMap<String, Panel>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PanelError {
    LimitReached,
    DuplicateProfile,
    InvalidRelayUrl,
}

impl std::fmt::Display for PanelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LimitReached => "Deux panneaux au maximum peuvent être ouverts.",
            Self::DuplicateProfile => "Ce profil est déjà affiché.",
            Self::InvalidRelayUrl => "Un panneau doit viser uniquement son relais local.",
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
        if self.panels.len() >= MAXIMUM_OPEN_PANELS {
            return Err(PanelError::LimitReached);
        }
        if self
            .panels
            .values()
            .any(|panel| panel.profile_id == profile_id)
        {
            return Err(PanelError::DuplicateProfile);
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
    pub fn panels(&self) -> impl Iterator<Item = &Panel> {
        self.panels.values()
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

#[cfg(test)]
mod tests {
    use super::{is_loopback_relay_url, MAXIMUM_OPEN_PANELS, PanelError, PanelRegistry};
    #[test]
    fn labels_sont_uniques_limites_a_deux_et_url_hors_loopback_refusee() {
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
        registry
            .open("local", "http://127.0.0.1:39002/?token=fixture")
            .unwrap();
        assert_eq!(registry.panels().count(), MAXIMUM_OPEN_PANELS);
        assert!(matches!(
            registry.open("third", "http://127.0.0.1:39003/?token=fixture"),
            Err(PanelError::LimitReached)
        ));
        assert!(matches!(
            PanelRegistry::default().open("bad", "https://example.com/?token=fixture"),
            Err(PanelError::InvalidRelayUrl)
        ));
    }
}
