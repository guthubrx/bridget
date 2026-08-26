//! Calcul pur de la carte de criticité et du régime proposé.
//!
//! Ce module ne lit ni dépôt, ni catalogue, ni base : les adaptateurs lui
//! remettent un instantané borné. Les sorties ne recopient aucun contenu
//! source, seulement des identifiants de règles, de faits et de chemins.

use crate::catalogue::Severity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{self, Write};

/// Régimes fermés, ordonnés du moins au plus exigeant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReviewRegime {
    #[serde(rename = "revue_simple")]
    Simple,
    #[serde(rename = "jury_1_plus_1")]
    JuryOnePlusOne,
    #[serde(rename = "jury_2x2")]
    JuryTwoByTwo,
}

impl ReviewRegime {
    pub const ALL: [Self; 3] = [Self::Simple, Self::JuryOnePlusOne, Self::JuryTwoByTwo];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "revue_simple",
            Self::JuryOnePlusOne => "jury_1_plus_1",
            Self::JuryTwoByTwo => "jury_2x2",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|regime| regime.as_str() == value)
    }
}

/// Graine générique identique pour chaque projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedRule {
    PersistenceSchema,
    ProtocolFormat,
    AuthorizationPermissions,
    ExternalInputDurableWrite,
}

impl SeedRule {
    /// Cette table fermée garantit que la carte ne naît jamais sans règle.
    pub const ALL: [Self; 4] = [
        Self::PersistenceSchema,
        Self::ProtocolFormat,
        Self::AuthorizationPermissions,
        Self::ExternalInputDurableWrite,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PersistenceSchema => "persistence_schema",
            Self::ProtocolFormat => "protocol_format",
            Self::AuthorizationPermissions => "authorization_permissions",
            Self::ExternalInputDurableWrite => "external_input_durable_write",
        }
    }
}

/// Voie ayant produit une preuve de criticité.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Seed,
    DiffContent,
    Registry,
    Contract,
    SelfProtection,
}

/// Origine fermée d'une citation non résolue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationSource {
    Registry,
    Contract,
}

/// Fait structuré persistant, sans contenu de diff ou de constat.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriticalEvidence {
    pub kind: EvidenceKind,
    pub source_id: String,
    pub rule: Option<SeedRule>,
}

/// Zone élue, toujours exprimée par son chemin complet relatif au dépôt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriticalZone {
    pub path: String,
    pub evidence: Vec<CriticalEvidence>,
}

/// Dette visible : le jeton brut est remplacé par son empreinte.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnresolvedCitation {
    pub source_kind: CitationSource,
    pub source_id: String,
    pub token_hash: String,
    pub candidates: Vec<String>,
}

/// Chemin suivi au commit de tête et nombre de lignes de son blob.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedPath {
    pub path: String,
    pub line_count: usize,
}

/// Modification déjà mesurée entre deux objets Git immuables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub patch: String,
    pub head_content: Option<String>,
}

/// Constat ouvert projeté depuis le registre déclaré.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryFinding {
    pub id: String,
    pub severity: Severity,
    pub text: String,
}

/// Document contractuel lu dans l'arbre du commit de tête.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDocument {
    pub source_path: String,
    pub content: String,
}

/// Univers borné remis au calculateur pur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositorySnapshot {
    pub paths: Vec<TrackedPath>,
    pub changes: Vec<FileChange>,
    pub contracts: Vec<ContractDocument>,
    pub open_findings: Vec<RegistryFinding>,
}

/// Résultat déterministe du calcul pour une paire base/tête.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriticalityMap {
    pub seed_rules: Vec<SeedRule>,
    pub zones: Vec<CriticalZone>,
    pub unresolved_citations: Vec<UnresolvedCitation>,
    pub proposed_regime: ReviewRegime,
    pub critical_changed_paths: Vec<String>,
}

