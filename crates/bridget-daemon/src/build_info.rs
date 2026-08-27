/// Identifiant du commit embarqué dans le binaire au moment de sa compilation.
pub const BUILD_ID: &str = env!("BRIDGET_BUILD_ID");

/// Désignation employée quand la machine du daemon n'est pas attestée.
pub const MACHINE_NON_ATTESTEE: &str = "machine non attestée";

/// Nom de la machine qui exécute ce binaire.
///
/// **Ré-export**, pas une copie : il n'existe qu'une seule implémentation, dans
/// `bridget_core::host`. Le langage interdit ici la divergence qu'un simple
/// renvoi manuel finirait par autoriser — et le greffe, qui ne dépend pas de ce
/// paquet, lit exactement la même.
pub use bridget_core::local_host;

pub fn stale_daemon_warning(daemon_build_id: &str) -> Option<String> {
    stale_daemon_warning_at(daemon_build_id, None)
}

pub fn stale_daemon_warning_at(daemon_build_id: &str, daemon_host: Option<&str>) -> Option<String> {
    stale_daemon_warning_for(BUILD_ID, &local_host(), daemon_build_id, daemon_host)
}

/// Commande de relance, **attribuée** à la machine où le daemon tourne.
///
/// Un daemon distant ne se relance pas par une commande locale : la nommer
/// serait la quatrième erreur d'attribution de la ligne fondatrice
/// (`launchctl` + uid local pour un daemon qui est ailleurs).
fn remediation(local_host: &str, daemon_host: Option<&str>) -> String {
    match daemon_host {
        Some(host) if host != local_host => {
            format!("relancer le daemon sur {host} — aucune commande locale ne l'atteint")
        }
        _ if cfg!(target_os = "macos") => {
            format!("launchctl kickstart -k gui/{}/com.bridget.daemon", unsafe {
                libc::getuid()
            })
        }
        _ => "systemctl --user restart bridget-daemon".to_string(),
    }
}

