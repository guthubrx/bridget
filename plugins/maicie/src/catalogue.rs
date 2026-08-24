//! Journal canonique du catalogue Maicie (session 017).
//!
//! Autorité unique : le fichier déclaré, append-only. `registre list` est la
//! seule vue humaine. Aucune écriture dans les plans, `tasks.md` ou issues
//! de l'hôte. Les références restent des identifiants exacts.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

const CATALOGUE_VERSION: u32 = 1;
const FORBIDDEN_BASENAMES: &[&str] = &[
    "tasks.md",
    "plan.md",
    "spec.md",
    "implementation.md",
    "quickstart.md",
];

/// Erreurs fermées du journal de catalogue.
#[derive(Debug)]
pub enum CatalogueError {
    Io(io::Error),
    Format(String),
    IdempotenceConflict { id: String },
    PathRefuse { reason: String },
    ReferenceInconnue { field: &'static str, id: String },
    TransitionInvalide(String),
}

impl fmt::Display for CatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(f, "erreur I/O catalogue: {source}"),
            Self::Format(reason) => write!(f, "catalogue refusé: {reason}"),
            Self::IdempotenceConflict { id } => {
                write!(
                    f,
                    "catalogue refusé: id '{id}' déjà présent avec des octets divergents"
                )
            }
            Self::PathRefuse { reason } => write!(f, "chemin catalogue refusé: {reason}"),
            Self::ReferenceInconnue { field, id } => {
                write!(f, "référence inconnue ({field}={id})")
            }
            Self::TransitionInvalide(reason) => write!(f, "transition refusée: {reason}"),
        }
    }
}

impl std::error::Error for CatalogueError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for CatalogueError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

impl From<serde_json::Error> for CatalogueError {
    fn from(source: serde_json::Error) -> Self {
        Self::Format(source.to_string())
    }
}

/// Sévérité déclarée par l'émetteur — jamais calculée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Blocker,
    Major,
    Minor,
    Info,
}

impl Severity {
    fn sort_rank(self) -> u8 {
        match self {
            Self::Blocker => 0,
            Self::Major => 1,
            Self::Minor => 2,
            Self::Info => 3,
        }
    }
}

/// Nature fermée d'une mission source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionSourceKind {
    Mission,
    Incident,
    Review,
    Gate,
}

/// Source typée d'un constat. Un gate porte explicitement son échec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MissionSource {
    pub kind: MissionSourceKind,
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed: Option<bool>,
}

impl MissionSource {
    fn validate(&self) -> Result<(), CatalogueError> {
        if self.id.trim().is_empty() {
            return Err(CatalogueError::Format(
                "mission_source.id ne peut pas être vide".into(),
            ));
        }
        match self.kind {
            MissionSourceKind::Gate => {
                if self.failed != Some(true) {
                    return Err(CatalogueError::Format(
                        "une source gate doit déclarer failed=true".into(),
                    ));
                }
            }
            MissionSourceKind::Mission
            | MissionSourceKind::Incident
            | MissionSourceKind::Review => {
                if self.failed.is_some() {
                    return Err(CatalogueError::Format(
                        "failed n'est admis que pour une source gate".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn is_failed_gate(&self) -> bool {
        matches!(self.kind, MissionSourceKind::Gate) && self.failed == Some(true)
    }
}

/// Entrée `add` du journal v1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddEntry {
    pub v: u32,
    pub kind: AddKind,
    pub id: String,
    pub date: String,
    pub mission_source: MissionSource,
    pub severity: Severity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence_of: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddKind {
    Add,
}

/// Transition attestée `open → delivered`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionEntry {
    pub v: u32,
    pub kind: TransitionKind,
    pub constat_id: String,
    pub from: ConstatState,
    pub to: ConstatState,
    pub objective_id: String,
    pub observed_at: String,
    pub trigger: TransitionTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionKind {
    Transition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstatState {
    Open,
    Delivered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionTrigger {
    ObjectiveClosed,
}

/// Entrée historique incomplète, hors liste ouverte, comptée dans P.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingQualificationEntry {
    pub v: u32,
    pub kind: PendingKind,
    pub id: String,
    pub provenance_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingKind {
    PendingQualification,
}

/// Ligne fermée du journal.
///
/// Pas de `Deserialize` : la lecture passe exclusivement par le dispatch fermé
/// `parse_closed_line` (kind explicite). Un `#[serde(untagged)]` ici serait
/// du code mort et un chemin de contournement trompeur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogueEntry {
    Add(AddEntry),
    Transition(TransitionEntry),
    PendingQualification(PendingQualificationEntry),
}

impl CatalogueEntry {
    fn validate_standalone(&self) -> Result<(), CatalogueError> {
        match self {
            Self::Add(entry) => validate_add_shape(entry),
            Self::Transition(entry) => validate_transition_shape(entry),
            Self::PendingQualification(entry) => validate_pending_shape(entry),
        }
    }

    fn identity_key(&self) -> String {
        match self {
            Self::Add(entry) => format!("add:{}", entry.id),
            Self::Transition(entry) => format!(
                "transition:{}:{}:{:?}",
                entry.constat_id, entry.objective_id, entry.trigger
            ),
            Self::PendingQualification(entry) => format!("pending:{}", entry.id),
        }
    }
}

/// Issue d'un append sous verrou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendOutcome {
    Appended,
    IdempotentNoop,
}

/// Compte-rendu d'une migration de corpus prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationReport {
    pub read: usize,
    pub appended: usize,
    pub skipped: usize,
}

/// État dérivé d'un constat dans la projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedState {
    Open,
    Delivered,
    PendingQualification,
}

/// Ligne ouverte de la vue `registre list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenConstatView {
    pub id: String,
    pub date: String,
    pub severity: Severity,
    pub recurrence_of: Option<String>,
    pub gate_failed: bool,
    pub text: String,
    pub mission_source: MissionSource,
}

/// Entrée encore en attente de qualification humaine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingView {
    pub id: String,
    pub provenance_id: String,
    pub text: String,
}

/// Pied de page déterministe N/M/K/P.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistreFooter {
    pub ouverts: usize,
    pub recurrents: usize,
    pub gates_rates: usize,
    pub pending_qualification: usize,
}

/// Vue pure du registre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistreView {
    pub ouverts: Vec<OpenConstatView>,
    /// Pendings non encore qualifiés (aucun `add` de même `id`).
    pub attente: Vec<PendingView>,
    pub footer: RegistreFooter,
}