/// Charge fermée de soumission. L'auteur vient de l'enveloppe attestée et le
/// chemin local du dépôt vient de la configuration, jamais de cette charge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewLotSubmitPayload {
    pub project_id: String,
    pub branch_ref: String,
    pub base: String,
    pub head: String,
}

/// Charge fermée de décision. L'écart entre les deux régimes est la donnée ;
/// aucun motif libre n'appartient au contrat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRegimeSelectPayload {
    pub submission_id: String,
    pub retained_regime: ReviewRegime,
}

/// Origine de la décision exigée pour une soumission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSubmissionControl {
    ReferentSelectionRequired,
    FixedByReferentDecision,
}

/// États atteignables avant le futur lot d'élection des relecteurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewSubmissionState {
    #[serde(rename = "awaiting_decision")]
    AwaitingDecision,
    #[serde(rename = "decision_recorded")]
    DecisionRecordedElectionUnavailable,
}

/// Sens calculé de l'écart entre proposition et décision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDirection {
    Same,
    Strengthened,
    Lightened,
}

/// États fermés d'un écart. La décision pure ne peut créer que `Open` ; les
/// autres valeurs seront alimentées par les faits exacts des lots dépendants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDeviationState {
    Open,
    Confirmed,
    NotConfirmed,
    Refuted,
}

/// Soumission sans contenu source. Les chemins conservés sont uniquement ceux
/// du diff qui ont réellement déclenché le régime proposé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewSubmission {
    pub submission_id: String,
    pub project_id: String,
    pub author_id: String,
    pub branch_ref: String,
    pub base: String,
    pub head: String,
    pub proposed_regime: ReviewRegime,
    pub retained_regime: Option<ReviewRegime>,
    pub critical_changed_paths: Vec<String>,
    pub unresolved_citations: usize,
    pub control: ReviewSubmissionControl,
}

impl ReviewSubmission {
    /// Projette l'état sans le stocker une seconde fois.
    /// Complexité : O(1).
    pub const fn state(&self) -> ReviewSubmissionState {
        if self.retained_regime.is_some() {
            ReviewSubmissionState::DecisionRecordedElectionUnavailable
        } else {
            ReviewSubmissionState::AwaitingDecision
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewDeviation {
    pub deviation_id: String,
    pub submission_id: String,
    pub direction: ReviewDirection,
    pub state: ReviewDeviationState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRegimeDecision {
    pub decision_id: String,
    pub submission_id: String,
    pub proposed_regime: ReviewRegime,
    pub retained_regime: ReviewRegime,
    pub direction: ReviewDirection,
    pub decided_by: String,
    pub deviation: Option<ReviewDeviation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewSubmissionError {
    InvalidField(&'static str),
}

impl fmt::Display for ReviewSubmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidField(field) => {
                write!(formatter, "champ de soumission invalide : {field}")
            }
        }
    }
}

impl Error for ReviewSubmissionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewDecisionError {
    ReferentMismatch,
    SubmissionMismatch,
    SelfRegimeFixed,
    DecisionAlreadyRecorded,
}

impl fmt::Display for ReviewDecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReferentMismatch => formatter.write_str("émetteur différent du référent"),
            Self::SubmissionMismatch => formatter.write_str("soumission différente"),
            Self::SelfRegimeFixed => formatter.write_str("régime propre fixé par le référent"),
            Self::DecisionAlreadyRecorded => formatter.write_str("décision déjà enregistrée"),
        }
    }
}

impl Error for ReviewDecisionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewError {
    InvalidPath { field: &'static str, path: String },
    DuplicateTrackedPath(String),
    ChangeWithoutPath,
}

impl fmt::Display for ReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath { field, path } => {
                write!(formatter, "chemin {field} non canonique : {path}")
            }
            Self::DuplicateTrackedPath(path) => {
                write!(formatter, "chemin suivi dupliqué : {path}")
            }
            Self::ChangeWithoutPath => formatter.write_str("modification sans chemin"),
        }
    }
}

impl Error for ReviewError {}

