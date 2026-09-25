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
    // Le PID sépare les écrivains inter-processus ; le suffixe `.tmp` reste
    // stable afin que les réconciliations existantes ignorent aussi bien les
    // anciens `.N.tmp` que les nouveaux `.pid-N.tmp` laissés par un crash.
    let temporary = parent.join(format!(
        ".{}.{}-{}.tmp",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
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
    use std::process::Command;
    use std::thread;
    use std::time::{Duration, Instant};

    const CONCURRENT_CHILD: &str = "BRIDGET_FSUTIL_CONCURRENT_CHILD";
    const CONCURRENT_PATH: &str = "BRIDGET_FSUTIL_CONCURRENT_PATH";
    const CONCURRENT_READY: &str = "BRIDGET_FSUTIL_CONCURRENT_READY";
    const CONCURRENT_RELEASE: &str = "BRIDGET_FSUTIL_CONCURRENT_RELEASE";
    const CONCURRENT_CONTENT: &str = "BRIDGET_FSUTIL_CONCURRENT_CONTENT";

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

    #[test]
    fn atomic_temporary_names_are_unique_across_processes() {
        if std::env::var_os(CONCURRENT_CHILD).is_some() {
            concurrent_writer_child();
            return;
        }

        let root =
            std::env::temp_dir().join(format!("bridget-fsutil-concurrent-{}", std::process::id()));
        let path = root.join("fleet.json");
        let _ = fs::remove_dir_all(&root);
        create_private_dir(&root).unwrap();

        let current_exe = std::env::current_exe().unwrap();
        let mut children = Vec::new();
        let mut ready_paths = Vec::new();
        let mut release_paths = Vec::new();
        for index in 0..2 {
            let ready = root.join(format!("ready-{index}"));
            let release = root.join(format!("release-{index}"));
            let child = Command::new(&current_exe)
                .arg("--exact")
                .arg("fsutil::tests::atomic_temporary_names_are_unique_across_processes")
                .arg("--nocapture")
                .env(CONCURRENT_CHILD, "1")
                .env(CONCURRENT_PATH, &path)
                .env(CONCURRENT_READY, &ready)
                .env(CONCURRENT_RELEASE, &release)
                .env(CONCURRENT_CONTENT, format!("writer-{index}"))
                .spawn()
                .unwrap();
            children.push(child);
            ready_paths.push(ready);
            release_paths.push(release);
        }

        for ready in &ready_paths {
            wait_until(
                || ready.exists(),
                "temporaire ouvert par les deux processus",
            );
        }
        let temporaries = fs::read_dir(&root)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".tmp"))
            .collect::<Vec<_>>();
        assert_eq!(temporaries.len(), 2);
        assert!(
            temporaries
                .iter()
                .all(|name| name.starts_with(".fleet.json."))
        );
        assert!(temporaries.iter().all(|name| {
            name.rsplit_once('-')
                .is_some_and(|(_, counter)| counter == "0.tmp")
        }));

        for release in &release_paths {
            fs::write(release, b"release").unwrap();
        }
        for mut child in children {
            assert!(child.wait().unwrap().success());
        }

        assert!(matches!(
            fs::read(&path).unwrap().as_slice(),
            b"writer-0" | b"writer-1"
        ));
        assert!(
            fs::read_dir(&root)
                .unwrap()
                .filter_map(Result::ok)
                .all(|entry| !entry.file_name().to_string_lossy().ends_with(".tmp"))
        );
        let _ = fs::remove_dir_all(root);
    }

    fn concurrent_writer_child() {
        let path = std::env::var_os(CONCURRENT_PATH).unwrap();
        let ready = std::env::var_os(CONCURRENT_READY).unwrap();
        let release = std::env::var_os(CONCURRENT_RELEASE).unwrap();
        let content = std::env::var(CONCURRENT_CONTENT).unwrap();
        write_private_file_atomic_observed(Path::new(&path), content.as_bytes(), |phase| {
            if phase == AtomicWritePhase::BeforeRename {
                fs::write(&ready, b"ready")?;
                wait_until(
                    || Path::new(&release).exists(),
                    "liberation du processus enfant",
                );
            }
            Ok(())
        })
        .unwrap();
    }

    fn wait_until(mut predicate: impl FnMut() -> bool, label: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !predicate() {
            assert!(Instant::now() < deadline, "timeout : {label}");
            thread::sleep(Duration::from_millis(5));
        }
    }
}
