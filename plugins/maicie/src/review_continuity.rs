//! Continuité d'un verdict de revue face à la tête distante courante.
//!
//! L'observation ne modifie ni référence, ni index, ni worktree. Les objets
//! absents du dépôt observé sont téléchargés dans un object store temporaire
//! isolé, supprimé à la fin de la mesure.

use crate::domain::Delegation;
use bridget_transport::protocol::{ReviewTarget, ReviewVerdict, ReviewVerdictEvidence};
use serde::Serialize;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use uuid::Uuid;

const MAX_GIT_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewHeadRelation {
    StillAncestor { observed_head: String },
    Rewritten { observed_head: String },
}

/// Verdict relu depuis les octets terminaux déjà persistés par le guichet.
/// Ce type est une projection : il n'ajoute ni colonne ni seconde vérité.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoredReviewVerdict {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    pub evidence: ReviewVerdictEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewContinuityState {
    TargetAbsent,
    VerdictAbsent,
    StillAncestor,
    Rewritten,
    Unobservable,
}

impl ReviewContinuityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TargetAbsent => "target_absent",
            Self::VerdictAbsent => "verdict_absent",
            Self::StillAncestor => "still_ancestor",
            Self::Rewritten => "rewritten",
            Self::Unobservable => "unobservable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewUnobservableReason {
    ReviewProjectAbsent,
    VerdictWithoutTarget,
    EvidenceMismatch,
    InvalidTarget,
    RepositoryUnavailable,
    RemoteUnavailable,
    RemoteOutputInvalid,
    ObjectStoreUnavailable,
    TemporaryObjectStoreUnavailable,
    FetchFailed,
    AncestryUnavailable,
}

impl ReviewUnobservableReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReviewProjectAbsent => "review_project_absent",
            Self::VerdictWithoutTarget => "verdict_without_target",
            Self::EvidenceMismatch => "evidence_mismatch",
            Self::InvalidTarget => "invalid_target",
            Self::RepositoryUnavailable => "repository_unavailable",
            Self::RemoteUnavailable => "remote_unavailable",
            Self::RemoteOutputInvalid => "remote_output_invalid",
            Self::ObjectStoreUnavailable => "object_store_unavailable",
            Self::TemporaryObjectStoreUnavailable => "temporary_object_store_unavailable",
            Self::FetchFailed => "fetch_failed",
            Self::AncestryUnavailable => "ancestry_unavailable",
        }
    }
}

/// Vue fermée rendue dans `status` et dans la carte de reprise.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewContinuityObservation {
    pub delegation_id: Uuid,
    pub state: ReviewContinuityState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<ReviewVerdict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<ReviewUnobservableReason>,
}

impl ReviewContinuityObservation {
    fn new(delegation_id: Uuid, state: ReviewContinuityState) -> Self {
        Self {
            delegation_id,
            state,
            target_ref: None,
            reviewed_head: None,
            observed_head: None,
            verdict: None,
            reason: None,
        }
    }
}

/// Observateur de campagne : une même cible Git n'est interrogée qu'une fois
/// pendant un `status`, même si plusieurs délégations la référencent.
pub struct ReviewContinuityObserver<'a> {
    repository_root: Option<&'a Path>,
    cache: BTreeMap<(String, String), Result<ReviewHeadRelation, ReviewContinuityError>>,
}

impl<'a> ReviewContinuityObserver<'a> {
    pub fn new(repository_root: Option<&'a Path>) -> Self {
        Self {
            repository_root,
            cache: BTreeMap::new(),
        }
    }

    /// Complexité : O(log u) par délégation, avec une I/O Git uniquement au
    /// premier passage de chacune des `u` cibles distinctes.
    pub fn observe(
        &mut self,
        delegation: &Delegation,
        stored: Option<&StoredReviewVerdict>,
    ) -> ReviewContinuityObservation {
        observe_delegation_review_with(delegation, stored, |target_ref, reviewed_head| {
            let key = (target_ref.to_string(), reviewed_head.to_string());
            if let Some(cached) = self.cache.get(&key) {
                return cached.clone();
            }
            let result = self
                .repository_root
                .ok_or(ReviewContinuityError::ReviewProjectAbsent)
                .and_then(|root| observe_reviewed_head(root, target_ref, reviewed_head));
            self.cache.insert(key, result.clone());
            result
        })
    }
}

/// Projette un fait durable et une observation Git sans rabattre une absence
/// sur un succès. `repository_root=None` signifie que le projet de revue n'est
/// pas configuré sur cette surface.
pub fn observe_delegation_review(
    repository_root: Option<&Path>,
    delegation: &Delegation,
    stored: Option<&StoredReviewVerdict>,
) -> ReviewContinuityObservation {
    observe_delegation_review_with(delegation, stored, |target_ref, reviewed_head| {
        let repository_root = repository_root.ok_or(ReviewContinuityError::ReviewProjectAbsent)?;
        observe_reviewed_head(repository_root, target_ref, reviewed_head)
    })
}

