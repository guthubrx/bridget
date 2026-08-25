//! Mesure bornée d'un diff entre deux objets Git immuables.

use crate::config::ReviewProjectConfig;
use crate::review::{
    ContractDocument, CriticalityMap, FileChange, RegistryFinding, RepositorySnapshot, ReviewError,
    ReviewLotSubmitPayload, ReviewSubmission, ReviewSubmissionError, TrackedPath,
    calculate_criticality, create_review_submission, is_canonical_sha, is_full_branch_ref,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::thread;

const SMALL_OUTPUT_LIMIT: usize = 8 * 1024;
const STDERR_LIMIT: usize = 64 * 1024;

/// Bornes fermées d'une mesure. Un dépassement refuse tout l'instantané.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewGitLimits {
    pub max_paths: usize,
    pub max_path_bytes: usize,
    pub max_blob_bytes: usize,
    pub max_diff_bytes: usize,
    pub max_contract_bytes: usize,
    pub max_findings: usize,
    pub max_registry_bytes: usize,
    pub max_metadata_bytes: usize,
}

impl Default for ReviewGitLimits {
    fn default() -> Self {
        Self {
            max_paths: 100_000,
            max_path_bytes: 4 * 1024,
            max_blob_bytes: 64 * 1024 * 1024,
            max_diff_bytes: 16 * 1024 * 1024,
            max_contract_bytes: 8 * 1024 * 1024,
            max_findings: 10_000,
            max_registry_bytes: 16 * 1024 * 1024,
            max_metadata_bytes: 32 * 1024 * 1024,
        }
    }
}

/// Ressource dont la borne a été franchie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitKind {
    PathCount,
    PathBytes,
    BlobBytes,
    DiffBytes,
    ContractBytes,
    FindingCount,
    RegistryBytes,
    MetadataBytes,
}

/// Entrée attestée par le guichet et configuration locale du projet.
#[derive(Debug, Clone, Copy)]
pub struct GitMeasurementRequest<'a> {
    pub repository_root: &'a Path,
    pub branch_ref: &'a str,
    pub base: &'a str,
    pub head: &'a str,
    pub open_findings: &'a [RegistryFinding],
    pub limits: ReviewGitLimits,
}

/// Entrées transitoires de la préparation. La racine vient exclusivement de
/// la configuration chargée ; la charge réseau ne peut pas la remplacer.
#[derive(Debug, Clone, Copy)]
pub struct ReviewPreparationRequest<'a> {
    pub project: &'a ReviewProjectConfig,
    pub payload: &'a ReviewLotSubmitPayload,
    pub author_id: &'a str,
    pub open_findings: &'a [RegistryFinding],
    pub limits: ReviewGitLimits,
}

/// Sortie sans instantané : aucun contenu de diff, contrat ou constat n'est
/// conservé au-delà du calcul.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedReviewSubmission {
    pub criticality: CriticalityMap,
    pub submission: ReviewSubmission,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewGitError {
    RepositoryUnavailable,
    BranchRefNotFull,
    InvalidCommit {
        field: &'static str,
    },
    BranchHeadMismatch,
    CommitUnavailable {
        field: &'static str,
    },
    BaseNotAncestor,
    SnapshotLimitExceeded {
        kind: LimitKind,
        limit: usize,
        observed: usize,
    },
    InvalidGitOutput {
        step: &'static str,
    },
    GitCommandFailed {
        step: &'static str,
        code: Option<i32>,
    },
    Io {
        step: &'static str,
        detail: String,
    },
    Review(ReviewError),
}

