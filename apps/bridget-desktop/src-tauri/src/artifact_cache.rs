//! Cache local évictible des artefacts consultés par Bridget Desktop.
//!
//! Il ne possède aucune autorité de conservation : l'original canonique reste
//! côté instance Bridget. Le cache est identifié par la version et l'empreinte
//! fournies par le relais, est vérifié à chaque lecture et peut être vidé sans
//! changer l'historique de conversation ou l'artefact publié.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CachedArtifactEntryV1 {
    pub version_ref: String,
    pub digest: String,
    pub media_type: String,
    pub byte_length: u64,
    pub last_accessed_at: i64,
    pub pinned: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CacheIndexV1 {
    version: u8,
    entries: BTreeMap<String, CachedArtifactEntryV1>,
}

#[derive(Debug)]
pub enum ArtifactCacheError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidInput,
    Integrity,
}

impl std::fmt::Display for ArtifactCacheError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "cache local d’artefacts: {error}"),
            Self::Json(error) => write!(formatter, "index de cache local: {error}"),
            Self::InvalidInput => formatter.write_str("entrée de cache d’artefact invalide"),
            Self::Integrity => formatter.write_str("cache d’artefact corrompu"),
        }
    }
}

impl std::error::Error for ArtifactCacheError {}

impl From<io::Error> for ArtifactCacheError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ArtifactCacheError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct ArtifactCache {
    root: PathBuf,
}

impl ArtifactCache {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, ArtifactCacheError> {
        let root = root.into();
        fs::create_dir_all(root.join("blobs"))?;
        set_private_dir(&root)?;
        set_private_dir(&root.join("blobs"))?;
        Ok(Self { root })
    }

    pub fn put(
        &self,
        version_ref: &str,
        digest: &str,
        media_type: &str,
        bytes: &[u8],
        pinned: bool,
        now: i64,
    ) -> Result<CachedArtifactEntryV1, ArtifactCacheError> {
        if !valid_reference(version_ref)
            || !valid_digest(digest)
            || media_type.trim().is_empty()
            || now < 0
        {
            return Err(ArtifactCacheError::InvalidInput);
        }
        if sha256_hex(bytes) != digest {
            return Err(ArtifactCacheError::Integrity);
        }
        let path = self.path_for_digest(digest)?;
        if !path.exists() {
            write_private_atomic(&path, bytes)?;
        }
        let entry = CachedArtifactEntryV1 {
            version_ref: version_ref.to_string(),
            digest: digest.to_string(),
            media_type: media_type.to_string(),
            byte_length: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            last_accessed_at: now,
            pinned,
        };
        let mut index = self.load_index()?;
        index.entries.insert(version_ref.to_string(), entry.clone());
        self.write_index(&index)?;
        Ok(entry)
    }

