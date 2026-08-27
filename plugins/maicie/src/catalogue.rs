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

/// Nature d'une entrée du registre — cycles de vie distincts.
///
/// `constat` : défaut mesuré, fermable. `regle` : invariant de méthode,
/// jamais fermé. `resultat` : acquis déjà consigné, pas un dû.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryNature {
    #[default]
    Constat,
    Regle,
    Resultat,
}

impl EntryNature {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Constat => "constat",
            Self::Regle => "regle",
            Self::Resultat => "resultat",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "constat" => Some(Self::Constat),
            "regle" => Some(Self::Regle),
            "resultat" => Some(Self::Resultat),
            _ => None,
        }
    }

    pub const fn is_constat(self) -> bool {
        matches!(self, Self::Constat)
    }

    fn skip_if_constat(nature: &EntryNature) -> bool {
        nature.is_constat()
    }

    /// Seul un constat se ferme ou se réfute.
    pub const fn allows_closure(self) -> bool {
        self.is_constat()
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
    /// Cycle de vie. Absent au journal → `constat` (rétrocompat).
    #[serde(default, skip_serializing_if = "EntryNature::skip_if_constat")]
    pub nature: EntryNature,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurrence_of: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddKind {
    Add,
}

/// Transition attestée. Même état dérivé `delivered` pour traité/réfuté —
/// **pas la même lecture**. `requalified` reste `open` (charge réelle ajustée).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransitionEntry {
    pub v: u32,
    pub kind: TransitionKind,
    pub constat_id: String,
    pub from: ConstatState,
    pub to: ConstatState,
    /// Obligatoire pour `objective_closed` ; vide sinon.
    pub objective_id: String,
    pub observed_at: String,
    pub trigger: TransitionTrigger,
    /// Raison typée (snake_case) — jamais un champ libre obligatoire.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raison: Option<String>,
    /// Preuve `sha:` ou `mesure:` selon le geste.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Requalification : sévérité d'origine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_from: Option<Severity>,
    /// Requalification : nouvelle sévérité.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity_to: Option<Severity>,
    /// Reclassification : nature d'origine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nature_from: Option<EntryNature>,
    /// Reclassification : nouvelle nature.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nature_to: Option<EntryNature>,
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
    /// Ce fut vrai, ce ne l'est plus.
    RemediedAttested,
    /// Ce ne fut jamais vrai.
    Refuted,
    /// Vrai, sévérité/portée changée — reste ouvert.
    Requalified,
    /// Transition erronée annulée par amendement — revient à ouvert.
    /// L'historique (fermeture + rectification) reste lisible.
    Rectified,
}

/// Raisons fermées de fermeture (traité).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaisonFermeture {
    CorrigeEnProduction,
    CorrigeParLot,
    AbsorbeParAutreConstat,
    ObjectifClos,
}

impl RaisonFermeture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CorrigeEnProduction => "corrige_en_production",
            Self::CorrigeParLot => "corrige_par_lot",
            Self::AbsorbeParAutreConstat => "absorbe_par_autre_constat",
            Self::ObjectifClos => "objectif_clos",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "corrige_en_production" => Some(Self::CorrigeEnProduction),
            "corrige_par_lot" => Some(Self::CorrigeParLot),
            "absorbe_par_autre_constat" => Some(Self::AbsorbeParAutreConstat),
            "objectif_clos" => Some(Self::ObjectifClos),
            _ => None,
        }
    }
}

/// Raisons fermées de réfutation (plus grave — efface une charge).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaisonRefutation {
    ChargeFausseMesuree,
    HorsPerimetre,
    DejaCouvert,
    ErreurDeLecture,
}

impl RaisonRefutation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ChargeFausseMesuree => "charge_fausse_mesuree",
            Self::HorsPerimetre => "hors_perimetre",
            Self::DejaCouvert => "deja_couvert",
            Self::ErreurDeLecture => "erreur_de_lecture",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "charge_fausse_mesuree" => Some(Self::ChargeFausseMesuree),
            "hors_perimetre" => Some(Self::HorsPerimetre),
            "deja_couvert" => Some(Self::DejaCouvert),
            "erreur_de_lecture" => Some(Self::ErreurDeLecture),
            _ => None,
        }
    }
}

/// Raisons fermées de requalification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaisonRequalification {
    SeveriteAjustee,
    PerimetreAffine,
    NatureReclassee,
}

impl RaisonRequalification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SeveriteAjustee => "severite_ajustee",
            Self::PerimetreAffine => "perimetre_affine",
            Self::NatureReclassee => "nature_reclassee",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "severite_ajustee" => Some(Self::SeveriteAjustee),
            "perimetre_affine" => Some(Self::PerimetreAffine),
            "nature_reclassee" => Some(Self::NatureReclassee),
            _ => None,
        }
    }
}

/// Raisons fermées de rectification (amende une transition erronée).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RaisonRectification {
    /// Fermeture (`remedied_attested`) posée par erreur.
    FermetureErronee,
    /// Solde de mission (`objective_closed`) posé à tort — n'atteste pas
    /// une correction de fond, donc rectifiable (contrairement à l'ancienne
    /// garde « fermeture juste intacte »).
    SoldeMissionErrone,
    /// Réfutation posée par erreur.
    RefutationErronee,
}

impl RaisonRectification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FermetureErronee => "fermeture_erronee",
            Self::SoldeMissionErrone => "solde_mission_errone",
            Self::RefutationErronee => "refutation_erronee",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "fermeture_erronee" => Some(Self::FermetureErronee),
            "solde_mission_errone" => Some(Self::SoldeMissionErrone),
            "refutation_erronee" => Some(Self::RefutationErronee),
            _ => None,
        }
    }

    /// Trigger de livraison que cette raison est autorisée à amender.
    pub const fn amends(self) -> TransitionTrigger {
        match self {
            Self::FermetureErronee => TransitionTrigger::RemediedAttested,
            Self::SoldeMissionErrone => TransitionTrigger::ObjectiveClosed,
            Self::RefutationErronee => TransitionTrigger::Refuted,
        }
    }
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
            Self::Transition(entry) => match entry.trigger {
                TransitionTrigger::ObjectiveClosed => format!(
                    "transition:{}:{}:{:?}",
                    entry.constat_id, entry.objective_id, entry.trigger
                ),
                TransitionTrigger::RemediedAttested
                | TransitionTrigger::Refuted
                | TransitionTrigger::Requalified
                | TransitionTrigger::Rectified => format!(
                    "transition:{}:{:?}:{}:{}",
                    entry.constat_id,
                    entry.trigger,
                    entry.raison.as_deref().unwrap_or(""),
                    entry.reference.as_deref().unwrap_or("")
                ),
            },
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

/// Lien d'arbitrage déclaré `(constat_id, objective_id)`.
///
/// Ce fait naît d'une délégation durable (T1708 / store) ; le journal ne le
/// invente jamais. La réconciliation ne consomme que des couples fournis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArbitrationLink {
    pub constat_id: String,
    pub objective_id: String,
}

/// Clôture d'objectif attestée par l'état durable (événement ou réconciliation).
///
/// `observed_at` est l'horodatage du fait attesté — jamais l'horloge locale
/// du lecteur, ni un silence, ni une échéance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttestedClosure {
    pub objective_id: String,
    pub observed_at: String,
}

/// Compte-rendu d'une réconciliation de clôtures attestées.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReconcileReport {
    pub appended: usize,
    pub skipped: usize,
}

/// Type de fait couvert par le contrat FR-1711 (spec 017).
///
/// Fermé : ajouter une variante = changer le contrat (revue hostile).
/// La sévérité associée est une **transcription**, pas un jugement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoveredFactKind {
    /// Gate raté → `blocker` (définition d'un gate).
    GateFailed,
    /// Verdict de revue `AMENDER` → `major` (décision déjà prise par le relecteur).
    ReviewAmender,
}

impl CoveredFactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GateFailed => "gate_failed",
            Self::ReviewAmender => "review_amender",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "gate_failed" => Some(Self::GateFailed),
            "review_amender" => Some(Self::ReviewAmender),
            _ => None,
        }
    }

    /// Correspondance totale et sans exception (FR-1711).
    pub const fn derived_severity(self) -> Severity {
        match self {
            Self::GateFailed => Severity::Blocker,
            Self::ReviewAmender => Severity::Major,
        }
    }

    pub const fn mission_source_kind(self) -> MissionSourceKind {
        match self {
            Self::GateFailed => MissionSourceKind::Gate,
            Self::ReviewAmender => MissionSourceKind::Review,
        }
    }
}

/// Fait observé à transcrire vers le journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedFact {
    /// Type machine (`gate_failed`, `review_amender`, ou hors table).
    pub kind: String,
    pub source_id: String,
    pub date: String,
    pub text: String,
}

/// Issue d'une transcription FR-1711.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptionOutcome {
    /// Fait couvert → `add` avec sévérité dérivée.
    CoveredAdd(AddEntry),
    /// Hors table → attente humaine, zéro sévérité inventée.
    Pending(PendingQualificationEntry),
}

/// Transcrit un fait observé selon le contrat FR-1711.
///
/// Couvert → `add` ; hors table → `pending_qualification`. Aucune case
/// ambiguë : si le type n'est pas dans la table, le défaut est l'attente.
pub fn transcribe_observed_fact(
    fact: &ObservedFact,
) -> Result<TranscriptionOutcome, CatalogueError> {
    if fact.source_id.trim().is_empty() {
        return Err(CatalogueError::Format(
            "source_id obligatoire pour une transcription".into(),
        ));
    }
    if fact.text.is_empty() {
        return Err(CatalogueError::Format(
            "texte verbatim obligatoire pour une transcription".into(),
        ));
    }
    validate_rfc3339_with_offset(&fact.date)?;

    if let Some(covered) = CoveredFactKind::parse(fact.kind.trim()) {
        let id = format!("{}:{}", covered.as_str(), fact.source_id);
        let mission_source = MissionSource {
            kind: covered.mission_source_kind(),
            id: fact.source_id.clone(),
            failed: match covered {
                CoveredFactKind::GateFailed => Some(true),
                CoveredFactKind::ReviewAmender => None,
            },
        };
        mission_source.validate()?;
        let add = AddEntry {
            v: CATALOGUE_VERSION,
            kind: AddKind::Add,
            id,
            date: fact.date.clone(),
            mission_source,
            severity: covered.derived_severity(),
            nature: EntryNature::Constat,
            recurrence_of: None,
            text: fact.text.clone(),
        };
        validate_add_shape(&add)?;
        return Ok(TranscriptionOutcome::CoveredAdd(add));
    }

    // Défaut : attente, jamais invention de sévérité.
    let pending = PendingQualificationEntry {
        v: CATALOGUE_VERSION,
        kind: PendingKind::PendingQualification,
        id: format!("pending:{}:{}", fact.kind.trim(), fact.source_id),
        provenance_id: format!("uncovered:{}:{}", fact.kind.trim(), fact.source_id),
        text: fact.text.clone(),
    };
    validate_pending_shape(&pending)?;
    Ok(TranscriptionOutcome::Pending(pending))
}

/// État dérivé d'un constat dans la projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivedState {
    Open,
    Delivered,
    PendingQualification,
}

/// Ligne ouverte de la vue `registre list`.
/// Constat ouvert (éventuellement requalifié / rectifié).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenConstatView {
    pub id: String,
    pub date: String,
    pub severity: Severity,
    pub nature: EntryNature,
    pub recurrence_of: Option<String>,
    pub gate_failed: bool,
    pub text: String,
    pub mission_source: MissionSource,
    /// True si une requalification a ajusté la sévérité ou la nature.
    pub requalifie: bool,
    /// True si une rectification a amendé une transition erronée —
    /// l'entrée ne se lit pas comme jamais touchée.
    pub rectifie: bool,
}

