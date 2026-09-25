//! Nom de la machine courante — **source unique** du projet.
//!
//! Elle vit ici, dans le socle commun, et non dans le daemon : le greffe
//! (`plugins/service-compagnon`) ne dépend que de `bridget-core` et `bridget-transport`,
//! jamais de `bridget-daemon`. L'y laisser aurait obligé chaque appelant hors
//! daemon à recopier ces quelques lignes — et deux calculs libres de diverger
//! du nom de machine, c'est exactement la maladie que ce chantier soigne.
//!
//! Une seule vérité, donc, pour l'annuaire, les refus de lancement, le contrôle
//! de péremption, et les gardes qui décident **sur quelle machine** on écrit.

/// Valeur rendue quand la machine n'a PAS pu être déterminée.
///
/// Ce n'est PAS une identité, c'est un repli d'affichage. Deux machines
/// différentes qui échouent toutes deux à se nommer rendent cette MÊME chaîne :
/// la comparer par égalité les déclarerait identiques. Toute garde qui décide
/// « même machine » doit donc passer par [`host_is_attested`] avant de comparer.
pub const HOTE_NON_ATTESTE: &str = "inconnu";

/// Ce nom désigne-t-il vraiment une machine ?
///
/// Prédicat PARTAGÉ, exposé pour que personne n'ait à figer la chaîne de repli
/// dans son propre code — deux écritures de la même sentinelle seraient deux
/// vérités libres de diverger, et c'est exactement ce que ce module existe pour
/// empêcher.
pub fn host_is_attested(host: &str) -> bool {
    let host = host.trim();
    !host.is_empty() && host != HOTE_NON_ATTESTE
}

/// Nom de la machine courante.
///
/// `HOSTNAME` s'il est renseigné et non vide, sinon la commande `hostname`,
/// sinon [`HOTE_NON_ATTESTE`] — jamais une valeur inventée ni un `Option` que
/// l'appelant traiterait comme « local par défaut ».
pub fn local_host() -> String {
    if let Ok(host) = std::env::var("HOSTNAME")
        && !host.trim().is_empty()
    {
        return host.trim().to_string();
    }
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|host| host.trim().to_string())
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| HOTE_NON_ATTESTE.to_string())
}

#[cfg(test)]
mod tests {
    use super::{HOTE_NON_ATTESTE, host_is_attested, local_host};

    /// La valeur rendue est toujours utilisable comme désignation de machine :
    /// non vide, sans espace de bordure, et jamais un `Option` déguisé.
    ///
    /// Mutant qui tue ce test : rendre `String::new()` quand `hostname` échoue
    /// au lieu de `inconnu` → la première assertion meurt.
    #[test]
    fn le_nom_de_machine_est_toujours_une_designation_utilisable() {
        let hote = local_host();
        assert!(
            !hote.is_empty(),
            "une machine sans nom doit se dire « inconnu », jamais rien"
        );
        assert_eq!(hote.trim(), hote, "aucun espace de bordure : {hote:?}");
        // Stabilité : deux lectures consécutives dans le même environnement
        // doivent concorder, sinon aucune garde bâtie dessus ne tient.
        assert_eq!(hote, local_host());
    }

    /// UNE ABSENCE N'EST PAS UNE IDENTITÉ, et deux absences ne sont pas égales.
    ///
    /// Signalé par rc7 : deux machines qui échouent toutes deux à se nommer
    /// rendent la même chaîne. Une garde qui comparerait par égalité les
    /// déclarerait identiques — et avec le chemin de base standard, identique
    /// partout, elle autoriserait une écriture du mauvais côté.
    ///
    /// Mutant qui tue ce test : faire rendre `true` à `host_is_attested` pour
    /// la sentinelle → la première assertion meurt.
    #[test]
    fn un_nom_de_repli_n_atteste_aucune_machine() {
        assert!(
            !host_is_attested(HOTE_NON_ATTESTE),
            "le repli désigne une absence, jamais une machine"
        );
        assert!(!host_is_attested(""), "le vide n'atteste rien non plus");
        assert!(!host_is_attested("   "), "ni les espaces");
        // Contrôle positif : un vrai nom est bien attesté. Sans lui, un
        // prédicat qui refuserait TOUT passerait les assertions ci-dessus.
        assert!(host_is_attested("poste-beta"));
        assert!(host_is_attested("poste-alpha"));
    }
}
