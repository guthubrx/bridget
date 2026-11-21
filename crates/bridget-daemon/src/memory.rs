//! Réglage et restitution mémoire de l'allocateur système sous Linux glibc.

/// Nombre maximal d'arènes glibc par défaut lorsque `MALLOC_ARENA_MAX` n'est pas fixé.
/// Deux arènes suffisent pour le démon tout en évitant la multiplication d'arènes
/// par fil de connexion sur les serveurs multi-cœurs.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub const DEFAULT_LINUX_MALLOC_ARENA_MAX: libc::c_int = 2;

/// Borne le nombre d'arènes de la glibc au démarrage du processus, avant tout
/// lancement de fil d'exécution secondaire. Respecte `MALLOC_ARENA_MAX` s'il est
/// déjà défini dans l'environnement.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub fn configure_allocator() {
    if std::env::var_os("MALLOC_ARENA_MAX").is_some() {
        return;
    }
    // SAFETY: `mallopt(M_ARENA_MAX, ...)` est appelé au début de `main` avant la
    // création de fils secondaires ; ses arguments sont deux entiers valides de la glibc.
    unsafe {
        let _ = libc::mallopt(libc::M_ARENA_MAX, DEFAULT_LINUX_MALLOC_ARENA_MAX);
    }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub fn configure_allocator() {}

/// Demande à la glibc de restituer au noyau Linux les pages libres retenues dans
/// les arènes du tas. Retourne `true` si de la mémoire a effectivement été rendue.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub fn trim_allocator_memory() -> bool {
    // SAFETY: `malloc_trim(0)` est sûr en contexte multi-fils dans la glibc et ne
    // déréférence aucun pointeur.
    unsafe { libc::malloc_trim(0) != 0 }
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub fn trim_allocator_memory() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_et_trim_ne_paniquent_pas() {
        configure_allocator();
        let _ = trim_allocator_memory();
    }
}