/// Journal verrouillé du catalogue déclaré.
pub struct CatalogueJournal {
    path: PathBuf,
    file: File,
}

impl CatalogueJournal {
    /// Ouvre (ou crée) le fichier régulier déclaré, sous verrou exclusif.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CatalogueError> {
        let path = path.as_ref();
        validate_catalogue_path(path, None)?;
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .mode(0o644)
            .open(path)?;
        refuse_if_symlink(path)?;
        let journal = Self {
            path: path.to_path_buf(),
            file,
        };
        journal.lock_exclusive()?;
        Ok(journal)
    }

    /// Chemin du journal déclaré.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Lit toutes les lignes JSON fermées, dans l'ordre physique.
    pub fn read_entries(&mut self) -> Result<Vec<CatalogueEntry>, CatalogueError> {
        Ok(self.read_journal()?.entries)
    }

    /// Lit le journal et remonte un éventuel avertissement de queue arrachée.
    pub fn read_journal(&mut self) -> Result<ParsedJournal, CatalogueError> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        self.file.read_to_end(&mut bytes)?;
        parse_journal_bytes(&bytes)
    }

    /// Ajoute un `add` sous le même verrou : collision, no-op ou append.
    pub fn append_add(&mut self, entry: AddEntry) -> Result<AppendOutcome, CatalogueError> {
        let entry = CatalogueEntry::Add(entry);
        entry.validate_standalone()?;
        self.append_entry(entry)
    }

    /// Ajoute une transition idempotente sous le même verrou.
    pub fn append_transition(
        &mut self,
        entry: TransitionEntry,
    ) -> Result<AppendOutcome, CatalogueError> {
        let entry = CatalogueEntry::Transition(entry);
        entry.validate_standalone()?;
        self.append_entry(entry)
    }

    /// Ajoute une entrée de migration en attente de qualification.
    pub fn append_pending(
        &mut self,
        entry: PendingQualificationEntry,
    ) -> Result<AppendOutcome, CatalogueError> {
        let entry = CatalogueEntry::PendingQualification(entry);
        entry.validate_standalone()?;
        self.append_entry(entry)
    }

    /// Migre un corpus prose intermédiaire `{provenance_id, text, …}` vers le
    /// journal. Parse intégral avant toute écriture : une ligne mal formée
    /// refuse tout le lot. Une provenance déjà présente n'est jamais
    /// redoublée (rejeu prudent du référent).
    pub fn migrate_prose_file(
        &mut self,
        prose_path: impl AsRef<Path>,
    ) -> Result<MigrationReport, CatalogueError> {
        let prose_path = prose_path.as_ref();
        let bytes = fs::read(prose_path)?;
        let records = parse_prose_corpus(&bytes)?;
        let mut appended = 0usize;
        let mut skipped = 0usize;
        for record in &records {
            let entry = migrate_prose_record(record)?;
            let outcome = match entry {
                CatalogueEntry::PendingQualification(pending) => {
                    self.append_pending_dedup_provenance(pending)?
                }
                CatalogueEntry::Add(add) => self.append_add(add)?,
                CatalogueEntry::Transition(_) => {
                    return Err(CatalogueError::Format(
                        "migration : une transition ne peut pas naître d'un corpus prose".into(),
                    ));
                }
            };
            match outcome {
                AppendOutcome::Appended => appended += 1,
                AppendOutcome::IdempotentNoop => skipped += 1,
            }
        }
        Ok(MigrationReport {
            read: records.len(),
            appended,
            skipped,
        })
    }

    /// Qualifie une entrée `pending_qualification` en appendant un `add`.
    ///
    /// Le texte est recopié VERBATIM depuis le pending ; la sévérité et la
    /// source sont fournies par l'humain. Ce n'est pas une `transition`
    /// (réservée à open→delivered) : la conception gelée ne connaît que
    /// `add` pour un constat complet. La ligne pending reste au journal
    /// (append-only) ; la projection cesse de la compter dans P dès qu'un
    /// `add` de même `id` existe.
    pub fn qualify_pending(
        &mut self,
        pending_id: &str,
        severity: Severity,
        mission_source: MissionSource,
        date: String,
    ) -> Result<AppendOutcome, CatalogueError> {
        let existing = self.read_entries()?;
        let pending = existing
            .iter()
            .find_map(|entry| match entry {
                CatalogueEntry::PendingQualification(pending) if pending.id == pending_id => {
                    Some(pending.clone())
                }
                _ => None,
            })
            .ok_or_else(|| CatalogueError::ReferenceInconnue {
                field: "pending_id",
                id: pending_id.to_string(),
            })?;
        let add = AddEntry {
            v: CATALOGUE_VERSION,
            kind: AddKind::Add,
            id: pending.id.clone(),
            date,
            mission_source,
            severity,
            recurrence_of: None,
            text: pending.text.clone(),
        };
        validate_add_shape(&add)?;
        self.append_entry_with_existing(CatalogueEntry::Add(add), &existing)
    }

    fn append_pending_dedup_provenance(
        &mut self,
        entry: PendingQualificationEntry,
    ) -> Result<AppendOutcome, CatalogueError> {
        let existing = self.read_entries()?;
        if existing.iter().any(|previous| match previous {
            CatalogueEntry::PendingQualification(pending) => {
                pending.provenance_id == entry.provenance_id
            }
            _ => false,
        }) {
            return Ok(AppendOutcome::IdempotentNoop);
        }
        let wrapped = CatalogueEntry::PendingQualification(entry);
        wrapped.validate_standalone()?;
        self.append_entry_with_existing(wrapped, &existing)
    }

    fn append_entry(&mut self, entry: CatalogueEntry) -> Result<AppendOutcome, CatalogueError> {
        let existing = self.read_entries()?;
        self.append_entry_with_existing(entry, &existing)
    }

    fn append_entry_with_existing(
        &mut self,
        entry: CatalogueEntry,
        existing: &[CatalogueEntry],
    ) -> Result<AppendOutcome, CatalogueError> {
        validate_entry_against_journal(&entry, existing)?;
        let line = canonical_line(&entry)?;
        let key = entry.identity_key();
        for previous in existing {
            if previous.identity_key() != key {
                continue;
            }
            let previous_line = canonical_line(previous)?;
            if previous_line == line {
                return Ok(AppendOutcome::IdempotentNoop);
            }
            let id = match &entry {
                CatalogueEntry::Add(add) => add.id.clone(),
                CatalogueEntry::Transition(transition) => transition.constat_id.clone(),
                CatalogueEntry::PendingQualification(pending) => pending.id.clone(),
            };
            return Err(CatalogueError::IdempotenceConflict { id });
        }
        // Un seul write : ligne + LF. O_APPEND n'est atomique que pour un
        // appel sous la taille de tampon ; deux appels exposeraient une
        // fenêtre où le journal se termine sans terminateur.
        let mut record = line;
        record.push('\n');
        self.file.write_all(record.as_bytes())?;
        self.file.flush()?;
        self.file.sync_data()?;
        Ok(AppendOutcome::Appended)
    }

    fn lock_exclusive(&self) -> Result<(), CatalogueError> {
        let result = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_EX) };
        if result != 0 {
            return Err(CatalogueError::Io(io::Error::last_os_error()));
        }
        Ok(())
    }
}