/// Entrée encore en attente de qualification humaine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingView {
    pub id: String,
    pub provenance_id: String,
    pub text: String,
}

/// Polarité d'un constat retiré des ouverts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosedKind {
    /// Fermeture de fond (`remedied_attested`) — preuve typée.
    Traite,
    /// Solde de mission (`objective_closed`) — pas une preuve de correction.
    Solde,
    Refute,
}

/// Constat traité ou réfuté — lisible distinctement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedConstatView {
    pub id: String,
    pub kind: ClosedKind,
    pub observed_at: String,
    pub text: String,
    pub raison: Option<String>,
    pub reference: Option<String>,
    pub recurrence_of: Option<String>,
    /// Trigger de la livraison active — `objective_closed` n'est pas une
    /// fermeture prouvée (mission soldée ≠ défaut corrigé).
    pub trigger: TransitionTrigger,
    /// Rempli pour `objective_closed` ; vide sinon.
    pub objective_id: String,
}

/// Historique d'une fermeture erronée puis rectifiée — consultable sans
/// confondre avec un dû jamais touché ni une fermeture juste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RectifiedConstatView {
    pub id: String,
    pub text: String,
    /// Polarité de la transition fautive (traité / réfuté).
    pub closed_kind: ClosedKind,
    pub closed_at: String,
    pub closed_raison: Option<String>,
    pub closed_reference: Option<String>,
    pub rectified_at: String,
    pub rectifie_raison: Option<String>,
    pub rectifie_reference: Option<String>,
    pub recurrence_of: Option<String>,
}

/// Pied : dû / règles / résultats séparés + T/S/R/Q + rectifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistreFooter {
    /// Ouverts de nature `constat` — le dû fermable.
    pub ouverts: usize,
    pub ouverts_regle: usize,
    pub ouverts_resultat: usize,
    pub recurrents: usize,
    pub gates_rates: usize,
    pub pending_qualification: usize,
    /// Fermetures de fond (`remedied_attested`) — preuve typée.
    pub traites: usize,
    /// Soldes de mission (`objective_closed`) — pas un dû réglé.
    pub soldes: usize,
    pub refutes: usize,
    pub requalifies: usize,
    /// Nombre d'événements de rectification (historique).
    pub rectifies: usize,
}

