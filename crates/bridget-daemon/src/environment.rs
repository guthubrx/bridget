//! Espace d'état du noyau, indépendant du HOME utilisé par les fournisseurs.
//!
//! Résolution sans I/O d'écriture, puis création privée explicite. Aucune
//! migration implicite du produit historique et aucun repli sous /tmp.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    pub root: PathBuf,
    pub socket: PathBuf,
}

impl Namespace {
    /// Parcours des noms d'environnement puis O(profondeur du chemin), sans
    /// création ni connexion. Les valeurs du runtime retiré ne sont jamais lues.
    pub fn from_environment() -> Result<Self, String> {
        reject_removed_runtime_environment(std::env::vars_os().map(|(key, _)| key))?;
        Self::resolve(
            std::env::var_os("BRIDGET_HOME").map(PathBuf::from),
            std::env::var_os("BRIDGET_SOCKET").map(PathBuf::from),
            std::env::var_os("HOME").map(PathBuf::from),
        )
    }

    pub fn resolve(
        root: Option<PathBuf>,
        socket: Option<PathBuf>,
        provider_home: Option<PathBuf>,
    ) -> Result<Self, String> {
        let root = match root {
            Some(root) => root,
            None => provider_home
                .ok_or("HOME absent : fournir BRIDGET_HOME absolu, aucun repli temporaire")?
                .join(".cache/bridget-core"),
        };
        validate_path(&root)?;
        if root.parent().is_none() || root == Path::new("/") {
            return Err("BRIDGET_HOME doit désigner un répertoire privé dédié".into());
        }
        let socket = socket.unwrap_or_else(|| root.join("bridget.sock"));
        validate_path(&socket)?;
        if socket.parent() != Some(root.as_path()) {
            return Err("BRIDGET_SOCKET doit rester directement dans BRIDGET_HOME ; choisir une racine courte si nécessaire".into());
        }
        // SUN_LEN est plus strict sous macOS. La limite portable évite un
        // échec au bind après création de l'état durable.
        use std::os::unix::ffi::OsStrExt;
        if socket.as_os_str().as_bytes().len() >= 104 {
            return Err("socket trop longue : choisir un BRIDGET_HOME absolu plus court".into());
        }
        validate_private_directory_if_present(&root)?;
        for name in [
            "managed",
            "agent-names",
            "agent-pids",
            "sessions",
            "state",
            "artifacts",
            "tmp",
            "spawn-orders",
            "reaper",
        ] {
            validate_private_directory_if_present(&root.join(name))?;
        }
        validate_state_file(&socket, true)?;
        for name in [
            "bridget.db",
            "daemon.log",
            "agents.json",
            "bridget.fleet.json",
            "federation.env",
            "human-channel.json",
        ] {
            validate_state_file(&root.join(name), false)?;
        }
        validate_state_file(&socket.with_extension("pid"), false)?;
        Ok(Self { root, socket })
    }

    pub fn prepare(&self) -> Result<(), String> {
        ensure_private_directory(&self.root)?;
        ensure_private_directory(&self.root.join("tmp"))?;
        Ok(())
    }

    /// Namespace transmis aux processus enfants, jamais les credentials.
    pub fn child_environment(&self) -> [(OsString, OsString); 2] {
        [
            (
                OsString::from("BRIDGET_HOME"),
                self.root.as_os_str().to_owned(),
            ),
            (
                OsString::from("BRIDGET_SOCKET"),
                self.socket.as_os_str().to_owned(),
            ),
        ]
    }
}

/// La présence d'une clé suffit, indépendamment de sa valeur (vide, secrète ou
/// non UTF-8). Aucune liste de suffixes ne peut laisser passer une ancienne voie.
fn reject_removed_runtime_environment(
    keys: impl IntoIterator<Item = OsString>,
) -> Result<(), String> {
    use std::os::unix::ffi::OsStrExt;
    if let Some(key) = keys
        .into_iter()
        .find(|key| key.as_os_str().as_bytes().starts_with(b"BRIDGET_RUNTIME_"))
    {
        return Err(format!(
            "{} n'est pas pris en charge par le noyau communication ; utiliser BRIDGET_HOME/BRIDGET_SOCKET",
            key.to_string_lossy()
        ));
    }
    Ok(())
}