impl Drop for CatalogueJournal {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

/// Valide un chemin avant toute écriture.
///
/// `project_root` optionnel force le catalogue à rester sous le projet hôte.
/// Les basenames d'artefacts de workflow (`tasks.md`, plans, specs) sont
/// toujours refusés. Un symlink est refusé même s'il pointe vers un régulier.
pub fn validate_catalogue_path(
    path: &Path,
    project_root: Option<&Path>,
) -> Result<(), CatalogueError> {
    let basename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if FORBIDDEN_BASENAMES
        .iter()
        .any(|forbidden| basename.eq_ignore_ascii_case(forbidden))
    {
        return Err(CatalogueError::PathRefuse {
            reason: format!("écriture interdite vers l'artefact hôte '{basename}'"),
        });
    }
    if let Some(root) = project_root {
        let root = root
            .canonicalize()
            .map_err(|source| CatalogueError::PathRefuse {
                reason: format!("racine projet illisible: {source}"),
            })?;
        let candidate = if path.exists() {
            path.canonicalize()
                .map_err(|source| CatalogueError::PathRefuse {
                    reason: format!("chemin illisible: {source}"),
                })?
        } else {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let file_name = path.file_name().ok_or_else(|| CatalogueError::PathRefuse {
                reason: "chemin sans nom de fichier".into(),
            })?;
            let parent = if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            };
            if parent.exists() {
                parent
                    .canonicalize()
                    .map_err(|source| CatalogueError::PathRefuse {
                        reason: format!("parent illisible: {source}"),
                    })?
                    .join(file_name)
            } else {
                path.to_path_buf()
            }
        };
        if !candidate.starts_with(&root) {
            return Err(CatalogueError::PathRefuse {
                reason: "chemin hors du projet hôte déclaré".into(),
            });
        }
    }
    if path.exists() {
        refuse_if_symlink(path)?;
        let metadata = fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(CatalogueError::PathRefuse {
                reason: "le catalogue déclaré doit être un fichier régulier".into(),
            });
        }
    }
    Ok(())
}

fn refuse_if_symlink(path: &Path) -> Result<(), CatalogueError> {
    let meta = fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Err(CatalogueError::PathRefuse {
            reason: "symlink refusé pour le catalogue déclaré".into(),
        });
    }
    Ok(())
}

/// Résultat d'une lecture de journal : entrées valides et éventuelle queue
/// arrachée (crash entre octets de ligne et LF).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedJournal {
    pub entries: Vec<CatalogueEntry>,
    /// Présent si le fichier se termine par une ligne sans LF et invalide :
    /// la ligne est ignorée, le reste du journal reste lisible.
    pub torn_tail_warning: Option<String>,
}

