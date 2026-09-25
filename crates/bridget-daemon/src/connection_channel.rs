use std::path::{Path, PathBuf};

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn preferred_value(primary: Option<&str>, legacy: Option<&str>) -> Option<String> {
    non_empty(primary).or_else(|| non_empty(legacy))
}

fn federation_channel(config: &str) -> Option<String> {
    let value = |key: &str| {
        config.lines().find_map(|line| {
            line.strip_prefix(key)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
    };
    preferred_value(value("channel="), value("transport="))
}

/// Résout deux attestations indépendantes sans transformer leur ordre de
/// lecture en règle métier. Une divergence reste inconnue : l'environnement
/// est propre au processus mais peut être hérité, tandis que le fichier est
/// écrit par l'installateur du tunnel mais peut persister après lui.
pub(crate) fn attested_channel_from_sources(
    environment_channel: Option<&str>,
    legacy_environment_transport: Option<&str>,
    federation_config: Option<&str>,
) -> Option<String> {
    let environment = preferred_value(environment_channel, legacy_environment_transport);
    let federation = federation_config.and_then(federation_channel);
    match (environment, federation) {
        (Some(environment), Some(federation)) if environment != federation => None,
        (Some(environment), _) => Some(environment),
        (None, federation) => federation,
    }
}

fn federation_config_path() -> Option<PathBuf> {
    crate::environment::Namespace::from_environment()
        .ok()
        .map(|namespace| namespace.root.join("federation.env"))
}

fn read_federation_config(path: Option<&Path>) -> Option<String> {
    path.and_then(|path| {
        crate::environment::validate_state_file(path, false).ok()?;
        std::fs::read_to_string(path).ok()
    })
}

/// Canal attesté pour le processus courant. L'absence ou la contradiction
/// produit `None`; le type Unix de la socket n'est jamais une preuve réseau.
pub(crate) fn attested_connection_channel() -> Option<String> {
    let environment_channel = std::env::var("BRIDGET_CHANNEL").ok();
    let legacy_environment_transport = std::env::var("BRIDGET_TRANSPORT").ok();
    let config_path = federation_config_path();
    let federation_config = read_federation_config(config_path.as_deref());
    attested_channel_from_sources(
        environment_channel.as_deref(),
        legacy_environment_transport.as_deref(),
        federation_config.as_deref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_024_nouvelle_cle_prime_sur_alias_dans_chaque_source() {
        assert_eq!(
            attested_channel_from_sources(Some("unix"), Some("ancien"), None),
            Some("unix".to_string())
        );
        assert_eq!(
            attested_channel_from_sources(None, Some("ssh-unix"), None),
            Some("ssh-unix".to_string())
        );
        assert_eq!(
            attested_channel_from_sources(None, None, Some("transport=ancien\nchannel=ssh-unix\n")),
            Some("ssh-unix".to_string())
        );
    }

    #[test]
    fn spec_024_sources_concordantes_ou_uniques_attestent_le_canal() {
        assert_eq!(
            attested_channel_from_sources(Some("ssh-unix"), None, None),
            Some("ssh-unix".to_string())
        );
        assert_eq!(
            attested_channel_from_sources(None, None, Some("channel=unix\n")),
            Some("unix".to_string())
        );
        assert_eq!(
            attested_channel_from_sources(Some("ssh-unix"), None, Some("channel=ssh-unix\n")),
            Some("ssh-unix".to_string())
        );
    }

    #[test]
    fn spec_024_divergence_entre_environnement_et_fichier_reste_inconnue() {
        assert_eq!(
            attested_channel_from_sources(Some("unix"), None, Some("channel=ssh-unix\n")),
            None
        );
    }

    #[test]
    fn spec_024_absence_d_attestation_ne_devient_pas_unix() {
        assert_eq!(attested_channel_from_sources(None, None, None), None);
        assert_eq!(
            attested_channel_from_sources(Some("  "), None, Some("channel=  \n")),
            None
        );
    }
}
