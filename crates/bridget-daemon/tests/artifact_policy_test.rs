use bridget_daemon::artifact_policy::{
    ArtifactPolicy, ArtifactPolicyError, CacheEntryAge, GIB, PublicationCapacity,
};
use bridget_daemon::artifact_types::ArtifactPublicationV1;

#[test]
fn limites_cache_et_contenu_publie_sont_bornees() {
    let policy = ArtifactPolicy::default();
    assert_eq!(policy.cache_max_bytes, GIB);
    assert_eq!(policy.cache_max_age_days, 30);
    assert_eq!(
        policy.publication_capacity(7 * GIB, GIB),
        PublicationCapacity::Warning
    );
    assert_eq!(
        policy.publication_capacity(10 * GIB, 1),
        PublicationCapacity::Blocked
    );
    assert_eq!(
        ArtifactPolicy {
            binary_blob_max_bytes: 10,
            ..policy
        }
        .validate_blob_length(11),
        Err(ArtifactPolicyError::BlobTooLarge { max_bytes: 10 })
    );
}

#[test]
fn publication_et_evicition_respectent_provenance_et_pinning() {
    let policy = ArtifactPolicy::default();
    let publication: ArtifactPublicationV1 =
        serde_json::from_str(include_str!("fixtures/artifacts/chart-external-v1.json")).unwrap();
    policy.validate_publication(&publication).unwrap();

    let now = 1_800_000_000;
    let entries = [
        CacheEntryAge {
            last_accessed_at: now - 31 * 24 * 60 * 60,
            byte_length: 1024,
            pinned: false,
        },
        CacheEntryAge {
            last_accessed_at: now - 90 * 24 * 60 * 60,
            byte_length: 4096,
            pinned: true,
        },
    ];
    assert_eq!(policy.cache_eviction_order(&entries, now), vec![0]);
}