/// Parse un journal entier (en-tête Markdown éventuel ignoré hors lignes JSON).
///
/// Une ligne invalide au milieu (terminée par LF) refuse le journal. Une
/// dernière ligne sans terminateur et invalide est seulement signalée : c'est
/// une écriture arrachée, pas une corruption du corpus déjà scellé.
pub fn parse_journal_bytes(bytes: &[u8]) -> Result<ParsedJournal, CatalogueError> {
    let text = std::str::from_utf8(bytes).map_err(|source| {
        CatalogueError::Format(format!("journal non UTF-8: {source}"))
    })?;
    let ends_with_lf = text.ends_with('\n');
    let mut parts: Vec<&str> = text.split('\n').collect();
    if ends_with_lf {
        // `split` laisse un dernier segment vide après le LF final.
        let _ = parts.pop();
    }
    let (complete_lines, torn_tail) = if ends_with_lf || parts.is_empty() {
        (parts.as_slice(), None)
    } else {
        let (tail, rest) = parts.split_last().expect("parts non vide");
        (rest, Some(*tail))
    };

    let mut entries = Vec::new();
    for (index, line) in complete_lines.iter().enumerate() {
        push_journal_line(line, index + 1, &mut entries)?;
    }

    let mut torn_tail_warning = None;
    if let Some(tail) = torn_tail {
        let trimmed = tail.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("<!--") {
            match try_parse_journal_line(trimmed) {
                Ok(entry) => entries.push(entry),
                Err(_) => {
                    torn_tail_warning = Some(format!(
                        "ligne finale arrachée ignorée (sans terminateur LF) : {trimmed}"
                    ));
                }
            }
        }
    }

    Ok(ParsedJournal {
        entries,
        torn_tail_warning,
    })
}

fn push_journal_line(
    line: &str,
    line_number: usize,
    entries: &mut Vec<CatalogueEntry>,
) -> Result<(), CatalogueError> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("<!--") {
        return Ok(());
    }
    if !trimmed.starts_with('{') {
        return Err(CatalogueError::Format(format!(
            "ligne {line_number}: prose hors format fermé refusée"
        )));
    }
    let entry = parse_closed_line(trimmed).map_err(|error| match error {
        CatalogueError::Format(reason) => {
            CatalogueError::Format(format!("ligne {line_number}: {reason}"))
        }
        other => other,
    })?;
    entries.push(entry);
    Ok(())
}

fn try_parse_journal_line(trimmed: &str) -> Result<CatalogueEntry, CatalogueError> {
    if !trimmed.starts_with('{') {
        return Err(CatalogueError::Format(
            "prose hors format fermé refusée".into(),
        ));
    }
    parse_closed_line(trimmed)
}

/// Parse une ligne JSON fermée ; type/champ inconnu → refus.
pub fn parse_closed_line(line: &str) -> Result<CatalogueEntry, CatalogueError> {
    let value: Value = serde_json::from_str(line)?;
    let object = value
        .as_object()
        .ok_or_else(|| CatalogueError::Format("ligne JSON non objet".into()))?;
    let kind = object
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| CatalogueError::Format("champ kind manquant".into()))?;
    let entry = match kind {
        "add" => {
            let entry: AddEntry = serde_json::from_value(value)?;
            CatalogueEntry::Add(entry)
        }
        "transition" => {
            let entry: TransitionEntry = serde_json::from_value(value)?;
            CatalogueEntry::Transition(entry)
        }
        "pending_qualification" => {
            let entry: PendingQualificationEntry = serde_json::from_value(value)?;
            CatalogueEntry::PendingQualification(entry)
        }
        other => {
            return Err(CatalogueError::Format(format!(
                "kind inconnu '{other}' (fermé: add|transition|pending_qualification)"
            )));
        }
    };
    entry.validate_standalone()?;
    Ok(entry)
}

fn canonical_line(entry: &CatalogueEntry) -> Result<String, CatalogueError> {
    let value = match entry {
        CatalogueEntry::Add(entry) => serde_json::to_value(entry)?,
        CatalogueEntry::Transition(entry) => serde_json::to_value(entry)?,
        CatalogueEntry::PendingQualification(entry) => serde_json::to_value(entry)?,
    };
    Ok(serde_json::to_string(&value)?)
}

fn validate_add_shape(entry: &AddEntry) -> Result<(), CatalogueError> {
    if entry.v != CATALOGUE_VERSION {
        return Err(CatalogueError::Format(format!(
            "version catalogue attendue {CATALOGUE_VERSION}, reçue {}",
            entry.v
        )));
    }
    if entry.id.trim().is_empty() {
        return Err(CatalogueError::Format("id de constat vide".into()));
    }
    if entry.text.is_empty() {
        return Err(CatalogueError::Format("text verbatim absent".into()));
    }
    validate_rfc3339_with_offset(&entry.date)?;
    entry.mission_source.validate()?;
    if let Some(parent) = &entry.recurrence_of {
        if parent.trim().is_empty() {
            return Err(CatalogueError::Format("recurrence_of vide".into()));
        }
        if parent == &entry.id {
            return Err(CatalogueError::Format(
                "recurrence_of ne peut pas former une boucle sur soi".into(),
            ));
        }
    }
    Ok(())
}

fn validate_transition_shape(entry: &TransitionEntry) -> Result<(), CatalogueError> {
    if entry.v != CATALOGUE_VERSION {
        return Err(CatalogueError::Format(format!(
            "version catalogue attendue {CATALOGUE_VERSION}, reçue {}",
            entry.v
        )));
    }
    if entry.constat_id.trim().is_empty() || entry.objective_id.trim().is_empty() {
        return Err(CatalogueError::Format(
            "constat_id et objective_id sont obligatoires".into(),
        ));
    }
    if entry.from != ConstatState::Open || entry.to != ConstatState::Delivered {
        return Err(CatalogueError::TransitionInvalide(
            "seule la transition open→delivered est admise en v1".into(),
        ));
    }
    validate_rfc3339_with_offset(&entry.observed_at)?;
    Ok(())
}

