//! Persistance SQLite additive des artefacts immuables.
//!
//! Le store ne possède pas les fichiers binaires. Il enregistre uniquement
//! leurs empreintes, tailles et références, afin que le magasin de blobs puisse
//! rester privé et que les vues UI ne reçoivent jamais un chemin système.

use crate::artifact_types::{
    ArtifactKind, ArtifactPublicationV1, ArtifactReceiptV1, ArtifactState, ArtifactStorageState,
    canonical_json_bytes, sha256_hex,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug)]
pub enum ArtifactStoreError {
    Sqlite(rusqlite::Error),
    Serialization(serde_json::Error),
    InvalidInput(&'static str),
    UnknownArtifact,
    CrossProject,
    IdempotencyConflict,
    CorruptReceipt,
    CorruptManifest,
    BlobMetadataConflict,
}

impl std::fmt::Display for ArtifactStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(error) => write!(formatter, "SQLite artefact: {error}"),
            Self::Serialization(error) => write!(formatter, "sérialisation artefact: {error}"),
            Self::InvalidInput(detail) => write!(formatter, "entrée artefact invalide: {detail}"),
            Self::UnknownArtifact => formatter.write_str("artefact inconnu"),
            Self::CrossProject => formatter.write_str("artefact hors du projet attesté"),
            Self::IdempotencyConflict => formatter.write_str("conflit de clé d'idempotence"),
            Self::CorruptReceipt => formatter.write_str("reçu d'artefact corrompu"),
            Self::CorruptManifest => formatter.write_str("manifeste d'artefact corrompu"),
            Self::BlobMetadataConflict => {
                formatter.write_str("métadonnées de blob contradictoires")
            }
        }
    }
}

impl std::error::Error for ArtifactStoreError {}

