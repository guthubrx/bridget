//! Nom de la machine courante — **source unique** du projet.
//!
//! Elle vit ici, dans le socle commun, et non dans le daemon : le greffe
//! (`plugins/maicie`) ne dépend que de `bridget-core` et `bridget-transport`,
//! jamais de `bridget-daemon`. L'y laisser aurait obligé chaque appelant hors
//! daemon à recopier ces quelques lignes — et deux calculs libres de diverger
//! du nom de machine, c'est exactement la maladie que ce chantier soigne.
//!
//! Une seule vérité, donc, pour l'annuaire, les refus de lancement, le contrôle
//! de péremption, et les gardes qui décident **sur quelle machine** on écrit.

/// Nom de la machine courante.
///
/// `HOSTNAME` s'il est renseigné et non vide, sinon la commande `hostname`,
/// sinon la chaîne `inconnu` — jamais une valeur inventée ni un `Option` que
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
        .unwrap_or_else(|| "inconnu".to_string())
}

#[cfg(test)]
mod tests {
    use super::local_host;

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
}