impl std::fmt::Display for ReviewGitError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RepositoryUnavailable => formatter.write_str("dépôt configuré indisponible"),
            Self::BranchRefNotFull => formatter.write_str("référence Git non complète"),
            Self::InvalidCommit { field } => write!(formatter, "SHA {field} non canonique"),
            Self::BranchHeadMismatch => {
                formatter.write_str("la référence ne pointe pas sur la tête soumise")
            }
            Self::CommitUnavailable { field } => write!(formatter, "commit {field} absent"),
            Self::BaseNotAncestor => formatter.write_str("la base n'est pas ancêtre de la tête"),
            Self::SnapshotLimitExceeded {
                kind,
                limit,
                observed,
            } => write!(
                formatter,
                "borne d'instantané {kind:?} dépassée : {observed} > {limit}"
            ),
            Self::InvalidGitOutput { step } => write!(formatter, "sortie Git invalide : {step}"),
            Self::GitCommandFailed { step, code } => {
                write!(formatter, "commande Git échouée : {step} ({code:?})")
            }
            Self::Io { step, detail } => write!(formatter, "I/O Git {step} : {detail}"),
            Self::Review(source) => write!(formatter, "instantané invalide : {source}"),
        }
    }
}

impl Error for ReviewGitError {}

impl From<ReviewError> for ReviewGitError {
    fn from(source: ReviewError) -> Self {
        Self::Review(source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewPreparationError {
    ProjectMismatch,
    Git(ReviewGitError),
    Submission(ReviewSubmissionError),
}

impl std::fmt::Display for ReviewPreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectMismatch => {
                formatter.write_str("projet soumis différent du projet configuré")
            }
            Self::Git(source) => write!(formatter, "mesure Git refusée : {source}"),
            Self::Submission(source) => write!(formatter, "soumission invalide : {source}"),
        }
    }
}

impl Error for ReviewPreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProjectMismatch => None,
            Self::Git(source) => Some(source),
            Self::Submission(source) => Some(source),
        }
    }
}

impl From<ReviewGitError> for ReviewPreparationError {
    fn from(source: ReviewGitError) -> Self {
        Self::Git(source)
    }
}

impl From<ReviewSubmissionError> for ReviewPreparationError {
    fn from(source: ReviewSubmissionError) -> Self {
        Self::Submission(source)
    }
}

#[derive(Debug, Clone)]
struct TreeEntry {
    oid: String,
    size: usize,
    path: String,
}

#[derive(Debug)]
struct GitCapture {
    status: ExitStatus,
    stdout: BoundedBytes,
    _stderr: BoundedBytes,
}

#[derive(Debug)]
struct BoundedBytes {
    bytes: Vec<u8>,
    observed: usize,
}

impl BoundedBytes {
    fn exceeded(&self, limit: usize) -> bool {
        self.observed > limit
    }
}

/// Construit un instantané depuis les arbres `base` et `head`, sans checkout.
///
/// Les arbres sont parcourus une fois, puis les blobs de tête sont lus par un
/// unique `cat-file --batch`. La configuration du worktree et les variables
/// `GIT_*` héritées ne participent pas à la mesure.
pub fn measure_repository(
    request: &GitMeasurementRequest<'_>,
) -> Result<RepositorySnapshot, ReviewGitError> {
    validate_request(request)?;
    validate_findings(request.open_findings, request.limits)?;
    let root = canonical_repository_root(request.repository_root)?;
    verify_branch_head(&root, request.branch_ref, request.head)?;
    verify_commit(&root, "base", request.base)?;
    verify_commit(&root, "head", request.head)?;
    verify_ancestor(&root, request.base, request.head)?;

    let head_tree = read_tree(&root, request.head, request.limits)?;
    let base_tree = read_tree(&root, request.base, request.limits)?;
    let changes = read_changes(&root, request.base, request.head, request.limits)?;
    let head_blobs = read_tree_blobs(
        &root,
        &head_tree,
        request.limits.max_blob_bytes,
        LimitKind::BlobBytes,
    )?;

    let base_by_path: BTreeMap<&str, &TreeEntry> = base_tree
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let head_by_path: BTreeMap<&str, &TreeEntry> = head_tree
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect();
    let old_entries: Vec<TreeEntry> = changes
        .iter()
        .filter_map(|change| change.old_path.as_deref())
        .filter_map(|path| base_by_path.get(path).copied())
        .cloned()
        .collect();
    let base_changed_blobs = read_tree_blobs(
        &root,
        &old_entries,
        request.limits.max_diff_bytes,
        LimitKind::DiffBytes,
    )?;

    let paths = head_tree
        .iter()
        .map(|entry| {
            let bytes = head_blobs
                .get(&entry.oid)
                .ok_or(ReviewGitError::InvalidGitOutput { step: "cat-file" })?;
            Ok(TrackedPath {
                path: entry.path.clone(),
                line_count: line_count(bytes),
            })
        })
        .collect::<Result<Vec<_>, ReviewGitError>>()?;

    let contracts = collect_contracts(&head_tree, &head_blobs, request.limits.max_contract_bytes)?;
    let changes = hydrate_changes(
        changes,
        &base_by_path,
        &head_by_path,
        &base_changed_blobs,
        &head_blobs,
        request.limits.max_diff_bytes,
    )?;

    Ok(RepositorySnapshot {
        paths,
        changes,
        contracts,
        open_findings: request.open_findings.to_vec(),
    })
}