impl From<rusqlite::Error> for ArtifactStoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<serde_json::Error> for ArtifactStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactPublicationContext {
    pub project_id: String,
    pub conversation_reference: String,
    pub turn_reference: String,
    pub created_by: String,
    pub origin_instance: String,
    pub observed_at: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactBlobReference {
    pub digest: String,
    pub media_type: String,
    pub byte_length: u64,
    /// Chemin interne Bridget seulement. Les routes UI ne le lisent jamais.
    pub canonical_path: String,
}

#[derive(Clone, Debug)]
pub struct ArtifactStoreWrite {
    pub artifact_ref: String,
    /// `None` crée l'artefact initial. `Some` crée une version enfant.
    pub existing_artifact_ref: Option<String>,
    pub version_ref: String,
    pub context: ArtifactPublicationContext,
    pub publication: ArtifactPublicationV1,
    pub state: ArtifactState,
    pub receipt: ArtifactReceiptV1,
    pub blob_references: Vec<ArtifactBlobReference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactPersistResult {
    Created(ArtifactReceiptV1),
    Replayed(ArtifactReceiptV1),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactListItem {
    pub artifact_ref: String,
    pub version_ref: String,
    pub project_id: String,
    pub conversation_reference: String,
    pub turn_reference: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub state: ArtifactState,
    pub pinned: bool,
    pub created_at: i64,
    pub version_created_at: i64,
}

/// Référence de lecture d'une version exacte dans son fil d'origine.
///
/// Une publication ultérieure peut rendre une autre version « courante », mais
/// cette projection garde le lien historique vers la version réellement
/// montrée pendant le tour. Elle ne contient jamais de chemin local.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactConversationReference {
    pub reference_id: String,
    pub artifact_ref: String,
    pub version_ref: String,
    pub project_id: String,
    pub conversation_reference: String,
    pub turn_reference: String,
    pub kind: ArtifactKind,
    pub title: String,
    pub state: ArtifactState,
    pub pinned: bool,
    pub version_created_at: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArtifactVersionDetail {
    pub item: ArtifactListItem,
    /// La référence affichée peut être historique. Ce drapeau permet à la
    /// conversation de le dire sans remplacer silencieusement la version
    /// réellement publiée dans le tour.
    pub is_current: bool,
    pub publication: ArtifactPublicationV1,
    /// Octets persistés et vérifiés : jamais une reconstruction pour le lecteur.
    pub manifest_bytes: Vec<u8>,
    pub content_digest: String,
    pub payload_digest: String,
    pub provenance_digest: String,
    pub parent_version_ref: Option<String>,
    pub quality_notices: Vec<String>,
    pub publication_reason: String,
}

/// Compteurs d'exploitation agrégés, strictement non sensibles. Ils ne
/// contiennent ni titres, ni sources, ni identités d'agent.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ArtifactMetricsSnapshot {
    pub canonical_blob_bytes: u64,
    pub publication_failures: u64,
    pub restored_versions: u64,
    pub evictions: u64,
}

/// Métadonnées internes d'un blob, consultables seulement après le contrôle de
/// portée projet effectué par le store. Le chemin ne doit jamais être sérialisé
/// dans une réponse UI ou MCP.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactVersionBlob {
    pub(crate) digest: String,
    pub(crate) media_type: String,
    pub(crate) byte_length: u64,
    pub(crate) canonical_path: String,
}

pub struct ArtifactStore {
    conn: Connection,
}

impl ArtifactStore {
    pub fn open(path: &Path) -> Result<Self, ArtifactStoreError> {
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(2))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), ArtifactStoreError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS artifacts (
                artifact_ref TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                conversation_reference TEXT NOT NULL,
                turn_reference TEXT NOT NULL,
                kind TEXT NOT NULL CHECK (kind IN ('chart', 'kpi', 'table', 'timeline', 'image', 'file', 'html')),
                title TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                current_version_ref TEXT,
                pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
                deleted_at INTEGER,
                origin_instance TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_artifacts_project_current
                ON artifacts(project_id, deleted_at, created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_artifacts_conversation
                ON artifacts(project_id, conversation_reference, created_at DESC);

            CREATE TABLE IF NOT EXISTS artifact_versions (
                version_ref TEXT PRIMARY KEY,
                artifact_ref TEXT NOT NULL REFERENCES artifacts(artifact_ref),
                version_number INTEGER NOT NULL CHECK (version_number > 0),
                parent_version_ref TEXT REFERENCES artifact_versions(version_ref),
                conversation_reference TEXT NOT NULL,
                turn_reference TEXT NOT NULL,
                payload_digest TEXT NOT NULL,
                provenance_digest TEXT NOT NULL,
                content_digest TEXT NOT NULL,
                manifest_json BLOB NOT NULL,
                state TEXT NOT NULL CHECK (state IN ('published', 'partial', 'unavailable', 'failed', 'deleted')),
                quality_notices_json BLOB NOT NULL,
                publication_reason TEXT NOT NULL CHECK (publication_reason IN ('initial', 'refresh', 'restore_changed', 'save_interaction')),
                created_by TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE (artifact_ref, version_number)
            );
            CREATE INDEX IF NOT EXISTS idx_artifact_versions_artifact
                ON artifact_versions(artifact_ref, version_number DESC);
            CREATE INDEX IF NOT EXISTS idx_artifact_versions_conversation
                ON artifact_versions(conversation_reference, turn_reference, created_at);

            CREATE TABLE IF NOT EXISTS artifact_sources (
                version_ref TEXT NOT NULL REFERENCES artifact_versions(version_ref) ON DELETE CASCADE,
                position INTEGER NOT NULL CHECK (position >= 0),
                source_kind TEXT NOT NULL,
                locator TEXT NOT NULL,
                fetched_at INTEGER,
                content_digest TEXT,
                citation TEXT NOT NULL,
                units TEXT,
                transformations_json BLOB NOT NULL,
                access_status TEXT NOT NULL,
                PRIMARY KEY (version_ref, position)
            );
            CREATE INDEX IF NOT EXISTS idx_artifact_sources_locator
                ON artifact_sources(locator);

            CREATE TABLE IF NOT EXISTS artifact_blobs (
                digest TEXT PRIMARY KEY,
                media_type TEXT NOT NULL,
                byte_length INTEGER NOT NULL CHECK (byte_length >= 0),
                canonical_path TEXT NOT NULL,
                reference_count INTEGER NOT NULL DEFAULT 0 CHECK (reference_count >= 0),
                written_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS artifact_version_blobs (
                version_ref TEXT NOT NULL REFERENCES artifact_versions(version_ref) ON DELETE CASCADE,
                digest TEXT NOT NULL REFERENCES artifact_blobs(digest),
                PRIMARY KEY (version_ref, digest)
            );
            CREATE INDEX IF NOT EXISTS idx_artifact_version_blobs_digest
                ON artifact_version_blobs(digest);

            CREATE TABLE IF NOT EXISTS artifact_publications (
                project_id TEXT NOT NULL,
                idempotency_key TEXT NOT NULL,
                content_digest TEXT NOT NULL,
                artifact_ref TEXT NOT NULL REFERENCES artifacts(artifact_ref),
                version_ref TEXT NOT NULL REFERENCES artifact_versions(version_ref),
                receipt_json BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (project_id, idempotency_key)
            );
            CREATE INDEX IF NOT EXISTS idx_artifact_publications_version
                ON artifact_publications(version_ref);

            CREATE TABLE IF NOT EXISTS artifact_references (
                reference_id TEXT PRIMARY KEY,
                artifact_ref TEXT NOT NULL REFERENCES artifacts(artifact_ref),
                version_ref TEXT NOT NULL REFERENCES artifact_versions(version_ref),
                scope TEXT NOT NULL CHECK (scope IN ('conversation', 'turn', 'explicit_agent_share', 'operator_bookmark')),
                target TEXT,
                created_by TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_artifact_references_version
                ON artifact_references(artifact_ref, version_ref, created_at);

            CREATE TABLE IF NOT EXISTS artifact_metric_counters (
                metric_key TEXT PRIMARY KEY CHECK (metric_key IN (
                    'publication_failures', 'restored_versions', 'evictions'
                )),
                value INTEGER NOT NULL DEFAULT 0 CHECK (value >= 0)
            );
            ",
        )?;
        Ok(())
    }

    pub fn persist_publication(
        &mut self,
        input: &ArtifactStoreWrite,
    ) -> Result<ArtifactPersistResult, ArtifactStoreError> {
        validate_write(input)?;
        let content_digest = input.publication.content_digest();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;

        if let Some((stored_digest, stored_receipt)) = tx
            .query_row(
                "SELECT content_digest, receipt_json FROM artifact_publications
                 WHERE project_id = ?1 AND idempotency_key = ?2",
                params![input.context.project_id, input.publication.idempotency_key],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)),
            )
            .optional()?
        {
            let receipt = serde_json::from_slice::<ArtifactReceiptV1>(&stored_receipt)
                .map_err(|_| ArtifactStoreError::CorruptReceipt)?;
            if stored_digest == content_digest {
                tx.commit()?;
                return Ok(ArtifactPersistResult::Replayed(receipt));
            }
            return Err(ArtifactStoreError::IdempotencyConflict);
        }

        let (artifact_ref, version_number, parent_version_ref) =
            resolve_artifact_version(&tx, input)?;
        let payload_digest = sha256_hex(&canonical_json_bytes(&input.publication.payload));
        let sources_json = serde_json::to_value(&input.publication.sources)?;
        let provenance_digest = sha256_hex(&canonical_json_bytes(&sources_json));
        let manifest_json = input.publication.canonical_bytes();
        let quality_notices_json = serde_json::to_vec(&input.publication.quality_notices)?;

        if input.existing_artifact_ref.is_none() {
            tx.execute(
                "INSERT INTO artifacts (
                    artifact_ref, project_id, conversation_reference, turn_reference, kind, title,
                    created_at, current_version_ref, pinned, deleted_at, origin_instance
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 0, NULL, ?8)",
                params![
                    artifact_ref,
                    input.context.project_id,
                    input.context.conversation_reference,
                    input.context.turn_reference,
                    input.publication.kind.as_db(),
                    input.publication.title,
                    input.context.observed_at,
                    input.context.origin_instance,
                ],
            )?;
        }