/// Compare deux identifiants libres — la production passe `BUILD_ID` en local.
///
/// Chaque fait rendu porte la machine sur laquelle il vaut : le build-id local
/// vaut sur `local_host`, celui du daemon sur `daemon_host`. Quand c'est le
/// CLIENT qui n'est pas identifiable, le daemon n'est pas mis en cause — c'était
/// l'attribution inversée mesurée sur toute machine fédérée, où `.git` absent
/// (rsync du déploiement) rend `BUILD_ID` = `unknown` et fait crier « daemon
/// périmé » à chaque commande.
pub fn stale_daemon_warning_for(
    local_build_id: &str,
    local_host: &str,
    daemon_build_id: &str,
    daemon_host: Option<&str>,
) -> Option<String> {
    (daemon_build_id != local_build_id).then(|| {
        // Incident fondateur (2026-08-23) : deux correctifs semblaient absents
        // pendant des heures parce qu'un daemon périmé continuait de répondre.
        if local_build_id == "unknown" {
            return format!(
                "client non identifiable sur {local_host} : compilé sans dépôt Git, \
                 build-id absent — le daemon ({daemon_build_id}) n'est pas mis en cause ; \
                 poser BRIDGET_BUILD_ID à la compilation"
            );
        }
        let machine = daemon_host.unwrap_or(MACHINE_NON_ATTESTEE);
        let remediation = remediation(local_host, daemon_host);
        if daemon_build_id == "unknown" {
            format!(
                "daemon build-id inconnu sur {machine} — client {local_build_id} \
                 sur {local_host} : {remediation}"
            )
        } else {
            format!(
                "daemon périmé sur {machine} ({daemon_build_id}) — client \
                 {local_build_id} sur {local_host} : {remediation}"
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Contrôle positif d'abord : identifiants égaux → silence. Sans lui, les
    /// assertions suivantes ne prouveraient pas que l'instrument sait se taire.
    #[test]
    fn ecart_de_build_id_observe_les_identifiants_pas_un_libelle() {
        let local = "build-local";
        assert!(stale_daemon_warning_for(local, "cartae", local, Some("cartae")).is_none());

        let warning = stale_daemon_warning_for(local, "cartae", "build-daemon", Some("cartae"))
            .expect("écart signalé");
        assert!(warning.contains("build-daemon"));
        assert!(warning.contains(local));

        // Surface production : même binaire → silence.
        assert!(stale_daemon_warning_at(BUILD_ID, Some(&local_host())).is_none());
        assert!(stale_daemon_warning(BUILD_ID).is_none());
    }

    /// POINT 3 — l'attribution inversée. Un client sans build-id ne doit PAS
    /// produire une affirmation sur le daemon.
    /// Mutant qui tue ce test : retirer la branche `local_build_id == "unknown"`
    /// → le message retombe sur « daemon périmé » et l'assertion suivante meurt.
    #[test]
    fn un_client_non_identifiable_n_accuse_pas_le_daemon() {
        let warning =
            stale_daemon_warning_for("unknown", "cartae", "24e8003", Some("monordinateur"))
                .expect("écart signalé");
        assert!(
            !warning.contains("daemon périmé"),
            "le client inconnu ne doit pas accuser le daemon : {warning}"
        );
        assert!(
            warning.contains("client non identifiable sur cartae"),
            "le fait doit porter la machine où il vaut : {warning}"
        );
        // Contrôle positif du même instrument : un client identifiable face au
        // même daemon rend bien, lui, un verdict de péremption.
        let vrai_ecart =
            stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
                .expect("écart signalé");
        assert!(vrai_ecart.contains("daemon périmé"));
    }

    /// POINT 4 — le fait est attaché à la machine où il vaut, pas à la mienne.
    /// Mutant qui tue ce test : rendre `machine` = `local_host` → l'assertion
    /// « daemon périmé sur monordinateur » meurt.
    #[test]
    fn le_verdict_nomme_la_machine_du_daemon_et_non_la_mienne() {
        let warning =
            stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
                .expect("écart signalé");
        assert!(
            warning.contains("daemon périmé sur monordinateur"),
            "le verdict doit être attaché à la machine du daemon : {warning}"
        );
        assert!(
            warning.contains("client client-neuf sur cartae"),
            "le build-id local doit être attaché à MA machine : {warning}"
        );
    }

    /// POINT 4 — un daemon distant ne se relance pas par une commande locale.
    /// Mutant qui tue ce test : retirer le bras `Some(host) if host != local_host`
    /// → la remédiation redevient une commande locale et l'assertion meurt.
    #[test]
    fn un_daemon_distant_ne_propose_aucune_commande_locale() {
        let warning =
            stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("monordinateur"))
                .expect("écart signalé");
        assert!(
            warning.contains("relancer le daemon sur monordinateur"),
            "la remédiation doit nommer la machine à traiter : {warning}"
        );
        assert!(
            !warning.contains("launchctl"),
            "commande locale : {warning}"
        );
        assert!(
            !warning.contains("systemctl"),
            "commande locale : {warning}"
        );
    }

    /// POINT 4 — la commande locale est celle de CETTE plateforme.
    /// Deux littéraux, un par plateforme : l'oracle ne recalcule pas la valeur
    /// avec le code de production, il la nomme.
    /// Mutant qui tue ce test : retirer le `cfg!(target_os = "macos")` → une des
    /// deux plateformes reçoit la commande de l'autre.
    #[test]
    fn la_remediation_locale_est_celle_de_la_plateforme() {
        let warning = stale_daemon_warning_for("client-neuf", "cartae", "24e8003", Some("cartae"))
            .expect("écart signalé");
        if cfg!(target_os = "macos") {
            assert!(warning.contains("launchctl kickstart -k gui/"), "{warning}");
            assert!(!warning.contains("systemctl"), "{warning}");
        } else {
            assert!(
                warning.contains("systemctl --user restart bridget-daemon"),
                "{warning}"
            );
            assert!(!warning.contains("launchctl"), "{warning}");
        }
    }

    /// Machine du daemon non attestée : le message le DIT au lieu de laisser
    /// croire qu'il parle de la machine locale.
    /// Mutant qui tue ce test : `unwrap_or(local_host)` → l'assertion meurt.
    #[test]
    fn une_machine_non_attestee_est_nommee_comme_telle() {
        let warning =
            stale_daemon_warning_for("client-neuf", "cartae", "24e8003", None).expect("écart");
        assert!(warning.contains(MACHINE_NON_ATTESTEE), "{warning}");
        assert!(
            !warning.contains("daemon périmé sur cartae"),
            "une machine inconnue ne doit pas être supposée locale : {warning}"
        );
    }

    /// La chaîne complète doit rendre UNE SEULE valeur.
    ///
    /// `wrapper::host_name` -> `build_info::local_host` -> `bridget_core::local_host`.
    /// CE QUE CET ORACLE PROUVE ET CE QU'IL NE PROUVE PAS, je le dis ici plutôt
    /// que de laisser croire : il constate l'égalité des valeurs rendues. Il ne
    /// PROTÈGE pas contre une copie fidèle. Ce qui protège, c'est que
    /// `build_info::local_host` est un `pub use` — il n'existe qu'un seul item,
    /// et le langage interdit qu'un second en diverge.
    #[test]
    fn les_deux_chemins_rendent_la_meme_machine() {
        assert_eq!(local_host(), bridget_core::local_host());
        assert_eq!(
            local_host(),
            bridget_core::host::local_host(),
            "le ré-export et le chemin complet désignent le même item"
        );
    }
}