const SELF_CRITICAL_PATHS: [&str; 4] = [
    "plugins/maicie/src/config.rs",
    "plugins/maicie/src/review.rs",
    "plugins/maicie/src/review_git.rs",
    "specs/025-carte-criticite-regime/contracts/revue-lot-v1.md",
];

/// Décision explicite du référent : le noyau F38 reste toujours en jury 2×2.
/// Cette valeur n'est ni proposée, ni apprise, ni recalculée par la carte.
pub const F38_FIXED_REGIME: ReviewRegime = ReviewRegime::JuryTwoByTwo;

/// Crée la représentation pure d'une soumission mesurée.
///
/// Complexité : O(Z + C), avec Z preuves de zone et C chemins critiques.
pub fn create_review_submission(
    payload: &ReviewLotSubmitPayload,
    author_id: &str,
    map: &CriticalityMap,
) -> Result<ReviewSubmission, ReviewSubmissionError> {
    validate_submission_field("project_id", &payload.project_id)?;
    validate_submission_field("author_id", author_id)?;
    if !is_full_branch_ref(&payload.branch_ref) {
        return Err(ReviewSubmissionError::InvalidField("branch_ref"));
    }
    if !is_canonical_sha(&payload.base) {
        return Err(ReviewSubmissionError::InvalidField("base"));
    }
    if !is_canonical_sha(&payload.head) {
        return Err(ReviewSubmissionError::InvalidField("head"));
    }

    let fixed_by_referent = map.zones.iter().any(|zone| {
        zone.evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::SelfProtection)
    });
    let proposed_regime = if fixed_by_referent {
        F38_FIXED_REGIME
    } else {
        map.proposed_regime
    };
    let (retained_regime, control) = if fixed_by_referent {
        (
            Some(F38_FIXED_REGIME),
            ReviewSubmissionControl::FixedByReferentDecision,
        )
    } else {
        (None, ReviewSubmissionControl::ReferentSelectionRequired)
    };

    Ok(ReviewSubmission {
        submission_id: deterministic_review_id(
            b"review-submission-v1",
            &[
                &payload.project_id,
                &payload.branch_ref,
                &payload.base,
                &payload.head,
            ],
        ),
        project_id: payload.project_id.clone(),
        author_id: author_id.to_string(),
        branch_ref: payload.branch_ref.clone(),
        base: payload.base.clone(),
        head: payload.head.clone(),
        proposed_regime,
        retained_regime,
        critical_changed_paths: map.critical_changed_paths.clone(),
        unresolved_citations: map.unresolved_citations.len(),
        control,
    })
}

/// Confronte le choix explicite du référent à la proposition persistable.
///
/// Complexité : O(1). La mutation n'arrive qu'après toutes les gardes, afin
/// qu'un refus laisse l'objet appelant bit-à-bit inchangé.
pub fn decide_review_regime(
    submission: &mut ReviewSubmission,
    actor_id: &str,
    referent_id: &str,
    payload: &ReviewRegimeSelectPayload,
) -> Result<ReviewRegimeDecision, ReviewDecisionError> {
    if referent_id.is_empty() || actor_id != referent_id {
        return Err(ReviewDecisionError::ReferentMismatch);
    }
    if payload.submission_id != submission.submission_id {
        return Err(ReviewDecisionError::SubmissionMismatch);
    }
    if submission.control == ReviewSubmissionControl::FixedByReferentDecision {
        return Err(ReviewDecisionError::SelfRegimeFixed);
    }
    if submission.retained_regime.is_some() {
        return Err(ReviewDecisionError::DecisionAlreadyRecorded);
    }

    let direction = match payload.retained_regime.cmp(&submission.proposed_regime) {
        std::cmp::Ordering::Equal => ReviewDirection::Same,
        std::cmp::Ordering::Greater => ReviewDirection::Strengthened,
        std::cmp::Ordering::Less => ReviewDirection::Lightened,
    };
    let decision_id =
        deterministic_review_id(b"review-regime-decision-v1", &[&submission.submission_id]);
    let deviation = (direction != ReviewDirection::Same).then(|| ReviewDeviation {
        deviation_id: deterministic_review_id(b"review-regime-deviation-v1", &[&decision_id]),
        submission_id: submission.submission_id.clone(),
        direction,
        state: ReviewDeviationState::Open,
    });
    let decision = ReviewRegimeDecision {
        decision_id,
        submission_id: submission.submission_id.clone(),
        proposed_regime: submission.proposed_regime,
        retained_regime: payload.retained_regime,
        direction,
        decided_by: actor_id.to_string(),
        deviation,
    };

    submission.retained_regime = Some(payload.retained_regime);
    Ok(decision)
}