fn validate_pending_shape(entry: &PendingQualificationEntry) -> Result<(), CatalogueError> {
    if entry.v != CATALOGUE_VERSION {
        return Err(CatalogueError::Format(format!(
            "version catalogue attendue {CATALOGUE_VERSION}, reçue {}",
            entry.v
        )));
    }
    if entry.id.trim().is_empty() || entry.provenance_id.trim().is_empty() {
        return Err(CatalogueError::Format(
            "id et provenance_id obligatoires pour pending_qualification".into(),
        ));
    }
    if entry.text.is_empty() {
        return Err(CatalogueError::Format(
            "texte verbatim obligatoire pour pending_qualification".into(),
        ));
    }
    Ok(())
}

fn validate_entry_against_journal(
    entry: &CatalogueEntry,
    existing: &[CatalogueEntry],
) -> Result<(), CatalogueError> {
    let known_constats: BTreeSet<String> = existing
        .iter()
        .filter_map(|line| match line {
            CatalogueEntry::Add(add) => Some(add.id.clone()),
            CatalogueEntry::PendingQualification(pending) => Some(pending.id.clone()),
            CatalogueEntry::Transition(_) => None,
        })
        .collect();
    match entry {
        CatalogueEntry::Add(add) => {
            if let Some(parent) = &add.recurrence_of
                && !known_constats.contains(parent)
            {
                return Err(CatalogueError::ReferenceInconnue {
                    field: "recurrence_of",
                    id: parent.clone(),
                });
            }
        }
        CatalogueEntry::Transition(transition) => {
            if !known_constats.contains(&transition.constat_id) {
                return Err(CatalogueError::ReferenceInconnue {
                    field: "constat_id",
                    id: transition.constat_id.clone(),
                });
            }
        }
        CatalogueEntry::PendingQualification(_) => {}
    }
    Ok(())
}

/// Horodatage RFC 3339 / ISO-8601 avec fuseau explicite (Z ou ±HH:MM).
pub fn validate_rfc3339_with_offset(value: &str) -> Result<(), CatalogueError> {
    let bytes = value.as_bytes();
    if bytes.len() < 20 {
        return Err(CatalogueError::Format(format!(
            "horodatage RFC 3339 invalide ou sans fuseau: {value}"
        )));
    }
    // YYYY-MM-DDTHH:MM:SS
    let head_ok = bytes[4] == b'-'
        && bytes[7] == b'-'
        && (bytes[10] == b'T' || bytes[10] == b't')
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[17..19].iter().all(u8::is_ascii_digit);
    if !head_ok {
        return Err(CatalogueError::Format(format!(
            "horodatage RFC 3339 invalide: {value}"
        )));
    }
    let mut index = 19;
    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        let start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index == start {
            return Err(CatalogueError::Format(format!(
                "fraction d'horodatage vide: {value}"
            )));
        }
    }
    let tz = &value[index..];
    let tz_ok = tz == "Z"
        || tz == "z"
        || ((tz.starts_with('+') || tz.starts_with('-'))
            && tz.len() == 6
            && tz.as_bytes()[3] == b':'
            && tz[1..3].bytes().all(|b| b.is_ascii_digit())
            && tz[4..6].bytes().all(|b| b.is_ascii_digit()));
    if !tz_ok {
        return Err(CatalogueError::Format(format!(
            "horodatage sans fuseau explicite: {value}"
        )));
    }
    Ok(())
}

/// Entrée prose de migration : texte + provenance, champs optionnels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProseMigrationRecord {
    pub provenance_id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mission_source: Option<MissionSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence_of: Option<String>,
}

/// Migre une entrée prose sans inventer sévérité, source ni récurrence.
pub fn migrate_prose_record(
    record: &ProseMigrationRecord,
) -> Result<CatalogueEntry, CatalogueError> {
    if record.text.is_empty() || record.provenance_id.trim().is_empty() {
        return Err(CatalogueError::Format(
            "migration : text et provenance_id obligatoires".into(),
        ));
    }
    let complete = record.date.is_some()
        && record.mission_source.is_some()
        && record.severity.is_some()
        && record.id.as_ref().is_some_and(|id| !id.trim().is_empty());
    if complete {
        let add = AddEntry {
            v: CATALOGUE_VERSION,
            kind: AddKind::Add,
            id: record.id.clone().expect("id présent si complete"),
            date: record.date.clone().expect("date présente"),
            mission_source: record.mission_source.clone().expect("source présente"),
            severity: record.severity.expect("sévérité présente"),
            recurrence_of: record.recurrence_of.clone(),
            text: record.text.clone(),
        };
        validate_add_shape(&add)?;
        Ok(CatalogueEntry::Add(add))
    } else {
        let id = record
            .id
            .clone()
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| format!("pending:{}", record.provenance_id));
        let pending = PendingQualificationEntry {
            v: CATALOGUE_VERSION,
            kind: PendingKind::PendingQualification,
            id,
            provenance_id: record.provenance_id.clone(),
            text: record.text.clone(),
        };
        validate_pending_shape(&pending)?;
        Ok(CatalogueEntry::PendingQualification(pending))
    }
}

