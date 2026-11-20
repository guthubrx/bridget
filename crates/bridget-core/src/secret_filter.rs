//! Filtre préventif configurable des motifs de secrets manifestes avant persistance.
//!
//! Par défaut (`SecretFilterMode::Off`), aucune transformation n'est appliquée
//! afin de préserver la rétrocompatibilité complète des tests et échanges.
//! Les modes `Redact` (masquage) et `Reject` (refus) se pilotent par configuration
//! ou via `BRIDGET_SECRET_FILTER=redact|reject`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecretFilterMode {
    #[default]
    Off,
    Redact,
    Reject,
}

impl SecretFilterMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "redact" | "mask" => Self::Redact,
            "reject" | "block" | "deny" => Self::Reject,
            _ => Self::Off,
        }
    }

    pub fn from_env() -> Self {
        std::env::var("BRIDGET_SECRET_FILTER")
            .map(|v| Self::parse(&v))
            .unwrap_or(Self::Off)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretFilterOutcome {
    Clean,
    Redacted { text: String, matches: usize },
    Rejected { rule: &'static str },
}

const SECRET_PREFIXES: &[(&str, &str, usize)] = &[
    ("openai_api_key", "sk-proj-", 20),
    ("anthropic_api_key", "sk-ant-", 20),
    ("github_pat", "ghp_", 20),
    ("github_fine_grained_pat", "github_pat_", 20),
    ("aws_access_key", "AKIA", 16),
    ("slack_token", "xoxb-", 16),
    ("slack_token", "xoxp-", 16),
    ("private_key_pem", "-----BEGIN PRIVATE KEY-----", 0),
    ("private_key_pem", "-----BEGIN OPENSSH PRIVATE KEY-----", 0),
    ("private_key_pem", "-----BEGIN RSA PRIVATE KEY-----", 0),
];

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'+' | b'/' | b'=')
}

/// Inspecte un texte et applique la politique de filtrage configurée.
pub fn filter_secrets(text: &str, mode: SecretFilterMode) -> SecretFilterOutcome {
    if mode == SecretFilterMode::Off {
        return SecretFilterOutcome::Clean;
    }

    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let bytes = text.as_bytes();
    let mut matches = 0usize;

    while i < bytes.len() {
        let rest = &text[i..];
        let mut matched: Option<(&'static str, usize)> = None;
        for &(rule, prefix, min_tail) in SECRET_PREFIXES {
            if rest.starts_with(prefix) {
                if min_tail == 0 {
                    // PEM header: mask until end marker or end of text.
                    let end = rest
                        .find("-----END ")
                        .and_then(|pos| {
                            rest[pos..]
                                .find("-----\n")
                                .map(|e| pos + e + 6)
                                .or_else(|| rest[pos..].find("-----").map(|e| pos + e + 5))
                        })
                        .unwrap_or(rest.len());
                    matched = Some((rule, end));
                    break;
                }
                let mut end = prefix.len();
                while end < rest.len() && is_token_char(rest.as_bytes()[end]) {
                    end += 1;
                }
                if end - prefix.len() >= min_tail {
                    matched = Some((rule, end));
                    break;
                }
            }
        }

        if let Some((rule, span)) = matched {
            if mode == SecretFilterMode::Reject {
                return SecretFilterOutcome::Rejected { rule };
            }
            matches += 1;
            out.push_str("[SECRET_MASQUE:");
            out.push_str(rule);
            out.push(']');
            i += span;
        } else {
            let ch = rest.chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    if matches == 0 {
        SecretFilterOutcome::Clean
    } else {
        SecretFilterOutcome::Redacted { text: out, matches }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_mode_leaves_input_unchanged() {
        let sample = "key=ghp_1234567890abcdefghij123456";
        assert_eq!(
            filter_secrets(sample, SecretFilterMode::Off),
            SecretFilterOutcome::Clean
        );
    }

    #[test]
    fn redact_mode_masks_detected_tokens() {
        let sample = "token ghp_1234567890abcdefghij123456 fin";
        let outcome = filter_secrets(sample, SecretFilterMode::Redact);
        assert_eq!(
            outcome,
            SecretFilterOutcome::Redacted {
                text: "token [SECRET_MASQUE:github_pat] fin".into(),
                matches: 1,
            }
        );
    }

    #[test]
    fn reject_mode_refuses_detected_tokens() {
        let sample =
            "-----BEGIN OPENSSH PRIVATE KEY-----\nabc\n-----END OPENSSH PRIVATE KEY-----\n";
        assert_eq!(
            filter_secrets(sample, SecretFilterMode::Reject),
            SecretFilterOutcome::Rejected {
                rule: "private_key_pem"
            }
        );
    }

    #[test]
    fn normal_text_and_short_prefixes_pass_clean() {
        let sample = "AKIA123 court et texte normal";
        assert_eq!(
            filter_secrets(sample, SecretFilterMode::Reject),
            SecretFilterOutcome::Clean
        );
    }
}