#[derive(Debug)]
struct PathIndex {
    exact: BTreeSet<String>,
    suffixes: BTreeMap<String, Vec<String>>,
    line_counts: BTreeMap<String, usize>,
}

#[derive(Debug)]
enum Resolution {
    Resolved(String),
    Unresolved(Vec<String>),
}

#[derive(Default)]
struct RegistryVotes {
    blockers: BTreeSet<String>,
    majors: BTreeSet<String>,
}

/// Calcule la carte en un passage sur chaque collection de l'instantané.
///
/// L'index préconstruit ramène chaque jeton de citation à une recherche dans
/// une table ordonnée. Une ambiguïté n'ajoute jamais de zone ; elle devient
/// une dette empreintée. Les contenus d'entrée meurent à cette frontière.
pub fn calculate_criticality(snapshot: &RepositorySnapshot) -> Result<CriticalityMap, ReviewError> {
    let index = PathIndex::new(&snapshot.paths)?;
    let mut zones: BTreeMap<String, BTreeSet<CriticalEvidence>> = BTreeMap::new();
    let mut unresolved = BTreeSet::new();

    elect_registry(snapshot, &index, &mut zones, &mut unresolved);
    elect_contracts(snapshot, &index, &mut zones, &mut unresolved)?;

    let mut affected_paths = BTreeSet::new();
    let mut self_protected_paths = BTreeSet::new();
    for change in &snapshot.changes {
        let paths = canonical_change_paths(change)?;
        affected_paths.extend(paths.iter().cloned());
        for path in &paths {
            if SELF_CRITICAL_PATHS.contains(&path.as_str()) {
                self_protected_paths.insert(path.clone());
                add_evidence(
                    &mut zones,
                    path,
                    CriticalEvidence {
                        kind: EvidenceKind::SelfProtection,
                        source_id: "user_fixed_core_regime".to_string(),
                        rule: None,
                    },
                );
            }
        }
        elect_change(change, &paths, &mut zones);
    }

    let zone_paths: BTreeSet<&str> = zones.keys().map(String::as_str).collect();
    let critical_changed_paths: Vec<String> = affected_paths
        .into_iter()
        .filter(|path| zone_paths.contains(path.as_str()))
        .collect();
    let proposed_regime = if self_protected_paths.is_empty() {
        if critical_changed_paths.is_empty() {
            ReviewRegime::Simple
        } else {
            ReviewRegime::JuryOnePlusOne
        }
    } else {
        F38_FIXED_REGIME
    };

    Ok(CriticalityMap {
        seed_rules: SeedRule::ALL.to_vec(),
        zones: zones
            .into_iter()
            .map(|(path, evidence)| CriticalZone {
                path,
                evidence: evidence.into_iter().collect(),
            })
            .collect(),
        unresolved_citations: unresolved.into_iter().collect(),
        proposed_regime,
        critical_changed_paths,
    })
}

