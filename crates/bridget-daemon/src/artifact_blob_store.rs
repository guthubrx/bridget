//! Magasin durable de blobs canoniques, adressés par SHA-256.
//!
//! La racine appartient à l'instance Bridget qui publie. Elle n'est ni le
//! cache de consultation du Desktop, ni une surface de fichiers arbitraires.

use crate::artifact_policy::{ArtifactPolicy, ArtifactPolicyError};
use crate::artifact_types::sha256_hex;
use bridget_transport::fsutil::{create_private_dir, write_private_file_atomic};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ArtifactBlobStoreError {
    InvalidDigest,
    Missing,
    Corrupt { expected: String, actual: String },
    Policy(ArtifactPolicyError),
    Io(io::Error),
}

impl std::fmt::Display for ArtifactBlobStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDigest => formatter.write_str("empreinte de blob invalide"),
            Self::Missing => formatter.write_str("blob canonique absent"),
            Self::Corrupt { expected, actual } => {
                write!(
                    formatter,
                    "blob canonique corrompu: attendu {expected}, obtenu {actual}"
                )
            }
            Self::Policy(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "magasin de blobs: {error}"),
        }
    }
}

impl std::error::Error for ArtifactBlobStoreError {}

impl From<io::Error> for ArtifactBlobStoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ArtifactPolicyError> for ArtifactBlobStoreError {
    fn from(error: ArtifactPolicyError) -> Self {
        Self::Policy(error)
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactBlobStore {
    root: PathBuf,
    policy: ArtifactPolicy,
}

impl ArtifactBlobStore {
    pub fn open(
        root: impl AsRef<Path>,
        policy: ArtifactPolicy,
    ) -> Result<Self, ArtifactBlobStoreError> {
        let root = root.as_ref().to_path_buf();
        create_private_dir(&root)?;
        create_private_dir(&root.join("blobs"))?;
        Ok(Self { root, policy })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Persiste les octets après contrôle de taille et retourne leur référence
    /// stable. Deux écritures des mêmes octets partagent le même fichier.
    pub fn put(&self, bytes: &[u8]) -> Result<String, ArtifactBlobStoreError> {
        let byte_length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        self.policy.validate_blob_length(byte_length)?;
        let digest = sha256_hex(bytes);
        let path = self.path_for_digest(&digest)?;
        if path.exists() {
            self.verify_at_path(&path, &digest)?;
            return Ok(digest);
        }
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("blob sans répertoire parent"))?;
        create_private_dir(parent)?;
        write_private_file_atomic(&path, bytes)?;
        self.verify_at_path(&path, &digest)?;
        Ok(digest)
    }

    pub fn read(&self, digest: &str) -> Result<Vec<u8>, ArtifactBlobStoreError> {
        let path = self.path_for_digest(digest)?;
        if !path.is_file() {
            return Err(ArtifactBlobStoreError::Missing);
        }
        self.verify_at_path(&path, digest)?;
        Ok(fs::read(path)?)
    }

    /// Vérifie le blob entier sur un descripteur privé unique, mais ne conserve
    /// en mémoire que le fragment demandé. Aucun cache de vérification : une
    /// altération entre deux appels doit être refusée au second appel aussi.
    pub fn read_slice(
        &self,
        digest: &str,
        expected_len: u64,
        offset: u64,
        limit: u32,
    ) -> Result<Vec<u8>, ArtifactBlobStoreError> {
        self.policy.validate_blob_length(expected_len)?;
        if limit == 0
            || limit > bridget_transport::protocol::MAX_ARTIFACT_READ_BYTES
            || offset > expected_len
        {
            return Err(ArtifactBlobStoreError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "fenêtre de lecture invalide",
            )));
        }
        let path = self.path_for_digest(digest)?;
        crate::environment::validate_state_file(&path, false)
            .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error))?;
        let mut file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
            .map_err(|error| {
                if error.kind() == io::ErrorKind::NotFound {
                    ArtifactBlobStoreError::Missing
                } else {
                    error.into()
                }
            })?;
        let before = file.metadata()?;
        if !before.is_file()
            || before.uid() != unsafe { libc::geteuid() }
            || before.mode() & 0o077 != 0
            || before.len() != expected_len
        {
            return Err(ArtifactBlobStoreError::Corrupt {
                expected: digest.into(),
                actual: "métadonnées incohérentes".into(),
            });
        }
        let mut hash = Sha256::new();
        let mut buffer = [0_u8; 16 * 1024];
        let mut observed = 0_u64;
        let mut bytes = Vec::with_capacity(limit as usize);
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            let end = observed
                .checked_add(count as u64)
                .ok_or_else(|| io::Error::other("taille débordée"))?;
            if end > expected_len {
                return Err(ArtifactBlobStoreError::Corrupt {
                    expected: digest.into(),
                    actual: "taille modifiée".into(),
                });
            }
            hash.update(&buffer[..count]);
            let from = offset.max(observed);
            let to = offset.saturating_add(u64::from(limit)).min(end);
            if from < to {
                bytes.extend_from_slice(
                    &buffer[(from - observed) as usize..(to - observed) as usize],
                );
            }
            observed = end;
        }
        let actual = format!("{:x}", hash.finalize());
        let after = file.metadata()?;
        if actual != digest
            || observed != expected_len
            || before.mtime() != after.mtime()
            || before.mtime_nsec() != after.mtime_nsec()
            || before.ctime() != after.ctime()
            || before.ctime_nsec() != after.ctime_nsec()
        {
            return Err(ArtifactBlobStoreError::Corrupt {
                expected: digest.into(),
                actual,
            });
        }
        Ok(bytes)
    }

    pub fn contains(&self, digest: &str) -> Result<bool, ArtifactBlobStoreError> {
        let path = self.path_for_digest(digest)?;
        if !path.is_file() {
            return Ok(false);
        }
        self.verify_at_path(&path, digest)?;
        Ok(true)
    }

    /// Résout un chemin canonique interne. Cette méthode est réservée aux
    /// services du daemon et ne doit jamais franchir une frontière UI ou MCP.
    pub(crate) fn canonical_path(&self, digest: &str) -> Result<PathBuf, ArtifactBlobStoreError> {
        self.path_for_digest(digest)
    }

    fn verify_at_path(&self, path: &Path, expected: &str) -> Result<(), ArtifactBlobStoreError> {
        let bytes = fs::read(path)?;
        let actual = sha256_hex(&bytes);
        if actual != expected {
            return Err(ArtifactBlobStoreError::Corrupt {
                expected: expected.to_string(),
                actual,
            });
        }
        Ok(())
    }

    fn path_for_digest(&self, digest: &str) -> Result<PathBuf, ArtifactBlobStoreError> {
        if !is_sha256_hex(digest) {
            return Err(ArtifactBlobStoreError::InvalidDigest);
        }
        Ok(self.root.join("blobs").join(&digest[..2]).join(digest))
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fixture_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-artifact-blob-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn blob_est_prive_adresse_par_empreinte_et_dedoublonne() {
        let root = fixture_root("dedup");
        let store = ArtifactBlobStore::open(&root, ArtifactPolicy::default()).unwrap();
        let first = store.put(b"contenu durable").unwrap();
        let second = store.put(b"contenu durable").unwrap();
        assert_eq!(first, second);
        assert_eq!(store.read(&first).unwrap(), b"contenu durable");
        let path = store.canonical_path(&first).unwrap();
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(root).unwrap().permissions().mode() & 0o777,
            0o700
        );
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn corruption_ne_devient_jamais_un_blob_lisible() {
        let root = fixture_root("corrupt");
        let store = ArtifactBlobStore::open(&root, ArtifactPolicy::default()).unwrap();
        let digest = store.put(b"contenu sain").unwrap();
        let path = store.canonical_path(&digest).unwrap();
        fs::write(path, b"contenu altere").unwrap();
        assert!(matches!(
            store.read(&digest),
            Err(ArtifactBlobStoreError::Corrupt { .. })
        ));
        let _ = fs::remove_dir_all(store.root());
    }
}