/// À appeler au point d'entrée CLI, avant tout thread. HOME n'est jamais
/// modifié : les CLI fournisseur conservent leurs abonnements et profils.
pub fn initialize_process() -> Result<(), String> {
    let namespace = Namespace::from_environment()?;
    namespace.prepare()?;
    for (key, value) in namespace.child_environment() {
        // Le CLI n'a pas encore démarré de thread ni de fournisseur.
        unsafe { std::env::set_var(key, value) };
    }
    // SQLite, fichiers d'état et sockets naissent privés ; aucun chmod
    // a posteriori ne rattrape une fenêtre d'exposition.
    unsafe { libc::umask(0o077) };
    Ok(())
}

/// Résolution explicite pour les API de wrapper qui acceptent un HOME de
/// fixture. En production BRIDGET_HOME, normalisé à l'entrée, fait autorité.
pub fn root_for_home(home: &Path) -> Result<PathBuf, String> {
    Namespace::resolve(
        std::env::var_os("BRIDGET_HOME").map(PathBuf::from),
        std::env::var_os("BRIDGET_SOCKET").map(PathBuf::from),
        Some(home.to_path_buf()),
    )
    .map(|namespace| namespace.root)
}

pub fn validate_path(path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(format!(
            "chemin d'état absolu sans '..' requis : {}",
            path.display()
        ));
    }
    let components: Vec<_> = path.components().map(|part| part.as_os_str()).collect();
    let historical = components
        .windows(2)
        .any(|pair| (pair[0] == ".cache" || pair[0] == ".config") && pair[1] == "bridget")
        || components.windows(3).any(|parts| {
            parts[0] == ".local"
                && (parts[1] == "share" || parts[1] == "state")
                && parts[2] == "bridget"
        })
        || components.windows(3).any(|parts| {
            parts[0] == "Library" && parts[1] == "Application Support" && parts[2] == "bridget"
        })
        || path == Path::new("/tmp/bridget")
        || path.starts_with("/tmp/bridget/")
        || path == Path::new("/tmp/bridget.sock")
        || path == Path::new("/private/tmp/bridget")
        || path.starts_with("/private/tmp/bridget/");
    if historical {
        return Err(format!(
            "namespace historique interdit : {}",
            path.display()
        ));
    }
    // Refuser les liens contrôlés par l'utilisateur à chaque composant ;
    // /tmp et /var sont les alias système standards de macOS.
    let mut ancestor = PathBuf::new();
    for component in path.components() {
        ancestor.push(component.as_os_str());
        match fs::symlink_metadata(&ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let system_alias = cfg!(target_os = "macos")
                    && (ancestor == Path::new("/tmp") || ancestor == Path::new("/var"))
                    && metadata.uid() == 0;
                if !system_alias {
                    return Err(format!("symlink d'état interdit : {}", ancestor.display()));
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => break,
            Err(error) => return Err(format!("inspection {} : {error}", ancestor.display())),
        }
    }
    Ok(())
}

fn is_private_owned(metadata: &fs::Metadata, expected_uid: u32) -> bool {
    metadata.uid() == expected_uid && metadata.permissions().mode() & 0o077 == 0
}