fn observe_delegation_review_with(
    delegation: &Delegation,
    stored: Option<&StoredReviewVerdict>,
    mut observe: impl FnMut(&str, &str) -> Result<ReviewHeadRelation, ReviewContinuityError>,
) -> ReviewContinuityObservation {
    let Some(target) = delegation.review_target.as_ref() else {
        let mut observation = ReviewContinuityObservation::new(
            delegation.id,
            if stored.is_some() {
                ReviewContinuityState::Unobservable
            } else {
                ReviewContinuityState::TargetAbsent
            },
        );
        if stored.is_some() {
            observation.reason = Some(ReviewUnobservableReason::VerdictWithoutTarget);
        }
        return observation;
    };

    let mut observation =
        ReviewContinuityObservation::new(delegation.id, ReviewContinuityState::VerdictAbsent);
    observation.target_ref = Some(target.target_ref.clone());
    observation.reviewed_head = Some(target.expected_head.clone());
    let Some(stored) = stored else {
        return observation;
    };
    observation.verdict = Some(stored.evidence.verdict);
    observation.reviewed_head = Some(stored.evidence.measured_head.clone());
    if stored.objective_id != delegation.objectif_id
        || stored.delegation_id != delegation.id
        || !stored.evidence.is_valid()
        || stored.evidence.target_ref != target.target_ref
        || stored.evidence.expected_head != target.expected_head
        || stored.evidence.measured_head != target.expected_head
    {
        observation.state = ReviewContinuityState::Unobservable;
        observation.reason = Some(ReviewUnobservableReason::EvidenceMismatch);
        return observation;
    }
    match observe(&stored.evidence.target_ref, &stored.evidence.measured_head) {
        Ok(ReviewHeadRelation::StillAncestor { observed_head }) => {
            observation.state = ReviewContinuityState::StillAncestor;
            observation.observed_head = Some(observed_head);
        }
        Ok(ReviewHeadRelation::Rewritten { observed_head }) => {
            observation.state = ReviewContinuityState::Rewritten;
            observation.observed_head = Some(observed_head);
        }
        Err(error) => {
            observation.state = ReviewContinuityState::Unobservable;
            observation.reason = Some(error.reason());
        }
    }
    observation
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewContinuityError {
    InvalidTarget,
    ReviewProjectAbsent,
    RepositoryUnavailable,
    RemoteUnavailable,
    RemoteOutputInvalid,
    ObjectStoreUnavailable,
    TemporaryObjectStoreUnavailable,
    FetchFailed(String),
    AncestryUnavailable,
}

impl std::fmt::Display for ReviewContinuityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidTarget => "cible de revue invalide",
            Self::ReviewProjectAbsent => "projet de revue absent",
            Self::RepositoryUnavailable => "dépôt Git indisponible",
            Self::RemoteUnavailable => "tête distante indisponible",
            Self::RemoteOutputInvalid => "sortie de tête distante invalide",
            Self::ObjectStoreUnavailable => "object store Git indisponible",
            Self::TemporaryObjectStoreUnavailable => "object store temporaire indisponible",
            Self::FetchFailed(detail) => {
                return write!(
                    formatter,
                    "objets de la tête distante indisponibles : {detail}"
                );
            }
            Self::AncestryUnavailable => "relation d'ancêtralité indisponible",
        })
    }
}

impl Error for ReviewContinuityError {}

impl ReviewContinuityError {
    fn reason(&self) -> ReviewUnobservableReason {
        match self {
            Self::InvalidTarget => ReviewUnobservableReason::InvalidTarget,
            Self::ReviewProjectAbsent => ReviewUnobservableReason::ReviewProjectAbsent,
            Self::RepositoryUnavailable => ReviewUnobservableReason::RepositoryUnavailable,
            Self::RemoteUnavailable => ReviewUnobservableReason::RemoteUnavailable,
            Self::RemoteOutputInvalid => ReviewUnobservableReason::RemoteOutputInvalid,
            Self::ObjectStoreUnavailable => ReviewUnobservableReason::ObjectStoreUnavailable,
            Self::TemporaryObjectStoreUnavailable => {
                ReviewUnobservableReason::TemporaryObjectStoreUnavailable
            }
            Self::FetchFailed(_) => ReviewUnobservableReason::FetchFailed,
            Self::AncestryUnavailable => ReviewUnobservableReason::AncestryUnavailable,
        }
    }
}