/// Parse un corpus prose JSONL. Toute ligne non vide doit être un
/// `ProseMigrationRecord` fermé ; le premier défaut refuse le fichier entier
/// avant toute écriture journal.
pub fn parse_prose_corpus(bytes: &[u8]) -> Result<Vec<ProseMigrationRecord>, CatalogueError> {
    let mut records = Vec::new();
    for (index, raw) in BufReader::new(bytes).lines().enumerate() {
        let line = raw?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let record: ProseMigrationRecord = serde_json::from_str(trimmed).map_err(|source| {
            CatalogueError::Format(format!("corpus prose ligne {}: {}", index + 1, source))
        })?;
        if record.provenance_id.trim().is_empty() || record.text.is_empty() {
            return Err(CatalogueError::Format(format!(
                "corpus prose ligne {}: provenance_id et text obligatoires",
                index + 1
            )));
        }
        records.push(record);
    }
    Ok(records)
}

/// Réduit le journal en vue déterministe (aucune écriture).
pub fn project_registre(entries: &[CatalogueEntry]) -> RegistreView {
    let mut delivered: BTreeSet<String> = BTreeSet::new();
    let mut adds: BTreeMap<String, &AddEntry> = BTreeMap::new();
    let mut pendings: BTreeMap<String, &PendingQualificationEntry> = BTreeMap::new();

    for entry in entries {
        match entry {
            CatalogueEntry::Add(add) => {
                adds.insert(add.id.clone(), add);
            }
            CatalogueEntry::Transition(transition) => {
                delivered.insert(transition.constat_id.clone());
            }
            CatalogueEntry::PendingQualification(pending) => {
                pendings.insert(pending.id.clone(), pending);
            }
        }
    }

    let mut ouverts: Vec<OpenConstatView> = adds
        .values()
        .filter(|add| !delivered.contains(&add.id))
        .map(|add| OpenConstatView {
            id: add.id.clone(),
            date: add.date.clone(),
            severity: add.severity,
            recurrence_of: add.recurrence_of.clone(),
            gate_failed: add.mission_source.is_failed_gate(),
            text: add.text.clone(),
            mission_source: add.mission_source.clone(),
        })
        .collect();

    ouverts.sort_by(compare_open_constats);

    let mut attente: Vec<PendingView> = pendings
        .values()
        .filter(|pending| !adds.contains_key(&pending.id))
        .map(|pending| PendingView {
            id: pending.id.clone(),
            provenance_id: pending.provenance_id.clone(),
            text: pending.text.clone(),
        })
        .collect();
    attente.sort_by(|left, right| {
        left.provenance_id
            .cmp(&right.provenance_id)
            .then_with(|| left.id.cmp(&right.id))
    });

    let recurrents = ouverts
        .iter()
        .filter(|item| item.recurrence_of.is_some())
        .count();
    let gates_rates = ouverts.iter().filter(|item| item.gate_failed).count();
    let footer = RegistreFooter {
        ouverts: ouverts.len(),
        recurrents,
        gates_rates,
        pending_qualification: attente.len(),
    };
    RegistreView {
        ouverts,
        attente,
        footer,
    }
}

fn compare_open_constats(left: &OpenConstatView, right: &OpenConstatView) -> Ordering {
    left.severity
        .sort_rank()
        .cmp(&right.severity.sort_rank())
        .then_with(|| {
            right
                .recurrence_of
                .is_some()
                .cmp(&left.recurrence_of.is_some())
        })
        .then_with(|| right.gate_failed.cmp(&left.gate_failed))
        .then_with(|| left.date.cmp(&right.date))
        .then_with(|| left.id.cmp(&right.id))
}

/// Rend la vue humaine d'autorité (SC-1704) : constats ouverts + pied.
pub fn render_registre_list(view: &RegistreView) -> String {
    render_registre_list_with_attente(view, false)
}

