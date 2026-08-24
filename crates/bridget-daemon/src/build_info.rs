/// Identifiant du commit embarqué dans le binaire au moment de sa compilation.
pub const BUILD_ID: &str = env!("BRIDGET_BUILD_ID");

pub fn stale_daemon_warning(daemon_build_id: &str) -> Option<String> {
    stale_daemon_warning_for(BUILD_ID, daemon_build_id)
}

/// Compare deux identifiants libres — la production passe `BUILD_ID` en local.
pub fn stale_daemon_warning_for(local_build_id: &str, daemon_build_id: &str) -> Option<String> {
    (daemon_build_id != local_build_id).then(|| {
        // Incident fondateur (2026-08-23) : deux correctifs semblaient absents
        // pendant des heures parce qu'un daemon périmé continuait de répondre.
        let remediation = format!("launchctl kickstart -k gui/{}/com.bridget.daemon", unsafe {
            libc::getuid()
        });
        if daemon_build_id == "unknown" {
            format!("daemon build-id inconnu : {remediation}")
        } else {
            format!("daemon périmé ({daemon_build_id} vs {local_build_id}) : {remediation}")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecart_de_build_id_observe_les_identifiants_pas_un_libelle() {
        // Mutation : inverser la comparaison → l'égalité locale/daemon échoue.
        // Mutation : omettre un des deux id dans le message → contains échoue.
        let local = "build-local";
        assert!(stale_daemon_warning_for(local, local).is_none());

        let warning = stale_daemon_warning_for(local, "build-daemon").expect("écart signalé");
        assert!(warning.contains("build-daemon"));
        assert!(warning.contains(local));
        let remediation = format!("gui/{}/com.bridget.daemon", unsafe { libc::getuid() });
        assert!(warning.contains(&remediation));

        let unknown = stale_daemon_warning_for(local, "unknown").expect("inconnu signalé");
        assert!(unknown.contains(&remediation));

        // Surface production : même binaire → silence.
        assert!(stale_daemon_warning(BUILD_ID).is_none());
    }
}