/// Mesure la paire Git configurée, calcule la carte puis prépare la soumission.
///
/// Aucun reçu ni état durable n'est produit. L'instantané contenant les textes
/// sources est détruit au retour ; seule la projection structurée subsiste.
/// Complexité : O(P + B + D + F), avec P chemins, B octets de blobs, D octets
/// de diff et F constats ouverts, sous les bornes de `ReviewGitLimits`.
pub fn prepare_review_submission(
    request: &ReviewPreparationRequest<'_>,
) -> Result<PreparedReviewSubmission, ReviewPreparationError> {
    if request.payload.project_id != request.project.project_id {
        return Err(ReviewPreparationError::ProjectMismatch);
    }

    let snapshot = measure_repository(&GitMeasurementRequest {
        repository_root: &request.project.repository_root,
        branch_ref: &request.payload.branch_ref,
        base: &request.payload.base,
        head: &request.payload.head,
        open_findings: request.open_findings,
        limits: request.limits,
    })?;
    let criticality = calculate_criticality(&snapshot)
        .map_err(ReviewGitError::from)
        .map_err(ReviewPreparationError::from)?;
    let submission = create_review_submission(request.payload, request.author_id, &criticality)?;

    Ok(PreparedReviewSubmission {
        criticality,
        submission,
    })
}

fn validate_request(request: &GitMeasurementRequest<'_>) -> Result<(), ReviewGitError> {
    if !is_full_branch_ref(request.branch_ref) {
        return Err(ReviewGitError::BranchRefNotFull);
    }
    if !is_canonical_sha(request.base) {
        return Err(ReviewGitError::InvalidCommit { field: "base" });
    }
    if !is_canonical_sha(request.head) {
        return Err(ReviewGitError::InvalidCommit { field: "head" });
    }
    Ok(())
}

fn validate_findings(
    findings: &[RegistryFinding],
    limits: ReviewGitLimits,
) -> Result<(), ReviewGitError> {
    if findings.len() > limits.max_findings {
        return Err(limit_error(
            LimitKind::FindingCount,
            limits.max_findings,
            findings.len(),
        ));
    }
    let observed = findings.iter().try_fold(0usize, |total, finding| {
        total
            .checked_add(finding.id.len())
            .and_then(|value| value.checked_add(finding.text.len()))
            .ok_or_else(|| {
                limit_error(
                    LimitKind::RegistryBytes,
                    limits.max_registry_bytes,
                    usize::MAX,
                )
            })
    })?;
    if observed > limits.max_registry_bytes {
        return Err(limit_error(
            LimitKind::RegistryBytes,
            limits.max_registry_bytes,
            observed,
        ));
    }
    Ok(())
}