/// Vue pure du registre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistreView {
    pub ouverts: Vec<OpenConstatView>,
    pub attente: Vec<PendingView>,
    /// Fermetures de fond prouvées.
    pub traites: Vec<ClosedConstatView>,
    /// Soldes de mission (auto, sans preuve de correction).
    pub soldes: Vec<ClosedConstatView>,
    pub refutes: Vec<ClosedConstatView>,
    /// Historique des rectifications (fermeture fautive + amendement).
    pub rectifies: Vec<RectifiedConstatView>,
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
            nature: EntryNature::Constat,
            recurrence_of: None,
            text: pending.text.clone(),
        };
        validate_add_shape(&add)?;
        self.append_entry_with_existing(CatalogueEntry::Add(add), &existing)
    }

    /// Transcrit un fait observé (FR-1711) et l'append au journal.
    ///
    /// Fait couvert → `add` à sévérité dérivée. Hors table →
    /// `pending_qualification` (défaut = attente, jamais invention).
    pub fn consign_observed_fact(
        &mut self,
        fact: &ObservedFact,
    ) -> Result<(TranscriptionOutcome, AppendOutcome), CatalogueError> {
        match transcribe_observed_fact(fact)? {
            TranscriptionOutcome::CoveredAdd(add) => {
                let outcome = self.append_add(add.clone())?;
                Ok((TranscriptionOutcome::CoveredAdd(add), outcome))
            }
            TranscriptionOutcome::Pending(pending) => {
                let outcome = self.append_pending_dedup_provenance(pending.clone())?;
                Ok((TranscriptionOutcome::Pending(pending), outcome))
            }
        }
    }

    /// Append une transition `open → delivered` uniquement si le couple
    /// `(constat_id, objective_id)` est fourni comme lien d'arbitrage ET que
    /// la clôture attestée porte le même `objective_id`.
    ///
    /// Aucune sévérité, aucun classement, aucune déduction textuelle : seuls
    /// des faits déclarés. Identité idempotente
    /// `(constat_id, objective_id, objective_closed)`.
    pub fn append_delivered_for_attested_closure(
        &mut self,
        link: &ArbitrationLink,
        closure: &AttestedClosure,
    ) -> Result<AppendOutcome, CatalogueError> {
        if link.constat_id.trim().is_empty() || link.objective_id.trim().is_empty() {
            return Err(CatalogueError::Format(
                "lien d'arbitrage : constat_id et objective_id obligatoires".into(),
            ));
        }
        if link.objective_id != closure.objective_id {
            return Err(CatalogueError::TransitionInvalide(format!(
                "clôture objective_id={} hors lien d'arbitrage (attendu {})",
                closure.objective_id, link.objective_id
            )));
        }
        let entry = TransitionEntry {
            v: CATALOGUE_VERSION,
            kind: TransitionKind::Transition,
            constat_id: link.constat_id.clone(),
            from: ConstatState::Open,
            to: ConstatState::Delivered,
            objective_id: link.objective_id.clone(),
            observed_at: closure.observed_at.clone(),
            trigger: TransitionTrigger::ObjectiveClosed,
            raison: None,
            reference: None,
            severity_from: None,
            severity_to: None,
            nature_from: None,
            nature_to: None,
        };
        let existing = self.read_entries()?;
        let nature = resolve_open_nature(&link.constat_id, &existing).unwrap_or(EntryNature::Constat);
        if !nature.allows_closure() {
            return Err(CatalogueError::TransitionInvalide(format!(
                "clôture refusée : nature={} n'est pas un constat fermable",
                nature.as_str()
            )));
        }
        self.append_transition(entry)
    }

    /// Traite un constat : ce fut vrai, ce ne l'est plus.
    pub fn close_constat_attested(
        &mut self,
        constat_id: &str,
        raison: RaisonFermeture,
        reference: &str,
        observed_at: &str,
    ) -> Result<AppendOutcome, CatalogueError> {
        crate::preuve::parse_reference_fermeture(reference).map_err(|error| {
            CatalogueError::TransitionInvalide(format!("fermeture refusée : {error}"))
        })?;
        self.settle_delivered(
            constat_id,
            TransitionTrigger::RemediedAttested,
            raison.as_str(),
            reference,
            observed_at,
        )
    }

    /// Réfute un constat : ce ne fut jamais vrai. Preuve `mesure:` obligatoire.
    pub fn refute_constat_attested(
        &mut self,
        constat_id: &str,
        raison: RaisonRefutation,
        reference: &str,
        observed_at: &str,
    ) -> Result<AppendOutcome, CatalogueError> {
        crate::preuve::parse_reference_refutation(reference).map_err(|error| {
            CatalogueError::TransitionInvalide(format!("réfutation refusée : {error}"))
        })?;
        self.settle_delivered(
            constat_id,
            TransitionTrigger::Refuted,
            raison.as_str(),
            reference,
            observed_at,
        )
    }

    /// Requalifie : sévérité et/ou nature — reste ouvert.
    ///
    /// Au moins un des deux changements est obligatoire. Une règle ne peut
    /// jamais porter `blocker` (refus à l'écriture).
    pub fn requalify_constat(
        &mut self,
        constat_id: &str,
        severity: Option<(Severity, Severity)>,
        nature: Option<(EntryNature, EntryNature)>,
        raison: RaisonRequalification,
        reference: Option<&str>,
        observed_at: &str,
    ) -> Result<AppendOutcome, CatalogueError> {
        let severity_change = severity.is_some_and(|(from, to)| from != to);
        let nature_change = nature.is_some_and(|(from, to)| from != to);
        if !severity_change && !nature_change {
            return Err(CatalogueError::TransitionInvalide(
                "requalification refusée : sévérité et nature inchangées".into(),
            ));
        }
        if let Some((from, to)) = severity {
            if from == to && !nature_change {
                return Err(CatalogueError::TransitionInvalide(
                    "requalification refusée : sévérité inchangée".into(),
                ));
            }
            let _ = (from, to);
        }
        let reference = match reference {
            Some(raw) if !raw.trim().is_empty() => {
                crate::preuve::parse_reference_fermeture(raw).map_err(|error| {
                    CatalogueError::TransitionInvalide(format!("requalification refusée : {error}"))
                })?;
                Some(raw.trim().to_string())
            }
            _ => None,
        };
        if constat_id.trim().is_empty() {
            return Err(CatalogueError::Format("constat_id vide".into()));
        }
        let existing = self.read_entries()?;
        let already_delivered = is_currently_delivered(constat_id, &existing);
        if already_delivered {
            return Err(CatalogueError::TransitionInvalide(format!(
                "constat {constat_id} déjà delivered : requalification refusée"
            )));
        }
        let effective_severity = severity
            .map(|(_, to)| to)
            .or_else(|| resolve_open_severity(constat_id, &existing))
            .ok_or_else(|| CatalogueError::ReferenceInconnue {
                field: "constat_id",
                id: constat_id.to_string(),
            })?;
        let target_nature = nature
            .map(|(_, to)| to)
            .or_else(|| resolve_open_nature(constat_id, &existing))
            .unwrap_or(EntryNature::Constat);
        if let Some((from, _)) = severity {
            let current = resolve_open_severity(constat_id, &existing).ok_or_else(|| {
                CatalogueError::ReferenceInconnue {
                    field: "constat_id",
                    id: constat_id.to_string(),
                }
            })?;
            if current != from {
                return Err(CatalogueError::TransitionInvalide(format!(
                    "requalification refusée : severity_from périmée (courante={current:?}, fournie={from:?})"
                )));
            }
        }
        if let Some((from, _)) = nature {
            let current = resolve_open_nature(constat_id, &existing).ok_or_else(|| {
                CatalogueError::ReferenceInconnue {
                    field: "constat_id",
                    id: constat_id.to_string(),
                }
            })?;
            if current != from {
                return Err(CatalogueError::TransitionInvalide(format!(
                    "requalification refusée : nature_from périmée (courante={current:?}, fournie={from:?})"
                )));
            }
        }
        if target_nature == EntryNature::Regle && effective_severity == Severity::Blocker {
            return Err(CatalogueError::TransitionInvalide(
                "requalification refusée : une règle ne peut pas porter blocker".into(),
            ));
        }
        let (severity_from, severity_to) = match severity {
            Some((from, to)) => (Some(from), Some(to)),
            None => (None, None),
        };
        let (nature_from, nature_to) = match nature {
            Some((from, to)) => (Some(from), Some(to)),
            None => (None, None),
        };
        let entry = TransitionEntry {
            v: CATALOGUE_VERSION,
            kind: TransitionKind::Transition,
            constat_id: constat_id.to_string(),
            from: ConstatState::Open,
            to: ConstatState::Open,
            objective_id: String::new(),
            observed_at: observed_at.to_string(),
            trigger: TransitionTrigger::Requalified,
            raison: Some(raison.as_str().to_string()),
            reference,
            severity_from,
            severity_to,
            nature_from,
            nature_to,
        };
        self.append_entry_with_existing(CatalogueEntry::Transition(entry), &existing)
    }

    /// Amende une transition erronée : `delivered → open`, append-only.
    ///
    /// Un `objective_closed` (solde de mission) EST rectifiable : il n'atteste
    /// pas une correction de fond. Une fermeture `remedied_attested` ou une
    /// réfutation aussi, via leur raison dédiée. L'historique (livraison +
    /// rectification) reste dans le journal — l'entrée ne se lit plus comme
    /// intacte.
    pub fn rectify_constat_attested(
        &mut self,
        constat_id: &str,
        raison: RaisonRectification,
        reference: &str,
        observed_at: &str,
    ) -> Result<AppendOutcome, CatalogueError> {
        crate::preuve::parse_reference_fermeture(reference).map_err(|error| {
            CatalogueError::TransitionInvalide(format!("rectification refusée : {error}"))
        })?;
        if constat_id.trim().is_empty() {
            return Err(CatalogueError::Format("constat_id vide".into()));
        }
        let entry = TransitionEntry {
            v: CATALOGUE_VERSION,
            kind: TransitionKind::Transition,
            constat_id: constat_id.to_string(),
            from: ConstatState::Delivered,
            to: ConstatState::Open,
            objective_id: String::new(),
            observed_at: observed_at.to_string(),
            trigger: TransitionTrigger::Rectified,
            raison: Some(raison.as_str().to_string()),
            reference: Some(reference.trim().to_string()),
            severity_from: None,
            severity_to: None,
            nature_from: None,
            nature_to: None,
        };
        validate_transition_shape(&entry)?;
        let existing = self.read_entries()?;
        let has_add = existing.iter().any(|line| match line {
            CatalogueEntry::Add(add) => add.id == constat_id,
            _ => false,
        });
        if !has_add {
            return Err(CatalogueError::ReferenceInconnue {
                field: "constat_id",
                id: constat_id.to_string(),
            });
        }
        let Some(current) = current_delivery_trigger(constat_id, &existing) else {
            return Err(CatalogueError::TransitionInvalide(format!(
                "rectification refusée : {constat_id} n'est pas delivered"
            )));
        };
        if current != raison.amends() {
            return Err(CatalogueError::TransitionInvalide(format!(
                "rectification refusée : raison {} n'amende pas un {:?}",
                raison.as_str(),
                current
            )));
        }
        let wrapped = CatalogueEntry::Transition(entry);
        let key = wrapped.identity_key();
        let line = canonical_line(&wrapped)?;
        for previous in &existing {
            if previous.identity_key() == key && canonical_line(previous)? == line {
                return Ok(AppendOutcome::IdempotentNoop);
            }
        }
        self.append_entry_with_existing(wrapped, &existing)
    }

    fn settle_delivered(
        &mut self,
        constat_id: &str,
        trigger: TransitionTrigger,
        raison: &str,
        reference: &str,
        observed_at: &str,
    ) -> Result<AppendOutcome, CatalogueError> {
        let verb = match trigger {
            TransitionTrigger::RemediedAttested => "fermeture",
            TransitionTrigger::Refuted => "réfutation",
            _ => {
                return Err(CatalogueError::TransitionInvalide(
                    "settle_delivered : trigger inadmissible".into(),
                ));
            }
        };
        if constat_id.trim().is_empty() {
            return Err(CatalogueError::Format("constat_id vide".into()));
        }
        let entry = TransitionEntry {
            v: CATALOGUE_VERSION,
            kind: TransitionKind::Transition,
            constat_id: constat_id.to_string(),
            from: ConstatState::Open,
            to: ConstatState::Delivered,
            objective_id: String::new(),
            observed_at: observed_at.to_string(),
            trigger,
            raison: Some(raison.to_string()),
            reference: Some(reference.trim().to_string()),
            severity_from: None,
            severity_to: None,
            nature_from: None,
            nature_to: None,
        };
        validate_transition_shape(&entry)?;
        let existing = self.read_entries()?;
        let has_add = existing.iter().any(|line| match line {
            CatalogueEntry::Add(add) => add.id == constat_id,
            _ => false,
        });
        if !has_add {
            return Err(CatalogueError::ReferenceInconnue {
                field: "constat_id",
                id: constat_id.to_string(),
            });
        }
        let nature = resolve_open_nature(constat_id, &existing).unwrap_or(EntryNature::Constat);
        if !nature.allows_closure() {
            return Err(CatalogueError::TransitionInvalide(format!(
                "{verb} refusée : nature={} n'est pas un constat fermable",
                nature.as_str()
            )));
        }
        let already_delivered = is_currently_delivered(constat_id, &existing);
        let wrapped = CatalogueEntry::Transition(entry);
        let key = wrapped.identity_key();
        let line = canonical_line(&wrapped)?;
        if already_delivered {
            for previous in &existing {
                if previous.identity_key() == key && canonical_line(previous)? == line {
                    return Ok(AppendOutcome::IdempotentNoop);
                }
            }
            return Err(CatalogueError::TransitionInvalide(format!(
                "constat {constat_id} déjà delivered : {verb} refusée"
            )));
        }
        self.append_entry_with_existing(wrapped, &existing)
    }

    /// Réconcilie les clôtures attestées contre les liens d'arbitrage fournis.
    ///
    /// Une clôture sans lien correspondant est ignorée (pas de transition
    /// inventée). Un lien sans clôture reste ouvert. Le rejeu des mêmes faits
    /// est un no-op. La lecture des liens et des états durables passe par
    /// `app::reconcile_catalogue_from_store` (store → journal).
    pub fn reconcile_attested_closures(
        &mut self,
        links: &[ArbitrationLink],
        closures: &[AttestedClosure],
    ) -> Result<ReconcileReport, CatalogueError> {
        let mut closures_by_objective: BTreeMap<&str, &AttestedClosure> = BTreeMap::new();
        for closure in closures {
            if closure.objective_id.trim().is_empty() {
                return Err(CatalogueError::Format(
                    "clôture attestée : objective_id obligatoire".into(),
                ));
            }
            // Première attestation gagne : un second horodatage divergent pour
            // le même objectif serait un conflit d'autorité amont, pas du journal.
            closures_by_objective
                .entry(closure.objective_id.as_str())
                .or_insert(closure);
        }
        let mut appended = 0usize;
        let mut skipped = 0usize;
        for link in links {
            let Some(closure) = closures_by_objective.get(link.objective_id.as_str()) else {
                continue;
            };
            match self.append_delivered_for_attested_closure(link, closure)? {
                AppendOutcome::Appended => appended += 1,
                AppendOutcome::IdempotentNoop => skipped += 1,
            }
        }
        Ok(ReconcileReport { appended, skipped })
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
        if !path.exists() && path.file_name().is_none() {
            return Err(CatalogueError::PathRefuse {
                reason: "chemin sans nom de fichier".into(),
            });
        }
        let candidate =
            canonicalize_with_missing_tail(path).map_err(|source| CatalogueError::PathRefuse {
                reason: format!("chemin illisible: {source}"),
            })?;
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

/// Canonicalise le plus proche ancêtre existant puis normalise le reliquat.
///
/// Cette construction évite de comparer une racine résolue à un candidat brut
/// quand une écriture doit encore créer son arborescence.
fn canonicalize_with_missing_tail(path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        return path.canonicalize();
    }
    let existing = path
        .ancestors()
        .find(|ancestor| !ancestor.as_os_str().is_empty() && ancestor.exists());
    let (existing, missing_tail) = match existing {
        Some(existing) => {
            let tail = path.strip_prefix(existing).map_err(io::Error::other)?;
            (existing, tail)
        }
        None => (Path::new("."), path),
    };
    let canonical = existing.canonicalize()?;
    Ok(normalize_lexically(&canonical.join(missing_tail)))
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            std::path::Component::RootDir => normalized.push(component.as_os_str()),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::Normal(segment) => normalized.push(segment),
        }
    }
    normalized
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
    let text = std::str::from_utf8(bytes)
        .map_err(|source| CatalogueError::Format(format!("journal non UTF-8: {source}")))?;
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
    if entry.nature == EntryNature::Regle && entry.severity == Severity::Blocker {
        return Err(CatalogueError::Format(
            "une règle ne peut pas porter severity=blocker".into(),
        ));
    }
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
    if entry.constat_id.trim().is_empty() {
        return Err(CatalogueError::Format("constat_id obligatoire".into()));
    }
    validate_rfc3339_with_offset(&entry.observed_at)?;
    match entry.trigger {
        TransitionTrigger::ObjectiveClosed => {
            if entry.objective_id.trim().is_empty() {
                return Err(CatalogueError::Format(
                    "objective_id obligatoire pour trigger objective_closed".into(),
                ));
            }
            if entry.from != ConstatState::Open || entry.to != ConstatState::Delivered {
                return Err(CatalogueError::TransitionInvalide(
                    "objective_closed : seule open→delivered est admise".into(),
                ));
            }
            if entry.raison.is_some()
                || entry.reference.is_some()
                || entry.severity_from.is_some()
                || entry.severity_to.is_some()
                || entry.nature_from.is_some()
                || entry.nature_to.is_some()
            {
                return Err(CatalogueError::TransitionInvalide(
                    "raison/référence/sévérité/nature interdites pour objective_closed".into(),
                ));
            }
        }
        TransitionTrigger::RemediedAttested | TransitionTrigger::Refuted => {
            if !entry.objective_id.is_empty() {
                return Err(CatalogueError::TransitionInvalide(
                    "objective_id doit être vide pour fermeture/réfutation".into(),
                ));
            }
            if entry.from != ConstatState::Open || entry.to != ConstatState::Delivered {
                return Err(CatalogueError::TransitionInvalide(
                    "fermeture/réfutation : open→delivered obligatoire".into(),
                ));
            }
            let Some(raison) = entry.raison.as_deref() else {
                return Err(CatalogueError::TransitionInvalide(
                    "raison typée obligatoire".into(),
                ));
            };
            let Some(reference) = entry.reference.as_deref() else {
                return Err(CatalogueError::TransitionInvalide(
                    "référence obligatoire".into(),
                ));
            };
            match entry.trigger {
                TransitionTrigger::RemediedAttested => {
                    if RaisonFermeture::parse(raison).is_none() {
                        return Err(CatalogueError::TransitionInvalide(format!(
                            "raison de fermeture inconnue '{raison}'"
                        )));
                    }
                    crate::preuve::parse_reference_fermeture(reference).map_err(|error| {
                        CatalogueError::TransitionInvalide(format!("fermeture refusée : {error}"))
                    })?;
                }
                TransitionTrigger::Refuted => {
                    if RaisonRefutation::parse(raison).is_none() {
                        return Err(CatalogueError::TransitionInvalide(format!(
                            "raison de réfutation inconnue '{raison}'"
                        )));
                    }
                    crate::preuve::parse_reference_refutation(reference).map_err(|error| {
                        CatalogueError::TransitionInvalide(format!("réfutation refusée : {error}"))
                    })?;
                }
                _ => unreachable!(),
            }
            if entry.severity_from.is_some()
                || entry.severity_to.is_some()
                || entry.nature_from.is_some()
                || entry.nature_to.is_some()
            {
                return Err(CatalogueError::TransitionInvalide(
                    "sévérité/nature interdites pour fermeture/réfutation".into(),
                ));
            }
        }
        TransitionTrigger::Requalified => {
            if !entry.objective_id.is_empty() {
                return Err(CatalogueError::TransitionInvalide(
                    "objective_id doit être vide pour requalification".into(),
                ));
            }
            if entry.from != ConstatState::Open || entry.to != ConstatState::Open {
                return Err(CatalogueError::TransitionInvalide(
                    "requalification : open→open obligatoire".into(),
                ));
            }
            let Some(raison) = entry.raison.as_deref() else {
                return Err(CatalogueError::TransitionInvalide(
                    "raison de requalification obligatoire".into(),
                ));
            };
            if RaisonRequalification::parse(raison).is_none() {
                return Err(CatalogueError::TransitionInvalide(format!(
                    "raison de requalification inconnue '{raison}'"
                )));
            }
            match (entry.severity_from, entry.severity_to, entry.nature_from, entry.nature_to)
            {
                (Some(from), Some(to), nature_from, nature_to) if from != to => {
                    if let (Some(nf), Some(nt)) = (nature_from, nature_to) {
                        if nf == nt {
                            return Err(CatalogueError::TransitionInvalide(
                                "requalification : nature_from ≠ nature_to si les deux sont présents"
                                    .into(),
                            ));
                        }
                    } else if nature_from.is_some() || nature_to.is_some() {
                        return Err(CatalogueError::TransitionInvalide(
                            "requalification : nature_from et nature_to ensemble ou absents".into(),
                        ));
                    }
                }
                (None, None, Some(from), Some(to)) if from != to => {}
                (Some(from), Some(to), Some(nf), Some(nt)) if from == to && nf != nt => {}
                _ => {
                    return Err(CatalogueError::TransitionInvalide(
                        "requalification : changer severity_from≠severity_to et/ou nature_from≠nature_to"
                            .into(),
                    ));
                }
            }
            if entry.nature_to == Some(EntryNature::Regle) {
                let blocker = entry.severity_to == Some(Severity::Blocker)
                    || (entry.severity_to.is_none()
                        && entry.severity_from == Some(Severity::Blocker));
                if blocker {
                    return Err(CatalogueError::TransitionInvalide(
                        "requalification refusée : une règle ne peut pas porter blocker".into(),
                    ));
                }
            }
            if let Some(reference) = entry.reference.as_deref() {
                crate::preuve::parse_reference_fermeture(reference).map_err(|error| {
                    CatalogueError::TransitionInvalide(format!("requalification refusée : {error}"))
                })?;
            }
        }
        TransitionTrigger::Rectified => {
            if !entry.objective_id.is_empty() {
                return Err(CatalogueError::TransitionInvalide(
                    "objective_id doit être vide pour rectification".into(),
                ));
            }
            if entry.from != ConstatState::Delivered || entry.to != ConstatState::Open {
                return Err(CatalogueError::TransitionInvalide(
                    "rectification : delivered→open obligatoire".into(),
                ));
            }
            let Some(raison) = entry.raison.as_deref() else {
                return Err(CatalogueError::TransitionInvalide(
                    "raison de rectification obligatoire".into(),
                ));
            };
            if RaisonRectification::parse(raison).is_none() {
                return Err(CatalogueError::TransitionInvalide(format!(
                    "raison de rectification inconnue '{raison}'"
                )));
            }
            let Some(reference) = entry.reference.as_deref() else {
                return Err(CatalogueError::TransitionInvalide(
                    "référence obligatoire pour rectification".into(),
                ));
            };
            crate::preuve::parse_reference_fermeture(reference).map_err(|error| {
                CatalogueError::TransitionInvalide(format!("rectification refusée : {error}"))
            })?;
            if entry.severity_from.is_some()
                || entry.severity_to.is_some()
                || entry.nature_from.is_some()
                || entry.nature_to.is_some()
            {
                return Err(CatalogueError::TransitionInvalide(
                    "sévérité/nature interdites pour rectification".into(),
                ));
            }
        }
    }
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
            nature: EntryNature::Constat,
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
    let mut delivered_by: BTreeMap<String, &TransitionEntry> = BTreeMap::new();
    let mut severity_override: BTreeMap<String, Severity> = BTreeMap::new();
    let mut nature_override: BTreeMap<String, EntryNature> = BTreeMap::new();
    let mut requalifie_ids: BTreeSet<String> = BTreeSet::new();
    let mut rectifie_ids: BTreeSet<String> = BTreeSet::new();
    let mut rectifies: Vec<RectifiedConstatView> = Vec::new();
    let mut adds: BTreeMap<String, &AddEntry> = BTreeMap::new();
    let mut pendings: BTreeMap<String, &PendingQualificationEntry> = BTreeMap::new();

    for entry in entries {
        match entry {
            CatalogueEntry::Add(add) => {
                adds.insert(add.id.clone(), add);
            }
            CatalogueEntry::Transition(transition) => match transition.trigger {
                TransitionTrigger::Requalified => {
                    requalifie_ids.insert(transition.constat_id.clone());
                    if let Some(severity) = transition.severity_to {
                        severity_override.insert(transition.constat_id.clone(), severity);
                    }
                    if let Some(nature) = transition.nature_to {
                        nature_override.insert(transition.constat_id.clone(), nature);
                    }
                }
                TransitionTrigger::ObjectiveClosed
                | TransitionTrigger::RemediedAttested
                | TransitionTrigger::Refuted => {
                    // Dernière livraison gagne (après une rectification, une
                    // nouvelle fermeture reprend l'autorité).
                    delivered_by.insert(transition.constat_id.clone(), transition);
                }
                TransitionTrigger::Rectified => {
                    if let Some(prev) = delivered_by.remove(&transition.constat_id) {
                        let add = adds.get(&transition.constat_id);
                        let closed_kind = match prev.trigger {
                            TransitionTrigger::Refuted => ClosedKind::Refute,
                            TransitionTrigger::ObjectiveClosed => ClosedKind::Solde,
                            _ => ClosedKind::Traite,
                        };
                        rectifies.push(RectifiedConstatView {
                            id: transition.constat_id.clone(),
                            text: add.map(|a| a.text.clone()).unwrap_or_default(),
                            closed_kind,
                            closed_at: prev.observed_at.clone(),
                            closed_raison: prev.raison.clone(),
                            closed_reference: prev.reference.clone(),
                            rectified_at: transition.observed_at.clone(),
                            rectifie_raison: transition.raison.clone(),
                            rectifie_reference: transition.reference.clone(),
                            recurrence_of: add.and_then(|a| a.recurrence_of.clone()),
                        });
                    }
                    rectifie_ids.insert(transition.constat_id.clone());
                }
            },
            CatalogueEntry::PendingQualification(pending) => {
                pendings.insert(pending.id.clone(), pending);
            }
        }
    }

    let mut ouverts: Vec<OpenConstatView> = adds
        .values()
        .filter(|add| !delivered_by.contains_key(&add.id))
        .map(|add| {
            let requalifie = requalifie_ids.contains(&add.id);
            let rectifie = rectifie_ids.contains(&add.id);
            let severity = severity_override
                .get(&add.id)
                .copied()
                .unwrap_or(add.severity);
            let nature = nature_override
                .get(&add.id)
                .copied()
                .unwrap_or(add.nature);
            OpenConstatView {
                id: add.id.clone(),
                date: add.date.clone(),
                severity,
                nature,
                recurrence_of: add.recurrence_of.clone(),
                gate_failed: add.mission_source.is_failed_gate(),
                text: add.text.clone(),
                mission_source: add.mission_source.clone(),
                requalifie,
                rectifie,
            }
        })
        .collect();

    ouverts.sort_by(compare_open_constats);

    let mut traites: Vec<ClosedConstatView> = Vec::new();
    let mut soldes: Vec<ClosedConstatView> = Vec::new();
    let mut refutes: Vec<ClosedConstatView> = Vec::new();
    for (constat_id, transition) in &delivered_by {
        let Some(add) = adds.get(constat_id) else {
            continue;
        };
        let kind = match transition.trigger {
            TransitionTrigger::Refuted => ClosedKind::Refute,
            TransitionTrigger::ObjectiveClosed => ClosedKind::Solde,
            _ => ClosedKind::Traite,
        };
        let closed = ClosedConstatView {
            id: add.id.clone(),
            kind,
            observed_at: transition.observed_at.clone(),
            text: add.text.clone(),
            raison: transition.raison.clone(),
            reference: transition.reference.clone(),
            recurrence_of: add.recurrence_of.clone(),
            trigger: transition.trigger,
            objective_id: transition.objective_id.clone(),
        };
        match kind {
            ClosedKind::Traite => traites.push(closed),
            ClosedKind::Solde => soldes.push(closed),
            ClosedKind::Refute => refutes.push(closed),
        }
    }
    traites.sort_by(|a, b| a.observed_at.cmp(&b.observed_at).then_with(|| a.id.cmp(&b.id)));
    soldes.sort_by(|a, b| a.observed_at.cmp(&b.observed_at).then_with(|| a.id.cmp(&b.id)));
    refutes.sort_by(|a, b| a.observed_at.cmp(&b.observed_at).then_with(|| a.id.cmp(&b.id)));
    rectifies.sort_by(|a, b| {
        a.rectified_at
            .cmp(&b.rectified_at)
            .then_with(|| a.id.cmp(&b.id))
    });

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

    let ouverts_du = ouverts
        .iter()
        .filter(|item| item.nature == EntryNature::Constat)
        .count();
    let ouverts_regle = ouverts
        .iter()
        .filter(|item| item.nature == EntryNature::Regle)
        .count();
    let ouverts_resultat = ouverts
        .iter()
        .filter(|item| item.nature == EntryNature::Resultat)
        .count();
    let recurrents = ouverts
        .iter()
        .filter(|item| item.nature == EntryNature::Constat && item.recurrence_of.is_some())
        .count();
    let gates_rates = ouverts
        .iter()
        .filter(|item| item.nature == EntryNature::Constat && item.gate_failed)
        .count();
    let requalifies = ouverts.iter().filter(|item| item.requalifie).count();
    let footer = RegistreFooter {
        ouverts: ouverts_du,
        ouverts_regle,
        ouverts_resultat,
        recurrents,
        gates_rates,
        pending_qualification: attente.len(),
        traites: traites.len(),
        soldes: soldes.len(),
        refutes: refutes.len(),
        requalifies,
        rectifies: rectifies.len(),
    };
    RegistreView {
        ouverts,
        attente,
        traites,
        soldes,
        refutes,
        rectifies,
        footer,
    }
}

