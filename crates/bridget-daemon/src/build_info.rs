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
            "machine du daemon non attestée — on ne sait pas où vérifier les versions ; \
             renseigner HOSTNAME de part et d'autre avant toute relance ({})",
            describe_host(daemon_host)
        );
    }
    match daemon_host {
        Some(host) if host != local_host => {
            format!("vérifier les versions actives sur {host} avec bridget updates status ; un écart de build ne donne aucun ordre de version")
        }
        // Le noyau extrait n'installe aucun service global. Recommander le
        // label historique agirait sur une autre installation de Bridget.
        _ => "consulter bridget updates status dans le namespace BRIDGET_HOME/BRIDGET_SOCKET vérifié ; un écart de build ne prouve pas une version ancienne ; ne pas relancer le service historique".to_string(),
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
                "build du daemon différent sur {machine} ({daemon_build_id}) — client \
                 {local_build_id} sur {ici} : {remediation}"
            )
        }
    })
}

#[cfg(test)]
#[path = "build_info_tests.rs"]
mod tests;