pub fn validate_private_directory_if_present(path: &Path) -> Result<(), String> {
    validate_path(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if !metadata.is_dir() || !is_private_owned(&metadata, unsafe { libc::geteuid() }) {
                return Err(format!(
                    "répertoire privé 0700 appartenant à l'utilisateur requis : {}",
                    path.display()
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("inspection {} : {error}", path.display())),
    }
}

pub fn ensure_private_directory(path: &Path) -> Result<(), String> {
    validate_private_directory_if_present(path)?;
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|error| format!("création privée {} : {error}", path.display()))?;
    validate_private_directory_if_present(path)
}

/// Préflight des états existants AVANT tout bootstrap/reconcile : une racine
/// privée ne protège pas un marqueur ou un sous-dossier qui est un lien vers
/// une autre instance. O(N) métadonnées, borné ; aucun contenu n'est interprété.
pub fn validate_existing_tree(root: &Path) -> Result<(), String> {
    validate_private_directory_if_present(root)?;
    if !root.exists() {
        return Ok(());
    }
    let mut directories = vec![(root.to_path_buf(), 0usize)];
    let mut entries = 0usize;
    while let Some((directory, depth)) = directories.pop() {
        if depth > 64 {
            return Err("namespace trop profond pour être validé".into());
        }
        let children = match fs::read_dir(&directory) {
            Ok(children) => children,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("inspection {} : {error}", directory.display())),
        };
        for entry in children {
            let entry = entry.map_err(|e| format!("inspection namespace : {e}"))?;
            entries += 1;
            if entries > 100_000 {
                return Err("namespace trop volumineux pour être validé".into());
            }
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                // Les temporaires peuvent être retirés par leur propriétaire
                // pendant la lecture ; leur absence n'autorise aucun accès.
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(format!("inspection {} : {error}", path.display())),
            };
            if metadata.file_type().is_symlink() {
                return Err(format!("symlink d'état interdit : {}", path.display()));
            }
            if !is_private_owned(&metadata, unsafe { libc::geteuid() }) {
                return Err(format!(
                    "état non privé dans le namespace : {}",
                    path.display()
                ));
            }
            if metadata.is_dir() {
                directories.push((path, depth + 1));
            } else if !metadata.is_file() && !metadata.file_type().is_socket() {
                return Err(format!("type d'état interdit : {}", path.display()));
            }
        }
    }
    Ok(())
}

pub fn validate_state_file(path: &Path, socket: bool) -> Result<(), String> {
    validate_path(path)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let valid_type = if socket {
                metadata.file_type().is_socket()
            } else {
                metadata.is_file()
            };
            if !valid_type || !is_private_owned(&metadata, unsafe { libc::geteuid() }) {
                return Err(format!(
                    "état privé de type/propriétaire valide requis : {}",
                    path.display()
                ));
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("inspection {} : {error}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::reject_removed_runtime_environment;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn fichier_prive_ne_suffit_pas_si_le_proprietaire_attendu_differe() {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let path = std::env::temp_dir().join(format!("owner-{}.test", uuid::Uuid::new_v4()));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let metadata = file.metadata().unwrap();
        let actual = unsafe { libc::geteuid() };
        assert_eq!(metadata.uid(), actual);
        assert!(super::is_private_owned(&metadata, actual));
        // Métadonnées d'un VRAI fichier 0600 ; identité attendue injectée,
        // aucun chown privilégié ni altération d'un fichier d'un autre compte.
        // Retirer la comparaison UID rend cette assertion rouge.
        assert!(!super::is_private_owned(&metadata, actual ^ 1));
        drop(file);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn environnement_sans_prefixe_runtime_reste_accepte() {
        assert!(reject_removed_runtime_environment([]).is_ok());
        assert!(
            reject_removed_runtime_environment(
                [
                    "HOME",
                    "BRIDGET_HOME",
                    "BRIDGET_RUNTIME",
                    "XBRIDGET_RUNTIME_SOCKET"
                ]
                .into_iter()
                .map(OsString::from)
            )
            .is_ok()
        );
    }

    #[test]
    fn toute_cle_runtime_est_refusee_sans_dependre_de_la_valeur() {
        for value in [OsString::new(), OsString::from_vec(b"secret-\xff".to_vec())] {
            let variables = [(OsString::from("BRIDGET_RUNTIME_FUTURE"), value)];
            let error =
                reject_removed_runtime_environment(variables.into_iter().map(|(key, _)| key))
                    .unwrap_err();
            assert_eq!(
                error,
                "BRIDGET_RUNTIME_FUTURE n'est pas pris en charge par le noyau communication ; utiliser BRIDGET_HOME/BRIDGET_SOCKET"
            );
            assert!(!error.contains("secret"));
        }
    }

    #[test]
    fn prefixe_runtime_est_reconnu_meme_avec_suffixe_non_utf8_ou_vide() {
        for key in [
            OsString::from("BRIDGET_RUNTIME_"),
            OsString::from_vec(b"BRIDGET_RUNTIME_\xff".to_vec()),
        ] {
            assert!(reject_removed_runtime_environment([key]).is_err());
        }
    }
}