fn resolve_open_nature(constat_id: &str, entries: &[CatalogueEntry]) -> Option<EntryNature> {
    let mut nature = None;
    for entry in entries {
        match entry {
            CatalogueEntry::Add(add) if add.id == constat_id => {
                nature = Some(add.nature);
            }
            CatalogueEntry::Transition(transition)
                if transition.constat_id == constat_id
                    && transition.trigger == TransitionTrigger::Requalified =>
            {
                if let Some(to) = transition.nature_to {
                    nature = Some(to);
                }
            }
            _ => {}
        }
    }
    nature
}

/// Dernier trigger de livraison encore actif (`None` = ouvert).
fn current_delivery_trigger(
    constat_id: &str,
    entries: &[CatalogueEntry],
) -> Option<TransitionTrigger> {
    let mut current = None;
    for entry in entries {
        let CatalogueEntry::Transition(transition) = entry else {
            continue;
        };
        if transition.constat_id != constat_id {
            continue;
        }
        match transition.trigger {
            TransitionTrigger::ObjectiveClosed
            | TransitionTrigger::RemediedAttested
            | TransitionTrigger::Refuted => {
                current = Some(transition.trigger);
            }
            TransitionTrigger::Rectified => {
                current = None;
            }
            TransitionTrigger::Requalified => {}
        }
    }
    current
}

fn is_currently_delivered(constat_id: &str, entries: &[CatalogueEntry]) -> bool {
    current_delivery_trigger(constat_id, entries).is_some()
}

