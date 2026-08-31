//! Politique bornée de publication et de conservation des artefacts.
//!
//! Les valeurs de ce module sont la source d'autorité daemon. Les préférences
//! Desktop ne règlent que le cache de ce Mac et ne modifient jamais ces seuils
//! de contenus canoniques publiés.

use crate::artifact_types::{
    ArtifactPublicationV1, ArtifactValidationError, ArtifactValidationLimits,
};

pub const KIB: u64 = 1024;
pub const MIB: u64 = 1024 * KIB;
pub const GIB: u64 = 1024 * MIB;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactPolicy {
    pub manifest_max_bytes: usize,
    pub structured_payload_max_bytes: usize,
    pub binary_blob_max_bytes: u64,
    pub max_sources: usize,
    pub cache_max_bytes: u64,
    pub cache_max_age_days: u32,
    pub published_warning_bytes: u64,
    pub published_block_bytes: u64,
}

impl Default for ArtifactPolicy {
    fn default() -> Self {
        Self {
            manifest_max_bytes: 512 * KIB as usize,
            structured_payload_max_bytes: 16 * MIB as usize,
            binary_blob_max_bytes: 128 * MIB,
            max_sources: 100,
            cache_max_bytes: GIB,
            cache_max_age_days: 30,
            published_warning_bytes: 8 * GIB,
            published_block_bytes: 10 * GIB,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationCapacity {
    Allowed,
    Warning,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CacheEntryAge {
    pub last_accessed_at: i64,
    pub byte_length: u64,
    pub pinned: bool,
}

impl ArtifactPolicy {
    pub fn validation_limits(self) -> ArtifactValidationLimits {
        ArtifactValidationLimits {
            manifest_max_bytes: self.manifest_max_bytes,
            structured_payload_max_bytes: self.structured_payload_max_bytes,
            max_sources: self.max_sources,
        }
    }

    pub fn validate_publication(
        self,
        publication: &ArtifactPublicationV1,
    ) -> Result<(), ArtifactValidationError> {
        publication.validate(self.validation_limits())
    }

    pub fn validate_blob_length(self, byte_length: u64) -> Result<(), ArtifactPolicyError> {
        if byte_length > self.binary_blob_max_bytes {
            return Err(ArtifactPolicyError::BlobTooLarge {
                max_bytes: self.binary_blob_max_bytes,
            });
        }
        Ok(())
    }

    pub fn publication_capacity(
        self,
        current_bytes: u64,
        incoming_bytes: u64,
    ) -> PublicationCapacity {
        let projected = current_bytes.saturating_add(incoming_bytes);
        if projected > self.published_block_bytes {
            PublicationCapacity::Blocked
        } else if projected >= self.published_warning_bytes {
            PublicationCapacity::Warning
        } else {
            PublicationCapacity::Allowed
        }
    }

    pub fn cache_expired(self, last_accessed_at: i64, now: i64) -> bool {
        let max_age_secs = i64::from(self.cache_max_age_days).saturating_mul(24 * 60 * 60);
        now.saturating_sub(last_accessed_at) > max_age_secs
    }

    /// Ordre déterministe : entrées non épinglées expirées, puis plus anciennes,
    /// puis les plus lourdes. L'appelant supprime jusqu'au retour sous plafond.
    pub fn cache_eviction_order(self, entries: &[CacheEntryAge], now: i64) -> Vec<usize> {
        let mut candidates = entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| !entry.pinned)
            .map(|(index, entry)| {
                (
                    index,
                    self.cache_expired(entry.last_accessed_at, now),
                    entry.last_accessed_at,
                    entry.byte_length,
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|(index, expired, last_accessed_at, byte_length)| {
            (
                !expired,
                *last_accessed_at,
                std::cmp::Reverse(*byte_length),
                *index,
            )
        });
        candidates.into_iter().map(|(index, ..)| index).collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactPolicyError {
    BlobTooLarge { max_bytes: u64 },
}

impl std::fmt::Display for ArtifactPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlobTooLarge { max_bytes } => {
                write!(formatter, "blob d'artefact supérieur à {max_bytes} octets")
            }
        }
    }
}

impl std::error::Error for ArtifactPolicyError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seuils_par_defaut_sont_les_valeurs_documentees() {
        assert_eq!(ArtifactPolicy::default().cache_max_bytes, GIB);
        assert_eq!(ArtifactPolicy::default().cache_max_age_days, 30);
        assert_eq!(ArtifactPolicy::default().published_warning_bytes, 8 * GIB);
        assert_eq!(ArtifactPolicy::default().published_block_bytes, 10 * GIB);
    }

    #[test]
    fn capacite_de_publication_ne_purge_jamais_silencieusement() {
        let policy = ArtifactPolicy::default();
        assert_eq!(
            policy.publication_capacity(7 * GIB, GIB),
            PublicationCapacity::Warning
        );
        assert_eq!(
            policy.publication_capacity(10 * GIB, 1),
            PublicationCapacity::Blocked
        );
    }

    #[test]
    fn purge_du_cache_ne_propose_jamais_un_pinning() {
        let policy = ArtifactPolicy::default();
        let now = 1_800_000_000;
        let entries = [
            CacheEntryAge {
                last_accessed_at: now - 40 * 24 * 60 * 60,
                byte_length: 10,
                pinned: false,
            },
            CacheEntryAge {
                last_accessed_at: now - 50 * 24 * 60 * 60,
                byte_length: 20,
                pinned: true,
            },
            CacheEntryAge {
                last_accessed_at: now - 5,
                byte_length: 30,
                pinned: false,
            },
        ];
        assert_eq!(policy.cache_eviction_order(&entries, now), vec![0, 2]);
    }
}