impl PathIndex {
    fn new(paths: &[TrackedPath]) -> Result<Self, ReviewError> {
        let mut exact = BTreeSet::new();
        let mut suffixes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut line_counts = BTreeMap::new();

        for tracked in paths {
            let canonical = canonical_path("suivi", &tracked.path)?;
            if !exact.insert(canonical.clone()) {
                return Err(ReviewError::DuplicateTrackedPath(canonical));
            }
            line_counts.insert(canonical.clone(), tracked.line_count);
            let components: Vec<&str> = canonical.split('/').collect();
            for start in 0..components.len() {
                suffixes
                    .entry(components[start..].join("/"))
                    .or_default()
                    .push(canonical.clone());
            }
        }
        for candidates in suffixes.values_mut() {
            candidates.sort();
            candidates.dedup();
        }
        Ok(Self {
            exact,
            suffixes,
            line_counts,
        })
    }

    fn registry(&self, token: &str, line: Option<usize>) -> Option<Resolution> {
        if self.exact.contains(token) {
            return Some(Resolution::Resolved(token.to_string()));
        }
        let candidates = self.suffixes.get(token)?.clone();
        if candidates.len() == 1 {
            return Some(Resolution::Resolved(candidates[0].clone()));
        }
        if let Some(line) = line.filter(|line| *line > 0) {
            let reaching_line: Vec<String> = candidates
                .iter()
                .filter(|path| {
                    self.line_counts
                        .get(*path)
                        .is_some_and(|count| *count >= line)
                })
                .cloned()
                .collect();
            if reaching_line.len() == 1 {
                return Some(Resolution::Resolved(reaching_line[0].clone()));
            }
            if !reaching_line.is_empty() {
                return Some(Resolution::Unresolved(reaching_line));
            }
        }
        Some(Resolution::Unresolved(candidates))
    }

    fn contract(&self, token: &str) -> Option<Resolution> {
        if self.exact.contains(token) {
            return Some(Resolution::Resolved(token.to_string()));
        }
        self.suffixes
            .get(token)
            .cloned()
            .map(Resolution::Unresolved)
    }
}

fn elect_registry(
    snapshot: &RepositorySnapshot,
    index: &PathIndex,
    zones: &mut BTreeMap<String, BTreeSet<CriticalEvidence>>,
    unresolved: &mut BTreeSet<UnresolvedCitation>,
) {
    let mut votes: BTreeMap<String, RegistryVotes> = BTreeMap::new();
    for finding in &snapshot.open_findings {
        if !matches!(finding.severity, Severity::Blocker | Severity::Major) {
            continue;
        }
        let mut finding_paths = BTreeSet::new();
        for token in citation_tokens(&finding.text, index) {
            let Some(resolution) = index.registry(&token.path, token.line) else {
                continue;
            };
            match resolution {
                Resolution::Resolved(path) => {
                    finding_paths.insert(path);
                }
                Resolution::Unresolved(candidates) => {
                    unresolved.insert(unresolved_citation(
                        CitationSource::Registry,
                        &finding.id,
                        &token.raw,
                        candidates,
                    ));
                }
            }
        }
        for path in finding_paths {
            let vote = votes.entry(path).or_default();
            match finding.severity {
                Severity::Blocker => {
                    vote.blockers.insert(finding.id.clone());
                }
                Severity::Major => {
                    vote.majors.insert(finding.id.clone());
                }
                Severity::Minor | Severity::Info => {}
            }
        }
    }

    for (path, vote) in votes {
        if vote.blockers.is_empty() && vote.majors.len() < 2 {
            continue;
        }
        for source_id in vote.blockers.into_iter().chain(vote.majors) {
            add_evidence(
                zones,
                &path,
                CriticalEvidence {
                    kind: EvidenceKind::Registry,
                    source_id,
                    rule: None,
                },
            );
        }
    }
}