/// Mesure la relation entre le SHA jugé et la tête distante courante.
///
/// Le nom du remote et la branche viennent du `ReviewTarget` fermé. La tête
/// observée vient de `ls-remote`; aucun ref local potentiellement périmé ne fait
/// autorité.
pub fn observe_reviewed_head(
    repository_root: &Path,
    target_ref: &str,
    reviewed_head: &str,
) -> Result<ReviewHeadRelation, ReviewContinuityError> {
    let target = ReviewTarget {
        target_ref: target_ref.to_string(),
        expected_head: reviewed_head.to_string(),
    };
    let Some((remote, branch)) = target.remote_and_branch() else {
        return Err(ReviewContinuityError::InvalidTarget);
    };
    let repository_root = canonical_repository_root(repository_root)?;
    let remote_ref = format!("refs/heads/{branch}");
    let output = git_output(
        &repository_root,
        &[
            "ls-remote",
            "--exit-code",
            "--refs",
            "--",
            remote,
            &remote_ref,
        ],
        &[],
    )?;
    if !output.status.success() {
        return Err(ReviewContinuityError::RemoteUnavailable);
    }
    let observed_head = parse_remote_head(&output.stdout, &remote_ref)?;
    if observed_head == reviewed_head {
        return Ok(ReviewHeadRelation::StillAncestor { observed_head });
    }

    let remote_url = remote_url(&repository_root, remote)?;
    let temporary = TemporaryRepository::new(&repository_root)?;
    let environment = temporary.environment();
    let fetch = git_output(
        &temporary.repository,
        &[
            "fetch",
            "--quiet",
            "--no-tags",
            "--no-write-fetch-head",
            "--",
            &remote_url,
            &remote_ref,
        ],
        &environment,
    )?;
    if !fetch.status.success() {
        return Err(ReviewContinuityError::FetchFailed(
            String::from_utf8_lossy(&fetch.stderr).trim().to_string(),
        ));
    }
    let ancestry = git_output(
        &temporary.repository,
        &["merge-base", "--is-ancestor", reviewed_head, &observed_head],
        &environment,
    )?;
    match ancestry.status.code() {
        Some(0) => Ok(ReviewHeadRelation::StillAncestor { observed_head }),
        Some(1) => Ok(ReviewHeadRelation::Rewritten { observed_head }),
        _ => Err(ReviewContinuityError::AncestryUnavailable),
    }
}

fn remote_url(root: &Path, remote: &str) -> Result<String, ReviewContinuityError> {
    let output = git_output(root, &["remote", "get-url", "--", remote], &[])?;
    if !output.status.success() {
        return Err(ReviewContinuityError::RemoteUnavailable);
    }
    let value = std::str::from_utf8(&output.stdout)
        .map_err(|_| ReviewContinuityError::RemoteOutputInvalid)?
        .trim();
    if value.is_empty() || value.contains(['\n', '\r']) {
        return Err(ReviewContinuityError::RemoteOutputInvalid);
    }
    Ok(value.to_string())
}

fn canonical_repository_root(root: &Path) -> Result<PathBuf, ReviewContinuityError> {
    let canonical =
        fs::canonicalize(root).map_err(|_| ReviewContinuityError::RepositoryUnavailable)?;
    let output = git_output(&canonical, &["rev-parse", "--is-inside-work-tree"], &[])?;
    if !output.status.success() || output.stdout != b"true\n" {
        return Err(ReviewContinuityError::RepositoryUnavailable);
    }
    Ok(canonical)
}

fn parse_remote_head(stdout: &[u8], expected_ref: &str) -> Result<String, ReviewContinuityError> {
    let value =
        std::str::from_utf8(stdout).map_err(|_| ReviewContinuityError::RemoteOutputInvalid)?;
    let mut lines = value.lines();
    let line = lines
        .next()
        .ok_or(ReviewContinuityError::RemoteOutputInvalid)?;
    if lines.next().is_some() {
        return Err(ReviewContinuityError::RemoteOutputInvalid);
    }
    let mut fields = line.split('\t');
    let head = fields
        .next()
        .ok_or(ReviewContinuityError::RemoteOutputInvalid)?;
    let remote_ref = fields
        .next()
        .ok_or(ReviewContinuityError::RemoteOutputInvalid)?;
    if fields.next().is_some()
        || remote_ref != expected_ref
        || head.len() != 40
        || !head
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ReviewContinuityError::RemoteOutputInvalid);
    }
    Ok(head.to_string())
}

fn git_output(
    root: &Path,
    arguments: &[&str],
    environment: &[(&str, &Path)],
) -> Result<Output, ReviewContinuityError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(arguments)
        .env("LC_ALL", "C")
        .env("GIT_TERMINAL_PROMPT", "0");
    for (key, value) in environment {
        command.env(key, value);
    }
    let output = command
        .output()
        .map_err(|_| ReviewContinuityError::RepositoryUnavailable)?;
    if output.stdout.len() > MAX_GIT_OUTPUT_BYTES || output.stderr.len() > MAX_GIT_OUTPUT_BYTES {
        return Err(ReviewContinuityError::RemoteOutputInvalid);
    }
    Ok(output)
}