fn canonical_repository_root(root: &Path) -> Result<PathBuf, ReviewGitError> {
    if !root.is_absolute() || !root.is_dir() {
        return Err(ReviewGitError::RepositoryUnavailable);
    }
    let canonical = fs::canonicalize(root).map_err(|_| ReviewGitError::RepositoryUnavailable)?;
    let capture = git_capture(
        &canonical,
        &["rev-parse", "--path-format=absolute", "--show-toplevel"],
        None,
        SMALL_OUTPUT_LIMIT,
        "repository-root",
    )?;
    if !capture.status.success() || capture.stdout.exceeded(SMALL_OUTPUT_LIMIT) {
        return Err(ReviewGitError::RepositoryUnavailable);
    }
    let measured = output_line(&capture.stdout.bytes, "repository-root")?;
    let measured = fs::canonicalize(measured).map_err(|_| ReviewGitError::RepositoryUnavailable)?;
    if measured != canonical {
        return Err(ReviewGitError::RepositoryUnavailable);
    }
    Ok(canonical)
}

fn verify_branch_head(root: &Path, branch_ref: &str, head: &str) -> Result<(), ReviewGitError> {
    let expression = format!("{branch_ref}^{{commit}}");
    let capture = git_capture(
        root,
        &["rev-parse", "--verify", "--end-of-options", &expression],
        None,
        SMALL_OUTPUT_LIMIT,
        "branch-head",
    )?;
    if !capture.status.success() || capture.stdout.exceeded(SMALL_OUTPUT_LIMIT) {
        return Err(ReviewGitError::BranchHeadMismatch);
    }
    if output_line(&capture.stdout.bytes, "branch-head")? != head {
        return Err(ReviewGitError::BranchHeadMismatch);
    }
    Ok(())
}

fn verify_commit(root: &Path, field: &'static str, sha: &str) -> Result<(), ReviewGitError> {
    let expression = format!("{sha}^{{commit}}");
    let capture = git_capture(
        root,
        &["cat-file", "-e", &expression],
        None,
        SMALL_OUTPUT_LIMIT,
        "commit-exists",
    )?;
    if !capture.status.success() {
        return Err(ReviewGitError::CommitUnavailable { field });
    }
    Ok(())
}

fn verify_ancestor(root: &Path, base: &str, head: &str) -> Result<(), ReviewGitError> {
    let capture = git_capture(
        root,
        &["merge-base", "--is-ancestor", base, head],
        None,
        SMALL_OUTPUT_LIMIT,
        "merge-base",
    )?;
    match capture.status.code() {
        Some(0) => Ok(()),
        Some(1) => Err(ReviewGitError::BaseNotAncestor),
        code => Err(ReviewGitError::GitCommandFailed {
            step: "merge-base",
            code,
        }),
    }
}

fn read_tree(
    root: &Path,
    commit: &str,
    limits: ReviewGitLimits,
) -> Result<Vec<TreeEntry>, ReviewGitError> {
    let capture = git_capture(
        root,
        &["ls-tree", "-r", "-z", "-l", "--full-tree", commit, "--"],
        None,
        limits.max_metadata_bytes,
        "ls-tree",
    )?;
    require_success(&capture, "ls-tree")?;
    ensure_capture_limit(
        &capture.stdout,
        limits.max_metadata_bytes,
        LimitKind::MetadataBytes,
    )?;
    let mut entries = Vec::new();
    for record in capture.stdout.bytes.split(|byte| *byte == 0) {
        if record.is_empty() {
            continue;
        }
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or(ReviewGitError::InvalidGitOutput { step: "ls-tree" })?;
        let metadata = std::str::from_utf8(&record[..tab])
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "ls-tree" })?;
        let fields: Vec<&str> = metadata.split_whitespace().collect();
        if fields.len() != 4 {
            return Err(ReviewGitError::InvalidGitOutput { step: "ls-tree" });
        }
        if fields[1] != "blob" {
            continue;
        }
        let path = std::str::from_utf8(&record[tab + 1..])
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "tree-path" })?;
        ensure_path(path, limits)?;
        let size = fields[3]
            .parse::<usize>()
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "blob-size" })?;
        entries.push(TreeEntry {
            oid: fields[2].to_string(),
            size,
            path: path.to_string(),
        });
        if entries.len() > limits.max_paths {
            return Err(limit_error(
                LimitKind::PathCount,
                limits.max_paths,
                entries.len(),
            ));
        }
    }
    Ok(entries)
}

