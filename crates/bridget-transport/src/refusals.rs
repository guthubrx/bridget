//! SPEC-088 : forme unique d'un refus et reconnaissance fermée des refus de
//! sandbox sur la sortie brute d'une commande. Aucune déduction : une ligne
//! qui ne correspond à aucun motif n'est pas un refus.

use serde::{Deserialize, Serialize};

/// Borne de la ligne brute conservée avec un refus (caractères).
pub const MAX_RAW_CHARS: usize = 512;

/// Couches fermées. Ajouter une couche est une décision de spécification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalLayer {
    BrowserContent,
    ArtifactSandbox,
    ProviderSandbox,
    ServerRuntime,
    BridgetSystem,
    ReferentControl,
    BridgetPolicy,
    Unknown,
}

impl RefusalLayer {
    pub const ALL: &'static [Self] = &[
        Self::BrowserContent,
        Self::ArtifactSandbox,
        Self::ProviderSandbox,
        Self::ServerRuntime,
        Self::BridgetSystem,
        Self::ReferentControl,
        Self::BridgetPolicy,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BrowserContent => "browser_content",
            Self::ArtifactSandbox => "artifact_sandbox",
            Self::ProviderSandbox => "provider_sandbox",
            Self::ServerRuntime => "server_runtime",
            Self::BridgetSystem => "bridget_system",
            Self::ReferentControl => "referent_control",
            Self::BridgetPolicy => "bridget_policy",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|layer| layer.as_str() == raw)
    }
}

/// Geste proposé au référent pour changer le comportement refusé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RefusalGesture {
    /// Activer une préférence locale du navigateur (clé fermée côté vue).
    LocalToggle {
        target: String,
    },
    /// Ouvrir une ligne de la page Droits.
    RightsLine {
        target: String,
    },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefusalAttribution {
    Bridget,
    Agent,
}

/// Forme unique d'un refus (ADR-028) : qui refuse, ce qui est empêché, le
/// geste, la ligne brute qui permet de contredire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Refusal {
    pub layer: RefusalLayer,
    pub prevented: String,
    pub gesture: RefusalGesture,
    pub raw: String,
    pub at: i64,
    pub attributed_to: RefusalAttribution,
}

/// Motifs contractuels (`specs/088-droits/contracts/rights-v1.md`). Chaque
/// entrée : préfixe de ligne, fragment requis plus loin dans la ligne.
const SANDBOX_PATTERNS: &[(&str, &[&str])] = &[
    ("bwrap: ", &["Operation not permitted", "Permission denied"]),
    ("sandbox-exec: ", &["deny"]),
    ("Sandbox: ", &["deny"]),
];

/// Reconnaît un refus de sandbox du fournisseur sur UNE ligne de sortie brute.
/// Complexité : O(k) avec k = nombre de motifs (3).
pub fn recognize_sandbox_refusal(line: &str, at: i64) -> Option<Refusal> {
    let trimmed = line.trim_start();
    for (prefix, fragments) in SANDBOX_PATTERNS {
        if let Some(rest) = trimmed.strip_prefix(prefix)
            && fragments.iter().any(|fragment| rest.contains(fragment))
        {
            return Some(Refusal {
                layer: RefusalLayer::ProviderSandbox,
                prevented: "shell".to_string(),
                gesture: RefusalGesture::RightsLine {
                    target: "shell".to_string(),
                },
                raw: bounded_raw(trimmed),
                at,
                attributed_to: RefusalAttribution::Bridget,
            });
        }
    }
    None
}

/// Tronque à la borne sans couper un caractère.
pub fn bounded_raw(line: &str) -> String {
    line.trim_end().chars().take(MAX_RAW_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bwrap_operation_not_permitted_est_un_refus_de_sandbox() {
        let refusal = recognize_sandbox_refusal(
            "bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted",
            7,
        )
        .expect("refus attendu");
        assert_eq!(refusal.layer, RefusalLayer::ProviderSandbox);
        assert_eq!(refusal.prevented, "shell");
        assert_eq!(
            refusal.gesture,
            RefusalGesture::RightsLine {
                target: "shell".to_string()
            }
        );
        assert_eq!(refusal.at, 7);
        assert_eq!(refusal.attributed_to, RefusalAttribution::Bridget);
        assert!(refusal.raw.starts_with("bwrap: loopback"));
    }

    #[test]
    fn bwrap_permission_denied_et_seatbelt_sont_reconnus() {
        assert!(recognize_sandbox_refusal("bwrap: execvp: Permission denied", 0).is_some());
        assert!(recognize_sandbox_refusal("sandbox-exec: deny file-read-data /etc", 0).is_some());
        assert!(
            recognize_sandbox_refusal("  Sandbox: bash(12) deny(1) network-outbound", 0).is_some()
        );
    }

    #[test]
    fn une_ligne_sans_motif_n_est_pas_un_refus() {
        // Contrôle positif de l'absence : des lignes d'erreur ordinaires.
        assert!(recognize_sandbox_refusal("", 0).is_none());
        assert!(
            recognize_sandbox_refusal("ls: cannot access 'x': No such file or directory", 0)
                .is_none()
        );
        assert!(recognize_sandbox_refusal("Operation not permitted", 0).is_none());
        assert!(recognize_sandbox_refusal("echo bwrap: Operation not permitted", 0).is_none());
    }

    #[test]
    fn la_ligne_brute_est_bornee_sans_couper_un_caractere() {
        let long = format!("bwrap: {}é Operation not permitted", "x".repeat(600));
        let refusal = recognize_sandbox_refusal(&long, 0).unwrap();
        assert_eq!(refusal.raw.chars().count(), MAX_RAW_CHARS);
    }

    #[test]
    fn les_couches_se_parsent_dans_les_deux_sens() {
        for layer in RefusalLayer::ALL {
            assert_eq!(RefusalLayer::parse(layer.as_str()), Some(*layer));
        }
        assert_eq!(RefusalLayer::parse("sandbox"), None);
        let json = serde_json::to_string(&RefusalLayer::ProviderSandbox).unwrap();
        assert_eq!(json, "\"provider_sandbox\"");
    }
}
