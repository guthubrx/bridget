//! Exécutable compagnon Maicie.
//!
//! Maicie demeure un processus séparé de Bridget. Sa connexion publique au
//! daemon sera possédée par `BridgetClient`, afin que le binaire ne charge
//! jamais de code interne du daemon.

/// Identité stable réservée au compagnon d'orchestration.
pub const MAICIE_IDENTITY: &str = "maicie";

fn main() {
    println!("Maicie prêt : identité {MAICIE_IDENTITY}");
}

#[cfg(test)]
mod tests {
    use super::MAICIE_IDENTITY;

    #[test]
    fn identite_du_compagnon_est_stable() {
        assert_eq!(MAICIE_IDENTITY, "maicie");
    }
}