fn read_changes(
    root: &Path,
    base: &str,
    head: &str,
    limits: ReviewGitLimits,
) -> Result<Vec<FileChange>, ReviewGitError> {
    let capture = git_capture(
        root,
        &[
            "diff",
            "--name-status",
            "-z",
            "--find-renames",
            "--no-ext-diff",
            "--no-textconv",
            base,
            head,
            "--",
        ],
        None,
        limits.max_metadata_bytes,
        "diff-name-status",
    )?;
    require_success(&capture, "diff-name-status")?;
    ensure_capture_limit(
        &capture.stdout,
        limits.max_metadata_bytes,
        LimitKind::MetadataBytes,
    )?;
    let fields: Vec<&[u8]> = capture
        .stdout
        .bytes
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    let mut cursor = 0;
    let mut changes = Vec::new();
    while cursor < fields.len() {
        let status =
            std::str::from_utf8(fields[cursor]).map_err(|_| ReviewGitError::InvalidGitOutput {
                step: "diff-status",
            })?;
        cursor += 1;
        let kind = status
            .as_bytes()
            .first()
            .copied()
            .ok_or(ReviewGitError::InvalidGitOutput {
                step: "diff-status",
            })?;
        let (old_path, new_path) = match kind {
            b'R' | b'C' => {
                let old = next_diff_path(&fields, &mut cursor, limits)?;
                let new = next_diff_path(&fields, &mut cursor, limits)?;
                (Some(old), Some(new))
            }
            b'A' => (None, Some(next_diff_path(&fields, &mut cursor, limits)?)),
            b'D' => (Some(next_diff_path(&fields, &mut cursor, limits)?), None),
            b'M' | b'T' => {
                let path = next_diff_path(&fields, &mut cursor, limits)?;
                (Some(path.clone()), Some(path))
            }
            _ => {
                return Err(ReviewGitError::InvalidGitOutput {
                    step: "diff-status",
                });
            }
        };
        changes.push(FileChange {
            old_path,
            new_path,
            patch: String::new(),
            head_content: None,
        });
        if changes.len() > limits.max_paths {
            return Err(limit_error(
                LimitKind::PathCount,
                limits.max_paths,
                changes.len(),
            ));
        }
    }
    Ok(changes)
}

fn next_diff_path(
    fields: &[&[u8]],
    cursor: &mut usize,
    limits: ReviewGitLimits,
) -> Result<String, ReviewGitError> {
    let bytes = fields
        .get(*cursor)
        .ok_or(ReviewGitError::InvalidGitOutput { step: "diff-path" })?;
    *cursor += 1;
    let path = std::str::from_utf8(bytes)
        .map_err(|_| ReviewGitError::InvalidGitOutput { step: "diff-path" })?;
    ensure_path(path, limits)?;
    Ok(path.to_string())
}

fn read_tree_blobs(
    root: &Path,
    entries: &[TreeEntry],
    max_bytes: usize,
    limit_kind: LimitKind,
) -> Result<BTreeMap<String, Vec<u8>>, ReviewGitError> {
    let mut unique = BTreeMap::new();
    for entry in entries {
        unique.entry(entry.oid.clone()).or_insert(entry.size);
    }
    let total = unique.values().try_fold(0usize, |sum, size| {
        sum.checked_add(*size)
            .ok_or_else(|| limit_error(limit_kind, max_bytes, usize::MAX))
    })?;
    if total > max_bytes {
        return Err(limit_error(limit_kind, max_bytes, total));
    }
    if unique.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut input = Vec::with_capacity(unique.len() * 41);
    for oid in unique.keys() {
        input.extend_from_slice(oid.as_bytes());
        input.push(b'\n');
    }
    let overhead = unique
        .len()
        .checked_mul(100)
        .ok_or_else(|| limit_error(LimitKind::MetadataBytes, max_bytes, usize::MAX))?;
    let output_limit = total
        .checked_add(overhead)
        .ok_or_else(|| limit_error(LimitKind::MetadataBytes, max_bytes, usize::MAX))?;
    let capture = git_capture(
        root,
        &["cat-file", "--batch"],
        Some(input),
        output_limit,
        "cat-file",
    )?;
    require_success(&capture, "cat-file")?;
    ensure_capture_limit(&capture.stdout, output_limit, limit_kind)?;
    parse_batch_blobs(&capture.stdout.bytes, &unique)
}