/// Même vue, avec la section des entrées encore en attente de qualification
/// (id, provenance, texte verbatim). Drapeau CLI `--attente`.
pub fn render_registre_list_with_attente(view: &RegistreView, show_attente: bool) -> String {
    let mut out = String::new();
    out.push_str("registre list\n");
    if view.ouverts.is_empty() {
        out.push_str("(aucun constat ouvert)\n");
    } else {
        for item in &view.ouverts {
            let recurrence = item
                .recurrence_of
                .as_deref()
                .map(|id| format!(" recurrence_of={id}"))
                .unwrap_or_default();
            let gate = if item.gate_failed { " gate=failed" } else { "" };
            out.push_str(&format!(
                "- [{severity:?}] {id} {date} source={source_kind}/{source_id}{recurrence}{gate}\n  {text}\n",
                severity = item.severity,
                id = item.id,
                date = item.date,
                source_kind = match item.mission_source.kind {
                    MissionSourceKind::Mission => "mission",
                    MissionSourceKind::Incident => "incident",
                    MissionSourceKind::Review => "review",
                    MissionSourceKind::Gate => "gate",
                },
                source_id = item.mission_source.id,
                text = item.text,
            ));
        }
    }
    if show_attente {
        out.push_str("--- en attente de qualification ---\n");
        if view.attente.is_empty() {
            out.push_str("(aucune entrée en attente)\n");
        } else {
            for item in &view.attente {
                out.push_str(&format!(
                    "- id={id} provenance={provenance}\n  {text}\n",
                    id = item.id,
                    provenance = item.provenance_id,
                    text = item.text,
                ));
            }
        }
    }
    out.push_str(&format!(
        "pied: {n} constats OUVERTS dont {m} récurrents, {k} liés à un gate raté, {p} en attente de qualification\n",
        n = view.footer.ouverts,
        m = view.footer.recurrents,
        k = view.footer.gates_rates,
        p = view.footer.pending_qualification,
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn sample_add(id: &str, severity: Severity, date: &str) -> AddEntry {
        AddEntry {
            v: 1,
            kind: AddKind::Add,
            id: id.into(),
            date: date.into(),
            mission_source: MissionSource {
                kind: MissionSourceKind::Incident,
                id: "inc-1".into(),
                failed: None,
            },
            severity,
            recurrence_of: None,
            text: format!("texte {id}"),
        }
    }

    #[test]
    fn refuse_kind_et_champ_inconnus() {
        assert!(parse_closed_line(r#"{"v":1,"kind":"score","id":"x"}"#).is_err());
        assert!(parse_closed_line(
            r#"{"v":1,"kind":"add","id":"c1","date":"2026-08-23T22:00:00+02:00","mission_source":{"kind":"incident","id":"i1"},"severity":"blocker","text":"ok","extra":1}"#
        )
        .is_err());
        assert!(parse_closed_line(
            r#"{"v":1,"kind":"add","id":"c1","date":"2026-08-23T22:00:00+02:00","mission_source":{"kind":"incident","id":"i1"},"severity":"critique","text":"x"}"#
        )
        .is_err());
    }

    #[test]
    fn refuse_horodatage_sans_fuseau() {
        assert!(validate_rfc3339_with_offset("2026-08-23T22:00:00").is_err());
        assert!(validate_rfc3339_with_offset("2026-08-23T22:00:00Z").is_ok());
        assert!(validate_rfc3339_with_offset("2026-08-23T22:00:00.123+02:00").is_ok());
    }

    #[test]
    fn path_refuse_tasks_md_et_symlink() {
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-path-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let tasks = root.join("tasks.md");
        fs::write(&tasks, "x").unwrap();
        assert!(validate_catalogue_path(&tasks, Some(&root)).is_err());

        let target = root.join("real.jsonl");
        fs::write(&target, "").unwrap();
        let link = root.join("link.jsonl");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(validate_catalogue_path(&link, Some(&root)).is_err());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn add_idempotent_et_conflit_divergent() {
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-idem-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        {
            let mut journal = CatalogueJournal::open(&path).unwrap();
            let entry = sample_add("c1", Severity::Major, "2026-08-23T22:00:00+02:00");
            assert_eq!(
                journal.append_add(entry.clone()).unwrap(),
                AppendOutcome::Appended
            );
            assert_eq!(
                journal.append_add(entry).unwrap(),
                AppendOutcome::IdempotentNoop
            );
            let mut divergent = sample_add("c1", Severity::Major, "2026-08-23T22:00:00+02:00");
            divergent.text = "autre texte".into();
            assert!(matches!(
                journal.append_add(divergent),
                Err(CatalogueError::IdempotenceConflict { .. })
            ));
            assert_eq!(journal.read_entries().unwrap().len(), 1);
        }
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn projection_trie_et_footer_deterministe() {
        let entries = vec![
            CatalogueEntry::Add(sample_add("b", Severity::Info, "2026-08-24T10:00:00Z")),
            CatalogueEntry::Add({
                let mut entry = sample_add("a", Severity::Blocker, "2026-08-23T10:00:00Z");
                entry.mission_source = MissionSource {
                    kind: MissionSourceKind::Gate,
                    id: "G1".into(),
                    failed: Some(true),
                };
                entry.recurrence_of = Some("b".into());
                entry
            }),
            CatalogueEntry::PendingQualification(PendingQualificationEntry {
                v: 1,
                kind: PendingKind::PendingQualification,
                id: "p1".into(),
                provenance_id: "prov-1".into(),
                text: "historique".into(),
            }),
        ];
        // recurrence_of=a requires b to exist first in journal when appending;
        // here we build projection directly: fix order so b exists conceptually.
        let view = project_registre(&entries);
        assert_eq!(view.footer.ouverts, 2);
        assert_eq!(view.footer.recurrents, 1);
        assert_eq!(view.footer.gates_rates, 1);
        assert_eq!(view.footer.pending_qualification, 1);
        assert_eq!(view.ouverts[0].id, "a");
        let rendered = render_registre_list(&view);
        let mut permuted = entries;
        permuted.reverse();
        let again = render_registre_list(&project_registre(&permuted));
        assert_eq!(rendered, again);
    }

    #[test]
    fn writers_concurrents_conservent_deux_lignes() {
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-conc-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        File::create(&path).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let path_a = path.clone();
        let path_b = path.clone();
        let barrier_a = barrier.clone();
        let barrier_b = barrier;
        let handle_a = thread::spawn(move || {
            barrier_a.wait();
            let mut journal = CatalogueJournal::open(&path_a).unwrap();
            journal
                .append_add(sample_add(
                    "w1",
                    Severity::Minor,
                    "2026-08-23T21:00:00+02:00",
                ))
                .unwrap()
        });
        let handle_b = thread::spawn(move || {
            barrier_b.wait();
            let mut journal = CatalogueJournal::open(&path_b).unwrap();
            journal
                .append_add(sample_add(
                    "w2",
                    Severity::Info,
                    "2026-08-23T21:01:00+02:00",
                ))
                .unwrap()
        });
        assert_eq!(handle_a.join().unwrap(), AppendOutcome::Appended);
        assert_eq!(handle_b.join().unwrap(), AppendOutcome::Appended);
        let mut journal = CatalogueJournal::open(&path).unwrap();
        let entries = journal.read_entries().unwrap();
        assert_eq!(entries.len(), 2);
        let ids: BTreeSet<_> = entries
            .iter()
            .filter_map(|entry| match entry {
                CatalogueEntry::Add(add) => Some(add.id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(ids, BTreeSet::from(["w1", "w2"]));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn migrer_corpus_puis_rejeu_ne_redouble_pas_les_provenances() {
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-migrer-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let journal_path = root.join("catalogue.jsonl");
        let prose_path = root.join("prose.jsonl");
        fs::write(
            &prose_path,
            concat!(
                r#"{"provenance_id":"doc#1","text":"premier constat historique"}"#,
                "\n",
                r#"{"provenance_id":"doc#2","text":"second constat historique"}"#,
                "\n",
            ),
        )
        .unwrap();
        {
            let mut journal = CatalogueJournal::open(&journal_path).unwrap();
            let first = journal.migrate_prose_file(&prose_path).unwrap();
            assert_eq!(first.read, 2);
            assert_eq!(first.appended, 2);
            assert_eq!(first.skipped, 0);
            let second = journal.migrate_prose_file(&prose_path).unwrap();
            assert_eq!(second.read, 2);
            assert_eq!(second.appended, 0);
            assert_eq!(second.skipped, 2);
            assert_eq!(journal.read_entries().unwrap().len(), 2);
        }
        // Malformé : aucune écriture supplémentaire.
        fs::write(&prose_path, "{\"text\":\"sans provenance\"}\n").unwrap();
        let mut journal = CatalogueJournal::open(&journal_path).unwrap();
        let before = journal.read_entries().unwrap().len();
        assert!(journal.migrate_prose_file(&prose_path).is_err());
        assert_eq!(journal.read_entries().unwrap().len(), before);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn qualifier_append_add_verbatim_et_retire_de_p() {
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-qualif-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        journal
            .append_pending(PendingQualificationEntry {
                v: 1,
                kind: PendingKind::PendingQualification,
                id: "pending:doc#1".into(),
                provenance_id: "doc#1".into(),
                text: "texte historique immuable".into(),
            })
            .unwrap();
        let before = project_registre(&journal.read_entries().unwrap());
        assert_eq!(before.footer.pending_qualification, 1);
        assert_eq!(before.attente[0].text, "texte historique immuable");

        let outcome = journal
            .qualify_pending(
                "pending:doc#1",
                Severity::Major,
                MissionSource {
                    kind: MissionSourceKind::Incident,
                    id: "i-1".into(),
                    failed: None,
                },
                "2026-08-24T06:15:00+02:00".into(),
            )
            .unwrap();
        assert_eq!(outcome, AppendOutcome::Appended);
        assert_eq!(
            journal
                .qualify_pending(
                    "pending:doc#1",
                    Severity::Major,
                    MissionSource {
                        kind: MissionSourceKind::Incident,
                        id: "i-1".into(),
                        failed: None,
                    },
                    "2026-08-24T06:15:00+02:00".into(),
                )
                .unwrap(),
            AppendOutcome::IdempotentNoop
        );

        let after = project_registre(&journal.read_entries().unwrap());
        assert_eq!(after.footer.pending_qualification, 0);
        assert_eq!(after.footer.ouverts, 1);
        assert_eq!(after.ouverts[0].text, "texte historique immuable");
        assert_eq!(after.ouverts[0].severity, Severity::Major);
        // La ligne pending reste au journal (append-only).
        assert_eq!(journal.read_entries().unwrap().len(), 2);
        let rendered = render_registre_list_with_attente(&after, true);
        assert!(rendered.contains("(aucune entrée en attente)"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn derniere_ligne_arrachee_laisse_les_precedentes_lisibles() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-torn-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        {
            let mut journal = CatalogueJournal::open(&path).unwrap();
            journal
                .append_add(sample_add("kept-1", Severity::Info, "2026-08-24T06:00:00Z"))
                .unwrap();
            journal
                .append_add(sample_add("kept-2", Severity::Minor, "2026-08-24T06:01:00Z"))
                .unwrap();
            journal
                .append_add(sample_add("torn", Severity::Major, "2026-08-24T06:02:00Z"))
                .unwrap();
        }
        let intact = fs::read(&path).unwrap();
        assert!(
            intact.ends_with(b"\n"),
            "l'append doit terminer chaque enregistrement par un seul write LF"
        );
        let text = String::from_utf8(intact).unwrap();
        let without_final_lf = text.trim_end_matches('\n');
        let last_start = without_final_lf.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let cut = last_start + (without_final_lf.len() - last_start) / 2;
        assert!(cut > last_start, "la troncature doit couper au milieu de la dernière ligne");
        fs::write(&path, &without_final_lf.as_bytes()[..cut]).unwrap();

        let parsed = parse_journal_bytes(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(parsed.entries.len(), 2, "les N-1 premières restent lisibles");
        let ids: BTreeSet<_> = parsed
            .entries
            .iter()
            .filter_map(|entry| match entry {
                CatalogueEntry::Add(add) => Some(add.id.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(ids, BTreeSet::from(["kept-1", "kept-2"]));
        let warning = parsed
            .torn_tail_warning
            .expect("la queue arrachée doit être signalée");
        assert!(warning.contains("ligne finale arrachée"));

        // Une ligne invalide au milieu reste une erreur franche.
        let mut middle_corrupt = String::new();
        middle_corrupt.push_str(
            r#"{"v":1,"kind":"add","id":"a","date":"2026-08-24T06:00:00Z","mission_source":{"kind":"incident","id":"i"},"severity":"info","text":"ok"}"#,
        );
        middle_corrupt.push('\n');
        middle_corrupt.push_str("{not-json\n");
        middle_corrupt.push_str(
            r#"{"v":1,"kind":"add","id":"b","date":"2026-08-24T06:01:00Z","mission_source":{"kind":"incident","id":"i"},"severity":"info","text":"ok"}"#,
        );
        middle_corrupt.push('\n');
        let err = parse_journal_bytes(middle_corrupt.as_bytes()).unwrap_err();
        assert!(err.to_string().contains("ligne 2"));
        let _ = fs::remove_dir_all(&root);
    }
}