fn resolve_open_severity(constat_id: &str, entries: &[CatalogueEntry]) -> Option<Severity> {
    let mut severity = None;
    for entry in entries {
        match entry {
            CatalogueEntry::Add(add) if add.id == constat_id => {
                severity = Some(add.severity);
            }
            CatalogueEntry::Transition(transition)
                if transition.constat_id == constat_id
                    && transition.trigger == TransitionTrigger::Requalified =>
            {
                if let Some(to) = transition.severity_to {
                    severity = Some(to);
                }
            }
            _ => {}
        }
    }
    severity
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

/// Sections de `registre list`.
/// Défaut : ouverts seuls + pied (les trois comptes).
/// `--fermes` / `--refutes` / `--rectifies` **restreignent** le corps à cet
/// état — ils n'ajoutent pas une section sous le mur des ouverts. `--attente`
/// seul reste un dépliage additif sur la vue des ouverts.
#[derive(Debug, Clone, Copy, Default)]
pub struct RegistreListSections {
    pub fermes: bool,
    pub refutes: bool,
    pub attente: bool,
    /// Historique des fermetures erronées puis amendées.
    pub rectifies: bool,
}

/// Rend la vue humaine d'autorité (SC-1704) : ouverts par défaut + pied.
pub fn render_registre_list(view: &RegistreView) -> String {
    render_registre_list_sections(view, RegistreListSections::default())
}

/// Compat : même rendu que `--attente` seul.
pub fn render_registre_list_with_attente(view: &RegistreView, show_attente: bool) -> String {
    render_registre_list_sections(
        view,
        RegistreListSections {
            attente: show_attente,
            ..RegistreListSections::default()
        },
    )
}

/// Vue filtrable : ouverts par défaut ; `--fermes` / `--refutes` / `--rectifies`
/// remplacent le corps (pas un élargissement). Pied toujours à trois comptes.
pub fn render_registre_list_sections(view: &RegistreView, sections: RegistreListSections) -> String {
    let mut out = String::new();
    out.push_str("registre list\n");
    // Restriction : dès qu'on demande un état spécialisé, les ouverts quittent
    // le corps — sinon 185 ouverts noient la relecture.
    let show_ouverts = !sections.fermes && !sections.refutes && !sections.rectifies;
    if show_ouverts {
        if view.ouverts.is_empty() {
            out.push_str("(aucun constat ouvert)\n");
        } else {
            for item in &view.ouverts {
                // Trois situations distinctes : jamais touchée / requalifiée /
                // fermée à tort puis rouverte.
                let badge = if item.rectifie {
                    "[RECTIFIÉ]"
                } else if item.requalifie {
                    "[REQUALIFIÉ]"
                } else {
                    "[OUVERT]"
                };
                let recurrence = item
                    .recurrence_of
                    .as_deref()
                    .map(|id| format!(" recurrence_of={id}"))
                    .unwrap_or_default();
                let gate = if item.gate_failed { " gate=failed" } else { "" };
                out.push_str(&format!(
                    "- {badge} [{severity:?}] nature={nature} {id} {date} source={source_kind}/{source_id}{recurrence}{gate}\n  {text}\n",
                    severity = item.severity,
                    nature = item.nature.as_str(),
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
    }
    if sections.fermes {
        out.push_str("--- fermés / soldés (fond prouvé ≠ mission soldée) ---\n");
        if view.traites.is_empty() && view.soldes.is_empty() {
            out.push_str("(aucun constat fermé ni soldé)\n");
        } else {
            for item in &view.traites {
                let recurrence = item
                    .recurrence_of
                    .as_deref()
                    .map(|id| format!(" recurrence_of={id}"))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "- [FERMÉ] {id} {observed} raison={raison} ref={reference}{recurrence}\n  {text}\n",
                    id = item.id,
                    observed = item.observed_at,
                    raison = item.raison.as_deref().unwrap_or("-"),
                    reference = item.reference.as_deref().unwrap_or("-"),
                    text = item.text,
                ));
            }
            for item in &view.soldes {
                let recurrence = item
                    .recurrence_of
                    .as_deref()
                    .map(|id| format!(" recurrence_of={id}"))
                    .unwrap_or_default();
                // Mission soldée ≠ défaut corrigé : badge et motif distincts,
                // jamais raison=-/ref=- qui se lisent comme preuve manquante.
                out.push_str(&format!(
                    "- [SOLDÉ] {id} {observed} motif=mission_soldee objective={objective}{recurrence}\n  {text}\n",
                    id = item.id,
                    observed = item.observed_at,
                    objective = item.objective_id,
                    text = item.text,
                ));
            }
        }
    }
    if sections.refutes {
        out.push_str("--- réfutés (ce ne fut jamais vrai) ---\n");
        if view.refutes.is_empty() {
            out.push_str("(aucun constat réfuté)\n");
        } else {
            for item in &view.refutes {
                let recurrence = item
                    .recurrence_of
                    .as_deref()
                    .map(|id| format!(" recurrence_of={id}"))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "- [RÉFUTÉ] {id} {observed} raison={raison} ref={reference}{recurrence}\n  {text}\n",
                    id = item.id,
                    observed = item.observed_at,
                    raison = item.raison.as_deref().unwrap_or("-"),
                    reference = item.reference.as_deref().unwrap_or("-"),
                    text = item.text,
                ));
            }
        }
    }
    if sections.rectifies {
        out.push_str(
            "--- rectifiés (fermés à tort puis rouverts — historique des deux transitions) ---\n",
        );
        if view.rectifies.is_empty() {
            out.push_str("(aucune rectification)\n");
        } else {
            for item in &view.rectifies {
                let recurrence = item
                    .recurrence_of
                    .as_deref()
                    .map(|id| format!(" recurrence_of={id}"))
                    .unwrap_or_default();
                let closed_label = match item.closed_kind {
                    ClosedKind::Traite => "fermeture",
                    ClosedKind::Solde => "solde_mission",
                    ClosedKind::Refute => "réfutation",
                };
                out.push_str(&format!(
                    "- [RECTIFIÉ] {id}{recurrence}\n  fautive={closed_label} {closed_at} raison={closed_raison} ref={closed_ref}\n  amendement={rect_at} raison={rect_raison} ref={rect_ref}\n  {text}\n",
                    id = item.id,
                    closed_at = item.closed_at,
                    closed_raison = item.closed_raison.as_deref().unwrap_or("-"),
                    closed_ref = item.closed_reference.as_deref().unwrap_or("-"),
                    rect_at = item.rectified_at,
                    rect_raison = item.rectifie_raison.as_deref().unwrap_or("-"),
                    rect_ref = item.rectifie_reference.as_deref().unwrap_or("-"),
                    text = item.text,
                ));
            }
        }
    }
    if sections.attente {
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
    // Pied : les états toujours, même si le corps est filtré.
    // FERMÉS = fond prouvé ; SOLDÉS = mission soldée (pas un dû réglé).
    out.push_str(&format!(
        "pied: {n} DÛ dont {m} récurrents, {k} gates ratés ; {nr} RÈGLES ; {ns} RÉSULTATS ; {p} en attente ; {t} FERMÉS, {s} SOLDÉS, {r} RÉFUTÉS, {q} REQUALIFIÉS, {x} RECTIFIÉS\n",
        n = view.footer.ouverts,
        m = view.footer.recurrents,
        k = view.footer.gates_rates,
        nr = view.footer.ouverts_regle,
        ns = view.footer.ouverts_resultat,
        p = view.footer.pending_qualification,
        t = view.footer.traites,
        s = view.footer.soldes,
        r = view.footer.refutes,
        q = view.footer.requalifies,
        x = view.footer.rectifies,
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
            nature: EntryNature::Constat,
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
        let root =
            std::env::temp_dir().join(format!("maicie-catalogue-torn-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        {
            let mut journal = CatalogueJournal::open(&path).unwrap();
            journal
                .append_add(sample_add("kept-1", Severity::Info, "2026-08-24T06:00:00Z"))
                .unwrap();
            journal
                .append_add(sample_add(
                    "kept-2",
                    Severity::Minor,
                    "2026-08-24T06:01:00Z",
                ))
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
        assert!(
            cut > last_start,
            "la troncature doit couper au milieu de la dernière ligne"
        );
        fs::write(&path, &without_final_lf.as_bytes()[..cut]).unwrap();

        let parsed = parse_journal_bytes(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            parsed.entries.len(),
            2,
            "les N-1 premières restent lisibles"
        );
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

    #[test]
    fn reconcile_attested_closures_une_seule_transition_au_rejeu() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-reconcile-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        journal
            .append_add(sample_add(
                "c-open",
                Severity::Major,
                "2026-08-24T06:00:00Z",
            ))
            .unwrap();

        let link = ArbitrationLink {
            constat_id: "c-open".into(),
            objective_id: "obj-1".into(),
        };
        let closure = AttestedClosure {
            objective_id: "obj-1".into(),
            observed_at: "2026-08-24T07:00:00Z".into(),
        };

        // Sans lien : clôture orpheline n'écrit rien.
        let orphan = journal
            .reconcile_attested_closures(&[], std::slice::from_ref(&closure))
            .unwrap();
        assert_eq!(orphan.appended, 0);
        assert_eq!(journal.read_entries().unwrap().len(), 1);

        // Sans clôture : le lien seul n'écrit rien.
        let waiting = journal
            .reconcile_attested_closures(std::slice::from_ref(&link), &[])
            .unwrap();
        assert_eq!(waiting.appended, 0);

        // Lien + clôture → une transition.
        let first = journal
            .reconcile_attested_closures(
                std::slice::from_ref(&link),
                std::slice::from_ref(&closure),
            )
            .unwrap();
        assert_eq!(first.appended, 1);
        assert_eq!(first.skipped, 0);

        // Rejeu → no-op (SC-1703).
        let replay = journal
            .reconcile_attested_closures(
                std::slice::from_ref(&link),
                std::slice::from_ref(&closure),
            )
            .unwrap();
        assert_eq!(replay.appended, 0);
        assert_eq!(replay.skipped, 1);

        let entries = journal.read_entries().unwrap();
        assert_eq!(entries.len(), 2);
        let view = project_registre(&entries);
        assert_eq!(view.footer.ouverts, 0);
        assert!(matches!(
            &entries[1],
            CatalogueEntry::Transition(t)
                if t.constat_id == "c-open"
                    && t.objective_id == "obj-1"
                    && t.trigger == TransitionTrigger::ObjectiveClosed
        ));

        // Clôture d'un autre objectif sans lien : zéro écriture (pas d'homonymie).
        let unrelated = journal
            .reconcile_attested_closures(
                &[],
                &[AttestedClosure {
                    objective_id: "obj-homonyme-titre".into(),
                    observed_at: "2026-08-24T08:00:00Z".into(),
                }],
            )
            .unwrap();
        assert_eq!(unrelated.appended, 0);
        assert_eq!(journal.read_entries().unwrap().len(), 2);

        // Lien vers un constat absent : refus franc.
        let err = journal
            .append_delivered_for_attested_closure(
                &ArbitrationLink {
                    constat_id: "absent".into(),
                    objective_id: "obj-x".into(),
                },
                &AttestedClosure {
                    objective_id: "obj-x".into(),
                    observed_at: "2026-08-24T09:00:00Z".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(
            err,
            CatalogueError::ReferenceInconnue {
                field: "constat_id",
                ..
            }
        ));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn fr1711_gate_failed_derive_blocker() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-fr1711-gate-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        let gate = ObservedFact {
            kind: "gate_failed".into(),
            source_id: "G1701".into(),
            date: "2026-08-24T06:30:00Z".into(),
            text: "gate G1701 rouge".into(),
        };
        let (out, first) = journal.consign_observed_fact(&gate).unwrap();
        assert_eq!(first, AppendOutcome::Appended);
        match out {
            TranscriptionOutcome::CoveredAdd(add) => {
                assert_eq!(add.id, "gate_failed:G1701");
                assert_eq!(add.severity, Severity::Blocker);
                assert_eq!(add.mission_source.kind, MissionSourceKind::Gate);
                assert_eq!(add.mission_source.failed, Some(true));
            }
            TranscriptionOutcome::Pending(_) => panic!("gate_failed doit être couvert"),
        }
        let (_, replay) = journal.consign_observed_fact(&gate).unwrap();
        assert_eq!(replay, AppendOutcome::IdempotentNoop);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn fr1711_review_amender_derive_major() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-fr1711-amender-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        let amender = ObservedFact {
            kind: "review_amender".into(),
            source_id: "r-hostile".into(),
            date: "2026-08-24T06:31:00Z".into(),
            text: "AMENDER : fenêtre de corruption".into(),
        };
        let (out, first) = journal.consign_observed_fact(&amender).unwrap();
        assert_eq!(first, AppendOutcome::Appended);
        match out {
            TranscriptionOutcome::CoveredAdd(add) => {
                assert_eq!(add.id, "review_amender:r-hostile");
                assert_eq!(add.severity, Severity::Major);
                assert_eq!(add.mission_source.kind, MissionSourceKind::Review);
                assert!(add.mission_source.failed.is_none());
            }
            TranscriptionOutcome::Pending(_) => panic!("review_amender doit être couvert"),
        }
        let (_, replay) = journal.consign_observed_fact(&amender).unwrap();
        assert_eq!(replay, AppendOutcome::IdempotentNoop);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn fr1711_type_hors_table_part_en_attente_sans_inventer() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-fr1711-hors-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        // APPROVE n'est pas dans la table : décision absente en amont → attente.
        let approve = ObservedFact {
            kind: "review_approve".into(),
            source_id: "r-ok".into(),
            date: "2026-08-24T06:32:00Z".into(),
            text: "APPROVE sans case dans la table".into(),
        };
        let (out, first) = journal.consign_observed_fact(&approve).unwrap();
        assert_eq!(first, AppendOutcome::Appended);
        match out {
            TranscriptionOutcome::Pending(pending) => {
                assert!(pending.id.contains("review_approve"));
                assert!(pending.provenance_id.starts_with("uncovered:"));
                assert_eq!(pending.text, "APPROVE sans case dans la table");
            }
            TranscriptionOutcome::CoveredAdd(_) => {
                panic!("hors table ne doit pas produire d'add ni de sévérité")
            }
        }
        let view = project_registre(&journal.read_entries().unwrap());
        assert_eq!(view.footer.ouverts, 0);
        assert_eq!(view.footer.pending_qualification, 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn traite_et_refute_ne_se_lisent_pas_pareil() {
        let root = std::env::temp_dir().join(format!(
            "maicie-catalogue-polarite-028-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let add = AddEntry {
            v: 1,
            kind: AddKind::Add,
            id: "c-charge".into(),
            date: "2026-08-24T22:00:00+02:00".into(),
            mission_source: MissionSource {
                kind: MissionSourceKind::Review,
                id: "rev-jury".into(),
                failed: None,
            },
            severity: Severity::Major,
            nature: EntryNature::Constat,
            recurrence_of: None,
            text: "trois décisions sans témoin".into(),
        };
        let mut j_t = CatalogueJournal::open(root.join("t.jsonl")).unwrap();
        j_t.append_add(add.clone()).unwrap();
        j_t.close_constat_attested(
            "c-charge",
            RaisonFermeture::CorrigeParLot,
            "mesure:3/3",
            "2026-08-25T08:00:00Z",
        )
        .unwrap();
        let mut j_r = CatalogueJournal::open(root.join("r.jsonl")).unwrap();
        j_r.append_add(add).unwrap();
        j_r.refute_constat_attested(
            "c-charge",
            RaisonRefutation::ChargeFausseMesuree,
            "mesure:3/3",
            "2026-08-25T08:00:00Z",
        )
        .unwrap();
        let vt = project_registre(&j_t.read_entries().unwrap());
        let vr = project_registre(&j_r.read_entries().unwrap());
        assert_eq!(vt.footer.traites, 1);
        assert_eq!(vt.footer.refutes, 0);
        assert_eq!(vr.footer.traites, 0);
        assert_eq!(vr.footer.refutes, 1);
        let sections = RegistreListSections {
            fermes: true,
            refutes: true,
            attente: false,
            rectifies: false,
        };
        let rt = render_registre_list_sections(&vt, sections);
        let rr = render_registre_list_sections(&vr, sections);
        assert!(rt.contains("[FERMÉ]"));
        assert!(!rt.contains("[RÉFUTÉ]"));
        assert!(rr.contains("[RÉFUTÉ]"));
        assert!(!rr.contains("[FERMÉ] c-charge"));
        assert_ne!(rt, rr);
        let _ = fs::remove_dir_all(&root);
    }

    fn add_avec_texte(id: &str, text: &str) -> AddEntry {
        AddEntry {
            v: 1,
            kind: AddKind::Add,
            id: id.into(),
            date: "2026-08-26T18:00:00+02:00".into(),
            mission_source: MissionSource {
                kind: MissionSourceKind::Review,
                id: "rev-temoin".into(),
                failed: None,
            },
            severity: Severity::Major,
            nature: EntryNature::Constat,
            recurrence_of: None,
            text: text.into(),
        }
    }

    /// (A) Ouvert → corps + pied exacts. Mutant text:"" ou id autre → A seul meurt.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_A_registre_ouvert_apparait_avec_contenu_exact() {
        let root = std::env::temp_dir().join(format!(
            "maicie-temoin-a-ouvert-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut journal = CatalogueJournal::open(root.join("c.jsonl")).unwrap();
        journal
            .append_add(add_avec_texte("temoin-ouvert-1", "TEXTE OUVERT EXACT"))
            .unwrap();
        let view = project_registre(&journal.read_entries().unwrap());
        let rendered = render_registre_list(&view);
        assert!(
            rendered.contains("temoin-ouvert-1"),
            "id ouvert en dur absent: {rendered}"
        );
        assert!(
            rendered.contains("TEXTE OUVERT EXACT"),
            "texte exact absent: {rendered}"
        );
        assert!(
            rendered.contains("[OUVERT]"),
            "badge OUVERT absent: {rendered}"
        );
        assert!(
            rendered.contains("pied: 1 DÛ") && rendered.contains("0 FERMÉS, 0 SOLDÉS, 0 RÉFUTÉS"),
            "pied ouvert exact attendu, reçu: {rendered}"
        );
        assert!(
            !rendered.contains("--- fermés") && !rendered.contains("--- réfutés"),
            "défaut list = ouverts seuls, sections fermées absentes: {rendered}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// (B) Fermé quitte les ouverts ; prouve d'abord l'existence ; un autre ouvert reste
    /// dans la vue **par défaut**. La vue `--fermes` montre le fermé (raison+réf) et
    /// n'élargit plus avec les ouverts — mutant qui garde l'additif : compagnon [OUVERT]
    /// dans `--fermes` → B meurt. Mutant qui omet le fermé de `--fermes` → B meurt.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_B_registre_ferme_quitte_les_ouverts_sans_vider_la_vue() {
        let root = std::env::temp_dir().join(format!(
            "maicie-temoin-b-ferme-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut journal = CatalogueJournal::open(root.join("c.jsonl")).unwrap();
        journal
            .append_add(add_avec_texte("temoin-a-fermer", "CHARGE A FERMER"))
            .unwrap();
        journal
            .append_add(add_avec_texte("temoin-reste-ouvert", "CHARGE QUI RESTE"))
            .unwrap();
        let avant = project_registre(&journal.read_entries().unwrap());
        let ids_avant: Vec<&str> = avant.ouverts.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids_avant,
            vec!["temoin-a-fermer", "temoin-reste-ouvert"],
            "précondition : les deux ouverts existent avant fermeture, reçu {ids_avant:?}"
        );
        assert_eq!(avant.footer.ouverts, 2);
        assert_eq!(avant.footer.traites, 0);
        journal
            .close_constat_attested(
                "temoin-a-fermer",
                RaisonFermeture::CorrigeParLot,
                "sha:a9353c1",
                "2026-08-26T20:00:00Z",
            )
            .unwrap();
        let apres = project_registre(&journal.read_entries().unwrap());
        let ids_apres: Vec<&str> = apres.ouverts.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids_apres,
            vec!["temoin-reste-ouvert"],
            "fermé hors ouverts + compagnon toujours là, reçu {ids_apres:?}"
        );
        assert_eq!(apres.footer.ouverts, 1);
        assert_eq!(apres.footer.traites, 1);
        assert_eq!(apres.footer.refutes, 0);

        // Contrôle positif vue défaut : compagnon présent, fermé retiré.
        let defaut = render_registre_list(&apres);
        assert!(
            defaut.contains("[OUVERT]") && defaut.contains("temoin-reste-ouvert"),
            "défaut doit encore montrer le compagnon ouvert: {defaut}"
        );
        assert!(
            !defaut.contains("temoin-a-fermer") && !defaut.contains("[FERMÉ]"),
            "défaut ne doit plus montrer ni lister le fermé: {defaut}"
        );
        assert!(
            defaut.contains("pied: 1 DÛ") && defaut.contains("1 FERMÉS, 0 SOLDÉS, 0 RÉFUTÉS"),
            "pied trois comptes après fermeture, reçu: {defaut}"
        );

        let rendered = render_registre_list_sections(
            &apres,
            RegistreListSections {
                fermes: true,
                refutes: false,
                attente: false,
            rectifies: false,
        },
        );
        assert!(
            rendered.contains("[FERMÉ] temoin-a-fermer")
                && rendered.contains("raison=corrige_par_lot")
                && rendered.contains("ref=sha:a9353c1"),
            "fermeture avec motif+réf en dur dans --fermes, reçu: {rendered}"
        );
        assert!(
            !rendered.contains("[OUVERT]") && !rendered.contains("temoin-reste-ouvert"),
            "--fermes restreint : pas d'ouverts (sinon 185+40=élargissement), reçu: {rendered}"
        );
        assert!(
            rendered.contains("pied: 1 DÛ") && rendered.contains("1 FERMÉS, 0 SOLDÉS, 0 RÉFUTÉS"),
            "pied inchangé sous filtre, reçu: {rendered}"
        );
        // Append-only : le fichier contient encore l'add d'origine + transition.
        let raw = fs::read_to_string(root.join("c.jsonl")).unwrap();
        assert!(
            raw.lines().count() >= 3,
            "append-only : add+add+transition, lignes={}",
            raw.lines().count()
        );
        assert!(
            raw.contains("\"kind\":\"add\"") && raw.contains("\"kind\":\"transition\""),
            "pas de réécriture destructive: {raw}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// (D) Oracle ciblé : une entrée fermée DOIT apparaître dans la vue des fermés.
    /// Meurt sur projection vide (prouve d'abord défaut ≠ vide et fermé hors défaut).
    /// Mutant : `--fermes` omet [FERMÉ] ou n'écrit pas raison/réf → D seul meurt.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_D_vue_fermes_montre_lentree_fermee_avec_preuve() {
        let root = std::env::temp_dir().join(format!(
            "maicie-temoin-d-vue-fermes-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut journal = CatalogueJournal::open(root.join("c.jsonl")).unwrap();
        journal
            .append_add(add_avec_texte(
                "constat/echeance-dix-minutes-condamne-tout-tour-long",
                "ECHEANCE QUI CONDAMNE",
            ))
            .unwrap();
        journal
            .append_add(add_avec_texte(
                "temoin-compagnon-ouvert",
                "RESTE OUVERT POUR CONTROLE",
            ))
            .unwrap();

        let avant = render_registre_list(&project_registre(&journal.read_entries().unwrap()));
        assert!(
            avant.contains("[OUVERT]")
                && avant.contains("constat/echeance-dix-minutes-condamne-tout-tour-long")
                && avant.contains("temoin-compagnon-ouvert"),
            "précondition défaut non vide (sinon assertion négative vacue): {avant}"
        );

        journal
            .close_constat_attested(
                "constat/echeance-dix-minutes-condamne-tout-tour-long",
                RaisonFermeture::CorrigeEnProduction,
                "sha:c3782e7b1c398ccf19d26d262f4f7cc169bcd16b",
                "2026-08-27T01:00:00Z",
            )
            .unwrap();
        let apres = project_registre(&journal.read_entries().unwrap());
        let defaut = render_registre_list(&apres);
        assert!(
            defaut.contains("[OUVERT]") && defaut.contains("temoin-compagnon-ouvert"),
            "contrôle positif : un ouvert apparaît encore au défaut: {defaut}"
        );
        assert!(
            !defaut.contains("constat/echeance-dix-minutes-condamne-tout-tour-long"),
            "fermé a quitté le défaut: {defaut}"
        );

        let fermes = render_registre_list_sections(
            &apres,
            RegistreListSections {
                fermes: true,
                refutes: false,
                attente: false,
            rectifies: false,
        },
        );
        assert!(
            fermes.contains("--- fermés")
                && fermes.contains(
                    "[FERMÉ] constat/echeance-dix-minutes-condamne-tout-tour-long"
                )
                && fermes.contains("raison=corrige_en_production")
                && fermes.contains("ref=sha:c3782e7b1c398ccf19d26d262f4f7cc169bcd16b"),
            "vue --fermes doit montrer le fermé avec raison typée et référence: {fermes}"
        );
        assert!(
            !fermes.contains("[OUVERT]") && !fermes.contains("temoin-compagnon-ouvert"),
            "vue --fermes ne doit pas élargir aux ouverts: {fermes}"
        );
        assert!(
            fermes.contains("1 FERMÉS, 0 SOLDÉS, 0 RÉFUTÉS"),
            "pied distingue fermés/réfutés: {fermes}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// (C) Réfuté ≠ fermé. Mutant qui appelle close au lieu de refute → C meurt (FERMÉS=1).
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_C_registre_refute_distinct_du_ferme() {
        let root = std::env::temp_dir().join(format!(
            "maicie-temoin-c-refute-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut journal = CatalogueJournal::open(root.join("c.jsonl")).unwrap();
        journal
            .append_add(add_avec_texte("temoin-a-refuter", "FAUSSE ALERTE"))
            .unwrap();
        journal
            .refute_constat_attested(
                "temoin-a-refuter",
                RaisonRefutation::ChargeFausseMesuree,
                "mesure:4/4",
                "2026-08-26T20:05:00Z",
            )
            .unwrap();
        let view = project_registre(&journal.read_entries().unwrap());
        assert_eq!(view.footer.ouverts, 0);
        assert_eq!(view.footer.traites, 0);
        assert_eq!(view.footer.refutes, 1);
        let rendered = render_registre_list_sections(
            &view,
            RegistreListSections {
                fermes: false,
                refutes: true,
                attente: false,
            rectifies: false,
        },
        );
        assert!(
            rendered.contains("[RÉFUTÉ] temoin-a-refuter")
                && rendered.contains("raison=charge_fausse_mesuree")
                && rendered.contains("ref=mesure:4/4"),
            "réfutation exacte dans --refutes, reçu: {rendered}"
        );
        assert!(
            !rendered.contains("[FERMÉ]") && !rendered.contains("[OUVERT]"),
            "--refutes restreint, pas fermé ni ouvert: {rendered}"
        );
        assert!(
            rendered.contains("0 FERMÉS, 0 SOLDÉS, 1 RÉFUTÉS"),
            "pied trois états, reçu: {rendered}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Contrôle positif d'abord : un constat se ferme. Puis la règle refuse.
    #[test]
    fn oracle_une_regle_ne_peut_pas_etre_fermee() {
        let root = std::env::temp_dir().join(format!(
            "maicie-nature-close-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        let mut constat = sample_add("c-ok", Severity::Major, "2026-08-27T01:00:00Z");
        constat.nature = EntryNature::Constat;
        journal.append_add(constat).unwrap();
        assert_eq!(
            journal
                .close_constat_attested(
                    "c-ok",
                    RaisonFermeture::CorrigeParLot,
                    "sha:4f6cf27",
                    "2026-08-27T01:01:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — un constat DOIT pouvoir se fermer (contrôle positif)"
        );
        let mut regle = sample_add("r-interdit", Severity::Major, "2026-08-27T01:00:00Z");
        regle.nature = EntryNature::Regle;
        journal.append_add(regle).unwrap();
        let err = journal
            .close_constat_attested(
                "r-interdit",
                RaisonFermeture::CorrigeParLot,
                "sha:4f6cf27",
                "2026-08-27T01:02:00Z",
            )
            .expect_err("PROMESSE — une règle NE DOIT PAS se fermer");
        let msg = err.to_string();
        assert!(
            msg.contains("n'est pas un constat fermable") || msg.contains("regle"),
            "motif de refus attendu, reçu: {msg}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    /// Contrôle positif : un constat porte Blocker. Puis la règle refuse.
    #[test]
    fn oracle_une_regle_ne_peut_pas_porter_blocker() {
        let mut ok = sample_add("c-blocker", Severity::Blocker, "2026-08-27T01:00:00Z");
        ok.nature = EntryNature::Constat;
        assert!(
            validate_add_shape(&ok).is_ok(),
            "PROMESSE — un constat DOIT pouvoir porter blocker"
        );
        let mut regle = sample_add("r-blocker", Severity::Blocker, "2026-08-27T01:00:00Z");
        regle.nature = EntryNature::Regle;
        let err = validate_add_shape(&regle).expect_err("PROMESSE — règle+blocker interdit");
        assert!(
            err.to_string().contains("blocker"),
            "refus blocker sur règle, reçu: {err}"
        );
    }

    /// Contrôle positif : un constat compte dans le dû. Une règle n'y entre pas.
    #[test]
    fn oracle_le_pied_ne_compte_pas_une_regle_dans_le_du() {
        let constat = CatalogueEntry::Add({
            let mut e = sample_add("c-du", Severity::Major, "2026-08-27T01:00:00Z");
            e.nature = EntryNature::Constat;
            e
        });
        let regle = CatalogueEntry::Add({
            let mut e = sample_add("r-hors-du", Severity::Major, "2026-08-27T01:00:00Z");
            e.nature = EntryNature::Regle;
            e
        });
        let view = project_registre(&[constat]);
        assert_eq!(
            view.footer.ouverts, 1,
            "PROMESSE — un constat DOIT compter dans le dû"
        );
        assert_eq!(view.footer.ouverts_regle, 0);
        let view = project_registre(&[regle]);
        assert_eq!(
            view.footer.ouverts, 0,
            "PROMESSE — une règle NE DOIT PAS compter dans le dû"
        );
        assert_eq!(view.footer.ouverts_regle, 1);
        let rendered = render_registre_list(&view);
        assert!(
            rendered.contains("pied: 0 DÛ") && rendered.contains("1 RÈGLES"),
            "pied scindé, reçu: {rendered}"
        );
    }

    /// Contrôle positif d'abord : un solde de mission (`objective_closed`)
    /// DOIT pouvoir être rectifié (il n'atteste pas une correction). Puis la
    /// transition erronée remedied (fixture réelle un-zero) aussi — append-only,
    /// badge RECTIFIÉ, pas « jamais touchée ».
    #[test]
    fn oracle_une_transition_erronee_peut_etre_rectifiee() {
        let root = std::env::temp_dir().join(format!(
            "maicie-rectif-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let path = root.join("catalogue.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();

        // (1) Solde de mission : rectifiable avec solde_mission_errone.
        // Mauvaise raison (fermeture_erronee) refuse — prouve le cas légitime.
        let mut solde = sample_add("c-solde-mission", Severity::Major, "2026-08-27T01:00:00Z");
        solde.nature = EntryNature::Constat;
        journal.append_add(solde).unwrap();
        let link = ArbitrationLink {
            constat_id: "c-solde-mission".into(),
            objective_id: "obj-solde".into(),
        };
        let closure = AttestedClosure {
            objective_id: "obj-solde".into(),
            observed_at: "2026-08-27T01:01:00Z".into(),
        };
        assert_eq!(
            journal
                .append_delivered_for_attested_closure(&link, &closure)
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — un solde de mission DOIT pouvoir être posé"
        );
        let view_solde = project_registre(&journal.read_entries().unwrap());
        assert_eq!(view_solde.footer.soldes, 1);
        assert_eq!(view_solde.footer.traites, 0);
        let err_mauvaise = journal
            .rectify_constat_attested(
                "c-solde-mission",
                RaisonRectification::FermetureErronee,
                "mesure:1/1 mauvaise raison",
                "2026-08-27T01:02:00Z",
            )
            .expect_err("fermeture_erronee n'amende pas objective_closed");
        assert!(
            err_mauvaise.to_string().contains("n'amende pas"),
            "refus raison inadaptée, reçu: {err_mauvaise}"
        );
        assert_eq!(
            journal
                .rectify_constat_attested(
                    "c-solde-mission",
                    RaisonRectification::SoldeMissionErrone,
                    "mesure:1/1 solde sans preuve de correction",
                    "2026-08-27T01:03:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — un solde de mission DOIT être rectifiable"
        );
        let view_apres_solde = project_registre(&journal.read_entries().unwrap());
        assert!(
            view_apres_solde
                .ouverts
                .iter()
                .any(|o| o.id == "c-solde-mission" && o.rectifie),
            "après rectif, le solde revient ouvert avec badge RECTIFIÉ"
        );
        assert_eq!(view_apres_solde.footer.soldes, 0);
        assert_eq!(view_apres_solde.footer.rectifies, 1);

        // (2) Fixture réelle : fermeture erronée de la règle permanente.
        let mut regle = sample_add(
            "regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide",
            Severity::Blocker,
            "2026-08-25T13:14:29.267957+02:00",
        );
        // Comme au journal de prod avant reclassement : nature=constat par défaut.
        regle.nature = EntryNature::Constat;
        regle.mission_source = MissionSource {
            kind: MissionSourceKind::Review,
            id: "relec6".into(),
            failed: None,
        };
        regle.text = "REGLE — un zero doit prouver que son univers n est pas vide".into();
        journal.append_add(regle).unwrap();
        assert_eq!(
            journal
                .close_constat_attested(
                    "regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide",
                    RaisonFermeture::CorrigeEnProduction,
                    "sha:475ef10",
                    "2026-08-27T01:30:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "fixture : la fermeture erronée a été acceptée (nature encore constat)"
        );

        // Sans rectification, requalifier est refusé (déjà delivered).
        let err_req = journal
            .requalify_constat(
                "regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide",
                None,
                Some((EntryNature::Constat, EntryNature::Regle)),
                RaisonRequalification::NatureReclassee,
                None,
                "2026-08-27T01:31:00Z",
            )
            .expect_err("sans rectification, requalifier refuse le delivered");
        assert!(
            err_req.to_string().contains("delivered"),
            "motif delivered attendu, reçu: {err_req}"
        );

        // (3) Assertion principale : la transition erronée PEUT être rectifiée.
        assert_eq!(
            journal
                .rectify_constat_attested(
                    "regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide",
                    RaisonRectification::FermetureErronee,
                    "mesure:1/1 fermeture erronee d une regle permanente",
                    "2026-08-27T01:32:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — une transition erronée DOIT pouvoir être rectifiée"
        );

        let entries = journal.read_entries().unwrap();
        let view = project_registre(&entries);
        let ouvert = view
            .ouverts
            .iter()
            .find(|o| o.id == "regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide")
            .expect("après rectification, la règle revient aux ouverts");
        assert!(
            ouvert.rectifie,
            "PROMESSE — une entrée rectifiée NE se lit PAS comme jamais touchée"
        );
        assert_eq!(
            view.footer.traites, 0,
            "aucune fermeture de fond restante"
        );
        assert_eq!(
            view.footer.rectifies, 2,
            "historique : solde + fermeture rectifiés"
        );

        let raw = fs::read_to_string(&path).unwrap();
        assert!(
            raw.contains("\"trigger\":\"remedied_attested\"")
                && raw.contains("\"trigger\":\"rectified\""),
            "append-only : fermeture ET rectification coexistent, reçu: {raw}"
        );

        // Trois situations distinctes pour un lecteur futur.
        let mut intact = sample_add("c-jamais-ferme", Severity::Minor, "2026-08-27T01:00:00Z");
        intact.nature = EntryNature::Constat;
        journal.append_add(intact).unwrap();
        let entries = journal.read_entries().unwrap();
        let view = project_registre(&entries);
        let rendered = render_registre_list(&view);
        assert!(
            rendered.contains("[OUVERT]")
                && rendered.contains("c-jamais-ferme")
                && rendered.contains("[RECTIFIÉ]")
                && rendered.contains("regle/un-zero-doit-prouver-que-son-univers-n-est-pas-vide"),
            "défaut : jamais touchée ≠ rectifiée, reçu: {rendered}"
        );
        let fermes = render_registre_list_sections(
            &view,
            RegistreListSections {
                fermes: true,
                ..RegistreListSections::default()
            },
        );
        assert!(
            !fermes.contains("[SOLDÉ] c-solde-mission")
                && !fermes.contains("[FERMÉ]")
                && !fermes.contains("regle/un-zero-doit-prouver"),
            "--fermes : soldes/fermés rectifiés absents (rouverts), reçu: {fermes}"
        );
        let hist = render_registre_list_sections(
            &view,
            RegistreListSections {
                rectifies: true,
                ..RegistreListSections::default()
            },
        );
        assert!(
            hist.contains("[RECTIFIÉ]")
                && hist.contains("fautive=solde_mission")
                && hist.contains("raison=solde_mission_errone")
                && hist.contains("fautive=fermeture")
                && hist.contains("raison=corrige_en_production")
                && hist.contains("ref=sha:475ef10")
                && hist.contains("raison=fermeture_erronee")
                && hist.contains("mesure:1/1 fermeture erronee d une regle permanente")
                && !hist.contains("[OUVERT]"),
            "--rectifies : les DEUX transitions consultables, reçu: {hist}"
        );

        // Une entrée jamais delivered ne se « rectifie » pas.
        let err_open = journal
            .rectify_constat_attested(
                "c-jamais-ferme",
                RaisonRectification::FermetureErronee,
                "mesure:1/1",
                "2026-08-27T01:33:00Z",
            )
            .expect_err("rectifier un ouvert doit échouer");
        assert!(
            err_open.to_string().contains("n'est pas delivered"),
            "refus ouvert, reçu: {err_open}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// Témoin 1 — reclassement Blocker→règle.
    /// Contrôle positif d'abord (cas légitime) : sinon projection vide.
    /// Puis la garde refuse le nature-seul ; la même transition nature+sévérité passe.
    #[test]
    fn oracle_reclassement_blocker_vers_regle_franchit_la_garde_avec_trace() {
        let root = std::env::temp_dir().join(format!(
            "maicie-oracle-blocker-regle-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("c.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();

        // (0) Cas légitime : constat Major → règle sans toucher la sévérité.
        let mut ok = sample_add("c-major-vers-regle", Severity::Major, "2026-08-27T04:00:00Z");
        ok.nature = EntryNature::Constat;
        journal.append_add(ok).unwrap();
        assert_eq!(
            journal
                .requalify_constat(
                    "c-major-vers-regle",
                    None,
                    Some((EntryNature::Constat, EntryNature::Regle)),
                    RaisonRequalification::NatureReclassee,
                    Some("sha:deadbeef"),
                    "2026-08-27T04:01:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — un non-blocker DOIT pouvoir devenir règle"
        );

        // (1) Blocker → règle SANS baisser la sévérité : la garde DOIT tuer.
        let mut blocker =
            sample_add("c-blocker-vers-regle", Severity::Blocker, "2026-08-27T04:00:00Z");
        blocker.nature = EntryNature::Constat;
        journal.append_add(blocker).unwrap();
        let err = journal
            .requalify_constat(
                "c-blocker-vers-regle",
                None,
                Some((EntryNature::Constat, EntryNature::Regle)),
                RaisonRequalification::NatureReclassee,
                Some("sha:deadbeef"),
                "2026-08-27T04:02:00Z",
            )
            .expect_err("PROMESSE — nature-seul Blocker→règle DOIT échouer");
        assert!(
            err.to_string().contains("une règle ne peut pas porter blocker"),
            "motif garde attendu, reçu: {err}"
        );

        // (2) Même geste avec nature ET sévérité : passe, trace au journal.
        assert_eq!(
            journal
                .requalify_constat(
                    "c-blocker-vers-regle",
                    Some((Severity::Blocker, Severity::Major)),
                    Some((EntryNature::Constat, EntryNature::Regle)),
                    RaisonRequalification::NatureReclassee,
                    Some("sha:cafebabe"),
                    "2026-08-27T04:03:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — Blocker→règle avec baisse de sévérité DOIT passer"
        );
        let entries = journal.read_entries().unwrap();
        let view = project_registre(&entries);
        let ouvert = view
            .ouverts
            .iter()
            .find(|o| o.id == "c-blocker-vers-regle")
            .expect("reste ouvert après requalification");
        assert_eq!(ouvert.nature, EntryNature::Regle);
        assert_eq!(ouvert.severity, Severity::Major);
        assert!(ouvert.requalifie);
        let raw = fs::read_to_string(&path).unwrap();
        assert!(
            raw.contains("\"severity_from\":\"blocker\"")
                && raw.contains("\"severity_to\":\"major\"")
                && raw.contains("\"nature_from\":\"constat\"")
                && raw.contains("\"nature_to\":\"regle\""),
            "trace nature+sévérité au journal, reçu: {raw}"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn requalification_refuse_nature_from_perimee() {
        let root = std::env::temp_dir().join(format!(
            "maicie-requalify-stale-nature-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("c.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        journal
            .append_add(sample_add(
                "c-stale-nature",
                Severity::Minor,
                "2026-08-27T05:00:00Z",
            ))
            .unwrap();
        journal
            .requalify_constat(
                "c-stale-nature",
                None,
                Some((EntryNature::Constat, EntryNature::Resultat)),
                RaisonRequalification::NatureReclassee,
                None,
                "2026-08-27T05:01:00Z",
            )
            .unwrap();
        let error = journal
            .requalify_constat(
                "c-stale-nature",
                None,
                Some((EntryNature::Constat, EntryNature::Regle)),
                RaisonRequalification::NatureReclassee,
                None,
                "2026-08-27T05:02:00Z",
            )
            .expect_err("nature_from périmée doit refuser");
        assert!(
            error.to_string().contains("nature_from périmée"),
            "motif attendu: {error}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn requalification_refuse_severite_from_perimee() {
        let root = std::env::temp_dir().join(format!(
            "maicie-requalify-stale-severity-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("c.jsonl");
        let mut journal = CatalogueJournal::open(&path).unwrap();
        journal
            .append_add(sample_add(
                "c-stale-severity",
                Severity::Minor,
                "2026-08-27T05:00:00Z",
            ))
            .unwrap();
        journal
            .requalify_constat(
                "c-stale-severity",
                Some((Severity::Minor, Severity::Major)),
                None,
                RaisonRequalification::SeveriteAjustee,
                None,
                "2026-08-27T05:01:00Z",
            )
            .unwrap();
        let error = journal
            .requalify_constat(
                "c-stale-severity",
                Some((Severity::Minor, Severity::Info)),
                None,
                RaisonRequalification::SeveriteAjustee,
                None,
                "2026-08-27T05:02:00Z",
            )
            .expect_err("severity_from périmée doit refuser");
        assert!(
            error.to_string().contains("severity_from périmée"),
            "motif attendu: {error}"
        );
        let _ = fs::remove_dir_all(root);
    }

    /// Témoin 2 — solde de mission ≠ fermeture prouvée.
    /// Contrôle positif d'abord : fermeture avec preuve se lit [FERMÉ] raison+ref.
    /// Puis objective_closed se lit [SOLDÉ] motif=mission_soldee — jamais comme
    /// une fermeture prouvée à raison=- / ref=-.
    #[test]
    fn oracle_solde_mission_ne_se_lit_pas_comme_fermeture_prouvee() {
        let root = std::env::temp_dir().join(format!(
            "maicie-oracle-solde-mission-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut journal = CatalogueJournal::open(root.join("c.jsonl")).unwrap();

        // (0) Cas légitime : fermeture prouvée.
        journal
            .append_add(add_avec_texte("c-prouve", "DEFAUT CORRIGE"))
            .unwrap();
        journal
            .close_constat_attested(
                "c-prouve",
                RaisonFermeture::CorrigeParLot,
                "sha:abcdef0",
                "2026-08-27T04:10:00Z",
            )
            .unwrap();
        let view_ok = project_registre(&journal.read_entries().unwrap());
        let fermes_ok = render_registre_list_sections(
            &view_ok,
            RegistreListSections {
                fermes: true,
                ..RegistreListSections::default()
            },
        );
        assert!(
            fermes_ok.contains("[FERMÉ] c-prouve")
                && fermes_ok.contains("raison=corrige_par_lot")
                && fermes_ok.contains("ref=sha:abcdef0"),
            "PROMESSE — fermeture prouvée lisible comme telle, reçu: {fermes_ok}"
        );
        assert!(
            !fermes_ok.contains("raison=-") && !fermes_ok.contains("ref=-"),
            "fermeture prouvée ne doit pas afficher des tirets, reçu: {fermes_ok}"
        );

        // (1) Solde de mission : trigger objective_closed, sans preuve typée.
        journal
            .append_add(add_avec_texte("c-solde", "MISSION SOLDEE SEULEMENT"))
            .unwrap();
        let link = ArbitrationLink {
            constat_id: "c-solde".into(),
            objective_id: "obj-solde-42".into(),
        };
        let closure = AttestedClosure {
            objective_id: "obj-solde-42".into(),
            observed_at: "2026-08-27T04:11:00Z".into(),
        };
        assert_eq!(
            journal
                .append_delivered_for_attested_closure(&link, &closure)
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — un solde de mission DOIT pouvoir être posé"
        );
        let view = project_registre(&journal.read_entries().unwrap());
        assert_eq!(view.footer.traites, 1, "fermeture prouvée compte FERMÉS");
        assert_eq!(view.footer.soldes, 1, "solde compte SOLDÉS, pas FERMÉS");
        let solde = view
            .soldes
            .iter()
            .find(|t| t.id == "c-solde")
            .expect("solde présent dans soldes (pas traites)");
        assert!(
            view.traites.iter().all(|t| t.id != "c-solde"),
            "solde ne doit pas gonfler les fermetures de fond"
        );
        assert_eq!(solde.trigger, TransitionTrigger::ObjectiveClosed);
        assert!(solde.raison.is_none() && solde.reference.is_none());
        let fermes = render_registre_list_sections(
            &view,
            RegistreListSections {
                fermes: true,
                ..RegistreListSections::default()
            },
        );
        assert!(
            fermes.contains("[SOLDÉ] c-solde")
                && fermes.contains("motif=mission_soldee")
                && fermes.contains("objective=obj-solde-42")
                && fermes.contains("1 FERMÉS, 1 SOLDÉS"),
            "solde visible comme mission soldée au pied, reçu: {fermes}"
        );
        assert!(
            !fermes.contains("[FERMÉ] c-solde")
                && !fermes.contains("raison=-")
                && !fermes.contains("ref=-"),
            "solde NE doit PAS se lire comme fermeture prouvée sans motif, reçu: {fermes}"
        );

        // (2) Rectifiable : une fermeture sans preuve ne peut pas être inattaquable.
        assert_eq!(
            journal
                .rectify_constat_attested(
                    "c-solde",
                    RaisonRectification::SoldeMissionErrone,
                    "mesure:1/1 solde sans correction attestée",
                    "2026-08-27T04:12:00Z",
                )
                .unwrap(),
            AppendOutcome::Appended,
            "PROMESSE — objective_closed DOIT être rectifiable"
        );
        let view2 = project_registre(&journal.read_entries().unwrap());
        assert_eq!(view2.footer.soldes, 0);
        assert!(
            view2
                .ouverts
                .iter()
                .any(|o| o.id == "c-solde" && o.rectifie),
            "après rectif, le solde revient au dû"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
