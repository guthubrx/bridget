/// Identifiant du commit embarqué dans le binaire au moment de sa compilation.
pub const BUILD_ID: &str = env!("BRIDGET_BUILD_ID");

/// Désignation employée quand la machine du daemon n'est pas attestée.
pub const MACHINE_NON_ATTESTEE: &str = "machine non attestée";

/// Rendu HUMAIN d'un hôte, quelle que soit la forme de son absence.
///
/// Il y avait DEUX vocabulaires pour le même fait : un champ absent devenait
/// « machine non attestée », mais un champ portant la valeur de repli
/// s'affichait « inconnu ». Deux littéraux libres pour « on ne sait pas quelle
/// machine » — donc un code qui reconnaît l'un ne reconnaît pas l'autre.
/// Ce sont bien LE MÊME FAIT, et ils se rendent désormais pareil.
///
/// Ne pas confondre avec les autres « inconnu » du dépôt — build-id, système
/// d'exploitation, nom d'outil : ceux-là désignent d'AUTRES faits et ne doivent
/// surtout pas être unifiés avec celui-ci.
pub fn describe_host(host: Option<&str>) -> &str {
    match host {
        Some(host) if bridget_core::host_is_attested(host) => host,
        _ => MACHINE_NON_ATTESTEE,
    }
}

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
    // AUCUNE COMMANDE LOCALE SANS MACHINE ATTESTEE. Si les deux côtés portent
    // la valeur de repli, l'égalité ne prouve rien : proposer `launchctl` ou
    // `systemctl` enverrait l'exploitant relancer un service SUR SA MACHINE
    // alors que le daemon est peut-être ailleurs. Il exécuterait la commande,
    // rien d'utile ne se produirait, et rien ne lui dirait pourquoi.
    // C'est le seul des trois points qui produit un GESTE, pas un verdict.
    if !bridget_core::host_is_attested(local_host)
        || !daemon_host.is_some_and(bridget_core::host_is_attested)
    {
        return format!(
            "machine du daemon non attestée — on ne sait pas où relancer ; \
             renseigner HOSTNAME de part et d'autre avant toute relance ({})",
            describe_host(daemon_host)
        );
    }
    match daemon_host {
        Some(host) if host != local_host => {
            format!("relancer le daemon sur {host} — aucune commande locale ne l'atteint")
        }
        // Le noyau extrait n'installe aucun service global. Recommander le
        // label historique agirait sur une autre installation de Bridget.
        _ => "relancer manuellement le daemon Bridget communication du namespace BRIDGET_HOME/BRIDGET_SOCKET vérifié — ne pas relancer le service historique".to_string(),
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
        let ici = describe_host(Some(local_host));
        if local_build_id == "unknown" {
            return format!(
                "client non identifiable sur {ici} : compilé sans dépôt Git, \
                 build-id absent — le daemon ({daemon_build_id}) n'est pas mis en cause ; \
                 poser BRIDGET_BUILD_ID à la compilation"
            );
        }
        let machine = describe_host(daemon_host);
        let remediation = remediation(local_host, daemon_host);
        if daemon_build_id == "unknown" {
            format!(
                "daemon build-id inconnu sur {machine} — client {local_build_id} \
                 sur {ici} : {remediation}"
            )
        } else {
            format!(
                "daemon périmé sur {machine} ({daemon_build_id}) — client \
                 {local_build_id} sur {ici} : {remediation}"
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
        assert!(stale_daemon_warning_for(local, "poste-beta", local, Some("poste-beta")).is_none());

        let warning =
            stale_daemon_warning_for(local, "poste-beta", "build-daemon", Some("poste-beta"))
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
            stale_daemon_warning_for("unknown", "poste-beta", "24e8003", Some("poste-alpha"))
                .expect("écart signalé");
        assert!(
            !warning.contains("daemon périmé"),
            "le client inconnu ne doit pas accuser le daemon : {warning}"
        );
        assert!(
            warning.contains("client non identifiable sur poste-beta"),
            "le fait doit porter la machine où il vaut : {warning}"
        );
        // Contrôle positif du même instrument : un client identifiable face au
        // même daemon rend bien, lui, un verdict de péremption.
        let vrai_ecart =
            stale_daemon_warning_for("client-neuf", "poste-beta", "24e8003", Some("poste-alpha"))
                .expect("écart signalé");
        assert!(vrai_ecart.contains("daemon périmé"));
    }

    /// POINT 4 — le fait est attaché à la machine où il vaut, pas à la mienne.
    /// Mutant qui tue ce test : rendre `machine` = `local_host` → l'assertion
    /// « daemon périmé sur poste-alpha » meurt.
    #[test]
    fn le_verdict_nomme_la_machine_du_daemon_et_non_la_mienne() {
        let warning =
            stale_daemon_warning_for("client-neuf", "poste-beta", "24e8003", Some("poste-alpha"))
                .expect("écart signalé");
        assert!(
            warning.contains("daemon périmé sur poste-alpha"),
            "le verdict doit être attaché à la machine du daemon : {warning}"
        );
        assert!(
            warning.contains("client client-neuf sur poste-beta"),
            "le build-id local doit être attaché à MA machine : {warning}"
        );
    }

    /// POINT 4 — un daemon distant ne se relance pas par une commande locale.
    /// Mutant qui tue ce test : retirer le bras `Some(host) if host != local_host`
    /// → la remédiation redevient une commande locale et l'assertion meurt.
    #[test]
    fn un_daemon_distant_ne_propose_aucune_commande_locale() {
        let warning =
            stale_daemon_warning_for("client-neuf", "poste-beta", "24e8003", Some("poste-alpha"))
                .expect("écart signalé");
        assert!(
            warning.contains("relancer le daemon sur poste-alpha"),
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

    /// Une remédiation du noyau indépendant ne doit jamais cibler le service
    /// installé par l'ancien produit, quelle que soit la plateforme.
    #[test]
    fn la_remediation_locale_reste_dans_le_namespace_independant() {
        let warning =
            stale_daemon_warning_for("client-neuf", "poste-beta", "24e8003", Some("poste-beta"))
                .expect("écart signalé");
        assert!(
            warning.contains("namespace BRIDGET_HOME/BRIDGET_SOCKET vérifié"),
            "{warning}"
        );
        assert!(
            warning.contains("ne pas relancer le service historique"),
            "{warning}"
        );
        assert!(!warning.contains("launchctl"), "{warning}");
        assert!(!warning.contains("systemctl"), "{warning}");
        assert!(!warning.contains("com.bridget.daemon"), "{warning}");
    }

    /// Machine du daemon non attestée : le message le DIT au lieu de laisser
    /// croire qu'il parle de la machine locale.
    /// Mutant qui tue ce test : `unwrap_or(local_host)` → l'assertion meurt.
    #[test]
    fn une_machine_non_attestee_est_nommee_comme_telle() {
        let warning =
            stale_daemon_warning_for("client-neuf", "poste-beta", "24e8003", None).expect("écart");
        assert!(warning.contains(MACHINE_NON_ATTESTEE), "{warning}");
        assert!(
            !warning.contains("daemon périmé sur poste-beta"),
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

    /// QUATRIEME ORACLE — le seul des trois points qui produit un GESTE.
    ///
    /// Quand aucune machine n'est attestee, `remediation` proposait une commande
    /// LOCALE parce que les deux cotes portaient la meme valeur de repli. On
    /// envoyait l'exploitant relancer un service sur SA machine alors que le
    /// daemon est peut-etre ailleurs : il execute, rien ne se passe, et rien ne
    /// lui dit pourquoi.
    ///
    /// Mutant qui tue ce test : retirer la garde d'attestation en tete de
    /// `remediation` -> une commande locale reapparait et l'assertion meurt en
    /// affichant le message complet.
    #[test]
    fn aucune_commande_locale_quand_la_machine_n_est_pas_attestee() {
        let sentinelle = bridget_core::HOTE_NON_ATTESTE;
        let warning = stale_daemon_warning_for("client", sentinelle, "daemon", Some(sentinelle))
            .expect("écart signalé");
        assert!(
            !warning.contains("launchctl") && !warning.contains("systemctl"),
            "aucune commande locale ne doit être proposée : {warning}"
        );
        assert!(
            warning.contains("on ne sait pas où relancer"),
            "le message doit dire qu'on ignore où agir : {warning}"
        );
        // L'hôte LOCAL aussi passe par le rendu unique : sans cela le message
        // disait « sur inconnu » d'un côté et « machine non attestée » de
        // l'autre — les deux vocabulaires, dans la MÊME phrase.
        assert!(
            !warning.contains(&format!("sur {sentinelle}")),
            "la machine locale non attestée doit se lire comme telle : {warning}"
        );

        // Cas mixte : un seul côté attesté ne suffit pas non plus.
        let mixte = stale_daemon_warning_for("client", "poste-beta", "daemon", Some(sentinelle))
            .expect("écart signalé");
        assert!(
            !mixte.contains("systemctl") && !mixte.contains("launchctl"),
            "{mixte}"
        );

        // CONTRÔLE POSITIF : deux machines attestées et identiques → une
        // relance manuelle du namespace local, jamais du service historique.
        let local = stale_daemon_warning_for("client", "poste-beta", "daemon", Some("poste-beta"))
            .expect("écart signalé");
        assert!(
            local.contains("relancer manuellement le daemon Bridget communication"),
            "une machine attestée et locale doit recevoir sa remédiation : {local}"
        );
    }

    /// Les DEUX formes de l'absence se rendent PAREIL.
    ///
    /// Mutant qui tue ce test : faire rendre `host` tel quel par `describe_host`
    /// pour la sentinelle -> la première assertion meurt en affichant « inconnu ».
    #[test]
    fn les_deux_formes_de_l_absence_se_rendent_pareil() {
        assert_eq!(
            describe_host(Some(bridget_core::HOTE_NON_ATTESTE)),
            MACHINE_NON_ATTESTEE,
            "un champ portant le repli doit se lire comme un champ absent"
        );
        assert_eq!(describe_host(None), MACHINE_NON_ATTESTEE);
        assert_eq!(describe_host(Some("   ")), MACHINE_NON_ATTESTEE);
        // Contrôle positif : un vrai nom passe intact.
        assert_eq!(describe_host(Some("poste-beta")), "poste-beta");
    }
}
