//! Primitives filesystem privées réutilisables par les composants Bridget.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

/// Crée un répertoire d'état privé (0700), partagé avec le store de reçus 012.
pub fn create_private_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

/// Remplace atomiquement un fichier privé (0600) après synchronisation disque.
///
/// Un échec avant le renommage nettoie le temporaire. Un échec après le
/// renommage est propagé : l'appelant doit alors traiter l'issue comme
/// indéterminée plutôt que supposer une écriture réussie.
pub fn write_private_file_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("fichier sans parent"))?;
    create_private_dir(parent)?;
    let suffix = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
    let temporary = parent.join(format!(".{}.{}.tmp", path.file_name().unwrap_or_default().to_string_lossy(), suffix));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600))?;
        fs::rename(&temporary, path)?;
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
        assert_eq!(fs::metadata(&root).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        let _ = fs::remove_dir_all(root);
    }
}