fn elect_contracts(
    snapshot: &RepositorySnapshot,
    index: &PathIndex,
    zones: &mut BTreeMap<String, BTreeSet<CriticalEvidence>>,
    unresolved: &mut BTreeSet<UnresolvedCitation>,
) -> Result<(), ReviewError> {
    for contract in &snapshot.contracts {
        let source_path = canonical_path("contrat", &contract.source_path)?;
        if !is_contract_path(&source_path) {
            continue;
        }
        for token in citation_tokens(&contract.content, index) {
            match index.contract(&token.path) {
                Some(Resolution::Resolved(path)) if !is_contract_path(&path) => {
                    add_evidence(
                        zones,
                        &path,
                        CriticalEvidence {
                            kind: EvidenceKind::Contract,
                            source_id: source_path.clone(),
                            rule: None,
                        },
                    );
                }
                Some(Resolution::Resolved(_)) => {}
                Some(Resolution::Unresolved(candidates)) => {
                    unresolved.insert(unresolved_citation(
                        CitationSource::Contract,
                        &source_path,
                        &token.raw,
                        candidates,
                    ));
                }
                None if token.looks_like_path => {
                    unresolved.insert(unresolved_citation(
                        CitationSource::Contract,
                        &source_path,
                        &token.raw,
                        Vec::new(),
                    ));
                }
                None => {}
            }
        }
    }
    Ok(())
}

fn elect_change(
    change: &FileChange,
    paths: &[String],
    zones: &mut BTreeMap<String, BTreeSet<CriticalEvidence>>,
) {
    let Some(elected_path) = change.new_path.as_deref().or(change.old_path.as_deref()) else {
        return;
    };
    let lower_patch = change.patch.to_ascii_lowercase();
    let lower_unit = format!(
        "{}\n{}",
        lower_patch,
        change
            .head_content
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
    );
    let lower_paths: Vec<String> = paths.iter().map(|path| path.to_ascii_lowercase()).collect();

    let persistence_path = lower_paths
        .iter()
        .any(|path| path.contains("migration") || path.ends_with("schema.sql"));
    let persistence_marker = first_marker(
        &lower_patch,
        &[
            "schema_version",
            "alter table",
            "create table",
            "pragma user_version",
        ],
    );
    if persistence_path || persistence_marker.is_some() {
        add_seed(zones, elected_path, SeedRule::PersistenceSchema);
        add_evidence(
            zones,
            elected_path,
            CriticalEvidence {
                kind: EvidenceKind::DiffContent,
                source_id: persistence_marker.unwrap_or("migration_path").to_string(),
                rule: None,
            },
        );
    }

    if lower_paths
        .iter()
        .any(|path| path_has_concept(path, &["protocol", "message", "wire", "envelope"]))
    {
        add_seed(zones, elected_path, SeedRule::ProtocolFormat);
    }
    if lower_paths.iter().any(|path| {
        path_has_concept(
            path,
            &[
                "auth",
                "permission",
                "capability",
                "approval",
                "access_control",
                "acl",
            ],
        )
    }) {
        add_seed(zones, elected_path, SeedRule::AuthorizationPermissions);
    }
    if contains_any(
        &lower_unit,
        &[
            ".recv(",
            "recv(",
            "read_frame",
            "request.body",
            "payload",
            "stdin",
            "socket.read",
            "read_to_end",
        ],
    ) && contains_any(
        &lower_unit,
        &[
            "insert into",
            "fs::write",
            ".write_all(",
            "file::create",
            "openoptions",
            ".append(",
            "persist(",
        ],
    ) {
        add_seed(zones, elected_path, SeedRule::ExternalInputDurableWrite);
    }
}

fn add_seed(zones: &mut BTreeMap<String, BTreeSet<CriticalEvidence>>, path: &str, rule: SeedRule) {
    add_evidence(
        zones,
        path,
        CriticalEvidence {
            kind: EvidenceKind::Seed,
            source_id: "generic_seed".to_string(),
            rule: Some(rule),
        },
    );
}

fn add_evidence(
    zones: &mut BTreeMap<String, BTreeSet<CriticalEvidence>>,
    path: &str,
    evidence: CriticalEvidence,
) {
    zones.entry(path.to_string()).or_default().insert(evidence);
}

fn canonical_change_paths(change: &FileChange) -> Result<Vec<String>, ReviewError> {
    let mut paths = BTreeSet::new();
    if let Some(path) = &change.old_path {
        paths.insert(canonical_path("ancien", path)?);
    }
    if let Some(path) = &change.new_path {
        paths.insert(canonical_path("nouveau", path)?);
    }
    if paths.is_empty() {
        return Err(ReviewError::ChangeWithoutPath);
    }
    Ok(paths.into_iter().collect())
}

