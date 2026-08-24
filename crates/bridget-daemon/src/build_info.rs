/// Identifiant du commit embarqué dans le binaire au moment de sa compilation.
pub const BUILD_ID: &str = env!("BRIDGET_BUILD_ID");

pub fn stale_daemon_warning(daemon_build_id: &str) -> Option<String> {
    (daemon_build_id != BUILD_ID).then(|| {
        // Incident fondateur (2026-08-23) : deux correctifs semblaient absents
        // pendant des heures parce qu'un daemon périmé continuait de répondre.
        let remediation = format!("launchctl kickstart -k gui/{}/com.bridget.daemon", unsafe {
            libc::getuid()
        });
        if daemon_build_id == "unknown" {
            format!("daemon build-id inconnu : {remediation}")
        } else {
            format!("daemon périmé ({daemon_build_id} vs {BUILD_ID}) : {remediation}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_id_identique_reste_silencieux_et_ecart_est_explicite() {
        assert!(stale_daemon_warning(BUILD_ID).is_none());
        let warning = stale_daemon_warning("obsolete-commit").expect("écart signalé");
        assert!(warning.contains("daemon périmé (obsolete-commit vs"));
        assert!(warning.contains("launchctl kickstart -k gui/"));
        assert!(
            stale_daemon_warning("unknown")
                .expect("inconnu signalé honnêtement")
                .starts_with("daemon build-id inconnu")
        );
    }
}