fn parse_batch_blobs(
    output: &[u8],
    expected: &BTreeMap<String, usize>,
) -> Result<BTreeMap<String, Vec<u8>>, ReviewGitError> {
    let mut cursor = 0;
    let mut blobs = BTreeMap::new();
    for (expected_oid, expected_size) in expected {
        let newline = output[cursor..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map(|offset| cursor + offset)
            .ok_or(ReviewGitError::InvalidGitOutput { step: "cat-file" })?;
        let header = std::str::from_utf8(&output[cursor..newline])
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "cat-file" })?;
        let fields: Vec<&str> = header.split_whitespace().collect();
        if fields.len() != 3 || fields[0] != expected_oid || fields[1] != "blob" {
            return Err(ReviewGitError::InvalidGitOutput { step: "cat-file" });
        }
        let size = fields[2]
            .parse::<usize>()
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "cat-file" })?;
        if size != *expected_size {
            return Err(ReviewGitError::InvalidGitOutput { step: "cat-file" });
        }
        let start = newline + 1;
        let end = start
            .checked_add(size)
            .ok_or(ReviewGitError::InvalidGitOutput { step: "cat-file" })?;
        if output.get(end) != Some(&b'\n') {
            return Err(ReviewGitError::InvalidGitOutput { step: "cat-file" });
        }
        blobs.insert(expected_oid.clone(), output[start..end].to_vec());
        cursor = end + 1;
    }
    if cursor != output.len() {
        return Err(ReviewGitError::InvalidGitOutput { step: "cat-file" });
    }
    Ok(blobs)
}

fn collect_contracts(
    tree: &[TreeEntry],
    blobs: &BTreeMap<String, Vec<u8>>,
    max_bytes: usize,
) -> Result<Vec<ContractDocument>, ReviewGitError> {
    let mut total = 0usize;
    let mut contracts = Vec::new();
    for entry in tree.iter().filter(|entry| is_contract_path(&entry.path)) {
        let bytes = blobs
            .get(&entry.oid)
            .ok_or(ReviewGitError::InvalidGitOutput { step: "contracts" })?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| limit_error(LimitKind::ContractBytes, max_bytes, usize::MAX))?;
        if total > max_bytes {
            return Err(limit_error(LimitKind::ContractBytes, max_bytes, total));
        }
        let content = std::str::from_utf8(bytes)
            .map_err(|_| ReviewGitError::InvalidGitOutput { step: "contracts" })?;
        contracts.push(ContractDocument {
            source_path: entry.path.clone(),
            content: content.to_string(),
        });
    }
    Ok(contracts)
}

fn hydrate_changes(
    changes: Vec<FileChange>,
    base_by_path: &BTreeMap<&str, &TreeEntry>,
    head_by_path: &BTreeMap<&str, &TreeEntry>,
    base_blobs: &BTreeMap<String, Vec<u8>>,
    head_blobs: &BTreeMap<String, Vec<u8>>,
    max_diff_bytes: usize,
) -> Result<Vec<FileChange>, ReviewGitError> {
    let mut total = 0usize;
    changes
        .into_iter()
        .map(|mut change| {
            let old = change
                .old_path
                .as_deref()
                .and_then(|path| base_by_path.get(path))
                .and_then(|entry| base_blobs.get(&entry.oid))
                .map(Vec::as_slice);
            let new = change
                .new_path
                .as_deref()
                .and_then(|path| head_by_path.get(path))
                .and_then(|entry| head_blobs.get(&entry.oid))
                .map(Vec::as_slice);
            let remaining = max_diff_bytes.saturating_sub(total);
            change.patch = changed_lines(old, new, remaining)?;
            total = total.saturating_add(change.patch.len());
            if total > max_diff_bytes {
                return Err(limit_error(LimitKind::DiffBytes, max_diff_bytes, total));
            }
            change.head_content = new
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .map(str::to_string);
            Ok(change)
        })
        .collect()
}