    pub fn get(
        &self,
        version_ref: &str,
        now: i64,
    ) -> Result<Option<(CachedArtifactEntryV1, Vec<u8>)>, ArtifactCacheError> {
        let mut index = self.load_index()?;
        let Some(entry) = index.entries.get_mut(version_ref) else {
            return Ok(None);
        };
        let bytes = match fs::read(self.path_for_digest(&entry.digest)?) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                index.entries.remove(version_ref);
                self.write_index(&index)?;
                return Ok(None);
            }
            Err(error) => return Err(error.into()),
        };
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) != entry.byte_length
            || sha256_hex(&bytes) != entry.digest
        {
            let _ = fs::remove_file(self.path_for_digest(&entry.digest)?);
            index.entries.remove(version_ref);
            self.write_index(&index)?;
            return Err(ArtifactCacheError::Integrity);
        }
        entry.last_accessed_at = now.max(entry.last_accessed_at);
        let output = entry.clone();
        self.write_index(&index)?;
        Ok(Some((output, bytes)))
    }

    /// Éviction déterministe, jamais des entrées épinglées : expirées, puis les
    /// moins récemment consultées, puis les plus lourdes. Retourne les versions
    /// retirées de ce Mac uniquement.
    pub fn evict(
        &self,
        max_bytes: u64,
        max_age_days: u32,
        now: i64,
    ) -> Result<Vec<String>, ArtifactCacheError> {
        let mut index = self.load_index()?;
        let max_age_secs = i64::from(max_age_days).saturating_mul(24 * 60 * 60);
        let mut total = index
            .entries
            .values()
            .map(|entry| entry.byte_length)
            .sum::<u64>();
        let mut candidates = index
            .entries
            .values()
            .filter(|entry| !entry.pinned)
            .cloned()
            .collect::<Vec<_>>();
        candidates.sort_by_key(|entry| {
            (
                now.saturating_sub(entry.last_accessed_at) <= max_age_secs,
                entry.last_accessed_at,
                std::cmp::Reverse(entry.byte_length),
                entry.version_ref.clone(),
            )
        });
        let mut removed = Vec::new();
        for entry in candidates {
            let expired = now.saturating_sub(entry.last_accessed_at) > max_age_secs;
            if !expired && total <= max_bytes {
                continue;
            }
            index.entries.remove(&entry.version_ref);
            total = total.saturating_sub(entry.byte_length);
            if !index
                .entries
                .values()
                .any(|other| other.digest == entry.digest)
            {
                let _ = fs::remove_file(self.path_for_digest(&entry.digest)?);
            }
            removed.push(entry.version_ref);
        }
        self.write_index(&index)?;
        Ok(removed)
    }

    pub fn clear(&self) -> Result<(), ArtifactCacheError> {
        let blobs = self.root.join("blobs");
        if blobs.exists() {
            fs::remove_dir_all(&blobs)?;
        }
        fs::create_dir_all(&blobs)?;
        set_private_dir(&blobs)?;
        self.write_index(&CacheIndexV1 {
            version: 1,
            entries: BTreeMap::new(),
        })
    }

    fn load_index(&self) -> Result<CacheIndexV1, ArtifactCacheError> {
        let path = self.root.join("index.json");
        let raw = match fs::read(&path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(CacheIndexV1 {
                    version: 1,
                    entries: BTreeMap::new(),
                });
            }
            Err(error) => return Err(error.into()),
        };
        let index: CacheIndexV1 = serde_json::from_slice(&raw)?;
        if index.version != 1 {
            return Err(ArtifactCacheError::InvalidInput);
        }
        Ok(index)
    }

    fn write_index(&self, index: &CacheIndexV1) -> Result<(), ArtifactCacheError> {
        let bytes = serde_json::to_vec(index)?;
        write_private_atomic(&self.root.join("index.json"), &bytes)
    }

    fn path_for_digest(&self, digest: &str) -> Result<PathBuf, ArtifactCacheError> {
        if !valid_digest(digest) {
            return Err(ArtifactCacheError::InvalidInput);
        }
        Ok(self.root.join("blobs").join(&digest[..2]).join(digest))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn valid_reference(value: &str) -> bool {
    value.len() <= 256
        && value.starts_with("artifact-version:")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn write_private_atomic(path: &Path, bytes: &[u8]) -> Result<(), ArtifactCacheError> {
    let parent = path.parent().ok_or(ArtifactCacheError::InvalidInput)?;
    fs::create_dir_all(parent)?;
    set_private_dir(parent)?;
    let temporary = path.with_extension("tmp");
    let _ = fs::remove_file(&temporary);
    fs::write(&temporary, bytes)?;
    set_private_file(&temporary)?;
    fs::rename(&temporary, path)?;
    set_private_file(path)?;
    Ok(())
}

#[cfg(unix)]
fn set_private_dir(path: &Path) -> Result<(), ArtifactCacheError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_dir(_path: &Path) -> Result<(), ArtifactCacheError> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file(path: &Path) -> Result<(), ArtifactCacheError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_file(_path: &Path) -> Result<(), ArtifactCacheError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-desktop-artifact-cache-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    #[test]
    fn cache_est_verifie_evictable_et_sans_autorite_canonique() {
        let root = root("evict");
        let cache = ArtifactCache::open(&root).unwrap();
        let alpha = b"alpha";
        let beta = b"beta";
        let alpha_digest = sha256_hex(alpha);
        let beta_digest = sha256_hex(beta);
        cache
            .put(
                "artifact-version:alpha",
                &alpha_digest,
                "text/plain",
                alpha,
                false,
                1,
            )
            .unwrap();
        cache
            .put(
                "artifact-version:beta",
                &beta_digest,
                "text/plain",
                beta,
                true,
                2,
            )
            .unwrap();
        assert_eq!(
            cache.get("artifact-version:alpha", 3).unwrap().unwrap().1,
            alpha
        );
        let removed = cache.evict(4, 1, 90_000).unwrap();
        assert_eq!(removed, vec!["artifact-version:alpha"]);
        assert!(
            cache
                .get("artifact-version:alpha", 90_001)
                .unwrap()
                .is_none()
        );
        assert!(
            cache
                .get("artifact-version:beta", 90_001)
                .unwrap()
                .is_some()
        );
        cache.clear().unwrap();
        assert!(
            cache
                .get("artifact-version:beta", 90_002)
                .unwrap()
                .is_none()
        );
        let _ = fs::remove_dir_all(root);
    }
}