        tx.execute(
            "INSERT INTO artifact_versions (
                version_ref, artifact_ref, version_number, parent_version_ref, conversation_reference,
                turn_reference, payload_digest, provenance_digest, content_digest, manifest_json,
                state, quality_notices_json, publication_reason, created_by, created_at
             ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
             )",
            params![
                input.version_ref,
                artifact_ref,
                version_number,
                parent_version_ref,
                input.context.conversation_reference,
                input.context.turn_reference,
                payload_digest,
                provenance_digest,
                content_digest,
                manifest_json,
                input.state.as_db(),
                quality_notices_json,
                input.publication.publication_reason.as_db(),
                input.context.created_by,
                input.context.observed_at,
            ],
        )?;

        for (position, source) in input.publication.sources.iter().enumerate() {
            tx.execute(
                "INSERT INTO artifact_sources (
                    version_ref, position, source_kind, locator, fetched_at, content_digest,
                    citation, units, transformations_json, access_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    input.version_ref,
                    i64::try_from(position)
                        .map_err(|_| ArtifactStoreError::InvalidInput("position source"))?,
                    source.source_kind.as_db(),
                    source.locator,
                    source.fetched_at,
                    source.content_digest,
                    source.citation,
                    source.units,
                    serde_json::to_vec(&source.transformations)?,
                    source.access_status.as_db(),
                ],
            )?;
        }

        for blob in &input.blob_references {
            upsert_blob_reference(&tx, &input.version_ref, blob, input.context.observed_at)?;
        }

        tx.execute(
            "UPDATE artifacts SET current_version_ref = ?1 WHERE artifact_ref = ?2",
            params![input.version_ref, artifact_ref],
        )?;
        let receipt_json = serde_json::to_vec(&input.receipt)?;
        tx.execute(
            "INSERT INTO artifact_publications (
                project_id, idempotency_key, content_digest, artifact_ref, version_ref, receipt_json, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                input.context.project_id,
                input.publication.idempotency_key,
                content_digest,
                artifact_ref,
                input.version_ref,
                receipt_json,
                input.context.observed_at,
            ],
        )?;
        tx.execute(
            "INSERT INTO artifact_references (
                reference_id, artifact_ref, version_ref, scope, target, created_by, created_at
             ) VALUES (?1, ?2, ?3, 'turn', ?4, ?5, ?6)",
            params![
                format!("artifact-reference:{}", input.version_ref),
                artifact_ref,
                input.version_ref,
                input.context.turn_reference,
                input.context.created_by,
                input.context.observed_at,
            ],
        )?;
        tx.commit()?;
        Ok(ArtifactPersistResult::Created(input.receipt.clone()))
    }

    pub fn list_project(
        &self,
        project_id: &str,
        limit: usize,
        before_created_at: Option<i64>,
    ) -> Result<Vec<ArtifactListItem>, ArtifactStoreError> {
        let bounded_limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(100);
        let before = before_created_at.unwrap_or(i64::MAX);
        let mut statement = self.conn.prepare(
            "SELECT a.artifact_ref, v.version_ref, a.project_id, v.conversation_reference,
                    v.turn_reference, a.kind, a.title, v.state, a.pinned, a.created_at, v.created_at
             FROM artifacts a
             JOIN artifact_versions v ON v.version_ref = a.current_version_ref
             WHERE a.project_id = ?1 AND a.deleted_at IS NULL AND v.created_at < ?2
             ORDER BY v.created_at DESC, a.artifact_ref DESC
             LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![project_id, before, bounded_limit],
            artifact_list_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ArtifactStoreError::Sqlite)
    }

    /// Recherche limitée au projet attesté. L'appelant qui veut rechercher
    /// ailleurs doit demander explicitement une autre portée à Bridget : il
    /// n'existe pas de recherche implicite sur tous les projets.
    pub fn search_project(
        &self,
        project_id: &str,
        query: &str,
        limit: usize,
        before_created_at: Option<i64>,
    ) -> Result<Vec<ArtifactListItem>, ArtifactStoreError> {
        let normalized = query.trim();
        if normalized.is_empty() {
            return self.list_project(project_id, limit, before_created_at);
        }
        let bounded_limit = i64::try_from(limit.clamp(1, 100)).unwrap_or(100);
        let before = before_created_at.unwrap_or(i64::MAX);
        let pattern = format!("%{normalized}%");
        let mut statement = self.conn.prepare(
            "SELECT a.artifact_ref, v.version_ref, a.project_id, v.conversation_reference,
                    v.turn_reference, a.kind, a.title, v.state, a.pinned, a.created_at, v.created_at
             FROM artifacts a
             JOIN artifact_versions v ON v.version_ref = a.current_version_ref
             WHERE a.project_id = ?1
               AND a.deleted_at IS NULL
               AND v.created_at < ?2
               AND (a.title LIKE ?3 ESCAPE '\\' OR v.manifest_json LIKE ?3 ESCAPE '\\')
             ORDER BY v.created_at DESC, a.artifact_ref DESC
             LIMIT ?4",
        )?;
        let rows = statement.query_map(
            params![project_id, before, pattern, bounded_limit],
            artifact_list_item,
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ArtifactStoreError::Sqlite)
    }

    /// Références exactes d'une conversation, dans l'ordre du fil. Le filtre
    /// projet est obligatoire, donc une route UI ne peut jamais transformer
    /// un `version_ref` deviné en lecture inter-projet.
    pub fn list_conversation_references(
        &self,
        project_id: &str,
        conversation_reference: &str,
        limit: usize,
    ) -> Result<Vec<ArtifactConversationReference>, ArtifactStoreError> {
        let bounded_limit = i64::try_from(limit.clamp(1, 200)).unwrap_or(200);
        let mut statement = self.conn.prepare(
            "SELECT r.reference_id, r.artifact_ref, r.version_ref, a.project_id,
                    v.conversation_reference, v.turn_reference, a.kind, a.title,
                    v.state, a.pinned, v.created_at
             FROM artifact_references r
             JOIN artifacts a ON a.artifact_ref = r.artifact_ref
             JOIN artifact_versions v ON v.version_ref = r.version_ref
             WHERE a.project_id = ?1
               AND v.conversation_reference = ?2
               AND a.deleted_at IS NULL
             ORDER BY v.created_at ASC, r.reference_id ASC
             LIMIT ?3",
        )?;
        let rows = statement.query_map(
            params![project_id, conversation_reference, bounded_limit],
            |row| {
                let kind: String = row.get(6)?;
                let state: String = row.get(8)?;
                Ok(ArtifactConversationReference {
                    reference_id: row.get(0)?,
                    artifact_ref: row.get(1)?,
                    version_ref: row.get(2)?,
                    project_id: row.get(3)?,
                    conversation_reference: row.get(4)?,
                    turn_reference: row.get(5)?,
                    kind: ArtifactKind::from_db(&kind).ok_or(rusqlite::Error::InvalidQuery)?,
                    title: row.get(7)?,
                    state: ArtifactState::from_db(&state).ok_or(rusqlite::Error::InvalidQuery)?,
                    pinned: row.get::<_, i64>(9)? != 0,
                    version_created_at: row.get(10)?,
                })
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ArtifactStoreError::Sqlite)
    }

    pub fn version_detail(
        &self,
        project_id: &str,
        version_ref: &str,
    ) -> Result<Option<ArtifactVersionDetail>, ArtifactStoreError> {
        // Le plafond d'entrée précède l'enrichissement HTML historique :
        // trois clés fixes et un digest SHA-256 de 64 octets, < 160 octets JSON.
        // La réserve bornée ne modifie aucun canon déjà persisté.
        let persisted_limit =
            crate::artifact_policy::ArtifactPolicy::default().manifest_max_bytes + 256;
        let raw = self
            .conn
            .query_row(
                "SELECT a.artifact_ref, v.version_ref, a.project_id, v.conversation_reference,
                        v.turn_reference, a.kind, a.title, v.state, a.pinned, a.created_at,
                        v.created_at, CASE WHEN length(v.manifest_json) <= ?3 THEN v.manifest_json END,
                        v.content_digest, v.payload_digest,
                        v.provenance_digest, v.parent_version_ref, v.quality_notices_json,
                        v.publication_reason, a.current_version_ref = v.version_ref
                 FROM artifact_versions v
                 JOIN artifacts a ON a.artifact_ref = v.artifact_ref
                 WHERE a.project_id = ?1 AND v.version_ref = ?2 AND a.deleted_at IS NULL",
                params![project_id, version_ref, persisted_limit],
                |row| {
                    Ok((
                        artifact_list_item(row)?,
                        row.get::<_, Option<Vec<u8>>>(11)?,
                        row.get::<_, String>(12)?,
                        row.get::<_, String>(13)?,
                        row.get::<_, String>(14)?,
                        row.get::<_, Option<String>>(15)?,
                        row.get::<_, Vec<u8>>(16)?,
                        row.get::<_, String>(17)?,
                        row.get::<_, i64>(18)? != 0,
                    ))
                },
            )
            .optional()?;
        let Some((
            item,
            manifest_json,
            content_digest,
            payload_digest,
            provenance_digest,
            parent_version_ref,
            quality_notices_json,
            publication_reason,
            is_current,
        )) = raw
        else {
            return Ok(None);
        };
        let manifest_json = manifest_json.ok_or(ArtifactStoreError::CorruptManifest)?;
        if sha256_hex(&manifest_json) != content_digest {
            return Err(ArtifactStoreError::CorruptManifest);
        }
        let publication = serde_json::from_slice(&manifest_json)
            .map_err(|_| ArtifactStoreError::CorruptManifest)?;
        let quality_notices = serde_json::from_slice(&quality_notices_json)
            .map_err(|_| ArtifactStoreError::CorruptManifest)?;
        Ok(Some(ArtifactVersionDetail {
            item,
            is_current,
            publication,
            manifest_bytes: manifest_json,
            content_digest,
            payload_digest,
            provenance_digest,
            parent_version_ref,
            quality_notices,
            publication_reason,
        }))
    }

    pub fn blob_reference_count(&self, digest: &str) -> Result<Option<u64>, ArtifactStoreError> {
        let count = self
            .conn
            .query_row(
                "SELECT reference_count FROM artifact_blobs WHERE digest = ?1",
                [digest],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        count
            .map(|value| {
                u64::try_from(value)
                    .map_err(|_| ArtifactStoreError::InvalidInput("compteur blob négatif"))
            })
            .transpose()
    }

    pub(crate) fn version_blob(
        &self,
        project_id: &str,
        version_ref: &str,
        digest: &str,
    ) -> Result<Option<ArtifactVersionBlob>, ArtifactStoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT b.digest, b.media_type, b.byte_length, b.canonical_path
                 FROM artifact_version_blobs vb
                 JOIN artifact_blobs b ON b.digest = vb.digest
                 JOIN artifact_versions v ON v.version_ref = vb.version_ref
                 JOIN artifacts a ON a.artifact_ref = v.artifact_ref
                 WHERE a.project_id = ?1
                   AND a.deleted_at IS NULL
                   AND vb.version_ref = ?2
                   AND vb.digest = ?3",
                params![project_id, version_ref, digest],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .optional()?;
        row.map(|(digest, media_type, byte_length, canonical_path)| {
            Ok(ArtifactVersionBlob {
                digest,
                media_type,
                byte_length: u64::try_from(byte_length)
                    .map_err(|_| ArtifactStoreError::InvalidInput("taille blob négative"))?,
                canonical_path,
            })
        })
        .transpose()
    }

    /// Volume réellement référencé par des artefacts publiés non supprimés.
    /// Un blob dédupliqué ne compte donc qu'une seule fois dans le quota global.
    pub fn total_published_blob_bytes(&self) -> Result<u64, ArtifactStoreError> {
        let total = self.conn.query_row(
            "SELECT COALESCE(SUM(byte_length), 0) FROM artifact_blobs WHERE reference_count > 0",
            [],
            |row| row.get::<_, i64>(0),
        )?;
        u64::try_from(total).map_err(|_| ArtifactStoreError::InvalidInput("volume blob négatif"))
    }

    pub fn record_metric(&self, metric: &str) -> Result<(), ArtifactStoreError> {
        if !matches!(
            metric,
            "publication_failures" | "restored_versions" | "evictions"
        ) {
            return Err(ArtifactStoreError::InvalidInput("métrique d'artefact"));
        }
        self.conn.execute(
            "INSERT INTO artifact_metric_counters(metric_key, value) VALUES (?1, 1)
             ON CONFLICT(metric_key) DO UPDATE SET value = value + 1",
            [metric],
        )?;
        Ok(())
    }

    pub fn metrics_snapshot(&self) -> Result<ArtifactMetricsSnapshot, ArtifactStoreError> {
        let mut snapshot = ArtifactMetricsSnapshot {
            canonical_blob_bytes: self.total_published_blob_bytes()?,
            ..ArtifactMetricsSnapshot::default()
        };
        let mut statement = self
            .conn
            .prepare("SELECT metric_key, value FROM artifact_metric_counters")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows {
            let (key, value) = row?;
            let value = u64::try_from(value)
                .map_err(|_| ArtifactStoreError::InvalidInput("compteur de métrique négatif"))?;
            match key.as_str() {
                "publication_failures" => snapshot.publication_failures = value,
                "restored_versions" => snapshot.restored_versions = value,
                "evictions" => snapshot.evictions = value,
                _ => {
                    return Err(ArtifactStoreError::InvalidInput(
                        "métrique d'artefact inconnue",
                    ));
                }
            }
        }
        Ok(snapshot)
    }

    /// Empreintes nécessaires à la restauration exacte. La méthode inclut les
    /// artefacts supprimés afin que le service puisse vérifier les blobs avant
    /// de réactiver les références dans la transaction de restauration.
    pub fn artifact_blob_digests(
        &self,
        project_id: &str,
        artifact_ref: &str,
    ) -> Result<Vec<String>, ArtifactStoreError> {
        let owner = self
            .conn
            .query_row(
                "SELECT project_id FROM artifacts WHERE artifact_ref = ?1",
                [artifact_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match owner.as_deref() {
            None => return Err(ArtifactStoreError::UnknownArtifact),
            Some(actual) if actual != project_id => return Err(ArtifactStoreError::CrossProject),
            Some(_) => {}
        }
        let mut statement = self.conn.prepare(
            "SELECT DISTINCT vb.digest
             FROM artifact_version_blobs vb
             JOIN artifact_versions v ON v.version_ref = vb.version_ref
             WHERE v.artifact_ref = ?1
             ORDER BY vb.digest ASC",
        )?;
        statement
            .query_map([artifact_ref], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(ArtifactStoreError::Sqlite)
    }

    /// Résolution interne d'une version, y compris après suppression logique,
    /// réservée aux actions de restauration explicites.
    pub fn artifact_ref_for_version(
        &self,
        project_id: &str,
        version_ref: &str,
    ) -> Result<Option<String>, ArtifactStoreError> {
        self.conn
            .query_row(
                "SELECT a.artifact_ref FROM artifact_versions v
                 JOIN artifacts a ON a.artifact_ref = v.artifact_ref
                 WHERE a.project_id = ?1 AND v.version_ref = ?2",
                params![project_id, version_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(ArtifactStoreError::Sqlite)
    }

    pub fn set_pinned(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
        pinned: bool,
    ) -> Result<(), ArtifactStoreError> {
        let changed = self.conn.execute(
            "UPDATE artifacts SET pinned = ?1
             WHERE artifact_ref = ?2 AND project_id = ?3 AND deleted_at IS NULL",
            params![i64::from(pinned), artifact_ref, project_id],
        )?;
        if changed == 0 {
            return Err(ArtifactStoreError::UnknownArtifact);
        }
        Ok(())
    }

    /// Réactive exactement les versions archivées d'un artefact supprimé. Il
    /// ne réécrit ni manifeste ni reçu et rend les mêmes blobs de nouveau
    /// comptés dans la conservation canonique.
    pub fn restore(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
    ) -> Result<Vec<String>, ArtifactStoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing_project = tx
            .query_row(
                "SELECT project_id FROM artifacts WHERE artifact_ref = ?1",
                [artifact_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match existing_project.as_deref() {
            None => return Err(ArtifactStoreError::UnknownArtifact),
            Some(value) if value != project_id => return Err(ArtifactStoreError::CrossProject),
            Some(_) => {}
        }
        let changed = tx.execute(
            "UPDATE artifacts SET deleted_at = NULL WHERE artifact_ref = ?1 AND deleted_at IS NOT NULL",
            [artifact_ref],
        )?;
        if changed == 0 {
            tx.commit()?;
            return Ok(Vec::new());
        }
        let mut statement = tx.prepare(
            "SELECT vb.digest, COUNT(*) FROM artifact_version_blobs vb
             JOIN artifact_versions v ON v.version_ref = vb.version_ref
             WHERE v.artifact_ref = ?1
             GROUP BY vb.digest",
        )?;
        let digests = statement
            .query_map([artifact_ref], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        for (digest, reference_count) in &digests {
            tx.execute(
                "UPDATE artifact_blobs SET reference_count = reference_count + ?2 WHERE digest = ?1",
                params![digest, reference_count],
            )?;
        }
        tx.commit()?;
        Ok(digests.into_iter().map(|(digest, _)| digest).collect())
    }

    /// Partage lisible et traçable avec un agent explicitement ciblé. Le
    /// partage ne change jamais la propriété de projet de l'artefact.
    pub fn share_with_agent(
        &mut self,
        project_id: &str,
        version_ref: &str,
        target_agent: &str,
        created_by: &str,
        created_at: i64,
    ) -> Result<String, ArtifactStoreError> {
        if !is_token(target_agent) || !is_token(created_by) || created_at < 0 {
            return Err(ArtifactStoreError::InvalidInput("partage explicite"));
        }
        let artifact_ref = self
            .conn
            .query_row(
                "SELECT a.artifact_ref FROM artifact_versions v
                 JOIN artifacts a ON a.artifact_ref = v.artifact_ref
                 WHERE a.project_id = ?1 AND v.version_ref = ?2 AND a.deleted_at IS NULL",
                params![project_id, version_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .ok_or(ArtifactStoreError::UnknownArtifact)?;
        let reference_id = format!("artifact-share:{}:{}", version_ref, uuid::Uuid::new_v4());
        self.conn.execute(
            "INSERT INTO artifact_references (
                reference_id, artifact_ref, version_ref, scope, target, created_by, created_at
             ) VALUES (?1, ?2, ?3, 'explicit_agent_share', ?4, ?5, ?6)",
            params![
                reference_id,
                artifact_ref,
                version_ref,
                target_agent,
                created_by,
                created_at,
            ],
        )?;
        Ok(reference_id)
    }

    pub fn soft_delete(
        &mut self,
        project_id: &str,
        artifact_ref: &str,
        deleted_at: i64,
    ) -> Result<Vec<String>, ArtifactStoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing_project = tx
            .query_row(
                "SELECT project_id FROM artifacts WHERE artifact_ref = ?1",
                [artifact_ref],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match existing_project.as_deref() {
            None => return Err(ArtifactStoreError::UnknownArtifact),
            Some(value) if value != project_id => return Err(ArtifactStoreError::CrossProject),
            Some(_) => {}
        }
        tx.execute(
            "UPDATE artifacts SET deleted_at = ?1 WHERE artifact_ref = ?2 AND deleted_at IS NULL",
            params![deleted_at, artifact_ref],
        )?;
        let mut statement = tx.prepare(
            "SELECT vb.digest, COUNT(*) FROM artifact_version_blobs vb
             JOIN artifact_versions v ON v.version_ref = vb.version_ref
             WHERE v.artifact_ref = ?1
             GROUP BY vb.digest",
        )?;
        let digests = statement
            .query_map([artifact_ref], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        drop(statement);
        for (digest, reference_count) in &digests {
            tx.execute(
                "UPDATE artifact_blobs SET reference_count = MAX(reference_count - ?2, 0)
                 WHERE digest = ?1",
                params![digest, reference_count],
            )?;
        }
        tx.commit()?;
        Ok(digests.into_iter().map(|(digest, _)| digest).collect())
    }
}

fn validate_write(input: &ArtifactStoreWrite) -> Result<(), ArtifactStoreError> {
    let context = &input.context;
    if !is_token(&input.artifact_ref)
        || !is_token(&input.version_ref)
        || !is_token(&context.project_id)
        || !is_token(&context.conversation_reference)
        || !is_token(&context.turn_reference)
        || !is_token(&context.created_by)
        || !is_token(&context.origin_instance)
        || context.observed_at < 0
    {
        return Err(ArtifactStoreError::InvalidInput("identité ou contexte"));
    }
    if input.receipt.artifact_ref != input.artifact_ref
        || input.receipt.version_ref != input.version_ref
        || input.receipt.state != input.state
        || input.receipt.warnings != input.publication.quality_notices
        || input.receipt.conversation_reference != context.conversation_reference
        || input.receipt.content_digest != input.publication.content_digest()
        || input.receipt.storage_state == ArtifactStorageState::CanonicalAndCached
    {
        return Err(ArtifactStoreError::InvalidInput("reçu non attesté"));
    }
    if input.existing_artifact_ref.is_none()
        && (input.publication.parent_artifact_ref.is_some()
            || input.publication.publication_reason.as_db() != "initial")
    {
        return Err(ArtifactStoreError::InvalidInput("création initiale"));
    }
    if let Some(existing) = &input.existing_artifact_ref
        && (!is_token(existing)
            || input.publication.parent_artifact_ref.as_deref() != Some(existing))
    {
        return Err(ArtifactStoreError::InvalidInput("parent enfant"));
    }
    if input.state == ArtifactState::Partial && input.publication.quality_notices.is_empty() {
        return Err(ArtifactStoreError::InvalidInput("partiel sans avis"));
    }
    if input.state == ArtifactState::Published && !input.publication.quality_notices.is_empty() {
        return Err(ArtifactStoreError::InvalidInput("avis sans état partiel"));
    }
    for blob in &input.blob_references {
        if !is_sha256_hex(&blob.digest)
            || blob.media_type.trim().is_empty()
            || blob.media_type.len() > 160
            || blob.canonical_path.trim().is_empty()
            || blob.canonical_path.len() > 4_096
        {
            return Err(ArtifactStoreError::InvalidInput("référence blob"));
        }
    }
    Ok(())
}

fn resolve_artifact_version(
    tx: &rusqlite::Transaction<'_>,
    input: &ArtifactStoreWrite,
) -> Result<(String, i64, Option<String>), ArtifactStoreError> {
    let Some(existing_ref) = input.existing_artifact_ref.as_deref() else {
        return Ok((input.artifact_ref.clone(), 1, None));
    };
    let existing = tx
        .query_row(
            "SELECT project_id, kind, current_version_ref, deleted_at FROM artifacts WHERE artifact_ref = ?1",
            [existing_ref],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                ))
            },
        )
        .optional()?
        .ok_or(ArtifactStoreError::UnknownArtifact)?;
    if existing.0 != input.context.project_id {
        return Err(ArtifactStoreError::CrossProject);
    }
    if existing.1 != input.publication.kind.as_db() || existing.3.is_some() {
        return Err(ArtifactStoreError::InvalidInput(
            "artefact enfant incompatible",
        ));
    }
    let parent = existing.2.ok_or(ArtifactStoreError::InvalidInput(
        "artefact sans version courante",
    ))?;
    let next_version = tx.query_row(
        "SELECT COALESCE(MAX(version_number), 0) + 1 FROM artifact_versions WHERE artifact_ref = ?1",
        [existing_ref],
        |row| row.get::<_, i64>(0),
    )?;
    Ok((existing_ref.to_string(), next_version, Some(parent)))
}

fn upsert_blob_reference(
    tx: &rusqlite::Transaction<'_>,
    version_ref: &str,
    blob: &ArtifactBlobReference,
    written_at: i64,
) -> Result<(), ArtifactStoreError> {
    let existing = tx
        .query_row(
            "SELECT media_type, byte_length, canonical_path FROM artifact_blobs WHERE digest = ?1",
            [&blob.digest],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .optional()?;
    let byte_length = i64::try_from(blob.byte_length)
        .map_err(|_| ArtifactStoreError::InvalidInput("taille blob"))?;
    if let Some((media_type, stored_size, canonical_path)) = existing {
        if media_type != blob.media_type
            || stored_size != byte_length
            || canonical_path != blob.canonical_path
        {
            return Err(ArtifactStoreError::BlobMetadataConflict);
        }
    } else {
        tx.execute(
            "INSERT INTO artifact_blobs (
                digest, media_type, byte_length, canonical_path, reference_count, written_at
             ) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
            params![
                blob.digest,
                blob.media_type,
                byte_length,
                blob.canonical_path,
                written_at,
            ],
        )?;
    }
    let inserted = tx.execute(
        "INSERT OR IGNORE INTO artifact_version_blobs (version_ref, digest) VALUES (?1, ?2)",
        params![version_ref, blob.digest],
    )?;
    if inserted == 1 {
        tx.execute(
            "UPDATE artifact_blobs SET reference_count = reference_count + 1 WHERE digest = ?1",
            [&blob.digest],
        )?;
    }
    Ok(())
}

fn artifact_list_item(row: &rusqlite::Row<'_>) -> rusqlite::Result<ArtifactListItem> {
    let kind: String = row.get(5)?;
    let state: String = row.get(7)?;
    let kind = ArtifactKind::from_db(&kind).ok_or(rusqlite::Error::InvalidQuery)?;
    let state = ArtifactState::from_db(&state).ok_or(rusqlite::Error::InvalidQuery)?;
    Ok(ArtifactListItem {
        artifact_ref: row.get(0)?,
        version_ref: row.get(1)?,
        project_id: row.get(2)?,
        conversation_reference: row.get(3)?,
        turn_reference: row.get(4)?,
        kind,
        title: row.get(6)?,
        state,
        pinned: row.get::<_, i64>(8)? != 0,
        created_at: row.get(9)?,
        version_created_at: row.get(10)?,
    })
}

fn is_token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
