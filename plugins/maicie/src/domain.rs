//! Entités de coordination Maicie et leurs transitions explicites.
//!
//! Les états de ce module décrivent uniquement la coordination. Les faits de
//! présence, livraison et délai restent détenus par Bridget.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            participant,
            instruction,
            duree,
            etat: EtatDelegation::Creee,
            raison,
        })
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