fn changed_lines(
    old: Option<&[u8]>,
    new: Option<&[u8]>,
    max_bytes: usize,
) -> Result<String, ReviewGitError> {
    let Some(old) = old.map(std::str::from_utf8).transpose().ok().flatten() else {
        return bounded_text(new, '+', max_bytes);
    };
    let Some(new) = new.map(std::str::from_utf8).transpose().ok().flatten() else {
        return bounded_text(Some(old.as_bytes()), '-', max_bytes);
    };
    if old == new {
        return Ok(String::new());
    }
    let old_lines: Vec<&str> = old.split_inclusive('\n').collect();
    let new_lines: Vec<&str> = new.split_inclusive('\n').collect();
    let mut new_remaining = line_counts(&new_lines);
    let mut old_remaining = line_counts(&old_lines);
    let mut output = String::new();
    for line in &old_lines {
        if consume_line(&mut new_remaining, line) {
            continue;
        }
        append_changed_line(&mut output, '-', line, max_bytes)?;
    }
    for line in &new_lines {
        if consume_line(&mut old_remaining, line) {
            continue;
        }
        append_changed_line(&mut output, '+', line, max_bytes)?;
    }
    Ok(output)
}

fn bounded_text(
    bytes: Option<&[u8]>,
    prefix: char,
    max_bytes: usize,
) -> Result<String, ReviewGitError> {
    let Some(text) = bytes.and_then(|bytes| std::str::from_utf8(bytes).ok()) else {
        return Ok(String::new());
    };
    let mut output = String::new();
    for line in text.split_inclusive('\n') {
        append_changed_line(&mut output, prefix, line, max_bytes)?;
    }
    Ok(output)
}

fn line_counts<'a>(lines: &[&'a str]) -> BTreeMap<&'a str, usize> {
    let mut counts = BTreeMap::new();
    for line in lines {
        *counts.entry(*line).or_insert(0) += 1;
    }
    counts
}

fn consume_line(counts: &mut BTreeMap<&str, usize>, line: &str) -> bool {
    let Some(count) = counts.get_mut(line) else {
        return false;
    };
    if *count == 0 {
        return false;
    }
    *count -= 1;
    true
}

fn append_changed_line(
    output: &mut String,
    prefix: char,
    line: &str,
    max_bytes: usize,
) -> Result<(), ReviewGitError> {
    let added = 1usize
        .checked_add(line.len())
        .and_then(|size| size.checked_add(usize::from(!line.ends_with('\n'))))
        .ok_or_else(|| limit_error(LimitKind::DiffBytes, max_bytes, usize::MAX))?;
    let observed = output.len().saturating_add(added);
    if observed > max_bytes {
        return Err(limit_error(LimitKind::DiffBytes, max_bytes, observed));
    }
    output.push(prefix);
    output.push_str(line);
    if !line.ends_with('\n') {
        output.push('\n');
    }
    Ok(())
}

fn line_count(bytes: &[u8]) -> usize {
    if bytes.is_empty() {
        0
    } else {
        bytes.iter().filter(|byte| **byte == b'\n').count() + usize::from(!bytes.ends_with(b"\n"))
    }
}

fn ensure_path(path: &str, limits: ReviewGitLimits) -> Result<(), ReviewGitError> {
    if path.len() > limits.max_path_bytes {
        return Err(limit_error(
            LimitKind::PathBytes,
            limits.max_path_bytes,
            path.len(),
        ));
    }
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(ReviewGitError::InvalidGitOutput { step: "path" });
    }
    Ok(())
}

fn is_contract_path(path: &str) -> bool {
    path.starts_with("specs/") && path.contains("/contracts/")
}

