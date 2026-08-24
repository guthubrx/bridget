//! Entités de coordination Maicie et leurs transitions explicites.
//!
//! Les états de ce module décrivent uniquement la coordination. Les faits de
//! présence, livraison et délai restent détenus par Bridget.

use serde::{Deserialize, Serialize};
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

/// Types de faits structurés consommables par la coordination active.
/// `Answered`, `Cancelled` et `TimedOut` restent des faits de cycle Bridget :
/// ils ne qualifient jamais une arête du DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeEvenementCoordination {
    ClotureObjectif,
    OuvertureDelegation,
    DeliveryReport,
    ReminderSent,
    Answered,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FraicheurCoordination {
    Fresh,
    Gap,
    Ended,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvenementCoordination {
    pub event_id: String,
    pub objectif_id: Uuid,
    pub delegation_id: Option<Uuid>,
    pub generation: Option<u64>,
    pub kind: TypeEvenementCoordination,
    pub observed_at: i64,
    pub freshness: FraicheurCoordination,
}

impl EvenementCoordination {
    pub fn verifier(&self) -> Result<(), DomainError> {
        if self.event_id.trim().is_empty() || self.observed_at <= 0 {
            return Err(DomainError::DonneeInvalide(
                "événement de coordination incomplet",
            ));
        }
        if self.generation == Some(0) {
            return Err(DomainError::DonneeInvalide("génération nulle"));
        }
        Ok(())
    }

    /// Une arête n'est qualifiée que par les deux preuves durables prévues par
    /// FR-1603. Les événements de cycle 015 ne sont jamais promus implicitement.
    pub fn peut_qualifier_une_arete(&self) -> bool {
        self.freshness == FraicheurCoordination::Fresh
            && matches!(self.kind, TypeEvenementCoordination::DeliveryReport)
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

/// Preuve évaluée créée en amont de la session 016. Le registre 016 la
/// conserve et la consomme, mais ne la fabrique jamais depuis un texte ou un
/// événement de cycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActeClotureEvaluee {
    pub acte_id: Uuid,
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    pub delivery_hash: String,
    pub issue_qualifiante: String,
    pub evaluated_at: i64,
}

impl ActeClotureEvaluee {
    pub fn verifier(&self) -> Result<(), DomainError> {
        let hash_valide = self.delivery_hash.len() == 64
            && self
                .delivery_hash
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit());
        if self.generation == 0
            || !hash_valide
            || self.issue_qualifiante.trim().is_empty()
            || self.evaluated_at <= 0
        {
            return Err(DomainError::DonneeInvalide(
                "acte de clôture évaluée invalide",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttenteNotification {
    pub attente_id: Uuid,
    pub objectif_id: Uuid,
    pub delegation_id: Option<Uuid>,
    pub kind: TypeEvenementCoordination,
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
        if !matches!(
            self.kind,
            TypeEvenementCoordination::ClotureObjectif
                | TypeEvenementCoordination::OuvertureDelegation
        ) {
            return Err(DomainError::DonneeInvalide(
                "type de notification non supporté en v1",
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationOutbox {
    pub message_id: Uuid,
    pub idempotency_key: String,
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
        let allowed = matches!(
            (self.etat, next),
            (
                EtatNotificationOutbox::Prepared,
                EtatNotificationOutbox::OutcomeUnknown
            ) | (
                EtatNotificationOutbox::Prepared,
                EtatNotificationOutbox::Accepted
            ) | (
                EtatNotificationOutbox::Prepared,
                EtatNotificationOutbox::Rejected
            ) | (
                EtatNotificationOutbox::OutcomeUnknown,
                EtatNotificationOutbox::Accepted
            ) | (
                EtatNotificationOutbox::OutcomeUnknown,
                EtatNotificationOutbox::Rejected
            )
        );
        if !allowed {
            return Err(DomainError::TransitionInterdite);
        }
        self.etat = next;
        Ok(())
    }
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