fn canonical_path(field: &'static str, raw: &str) -> Result<String, ReviewError> {
    let invalid = raw.is_empty()
        || raw.starts_with('/')
        || raw.contains('\\')
        || raw
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."));
    if invalid {
        return Err(ReviewError::InvalidPath {
            field,
            path: raw.to_string(),
        });
    }
    Ok(raw.to_string())
}

fn is_contract_path(path: &str) -> bool {
    path.starts_with("specs/") && path.contains("/contracts/")
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CitationToken {
    raw: String,
    path: String,
    line: Option<usize>,
    looks_like_path: bool,
}

fn citation_tokens(text: &str, index: &PathIndex) -> Vec<CitationToken> {
    let mut tokens = BTreeSet::new();
    for raw in text.split(|character: char| {
        !(character.is_alphanumeric() || matches!(character, '/' | '.' | '_' | '-' | ':'))
    }) {
        let raw = raw.trim_end_matches('.');
        if raw.is_empty() || raw.contains("://") {
            continue;
        }
        let (path, line) = split_line_suffix(raw);
        if path.is_empty() || path.starts_with('/') || path.contains("..") {
            continue;
        }
        let known = index.exact.contains(path) || index.suffixes.contains_key(path);
        let looks_like_path = path.contains('/')
            && path
                .rsplit('/')
                .next()
                .is_some_and(|name| name.contains('.'));
        if known || looks_like_path {
            tokens.insert(CitationToken {
                raw: raw.to_string(),
                path: path.to_string(),
                line,
                looks_like_path,
            });
        }
    }
    tokens.into_iter().collect()
}

fn split_line_suffix(token: &str) -> (&str, Option<usize>) {
    let Some((path, suffix)) = token.rsplit_once(':') else {
        return (token, None);
    };
    match suffix.parse::<usize>() {
        Ok(line) => (path, Some(line)),
        Err(_) => (token, None),
    }
}

fn unresolved_citation(
    source_kind: CitationSource,
    source_id: &str,
    raw_token: &str,
    candidates: Vec<String>,
) -> UnresolvedCitation {
    UnresolvedCitation {
        source_kind,
        source_id: source_id.to_string(),
        token_hash: sha256_hex(raw_token.as_bytes()),
        candidates,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    lowercase_hex(&digest)
}

fn deterministic_review_id(namespace: &[u8], fields: &[&str]) -> String {
    let mut digest = Sha256::new();
    digest.update((namespace.len() as u64).to_be_bytes());
    digest.update(namespace);
    for field in fields {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field.as_bytes());
    }
    lowercase_hex(&digest.finalize())
}

fn lowercase_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("écriture dans une String");
    }
    output
}

fn validate_submission_field(
    field: &'static str,
    value: &str,
) -> Result<(), ReviewSubmissionError> {
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_control) {
        Err(ReviewSubmissionError::InvalidField(field))
    } else {
        Ok(())
    }
}

pub(crate) fn is_canonical_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn is_full_branch_ref(value: &str) -> bool {
    let allowed_prefix = value.starts_with("refs/heads/") || value.starts_with("refs/remotes/");
    allowed_prefix
        && !value.ends_with('/')
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && !value.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || matches!(character, '~' | '^' | ':' | '?' | '*' | '[' | '\\')
        })
}

fn first_marker<'a>(text: &str, markers: &'a [&str]) -> Option<&'a str> {
    markers.iter().copied().find(|marker| text.contains(marker))
}

fn contains_any(text: &str, markers: &[&str]) -> bool {
    first_marker(text, markers).is_some()
}

fn path_has_concept(path: &str, concepts: &[&str]) -> bool {
    path.split('/')
        .any(|component| concepts.iter().any(|concept| component.contains(concept)))
}
