//! Entités de coordination Maicie et leurs transitions explicites.
//!
//! Les états de ce module décrivent uniquement la coordination. Les faits de
//! présence, livraison et délai restent détenus par Bridget.

use bridget_transport::protocol::{
    COORDINATION_STREAM_VERSION, CoordinationEventKind, DaemonToWrapper,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use uuid::Uuid;

pub const MAX_COORDINATION_NODES: usize = 100;
pub const MAX_COORDINATION_EDGES: usize = 300;
pub const MAX_FALLBACK_CANDIDATES: usize = 32;
pub const MAX_REEMISSIONS: u8 = 8;

#[path = "guichet.rs"]
pub mod guichet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainError {
    TransitionInterdite,
    DonneeInvalide(&'static str),
    ApprobationExpiree,
    ApprobationConsommee,
    ApprobationIncoherente,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeObjectif {
    Collaboratif,
    Delegue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatObjectif {
    Ouvert,
    EnCoordination,
    AEvaluer,
    Synthetise,
    Clos,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectifCoordonne {
    pub id: Uuid,
    pub but: String,
    pub mode: ModeObjectif,
    pub etat: EtatObjectif,
    pub cree_at: i64,
    pub mis_a_jour_at: i64,
    pub synthese: Option<String>,
    pub decision_en_attente_id: Option<Uuid>,
}

/// Décision locale explicitement auditée. Elle ne déclenche aucune I/O Bridget
/// à elle seule : les effets sont portés par les outboxes dédiées.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeDecision {
    AjouterParticipant,
    RetirerParticipant,
    /// Constate une issue terminale Bridget sans la transformer en clôture.
    ConstaterIssue,
    Relancer,
    ReveillerProfil,
    Cloturer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatDecision {
    Proposee,
    Approuvee,
    Refusee,
    Appliquee,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionCoordination {
    pub id: Uuid,
    pub objectif_id: Uuid,
    pub kind: TypeDecision,
    pub proposee_par: String,
    pub etat: EtatDecision,
    pub motif: String,
}

impl DecisionCoordination {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.proposee_par.trim().is_empty() || self.motif.trim().is_empty() {
            return Err(DomainError::DonneeInvalide("décision incomplète"));
        }
        Ok(())
    }
}

impl ObjectifCoordonne {
    pub fn nouveau(
        but: impl Into<String>,
        mode: ModeObjectif,
        now: i64,
    ) -> Result<Self, DomainError> {
        let but = but.into();
        if but.trim().is_empty() {
            return Err(DomainError::DonneeInvalide("but vide"));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            but,
            mode,
            etat: EtatObjectif::Ouvert,
            cree_at: now,
            mis_a_jour_at: now,
            synthese: None,
            decision_en_attente_id: None,
        })
    }

    pub fn transition(&mut self, next: EtatObjectif, now: i64) -> Result<(), DomainError> {
        let allowed = matches!(
            (self.etat, next),
            (EtatObjectif::Ouvert, EtatObjectif::EnCoordination)
                | (EtatObjectif::EnCoordination, EtatObjectif::AEvaluer)
                | (EtatObjectif::AEvaluer, EtatObjectif::Synthetise)
        );
        if !allowed {
            return Err(DomainError::TransitionInterdite);
        }
        if next == EtatObjectif::Synthetise && self.synthese.is_none() {
            return Err(DomainError::DonneeInvalide(
                "synthèse requise avant l'état synthétisé",
            ));
        }
        self.etat = next;
        self.mis_a_jour_at = now;
        Ok(())
    }

    /// La clôture est une décision explicite ; elle ne peut jamais être déduite
    /// d'une réponse Bridget ou d'une absence d'événement.
    pub fn clore(&mut self, now: i64) -> Result<(), DomainError> {
        if self.etat == EtatObjectif::Clos {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat = EtatObjectif::Clos;
        self.mis_a_jour_at = now;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClasseDuree {
    Courte,
    Normale,
    Longue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatDelegation {
    Creee,
    AEvaluer,
    Terminee,
    Annulee,
}

/// Types d'événements produits par le registre Maicie lui-même. Les faits
/// transport A utilisent directement `CoordinationEventKind` du protocole
/// public et ne sont jamais recopiés dans cette énumération.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeEvenementAttendu {
    ClotureObjectif,
    OuvertureDelegation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FraicheurCoordination {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvenementCoordination {
    canonical_bytes: Vec<u8>,
    event_id: String,
    request_id: String,
    kind: CoordinationEventKind,
    reminder_message_id: String,
    recipient: String,
    generation: u64,
    observed_at: i64,
    cursor: u64,
    freshness: FraicheurCoordination,
}

impl EvenementCoordination {
    /// Parse l'unique DTO public A et conserve les octets canoniques exacts.
    /// La comparaison après sérialisation refuse aussi une forme JSON locale
    /// équivalente mais non canonique avant toute décision du réducteur.
    pub fn depuis_trame_attestee(
        canonical_bytes: &[u8],
        freshness: FraicheurCoordination,
    ) -> Result<Self, DomainError> {
        let frame: DaemonToWrapper = serde_json::from_slice(canonical_bytes)
            .map_err(|_| DomainError::DonneeInvalide("trame attestée invalide"))?;
        let rendered = serde_json::to_vec(&frame)
            .map_err(|_| DomainError::DonneeInvalide("trame attestée invalide"))?;
        if rendered != canonical_bytes {
            return Err(DomainError::DonneeInvalide(
                "octets attestés non canoniques",
            ));
        }
        let DaemonToWrapper::CoordinationEvent {
            version,
            event_id,
            request_id,
            kind,
            reminder_message_id,
            recipient,
            generation,
            observed_at,
            cursor: Some(cursor),
        } = frame
        else {
            return Err(DomainError::DonneeInvalide(
                "trame hors événement de coordination cursé",
            ));
        };
        if version != COORDINATION_STREAM_VERSION
            || event_id.trim().is_empty()
            || request_id.trim().is_empty()
            || reminder_message_id.trim().is_empty()
            || recipient.trim().is_empty()
            || generation == 0
            || observed_at <= 0
            || cursor == 0
        {
            return Err(DomainError::DonneeInvalide(
                "événement de coordination incomplet",
            ));
        }
        Ok(Self {
            canonical_bytes: canonical_bytes.to_vec(),
            event_id,
            request_id,
            kind,
            reminder_message_id,
            recipient,
            generation,
            observed_at,
            cursor,
            freshness,
        })
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn event_id(&self) -> &str {
        &self.event_id
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn kind(&self) -> CoordinationEventKind {
        self.kind
    }

    pub fn reminder_message_id(&self) -> &str {
        &self.reminder_message_id
    }

    pub fn recipient(&self) -> &str {
        &self.recipient
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn observed_at(&self) -> i64 {
        self.observed_at
    }

    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    pub fn freshness(&self) -> FraicheurCoordination {
        self.freshness
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeQualificationDependance {
    HashGreffe,
    ClotureEvalueeExigee,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependanceDelegation {
    pub objectif_id: Uuid,
    pub prerequis_id: Uuid,
    pub dependant_id: Uuid,
    pub mode: ModeQualificationDependance,
}

impl DependanceDelegation {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.prerequis_id == self.dependant_id {
            return Err(DomainError::DonneeInvalide("dépendance réflexive"));
        }
        Ok(())
    }
}

/// Issue fermée d'une évaluation locale. Un libellé humain n'est jamais une
/// entrée du réducteur et ne peut donc pas qualifier une arête stricte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueClotureEvaluee {
    LivraisonValidee,
}

/// Fait structuré présenté au réducteur pour produire l'acte durable. Le fait
/// porte l'instant attesté ; le réducteur ne consulte jamais l'horloge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvaluationCloture {
    pub event_id: String,
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    pub delivery_hash: String,
    pub issue: IssueClotureEvaluee,
    pub evaluated_at: i64,
}

impl EvaluationCloture {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.event_id.trim().is_empty() {
            return Err(DomainError::DonneeInvalide(
                "évaluation sans identifiant d'événement",
            ));
        }
        verifier_preuve_livraison(self.generation, &self.delivery_hash, self.evaluated_at)
    }
}

/// Preuve évaluée produite uniquement par le réducteur 016 depuis une
/// `EvaluationCloture` structurée. Ses champs privés et l'absence volontaire
/// de `Deserialize` empêchent qu'un appelant fabrique l'autorité stricte à
/// partir d'un document externe.
///
/// ```compile_fail
/// use maicie::domain::ActeClotureEvaluee;
///
/// let _: ActeClotureEvaluee = serde_json::from_str(
///     r#"{"acte_id":"00000000-0000-0000-0000-000000000000"}"#,
/// )?;
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActeClotureEvaluee {
    acte_id: Uuid,
    objectif_id: Uuid,
    delegation_id: Uuid,
    generation: u64,
    delivery_hash: String,
    issue_qualifiante: IssueClotureEvaluee,
    evaluated_at: i64,
}

impl ActeClotureEvaluee {
    pub fn verifier(&self) -> Result<(), DomainError> {
        verifier_preuve_livraison(self.generation, &self.delivery_hash, self.evaluated_at)
    }

    pub fn acte_id(&self) -> Uuid {
        self.acte_id
    }

    pub fn objectif_id(&self) -> Uuid {
        self.objectif_id
    }

    pub fn delegation_id(&self) -> Uuid {
        self.delegation_id
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn delivery_hash(&self) -> &str {
        &self.delivery_hash
    }

    pub fn issue_qualifiante(&self) -> IssueClotureEvaluee {
        self.issue_qualifiante
    }

    pub fn evaluated_at(&self) -> i64 {
        self.evaluated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttenteNotification {
    pub attente_id: Uuid,
    pub objectif_id: Uuid,
    pub delegation_id: Option<Uuid>,
    pub kind: TypeEvenementAttendu,
    pub recipient: String,
    pub policy_version: u64,
}

impl AttenteNotification {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.recipient.trim().is_empty() || self.policy_version == 0 {
            return Err(DomainError::DonneeInvalide(
                "attente de notification invalide",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaitAppartenanceRepli {
    pub objectif_id: Uuid,
    pub participant_id: String,
    pub membership_version: u64,
    pub est_pilote: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolitiqueReassignation {
    pub delegation_id: Uuid,
    pub objectif_id: Uuid,
    pub classe: ClasseDuree,
    pub version: u64,
    pub seuil_relances: u32,
    pub max_reemissions: u8,
    pub chaine_repli: Vec<FaitAppartenanceRepli>,
}

impl PolitiqueReassignation {
    pub fn verifier(&self, participants_declares: &BTreeSet<String>) -> Result<(), DomainError> {
        if self.version == 0 || self.seuil_relances == 0 {
            return Err(DomainError::DonneeInvalide(
                "politique sans version ou seuil",
            ));
        }
        if !(1..=MAX_REEMISSIONS).contains(&self.max_reemissions) {
            return Err(DomainError::DonneeInvalide(
                "borne de réémission hors 1..=8",
            ));
        }
        if self.chaine_repli.len() > MAX_FALLBACK_CANDIDATES {
            return Err(DomainError::DonneeInvalide("chaîne de repli hors borne"));
        }
        let mut uniques = BTreeSet::new();
        for candidat in &self.chaine_repli {
            if candidat.objectif_id != self.objectif_id {
                return Err(DomainError::DonneeInvalide(
                    "candidat rattaché à un autre objectif",
                ));
            }
            if candidat.participant_id.trim().is_empty()
                || candidat.membership_version == 0
                || !participants_declares.contains(&candidat.participant_id)
            {
                return Err(DomainError::DonneeInvalide("candidat non déclaré"));
            }
            if candidat.est_pilote {
                return Err(DomainError::DonneeInvalide("pilote interdit en repli"));
            }
            if !uniques.insert(candidat.participant_id.as_str()) {
                return Err(DomainError::DonneeInvalide("candidat de repli dupliqué"));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatGenerationDelegation {
    Bloquee,
    Ouverte,
    Reassignee,
    Annulee,
    InterventionHumaineRequise,
}

impl EtatGenerationDelegation {
    pub fn est_active(self) -> bool {
        matches!(self, Self::Bloquee | Self::Ouverte)
    }
}

impl GenerationDelegation {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.generation == 0 || self.participant_id.trim().is_empty() {
            return Err(DomainError::DonneeInvalide("génération invalide"));
        }
        if self.generation == 1 && self.generation_precedente.is_some() {
            return Err(DomainError::DonneeInvalide(
                "première génération avec prédécesseur",
            ));
        }
        if self.generation > 1
            && self.generation_precedente != Some(self.generation.saturating_sub(1))
        {
            return Err(DomainError::DonneeInvalide(
                "chaîne de générations discontinue",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerationDelegation {
    pub delegation_id: Uuid,
    pub objectif_id: Uuid,
    pub generation: u64,
    pub participant_id: String,
    pub etat: EtatGenerationDelegation,
    pub generation_precedente: Option<u64>,
    pub trigger_event_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LigneeDelegation {
    pub delegation_id: Uuid,
    pub objectif_id: Uuid,
    pub generation_active: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatEpisodeRelance {
    Actif,
    AnnuleAdministrativement,
    Termine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpisodeRelance {
    pub delegation_id: Uuid,
    pub objectif_id: Uuid,
    pub generation: u64,
    pub request_id: String,
    pub request_ordinal: u8,
    pub reminder_count: u32,
    pub reemissions_used: u8,
    pub etat: EtatEpisodeRelance,
}

impl EpisodeRelance {
    pub fn verifier(&self, max_reemissions: u8) -> Result<(), DomainError> {
        if self.generation == 0
            || self.request_id.trim().is_empty()
            || self.request_ordinal == 0
            || self.reemissions_used > max_reemissions
            || max_reemissions > MAX_REEMISSIONS
        {
            return Err(DomainError::DonneeInvalide("épisode de relance invalide"));
        }
        Ok(())
    }
}

/// Faits fermés qu'un lot F29 peut soumettre au réducteur. Leur ordre dans
/// le lot n'a aucune autorité : une livraison durable est toujours examinée
/// avant les signaux de relance ou d'expiration de la même demande.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeFaitReassignation {
    DeliveryReport,
    ReminderSent,
    Answered,
    TimedOut,
    AnnulationAdministrative,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FaitReassignation {
    pub event_id: String,
    pub request_id: String,
    pub kind: TypeFaitReassignation,
    pub observed_at: i64,
    pub freshness: FraicheurCoordination,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_hash: Option<String>,
}

impl FaitReassignation {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.event_id.trim().is_empty()
            || self.request_id.trim().is_empty()
            || self.observed_at <= 0
        {
            return Err(DomainError::DonneeInvalide("fait F29 incomplet"));
        }
        match self.kind {
            TypeFaitReassignation::DeliveryReport => {
                let Some(hash) = &self.delivery_hash else {
                    return Err(DomainError::DonneeInvalide(
                        "rapport de livraison sans hash",
                    ));
                };
                let valide = hash.len() == 64
                    && hash
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
                if !valide {
                    return Err(DomainError::DonneeInvalide(
                        "hash de livraison F29 invalide",
                    ));
                }
            }
            _ if self.delivery_hash.is_some() => {
                return Err(DomainError::DonneeInvalide(
                    "hash de livraison sur un fait non livraison",
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotReassignation {
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    /// Instant attesté utilisé pour figer les outboxes ; jamais une horloge
    /// consultée par le réducteur.
    pub issued_at: i64,
    /// Échéance fournie par Bridget pour toute demande créée par ce lot.
    pub next_deadline_at: i64,
    pub faits: Vec<FaitReassignation>,
}

impl LotReassignation {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.generation == 0
            || self.issued_at <= 0
            || self.next_deadline_at <= self.issued_at
            || self.faits.is_empty()
        {
            return Err(DomainError::DonneeInvalide("lot F29 incomplet"));
        }
        let mut ids = BTreeSet::new();
        for fait in &self.faits {
            fait.verifier()?;
            if !ids.insert(fait.event_id.as_str()) {
                return Err(DomainError::DonneeInvalide("fait F29 dupliqué"));
            }
        }
        Ok(())
    }

    pub fn event_id_canonique(&self) -> Result<String, DomainError> {
        self.verifier()?;
        let mut ids: Vec<&str> = self
            .faits
            .iter()
            .map(|fait| fait.event_id.as_str())
            .collect();
        ids.sort_unstable();
        Ok(identifiant_deterministe(
            b"lot-reassignation-v1",
            &ids.iter().map(|id| id.as_bytes()).collect::<Vec<_>>(),
        )
        .to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeEffetDemandeSuivie {
    Annuler,
    Creer,
}

/// Effet logique produit sans I/O. Le store en fige ensuite les octets
/// filaires dans la même transaction que la décision et les générations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffetDemandeSuivie {
    pub effect_id: Uuid,
    pub kind: TypeEffetDemandeSuivie,
    pub generation: u64,
    pub request_id: String,
    pub recipient: String,
    pub deadline_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeNotificationReassignation {
    Sortant,
    Successeur,
    InterventionHumaineRequise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationReassignation {
    pub message_id: Uuid,
    pub generation: u64,
    pub recipient: String,
    pub kind: TypeNotificationReassignation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReductionReassignation {
    pub decision: DecisionCoordinationActive,
    pub source: GenerationDelegation,
    pub successeur: Option<GenerationDelegation>,
    pub episode_source: EpisodeRelance,
    pub episode_successeur: Option<EpisodeRelance>,
    pub effets_demandes: Vec<EffetDemandeSuivie>,
    pub notifications: Vec<NotificationReassignation>,
}

/// Réduit F29 à partir du seul snapshot épinglé. La chaîne de repli est
/// parcourue dans son ordre déclaré ; aucune cible proposée par l'appelant,
/// présence Bridget, disponibilité ou profil n'entre dans la décision.
pub fn reduire_reassignation(
    generation: &GenerationDelegation,
    politique: &PolitiqueReassignation,
    episode: &EpisodeRelance,
    generations_connues: &[GenerationDelegation],
    lot: &LotReassignation,
) -> Result<ReductionReassignation, DomainError> {
    generation.verifier()?;
    episode.verifier(politique.max_reemissions)?;
    lot.verifier()?;
    if generation.objectif_id != lot.objectif_id
        || generation.delegation_id != lot.delegation_id
        || generation.generation != lot.generation
        || politique.objectif_id != lot.objectif_id
        || politique.delegation_id != lot.delegation_id
        || episode.objectif_id != lot.objectif_id
        || episode.delegation_id != lot.delegation_id
        || episode.generation != lot.generation
    {
        return Err(DomainError::DonneeInvalide("contexte F29 divergent"));
    }

    let mut faits = lot.faits.clone();
    faits.sort_by(|left, right| {
        priorite_fait_reassignation(left.kind)
            .cmp(&priorite_fait_reassignation(right.kind))
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    let batch_event_id = lot.event_id_canonique()?;
    let decision_id = identifiant_deterministe(
        b"decision-reassignation-v1",
        &[
            lot.objectif_id.as_bytes(),
            lot.delegation_id.as_bytes(),
            &lot.generation.to_be_bytes(),
            batch_event_id.as_bytes(),
            &politique.version.to_be_bytes(),
        ],
    );
    let mut source = generation.clone();
    let mut episode_source = episode.clone();
    let mut effets_demandes = Vec::new();
    let mut notifications = Vec::new();
    let mut successeur = None;
    let mut episode_successeur = None;

    let active_facts: Vec<&FaitReassignation> = faits
        .iter()
        .filter(|fait| fait.request_id == episode.request_id)
        .collect();
    if active_facts
        .iter()
        .any(|fait| fait.freshness != FraicheurCoordination::Fresh)
    {
        return Err(DomainError::DonneeInvalide("observation F29 incomplète"));
    }

    let (kind, motif) = if !generation.etat.est_active()
        || episode.etat != EtatEpisodeRelance::Actif
        || active_facts.is_empty()
    {
        (TypeDecisionCoordinationActive::Aucun, "fait_tardif")
    } else if active_facts
        .iter()
        .any(|fait| fait.kind == TypeFaitReassignation::DeliveryReport)
    {
        episode_source.etat = EtatEpisodeRelance::Termine;
        (TypeDecisionCoordinationActive::Aucun, "livraison_terminale")
    } else if active_facts
        .iter()
        .any(|fait| fait.kind == TypeFaitReassignation::AnnulationAdministrative)
    {
        source.etat = EtatGenerationDelegation::Annulee;
        episode_source.etat = EtatEpisodeRelance::AnnuleAdministrativement;
        effets_demandes.push(effet_annulation(
            lot,
            &batch_event_id,
            &episode.request_id,
            &generation.participant_id,
        ));
        notifications.push(notification_reassignation(
            lot,
            &batch_event_id,
            &generation.participant_id,
            TypeNotificationReassignation::Sortant,
            lot.generation,
        ));
        (
            TypeDecisionCoordinationActive::Aucun,
            "annulation_administrative",
        )
    } else if active_facts
        .iter()
        .any(|fait| fait.kind == TypeFaitReassignation::Answered)
        && episode.reemissions_used < politique.max_reemissions
    {
        episode_source.etat = EtatEpisodeRelance::Termine;
        let next_ordinal = episode
            .request_ordinal
            .checked_add(1)
            .ok_or(DomainError::DonneeInvalide("ordinal F29 hors borne"))?;
        let request_id = identifiant_deterministe(
            b"demande-suivie-reemission-v1",
            &[
                lot.delegation_id.as_bytes(),
                &lot.generation.to_be_bytes(),
                &[next_ordinal],
                batch_event_id.as_bytes(),
            ],
        )
        .to_string();
        effets_demandes.push(effet_creation(
            lot,
            &batch_event_id,
            &request_id,
            &generation.participant_id,
            lot.generation,
        ));
        episode_successeur = Some(EpisodeRelance {
            delegation_id: lot.delegation_id,
            objectif_id: lot.objectif_id,
            generation: lot.generation,
            request_id,
            request_ordinal: next_ordinal,
            reminder_count: 0,
            reemissions_used: episode.reemissions_used + 1,
            etat: EtatEpisodeRelance::Actif,
        });
        (TypeDecisionCoordinationActive::Aucun, "demande_reemise")
    } else {
        let reminder_count = active_facts
            .iter()
            .filter(|fait| fait.kind == TypeFaitReassignation::ReminderSent)
            .count() as u32;
        episode_source.reminder_count =
            episode_source
                .reminder_count
                .checked_add(reminder_count)
                .ok_or(DomainError::DonneeInvalide("compteur F29 hors borne"))?;
        let doit_arbitrer = active_facts
            .iter()
            .any(|fait| fait.kind == TypeFaitReassignation::TimedOut)
            || (active_facts
                .iter()
                .any(|fait| fait.kind == TypeFaitReassignation::Answered)
                && episode.reemissions_used >= politique.max_reemissions)
            || episode_source.reminder_count >= politique.seuil_relances;
        if !doit_arbitrer {
            (TypeDecisionCoordinationActive::Aucun, "seuil_non_atteint")
        } else {
            episode_source.etat = EtatEpisodeRelance::Termine;
            effets_demandes.push(effet_annulation(
                lot,
                &batch_event_id,
                &episode.request_id,
                &generation.participant_id,
            ));
            let deja_consommes: BTreeSet<&str> = generations_connues
                .iter()
                .filter(|known| known.delegation_id == generation.delegation_id)
                .map(|known| known.participant_id.as_str())
                .collect();
            let candidat = politique.chaine_repli.iter().find(|candidate| {
                candidate.objectif_id == lot.objectif_id
                    && !candidate.est_pilote
                    && !deja_consommes.contains(candidate.participant_id.as_str())
            });
            if let Some(candidat) = candidat {
                source.etat = EtatGenerationDelegation::Reassignee;
                let next_generation = generation
                    .generation
                    .checked_add(1)
                    .ok_or(DomainError::DonneeInvalide("génération F29 hors borne"))?;
                let next_request_id = identifiant_deterministe(
                    b"demande-suivie-successeur-v1",
                    &[
                        lot.delegation_id.as_bytes(),
                        &next_generation.to_be_bytes(),
                        batch_event_id.as_bytes(),
                    ],
                )
                .to_string();
                let next = GenerationDelegation {
                    delegation_id: lot.delegation_id,
                    objectif_id: lot.objectif_id,
                    generation: next_generation,
                    participant_id: candidat.participant_id.clone(),
                    etat: EtatGenerationDelegation::Ouverte,
                    generation_precedente: Some(generation.generation),
                    trigger_event_id: Some(batch_event_id.clone()),
                };
                next.verifier()?;
                effets_demandes.push(effet_creation(
                    lot,
                    &batch_event_id,
                    &next_request_id,
                    &candidat.participant_id,
                    next_generation,
                ));
                notifications.push(notification_reassignation(
                    lot,
                    &batch_event_id,
                    &generation.participant_id,
                    TypeNotificationReassignation::Sortant,
                    lot.generation,
                ));
                notifications.push(notification_reassignation(
                    lot,
                    &batch_event_id,
                    &candidat.participant_id,
                    TypeNotificationReassignation::Successeur,
                    next_generation,
                ));
                episode_successeur = Some(EpisodeRelance {
                    delegation_id: lot.delegation_id,
                    objectif_id: lot.objectif_id,
                    generation: next_generation,
                    request_id: next_request_id,
                    request_ordinal: 1,
                    reminder_count: 0,
                    reemissions_used: 0,
                    etat: EtatEpisodeRelance::Actif,
                });
                successeur = Some(next);
                (
                    TypeDecisionCoordinationActive::Reassigner,
                    "successeur_preautorise",
                )
            } else {
                source.etat = EtatGenerationDelegation::InterventionHumaineRequise;
                notifications.push(notification_reassignation(
                    lot,
                    &batch_event_id,
                    &generation.participant_id,
                    TypeNotificationReassignation::InterventionHumaineRequise,
                    lot.generation,
                ));
                (
                    TypeDecisionCoordinationActive::InterventionHumaineRequise,
                    "chaine_preautorisee_epuisee",
                )
            }
        }
    };

    let decision = DecisionCoordinationActive {
        decision_id,
        objectif_id: lot.objectif_id,
        delegation_id: lot.delegation_id,
        generation: lot.generation,
        event_id: batch_event_id,
        policy_version: politique.version,
        kind,
        motif: motif.to_string(),
    };
    decision.verifier()?;
    episode_source.verifier(politique.max_reemissions)?;
    if let Some(next) = &episode_successeur {
        next.verifier(politique.max_reemissions)?;
    }
    Ok(ReductionReassignation {
        decision,
        source,
        successeur,
        episode_source,
        episode_successeur,
        effets_demandes,
        notifications,
    })
}

fn priorite_fait_reassignation(kind: TypeFaitReassignation) -> u8 {
    match kind {
        TypeFaitReassignation::DeliveryReport => 0,
        TypeFaitReassignation::AnnulationAdministrative => 1,
        TypeFaitReassignation::Answered => 2,
        TypeFaitReassignation::TimedOut => 3,
        TypeFaitReassignation::ReminderSent => 4,
    }
}

fn effet_annulation(
    lot: &LotReassignation,
    event_id: &str,
    request_id: &str,
    recipient: &str,
) -> EffetDemandeSuivie {
    EffetDemandeSuivie {
        effect_id: identifiant_deterministe(
            b"effet-annulation-demande-v1",
            &[
                lot.delegation_id.as_bytes(),
                event_id.as_bytes(),
                request_id.as_bytes(),
            ],
        ),
        kind: TypeEffetDemandeSuivie::Annuler,
        generation: lot.generation,
        request_id: request_id.to_string(),
        recipient: recipient.to_string(),
        deadline_at: None,
    }
}

fn effet_creation(
    lot: &LotReassignation,
    event_id: &str,
    request_id: &str,
    recipient: &str,
    generation: u64,
) -> EffetDemandeSuivie {
    EffetDemandeSuivie {
        effect_id: identifiant_deterministe(
            b"effet-creation-demande-v1",
            &[
                lot.delegation_id.as_bytes(),
                event_id.as_bytes(),
                request_id.as_bytes(),
            ],
        ),
        kind: TypeEffetDemandeSuivie::Creer,
        generation,
        request_id: request_id.to_string(),
        recipient: recipient.to_string(),
        deadline_at: Some(lot.next_deadline_at),
    }
}

fn notification_reassignation(
    lot: &LotReassignation,
    event_id: &str,
    recipient: &str,
    kind: TypeNotificationReassignation,
    generation: u64,
) -> NotificationReassignation {
    let kind_bytes = match kind {
        TypeNotificationReassignation::Sortant => b"sortant".as_slice(),
        TypeNotificationReassignation::Successeur => b"successeur".as_slice(),
        TypeNotificationReassignation::InterventionHumaineRequise => {
            b"intervention_humaine_requise".as_slice()
        }
    };
    NotificationReassignation {
        message_id: identifiant_deterministe(
            b"notification-reassignation-v1",
            &[
                lot.delegation_id.as_bytes(),
                event_id.as_bytes(),
                recipient.as_bytes(),
                kind_bytes,
            ],
        ),
        generation,
        recipient: recipient.to_string(),
        kind,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeDecisionCoordinationActive {
    Aucun,
    Notifier,
    Ouvrir,
    Reassigner,
    InterventionHumaineRequise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecisionCoordinationActive {
    pub decision_id: Uuid,
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    pub event_id: String,
    pub policy_version: u64,
    pub kind: TypeDecisionCoordinationActive,
    pub motif: String,
}

impl DecisionCoordinationActive {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.generation == 0
            || self.event_id.trim().is_empty()
            || self.policy_version == 0
            || self.motif.trim().is_empty()
        {
            return Err(DomainError::DonneeInvalide(
                "décision de coordination active invalide",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatNotificationOutbox {
    Prepared,
    OutcomeUnknown,
    Accepted,
    Rejected,
}

impl EtatNotificationOutbox {
    pub fn transition_vers(self, next: Self) -> Result<(), DomainError> {
        let allowed = matches!(
            (self, next),
            (
                Self::Prepared,
                Self::OutcomeUnknown | Self::Accepted | Self::Rejected
            ) | (Self::OutcomeUnknown, Self::Accepted | Self::Rejected)
        );
        if allowed {
            Ok(())
        } else {
            Err(DomainError::TransitionInterdite)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationOutbox {
    pub message_id: Uuid,
    pub idempotency_key: String,
    /// Horodatage canonique du `SendIdempotent`, figé avant toute I/O.
    pub issued_at: i64,
    pub objectif_id: Uuid,
    pub delegation_id: Option<Uuid>,
    pub generation: Option<u64>,
    pub event_id: String,
    pub policy_version: u64,
    pub recipient: String,
    pub message_bytes: Vec<u8>,
    pub etat: EtatNotificationOutbox,
}

impl NotificationOutbox {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.idempotency_key.trim().is_empty()
            || self.issued_at <= 0
            || self.event_id.trim().is_empty()
            || self.policy_version == 0
            || self.recipient.trim().is_empty()
            || self.message_bytes.is_empty()
            || self.generation == Some(0)
        {
            return Err(DomainError::DonneeInvalide("notification outbox invalide"));
        }
        Ok(())
    }

    pub fn transition(&mut self, next: EtatNotificationOutbox) -> Result<(), DomainError> {
        self.etat.transition_vers(next)?;
        self.etat = next;
        Ok(())
    }
}

/// Entrées fermées du réducteur. La variante transport conserve les octets
/// attestés ; la variante d'évaluation ne contient que des faits structurés.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntreeReductionCoordination {
    EvenementAtteste {
        objectif_id: Uuid,
        delegation_id: Uuid,
        evenement: EvenementCoordination,
    },
    ClotureEvaluee(EvaluationCloture),
}

impl EntreeReductionCoordination {
    pub fn objectif_id(&self) -> Uuid {
        match self {
            Self::EvenementAtteste { objectif_id, .. } => *objectif_id,
            Self::ClotureEvaluee(evaluation) => evaluation.objectif_id,
        }
    }

    pub fn delegation_id(&self) -> Uuid {
        match self {
            Self::EvenementAtteste { delegation_id, .. } => *delegation_id,
            Self::ClotureEvaluee(evaluation) => evaluation.delegation_id,
        }
    }

    pub fn generation(&self) -> u64 {
        match self {
            Self::EvenementAtteste { evenement, .. } => evenement.generation(),
            Self::ClotureEvaluee(evaluation) => evaluation.generation,
        }
    }

    pub fn event_id(&self) -> &str {
        match self {
            Self::EvenementAtteste { evenement, .. } => evenement.event_id(),
            Self::ClotureEvaluee(evaluation) => &evaluation.event_id,
        }
    }

    pub fn evenement_atteste(&self) -> Option<&EvenementCoordination> {
        match self {
            Self::EvenementAtteste { evenement, .. } => Some(evenement),
            Self::ClotureEvaluee(_) => None,
        }
    }
}

/// Transition fermée produite par le réducteur. Les prochaines politiques
/// étendront la variante génération sans changer la couture transactionnelle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum TransitionCoordinationActive {
    Aucune,
    Generation(GenerationDelegation),
    ClotureEvaluee(ActeClotureEvaluee),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReductionCoordinationActive {
    pub decision: DecisionCoordinationActive,
    pub transition: TransitionCoordinationActive,
    pub outboxes: Vec<NotificationOutbox>,
}

/// Réducteur pur commun aux politiques 016. Il n'accède qu'aux faits fournis,
/// refuse toute observation incomplète et dérive des identifiants stables de
/// champs structurés — jamais d'un texte rendu.
pub fn reduire_coordination(
    generation: &GenerationDelegation,
    politique: &PolitiqueReassignation,
    entree: &EntreeReductionCoordination,
) -> Result<ReductionCoordinationActive, DomainError> {
    generation.verifier()?;
    if !generation.etat.est_active()
        || generation.objectif_id != entree.objectif_id()
        || generation.delegation_id != entree.delegation_id()
        || generation.generation != entree.generation()
        || politique.objectif_id != generation.objectif_id
        || politique.delegation_id != generation.delegation_id
        || politique.version == 0
    {
        return Err(DomainError::DonneeInvalide(
            "snapshot, génération, politique et événement divergents",
        ));
    }

    let decision_id = identifiant_deterministe(
        b"decision-coordination-v1",
        &[
            generation.objectif_id.as_bytes(),
            generation.delegation_id.as_bytes(),
            &generation.generation.to_be_bytes(),
            entree.event_id().as_bytes(),
            &politique.version.to_be_bytes(),
        ],
    );
    let (kind, motif, transition) = match entree {
        EntreeReductionCoordination::EvenementAtteste { evenement, .. } => {
            if evenement.freshness() != FraicheurCoordination::Fresh {
                return Err(DomainError::DonneeInvalide(
                    "observation de coordination incomplète",
                ));
            }
            if evenement.recipient() != generation.participant_id {
                return Err(DomainError::DonneeInvalide(
                    "destinataire attesté et génération divergents",
                ));
            }
            match evenement.kind() {
                CoordinationEventKind::ReminderSent => (
                    TypeDecisionCoordinationActive::Aucun,
                    "reminder_sent_atteste".to_string(),
                    TransitionCoordinationActive::Aucune,
                ),
            }
        }
        EntreeReductionCoordination::ClotureEvaluee(evaluation) => {
            evaluation.verifier()?;
            let acte = ActeClotureEvaluee {
                acte_id: identifiant_deterministe(
                    b"acte-cloture-evaluee-v1",
                    &[
                        evaluation.objectif_id.as_bytes(),
                        evaluation.delegation_id.as_bytes(),
                        &evaluation.generation.to_be_bytes(),
                        evaluation.event_id.as_bytes(),
                    ],
                ),
                objectif_id: evaluation.objectif_id,
                delegation_id: evaluation.delegation_id,
                generation: evaluation.generation,
                delivery_hash: evaluation.delivery_hash.clone(),
                issue_qualifiante: evaluation.issue,
                evaluated_at: evaluation.evaluated_at,
            };
            acte.verifier()?;
            (
                TypeDecisionCoordinationActive::Aucun,
                "cloture_evaluee_attestee".to_string(),
                TransitionCoordinationActive::ClotureEvaluee(acte),
            )
        }
    };
    let decision = DecisionCoordinationActive {
        decision_id,
        objectif_id: generation.objectif_id,
        delegation_id: generation.delegation_id,
        generation: generation.generation,
        event_id: entree.event_id().to_string(),
        policy_version: politique.version,
        kind,
        motif,
    };
    decision.verifier()?;
    Ok(ReductionCoordinationActive {
        decision,
        transition,
        outboxes: Vec::new(),
    })
}

fn verifier_preuve_livraison(
    generation: u64,
    delivery_hash: &str,
    evaluated_at: i64,
) -> Result<(), DomainError> {
    let hash_valide = delivery_hash.len() == 64
        && delivery_hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if generation == 0 || !hash_valide || evaluated_at <= 0 {
        return Err(DomainError::DonneeInvalide(
            "acte de clôture évaluée invalide",
        ));
    }
    Ok(())
}

pub(crate) fn identifiant_deterministe(namespace: &[u8], fields: &[&[u8]]) -> Uuid {
    let mut digest = Sha256::new();
    digest.update((namespace.len() as u64).to_be_bytes());
    digest.update(namespace);
    for field in fields {
        digest.update((field.len() as u64).to_be_bytes());
        digest.update(field);
    }
    let bytes = digest.finalize();
    let mut uuid = [0_u8; 16];
    uuid.copy_from_slice(&bytes[..16]);
    uuid[6] = (uuid[6] & 0x0f) | 0x50;
    uuid[8] = (uuid[8] & 0x3f) | 0x80;
    Uuid::from_bytes(uuid)
}

/// Définition immuable enregistrée avant toute I/O. Le store complète chaque
/// politique avec une lignée et une génération initiale dérivées des faits
/// déjà présents dans le registre de l'objectif.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionCoordination {
    pub objectif_id: Uuid,
    pub dependencies: Vec<DependanceDelegation>,
    pub policies: Vec<PolitiqueReassignation>,
    pub attentes: Vec<AttenteNotification>,
}

impl DefinitionCoordination {
    pub fn verifier_bornes(&self) -> Result<(), DomainError> {
        if self.dependencies.len() > MAX_COORDINATION_EDGES {
            return Err(DomainError::DonneeInvalide(
                "graphe de coordination hors borne",
            ));
        }
        let mut edges = BTreeSet::new();
        let mut nodes = BTreeSet::new();
        for dependency in &self.dependencies {
            dependency.verifier()?;
            if dependency.objectif_id != self.objectif_id {
                return Err(DomainError::DonneeInvalide(
                    "dépendance rattachée à un autre objectif",
                ));
            }
            if !edges.insert((dependency.prerequis_id, dependency.dependant_id)) {
                return Err(DomainError::DonneeInvalide("dépendance dupliquée"));
            }
            nodes.insert(dependency.prerequis_id);
            nodes.insert(dependency.dependant_id);
        }
        let mut policy_ids = BTreeSet::new();
        for policy in &self.policies {
            if policy.objectif_id != self.objectif_id {
                return Err(DomainError::DonneeInvalide(
                    "politique rattachée à un autre objectif",
                ));
            }
            if !policy_ids.insert(policy.delegation_id) {
                return Err(DomainError::DonneeInvalide("politique dupliquée"));
            }
            nodes.insert(policy.delegation_id);
        }
        if nodes.len() > MAX_COORDINATION_NODES {
            return Err(DomainError::DonneeInvalide(
                "graphe de coordination hors borne",
            ));
        }
        let mut expectation_ids = BTreeSet::new();
        let mut expectation_keys = BTreeSet::new();
        for expectation in &self.attentes {
            expectation.verifier()?;
            if expectation.objectif_id != self.objectif_id {
                return Err(DomainError::DonneeInvalide(
                    "attente rattachée à un autre objectif",
                ));
            }
            if !expectation_ids.insert(expectation.attente_id) {
                return Err(DomainError::DonneeInvalide("attente dupliquée"));
            }
            let key = (
                expectation.delegation_id,
                format!("{:?}", expectation.kind),
                expectation.recipient.as_str(),
                expectation.policy_version,
            );
            if !expectation_keys.insert(key) {
                return Err(DomainError::DonneeInvalide(
                    "attente sémantiquement dupliquée",
                ));
            }
        }
        Ok(())
    }
}

/// Opérations métier fermées acceptées par le guichet Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationGuichet {
    DeliveryReport,
    MissionStatus,
    DeadlineQuestion,
}

/// État terminal attesté par Bridget. Il reste un fait de transport et ne
/// constitue jamais, à lui seul, une décision de coordination Maicie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatRequeteGuichet {
    Answered,
    Cancelled,
    TimedOut,
}

/// Issue métier locale d'une greffe. Le nom français évite de confondre
/// `DemandeDejaTerminale` avec le refus de transport homonyme de Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueGreffe {
    Accepted,
    DemandeDejaTerminale,
    Refusee,
}

/// Motif fermé d'un refus de relève. Il ne représente jamais une corruption
/// SQLite ou une panne de transport : ces deux familles restent des erreurs
/// techniques, sans reçu métier fabriqué.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MotifRefusGreffe {
    DelegationAbsente,
    RelationsInvalides,
    EnveloppeDivergente,
}

/// Preuve locale durable qu'une requête structurée a été traitée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceptionGreffe {
    pub issuer_scope: String,
    pub request_id: String,
    pub operation: OperationGuichet,
    pub canonical_request_bytes: Vec<u8>,
    pub objective_id: Option<Uuid>,
    pub delegation_id: Option<Uuid>,
    pub delivery_hash: Option<String>,
    pub response_message_id: String,
    pub claim_generation: u64,
    pub claim_token: String,
    pub reply_bytes: Vec<u8>,
    pub issue: IssueGreffe,
    pub decision_id: Option<Uuid>,
    pub processed_at: i64,
}

/// Jointure durable des deux canaux décrivant une même livraison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecuCorrelation {
    pub issuer_scope: String,
    pub request_id: String,
    pub in_reply_to: String,
    pub response_message_id: String,
    pub lifecycle_event_id: Option<String>,
    pub lifecycle_state: Option<EtatRequeteGuichet>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delegation {
    pub id: Uuid,
    pub objectif_id: Uuid,
    /// Identifiant exact du constat ayant motivé cette délégation.
    ///
    /// Le champ est absent des délégations ordinaires et des données
    /// historiques. Lorsqu'il est présent, il naît avec la délégation : il ne
    /// peut jamais être ajouté a posteriori ni reconstruit depuis un texte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constat_id: Option<String>,
    pub participant: String,
    pub instruction: String,
    pub duree: ClasseDuree,
    pub etat: EtatDelegation,
    pub raison: String,
}

impl Delegation {
    pub fn nouvelle(
        objectif_id: Uuid,
        participant: impl Into<String>,
        instruction: impl Into<String>,
        duree: ClasseDuree,
        raison: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let participant = participant.into();
        let instruction = instruction.into();
        let raison = raison.into();
        if participant.trim().is_empty()
            || instruction.trim().is_empty()
            || raison.trim().is_empty()
        {
            return Err(DomainError::DonneeInvalide("délégation incomplète"));
        }
        Ok(Self {
            id: Uuid::new_v4(),
            objectif_id,
            constat_id: None,
            participant,
            instruction,
            duree,
            etat: EtatDelegation::Creee,
            raison,
        })
    }

    /// Lie la délégation en cours de construction à un constat exact.
    ///
    /// Cette méthode consomme `self` afin que l'appelant construise le fait
    /// avant la transaction de création. Le store n'expose aucune primitive
    /// permettant de greffer ce lien sur une délégation déjà persistée.
    pub fn pour_constat(mut self, constat_id: impl Into<String>) -> Result<Self, DomainError> {
        let constat_id = constat_id.into();
        validate_constat_id(&constat_id)?;
        if self.constat_id.is_some() {
            return Err(DomainError::TransitionInterdite);
        }
        self.constat_id = Some(constat_id);
        Ok(self)
    }

    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.participant.trim().is_empty()
            || self.instruction.trim().is_empty()
            || self.raison.trim().is_empty()
        {
            return Err(DomainError::DonneeInvalide("délégation incomplète"));
        }
        if let Some(constat_id) = &self.constat_id {
            validate_constat_id(constat_id)?;
        }
        Ok(())
    }

    /// Projette le fait durable sans inventer de lien pour une délégation
    /// historique ou ordinaire.
    pub fn lien_arbitrage(&self) -> Result<Option<LienArbitrage>, DomainError> {
        self.verifier()?;
        Ok(self.constat_id.as_ref().map(|constat_id| LienArbitrage {
            constat_id: constat_id.clone(),
            objectif_id: self.objectif_id,
            delegation_id: self.id,
        }))
    }

    pub fn transition(&mut self, next: EtatDelegation) -> Result<(), DomainError> {
        if !matches!(
            (self.etat, next),
            (EtatDelegation::Creee, EtatDelegation::AEvaluer)
                | (EtatDelegation::AEvaluer, EtatDelegation::Terminee)
        ) {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat = next;
        Ok(())
    }

    pub fn annuler(&mut self) -> Result<(), DomainError> {
        if matches!(
            self.etat,
            EtatDelegation::Terminee | EtatDelegation::Annulee
        ) {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat = EtatDelegation::Annulee;
        Ok(())
    }
}

/// Fait durable reliant un constat au seul objectif porté par sa délégation.
///
/// Il s'agit d'une projection du document canonique [`Delegation`], jamais
/// d'une deuxième source de vérité ni d'une corrélation calculée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LienArbitrage {
    pub constat_id: String,
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
}

fn validate_constat_id(constat_id: &str) -> Result<(), DomainError> {
    if constat_id.is_empty()
        || constat_id.trim() != constat_id
        || constat_id.chars().any(char::is_control)
    {
        return Err(DomainError::DonneeInvalide("constat_id invalide"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatOutboxDelegation {
    Prepared,
    OutcomeUnknown,
    Accepted,
    Rejected,
}

impl EtatOutboxDelegation {
    /// Autorité unique des transitions persistées de l'outbox de délégation.
    /// Le store doit valider cette transition avant toute mise à jour SQLite.
    pub fn transition_vers(self, next: Self) -> Result<(), DomainError> {
        if !matches!(
            (self, next),
            (Self::Prepared, Self::OutcomeUnknown)
                | (Self::Prepared, Self::Accepted)
                | (Self::Prepared, Self::Rejected)
                | (Self::OutcomeUnknown, Self::Accepted)
                | (Self::OutcomeUnknown, Self::Rejected)
        ) {
            return Err(DomainError::TransitionInterdite);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxDelegation {
    pub message_id: Uuid,
    pub delegation_id: Uuid,
    pub target: String,
    pub body_bytes: Vec<u8>,
    pub reply: bool,
    pub timeout_secs: u64,
    pub deadline_contractuelle: i64,
    pub body_hash: Vec<u8>,
    pub etat: EtatOutboxDelegation,
    pub attempted_at: Option<i64>,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

impl OutboxDelegation {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.target.trim().is_empty() || self.body_bytes.is_empty() || self.body_hash.is_empty()
        {
            return Err(DomainError::DonneeInvalide(
                "enveloppe de délégation incomplète",
            ));
        }
        if self.retry_until > self.dedup_retained_until {
            return Err(DomainError::DonneeInvalide("retry hors horizon Bridget"));
        }
        Ok(())
    }

    pub fn transition(&mut self, next: EtatOutboxDelegation, now: i64) -> Result<(), DomainError> {
        self.etat.transition_vers(next)?;
        self.etat = next;
        self.attempted_at = Some(now);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSnapshot {
    Bridget,
    AcpSubscription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatFlux {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotTransport {
    pub message_id: Uuid,
    pub request_state: Option<String>,
    pub observed_at: i64,
    pub source: SourceSnapshot,
    pub subscription_id: Option<String>,
    pub seq: Option<u64>,
    pub stream_state: EtatFlux,
}

impl SnapshotTransport {
    pub fn verifier(&self) -> Result<(), DomainError> {
        match self.source {
            SourceSnapshot::Bridget if self.subscription_id.is_none() && self.seq.is_none() => {
                Ok(())
            }
            SourceSnapshot::AcpSubscription
                if self.subscription_id.is_some() && self.seq.is_some() =>
            {
                Ok(())
            }
            _ => Err(DomainError::DonneeInvalide(
                "corrélation de snapshot invalide",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatActivationProfil {
    Inactif,
    Proposition,
    Approuve,
    Lance,
    Connecte,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilEquipe {
    pub id: String,
    pub nom_affiche: String,
    pub capacites: Vec<String>,
    pub personnalite: String,
    pub outils: Vec<String>,
    pub spawn_order_ref: String,
    pub etat_activation: EtatActivationProfil,
}

impl ProfilEquipe {
    pub fn transition_activation(&mut self, next: EtatActivationProfil) -> Result<(), DomainError> {
        if !matches!(
            (self.etat_activation, next),
            (
                EtatActivationProfil::Inactif,
                EtatActivationProfil::Proposition
            ) | (
                EtatActivationProfil::Proposition,
                EtatActivationProfil::Approuve
            ) | (EtatActivationProfil::Approuve, EtatActivationProfil::Lance)
                | (EtatActivationProfil::Lance, EtatActivationProfil::Connecte)
        ) {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat_activation = next;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprobationActivation {
    pub id: Uuid,
    /// Généré à la proposition : tout replay SpawnOrder conserve cette clé.
    pub command_id: Uuid,
    pub objective_id: Uuid,
    pub profile_id: String,
    pub profile_hash: Vec<u8>,
    pub context_hash: Vec<u8>,
    pub context_scope: String,
    pub parameters: String,
    pub actor: String,
    pub expires_at: i64,
    pub consumed_at: Option<i64>,
}

impl ApprobationActivation {
    pub fn verifier_pour_dispatch(
        &self,
        now: i64,
        profile_hash: &[u8],
        context_hash: &[u8],
    ) -> Result<(), DomainError> {
        if self.actor != "local_human" {
            return Err(DomainError::DonneeInvalide("acteur d'approbation invalide"));
        }
        if self.consumed_at.is_some() {
            return Err(DomainError::ApprobationConsommee);
        }
        if now >= self.expires_at {
            return Err(DomainError::ApprobationExpiree);
        }
        if self.profile_hash != profile_hash || self.context_hash != context_hash {
            return Err(DomainError::ApprobationIncoherente);
        }
        Ok(())
    }

    pub fn consommer(
        &mut self,
        now: i64,
        profile_hash: &[u8],
        context_hash: &[u8],
    ) -> Result<(), DomainError> {
        self.verifier_pour_dispatch(now, profile_hash, context_hash)?;
        self.consumed_at = Some(now);
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatActivationOutbox {
    Dispatching,
    OutcomeUnknown,
    Applied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivationOutbox {
    pub command_id: Uuid,
    pub approval_id: Uuid,
    pub spawn_order_bytes: Vec<u8>,
    pub etat: EtatActivationOutbox,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

impl ActivationOutbox {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.spawn_order_bytes.is_empty() || self.retry_until > self.dedup_retained_until {
            return Err(DomainError::DonneeInvalide("activation outbox invalide"));
        }
        Ok(())
    }

    pub fn transition(&mut self, next: EtatActivationOutbox) -> Result<(), DomainError> {
        if !matches!(
            (self.etat, next),
            (
                EtatActivationOutbox::Dispatching,
                EtatActivationOutbox::OutcomeUnknown
            ) | (
                EtatActivationOutbox::Dispatching,
                EtatActivationOutbox::Applied
            ) | (
                EtatActivationOutbox::OutcomeUnknown,
                EtatActivationOutbox::Applied
            )
        ) {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat = next;
        Ok(())
    }
}