fn output_line<'a>(bytes: &'a [u8], step: &'static str) -> Result<&'a str, ReviewGitError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|_| ReviewGitError::InvalidGitOutput { step })?
        .trim_end_matches(['\r', '\n']);
    if value.is_empty() || value.contains(['\r', '\n']) {
        return Err(ReviewGitError::InvalidGitOutput { step });
    }
    Ok(value)
}

fn require_success(capture: &GitCapture, step: &'static str) -> Result<(), ReviewGitError> {
    if capture.status.success() {
        Ok(())
    } else {
        Err(ReviewGitError::GitCommandFailed {
            step,
            code: capture.status.code(),
        })
    }
}

fn ensure_capture_limit(
    bytes: &BoundedBytes,
    limit: usize,
    kind: LimitKind,
) -> Result<(), ReviewGitError> {
    if bytes.exceeded(limit) {
        Err(limit_error(kind, limit, bytes.observed))
    } else {
        Ok(())
    }
}

fn limit_error(kind: LimitKind, limit: usize, observed: usize) -> ReviewGitError {
    ReviewGitError::SnapshotLimitExceeded {
        kind,
        limit,
        observed,
    }
}

fn git_capture(
    root: &Path,
    arguments: &[&str],
    input: Option<Vec<u8>>,
    stdout_limit: usize,
    step: &'static str,
) -> Result<GitCapture, ReviewGitError> {
    let mut command = Command::new("git");
    remove_inherited_git_environment(&mut command);
    command
        .arg("-c")
        .arg("core.quotePath=true")
        .arg("-c")
        .arg("diff.external=")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| io_error(step, error))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io_error(step, "stdout absent"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io_error(step, "stderr absent"))?;
    let stdout_reader = thread::spawn(move || read_bounded_stdout(stdout, stdout_limit));
    let stderr_reader = thread::spawn(move || read_bounded_stderr(stderr, STDERR_LIMIT));
    let input_writer = input.map(|bytes| {
        let mut stdin = child.stdin.take().expect("stdin demandé");
        thread::spawn(move || {
            stdin.write_all(&bytes)?;
            drop(stdin);
            Ok::<(), io::Error>(())
        })
    });
    let status = child.wait().map_err(|error| io_error(step, error))?;
    if let Some(writer) = input_writer {
        writer
            .join()
            .map_err(|_| io_error(step, "thread stdin interrompu"))?
            .map_err(|error| io_error(step, error))?;
    }
    let stdout = stdout_reader
        .join()
        .map_err(|_| io_error(step, "thread stdout interrompu"))?
        .map_err(|error| io_error(step, error))?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| io_error(step, "thread stderr interrompu"))?
        .map_err(|error| io_error(step, error))?;
    Ok(GitCapture {
        status,
        stdout,
        _stderr: stderr,
    })
}

fn remove_inherited_git_environment(command: &mut Command) {
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_PAGER", "cat")
        .env("LC_ALL", "C");
}

fn read_bounded_stdout(reader: ChildStdout, limit: usize) -> Result<BoundedBytes, io::Error> {
    read_bounded(reader, limit)
}

fn read_bounded_stderr(reader: ChildStderr, limit: usize) -> Result<BoundedBytes, io::Error> {
    read_bounded(reader, limit)
}

fn read_bounded(mut reader: impl Read, limit: usize) -> Result<BoundedBytes, io::Error> {
    let mut stored = Vec::with_capacity(limit.min(64 * 1024));
    let mut observed = 0usize;
    let mut chunk = [0u8; 16 * 1024];
    loop {
        let read = reader.read(&mut chunk)?;
        if read == 0 {
            break;
        }
        observed = observed.saturating_add(read);
        let remaining = limit.saturating_sub(stored.len());
        stored.extend_from_slice(&chunk[..read.min(remaining)]);
    }
    Ok(BoundedBytes {
        bytes: stored,
        observed,
    })
}

fn io_error(step: &'static str, detail: impl ToString) -> ReviewGitError {
    ReviewGitError::Io {
        step,
        detail: detail.to_string(),
    }
}