struct TemporaryRepository {
    root: PathBuf,
    repository: PathBuf,
    alternate: PathBuf,
}

impl TemporaryRepository {
    fn new(repository_root: &Path) -> Result<Self, ReviewContinuityError> {
        let output = git_output(
            repository_root,
            &["rev-parse", "--git-path", "objects"],
            &[],
        )?;
        if !output.status.success() {
            return Err(ReviewContinuityError::ObjectStoreUnavailable);
        }
        let raw = std::str::from_utf8(&output.stdout)
            .map_err(|_| ReviewContinuityError::ObjectStoreUnavailable)?
            .trim();
        if raw.is_empty() {
            return Err(ReviewContinuityError::ObjectStoreUnavailable);
        }
        let raw = PathBuf::from(raw);
        let alternate = fs::canonicalize(if raw.is_absolute() {
            raw
        } else {
            repository_root.join(raw)
        })
        .map_err(|_| ReviewContinuityError::ObjectStoreUnavailable)?;
        if !alternate.is_dir() {
            return Err(ReviewContinuityError::ObjectStoreUnavailable);
        }

        let root = std::env::temp_dir().join(format!(
            "maicie-review-continuity-objects-{}",
            Uuid::new_v4()
        ));
        let repository = root.join("repository.git");
        fs::create_dir_all(&root)
            .map_err(|_| ReviewContinuityError::TemporaryObjectStoreUnavailable)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
                .map_err(|_| ReviewContinuityError::TemporaryObjectStoreUnavailable)?;
        }
        let initialized = Command::new("git")
            .args(["init", "--bare", "-q"])
            .arg(&repository)
            .env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|_| ReviewContinuityError::TemporaryObjectStoreUnavailable)?;
        if !initialized.status.success()
            || initialized.stdout.len() > MAX_GIT_OUTPUT_BYTES
            || initialized.stderr.len() > MAX_GIT_OUTPUT_BYTES
        {
            return Err(ReviewContinuityError::TemporaryObjectStoreUnavailable);
        }
        Ok(Self {
            root,
            repository,
            alternate,
        })
    }

    fn environment(&self) -> [(&str, &Path); 1] {
        [("GIT_ALTERNATE_OBJECT_DIRECTORIES", self.alternate.as_path())]
    }
}

impl Drop for TemporaryRepository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ClasseDuree;

    fn delegation() -> Delegation {
        Delegation::nouvelle(
            Uuid::new_v4(),
            "reviewer",
            "relire",
            ClasseDuree::Courte,
            "revue",
        )
        .unwrap()
    }

    #[test]
    fn sortie_distante_ambigue_est_refusee() {
        let output = b"1111111111111111111111111111111111111111\trefs/heads/x\n\
                       2222222222222222222222222222222222222222\trefs/heads/x\n";
        assert_eq!(
            parse_remote_head(output, "refs/heads/x"),
            Err(ReviewContinuityError::RemoteOutputInvalid)
        );
    }

    #[test]
    fn cible_absente_et_verdict_absent_restent_deux_etats_non_verts() {
        let ordinary = delegation();
        assert_eq!(
            observe_delegation_review(None, &ordinary, None).state,
            ReviewContinuityState::TargetAbsent
        );

        let review = delegation()
            .pour_revue(ReviewTarget {
                target_ref: "origin/session-fixture".to_string(),
                expected_head: "1".repeat(40),
            })
            .unwrap();
        assert_eq!(
            observe_delegation_review(None, &review, None).state,
            ReviewContinuityState::VerdictAbsent
        );
    }

    #[test]
    fn verdict_durable_sans_cible_reste_inobservable_sans_interroger_git() {
        let ordinary = delegation();
        let stored = StoredReviewVerdict {
            objective_id: ordinary.objectif_id,
            delegation_id: ordinary.id,
            evidence: ReviewVerdictEvidence {
                verdict: ReviewVerdict::Approve,
                target_ref: "origin/session-fixture".to_string(),
                expected_head: "1".repeat(40),
                measured_head: "1".repeat(40),
                observed_target_head: "1".repeat(40),
            },
        };
        assert!(ordinary.review_target.is_none());
        assert!(stored.evidence.is_valid());

        let mut git_observations = 0;
        let observation = observe_delegation_review_with(&ordinary, Some(&stored), |_, _| {
            git_observations += 1;
            Ok(ReviewHeadRelation::StillAncestor {
                observed_head: "1".repeat(40),
            })
        });

        assert_eq!(git_observations, 0);
        assert_eq!(observation.state, ReviewContinuityState::Unobservable);
        assert_eq!(
            observation.reason,
            Some(ReviewUnobservableReason::VerdictWithoutTarget)
        );
    }
}
