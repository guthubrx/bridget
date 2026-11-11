//! Récupération d'un verrou empoisonné avec trace et compteur (programme 260, lot 7).
//!
//! Le comportement historique est conservé : la garde est rendue même si un autre fil a paniqué
//! en tenant le verrou, et le service continue. On ajoute seulement un compteur et une trace.
use std::sync::PoisonError;
use std::sync::atomic::{AtomicU64, Ordering};

static POISON_RECOVERIES: AtomicU64 = AtomicU64::new(0);

/// Nombre de verrous empoisonnés récupérés depuis le démarrage du processus.
pub fn poison_recoveries() -> u64 {
    POISON_RECOVERIES.load(Ordering::Relaxed)
}

pub trait RecoverPoison<G> {
    /// Retourne la garde même si le verrou est empoisonné (comportement historique), et consigne le fait.
    fn recover_poison(self) -> G;
}

impl<G> RecoverPoison<G> for Result<G, PoisonError<G>> {
    fn recover_poison(self) -> G {
        self.unwrap_or_else(|poisoned| {
            let n = POISON_RECOVERIES.fetch_add(1, Ordering::Relaxed) + 1;
            if n.is_power_of_two() {
                log::warn!("verrou empoisonné récupéré ({n} fois depuis le démarrage)");
            }
            poisoned.into_inner()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn un_verrou_sain_rend_sa_valeur_sans_changer_le_compteur() {
        let avant = poison_recoveries();
        let verrou = Mutex::new(7_u32);
        assert_eq!(*verrou.lock().recover_poison(), 7);
        assert_eq!(poison_recoveries(), avant);
    }

    #[test]
    fn un_verrou_empoisonne_rend_sa_valeur_et_incremente_le_compteur() {
        let avant = poison_recoveries();
        let verrou = Arc::new(Mutex::new(9_u32));
        let autre = Arc::clone(&verrou);
        let _ = std::thread::spawn(move || {
            let _garde = autre.lock().unwrap();
            panic!("empoisonnement volontaire du test");
        })
        .join();
        assert!(verrou.is_poisoned());
        assert_eq!(*verrou.lock().recover_poison(), 9);
        assert!(poison_recoveries() > avant);
    }
}
