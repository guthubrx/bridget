//! Primitives filesystem privées réutilisables par les composants Bridget.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// Frontières observables d'un remplacement atomique.
///
/// L'observateur sert aux tests de crash par processus réel ; l'écriture de
/// production utilise la même fonction avec un observateur vide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicWritePhase {
    BeforeRename,
    AfterRename,
}

/// Crée un répertoire d'état privé (0700), partagé avec le store de reçus 012.
pub fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

/// Ouvre un fichier privé (0600) pour un état durable nécessitant un verrou.
pub fn open_private_file(path: &Path) -> io::Result<File> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("fichier sans parent"))?;
    create_private_dir(parent)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

/// Remplace atomiquement un fichier privé (0600) après synchronisation disque.
///
/// Un échec avant le renommage nettoie le temporaire. Un échec après le
/// renommage est propagé : l'appelant doit alors traiter l'issue comme
/// indéterminée plutôt que supposer une écriture réussie.
pub fn write_private_file_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_private_file_atomic_observed(path, bytes, |_| Ok(()))
}

/// Même remplacement durable avec observation déterministe des frontières.
///
/// L'observateur ne décide jamais du résultat métier et n'injecte aucune
/// erreur : il permet uniquement à un processus de test de se bloquer à une
/// frontière réelle, puis d'être terminé par son parent.
pub fn write_private_file_atomic_observed(
    path: &Path,
    bytes: &[u8],
    mut observer: impl FnMut(AtomicWritePhase) -> io::Result<()>,
) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("fichier sans parent"))?;
    create_private_dir(parent)?;
    let suffix = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        suffix
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
        observer(AtomicWritePhase::BeforeRename)?;
        fs::rename(&temporary, path)?;
        observer(AtomicWritePhase::AfterRename)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn private_helpers_enforce_permissions_and_replace_content() {
        let root = std::env::temp_dir().join(format!("bridget-fsutil-{}", std::process::id()));
        let path = root.join("entry");
        let _ = fs::remove_dir_all(&root);
        write_private_file_atomic(&path, b"first").unwrap();
        write_private_file_atomic(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(
            fs::metadata(&root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let _ = fs::remove_dir_all(root);
    }
}
