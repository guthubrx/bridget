//! Persistance SQLite privée de Maicie.
//!
//! La décision locale, la délégation et son outbox sont écrites dans une
//! transaction unique. Aucune méthode de reprise ne reconstruit l'enveloppe :
//! les octets préparés avant I/O sont l'autorité.

use crate::app::ConversationRecord;
use crate::bridget_client::{GuichetClaim, IdempotencyIssue, PublicMessage, SpawnOutcome};
use crate::control::{Admission, AutonomousEffect, ControlSnapshot, admit_autonomous_effect};
use crate::domain::guichet::{
    EvenementCycleGuichet, MutationReply, ProjectionReply, RapportLivraison, RequeteCanonique,
    delivery_reply_bytes, mutation_reply_bytes, projection_reply_bytes,
    reclaim_mutation_reply_bytes, reclaim_projection_reply_bytes, refusal_reply_bytes,
};
use crate::domain::{
    ActivationOutbox, ApprobationActivation, AttenteNotification, AttestationConsumption,
    ClasseDuree, CoutMissionAgent, DEPENDENCY_POLICY_VERSION, DecisionCoordination,
    DecisionCoordinationActive, DefinitionCoordination, Delegation, DependanceDelegation,
    DomainError, EffetDemandeSuivie, EntreeReductionCoordination, EpisodeRelance,
    EtatActivationOutbox, EtatDecision, EtatDelegation, EtatEpisodeRelance,
    EtatGenerationDelegation, EtatNotificationOutbox, EtatObjectif, EtatOutboxDelegation,
    EtatRequeteGuichet, EvenementCoordination, ExecutionProjection, FaitReassignation,
    FraicheurCoordination, GenerationDelegation, IssueGreffe, LienArbitrage, LigneeDelegation,
    LotReassignation, MotifRefusDelegationLocale, MotifRefusGreffe, NotificationOutbox,
    NotificationReassignation, ObjectifCoordonne, ObjectiveOpeningPermit, ObjectiveOrigin,
    OperationGuichet, OutboxDelegation, PolitiqueReassignation, ProjectIdentity,
    ProjectIdentityStatus, QualificationDependance, ReceptionGreffe, RecuCorrelation,
    ReductionCoordinationActive, ReductionOuvertureDelegation, ReductionReassignation,
    SuiteObjective, TransitionCoordinationActive, TypeDecision, TypeEffetDemandeSuivie,
    TypeEvenementAttendu, TypeFaitReassignation, TypeNotificationReassignation,
    identifiant_deterministe, reduire_coordination, reduire_ouverture_dependance,
    reduire_reassignation,
};
use crate::domain::{ProjectProfile, ProjectProfileApproval, ProjectProfileStatus};
use crate::outbox::{
    MAX_MESSAGE_BYTES, OutboxError, PendingDelegationOutbox, PreparedDelegation, RecoverySnapshot,
    StoreCommitPhase, stable_body_hash,
};
use crate::review_continuity::StoredReviewVerdict;
use crate::routines::{EtatOccurrence, EtatRoutine, Routine, RoutineOccurrence};
use bridget_transport::protocol::{
    CoordinationEventKind, GuichetOutcome, GuichetReplyPayload, ProjectBackend, ProjectBindOutcome,
    ProjectBindStatus, ResolvedProjectProfile, WrapperToDaemon, decode,
};
use rusqlite::{
    Connection, ErrorCode, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, DirBuilder, OpenOptions};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Version courante du schéma SQLite. Les oracles de migration doivent lire
/// cette constante — un littéral en dur meurt à chaque migration.
pub const SCHEMA_VERSION: i64 = 24;
const DATABASE_MODE: u32 = 0o600;
const DIRECTORY_MODE: u32 = 0o700;
type StoredActivationOutcome = (
    Vec<u8>,
    String,
    i64,
    Option<Vec<u8>>,
    String,
    String,
    String,
);
type StoredActivationApproval = (
    Vec<u8>,
    Vec<u8>,
    String,
    String,
    String,
    String,
    String,
    String,
);
type StoredActivationProposal = (Vec<u8>, Vec<u8>, String, String);
type StoredDelegationAggregates = (String, String, String, Vec<u8>, String, Vec<u8>);
type StoredCoordinationContext = (String, i64, String, String, Vec<u8>, i64, Vec<u8>);

/// Autorité d'écriture unique de l'état Maicie.
pub struct MaicieStore {
    path: PathBuf,
    connection: Connection,
    issuer_scope: String,
    /// Instantané de l'état de contrôle du référent, posé à la relève et
    /// jamais persisté (SPEC-087). Lu par tous les puits d'effet autonome.
    control: ControlSnapshot,
}

/// Résultat du contrôle de compatibilité qui précède l'activation d'un
/// binaire. Cette lecture ne crée ni base, ni table, ni sidecar volontaire :
/// elle dit seulement si le schéma autorise l'ouverture métier du greffe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaPreflight {
    pub database_version: Option<i64>,
    pub supported_version: i64,
    pub bootstrap_required: bool,
}

/// État durable de la commande d'enregistrement projet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectRegistrationState {
    Prepared,
    Binding,
    Bound,
    Failed,
    Expired,
}

impl ProjectRegistrationState {
    fn from_db(value: &str) -> Result<Self, StoreError> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "binding" => Ok(Self::Binding),
            "bound" => Ok(Self::Bound),
            "failed" => Ok(Self::Failed),
            "expired" => Ok(Self::Expired),
            _ => Err(StoreError::Corrupt("état enregistrement projet inconnu")),
        }
    }
}

/// Intention préparée avant tout appel à Bridget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRegistrationIntent {
    pub command_id: String,
    pub proposed_project_id: String,
    pub display_name: String,
    pub requested_root: String,
    pub canonical_payload: Vec<u8>,
    pub created_at: i64,
    pub retry_until: i64,
}

/// Vue durable et rejouable de la préparation d'une commande projet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRegistrationRecord {
    pub command_id: String,
    pub identity: ProjectIdentity,
    pub requested_root: String,
    pub state: ProjectRegistrationState,
    pub resolved_project_id: Option<String>,
    pub retry_until: i64,
    pub outbox_pending: bool,
    pub outcome: Option<ProjectBindOutcome>,
}

/// Paramètres figés pour créer l'outbox au déblocage F37 (aucune intention
/// d'envoi tant que la délégation reste en attente).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeferredDispatchParams {
    pub reply: bool,
    pub timeout_secs: u64,
    pub retry_until: i64,
    pub dedup_retained_until: i64,
    pub max_frame_bytes: usize,
    pub deadline_contractuelle: i64,
    pub issuer_scope: String,
}

/// Projection fermée des refus locaux de contrainte. Chaque champ correspond
/// à un variant métier ; aucun motif libre ne peut apparaître dans la vue.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CompteursRefusDelegationLocale {
    pub suite_aucune_avec_citation_non_classee: u64,
}

/// Résultat durable d'une commande `delegate` idempotente. Les identifiants
/// sont conservés séparément de l'outbox afin qu'un rejeu local ne dépende pas
/// de la disponibilité du transport Bridget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredDelegateResult {
    pub objective_id: Uuid,
    pub delegation_id: Uuid,
    /// Absent tant que la délégation est `EnAttentePrerequis` (aucune outbox).
    pub message_id: Option<Uuid>,
    pub participant: String,
    pub duration: ClasseDuree,
    pub timeout_secs: u64,
    pub deadline_contractuelle: i64,
}

/// Issue de la réservation atomique d'une commande `delegate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegateReservation {
    Created,
    Replay(StoredDelegateResult),
}

/// Vue corrélée d'un objectif privée de toute interprétation du transport.
/// Les décisions y figurent afin que les commandes explicites restent
/// auditables même lorsqu'elles ne créent aucune nouvelle délégation.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectiveSnapshot {
    pub objective: ObjectifCoordonne,
    pub delegations: Vec<crate::domain::Delegation>,
    pub decisions: Vec<DecisionCoordination>,
    /// Registre local de remise, distinct de toute observation ACP live.
    pub remises_locales: Vec<RemiseLocale>,
    /// Coûts portés à la clôture. Vide tant que l'objectif n'est pas clos.
    pub costs: Vec<CoutMissionAgent>,
}

/// Définition 016 et faits initiaux relus depuis le registre, sans aucune
/// consultation de la configuration courante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCoordinationSnapshot {
    pub definition: DefinitionCoordination,
    pub lineages: Vec<LigneeDelegation>,
    pub generations: Vec<GenerationDelegation>,
}

/// Résultat durable du réducteur commun. `replayed` distingue une première
/// application d'un rejeu strict du même événement sans modifier son issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCoordinationReduction {
    pub reduction: ReductionCoordinationActive,
    pub replayed: bool,
}

/// Effet filaire F29 figé avant toute I/O. `message_bytes` est l'autorité de
/// reprise, qu'il s'agisse d'un `CancelRequest` ou d'un message suivi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedRequestOutbox {
    pub effect_id: Uuid,
    pub issued_at: i64,
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    pub event_id: String,
    pub policy_version: u64,
    pub kind: TypeEffetDemandeSuivie,
    pub request_id: String,
    pub recipient: String,
    pub message_bytes: Vec<u8>,
    pub etat: EtatNotificationOutbox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredReassignmentReduction {
    pub reduction: ReductionReassignation,
    pub replayed: bool,
}

/// Corrélation locale d'une demande suivie. Elle provient uniquement des
/// outboxes/épisodes durables Maicie et ne déduit jamais une délégation d'un
/// texte transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReassignmentRequestContext {
    pub objectif_id: Uuid,
    pub delegation_id: Uuid,
    pub generation: u64,
    pub participant: String,
    pub timeout_secs: u64,
}

impl StoredCoordinationSnapshot {
    pub fn dependants_by_prerequisite(&self, prerequisite_id: Uuid) -> Vec<Uuid> {
        self.definition
            .dependencies
            .iter()
            .filter(|edge| edge.prerequis_id == prerequisite_id)
            .map(|edge| edge.dependant_id)
            .collect()
    }
}

/// Projection honnête de l'outbox durable pour `maicie status`.
///
/// Elle atteste ce que Maicie a persisté après un échange Bridget ; elle ne
/// remplace ni la fraîcheur, ni les événements du futur abonnement ACP T018.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RemiseLocale {
    pub delegation_id: Uuid,
    pub message_id: Uuid,
    pub state: EtatOutboxDelegation,
    pub issue: Option<Value>,
    pub observed_at: Option<i64>,
}

/// Faits locaux nécessaires aux projections du guichet. Ils sont lus depuis
/// une seule jointure SQLite ; aucune observation Bridget n'est reconstruite.
#[derive(Debug, Clone, PartialEq)]
pub struct GuichetProjectionFacts {
    pub objective: ObjectifCoordonne,
    pub delegation: Delegation,
    pub local_delivery: RemiseLocale,
    pub deadline_at: i64,
}

/// Motif local fermé quand les octets durables ne peuvent jamais produire une
/// remise valide. Il ne crée aucun nouvel état de domaine : l'outbox converge
/// vers `Rejected`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalFailureReason {
    FrameTooLarge,
    InvalidEnvelope,
}

impl LocalFailureReason {
    fn as_str(self) -> &'static str {
        match self {
            Self::FrameTooLarge => "frame_too_large",
            Self::InvalidEnvelope => "invalid_envelope",
        }
    }
}

/// Lecture de reprise par entrée : une corruption corrélable ne rend jamais
/// les autres outboxes indisponibles et peut être terminalisée sans rejeu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationRecoveryEntry {
    Pending(PendingDelegationOutbox),
    LocalFailure {
        objective_id: Uuid,
        delegation_id: Uuid,
        message_id: Uuid,
        reason: LocalFailureReason,
    },
}

/// Ligne de reprise d'un SpawnOrder. Les octets sont ceux validés lors de
/// l'approbation ; le store ne les désérialise ni ne les reconstruit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingActivationOutbox {
    pub activation: ActivationOutbox,
    pub approval: ApprobationActivation,
}

/// Réservation active d'une plage de ressource (politique 31).
/// La comparaison de `resource` est exacte : zéro intelligence sur les noms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRangeReservation {
    pub resource: String,
    pub objective_id: Uuid,
    pub reserved_at: i64,
}

/// Paramètres revalidés au moment du dispatch. Ils restent séparés de
/// l'approbation persistée : les hashes viennent de l'état courant, tandis que
/// les octets du SpawnOrder deviennent immuables seulement après validation.
pub struct ActivationApprovalRequest<'a> {
    pub now: i64,
    pub profile_hash: &'a [u8],
    pub context_hash: &'a [u8],
    pub spawn_order_bytes: &'a [u8],
    pub retry_until: i64,
    pub dedup_retained_until: i64,
}

/// Résultat durable d'une greffe. `reply_bytes` est l'autorité de reprise :
/// aucun appelant ne doit reconstruire la réponse depuis les autres champs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredGuichetReply {
    pub reception: ReceptionGreffe,
    pub correlation: Option<RecuCorrelation>,
    pub replayed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuichetLifecycleResult {
    Recorded,
    Replayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuichetCommitPhase {
    AfterDecisionInsert,
    AfterDependentDecision,
    AfterDependentTransition,
    AfterDependentOutbox,
    BeforeCommit,
    AfterCommit,
}

/// Frontières discriminantes de la transaction du réducteur 016.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinationCommitPhase {
    AfterEventInsert,
    AfterDecisionInsert,
    AfterTransition,
    AfterOutboxes,
    AfterDependentDecision,
    AfterDependentTransition,
    AfterDependentOutbox,
    /// Frontières F29 exposées aussi par le commit combiné T1610. Elles
    /// empêchent le point d'entrée de production de masquer une écriture
    /// partielle derrière son propre observateur de coordination.
    AfterDecision,
    AfterGenerations,
    AfterRequestOutboxes,
    AfterNotifications,
    BeforeCommit,
    AfterCommit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DependencyOpeningCommitPhase {
    Decision,
    Generation,
    Notification,
}

/// Frontières de la transaction F29. Elles permettent de prouver qu'aucune
/// génération, demande ou notification n'échappe isolément au rollback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReassignmentCommitPhase {
    AfterEvents,
    AfterDecision,
    AfterGenerations,
    AfterRequestOutboxes,
    AfterNotifications,
    BeforeCommit,
    AfterCommit,
}

/// Frontières discriminantes de l'unique transaction de clôture FR-1602.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveClosureCommitPhase {
    AfterObjectiveUpdate,
    AfterDecisionInsert,
    AfterOutboxes,
    BeforeCommit,
    AfterCommit,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovedSpawnOrder {
    #[serde(rename = "type")]
    kind: String,
    agent_type: String,
    agent_id: Option<String>,
    cwd: String,
    persistent: bool,
    command_id: String,
    issued_at: i64,
    deadline_at: i64,
}

impl MaicieStore {
    /// Vérifie sans écriture qu'un binaire peut ouvrir le greffe configuré.
    ///
    /// Une base absente ou SQLite vide est un bootstrap compatible. Une base
    /// antérieure ou postérieure au binaire rend la même erreur que l'ouverture
    /// métier, mais avant toute réconciliation ou attribution de commande.
    pub fn schema_preflight(path: impl AsRef<Path>) -> Result<SchemaPreflight, StoreError> {
        let path = path.as_ref();
        validate_database_path(path)?;
        match fs::symlink_metadata(path) {
            Ok(metadata) => {
                validate_private_parent(path)?;
                validate_private_database_metadata(&metadata)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                validate_private_parent_if_present(path)?;
            }
            Err(error) => return Err(StoreError::Io(error)),
        }

        let snapshot = snapshot_database_for_preflight(path)?;
        let database_version = if snapshot.database.exists() {
            let connection = Connection::open(&snapshot.database).map_err(StoreError::Sql)?;
            Some(
                connection
                    .query_row("PRAGMA user_version", [], |row| row.get(0))
                    .map_err(StoreError::Sql)?,
            )
        } else {
            None
        };

        // C'est volontairement l'ouverture métier complète : migrations sans
        // consentement, passage WAL et identité greffière sont exercés sur la
        // copie jetable. Un numéro de version exact ne suffit pas à prouver que
        // le DDL attendu par ce binaire existe réellement.
        drop(Self::open(&snapshot.database)?);
        Ok(SchemaPreflight {
            database_version,
            supported_version: SCHEMA_VERSION,
            bootstrap_required: database_version.unwrap_or(0) == 0,
        })
    }

    /// Ouvre la base privée sans migrer un schéma déjà versionné.
    ///
    /// Une base neuve (`user_version = 0` et aucun objet utilisateur dans
    /// `sqlite_master`) est bootstrappée : créer n'est pas migrer. Une base
    /// peuplée, même avec `user_version` remis à 0, ou dont le schéma est
    /// antérieur au binaire, est refusée tant que l'appelant n'a pas consenti
    /// via [`crate::install_publish::PublishedMigration::open`].
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::open_with_migration_consent(path, false)
    }

    /// Primitive brute réservée au module qui possède la preuve consommable
    /// de publication. Le crate binaire ne peut pas l'appeler.
    pub(crate) fn open_after_publication(
        publication: &crate::install_publish::PublishedMigration,
        _exclusion: &crate::install_publish::MigrationExclusion,
    ) -> Result<Self, StoreError> {
        Self::open_with_migration_consent(publication.database_path(), true)
    }

    fn open_with_migration_consent(
        path: impl AsRef<Path>,
        allow_upgrade: bool,
    ) -> Result<Self, StoreError> {
        let path = path.as_ref();
        validate_database_path(path)?;
        prepare_private_database(path)?;

        let mut connection = Connection::open(path).map_err(StoreError::Sql)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(StoreError::Sql)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;\n\
                 PRAGMA synchronous = FULL;",
            )
            .map_err(StoreError::Sql)?;
        migrate(&mut connection, allow_upgrade)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_retarget_requirements (\n                 agent_id TEXT PRIMARY KEY,\n                 created_at INTEGER NOT NULL\n             );",
        ).map_err(StoreError::Sql)?;
        set_wal_mode(&connection)?;
        let issuer_scope = load_or_create_issuer_scope(&mut connection)?;

        Ok(Self {
            path: path.to_path_buf(),
            connection,
            issuer_scope,
            control: ControlSnapshot::Unread,
        })
    }

    /// Remplace les références d'identité dans les projections SQL et les
    /// enveloppes JSON persistées. Les références absentes du mapping restent
    /// volontairement inchangées : elles ne sont jamais réattribuées à un
    /// autre agent par cette migration.
    pub fn migrate_agent_participants(
        &mut self,
        mapping: &std::collections::BTreeMap<String, String>,
    ) -> Result<(), StoreError> {
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        for (legacy, agent_id) in mapping {
            for statement in [
                "UPDATE delegation_outbox SET target = ?1 WHERE target = ?2",
                "UPDATE delegate_idempotency SET participant = ?1 WHERE participant = ?2",
                "UPDATE routines SET participant = ?1 WHERE participant = ?2",
                "UPDATE delegation_generations SET participant_id = ?1 WHERE participant_id = ?2",
                "UPDATE coordination_events SET recipient = ?1 WHERE recipient = ?2",
                "UPDATE coordination_expectations SET recipient = ?1 WHERE recipient = ?2",
                "UPDATE notification_outbox SET recipient = ?1 WHERE recipient = ?2",
                "UPDATE tracked_request_outbox SET recipient = ?1 WHERE recipient = ?2",
            ] {
                tx.execute(statement, rusqlite::params![agent_id, legacy])
                    .map_err(StoreError::Sql)?;
            }
        }

        rewrite_json_agent_references(&tx, "delegations", "id", "payload_json", mapping)?;
        rewrite_json_agent_references(
            &tx,
            "delegation_outbox",
            "message_id",
            "message_bytes",
            mapping,
        )?;
        rewrite_json_agent_references(
            &tx,
            "notification_outbox",
            "message_id",
            "message_bytes",
            mapping,
        )?;
        rewrite_json_agent_references(
            &tx,
            "tracked_request_outbox",
            "effect_id",
            "message_bytes",
            mapping,
        )?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Liste les principales d'agents encore présentes dans Maicie, y compris
    /// les enveloppes historisées. La migration peut ainsi convertir une
    /// référence qui n'apparaît plus dans la flotte courante sans l'effacer.
    pub fn agent_references_for_identity_migration(&self) -> Result<BTreeSet<String>, StoreError> {
        let mut references = BTreeSet::new();
        let mut statement = self
            .connection
            .prepare(
                "SELECT target FROM delegation_outbox
             UNION SELECT participant FROM delegate_idempotency
             UNION SELECT participant FROM routines
             UNION SELECT participant_id FROM delegation_generations
             UNION SELECT recipient FROM coordination_events
             UNION SELECT recipient FROM coordination_expectations
             UNION SELECT recipient FROM notification_outbox
             UNION SELECT recipient FROM tracked_request_outbox",
            )
            .map_err(StoreError::Sql)?;
        for reference in statement
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(StoreError::Sql)?
        {
            references.insert(reference.map_err(StoreError::Sql)?);
        }
        drop(statement);
        for (table, column) in [
            ("delegations", "payload_json"),
            ("delegation_outbox", "message_bytes"),
            ("notification_outbox", "message_bytes"),
            ("tracked_request_outbox", "message_bytes"),
        ] {
            let mut statement = self
                .connection
                .prepare(&format!("SELECT {column} FROM {table}"))
                .map_err(StoreError::Sql)?;
            for payload in statement
                .query_map([], |row| row.get::<_, Vec<u8>>(0))
                .map_err(StoreError::Sql)?
            {
                let value: serde_json::Value =
                    serde_json::from_slice(&payload.map_err(StoreError::Sql)?)
                        .map_err(StoreError::Json)?;
                collect_json_agent_references(&value, &mut references);
            }
        }
        Ok(references)
    }

    /// Marque une ou plusieurs cibles devenues orphelines. Tant que
    /// l'opérateur ne les a pas retargetées, aucune de leurs outboxes n'est
    /// présentée au dispatcher.
    pub fn mark_agents_requires_retarget(
        &mut self,
        agent_ids: &BTreeSet<String>,
    ) -> Result<(), StoreError> {
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS agent_retarget_requirements (
                 agent_id TEXT PRIMARY KEY,
                 created_at INTEGER NOT NULL
             );",
        )
        .map_err(StoreError::Sql)?;
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
            .unwrap_or_default();
        for agent_id in agent_ids {
            tx.execute(
                "INSERT OR IGNORE INTO agent_retarget_requirements(agent_id, created_at)
                 VALUES (?1, ?2)",
                params![agent_id, created_at],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.commit().map_err(StoreError::Sql)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn issuer_scope(&self) -> &str {
        &self.issuer_scope
    }

    pub fn schema_version(&self) -> Result<i64, StoreError> {
        self.connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(StoreError::Sql)
    }

    /// Prépare en une transaction l'identité pending, la commande immuable et
    /// l'outbox locale. Aucun appel Bridget n'est effectué ici.
    pub fn prepare_project_registration(
        &mut self,
        intent: &ProjectRegistrationIntent,
    ) -> Result<ProjectRegistrationRecord, StoreError> {
        validate_project_registration_intent(intent)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        if let Some(existing) = project_registration_for_command(&tx, &intent.command_id)? {
            if existing.canonical_payload != intent.canonical_payload {
                return Err(StoreError::EnvelopeMismatch);
            }
            let record = project_registration_record_from_stored(&tx, existing)?;
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(record);
        }

        let identity = ProjectIdentity::pending(
            intent.proposed_project_id.clone(),
            intent.display_name.clone(),
            intent.command_id.clone(),
            intent.created_at,
        )
        .map_err(StoreError::Domain)?;
        tx.execute(
            "INSERT INTO project_identities (
                 project_id, display_name, status, created_at, updated_at, registration_command_id
             ) VALUES (?1, ?2, 'pending_binding', ?3, ?3, ?4)",
            params![
                identity.project_id,
                identity.display_name,
                identity.created_at,
                identity.registration_command_id,
            ],
        )
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(ref failure, _)
                if failure.code == ErrorCode::ConstraintViolation =>
            {
                StoreError::Conflict("identité projet déjà préparée")
            }
            other => StoreError::Sql(other),
        })?;
        tx.execute(
            "INSERT INTO project_registration_commands (
                 command_id, canonical_payload, proposed_project_id, resolved_project_id,
                 requested_root, state, retry_until
             ) VALUES (?1, ?2, ?3, NULL, ?4, 'prepared', ?5)",
            params![
                intent.command_id,
                intent.canonical_payload,
                intent.proposed_project_id,
                intent.requested_root,
                intent.retry_until,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO project_registration_outbox (
                 command_id, canonical_request, state
             ) VALUES (?1, ?2, 'prepared')",
            params![intent.command_id, intent.canonical_payload],
        )
        .map_err(StoreError::Sql)?;
        let stored = project_registration_for_command(&tx, &intent.command_id)?.ok_or(
            StoreError::Corrupt("commande projet absente après insertion"),
        )?;
        let record = project_registration_record_from_stored(&tx, stored)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(record)
    }

    /// Relit l'issue durable d'une commande sans accéder au store Bridget.
    pub fn project_registration(
        &self,
        command_id: &str,
    ) -> Result<Option<ProjectRegistrationRecord>, StoreError> {
        project_registration_for_command(&self.connection, command_id)?
            .map(|stored| project_registration_record_from_stored(&self.connection, stored))
            .transpose()
    }

    /// Relit les octets immuables de l'outbox de liaison pour rejouer la même
    /// commande après une interruption, sans construire une seconde intention.
    pub fn project_registration_request_bytes(
        &self,
        command_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        self.connection
            .query_row(
                "SELECT canonical_request FROM project_registration_outbox WHERE command_id = ?1",
                [command_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)
    }

    /// Fige l'issue terminale renvoyée par Bridget et ne promeut l'identité
    /// qu'après une liaison host attestée. Un même résultat est rejouable;
    /// une issue différente pour la même commande est un conflit durable.
    pub fn resolve_project_registration(
        &mut self,
        outcome: &ProjectBindOutcome,
    ) -> Result<ProjectRegistrationRecord, StoreError> {
        validate_project_bind_outcome(outcome)?;
        let outcome_bytes = serde_json::to_vec(outcome).map_err(StoreError::Json)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = project_registration_for_command(&tx, &outcome.command_id)?
            .ok_or(StoreError::NotFound("commande projet inconnue"))?;
        if stored.proposed_project_id != outcome.project_id {
            return Err(StoreError::EnvelopeMismatch);
        }
        if let Some(existing_outcome) = &stored.outcome_json {
            if existing_outcome != &outcome_bytes {
                return Err(StoreError::Conflict("issue projet terminale déjà figée"));
            }
            let record = project_registration_record_from_stored(&tx, stored)?;
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(record);
        }

        let mut record = project_registration_record_from_stored(&tx, stored)?;
        if record.identity.status != ProjectIdentityStatus::PendingBinding {
            return Err(StoreError::Corrupt(
                "identité projet non pending sans issue durable",
            ));
        }
        let (state, resolved_project_id, outbox_state) = match outcome.status {
            ProjectBindStatus::Active => {
                let generation = outcome
                    .binding_generation
                    .ok_or(StoreError::Corrupt("issue active sans génération"))?;
                record
                    .identity
                    .activate(generation, outcome.observed_at)
                    .map_err(StoreError::Domain)?;
                persist_project_identity(&tx, &record.identity)?;
                (
                    ProjectRegistrationState::Bound,
                    Some(record.identity.project_id.clone()),
                    "applied",
                )
            }
            ProjectBindStatus::RegistrationConflict => {
                record
                    .identity
                    .registration_conflict(outcome.observed_at)
                    .map_err(StoreError::Domain)?;
                persist_project_identity(&tx, &record.identity)?;
                (
                    ProjectRegistrationState::Failed,
                    outcome.existing_project_id.clone(),
                    "rejected",
                )
            }
            ProjectBindStatus::BindingFailed => {
                (ProjectRegistrationState::Failed, None, "rejected")
            }
        };
        let changed = tx
            .execute(
                "UPDATE project_registration_commands
                 SET resolved_project_id = ?1, state = ?2, outcome_json = ?3,
                     outcome_observed_at = ?4
                 WHERE command_id = ?5 AND outcome_json IS NULL",
                params![
                    resolved_project_id,
                    project_registration_state_name(state),
                    outcome_bytes,
                    outcome.observed_at,
                    outcome.command_id,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "commande projet modifiée concurremment pendant sa résolution",
            ));
        }
        let changed = tx
            .execute(
                "UPDATE project_registration_outbox SET state = ?1 WHERE command_id = ?2
                 AND state IN ('prepared', 'outcome_unknown')",
                params![outbox_state, outcome.command_id],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Corrupt(
                "outbox projet absente ou déjà terminale sans issue durable",
            ));
        }
        let stored = project_registration_for_command(&tx, &outcome.command_id)?.ok_or(
            StoreError::Corrupt("commande projet absente après résolution"),
        )?;
        let result = project_registration_record_from_stored(&tx, stored)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(result)
    }

    pub fn project_identities(&self) -> Result<Vec<ProjectIdentity>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT project_id, display_name, status, created_at, updated_at,
                        registration_command_id
                 FROM project_identities ORDER BY created_at ASC, project_id ASC",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], project_identity_from_row)
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            row.map_err(StoreError::Sql)
                .and_then(decode_project_identity)
        })
        .collect()
    }

    /// Fige la définition 016 et sa génération initiale sous transaction
    /// `IMMEDIATE`. Toutes les références et politiques sont validées avant la
    /// première écriture ; une définition déjà figée ne peut être remplacée
    /// par une configuration courante différente.
    pub fn register_coordination_snapshot(
        &mut self,
        definition: &DefinitionCoordination,
    ) -> Result<(), StoreError> {
        definition.verifier_bornes().map_err(StoreError::Domain)?;
        if definition.policies.is_empty() {
            return Err(StoreError::Invalid("définition de coordination vide"));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let objective_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM objectives WHERE id = ?1)",
                [definition.objectif_id.to_string()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        if !objective_exists {
            return Err(StoreError::NotFound("objectif de coordination absent"));
        }

        let delegations = load_delegations_for_coordination(&tx, definition.objectif_id)?;
        let participants: BTreeSet<String> = delegations
            .values()
            .map(|item| item.participant.clone())
            .collect();
        for policy in &definition.policies {
            if !delegations.contains_key(&policy.delegation_id) {
                return Err(StoreError::Invalid(
                    "politique liée à une délégation étrangère",
                ));
            }
            policy.verifier(&participants).map_err(StoreError::Domain)?;
            if policy
                .chaine_repli
                .iter()
                .any(|candidate| candidate.participant_id == crate::MAICIE_IDENTITY)
            {
                return Err(StoreError::Invalid("pilote interdit en repli"));
            }
        }
        for edge in &definition.dependencies {
            if !delegations.contains_key(&edge.prerequis_id)
                || !delegations.contains_key(&edge.dependant_id)
            {
                return Err(StoreError::Invalid(
                    "dépendance liée à une délégation étrangère",
                ));
            }
            if delegations[&edge.dependant_id].etat != EtatDelegation::Creee {
                return Err(StoreError::Invalid(
                    "dépendance déclarée après activation du dépendant",
                ));
            }
            let prerequisite_already_delivered: bool = tx
                .query_row(
                    "SELECT EXISTS(
                         SELECT 1 FROM guichet_receptions
                         WHERE operation='delivery_report' AND outcome='accepted'
                           AND objective_id=?1 AND delegation_id=?2
                     )",
                    params![edge.objectif_id.to_string(), edge.prerequis_id.to_string(),],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            if prerequisite_already_delivered {
                return Err(StoreError::Invalid(
                    "dépendance déclarée après fait qualifiant",
                ));
            }
        }
        for expectation in &definition.attentes {
            if expectation
                .delegation_id
                .is_some_and(|id| !delegations.contains_key(&id))
            {
                return Err(StoreError::Invalid(
                    "attente liée à une délégation étrangère",
                ));
            }
        }
        validate_coordination_dag(definition)?;

        let canonical_definition = canonical_coordination_definition(definition);
        let blocked: BTreeSet<Uuid> = canonical_definition
            .dependencies
            .iter()
            .map(|edge| edge.dependant_id)
            .collect();
        let mut coordinated_delegations: BTreeSet<Uuid> = canonical_definition
            .policies
            .iter()
            .map(|policy| policy.delegation_id)
            .collect();
        for edge in &canonical_definition.dependencies {
            coordinated_delegations.insert(edge.prerequis_id);
            coordinated_delegations.insert(edge.dependant_id);
        }
        let lineages: Vec<LigneeDelegation> = coordinated_delegations
            .iter()
            .map(|delegation_id| LigneeDelegation {
                delegation_id: *delegation_id,
                objectif_id: canonical_definition.objectif_id,
                generation_active: 1,
            })
            .collect();
        let generations: Vec<GenerationDelegation> = coordinated_delegations
            .iter()
            .map(|delegation_id| GenerationDelegation {
                delegation_id: *delegation_id,
                objectif_id: canonical_definition.objectif_id,
                generation: 1,
                participant_id: delegations[delegation_id].participant.clone(),
                etat: if blocked.contains(delegation_id) {
                    EtatGenerationDelegation::Bloquee
                } else {
                    EtatGenerationDelegation::Ouverte
                },
                generation_precedente: None,
                trigger_event_id: None,
            })
            .collect();
        let expected = StoredCoordinationSnapshot {
            definition: canonical_definition,
            lineages,
            generations,
        };
        if let Some(stored) = load_coordination_snapshot(&tx, definition.objectif_id)? {
            if stored == expected {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict(
                "snapshot de coordination déjà figé différemment",
            ));
        }

        insert_coordination_snapshot(&tx, &expected)?;
        tx.commit().map_err(StoreError::Sql)
    }

    pub fn coordination_snapshot(
        &self,
        objectif_id: Uuid,
    ) -> Result<Option<StoredCoordinationSnapshot>, StoreError> {
        load_coordination_snapshot(&self.connection, objectif_id)
    }

    /// Dernier curseur dont tous les effets locaux ont été commités. Une
    /// transaction avortée ne peut donc jamais faire sauter un événement au
    /// prochain `coordination_subscribe`.
    pub fn coordination_cursor(&self) -> Result<Option<u64>, StoreError> {
        let cursor: Option<i64> = self
            .connection
            .query_row("SELECT MAX(cursor) FROM coordination_events", [], |row| {
                row.get(0)
            })
            .map_err(StoreError::Sql)?;
        cursor
            .map(|value| {
                u64::try_from(value).map_err(|_| StoreError::Corrupt("curseur local invalide"))
            })
            .transpose()
    }

    /// Résout une demande suivie depuis les faits durables Maicie. L'épisode
    /// F29 gagne ; la demande initiale historique est admise uniquement sur la
    /// génération 1 explicitement persistée.
    pub fn reassignment_request_context(
        &self,
        request_id: &str,
    ) -> Result<ReassignmentRequestContext, StoreError> {
        reassignment_request_context_from(&self.connection, request_id)
    }

    /// Construit le lot normalisé autour d'un fait Bridget et y joint tout
    /// rapport de livraison local déjà durable pour la même demande. Cette
    /// priorité est factuelle ; aucun contenu humain n'est consulté.
    pub fn reassignment_batch_for_fact(
        &self,
        fact: FaitReassignation,
    ) -> Result<LotReassignation, StoreError> {
        reassignment_batch_for_fact_from(&self.connection, fact)
    }

    /// Mutant réservé à l'oracle causal T1610. Il reproduit le défaut interdit
    /// dans le chemin lui-même : un terminal transport est traité comme une
    /// qualification F28, puis traverse le vrai réducteur et la vraie écriture
    /// d'ouverture. Aucun appel de production ne possède cette voie.
    pub(crate) fn inject_forbidden_transport_terminal_into_f28_for_test(
        &mut self,
        request_id: &str,
        event_id: &str,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let context = reassignment_request_context_from(&tx, request_id)?;
        open_ready_dependents_with_qualification_mutator(
            &tx,
            context.delegation_id,
            event_id,
            observed_at,
            |qualifications| {
                for qualification in qualifications {
                    if qualification.dependance.prerequis_id == context.delegation_id {
                        qualification.hash_greffe = true;
                        qualification.cloture_evaluee = true;
                    }
                }
            },
            |_| Ok(()),
        )?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Applique un terminal Bridget sous un verrou pris AVANT la lecture des
    /// rapports de livraison. Un rapport concurrent se linéarise donc avant
    /// l'arbitrage (et gagne) ou après celui-ci, jamais dans une fenêtre TOCTOU.
    pub fn apply_reassignment_fact(
        &mut self,
        fact: FaitReassignation,
    ) -> Result<StoredReassignmentReduction, StoreError> {
        // SPEC-087 : une réassignation crée une génération, c'est un effet
        // autonome. Différée par la garde, elle n'est pas réduite : le fait
        // reste au guichet et revient à la relève suivante.
        if let Admission::Deferred { motif } =
            admit_autonomous_effect(AutonomousEffect::Reassignment, self.control)
        {
            return Err(StoreError::Conflict(if motif == "pause" {
                "réassignation différée : pause du référent"
            } else {
                "réassignation différée : état de contrôle inconnu"
            }));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let lot = reassignment_batch_for_fact_from(&tx, fact)?;
        let mut observer = |_| Ok(());
        let stored = apply_reassignment_batch_in_transaction(&tx, &lot, &mut observer)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(stored)
    }

    /// Réduit puis applique un fait 016 sous une transaction `IMMEDIATE`.
    /// La méthode n'effectue aucune I/O externe : événements, décision,
    /// transition et outboxes deviennent visibles ensemble au commit.
    pub fn apply_coordination_reduction(
        &mut self,
        input: &EntreeReductionCoordination,
    ) -> Result<StoredCoordinationReduction, StoreError> {
        self.apply_coordination_reduction_observed(input, |_| Ok(()))
    }

    /// Variante instrumentée réservée aux barrières transactionnelles et aux
    /// preuves de contention. Une erreur de l'observateur avant le commit
    /// provoque le rollback SQLite normal.
    pub fn apply_coordination_reduction_observed(
        &mut self,
        input: &EntreeReductionCoordination,
        mut observer: impl FnMut(CoordinationCommitPhase) -> Result<(), StoreError>,
    ) -> Result<StoredCoordinationReduction, StoreError> {
        if input.event_id().trim().is_empty() || input.generation() == 0 {
            return Err(StoreError::Invalid("entrée du réducteur incomplète"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = apply_coordination_reduction_in_transaction(&tx, input, &mut observer)?;
        observer(CoordinationCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(CoordinationCommitPhase::AfterCommit)?;
        Ok(stored)
    }

    /// Enregistre un `reminder_sent` attesté **sans** le réduire en F29.
    ///
    /// Utilisé quand Bridget pousse un rappel pour une demande étrangère au
    /// greffe (conversation libre, relance hors Maicie). Le curseur doit
    /// avancer sinon chaque commande re-consomme le poison. Aucune délégation
    /// n'est inventée — même doctrine que le `NotFound` toléré sur les
    /// terminaux guichet hors F29.
    pub fn acknowledge_untracked_coordination_event(
        &mut self,
        event: &EvenementCoordination,
    ) -> Result<(), StoreError> {
        if event.event_id().trim().is_empty()
            || event.request_id().trim().is_empty()
            || event.cursor() == 0
        {
            return Err(StoreError::Invalid("événement de coordination incomplet"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let existing: Option<Vec<u8>> = tx
            .query_row(
                "SELECT canonical_bytes FROM coordination_events WHERE event_id = ?1",
                [event.event_id()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        match existing {
            Some(bytes) if bytes == event.canonical_bytes() => {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            Some(_) => return Err(StoreError::EnvelopeMismatch),
            None => {}
        }
        let inserted = tx
            .execute(
                "INSERT INTO coordination_events(
                     event_id, request_id, kind, reminder_message_id, recipient,
                     transport_generation, cursor, freshness, observed_at, canonical_bytes
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    event.event_id(),
                    event.request_id(),
                    coordination_event_kind_name(event.kind()),
                    event.reminder_message_id(),
                    event.recipient(),
                    i64::try_from(event.generation())
                        .map_err(|_| StoreError::Invalid("génération transport hors borne"))?,
                    i64::try_from(event.cursor())
                        .map_err(|_| StoreError::Invalid("curseur transport hors borne"))?,
                    coordination_freshness_name(event.freshness()),
                    event.observed_at(),
                    event.canonical_bytes(),
                ],
            )
            .map_err(map_coordination_insert_error)?;
        if inserted != 1 {
            return Err(StoreError::Conflict(
                "événement de coordination non enregistré",
            ));
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(())
    }

    /// Applique un rappel attesté et son fait F29 sous le même verrou SQLite.
    /// Le curseur n'est donc jamais avancé si la décision de réassignation,
    /// ses transitions ou ses outboxes échouent. Le lot est construit après
    /// `BEGIN IMMEDIATE`, ce qui ferme la course avec un rapport de livraison.
    pub fn apply_attested_coordination_and_reassignment(
        &mut self,
        input: &EntreeReductionCoordination,
        fact: FaitReassignation,
        mut observer: impl FnMut(CoordinationCommitPhase) -> Result<(), StoreError>,
    ) -> Result<(StoredCoordinationReduction, StoredReassignmentReduction), StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let lot = reassignment_batch_for_fact_from(&tx, fact)?;
        if input.objectif_id() != lot.objectif_id
            || input.delegation_id() != lot.delegation_id
            || input.generation() != lot.generation
        {
            return Err(StoreError::Invalid("rappel et lot F29 divergents"));
        }
        let coordination = apply_coordination_reduction_in_transaction(&tx, input, &mut observer)?;
        let mut reassignment_observer = |phase| match phase {
            ReassignmentCommitPhase::AfterDecision => {
                observer(CoordinationCommitPhase::AfterDecision)
            }
            ReassignmentCommitPhase::AfterGenerations => {
                observer(CoordinationCommitPhase::AfterGenerations)
            }
            ReassignmentCommitPhase::AfterRequestOutboxes => {
                observer(CoordinationCommitPhase::AfterRequestOutboxes)
            }
            ReassignmentCommitPhase::AfterNotifications => {
                observer(CoordinationCommitPhase::AfterNotifications)
            }
            ReassignmentCommitPhase::AfterEvents
            | ReassignmentCommitPhase::BeforeCommit
            | ReassignmentCommitPhase::AfterCommit => Ok(()),
        };
        let reassignment =
            apply_reassignment_batch_in_transaction(&tx, &lot, &mut reassignment_observer)?;
        observer(CoordinationCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(CoordinationCommitPhase::AfterCommit)?;
        Ok((coordination, reassignment))
    }

    /// Applique un lot F29 sans I/O externe. Les faits déjà consommés sont
    /// dédupliqués par leurs octets structurés ; un rejeu exact relit la
    /// réduction durable au lieu de recalculer depuis l'état courant.
    pub fn apply_reassignment_batch(
        &mut self,
        lot: &LotReassignation,
    ) -> Result<StoredReassignmentReduction, StoreError> {
        self.apply_reassignment_batch_observed(lot, |_| Ok(()))
    }

    pub fn apply_reassignment_batch_observed(
        &mut self,
        lot: &LotReassignation,
        mut observer: impl FnMut(ReassignmentCommitPhase) -> Result<(), StoreError>,
    ) -> Result<StoredReassignmentReduction, StoreError> {
        lot.verifier().map_err(StoreError::Domain)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = apply_reassignment_batch_in_transaction(&tx, lot, &mut observer)?;
        observer(ReassignmentCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(ReassignmentCommitPhase::AfterCommit)?;
        Ok(stored)
    }

    pub fn pending_tracked_request_outboxes(
        &self,
    ) -> Result<Vec<TrackedRequestOutbox>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT effect_id, issued_at, objective_id, delegation_id, generation,
                        event_id, policy_version, kind, request_id, recipient,
                        message_bytes, state
                 FROM tracked_request_outbox
                 WHERE terminal = 0 AND state IN ('prepared','outcome_unknown')
                   AND NOT EXISTS (SELECT 1 FROM agent_retarget_requirements requirement WHERE requirement.agent_id = tracked_request_outbox.recipient)
                 ORDER BY effect_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, Vec<u8>>(10)?,
                    row.get::<_, String>(11)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let row = row.map_err(StoreError::Sql)?;
            Ok(TrackedRequestOutbox {
                effect_id: parse_uuid(&row.0)?,
                issued_at: row.1,
                objectif_id: parse_uuid(&row.2)?,
                delegation_id: parse_uuid(&row.3)?,
                generation: u64::try_from(row.4)
                    .map_err(|_| StoreError::Corrupt("génération d'effet F29 invalide"))?,
                event_id: row.5,
                policy_version: u64::try_from(row.6)
                    .map_err(|_| StoreError::Corrupt("version d'effet F29 invalide"))?,
                kind: parse_tracked_request_kind(&row.7)?,
                request_id: row.8,
                recipient: row.9,
                message_bytes: row.10,
                etat: parse_notification_state(&row.11)?,
            })
        })
        .collect()
    }

    /// Lit les notifications non terminales sans reconstruire leur charge.
    /// Les octets, la clé idempotente et l'horodatage canonique sont exactement
    /// ceux du commit métier.
    pub fn pending_notification_outboxes(&self) -> Result<Vec<NotificationOutbox>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT message_id, idempotency_key, issued_at, objective_id, delegation_id,
                        generation, event_id, policy_version, recipient, message_bytes, state
                 FROM notification_outbox
                 WHERE terminal = 0 AND state IN ('prepared','outcome_unknown')
                   AND NOT EXISTS (SELECT 1 FROM agent_retarget_requirements requirement WHERE requirement.agent_id = notification_outbox.recipient)
                 ORDER BY message_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, Vec<u8>>(9)?,
                    row.get::<_, String>(10)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (
                message_id,
                idempotency_key,
                issued_at,
                objective_id,
                delegation_id,
                generation,
                event_id,
                policy_version,
                recipient,
                message_bytes,
                state,
            ) = row.map_err(StoreError::Sql)?;
            let outbox = NotificationOutbox {
                message_id: parse_uuid(&message_id)?,
                idempotency_key,
                issued_at: issued_at.ok_or(StoreError::Corrupt(
                    "notification historique sans issued_at",
                ))?,
                objectif_id: parse_uuid(&objective_id)?,
                delegation_id: delegation_id.as_deref().map(parse_uuid).transpose()?,
                generation: generation
                    .map(|value| {
                        u64::try_from(value)
                            .map_err(|_| StoreError::Corrupt("génération notification invalide"))
                    })
                    .transpose()?,
                event_id,
                policy_version: u64::try_from(policy_version)
                    .map_err(|_| StoreError::Corrupt("version notification invalide"))?,
                recipient,
                message_bytes,
                etat: parse_notification_state(&state)?,
            };
            outbox.verifier().map_err(StoreError::Domain)?;
            Ok(outbox)
        })
        .collect()
    }

    /// Fige l'issue du transport pour une notification. Un rejeu de la même
    /// issue est idempotent ; une seconde issue terminale divergente est
    /// refusée sans perdre la preuve déjà durable.
    pub fn record_notification_issue(
        &mut self,
        message_id: Uuid,
        issue: &IdempotencyIssue,
    ) -> Result<(), StoreError> {
        let next_state = match issue {
            IdempotencyIssue::Accepted { .. } => EtatNotificationOutbox::Accepted,
            IdempotencyIssue::OutcomeUnknown { .. } => EtatNotificationOutbox::OutcomeUnknown,
            IdempotencyIssue::Rejected { .. }
            | IdempotencyIssue::EnvelopeMismatch
            | IdempotencyIssue::IdempotencyExpired
            | IdempotencyIssue::InvalidIssuedAt => EtatNotificationOutbox::Rejected,
        };
        let terminal = matches!(
            next_state,
            EtatNotificationOutbox::Accepted | EtatNotificationOutbox::Rejected
        );
        let issue_bytes = serde_json::to_vec(&encode_issue(issue)).map_err(StoreError::Json)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json FROM notification_outbox
                 WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((state, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("notification inconnue"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue notification déjà figée"));
        }
        let current_state = parse_notification_state(&state)?;
        if current_state != next_state {
            current_state
                .transition_vers(next_state)
                .map_err(|_| StoreError::Conflict("transition notification interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE notification_outbox
                 SET state = ?1, terminal = ?2, last_issue_json = ?3
                 WHERE message_id = ?4 AND terminal = 0 AND state = ?5",
                params![
                    notification_state_name(next_state),
                    i64::from(terminal),
                    issue_bytes,
                    message_id.to_string(),
                    state,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("notification modifiée concurremment"));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Greffe un rapport de livraison et son effet de coordination dans une
    /// transaction `IMMEDIATE` unique. Le reçu, les agrégats, la décision et
    /// les octets exacts de réponse deviennent visibles ensemble.
    pub fn graft_delivery_report(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
        report: &RapportLivraison,
        response_message_id: &str,
        now: i64,
    ) -> Result<StoredGuichetReply, StoreError> {
        self.graft_delivery_report_observed(
            claim,
            canonical,
            report,
            response_message_id,
            now,
            |_| Ok(()),
        )
    }

    /// Variante de test aux frontières transactionnelles. L'observateur ne
    /// remplace aucune faute d'I/O : il sert seulement de barrière pour tuer
    /// un vrai processus avant ou après le commit SQLite.
    pub fn graft_delivery_report_observed(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
        report: &RapportLivraison,
        response_message_id: &str,
        now: i64,
        mut observer: impl FnMut(GuichetCommitPhase) -> Result<(), StoreError>,
    ) -> Result<StoredGuichetReply, StoreError> {
        if now <= 0 || response_message_id.trim().is_empty() {
            return Err(StoreError::Invalid("réponse guichet incomplète"));
        }
        if canonical.issuer_scope != claim.issuer_scope
            || canonical.request_id != claim.request_id
            || canonical.request.operation() != OperationGuichet::DeliveryReport
        {
            return Err(StoreError::Invalid("claim et greffe divergents"));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        if let Some(reception) =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
        {
            if reception.canonical_request_bytes != claim.canonical_request {
                return Err(StoreError::EnvelopeMismatch);
            }
            let reception = refresh_reception_for_claim(&tx, reception, claim, report)?;
            let correlation = load_guichet_correlation(
                &tx,
                report.in_reply_to.as_str(),
                reception.response_message_id.as_str(),
            )?;
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(StoredGuichetReply {
                reception,
                correlation,
                replayed: true,
            });
        }

        let row: Option<StoredGuichetAggregates> = tx
            .query_row(
                "SELECT obj.state, obj.payload_json, d.state, d.payload_json, o.message_id\n\
                 FROM delegations d\n\
                 JOIN objectives obj ON obj.id = d.objective_id\n\
                 JOIN delegation_outbox o ON o.delegation_id = d.id\n\
                 WHERE d.id = ?1",
                [report.delegation_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((objective_state, objective_json, delegation_state, delegation_json, message_id)) =
            row
        else {
            return Err(StoreError::NotFound("délégation guichet absente"));
        };
        let mut objective: ObjectifCoordonne =
            serde_json::from_slice(&objective_json).map_err(StoreError::Json)?;
        let mut delegation: Delegation =
            serde_json::from_slice(&delegation_json).map_err(StoreError::Json)?;
        if objective.id != report.objective_id
            || delegation.objectif_id != objective.id
            || delegation.id != report.delegation_id
            || delegation.participant != canonical.from
            || message_id != report.in_reply_to
            || objective.etat != parse_objective_state(&objective_state)?
            || delegation.etat != parse_delegation_state(&delegation_state)?
        {
            return Err(StoreError::Invalid("relations du rapport invalides"));
        }
        if let Some(reason) = review_verdict_refusal(&delegation, report) {
            return Err(StoreError::GuichetRefusal(reason));
        }
        // Un objectif déjà clos (ou une délégation hors greffe) n'est pas une
        // faute fatale : c'est le cas `request_already_terminal` du contrat.
        // On refuse sans rouvrir, on journalise, et le reçu empêche le rejeu.
        let graftable = matches!(
            objective.etat,
            EtatObjectif::EnCoordination | EtatObjectif::AEvaluer
        ) && matches!(
            delegation.etat,
            EtatDelegation::Creee | EtatDelegation::AEvaluer
        );

        let lifecycle_state: Option<String> = tx
            .query_row(
                "SELECT state FROM guichet_lifecycle_events\n\
                 WHERE issuer_scope = ?1 AND request_id = ?2",
                params![canonical.issuer_scope, canonical.request_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let issue = match lifecycle_state.as_deref() {
            Some("cancelled" | "timed_out") => IssueGreffe::DemandeDejaTerminale,
            Some("answered") | None if graftable => IssueGreffe::Accepted,
            Some("answered") | None => IssueGreffe::DemandeDejaTerminale,
            Some(_) => return Err(StoreError::Corrupt("terminal guichet inconnu")),
        };
        let reply_bytes = delivery_reply_bytes(claim, report, response_message_id, issue)
            .map_err(|_| StoreError::Invalid("réponse guichet non sérialisable"))?;

        let previous_objective = objective.clone();
        let previous_delegation = delegation.clone();
        if graftable && objective.etat == EtatObjectif::EnCoordination {
            objective
                .transition(EtatObjectif::AEvaluer, now)
                .map_err(StoreError::Domain)?;
        }
        if graftable && delegation.etat == EtatDelegation::Creee {
            delegation
                .transition(EtatDelegation::AEvaluer)
                .map_err(StoreError::Domain)?;
        }
        if graftable
            && delegation.review_target.is_some()
            && delegation.etat == EtatDelegation::AEvaluer
        {
            delegation
                .transition(EtatDelegation::Terminee)
                .map_err(StoreError::Domain)?;
        }
        update_guichet_aggregates(
            &tx,
            &previous_objective,
            &objective,
            &previous_delegation,
            &delegation,
        )?;

        let decision = DecisionCoordination {
            id: Uuid::new_v4(),
            objectif_id: objective.id,
            kind: TypeDecision::ConstaterIssue,
            proposee_par: "maicie".to_string(),
            etat: EtatDecision::Appliquee,
            motif: if !graftable {
                format!(
                    "rapport refusé sans réouverture : état métier incompatible : {}",
                    report.delivery_hash
                )
            } else {
                match issue {
                    IssueGreffe::Accepted => match &report.review_verdict {
                        Some(evidence) => format!(
                            "verdict de revue {} reçu sur {} ; livraison {}",
                            evidence.verdict.as_str(),
                            evidence.measured_head,
                            report.delivery_hash
                        ),
                        None => {
                            format!("rapport de livraison greffé : {}", report.delivery_hash)
                        }
                    },
                    IssueGreffe::DemandeDejaTerminale => format!(
                        "rapport tardif greffé sans réouverture : {}",
                        report.delivery_hash
                    ),
                    IssueGreffe::Refusee => {
                        return Err(StoreError::Corrupt("refus greffé par la voie livraison"));
                    }
                }
            },
        };
        decision.verifier().map_err(StoreError::Domain)?;
        let decision_json = serde_json::to_vec(&decision).map_err(StoreError::Json)?;
        let inserted = tx
            .execute(
                "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
                 VALUES (?1, ?2, 'applied', ?3)",
                params![
                    decision.id.to_string(),
                    objective.id.to_string(),
                    decision_json
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("décision de greffe non enregistrée"));
        }
        observer(GuichetCommitPhase::AfterDecisionInsert)?;

        let operation = operation_name(OperationGuichet::DeliveryReport);
        let outcome = issue_name(issue);
        let inserted = tx
            .execute(
                "INSERT INTO guichet_receptions(\n\
                     issuer_scope, request_id, operation, canonical_request_bytes,\n\
                     objective_id, delegation_id, delivery_hash, in_reply_to,\n\
                     response_message_id, outcome, reply_bytes, decision_id, processed_at,\n\
                     claim_generation, claim_token\n\
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                params![
                    canonical.issuer_scope,
                    canonical.request_id,
                    operation,
                    claim.canonical_request,
                    objective.id.to_string(),
                    delegation.id.to_string(),
                    report.delivery_hash,
                    report.in_reply_to,
                    response_message_id,
                    outcome,
                    reply_bytes,
                    decision.id.to_string(),
                    now,
                    i64::try_from(claim.claim_generation)
                        .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                    claim.claim_token,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("reçu de greffe non enregistré"));
        }

        let correlation = upsert_guichet_correlation(
            &tx,
            &canonical.issuer_scope,
            &canonical.request_id,
            &report.in_reply_to,
            response_message_id,
        )?;
        let opening_event_id = format!(
            "delivery-report:{}:{}",
            canonical.issuer_scope, canonical.request_id
        );
        open_ready_dependents(&tx, report.delegation_id, &opening_event_id, now, |phase| {
            observer(match phase {
                DependencyOpeningCommitPhase::Decision => {
                    GuichetCommitPhase::AfterDependentDecision
                }
                DependencyOpeningCommitPhase::Generation => {
                    GuichetCommitPhase::AfterDependentTransition
                }
                DependencyOpeningCommitPhase::Notification => {
                    GuichetCommitPhase::AfterDependentOutbox
                }
            })
        })?;
        let reception =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?.ok_or(
                StoreError::Corrupt("reçu de greffe introuvable après insertion"),
            )?;
        observer(GuichetCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(GuichetCommitPhase::AfterCommit)?;
        Ok(StoredGuichetReply {
            reception,
            correlation: Some(correlation),
            replayed: false,
        })
    }

    /// Persiste un fait terminal Bridget sans lui attribuer d'effet métier.
    /// Un événement arrivé avant le rapport prépare seulement sa corrélation.
    pub fn record_guichet_lifecycle_event(
        &mut self,
        event: &EvenementCycleGuichet,
    ) -> Result<GuichetLifecycleResult, StoreError> {
        if event.state == EtatRequeteGuichet::Answered
            && (event.in_reply_to.is_none() || event.response_message_id.is_none())
        {
            return Err(StoreError::Invalid("événement answered non corrélé"));
        }
        let event_json = serde_json::to_vec(event).map_err(StoreError::Json)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let existing: Option<Vec<u8>> = tx
            .query_row(
                "SELECT payload_json FROM guichet_lifecycle_events\n\
                 WHERE issuer_scope = ?1 AND event_id = ?2",
                params![event.issuer_scope, event.event_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(existing) = existing {
            if existing != event_json {
                return Err(StoreError::EnvelopeMismatch);
            }
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(GuichetLifecycleResult::Replayed);
        }
        let terminal: Option<(String, String)> = tx
            .query_row(
                "SELECT event_id, state FROM guichet_lifecycle_events\n\
                 WHERE issuer_scope = ?1 AND request_id = ?2",
                params![event.issuer_scope, event.request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if terminal.is_some() {
            return Err(StoreError::Conflict("terminal guichet déjà attesté"));
        }
        let inserted = tx
            .execute(
                "INSERT INTO guichet_lifecycle_events(\n\
                     issuer_scope, event_id, request_id, state, observed_at,\n\
                     in_reply_to, response_message_id, payload_json\n\
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    event.issuer_scope,
                    event.event_id,
                    event.request_id,
                    lifecycle_state_name(event.state),
                    event.observed_at,
                    event.in_reply_to,
                    event.response_message_id,
                    event_json,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("événement guichet non enregistré"));
        }
        if let (Some(in_reply_to), Some(response_message_id)) =
            (&event.in_reply_to, &event.response_message_id)
        {
            upsert_guichet_correlation(
                &tx,
                &event.issuer_scope,
                &event.request_id,
                in_reply_to,
                response_message_id,
            )?;
            let changed = tx
                .execute(
                    "UPDATE guichet_correlations\n\
                     SET lifecycle_event_id = ?1, lifecycle_state = ?2\n\
                     WHERE in_reply_to = ?3 AND response_message_id = ?4\n\
                       AND issuer_scope = ?5 AND request_id = ?6",
                    params![
                        event.event_id,
                        lifecycle_state_name(event.state),
                        in_reply_to,
                        response_message_id,
                        event.issuer_scope,
                        event.request_id,
                    ],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict(
                    "corrélation terminale non enregistrée",
                ));
            }
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(GuichetLifecycleResult::Recorded)
    }

    /// Greffe un refus local de contrainte avant de le rendre à l'appelant.
    ///
    /// La ligne ne contient que des valeurs typées : horodatage, motif fermé
    /// et UUID effectivement reconnu dans ce store. Le compte retourné est lu
    /// dans la même transaction que l'insertion ; un refus sans ligne durable
    /// est donc impossible à présenter comme un refus métier.
    pub fn record_local_delegate_refusal(
        &mut self,
        observed_at: i64,
        reason: MotifRefusDelegationLocale,
        cited_objective_id: Uuid,
    ) -> Result<u64, StoreError> {
        if observed_at <= 0 || cited_objective_id.is_nil() {
            return Err(StoreError::Invalid("refus local incomplet"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let objective_exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM objectives WHERE id = ?1)",
                [cited_objective_id.to_string()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        if !objective_exists {
            return Err(StoreError::NotFound("objectif cité par le refus local"));
        }
        let inserted = tx
            .execute(
                "INSERT INTO local_delegate_refusals(
                     observed_at, reason, cited_objective_id
                 ) VALUES (?1, ?2, ?3)",
                params![observed_at, reason.code(), cited_objective_id.to_string(),],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("refus local non enregistré"));
        }
        let count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM local_delegate_refusals WHERE reason = ?1",
                [reason.code()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        let count = u64::try_from(count)
            .map_err(|_| StoreError::Corrupt("compteur de refus local invalide"))?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(count)
    }

    /// Compteurs durables exposés par `maicie status`. La projection reste
    /// locale et pure : aucune lecture du catalogue ni I/O Bridget.
    pub fn local_delegate_refusal_counts(
        &self,
    ) -> Result<CompteursRefusDelegationLocale, StoreError> {
        let count: i64 = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM local_delegate_refusals WHERE reason = ?1",
                [MotifRefusDelegationLocale::SuiteAucuneAvecCitationNonClassee.code()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        Ok(CompteursRefusDelegationLocale {
            suite_aucune_avec_citation_non_classee: u64::try_from(count)
                .map_err(|_| StoreError::Corrupt("compteur de refus local invalide"))?,
        })
    }

    /// Ajoute un message libre destiné à Maicie au journal privé append-only.
    /// Cette table est volontairement disjointe des agrégats d'orchestration :
    /// aucune écriture conversationnelle ne peut créer ou modifier un objectif,
    /// une délégation ou une outbox.
    pub fn record_conversation(&mut self, record: &ConversationRecord) -> Result<(), StoreError> {
        let inserted = self
            .connection
            .execute(
                "INSERT INTO conversation_records(sender, body_bytes) VALUES (?1, ?2)",
                params![record.sender, record.body.as_bytes()],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("conversation non enregistrée"));
        }
        Ok(())
    }

    /// Relit les conversations dans leur ordre append-only. Les octets du
    /// corps sont décodés sans transformation : une corruption est fail-closed.
    pub fn conversations(&self) -> Result<Vec<ConversationRecord>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT sender, body_bytes FROM conversation_records ORDER BY sequence ASC")
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (sender, body_bytes) = row.map_err(StoreError::Sql)?;
            let body = String::from_utf8(body_bytes)
                .map_err(|_| StoreError::Corrupt("corps de conversation non UTF-8"))?;
            Ok(ConversationRecord { sender, body })
        })
        .collect()
    }

    /// Lit les objectifs Maicie avec leurs délégations et décisions locales.
    /// Cette vue ne joint volontairement aucune donnée Bridget : T018 ajoutera
    /// les sources de transport publiques sans en faire une autorité métier.
    pub fn objective_snapshots(
        &self,
        objective_id: Option<Uuid>,
    ) -> Result<Vec<ObjectiveSnapshot>, StoreError> {
        // Complexité : O(n) requêtes associées (délégations, décisions,
        // remises) pour n objectifs. Le chemin `status` est borné par SC-008
        // à 100 objectifs et son p95 mesuré reste sous 250 ms.
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM objectives\n\
                 WHERE (?1 IS NULL OR id = ?1) ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.map(|id| id.to_string())], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let objective: ObjectifCoordonne =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if objective.id.to_string() != id || objective.etat != parse_objective_state(&state)? {
                return Err(StoreError::Corrupt("objectif et index SQLite divergents"));
            }
            let delegations = self.delegations_for(objective.id)?;
            let decisions = self.decisions_for(objective.id)?;
            let remises_locales = self.remises_locales_for(objective.id)?;
            let costs = self.mission_costs(objective.id)?;
            Ok(ObjectiveSnapshot {
                objective,
                delegations,
                decisions,
                remises_locales,
                costs,
            })
        })
        .collect()
    }

    /// Enregistre seulement une copie de fait Bridget. Les curseurs anciens ne
    /// réécrivent pas la dernière observation et aucun objectif n'est muté.
    pub fn upsert_execution_projection(
        &mut self,
        projection: &ExecutionProjection,
    ) -> Result<bool, StoreError> {
        projection.verifier().map_err(StoreError::Domain)?;
        let payload = serde_json::to_vec(projection).map_err(StoreError::Json)?;
        let changed = self
            .connection
            .execute(
                "INSERT INTO delegation_execution_projections(
                     delegation_id, execution_id, payload_json, observation_cursor,
                     source_generation, observed_at
                 ) VALUES(?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(delegation_id) DO UPDATE SET
                     execution_id = excluded.execution_id,
                     payload_json = excluded.payload_json,
                     observation_cursor = excluded.observation_cursor,
                     source_generation = excluded.source_generation,
                     observed_at = excluded.observed_at
                 WHERE excluded.source_generation > delegation_execution_projections.source_generation
                    OR (excluded.source_generation = delegation_execution_projections.source_generation
                        AND excluded.observation_cursor >= delegation_execution_projections.observation_cursor)",
                params![
                    projection.reference.delegation_id.to_string(),
                    projection.reference.execution_id,
                    payload,
                    projection.observation_cursor,
                    projection.source_generation,
                    projection.observed_at,
                ],
            )
            .map_err(StoreError::Sql)?;
        Ok(changed == 1)
    }

    /// Relit une observation opaque sans joindre l'état de mission propriétaire.
    pub fn execution_projection_for_delegation(
        &self,
        delegation_id: Uuid,
    ) -> Result<Option<ExecutionProjection>, StoreError> {
        let payload = self
            .connection
            .query_row(
                "SELECT payload_json FROM delegation_execution_projections WHERE delegation_id = ?1",
                [delegation_id.to_string()],
                |row| row.get::<_, Vec<u8>>(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        payload
            .map(|bytes| {
                let projection: ExecutionProjection =
                    serde_json::from_slice(&bytes).map_err(StoreError::Json)?;
                if projection.reference.delegation_id != delegation_id {
                    return Err(StoreError::Corrupt(
                        "projection et clé délégation divergentes",
                    ));
                }
                projection.verifier().map_err(StoreError::Domain)?;
                Ok(projection)
            })
            .transpose()
    }

    /// Relit tous les verdicts de revue depuis les réponses terminales déjà
    /// persistées. Aucun cache ni colonne parallèle ne peut diverger des
    /// octets qui ont réellement quitté la greffe.
    ///
    /// Complexité : O(r log r), où `r` est le nombre de rapports acceptés.
    pub fn review_verdicts(&self) -> Result<Vec<StoredReviewVerdict>, StoreError> {
        self.review_verdicts_matching(None)
    }

    /// Variante bornée à un objectif, utilisée par les lectures ciblées.
    pub fn review_verdicts_for(
        &self,
        objective_id: Uuid,
    ) -> Result<Vec<StoredReviewVerdict>, StoreError> {
        self.review_verdicts_matching(Some(objective_id))
    }

    fn review_verdicts_matching(
        &self,
        objective_id: Option<Uuid>,
    ) -> Result<Vec<StoredReviewVerdict>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT objective_id, delegation_id, reply_bytes
                 FROM guichet_receptions
                 WHERE (?1 IS NULL OR objective_id = ?1)
                   AND operation = 'delivery_report'
                   AND outcome = 'accepted'
                 ORDER BY objective_id, processed_at, request_id",
            )
            .map_err(StoreError::Sql)?;
        let requested = objective_id.map(|value| value.to_string());
        let rows = statement
            .query_map(params![requested.as_deref()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        let mut verdicts = Vec::new();
        let mut seen = BTreeSet::new();
        for row in rows {
            let (stored_objective, stored_delegation, reply_bytes) =
                row.map_err(StoreError::Sql)?;
            let stored_objective = parse_uuid(&stored_objective)?;
            let stored_delegation = parse_uuid(&stored_delegation)?;
            if objective_id.is_some_and(|expected| stored_objective != expected) {
                return Err(StoreError::Corrupt(
                    "objectif du verdict de revue divergent",
                ));
            }
            let reply = std::str::from_utf8(&reply_bytes)
                .map_err(|_| StoreError::Corrupt("réponse guichet non UTF-8"))?;
            let decoded: WrapperToDaemon = decode(reply).map_err(StoreError::Json)?;
            let WrapperToDaemon::GuichetReply {
                outcome: GuichetOutcome::Accepted,
                payload:
                    GuichetReplyPayload::DeliveryReport {
                        objective_id: payload_objective,
                        delegation_id: payload_delegation,
                        review_verdict,
                        ..
                    },
                ..
            } = decoded
            else {
                return Err(StoreError::Corrupt(
                    "réponse de verdict acceptée non canonique",
                ));
            };
            let payload_objective = parse_uuid(&payload_objective)?;
            let payload_delegation = parse_uuid(&payload_delegation)?;
            if payload_objective != stored_objective || payload_delegation != stored_delegation {
                return Err(StoreError::Corrupt(
                    "relations du verdict de revue divergentes",
                ));
            }
            let Some(evidence) = review_verdict else {
                continue;
            };
            if !evidence.is_valid() || !seen.insert((stored_objective, stored_delegation)) {
                return Err(StoreError::Corrupt("verdict de revue invalide ou dupliqué"));
            }
            verdicts.push(StoredReviewVerdict {
                objective_id: stored_objective,
                delegation_id: stored_delegation,
                evidence,
            });
        }
        Ok(verdicts)
    }

    /// Fenêtres de délégation : agent → premier `issued_at` d'outbox, sinon
    /// création de l'objectif. Sert à interroger le ledger Bridget sans
    /// inventer de tokens.
    pub fn delegation_cost_windows(
        &self,
        objective_id: Uuid,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        let objective = self
            .objective_snapshots(Some(objective_id))?
            .into_iter()
            .next()
            .ok_or(StoreError::NotFound("objectif absent"))?
            .objective;
        let mut windows: BTreeMap<String, i64> = BTreeMap::new();
        for delegation in self.delegations_for(objective_id)? {
            windows
                .entry(delegation.participant)
                .or_insert(objective.cree_at.max(1));
        }
        let mut statement = self
            .connection
            .prepare(
                "SELECT target, MIN(issued_at) FROM delegation_outbox
                 WHERE objective_id = ?1 GROUP BY target",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(StoreError::Sql)?;
        for row in rows {
            let (target, issued_at) = row.map_err(StoreError::Sql)?;
            if issued_at > 0 {
                windows.insert(target, issued_at);
            }
        }
        Ok(windows.into_iter().collect())
    }

    /// Coûts portés par un objectif clos. Vide tant que l'objectif n'est pas
    /// clôturé : le greffe n'anticipe jamais une consommation.
    pub fn mission_costs(&self, objective_id: Uuid) -> Result<Vec<CoutMissionAgent>, StoreError> {
        self.query_mission_costs(Some(objective_id))
    }

    /// Tous les coûts de missions closes, pour `registre list`.
    pub fn all_mission_costs(&self) -> Result<Vec<CoutMissionAgent>, StoreError> {
        self.query_mission_costs(None)
    }

    fn query_mission_costs(
        &self,
        objective_id: Option<Uuid>,
    ) -> Result<Vec<CoutMissionAgent>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT objective_id, agent, from_secs, to_secs, attested,
                        turns, input_tokens, output_tokens,
                        cache_creation_input_tokens, cache_read_input_tokens,
                        facturable_tokens
                 FROM objective_costs
                 WHERE (?1 IS NULL OR objective_id = ?1)
                 ORDER BY objective_id, agent",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.map(|id| id.to_string())], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, Option<i64>>(10)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        let mut costs = Vec::new();
        for row in rows {
            let (
                _objective_id,
                agent,
                from_secs,
                to_secs,
                attested,
                turns,
                input_tokens,
                output_tokens,
                cache_creation,
                cache_read,
                facturable,
            ) = row.map_err(StoreError::Sql)?;
            let attested = attested == 1;
            if attested
                && (turns.is_none()
                    || input_tokens.is_none()
                    || output_tokens.is_none()
                    || cache_creation.is_none()
                    || cache_read.is_none()
                    || facturable.is_none())
            {
                return Err(StoreError::Corrupt("coût attesté incomplet"));
            }
            costs.push(CoutMissionAgent {
                agent,
                from_secs,
                to_secs,
                attested,
                turns: turns.map(|value| value as u64),
                input_tokens: input_tokens.map(|value| value as u64),
                output_tokens: output_tokens.map(|value| value as u64),
                cache_creation_input_tokens: cache_creation.map(|value| value as u64),
                cache_read_input_tokens: cache_read.map(|value| value as u64),
                facturable_tokens: facturable.map(|value| value as u64),
            });
        }
        Ok(costs)
    }

    /// Relit exclusivement les liens d'arbitrage déclarés à la création des
    /// délégations.
    ///
    /// Le document canonique de la délégation reste l'unique autorité : cette
    /// projection ne rapproche ni titres, ni textes, ni identifiants voisins.
    /// Une délégation sans `constat_id` ne produit donc aucune ligne.
    pub fn delegation_arbitration_links(&self) -> Result<Vec<LienArbitrage>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, objective_id, state, payload_json
                 FROM delegations ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Vec<u8>>(3)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        let mut links = Vec::new();
        for row in rows {
            let (id, objective_id, state, payload) = row.map_err(StoreError::Sql)?;
            let delegation: Delegation =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if delegation.id.to_string() != id
                || delegation.objectif_id.to_string() != objective_id
                || delegation.etat != parse_delegation_state(&state)?
            {
                return Err(StoreError::Corrupt(
                    "délégation d'arbitrage et index SQLite divergents",
                ));
            }
            if let Some(link) = delegation.lien_arbitrage().map_err(StoreError::Domain)? {
                links.push(link);
            }
        }
        Ok(links)
    }

    /// Relit un reçu de mutation avant tout nouvel effet. Une génération de
    /// claim plus récente ne modifie que l'enveloppe de lease ; le payload
    /// métier et ses identifiants restent ceux du premier commit durable.
    pub fn replay_guichet_mutation(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
    ) -> Result<Option<StoredGuichetReply>, StoreError> {
        if canonical.issuer_scope != claim.issuer_scope
            || canonical.request_id != claim.request_id
            || !matches!(
                canonical.request.operation(),
                OperationGuichet::Delegate
                    | OperationGuichet::RegistreAdd
                    | OperationGuichet::ObjectiveClose
            )
        {
            return Err(StoreError::Invalid("claim et mutation divergents"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let Some(mut reception) =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
        else {
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(None);
        };
        if reception.canonical_request_bytes != claim.canonical_request
            || reception.operation != canonical.request.operation()
        {
            return Err(StoreError::EnvelopeMismatch);
        }
        if claim.claim_generation == reception.claim_generation
            && claim.claim_token == reception.claim_token
        {
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(Some(StoredGuichetReply {
                reception,
                correlation: None,
                replayed: true,
            }));
        }
        if claim.claim_generation <= reception.claim_generation {
            return Err(StoreError::Conflict(
                "claim de mutation obsolète ou divergent",
            ));
        }
        let (reply_bytes, reply, stored_response_message_id) =
            reclaim_mutation_reply_bytes(claim, &reception.reply_bytes)
                .map_err(|_| StoreError::Corrupt("mutation durable invalide"))?;
        if reply.operation() != reception.operation
            || reply
                .objective_id()
                .map_err(|_| StoreError::Corrupt("objectif de mutation invalide"))?
                != reception.objective_id
            || reply
                .delegation_id()
                .map_err(|_| StoreError::Corrupt("délégation de mutation invalide"))?
                != reception.delegation_id
            || reply
                .decision_id()
                .map_err(|_| StoreError::Corrupt("décision de mutation invalide"))?
                != reception.decision_id
            || stored_response_message_id != reception.response_message_id
        {
            return Err(StoreError::Corrupt("mutation et reçu divergents"));
        }
        let changed = tx
            .execute(
                "UPDATE guichet_receptions\n\
                 SET claim_generation = ?1, claim_token = ?2, reply_bytes = ?3\n\
                 WHERE issuer_scope = ?4 AND request_id = ?5\n\
                   AND claim_generation = ?6 AND claim_token = ?7",
                params![
                    i64::try_from(claim.claim_generation)
                        .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                    claim.claim_token,
                    reply_bytes,
                    canonical.issuer_scope,
                    canonical.request_id,
                    i64::try_from(reception.claim_generation)
                        .map_err(|_| StoreError::Corrupt("génération de reçu invalide"))?,
                    reception.claim_token,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "reçu de mutation modifié concurremment",
            ));
        }
        reception = load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
            .ok_or(StoreError::Corrupt("mutation absente après mise à jour"))?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(Some(StoredGuichetReply {
            reception,
            correlation: None,
            replayed: true,
        }))
    }

    /// Persiste le reçu terminal d'une mutation déjà appliquée par le service
    /// partagé. Les opérations rejouables (`delegate`, journal append et
    /// clôture à clé stable) rendent sûr le crash entre effet et reçu.
    pub fn persist_guichet_mutation(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
        response_message_id: &str,
        now: i64,
        reply: &MutationReply,
    ) -> Result<StoredGuichetReply, StoreError> {
        if now <= 0 || response_message_id.trim().is_empty() {
            return Err(StoreError::Invalid("réponse de mutation incomplète"));
        }
        if let Some(stored) = self.replay_guichet_mutation(claim, canonical)? {
            return Ok(stored);
        }
        if reply.operation() != canonical.request.operation() {
            return Err(StoreError::Invalid("mutation et requête divergentes"));
        }
        let objective_id = reply
            .objective_id()
            .map_err(|_| StoreError::Invalid("objectif de mutation invalide"))?;
        let delegation_id = reply
            .delegation_id()
            .map_err(|_| StoreError::Invalid("délégation de mutation invalide"))?;
        let decision_id = reply
            .decision_id()
            .map_err(|_| StoreError::Invalid("décision de mutation invalide"))?;
        let reply_bytes = mutation_reply_bytes(claim, response_message_id, reply)
            .map_err(|_| StoreError::Invalid("mutation non sérialisable"))?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let inserted = tx
            .execute(
                "INSERT INTO guichet_receptions(\n\
                     issuer_scope, request_id, operation, canonical_request_bytes,\n\
                     objective_id, delegation_id, delivery_hash, in_reply_to,\n\
                     response_message_id, outcome, reply_bytes, decision_id, processed_at,\n\
                     claim_generation, claim_token\n\
                 ) VALUES (?1,?2,?3,?4,?5,?6,NULL,?7,?8,'accepted',?9,?10,?11,?12,?13)",
                params![
                    canonical.issuer_scope,
                    canonical.request_id,
                    operation_name(reply.operation()),
                    claim.canonical_request,
                    objective_id.map(|id| id.to_string()),
                    delegation_id.map(|id| id.to_string()),
                    canonical.request_id,
                    response_message_id,
                    reply_bytes,
                    decision_id.map(|id| id.to_string()),
                    now,
                    i64::try_from(claim.claim_generation)
                        .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                    claim.claim_token,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("reçu de mutation non enregistré"));
        }
        let reception =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
                .ok_or(StoreError::Corrupt("mutation introuvable après insertion"))?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(StoredGuichetReply {
            reception,
            correlation: None,
            replayed: false,
        })
    }

    /// Persiste les octets exacts d'une projection consultative. Une relève
    /// répétée rejoue le reçu ; une nouvelle génération ne change que
    /// l'enveloppe de claim, jamais les faits déjà répondus.
    pub fn persist_guichet_projection(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
        response_message_id: &str,
        now: i64,
        build: impl FnOnce(&GuichetProjectionFacts) -> Result<ProjectionReply, StoreError>,
    ) -> Result<StoredGuichetReply, StoreError> {
        if now <= 0 || response_message_id.trim().is_empty() {
            return Err(StoreError::Invalid("réponse guichet incomplète"));
        }
        if canonical.issuer_scope != claim.issuer_scope
            || canonical.request_id != claim.request_id
            || canonical.request.operation() == OperationGuichet::DeliveryReport
        {
            return Err(StoreError::Invalid("claim et projection divergents"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;

        if let Some(mut reception) =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
        {
            if reception.canonical_request_bytes != claim.canonical_request
                || reception.operation != canonical.request.operation()
            {
                return Err(StoreError::EnvelopeMismatch);
            }
            if claim.claim_generation == reception.claim_generation
                && claim.claim_token == reception.claim_token
            {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(StoredGuichetReply {
                    reception,
                    correlation: None,
                    replayed: true,
                });
            }
            if claim.claim_generation <= reception.claim_generation {
                return Err(StoreError::EnvelopeMismatch);
            }
            let (reply_bytes, stored_projection, stored_response_message_id) =
                reclaim_projection_reply_bytes(claim, &reception.reply_bytes)
                    .map_err(|_| StoreError::Corrupt("projection durable invalide"))?;
            if stored_projection.operation() != reception.operation
                || stored_projection
                    .delegation_id()
                    .map_err(|_| StoreError::Corrupt("référence de projection invalide"))?
                    != reception
                        .delegation_id
                        .ok_or(StoreError::Corrupt("projection sans délégation"))?
                || stored_response_message_id != reception.response_message_id
            {
                return Err(StoreError::Corrupt("projection et reçu divergents"));
            }
            let updated = tx
                .execute(
                    "UPDATE guichet_receptions\n\
                     SET claim_generation = ?1, claim_token = ?2, reply_bytes = ?3\n\
                     WHERE issuer_scope = ?4 AND request_id = ?5\n\
                       AND claim_generation = ?6 AND claim_token = ?7",
                    params![
                        i64::try_from(claim.claim_generation)
                            .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                        claim.claim_token,
                        reply_bytes,
                        canonical.issuer_scope,
                        canonical.request_id,
                        i64::try_from(reception.claim_generation).map_err(|_| {
                            StoreError::Corrupt("génération de reçu hors borne")
                        })?,
                        reception.claim_token,
                    ],
                )
                .map_err(StoreError::Sql)?;
            if updated != 1 {
                return Err(StoreError::Conflict(
                    "reçu de projection modifié concurremment",
                ));
            }
            reception =
                load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?
                    .ok_or(StoreError::Corrupt("projection absente après mise à jour"))?;
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(StoredGuichetReply {
                reception,
                correlation: None,
                replayed: true,
            });
        }

        let delegation_id = match &canonical.request {
            crate::domain::guichet::RequeteGuichet::MissionStatus { delegation_id }
            | crate::domain::guichet::RequeteGuichet::DeadlineQuestion { delegation_id } => {
                *delegation_id
            }
            crate::domain::guichet::RequeteGuichet::DeliveryReport(_)
            | crate::domain::guichet::RequeteGuichet::Delegate(_)
            | crate::domain::guichet::RequeteGuichet::RegistreAdd(_)
            | crate::domain::guichet::RequeteGuichet::ObjectiveClose(_) => {
                return Err(StoreError::Invalid("opération de projection interdite"));
            }
        };
        let facts = load_guichet_projection_facts(&tx, delegation_id)?;
        if facts.delegation.participant != canonical.from {
            return Err(StoreError::Invalid("relations de projection invalides"));
        }
        let projection = build(&facts)?;
        if projection.operation() != canonical.request.operation()
            || projection
                .delegation_id()
                .map_err(|_| StoreError::Invalid("référence de projection invalide"))?
                != delegation_id
        {
            return Err(StoreError::Invalid("projection et requête divergentes"));
        }
        let reply_bytes = projection_reply_bytes(claim, response_message_id, &projection)
            .map_err(|_| StoreError::Invalid("projection guichet non sérialisable"))?;
        let inserted = tx
            .execute(
                "INSERT INTO guichet_receptions(\n\
                     issuer_scope, request_id, operation, canonical_request_bytes,\n\
                     objective_id, delegation_id, delivery_hash, in_reply_to,\n\
                     response_message_id, outcome, reply_bytes, decision_id, processed_at,\n\
                     claim_generation, claim_token\n\
                 ) VALUES (?1,?2,?3,?4,?5,?6,NULL,?7,?8,'accepted',?9,NULL,?10,?11,?12)",
                params![
                    canonical.issuer_scope,
                    canonical.request_id,
                    operation_name(projection.operation()),
                    claim.canonical_request,
                    facts.objective.id.to_string(),
                    delegation_id.to_string(),
                    canonical.request_id,
                    response_message_id,
                    reply_bytes,
                    now,
                    i64::try_from(claim.claim_generation)
                        .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                    claim.claim_token,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("projection guichet non enregistrée"));
        }
        let reception =
            load_guichet_reception(&tx, &canonical.issuer_scope, &canonical.request_id)?.ok_or(
                StoreError::Corrupt("projection introuvable après insertion"),
            )?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(StoredGuichetReply {
            reception,
            correlation: None,
            replayed: false,
        })
    }

    /// Persiste un refus déterministe sans modifier le reçu éventuellement
    /// associé à une autre enveloppe portant la même clé métier. Cette table
    /// séparée est nécessaire au cas `EnvelopeMismatch` : le premier reçu
    /// demeure l'autorité de son enveloppe, le refus devient l'autorité de la
    /// seconde, et les deux rejouent leurs octets propres.
    pub fn persist_guichet_refusal(
        &mut self,
        claim: &GuichetClaim,
        canonical: &RequeteCanonique,
        response_message_id: &str,
        now: i64,
        reason: MotifRefusGreffe,
    ) -> Result<StoredGuichetReply, StoreError> {
        if now <= 0 || response_message_id.trim().is_empty() {
            return Err(StoreError::Invalid("reçu de refus incomplet"));
        }
        if canonical.issuer_scope != claim.issuer_scope || canonical.request_id != claim.request_id
        {
            return Err(StoreError::Invalid("claim et refus divergents"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        if let Some(mut reception) = load_guichet_refusal_reception(
            &tx,
            &canonical.issuer_scope,
            &canonical.request_id,
            &claim.canonical_request,
        )? {
            if claim.claim_generation == reception.claim_generation
                && claim.claim_token == reception.claim_token
            {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(StoredGuichetReply {
                    reception,
                    correlation: None,
                    replayed: true,
                });
            }
            if claim.claim_generation <= reception.claim_generation {
                return Err(StoreError::Conflict("claim de refus obsolète ou divergent"));
            }
            let reply_bytes =
                refusal_reply_bytes(claim, &reception.response_message_id, canonical, reason)
                    .map_err(|_| StoreError::Invalid("réponse de refus non sérialisable"))?;
            let changed = tx
                .execute(
                    "UPDATE guichet_refusal_receptions\n\
                     SET claim_generation = ?1, claim_token = ?2, reply_bytes = ?3\n\
                     WHERE issuer_scope = ?4 AND request_id = ?5 AND canonical_request_bytes = ?6\n\
                       AND claim_generation = ?7 AND claim_token = ?8",
                    params![
                        i64::try_from(claim.claim_generation)
                            .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                        claim.claim_token,
                        reply_bytes,
                        canonical.issuer_scope,
                        canonical.request_id,
                        claim.canonical_request,
                        i64::try_from(reception.claim_generation)
                            .map_err(|_| StoreError::Corrupt("génération de refus invalide"))?,
                        reception.claim_token,
                    ],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("reçu de refus modifié concurremment"));
            }
            reception.claim_generation = claim.claim_generation;
            reception.claim_token = claim.claim_token.clone();
            reception.reply_bytes =
                refusal_reply_bytes(claim, &reception.response_message_id, canonical, reason)
                    .map_err(|_| StoreError::Invalid("réponse de refus non sérialisable"))?;
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(StoredGuichetReply {
                reception,
                correlation: None,
                replayed: true,
            });
        }

        let reply_bytes = refusal_reply_bytes(claim, response_message_id, canonical, reason)
            .map_err(|_| StoreError::Invalid("réponse de refus non sérialisable"))?;
        tx.execute(
            "INSERT INTO guichet_refusal_receptions(\n\
                 issuer_scope, request_id, canonical_request_bytes, operation, reason,\n\
                 response_message_id, reply_bytes, claim_generation, claim_token, processed_at\n\
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![
                canonical.issuer_scope,
                canonical.request_id,
                claim.canonical_request,
                operation_name(canonical.request.operation()),
                refusal_reason_name(reason),
                response_message_id,
                reply_bytes,
                i64::try_from(claim.claim_generation)
                    .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                claim.claim_token,
                now,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(StoredGuichetReply {
            reception: ReceptionGreffe {
                issuer_scope: canonical.issuer_scope.clone(),
                request_id: canonical.request_id.clone(),
                operation: canonical.request.operation(),
                canonical_request_bytes: claim.canonical_request.clone(),
                objective_id: None,
                delegation_id: None,
                delivery_hash: None,
                response_message_id: response_message_id.to_string(),
                claim_generation: claim.claim_generation,
                claim_token: claim.claim_token.clone(),
                reply_bytes,
                issue: IssueGreffe::Refusee,
                decision_id: None,
                processed_at: now,
            },
            correlation: None,
            replayed: false,
        })
    }

    fn remises_locales_for(&self, objective_id: Uuid) -> Result<Vec<RemiseLocale>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT delegation_id, message_id, state, last_issue_json, issue_observed_at\n\
                 FROM delegation_outbox WHERE objective_id = ?1 ORDER BY message_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<Vec<u8>>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (delegation_id, message_id, state, issue, observed_at) =
                row.map_err(StoreError::Sql)?;
            Ok(RemiseLocale {
                delegation_id: Uuid::parse_str(&delegation_id)
                    .map_err(|_| StoreError::Corrupt("delegation_id outbox invalide"))?,
                message_id: Uuid::parse_str(&message_id)
                    .map_err(|_| StoreError::Corrupt("message_id outbox invalide"))?,
                state: parse_outbox_state(&state)?,
                issue: issue
                    .map(|bytes| serde_json::from_slice(&bytes).map_err(StoreError::Json))
                    .transpose()?,
                observed_at,
            })
        })
        .collect()
    }

    /// Applique une décision locale explicite et son éventuelle mise à jour
    /// d'objectif dans une unique transaction. La décision est l'audit de
    /// l'effet : aucun des deux n'est durable seul.
    pub fn apply_objective_decision(
        &mut self,
        decision: &DecisionCoordination,
        updated_objective: Option<&ObjectifCoordonne>,
        expected_state: EtatObjectif,
    ) -> Result<(), StoreError> {
        decision.verifier().map_err(StoreError::Domain)?;
        if decision.etat != EtatDecision::Appliquee {
            return Err(StoreError::Invalid("décision non appliquée"));
        }
        if updated_objective.is_some_and(|objective| objective.id != decision.objectif_id) {
            return Err(StoreError::Invalid("décision et objectif divergents"));
        }
        let decision_json = serde_json::to_vec(decision).map_err(StoreError::Json)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored: Option<(String, Vec<u8>)> = tx
            .query_row(
                "SELECT state, payload_json FROM objectives WHERE id = ?1",
                [decision.objectif_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((stored_state, stored_payload)) = stored else {
            return Err(StoreError::NotFound("objectif absent"));
        };
        let stored_objective: ObjectifCoordonne =
            serde_json::from_slice(&stored_payload).map_err(StoreError::Json)?;
        let indexed_state = parse_objective_state(&stored_state)?;
        if stored_objective.id != decision.objectif_id || stored_objective.etat != indexed_state {
            return Err(StoreError::Corrupt("objectif et index SQLite divergents"));
        }
        if indexed_state == EtatObjectif::Clos {
            return Err(StoreError::Invalid("objectif déjà clos"));
        }
        if indexed_state != expected_state {
            return Err(StoreError::Conflict("objectif modifié concurremment"));
        }
        if decision.kind == TypeDecision::Cloturer
            || updated_objective.is_some_and(|objective| objective.etat == EtatObjectif::Clos)
        {
            return Err(StoreError::Invalid("clôture réservée à close_objective"));
        }
        if let Some(objective) = updated_objective {
            let payload = serde_json::to_vec(objective).map_err(StoreError::Json)?;
            let changed = tx
                .execute(
                    "UPDATE objectives SET state = ?1, payload_json = ?2\n\
                     WHERE id = ?3 AND state = ?4",
                    params![
                        objective_state_name(objective.etat),
                        payload,
                        objective.id.to_string(),
                        objective_state_name(expected_state),
                    ],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("objectif modifié concurremment"));
            }
        }
        let inserted = tx
            .execute(
                "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    decision.id.to_string(),
                    decision.objectif_id.to_string(),
                    decision_state_name(decision.etat),
                    decision_json,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("décision non enregistrée"));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Relit puis clôt l'objectif dans une transaction immédiate. La lecture
    /// interne interdit qu'une issue terminale validée entre une ancienne
    /// projection CLI et l'écriture de clôture.
    pub fn close_objective(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
    ) -> Result<DecisionCoordination, StoreError> {
        self.close_objective_observed(objective_id, reason, now, |_| Ok(()))
    }

    /// Réserve une plage de ressource pour un objectif (politique 31).
    /// Comparaison exacte de noms. Refuse si un *autre* objectif détient déjà
    /// la même ressource active. Rejouer pour le même titulaire est idempotent.
    pub fn reserve_resource_range(
        &mut self,
        resource: &str,
        objective_id: Uuid,
        now: i64,
    ) -> Result<ResourceRangeReservation, StoreError> {
        if resource.is_empty() || now <= 0 {
            return Err(StoreError::Invalid("réservation de plage invalide"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let objective_state: Option<String> = tx
            .query_row(
                "SELECT state FROM objectives WHERE id = ?1",
                [objective_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some(state) = objective_state else {
            return Err(StoreError::NotFound("objectif absent"));
        };
        if parse_objective_state(&state)? == EtatObjectif::Clos {
            return Err(StoreError::Invalid("objectif déjà clos"));
        }
        let existing: Option<(String, i64)> = tx
            .query_row(
                "SELECT objective_id, reserved_at FROM resource_range_reservations\n\
                 WHERE resource_name = ?1",
                [resource],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some((holder_raw, reserved_at)) = existing {
            let holder = parse_uuid(&holder_raw)?;
            if holder == objective_id {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(ResourceRangeReservation {
                    resource: resource.to_string(),
                    objective_id,
                    reserved_at,
                });
            }
            return Err(StoreError::ResourceHeld {
                resource: resource.to_string(),
                holder_objective_id: holder,
            });
        }
        let inserted = tx
            .execute(
                "INSERT INTO resource_range_reservations(resource_name, objective_id, reserved_at)\n\
                 VALUES (?1, ?2, ?3)",
                params![resource, objective_id.to_string(), now],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("réservation de plage non enregistrée"));
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(ResourceRangeReservation {
            resource: resource.to_string(),
            objective_id,
            reserved_at: now,
        })
    }

    /// Liste les réservations de plage actives, triées par nom exact.
    pub fn list_resource_ranges(&self) -> Result<Vec<ResourceRangeReservation>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT resource_name, objective_id, reserved_at\n\
                 FROM resource_range_reservations\n\
                 ORDER BY resource_name",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        let mut reservations = Vec::new();
        for row in rows {
            let (resource, objective_raw, reserved_at) = row.map_err(StoreError::Sql)?;
            reservations.push(ResourceRangeReservation {
                resource,
                objective_id: parse_uuid(&objective_raw)?,
                reserved_at,
            });
        }
        Ok(reservations)
    }

    pub fn insert_routine(&mut self, routine: &Routine) -> Result<(), StoreError> {
        let (suite_kind, suite_objective_id) = suite_columns(&routine.suite);
        let inserted = self
            .connection
            .execute(
                "INSERT INTO routines(\n\
                     id, goal, participant, period_secs, suite_kind, suite_objective_id,\n\
                     depends_on_json, references_json, template_hash, state,\n\
                     proposed_at, approved_at, paused_at, last_bucket\n\
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    routine.id.to_string(),
                    routine.goal,
                    routine.participant,
                    routine.period_secs,
                    suite_kind,
                    suite_objective_id,
                    serde_json::to_string(&routine.depends_on)
                        .map_err(|_| StoreError::Corrupt("depends_on JSON"))?,
                    serde_json::to_string(&routine.references)
                        .map_err(|_| StoreError::Corrupt("references JSON"))?,
                    routine.template_hash,
                    routine_state_name(routine.state),
                    routine.proposed_at,
                    routine.approved_at,
                    routine.paused_at,
                    routine.last_bucket,
                ],
            )
            .map_err(StoreError::Sql)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("routine non enregistrée"));
        }
        Ok(())
    }

    pub fn update_routine(&mut self, routine: &Routine) -> Result<(), StoreError> {
        let (suite_kind, suite_objective_id) = suite_columns(&routine.suite);
        let updated = self
            .connection
            .execute(
                "UPDATE routines SET\n\
                     goal = ?1, participant = ?2, period_secs = ?3,\n\
                     suite_kind = ?4, suite_objective_id = ?5,\n\
                     depends_on_json = ?6, references_json = ?7, template_hash = ?8,\n\
                     state = ?9, proposed_at = ?10, approved_at = ?11, paused_at = ?12,\n\
                     last_bucket = ?13\n\
                 WHERE id = ?14",
                params![
                    routine.goal,
                    routine.participant,
                    routine.period_secs,
                    suite_kind,
                    suite_objective_id,
                    serde_json::to_string(&routine.depends_on)
                        .map_err(|_| StoreError::Corrupt("depends_on JSON"))?,
                    serde_json::to_string(&routine.references)
                        .map_err(|_| StoreError::Corrupt("references JSON"))?,
                    routine.template_hash,
                    routine_state_name(routine.state),
                    routine.proposed_at,
                    routine.approved_at,
                    routine.paused_at,
                    routine.last_bucket,
                    routine.id.to_string(),
                ],
            )
            .map_err(StoreError::Sql)?;
        if updated != 1 {
            return Err(StoreError::NotFound("routine absente"));
        }
        Ok(())
    }

    /// Activation atomique : `proposed` → `active` sous garde du hash scellé.
    /// `updated != 1` = course ou hash divergent (ADR 011 compare-and-swap).
    pub fn activate_routine_cas(
        &mut self,
        routine_id: Uuid,
        template_hash: &[u8],
        approved_at: i64,
        last_bucket: i64,
    ) -> Result<Routine, StoreError> {
        let updated = self
            .connection
            .execute(
                "UPDATE routines SET state = 'active', approved_at = ?1, last_bucket = ?2,\n\
                     paused_at = NULL\n\
                 WHERE id = ?3 AND state = 'proposed' AND template_hash = ?4",
                params![
                    approved_at,
                    last_bucket,
                    routine_id.to_string(),
                    template_hash,
                ],
            )
            .map_err(StoreError::Sql)?;
        if updated != 1 {
            return Err(StoreError::Conflict(
                "activation routine refusée (état ou hash)",
            ));
        }
        self.load_routine(routine_id)?
            .ok_or(StoreError::NotFound("routine absente après activation"))
    }

    /// Clôture les occurrences ouvertes liées à un objectif (même transaction
    /// que la clôture d'objectif). Idempotent : 0 ligne = pas d'occurrence.
    pub fn terminate_occurrences_for_objective_tx(
        tx: &Transaction<'_>,
        objective_id: Uuid,
    ) -> Result<usize, StoreError> {
        let changed = tx
            .execute(
                "UPDATE routine_occurrences SET state = 'terminee'\n\
                 WHERE objective_id = ?1 AND state = 'ouverte'",
                [objective_id.to_string()],
            )
            .map_err(StoreError::Sql)?;
        Ok(changed)
    }

    /// Rattrapage : toute occurrence encore `ouverte` dont l'objectif est clos
    /// passe à `terminee`. Appelé en tête de chaque tick routines.
    pub fn terminate_occurrences_with_closed_objectives(&mut self) -> Result<usize, StoreError> {
        let changed = self
            .connection
            .execute(
                "UPDATE routine_occurrences\n\
                 SET state = 'terminee'\n\
                 WHERE state = 'ouverte'\n\
                   AND objective_id IS NOT NULL\n\
                   AND EXISTS (\n\
                       SELECT 1 FROM objectives\n\
                       WHERE objectives.id = routine_occurrences.objective_id\n\
                         AND objectives.state = 'clos'\n\
                   )",
                [],
            )
            .map_err(StoreError::Sql)?;
        Ok(changed)
    }

    /// Propriété : une occurrence `ouverte` ne doit jamais attester un mandat
    /// **mort** au sens du domaine (`EtatDelegation::est_mandat_mort`).
    /// Rétracte en `sautee/mandat_plus_vivant` pour que `has_open` retombe.
    ///
    /// La décision est portée par le domaine (match exhaustif), pas par une
    /// énumération SQL recopiée : un état terminal ajouté demain (ex.
    /// `soldee_par_cloture`) doit être classé dans `est_mandat_mort`, sinon
    /// la compilation casse. `Terminee` n'est pas un cadavre — mission
    /// accomplie, l'occurrence attend `terminate_occurrences_with_closed_objectives`.
    pub fn retract_occurrences_with_dead_mandates(&mut self) -> Result<usize, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT o.routine_id, o.bucket, o.delegation_id, d.state\n\
                 FROM routine_occurrences o\n\
                 LEFT JOIN delegations d ON d.id = o.delegation_id\n\
                 WHERE o.state = 'ouverte'",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(StoreError::Sql)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sql)?;
        drop(statement);

        let mut changed = 0usize;
        for (routine_id, bucket, delegation_id, delegation_state) in rows {
            let mort = match (delegation_id.as_deref(), delegation_state.as_deref()) {
                (None, _) => true,
                (Some(_), None) => true,
                (Some(_), Some(state)) => parse_delegation_state(state)?.est_mandat_mort(),
            };
            if !mort {
                continue;
            }
            changed += self
                .connection
                .execute(
                    "UPDATE routine_occurrences\n\
                     SET state = 'sautee', reason = 'mandat_plus_vivant'\n\
                     WHERE routine_id = ?1 AND bucket = ?2 AND state = 'ouverte'",
                    rusqlite::params![routine_id, bucket],
                )
                .map_err(StoreError::Sql)?;
        }
        Ok(changed)
    }

    pub fn load_routine(&self, routine_id: Uuid) -> Result<Option<Routine>, StoreError> {
        self.connection
            .query_row(
                "SELECT id, goal, participant, period_secs, suite_kind, suite_objective_id,\n\
                        depends_on_json, references_json, template_hash, state,\n\
                        proposed_at, approved_at, paused_at, last_bucket\n\
                 FROM routines WHERE id = ?1",
                [routine_id.to_string()],
                map_routine_row,
            )
            .optional()
            .map_err(StoreError::Sql)?
            .map(row_to_routine)
            .transpose()
    }

    pub fn list_routines(&self, state: Option<EtatRoutine>) -> Result<Vec<Routine>, StoreError> {
        let mut statement = if state.is_some() {
            self.connection
                .prepare(
                    "SELECT id, goal, participant, period_secs, suite_kind, suite_objective_id,\n\
                            depends_on_json, references_json, template_hash, state,\n\
                            proposed_at, approved_at, paused_at, last_bucket\n\
                     FROM routines WHERE state = ?1 ORDER BY proposed_at, id",
                )
                .map_err(StoreError::Sql)?
        } else {
            self.connection
                .prepare(
                    "SELECT id, goal, participant, period_secs, suite_kind, suite_objective_id,\n\
                            depends_on_json, references_json, template_hash, state,\n\
                            proposed_at, approved_at, paused_at, last_bucket\n\
                     FROM routines ORDER BY proposed_at, id",
                )
                .map_err(StoreError::Sql)?
        };
        let mapped = if let Some(filter) = state {
            statement
                .query_map([routine_state_name(filter)], map_routine_row)
                .map_err(StoreError::Sql)?
        } else {
            statement
                .query_map([], map_routine_row)
                .map_err(StoreError::Sql)?
        };
        let mut routines = Vec::new();
        for row in mapped {
            routines.push(row_to_routine(row.map_err(StoreError::Sql)?)?);
        }
        Ok(routines)
    }

    pub fn insert_occurrence(&mut self, occurrence: &RoutineOccurrence) -> Result<(), StoreError> {
        let inserted = self
            .connection
            .execute(
                "INSERT INTO routine_occurrences(\n\
                     routine_id, bucket, state, reason, objective_id, delegation_id, created_at\n\
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    occurrence.routine_id.to_string(),
                    occurrence.bucket,
                    occurrence_state_name(occurrence.state),
                    occurrence.reason,
                    occurrence.objective_id.map(|id| id.to_string()),
                    occurrence.delegation_id.map(|id| id.to_string()),
                    occurrence.created_at,
                ],
            )
            .map_err(|error| {
                if matches!(
                    error.sqlite_error_code(),
                    Some(ErrorCode::ConstraintViolation)
                ) {
                    StoreError::Conflict("occurrence déjà présente")
                } else {
                    StoreError::Sql(error)
                }
            })?;
        if inserted != 1 {
            return Err(StoreError::Conflict("occurrence non enregistrée"));
        }
        Ok(())
    }

    pub fn load_occurrence(
        &self,
        routine_id: Uuid,
        bucket: i64,
    ) -> Result<Option<RoutineOccurrence>, StoreError> {
        self.connection
            .query_row(
                "SELECT routine_id, bucket, state, reason, objective_id, delegation_id, created_at\n\
                 FROM routine_occurrences WHERE routine_id = ?1 AND bucket = ?2",
                params![routine_id.to_string(), bucket],
                map_occurrence_row,
            )
            .optional()
            .map_err(StoreError::Sql)?
            .map(row_to_occurrence)
            .transpose()
    }

    pub fn open_occurrence_for_routine(
        &self,
        routine_id: Uuid,
    ) -> Result<Option<RoutineOccurrence>, StoreError> {
        self.connection
            .query_row(
                "SELECT routine_id, bucket, state, reason, objective_id, delegation_id, created_at\n\
                 FROM routine_occurrences\n\
                 WHERE routine_id = ?1 AND state = 'ouverte'\n\
                 ORDER BY bucket DESC LIMIT 1",
                [routine_id.to_string()],
                map_occurrence_row,
            )
            .optional()
            .map_err(StoreError::Sql)?
            .map(row_to_occurrence)
            .transpose()
    }

    pub fn recent_occurrences(
        &self,
        routine_id: Uuid,
        state: EtatOccurrence,
        limit: i64,
    ) -> Result<Vec<RoutineOccurrence>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT routine_id, bucket, state, reason, objective_id, delegation_id, created_at\n\
                 FROM routine_occurrences\n\
                 WHERE routine_id = ?1 AND state = ?2\n\
                 ORDER BY bucket DESC LIMIT ?3",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map(
                params![routine_id.to_string(), occurrence_state_name(state), limit],
                map_occurrence_row,
            )
            .map_err(StoreError::Sql)?;
        let mut occurrences = Vec::new();
        for row in rows {
            occurrences.push(row_to_occurrence(row.map_err(StoreError::Sql)?)?);
        }
        Ok(occurrences)
    }

    /// Variante à observateur utilisée par les tests de crash transactionnel.
    /// L'observateur ne fait jamais partie du chemin de production normal.
    #[doc(hidden)]
    pub fn close_objective_observed<F>(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
        observer: F,
    ) -> Result<DecisionCoordination, StoreError>
    where
        F: FnMut(ObjectiveClosureCommitPhase) -> Result<(), StoreError>,
    {
        self.close_objective_observed_with_costs(objective_id, reason, now, None, observer)
    }

    /// Clôture en portant les coûts attestés (ou inconnus) dans la même
    /// transaction. Un agent absent des faits runtime reste `inconnu`.
    pub fn close_objective_with_costs(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
        costs: Vec<CoutMissionAgent>,
    ) -> Result<DecisionCoordination, StoreError> {
        self.close_objective_observed_with_costs(objective_id, reason, now, Some(costs), |_| Ok(()))
    }

    #[doc(hidden)]
    pub fn close_objective_observed_with_costs<F>(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
        costs: Option<Vec<CoutMissionAgent>>,
        observer: F,
    ) -> Result<DecisionCoordination, StoreError>
    where
        F: FnMut(ObjectiveClosureCommitPhase) -> Result<(), StoreError>,
    {
        self.close_objective_observed_with_costs_and_id(
            objective_id,
            reason,
            now,
            costs,
            None,
            observer,
        )
        .map(|(decision, _)| decision)
    }

    /// Clôture rejouable réservée au guichet. La clé ne devient jamais une
    /// autorité externe : elle sert uniquement à dériver l'identité stable de
    /// la décision dans la portée durable du store central.
    pub fn close_objective_idempotent_with_costs(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
        costs: Vec<CoutMissionAgent>,
        idempotency_key: &str,
    ) -> Result<(DecisionCoordination, bool), StoreError> {
        if idempotency_key.trim().is_empty() || idempotency_key.len() > 128 {
            return Err(StoreError::Invalid("clé de clôture invalide"));
        }
        let decision_id = identifiant_deterministe(
            b"maicie-guichet-objective-close-v1",
            &[self.issuer_scope.as_bytes(), idempotency_key.as_bytes()],
        );
        self.close_objective_observed_with_costs_and_id(
            objective_id,
            reason,
            now,
            Some(costs),
            Some(decision_id),
            |_| Ok(()),
        )
    }

    fn close_objective_observed_with_costs_and_id<F>(
        &mut self,
        objective_id: Uuid,
        reason: &str,
        now: i64,
        costs: Option<Vec<CoutMissionAgent>>,
        stable_decision_id: Option<Uuid>,
        mut observer: F,
    ) -> Result<(DecisionCoordination, bool), StoreError>
    where
        F: FnMut(ObjectiveClosureCommitPhase) -> Result<(), StoreError>,
    {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        if let Some(decision_id) = stable_decision_id {
            let existing: Option<(String, Vec<u8>)> = tx
                .query_row(
                    "SELECT state, payload_json FROM coordination_decisions WHERE id = ?1",
                    [decision_id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(StoreError::Sql)?;
            if let Some((state, payload)) = existing {
                let decision: DecisionCoordination =
                    serde_json::from_slice(&payload).map_err(StoreError::Json)?;
                let objective_state: Option<String> = tx
                    .query_row(
                        "SELECT state FROM objectives WHERE id = ?1",
                        [objective_id.to_string()],
                        |row| row.get(0),
                    )
                    .optional()
                    .map_err(StoreError::Sql)?;
                if decision.id != decision_id
                    || decision.objectif_id != objective_id
                    || decision.kind != TypeDecision::Cloturer
                    || decision.etat != EtatDecision::Appliquee
                    || decision.motif != reason
                    || parse_decision_state(&state)? != decision.etat
                    || objective_state
                        .as_deref()
                        .map(parse_objective_state)
                        .transpose()?
                        != Some(EtatObjectif::Clos)
                {
                    return Err(StoreError::EnvelopeMismatch);
                }
                tx.commit().map_err(StoreError::Sql)?;
                return Ok((decision, true));
            }
        }
        let stored: Option<(String, Vec<u8>)> = tx
            .query_row(
                "SELECT state, payload_json FROM objectives WHERE id = ?1",
                [objective_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((stored_state, stored_payload)) = stored else {
            return Err(StoreError::NotFound("objectif absent"));
        };
        let mut objective: ObjectifCoordonne =
            serde_json::from_slice(&stored_payload).map_err(StoreError::Json)?;
        let expected_state = parse_objective_state(&stored_state)?;
        if objective.id != objective_id || objective.etat != expected_state {
            return Err(StoreError::Corrupt("objectif et index SQLite divergents"));
        }
        objective
            .clore(now)
            .map_err(|_| StoreError::Invalid("objectif déjà clos"))?;
        let decision = DecisionCoordination {
            id: stable_decision_id.unwrap_or_else(Uuid::new_v4),
            objectif_id: objective_id,
            kind: TypeDecision::Cloturer,
            proposee_par: "maicie".to_string(),
            etat: EtatDecision::Appliquee,
            motif: reason.to_string(),
        };
        decision.verifier().map_err(StoreError::Domain)?;
        persist_objective_closure(
            &tx,
            &objective,
            expected_state,
            &decision,
            now,
            costs.as_deref(),
            &mut observer,
            self.control,
        )?;
        observer(ObjectiveClosureCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(ObjectiveClosureCommitPhase::AfterCommit)?;
        Ok((decision, false))
    }

    /// Relit une proposition d'activation durable pour une commande CLI
    /// séparée. Les bytes restent ceux déjà enregistrés dans l'approbation ;
    /// aucun SpawnOrder n'est reconstruit à cette frontière.
    pub fn activation_proposal(
        &self,
        approval_id: Uuid,
    ) -> Result<(DecisionCoordination, ApprobationActivation), StoreError> {
        let row: Option<StoredActivationProposal> = self
            .connection
            .query_row(
                "SELECT a.payload_json, d.payload_json, a.state, d.state\n\
                 FROM activation_approvals a\n\
                 JOIN coordination_decisions d ON d.id = a.decision_id\n\
                 WHERE a.id = ?1",
                [approval_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((approval_json, decision_json, approval_state, decision_state)) = row else {
            return Err(StoreError::NotFound("approbation inconnue"));
        };
        let approval: ApprobationActivation =
            serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
        let decision: DecisionCoordination =
            serde_json::from_slice(&decision_json).map_err(StoreError::Json)?;
        if approval.id != approval_id || approval.objective_id != decision.objectif_id {
            return Err(StoreError::Corrupt("proposition d'activation divergente"));
        }
        match (approval_state.as_str(), decision_state.as_str()) {
            ("proposed", "proposed") | ("approved", "approved") => Ok((decision, approval)),
            ("consumed", _) => Err(StoreError::Invalid("approbation déjà consommée")),
            _ => Err(StoreError::Corrupt("état de proposition divergent")),
        }
    }

    /// Persists a proposed project profile before host resolution or local approval.
    pub fn save_project_profile(&mut self, profile: &ProjectProfile) -> Result<(), StoreError> {
        if profile.status != ProjectProfileStatus::Proposed {
            return Err(StoreError::Invalid("project profile must be proposed"));
        }
        let payload = serde_json::to_vec(profile).map_err(StoreError::Json)?;
        let changed = self
            .connection
            .execute(
                "INSERT INTO project_profiles(
                     profile_id, project_id, state, payload_json, approval_json, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
                params![
                    profile.profile_id,
                    profile.project_id,
                    project_profile_status_text(profile.status),
                    payload,
                    profile.updated_at,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("project profile already exists"));
        }
        Ok(())
    }

    // Replaces an obsolete profile proposal atomically. The former local approval
    // is removed, so a fresh host resolution and local confirmation are required.
    pub fn replace_project_profile(&mut self, profile: &ProjectProfile) -> Result<(), StoreError> {
        if profile.status != ProjectProfileStatus::Proposed {
            return Err(StoreError::Invalid(
                "project profile replacement must be proposed",
            ));
        }
        let payload = serde_json::to_vec(profile).map_err(StoreError::Json)?;
        let changed = self.connection.execute(
            "UPDATE project_profiles
             SET project_id = ?1, state = ?2, payload_json = ?3, approval_json = NULL, updated_at = ?4
             WHERE profile_id = ?5",
            params![
                profile.project_id,
                project_profile_status_text(profile.status),
                payload,
                profile.updated_at,
                profile.profile_id,
            ],
        ).map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::NotFound("project profile missing"));
        }
        Ok(())
    }

    pub fn project_profile(&self, profile_id: &str) -> Result<Option<ProjectProfile>, StoreError> {
        let stored: Option<(String, Vec<u8>)> = self
            .connection
            .query_row(
                "SELECT state, payload_json FROM project_profiles WHERE profile_id = ?1",
                [profile_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        stored
            .map(|(state, payload)| {
                let profile: ProjectProfile =
                    serde_json::from_slice(&payload).map_err(StoreError::Json)?;
                if project_profile_status_text(profile.status) != state {
                    return Err(StoreError::Corrupt("project profile state mismatch"));
                }
                Ok(profile)
            })
            .transpose()
    }

    pub fn record_project_profile_resolution(
        &mut self,
        profile_id: &str,
        resolved: ResolvedProjectProfile,
        now: i64,
    ) -> Result<ProjectProfile, StoreError> {
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let stored: Option<(String, Vec<u8>)> = tx
            .query_row(
                "SELECT state, payload_json FROM project_profiles WHERE profile_id = ?1",
                [profile_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((state, payload)) = stored else {
            return Err(StoreError::NotFound("project profile missing"));
        };
        let mut profile: ProjectProfile =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if state != "proposed" || profile.status != ProjectProfileStatus::Proposed {
            return Err(StoreError::Conflict("project profile is not resolvable"));
        }
        profile
            .record_resolution(resolved, now)
            .map_err(StoreError::Domain)?;
        let profile_payload = serde_json::to_vec(&profile).map_err(StoreError::Json)?;
        let changed = tx.execute(
            "UPDATE project_profiles SET state = ?1, payload_json = ?2, updated_at = ?3 WHERE profile_id = ?4 AND state = ?5",
            params![project_profile_status_text(profile.status), profile_payload, profile.updated_at, profile.profile_id, "proposed"],
        ).map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("project profile changed concurrently"));
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(profile)
    }

    pub fn approve_project_profile(
        &mut self,
        profile_id: &str,
        profile_digest: String,
        now: i64,
    ) -> Result<(ProjectProfile, ProjectProfileApproval), StoreError> {
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let stored: Option<(String, Vec<u8>)> = tx
            .query_row(
                "SELECT state, payload_json FROM project_profiles WHERE profile_id = ?1",
                [profile_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((state, payload)) = stored else {
            return Err(StoreError::NotFound("project profile missing"));
        };
        let mut profile: ProjectProfile =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if state != "resolved" || profile.status != ProjectProfileStatus::Resolved {
            return Err(StoreError::Conflict("project profile is not approvable"));
        }
        let approval = profile
            .approve(profile_digest, now)
            .map_err(StoreError::Domain)?;
        let profile_payload = serde_json::to_vec(&profile).map_err(StoreError::Json)?;
        let approval_payload = serde_json::to_vec(&approval).map_err(StoreError::Json)?;
        let changed = tx
            .execute(
                "UPDATE project_profiles
                 SET state = ?1, payload_json = ?2, approval_json = ?3, updated_at = ?4
                 WHERE profile_id = ?5 AND state = ?6",
                params![
                    project_profile_status_text(profile.status),
                    profile_payload,
                    approval_payload,
                    profile.updated_at,
                    profile.profile_id,
                    "resolved",
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("project profile changed concurrently"));
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok((profile, approval))
    }

    fn delegations_for(
        &self,
        objective_id: Uuid,
    ) -> Result<Vec<crate::domain::Delegation>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM delegations\n\
                 WHERE objective_id = ?1 ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let delegation = serde_json::from_slice::<crate::domain::Delegation>(&payload)
                .map_err(StoreError::Json)?;
            delegation.verifier().map_err(StoreError::Domain)?;
            if delegation.id.to_string() != id
                || delegation.objectif_id != objective_id
                || delegation.etat != parse_delegation_state(&state)?
            {
                return Err(StoreError::Corrupt("délégation et index SQLite divergents"));
            }
            Ok(delegation)
        })
        .collect()
    }

    fn decisions_for(&self, objective_id: Uuid) -> Result<Vec<DecisionCoordination>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, state, payload_json FROM coordination_decisions\n\
                 WHERE objective_id = ?1 ORDER BY id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([objective_id.to_string()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (id, state, payload) = row.map_err(StoreError::Sql)?;
            let decision: DecisionCoordination =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if decision.id.to_string() != id
                || decision.objectif_id != objective_id
                || decision.etat != parse_decision_state(&state)?
            {
                return Err(StoreError::Corrupt("décision et index SQLite divergents"));
            }
            Ok(decision)
        })
        .collect()
    }

    /// Écrit l'agrégat complet dans une seule transaction SQLite.
    pub fn create_prepared_delegation(
        &mut self,
        prepared: &PreparedDelegation,
    ) -> Result<(), StoreError> {
        self.create_prepared_delegation_observed(prepared, |_| Ok(()))
    }

    /// Variante de production instrumentée par les crash-tests réels.
    pub fn create_prepared_delegation_observed(
        &mut self,
        prepared: &PreparedDelegation,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<(), StoreError> {
        prepared.validate().map_err(StoreError::Outbox)?;
        if prepared.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }

        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        // Cette API de bas niveau n'est utilisée que par les harnais de store.
        // Elle reste incapable de fabriquer une origine humaine : son seul
        // permit est explicitement automatique et subit la même vérification
        // au point INSERT que les deux réservations productives.
        let opening_permit = ObjectiveOpeningPermit::auto_generated();
        insert_prepared(&tx, prepared, &opening_permit)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(())
    }

    /// Retrouve objective_id + delegation_id pour une clé d'idempotence, sans
    /// exiger les octets canoniques (adoption d'un mandat orphelin routines).
    /// Joint `delegations` comme les autres lookups : une ligne d'idempotence
    /// orpheline de sa délégation ne doit pas produire une occurrence ouverte
    /// fantôme. Refuse les mandats **terminaux** (`EtatDelegation::est_terminal`)
    /// — clause SQL dérivée du domaine, jamais recopié à la main.
    pub fn lookup_delegate_ids_by_key(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<(Uuid, Uuid)>, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        let terminaux = EtatDelegation::sql_in_clause(EtatDelegation::est_terminal);
        let sql = format!(
            "SELECT i.objective_id, i.delegation_id\n\
             FROM delegate_idempotency i\n\
             JOIN delegations d ON d.id = i.delegation_id\n\
             WHERE i.idempotency_key = ?1\n\
               AND d.state NOT IN ({terminaux})"
        );
        let row = self
            .connection
            .query_row(&sql, [idempotency_key], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .optional()
            .map_err(StoreError::Sql)?;
        row.map(|(objective_raw, delegation_raw)| {
            Ok((parse_uuid(&objective_raw)?, parse_uuid(&delegation_raw)?))
        })
        .transpose()
    }

    /// Buckets portant un mandat `routine:{id}:{bucket}` encore **vivant**
    /// (délégation non terminale), dans `[from_bucket, to_bucket]` inclus.
    /// Sert au rattrapage borné : les orphelins hors `from..=current` doivent
    /// être adoptés avant le saut de `last_bucket`.
    pub fn list_routine_orphan_buckets(
        &self,
        routine_id: Uuid,
        from_bucket: i64,
        to_bucket: i64,
    ) -> Result<Vec<i64>, StoreError> {
        if from_bucket > to_bucket {
            return Ok(Vec::new());
        }
        let prefix = format!("routine:{routine_id}:");
        // UUID hex : pas de `%` / `_` à échapper — pas d'ESCAPE cosmétique.
        let pattern = format!("{prefix}%");
        let terminaux = EtatDelegation::sql_in_clause(EtatDelegation::est_terminal);
        let sql = format!(
            "SELECT i.idempotency_key\n\
             FROM delegate_idempotency i\n\
             JOIN delegations d ON d.id = i.delegation_id\n\
             WHERE i.idempotency_key LIKE ?1\n\
               AND d.state NOT IN ({terminaux})"
        );
        let mut stmt = self.connection.prepare(&sql).map_err(StoreError::Sql)?;
        let keys = stmt
            .query_map([pattern], |row| row.get::<_, String>(0))
            .map_err(StoreError::Sql)?;
        let mut buckets = Vec::new();
        for key in keys {
            let key = key.map_err(StoreError::Sql)?;
            let Some(suffix) = key.strip_prefix(&prefix) else {
                continue;
            };
            let Ok(bucket) = suffix.parse::<i64>() else {
                continue;
            };
            if bucket >= from_bucket && bucket <= to_bucket {
                buckets.push(bucket);
            }
        }
        buckets.sort_unstable();
        buckets.dedup();
        Ok(buckets)
    }

    /// Lit un résultat durable déjà réservé pour une clé de commande. Les
    /// octets canoniques font partie de l'identité : une clé réemployée pour
    /// une autre enveloppe est refusée sans écrire quoi que ce soit.
    pub fn lookup_delegate_replay(
        &self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
    ) -> Result<Option<StoredDelegateResult>, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        let stored = self
            .connection
            .query_row(
                "SELECT i.canonical_request_bytes, i.objective_id, i.delegation_id,\n\
                        i.message_id, i.participant, i.timeout_secs, d.payload_json,\n\
                        i.deadline_contractuelle\n\
                 FROM delegate_idempotency i\n\
                 JOIN delegations d ON d.id = i.delegation_id\n\
                 WHERE i.idempotency_key = ?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        stored
            .map(|stored| decode_delegate_result(stored, canonical_request_bytes))
            .transpose()
    }

    /// Réserve dans la même transaction l'agrégat et son résultat de commande.
    /// Le second lookup protège aussi une course entre le lookup optimiste du
    /// cas d'usage et l'insertion effective.
    pub fn lookup_or_reserve_delegate(
        &mut self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
        prepared: &PreparedDelegation,
        opening_permit: &ObjectiveOpeningPermit,
    ) -> Result<DelegateReservation, StoreError> {
        self.lookup_or_reserve_delegate_observed(
            idempotency_key,
            canonical_request_bytes,
            prepared,
            opening_permit,
            |_| Ok(()),
        )
    }

    /// Variante instrumentable pour les crash-tests à la frontière de commit.
    pub fn lookup_or_reserve_delegate_observed(
        &mut self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
        prepared: &PreparedDelegation,
        opening_permit: &ObjectiveOpeningPermit,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<DelegateReservation, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        prepared.validate().map_err(StoreError::Outbox)?;
        if prepared.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = tx
            .query_row(
                "SELECT i.canonical_request_bytes, i.objective_id, i.delegation_id,\n\
                        i.message_id, i.participant, i.timeout_secs, d.payload_json,\n\
                        i.deadline_contractuelle\n\
                 FROM delegate_idempotency i\n\
                 JOIN delegations d ON d.id = i.delegation_id\n\
                 WHERE i.idempotency_key = ?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(stored) = stored {
            return decode_delegate_result(stored, canonical_request_bytes)
                .map(DelegateReservation::Replay);
        }

        insert_prepared(&tx, prepared, opening_permit)?;
        tx.execute(
            "INSERT INTO delegate_idempotency(\n\
                 idempotency_key, canonical_request_bytes, objective_id, delegation_id,\n\
                 message_id, participant, timeout_secs, deadline_contractuelle\n\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                idempotency_key,
                canonical_request_bytes,
                prepared.objective.id.to_string(),
                prepared.delegation.id.to_string(),
                prepared.outbox.message_id.to_string(),
                prepared.delegation.participant,
                i64::try_from(prepared.outbox.timeout_secs)
                    .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
                prepared.outbox.deadline_contractuelle,
            ],
        )
        .map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(DelegateReservation::Created)
    }

    /// Objectifs présents en base parmi les identifiants fournis (lookup F37).
    pub fn existing_objective_ids(&self, candidates: &[Uuid]) -> Result<Vec<Uuid>, StoreError> {
        let mut found = Vec::new();
        for id in candidates {
            let exists: bool = self
                .connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM objectives WHERE id = ?1)",
                    [id.to_string()],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            if exists {
                found.push(*id);
            }
        }
        Ok(found)
    }

    /// `true` si l'objectif est présent et déjà clos.
    pub fn objective_is_closed(&self, objective_id: Uuid) -> Result<bool, StoreError> {
        let state: Option<String> = self
            .connection
            .query_row(
                "SELECT state FROM objectives WHERE id = ?1",
                [objective_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        Ok(matches!(state.as_deref(), Some("clos")))
    }

    /// Réserve une délégation sans outbox (F37 voie A) + arêtes objectif→objectif.
    pub fn lookup_or_reserve_waiting_delegate(
        &mut self,
        idempotency_key: &str,
        canonical_request_bytes: &[u8],
        objective: &ObjectifCoordonne,
        delegation: &Delegation,
        deferred: &DeferredDispatchParams,
        opening_permit: &ObjectiveOpeningPermit,
    ) -> Result<DelegateReservation, StoreError> {
        validate_delegate_idempotency_key(idempotency_key)?;
        if deferred.issuer_scope != self.issuer_scope {
            return Err(StoreError::Conflict(
                "issuer_scope différent de l'identité durable du store",
            ));
        }
        if delegation.etat != EtatDelegation::EnAttentePrerequis {
            return Err(StoreError::Invalid(
                "délégation d'attente hors EnAttentePrerequis",
            ));
        }
        if objective.depends_on.is_empty() {
            return Err(StoreError::Invalid("attente sans prérequis objectif"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let stored = tx
            .query_row(
                "SELECT i.canonical_request_bytes, i.objective_id, i.delegation_id,\n\
                        i.message_id, i.participant, i.timeout_secs, d.payload_json,\n\
                        i.deadline_contractuelle\n\
                 FROM delegate_idempotency i\n\
                 JOIN delegations d ON d.id = i.delegation_id\n\
                 WHERE i.idempotency_key = ?1",
                [idempotency_key],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, i64>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(stored) = stored {
            return decode_delegate_result(stored, canonical_request_bytes)
                .map(DelegateReservation::Replay);
        }

        open_objective(&tx, objective, opening_permit)?;
        let delegation_json = serde_json::to_vec(delegation).map_err(StoreError::Json)?;
        tx.execute(
            "INSERT INTO delegations(id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
            params![
                delegation.id.to_string(),
                objective.id.to_string(),
                delegation_state_name(delegation.etat),
                delegation_json
            ],
        )
        .map_err(StoreError::Sql)?;
        for prerequisite in &objective.depends_on {
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM objectives WHERE id = ?1)",
                    [prerequisite.to_string()],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            if !exists {
                return Err(StoreError::NotFound("prérequis objectif absent"));
            }
            if *prerequisite == objective.id {
                return Err(StoreError::Invalid("dépendance objective réflexive"));
            }
            tx.execute(
                "INSERT INTO objective_dependencies(
                     dependent_objective_id, prerequisite_objective_id
                 ) VALUES (?1, ?2)",
                params![objective.id.to_string(), prerequisite.to_string()],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.execute(
            "INSERT INTO deferred_delegation_dispatch(
                 delegation_id, reply, timeout_secs, retry_until, dedup_retained_until,
                 max_frame_bytes, idempotency_key, issuer_scope
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                delegation.id.to_string(),
                i64::from(deferred.reply),
                i64::try_from(deferred.timeout_secs)
                    .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
                deferred.retry_until,
                deferred.dedup_retained_until,
                i64::try_from(deferred.max_frame_bytes)
                    .map_err(|_| StoreError::Invalid("max_frame_bytes hors borne"))?,
                idempotency_key,
                deferred.issuer_scope,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO delegate_idempotency(\n\
                 idempotency_key, canonical_request_bytes, objective_id, delegation_id,\n\
                 message_id, participant, timeout_secs, deadline_contractuelle\n\
             ) VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7)",
            params![
                idempotency_key,
                canonical_request_bytes,
                objective.id.to_string(),
                delegation.id.to_string(),
                delegation.participant,
                i64::try_from(deferred.timeout_secs)
                    .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
                deferred.deadline_contractuelle,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(DelegateReservation::Created)
    }

    /// Dépendants OBJECTIF→OBJECTIF d'un prérequis (index F37).
    pub fn dependents_of_objective(&self, prerequisite: Uuid) -> Result<Vec<Uuid>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT dependent_objective_id FROM objective_dependencies
                 WHERE prerequisite_objective_id = ?1 ORDER BY dependent_objective_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([prerequisite.to_string()], |row| row.get::<_, String>(0))
            .map_err(StoreError::Sql)?;
        rows.map(|row| parse_uuid(&row.map_err(StoreError::Sql)?))
            .collect()
    }

    /// Désactive l'identité métier uniquement après la désactivation technique
    /// attestée par Bridget. Aucun objectif ni délégation n'est réécrit ici.
    /// Réactive l identité métier seulement après la liaison active attestée.
    pub fn activate_project_identity(
        &mut self,
        project_id: &str,
        binding_generation: u64,
        observed_at: i64,
    ) -> Result<ProjectIdentity, StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let mut identity = tx
            .query_row(
                "SELECT project_id, display_name, status, created_at, updated_at,
                        registration_command_id
                 FROM project_identities WHERE project_id = ?1",
                [project_id],
                project_identity_from_row,
            )
            .optional()
            .map_err(StoreError::Sql)?
            .ok_or(StoreError::NotFound("identité projet inconnue"))
            .and_then(decode_project_identity)?;
        identity
            .reactivate(binding_generation, observed_at)
            .map_err(StoreError::Domain)?;
        persist_project_identity(&tx, &identity)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(identity)
    }

    pub fn disable_project_identity(
        &mut self,
        project_id: &str,
        observed_at: i64,
    ) -> Result<ProjectIdentity, StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let mut identity = tx
            .query_row(
                "SELECT project_id, display_name, status, created_at, updated_at,
                        registration_command_id
                 FROM project_identities WHERE project_id = ?1",
                [project_id],
                project_identity_from_row,
            )
            .optional()
            .map_err(StoreError::Sql)?
            .ok_or(StoreError::NotFound("identité projet inconnue"))
            .and_then(decode_project_identity)?;
        if identity.status == ProjectIdentityStatus::Active {
            identity.disable(observed_at).map_err(StoreError::Domain)?;
            persist_project_identity(&tx, &identity)?;
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(identity)
    }

    /// Pose les arêtes OBJECTIF→OBJECTIF pour une délégation déjà créée
    /// (prérequis tous clos → dispatch immédiat, arêtes journalisées quand même).
    pub fn register_objective_dependencies(
        &mut self,
        dependent: Uuid,
        prerequisites: &[Uuid],
    ) -> Result<(), StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        for prerequisite in prerequisites {
            if *prerequisite == dependent {
                return Err(StoreError::Invalid("dépendance objective réflexive"));
            }
            let exists: bool = tx
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM objectives WHERE id = ?1)",
                    [prerequisite.to_string()],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            if !exists {
                return Err(StoreError::NotFound("prérequis objectif absent"));
            }
            if objective_dependency_creates_cycle(&tx, dependent, *prerequisite)? {
                return Err(StoreError::Invalid("cycle dans les dépendances"));
            }
            tx.execute(
                "INSERT OR IGNORE INTO objective_dependencies(
                     dependent_objective_id, prerequisite_objective_id
                 ) VALUES (?1, ?2)",
                params![dependent.to_string(), prerequisite.to_string()],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Retourne uniquement les outboxes non terminales, avec l'enveloppe
    /// exacte nécessaire au lookup puis au replay de T008.
    pub fn pending_delegation_outboxes(&self) -> Result<Vec<PendingDelegationOutbox>, StoreError> {
        self.raw_pending_delegations()?
            .into_iter()
            .map(TryInto::try_into)
            .collect()
    }

    /// Retourne chaque outbox indépendamment. Une enveloppe durable invalide
    /// garde ses identifiants de corrélation et devient un refus local que le
    /// reconciliateur persiste avant de poursuivre les autres entrées.
    pub fn delegation_recovery_entries(&self) -> Result<Vec<DelegationRecoveryEntry>, StoreError> {
        self.raw_pending_delegations()?
            .into_iter()
            .map(|raw| {
                let objective_id = parse_uuid(&raw.objective_id)?;
                let delegation_id = parse_uuid(&raw.delegation_id)?;
                let message_id = parse_uuid(&raw.message_id)?;
                let reason = if raw.body_bytes.len() > MAX_MESSAGE_BYTES
                    || raw.message_bytes.len() > MAX_MESSAGE_BYTES
                {
                    LocalFailureReason::FrameTooLarge
                } else {
                    LocalFailureReason::InvalidEnvelope
                };
                Ok(match raw.try_into() {
                    Ok(pending) => DelegationRecoveryEntry::Pending(pending),
                    Err(_) => DelegationRecoveryEntry::LocalFailure {
                        objective_id,
                        delegation_id,
                        message_id,
                        reason,
                    },
                })
            })
            .collect()
    }

    fn raw_pending_delegations(&self) -> Result<Vec<RawPending>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT objective_id, delegation_id, message_id, issuer_scope, issued_at,\n\
                        target, body_bytes, reply, timeout_secs, deadline_contractuelle,\n\
                        body_hash, message_bytes, state, attempted_at, retry_until,\n\
                        dedup_retained_until\n\
                 FROM delegation_outbox\n\
                 WHERE terminal = 0 AND state IN ('prepared', 'outcome_unknown')\n\
                   AND NOT EXISTS (SELECT 1 FROM agent_retarget_requirements requirement WHERE requirement.agent_id = delegation_outbox.target)\n\
                 ORDER BY issued_at, message_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], raw_pending_from_row)
            .map_err(StoreError::Sql)?;
        rows.map(|row| row.map_err(StoreError::Sql)).collect()
    }

    /// Persiste l'issue observée par lookup. Une issue terminale retire
    /// l'enveloppe de la liste de reprise sans supprimer sa preuve.
    pub fn record_lookup_issue(
        &mut self,
        message_id: Uuid,
        issue: &IdempotencyIssue,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let next_state = match issue {
            IdempotencyIssue::Accepted { .. } => EtatOutboxDelegation::Accepted,
            IdempotencyIssue::OutcomeUnknown { .. } => EtatOutboxDelegation::OutcomeUnknown,
            IdempotencyIssue::Rejected { .. }
            | IdempotencyIssue::EnvelopeMismatch
            | IdempotencyIssue::IdempotencyExpired
            | IdempotencyIssue::InvalidIssuedAt => EtatOutboxDelegation::Rejected,
        };
        let terminal = matches!(
            next_state,
            EtatOutboxDelegation::Accepted | EtatOutboxDelegation::Rejected
        );
        let issue_json = encode_issue(issue);
        let issue_bytes = serde_json::to_vec(&issue_json).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json\n\
                 FROM delegation_outbox WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((current_state_name, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("message_id inconnu"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                coordinate_terminal_issue(&tx, message_id, issue, observed_at)?;
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue terminale déjà figée"));
        }
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != next_state {
            current_state
                .transition_vers(next_state)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = ?1, terminal = ?2, last_issue_json = ?3,\n\
                     issue_observed_at = ?4, attempted_at = COALESCE(attempted_at, ?4)\n\
                 WHERE message_id = ?5 AND state = ?6 AND terminal = 0",
                params![
                    outbox_state_name(next_state),
                    i64::from(terminal),
                    issue_bytes,
                    observed_at,
                    message_id.to_string(),
                    current_state_name
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        coordinate_terminal_issue(&tx, message_id, issue, observed_at)?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Persiste un refus local déterministe observé. Le payload fermé permet
    /// au réconciliateur de prouver pourquoi aucun octet n'a été remis à
    /// Bridget, puis de coordonner l'objectif dans la même transaction.
    pub fn record_local_failure_at(
        &mut self,
        message_id: Uuid,
        reason: LocalFailureReason,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let issue_bytes =
            serde_json::to_vec(&json!({"local": reason.as_str()})).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current: Option<(String, i64, Option<Vec<u8>>)> = tx
            .query_row(
                "SELECT state, terminal, last_issue_json\n\
                 FROM delegation_outbox WHERE message_id = ?1",
                [message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((current_state_name, current_terminal, current_issue)) = current else {
            return Err(StoreError::NotFound("message_id inconnu"));
        };
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                coordinate_local_failure(&tx, message_id, reason, observed_at)?;
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict("issue terminale déjà figée"));
        }
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != EtatOutboxDelegation::Rejected {
            current_state
                .transition_vers(EtatOutboxDelegation::Rejected)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = 'rejected', terminal = 1, last_issue_json = ?1\n\
                 WHERE message_id = ?2 AND state = ?3 AND terminal = 0",
                params![issue_bytes, message_id.to_string(), current_state_name],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant le rejet local",
            ));
        }
        coordinate_local_failure(&tx, message_id, reason, observed_at)?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Marque une transmission sans issue durable. La ligne reste éligible au
    /// lookup de reprise ; aucun nouvel identifiant n'est créé.
    pub fn record_transport_uncertainty(
        &mut self,
        message_id: Uuid,
        attempted_at: i64,
    ) -> Result<(), StoreError> {
        if attempted_at <= 0 {
            return Err(StoreError::Invalid("attempted_at invalide"));
        }
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let current_state_name: Option<String> = tx
            .query_row(
                "SELECT state FROM delegation_outbox\n\
                 WHERE message_id = ?1 AND terminal = 0",
                [message_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some(current_state_name) = current_state_name else {
            return Err(StoreError::NotFound("outbox absente ou terminale"));
        };
        let current_state = parse_outbox_state(&current_state_name)?;
        if current_state != EtatOutboxDelegation::OutcomeUnknown {
            current_state
                .transition_vers(EtatOutboxDelegation::OutcomeUnknown)
                .map_err(|_| StoreError::Conflict("transition outbox interdite"))?;
        }
        let changed = tx
            .execute(
                "UPDATE delegation_outbox\n\
                 SET state = 'outcome_unknown', attempted_at = ?1\n\
                 WHERE message_id = ?2 AND state = ?3 AND terminal = 0",
                params![attempted_at, message_id.to_string(), current_state_name],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "outbox modifiée concurremment pendant la transition",
            ));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Snapshot corrélé à la délégation, utilisable par T008 sans second
    /// lookup local ni reconstruction de payload.
    pub fn recovery_snapshot(
        &self,
        message_id: Uuid,
    ) -> Result<Option<RecoverySnapshot>, StoreError> {
        let raw = self
            .connection
            .query_row(
                "SELECT o.objective_id, o.delegation_id, o.message_id, o.issuer_scope,\n\
                        o.issued_at, o.target, o.body_bytes, o.reply, o.timeout_secs,\n\
                        o.deadline_contractuelle, o.body_hash, o.message_bytes, o.state,\n\
                        o.attempted_at, o.retry_until, o.dedup_retained_until,\n\
                        obj.state, d.state, o.last_issue_json, o.issue_observed_at\n\
                 FROM delegation_outbox o\n\
                 JOIN objectives obj ON obj.id = o.objective_id\n\
                 JOIN delegations d ON d.id = o.delegation_id\n\
                 WHERE o.message_id = ?1",
                [message_id.to_string()],
                |row| {
                    Ok(RawRecovery {
                        pending: raw_pending_from_row(row)?,
                        objective_state: row.get(16)?,
                        delegation_state: row.get(17)?,
                        last_issue_json: row.get(18)?,
                        issue_observed_at: row.get(19)?,
                    })
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        raw.map(TryInto::try_into).transpose()
    }

    /// Persiste une proposition de réveil et son approbation mono-usage. Le
    /// `command_id` est déjà porté par l'approbation, donc existe avant toute
    /// I/O vers Bridget et reste stable sur chaque replay.
    pub fn create_activation_proposal(
        &mut self,
        decision: &DecisionCoordination,
        approval: &ApprobationActivation,
    ) -> Result<(), StoreError> {
        validate_activation_proposal(decision, approval)?;
        let decision_json = serde_json::to_vec(decision).map_err(StoreError::Json)?;
        let approval_json = serde_json::to_vec(approval).map_err(StoreError::Json)?;
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
             VALUES (?1, ?2, ?3, ?4)",
            params![
                decision.id.to_string(),
                decision.objectif_id.to_string(),
                decision_state_name(decision.etat),
                decision_json,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.execute(
            "INSERT INTO activation_approvals(\n\
                 id, command_id, decision_id, objective_id, profile_id, profile_hash,\n\
                 context_hash, state, expires_at, consumed_at, payload_json\n\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'proposed', ?8, NULL, ?9)",
            params![
                approval.id.to_string(),
                approval.command_id.to_string(),
                decision.id.to_string(),
                approval.objective_id.to_string(),
                approval.profile_id,
                approval.profile_hash,
                approval.context_hash,
                approval.expires_at,
                approval_json,
            ],
        )
        .map_err(StoreError::Sql)?;
        tx.commit().map_err(StoreError::Sql)
    }

    /// Revalide l'approbation contre les hashes courants et écrit l'outbox
    /// dans la même transaction que l'état approuvé de la décision. Un retry
    /// ne peut jamais remplacer les octets d'un SpawnOrder déjà enregistré.
    pub fn approve_activation(
        &mut self,
        approval_id: Uuid,
        request: &ActivationApprovalRequest<'_>,
    ) -> Result<ActivationOutbox, StoreError> {
        self.approve_activation_observed(approval_id, request, |_| Ok(()))
    }

    /// Variante instrumentée par les crash-tests : la proposition approuvée
    /// et son outbox restent une transaction SQLite indivisible.
    pub fn approve_activation_observed(
        &mut self,
        approval_id: Uuid,
        request: &ActivationApprovalRequest<'_>,
        mut observer: impl FnMut(StoreCommitPhase) -> Result<(), StoreError>,
    ) -> Result<ActivationOutbox, StoreError> {
        if request.spawn_order_bytes.is_empty()
            || request.retry_until < request.now
            || request.retry_until > request.dedup_retained_until
        {
            return Err(StoreError::Invalid("activation outbox invalide"));
        }
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let row: Option<StoredActivationApproval> = tx
            .query_row(
                "SELECT a.payload_json, d.payload_json, a.id, a.command_id, a.objective_id,\n\
                        a.state, d.objective_id, d.state\n\
                 FROM activation_approvals a\n\
                 JOIN coordination_decisions d ON d.id = a.decision_id\n\
                 WHERE a.id = ?1",
                [approval_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get(7)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((
            approval_json,
            decision_json,
            stored_approval_id,
            stored_command_id,
            stored_approval_objective_id,
            stored_approval_state,
            stored_decision_objective_id,
            stored_decision_state,
        )) = row
        else {
            return Err(StoreError::NotFound("approbation inconnue"));
        };
        let approval: ApprobationActivation =
            serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
        let mut decision: DecisionCoordination =
            serde_json::from_slice(&decision_json).map_err(StoreError::Json)?;
        if stored_approval_id != approval.id.to_string()
            || stored_command_id != approval.command_id.to_string()
            || stored_approval_objective_id != approval.objective_id.to_string()
            || stored_decision_objective_id != decision.objectif_id.to_string()
            || approval.objective_id != decision.objectif_id
        {
            return Err(StoreError::Corrupt("relations d'approbation divergentes"));
        }
        validate_approved_spawn_order(&approval, request.spawn_order_bytes)?;

        let existing: Option<(Vec<u8>, String, i64, i64)> = tx
            .query_row(
                "SELECT spawn_order_bytes, state, retry_until, dedup_retained_until\n\
                 FROM activation_outbox WHERE approval_id = ?1",
                [approval.id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some((stored_bytes, state, stored_retry_until, stored_retained_until)) = existing {
            if stored_bytes != request.spawn_order_bytes {
                return Err(StoreError::Conflict(
                    "SpawnOrder divergent pour la même approbation",
                ));
            }
            tx.commit().map_err(StoreError::Sql)?;
            return Ok(ActivationOutbox {
                command_id: approval.command_id,
                approval_id: approval.id,
                spawn_order_bytes: stored_bytes,
                etat: parse_activation_state(&state)?,
                retry_until: stored_retry_until,
                dedup_retained_until: stored_retained_until,
            });
        }

        if stored_approval_state != "proposed" || stored_decision_state != "proposed" {
            return Err(StoreError::Corrupt("état d'approbation divergent"));
        }

        approval
            .verifier_pour_dispatch(request.now, request.profile_hash, request.context_hash)
            .map_err(StoreError::Domain)?;
        if decision.etat != EtatDecision::Proposee {
            return Err(StoreError::Conflict("décision déjà traitée"));
        }
        decision.etat = EtatDecision::Approuvee;
        let decision_json = serde_json::to_vec(&decision).map_err(StoreError::Json)?;
        let activation = ActivationOutbox {
            command_id: approval.command_id,
            approval_id: approval.id,
            spawn_order_bytes: request.spawn_order_bytes.to_vec(),
            etat: EtatActivationOutbox::Dispatching,
            retry_until: request.retry_until,
            dedup_retained_until: request.dedup_retained_until,
        };
        activation.verifier().map_err(StoreError::Domain)?;
        let changed = tx
            .execute(
                "UPDATE coordination_decisions SET state = 'approved', payload_json = ?1\n\
             WHERE id = ?2 AND state = 'proposed'",
                params![decision_json, decision.id.to_string()],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("décision modifiée concurremment"));
        }
        let changed = tx
            .execute(
                "UPDATE activation_approvals SET state = 'approved'\n\
             WHERE id = ?1 AND state = 'proposed'",
                [approval.id.to_string()],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("approbation modifiée concurremment"));
        }
        tx.execute(
            "INSERT INTO activation_outbox(\n\
                 command_id, approval_id, spawn_order_bytes, state, retry_until,\n\
                 dedup_retained_until, terminal\n\
             ) VALUES (?1, ?2, ?3, 'dispatching', ?4, ?5, 0)",
            params![
                activation.command_id.to_string(),
                activation.approval_id.to_string(),
                activation.spawn_order_bytes,
                activation.retry_until,
                activation.dedup_retained_until,
            ],
        )
        .map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::BeforeCommit)?;
        tx.commit().map_err(StoreError::Sql)?;
        observer(StoreCommitPhase::AfterCommit)?;
        Ok(activation)
    }

    /// Une issue durable est la seule opération qui consomme l'approbation.
    /// La consommation et l'état terminal de l'outbox partagent la même
    /// transaction afin qu'un crash ne puisse pas laisser un SpawnOrder
    /// appliqué avec une approbation réutilisable.
    pub fn record_activation_outcome(
        &mut self,
        command_id: Uuid,
        outcome: &SpawnOutcome,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        self.record_activation_outcome_with_issue(command_id, outcome, observed_at, None)
    }

    /// Conserve qu'un SpawnOrder a été accepté, mais que le digest de sa
    /// définition résolue ne correspond plus à celui approuvé. L'agent a bien
    /// été lancé : le représenter comme un refus serait un faux historique.
    pub fn record_activation_definition_divergence(
        &mut self,
        command_id: Uuid,
        outcome: &SpawnOutcome,
        expected_digest: &[u8],
        received_digest: &str,
        observed_at: i64,
    ) -> Result<(), StoreError> {
        let SpawnOutcome::Accepted {
            command_id: accepted_id,
            agent_id,
        } = outcome
        else {
            return Err(StoreError::Invalid("une divergence exige SpawnAccepted"));
        };
        if accepted_id != &command_id.to_string() {
            return Err(StoreError::Invalid("command_id d'issue divergent"));
        }
        let issue = serde_json::to_vec(&json!({
            "kind": "definition_digest_divergent",
            "command_id": accepted_id,
            "agent_id": agent_id,
            "expected_definition_digest": hex_digest(expected_digest),
            "received_definition_digest": received_digest,
            "agent_launched_without_followup": true,
        }))
        .map_err(StoreError::Json)?;
        self.record_activation_outcome_with_issue(command_id, outcome, observed_at, Some(issue))
    }

    fn record_activation_outcome_with_issue(
        &mut self,
        command_id: Uuid,
        outcome: &SpawnOutcome,
        observed_at: i64,
        issue_override: Option<Vec<u8>>,
    ) -> Result<(), StoreError> {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        let (next_state, default_issue_bytes) = activation_issue(command_id, outcome)?;
        let issue_bytes = issue_override.unwrap_or(default_issue_bytes);
        let tx = self.connection.transaction().map_err(StoreError::Sql)?;
        let row: Option<StoredActivationOutcome> = tx
            .query_row(
                "SELECT a.payload_json, o.state, o.terminal, o.last_issue_json,\n\
                        o.command_id, o.approval_id, a.command_id\n\
                 FROM activation_outbox o\n\
                 JOIN activation_approvals a ON a.id = o.approval_id\n\
                 WHERE o.command_id = ?1",
                [command_id.to_string()],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some((
            approval_json,
            state,
            current_terminal,
            current_issue,
            stored_command_id,
            stored_approval_id,
            stored_approval_command_id,
        )) = row
        else {
            return Err(StoreError::NotFound("activation inconnue"));
        };
        let mut approval: ApprobationActivation =
            serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
        if stored_command_id != command_id.to_string()
            || stored_approval_command_id != command_id.to_string()
            || stored_approval_id != approval.id.to_string()
            || approval.command_id != command_id
        {
            return Err(StoreError::Corrupt("relations d'activation divergentes"));
        }
        let current = parse_activation_state(&state)?;
        if current_terminal == 1 {
            if current_issue.as_deref() == Some(issue_bytes.as_slice()) {
                tx.commit().map_err(StoreError::Sql)?;
                return Ok(());
            }
            return Err(StoreError::Conflict(
                "issue d'activation terminale déjà figée",
            ));
        }
        if next_state == EtatActivationOutbox::OutcomeUnknown {
            if !matches!(current, EtatActivationOutbox::Dispatching) {
                return Err(StoreError::Conflict("transition activation interdite"));
            }
            let changed = tx
                .execute(
                    "UPDATE activation_outbox\n\
                 SET state = 'outcome_unknown', last_issue_json = ?1, issue_observed_at = ?2\n\
                 WHERE command_id = ?3 AND state = 'dispatching' AND terminal = 0",
                    params![issue_bytes, observed_at, command_id.to_string()],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("activation modifiée concurremment"));
            }
            return tx.commit().map_err(StoreError::Sql);
        }
        approval.consumed_at = Some(observed_at);
        let approval_json = serde_json::to_vec(&approval).map_err(StoreError::Json)?;
        let changed = tx.execute(
            "UPDATE activation_approvals SET state = 'consumed', consumed_at = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND consumed_at IS NULL",
            params![observed_at, approval_json, approval.id.to_string()],
        )
        .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("approbation modifiée concurremment"));
        }
        let changed = tx.execute(
            "UPDATE activation_outbox SET state = 'applied', terminal = 1, last_issue_json = ?1, issue_observed_at = ?2\n\
             WHERE command_id = ?3 AND terminal = 0",
            params![issue_bytes, observed_at, command_id.to_string()],
        )
        .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("activation modifiée concurremment"));
        }
        tx.commit().map_err(StoreError::Sql)
    }

    pub fn pending_activation_outboxes(&self) -> Result<Vec<PendingActivationOutbox>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT o.command_id, o.approval_id, o.spawn_order_bytes, o.state,\n\
                        o.retry_until, o.dedup_retained_until, a.payload_json\n\
                 FROM activation_outbox o\n\
                 JOIN activation_approvals a ON a.id = o.approval_id\n\
                 WHERE o.terminal = 0\n\
                 ORDER BY o.command_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Vec<u8>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Vec<u8>>(6)?,
                ))
            })
            .map_err(StoreError::Sql)?;
        rows.map(|row| {
            let (
                command_id,
                approval_id,
                spawn_order_bytes,
                state,
                retry_until,
                retained_until,
                approval_json,
            ) = row.map_err(StoreError::Sql)?;
            let activation = ActivationOutbox {
                command_id: parse_uuid(&command_id)?,
                approval_id: parse_uuid(&approval_id)?,
                spawn_order_bytes,
                etat: parse_activation_state(&state)?,
                retry_until,
                dedup_retained_until: retained_until,
            };
            let approval = serde_json::from_slice(&approval_json).map_err(StoreError::Json)?;
            activation.verifier().map_err(StoreError::Domain)?;
            Ok(PendingActivationOutbox {
                activation,
                approval,
            })
        })
        .collect()
    }
}

pub(crate) fn set_wal_mode(connection: &Connection) -> Result<(), StoreError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match connection.query_row("PRAGMA journal_mode = WAL", [], |row| {
            row.get::<_, String>(0)
        }) {
            Ok(mode) if mode == "wal" => return Ok(()),
            Ok(mode) => return Err(StoreError::JournalModeUnavailable { actual: mode }),
            Err(rusqlite::Error::SqliteFailure(error, _))
                if error.code == ErrorCode::DatabaseBusy && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(StoreError::Sql(error)),
        }
    }
}

fn validate_database_path(path: &Path) -> Result<(), StoreError> {
    if !path.is_absolute() {
        return Err(StoreError::Invalid("chemin SQLite non absolu"));
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("bridget.db") {
        return Err(StoreError::Invalid(
            "bridget.db appartient exclusivement à Bridget",
        ));
    }
    if path.parent().is_none() {
        return Err(StoreError::Invalid("base SQLite sans répertoire parent"));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    device: u64,
    inode: u64,
    size: u64,
    modified_secs: i64,
    modified_nanos: i64,
    changed_secs: i64,
    changed_nanos: i64,
}

struct TemporaryDatabaseSnapshot {
    root: PathBuf,
    database: PathBuf,
}

impl Drop for TemporaryDatabaseSnapshot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn sidecar_path(database: &Path, suffix: &str) -> PathBuf {
    let mut path = database.as_os_str().to_os_string();
    path.push(suffix);
    PathBuf::from(path)
}

fn file_stamp(path: &Path) -> Result<Option<FileStamp>, StoreError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(FileStamp {
            device: metadata.dev(),
            inode: metadata.ino(),
            size: metadata.size(),
            modified_secs: metadata.mtime(),
            modified_nanos: metadata.mtime_nsec(),
            changed_secs: metadata.ctime(),
            changed_nanos: metadata.ctime_nsec(),
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(StoreError::Io(error)),
    }
}

fn database_file_stamps(database: &Path) -> Result<[Option<FileStamp>; 3], StoreError> {
    Ok([
        file_stamp(database)?,
        file_stamp(&sidecar_path(database, "-wal"))?,
        file_stamp(&sidecar_path(database, "-journal"))?,
    ])
}

/// Copie le fichier et ses journaux dans un répertoire privé avant ouverture.
/// Ouvrir directement une base WAL, même `READ_ONLY`, crée un `-wal` vide ou
/// modifie les octets de verrou du `-shm`. Le double relevé de métadonnées
/// refuse conservativement une copie traversée par une écriture concurrente.
fn snapshot_database_for_preflight(source: &Path) -> Result<TemporaryDatabaseSnapshot, StoreError> {
    'attempt: for _ in 0..3 {
        let before = database_file_stamps(source)?;
        let root = std::env::temp_dir().join(format!(
            "maicie-schema-preflight-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        DirBuilder::new()
            .mode(DIRECTORY_MODE)
            .create(&root)
            .map_err(StoreError::Io)?;
        let snapshot = TemporaryDatabaseSnapshot {
            database: root.join("maicie.sqlite3"),
            root,
        };
        if before[0].is_some() {
            match fs::copy(source, &snapshot.database) {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue 'attempt,
                Err(error) => return Err(StoreError::Io(error)),
            }
        }
        for (index, suffix) in [(1, "-wal"), (2, "-journal")] {
            if before[index].is_some() {
                match fs::copy(
                    sidecar_path(source, suffix),
                    sidecar_path(&snapshot.database, suffix),
                ) {
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        continue 'attempt;
                    }
                    Err(error) => return Err(StoreError::Io(error)),
                }
            }
        }
        if before == database_file_stamps(source)? {
            return Ok(snapshot);
        }
    }
    Err(StoreError::Conflict(
        "greffe modifié pendant le préflight de schéma",
    ))
}

fn validate_private_parent_if_present(path: &Path) -> Result<(), StoreError> {
    let parent = path.parent().expect("validé");
    match fs::symlink_metadata(parent) {
        Ok(_) => validate_private_parent(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(StoreError::Io(error)),
    }
}

fn validate_private_parent(path: &Path) -> Result<(), StoreError> {
    let parent = path.parent().expect("validé");
    let metadata = fs::symlink_metadata(parent).map_err(StoreError::Io)?;
    if !metadata.file_type().is_dir() || metadata.permissions().mode() & 0o777 != DIRECTORY_MODE {
        return Err(StoreError::Invalid(
            "répertoire SQLite non privé (0700 requis)",
        ));
    }
    Ok(())
}

fn validate_private_database_metadata(metadata: &fs::Metadata) -> Result<(), StoreError> {
    if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o777 != DATABASE_MODE {
        return Err(StoreError::Invalid(
            "fichier SQLite non privé (0600 requis)",
        ));
    }
    Ok(())
}

fn prepare_private_database(path: &Path) -> Result<(), StoreError> {
    let parent = path.parent().expect("validé");
    if !parent.exists() {
        let mut builder = DirBuilder::new();
        builder.recursive(true).mode(DIRECTORY_MODE);
        match builder.create(parent) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(StoreError::Io(error)),
        }
    }
    validate_private_parent(path)?;

    if !path.exists() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(DATABASE_MODE)
            .open(path)
        {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(StoreError::Io(error)),
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(StoreError::Io)?;
    validate_private_database_metadata(&metadata)
}

type StoredGuichetAggregates = (String, Vec<u8>, String, Vec<u8>, String);

type RawGuichetProjectionFacts = (
    String,
    Vec<u8>,
    String,
    Vec<u8>,
    String,
    String,
    Option<Vec<u8>>,
    Option<i64>,
    i64,
);

type RawGuichetRefusalReception = (String, String, String, Vec<u8>, i64, String, i64);

fn load_guichet_refusal_reception(
    tx: &Transaction<'_>,
    issuer_scope: &str,
    request_id: &str,
    canonical_request_bytes: &[u8],
) -> Result<Option<ReceptionGreffe>, StoreError> {
    let raw: Option<RawGuichetRefusalReception> = tx
        .query_row(
            "SELECT operation, response_message_id, reason, reply_bytes, claim_generation, claim_token, processed_at\n\
             FROM guichet_refusal_receptions\n\
             WHERE issuer_scope = ?1 AND request_id = ?2 AND canonical_request_bytes = ?3",
            params![issuer_scope, request_id, canonical_request_bytes],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    raw.map(
        |(
            operation,
            response_message_id,
            reason,
            reply_bytes,
            claim_generation,
            claim_token,
            processed_at,
        )| {
            parse_refusal_reason_name(&reason)?;
            Ok(ReceptionGreffe {
                issuer_scope: issuer_scope.to_string(),
                request_id: request_id.to_string(),
                operation: parse_operation_name(&operation)?,
                canonical_request_bytes: canonical_request_bytes.to_vec(),
                objective_id: None,
                delegation_id: None,
                delivery_hash: None,
                response_message_id,
                claim_generation: u64::try_from(claim_generation)
                    .map_err(|_| StoreError::Corrupt("génération de refus invalide"))?,
                claim_token,
                reply_bytes,
                issue: IssueGreffe::Refusee,
                decision_id: None,
                processed_at,
            })
        },
    )
    .transpose()
}

fn load_guichet_projection_facts(
    connection: &Connection,
    delegation_id: Uuid,
) -> Result<GuichetProjectionFacts, StoreError> {
    let raw: Option<RawGuichetProjectionFacts> = connection
        .query_row(
            "SELECT obj.state, obj.payload_json, d.state, d.payload_json,\n\
                    o.message_id, o.state, o.last_issue_json, o.issue_observed_at,\n\
                    o.deadline_contractuelle\n\
             FROM delegations d\n\
             JOIN objectives obj ON obj.id = d.objective_id\n\
             JOIN delegation_outbox o ON o.delegation_id = d.id\n\
             WHERE d.id = ?1",
            [delegation_id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((
        objective_state,
        objective_json,
        delegation_state,
        delegation_json,
        message_id,
        local_state,
        local_issue,
        local_observed_at,
        deadline_at,
    )) = raw
    else {
        return Err(StoreError::NotFound("délégation de projection absente"));
    };
    let objective: ObjectifCoordonne =
        serde_json::from_slice(&objective_json).map_err(StoreError::Json)?;
    let delegation: Delegation =
        serde_json::from_slice(&delegation_json).map_err(StoreError::Json)?;
    if objective.id != delegation.objectif_id
        || delegation.id != delegation_id
        || objective.etat != parse_objective_state(&objective_state)?
        || delegation.etat != parse_delegation_state(&delegation_state)?
        || deadline_at <= 0
    {
        return Err(StoreError::Corrupt("faits de projection divergents"));
    }
    Ok(GuichetProjectionFacts {
        objective,
        delegation,
        local_delivery: RemiseLocale {
            delegation_id,
            message_id: parse_uuid(&message_id)?,
            state: parse_outbox_state(&local_state)?,
            issue: local_issue
                .map(|bytes| serde_json::from_slice(&bytes).map_err(StoreError::Json))
                .transpose()?,
            observed_at: local_observed_at,
        },
        deadline_at,
    })
}

type RawGuichetReception = (
    String,
    Vec<u8>,
    String,
    Vec<u8>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    i64,
    i64,
    String,
);

fn load_guichet_reception(
    tx: &Transaction<'_>,
    issuer_scope: &str,
    request_id: &str,
) -> Result<Option<ReceptionGreffe>, StoreError> {
    let raw: Option<RawGuichetReception> = tx
        .query_row(
            "SELECT operation, canonical_request_bytes, response_message_id, reply_bytes,\n\
                    objective_id, delegation_id, delivery_hash, outcome, decision_id,\n\
                    in_reply_to, issuer_scope, processed_at, claim_generation, claim_token\n\
             FROM guichet_receptions WHERE issuer_scope = ?1 AND request_id = ?2",
            params![issuer_scope, request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                    row.get(8)?,
                    row.get(9)?,
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    raw.map(|raw| decode_guichet_reception(request_id, raw))
        .transpose()
}

fn decode_guichet_reception(
    request_id: &str,
    raw: RawGuichetReception,
) -> Result<ReceptionGreffe, StoreError> {
    let (
        operation,
        canonical_request_bytes,
        response_message_id,
        reply_bytes,
        objective_id,
        delegation_id,
        delivery_hash,
        outcome,
        decision_id,
        _in_reply_to,
        issuer_scope,
        processed_at,
        claim_generation,
        claim_token,
    ) = raw;
    Ok(ReceptionGreffe {
        issuer_scope,
        request_id: request_id.to_string(),
        operation: parse_operation_name(&operation)?,
        canonical_request_bytes,
        objective_id: objective_id.as_deref().map(parse_uuid).transpose()?,
        delegation_id: delegation_id.as_deref().map(parse_uuid).transpose()?,
        delivery_hash,
        response_message_id,
        claim_generation: u64::try_from(claim_generation)
            .map_err(|_| StoreError::Corrupt("génération de claim invalide"))?,
        claim_token,
        reply_bytes,
        issue: parse_issue_name(&outcome)?,
        decision_id: decision_id.as_deref().map(parse_uuid).transpose()?,
        processed_at,
    })
}

/// Une génération supérieure prouve que Bridget a libéré l'ancien claim sans
/// persister sa réponse. Le fait métier et `response_message_id` restent
/// immuables ; seule l'enveloppe de claim est régénérée. À génération égale,
/// les octets doivent être rejoués strictement à l'identique.
fn refresh_reception_for_claim(
    tx: &Transaction<'_>,
    mut reception: ReceptionGreffe,
    claim: &GuichetClaim,
    report: &RapportLivraison,
) -> Result<ReceptionGreffe, StoreError> {
    if claim.claim_generation == reception.claim_generation
        && claim.claim_token == reception.claim_token
    {
        return Ok(reception);
    }
    if claim.claim_generation <= reception.claim_generation {
        return Err(StoreError::Conflict("claim guichet obsolète ou divergent"));
    }
    let reply_bytes = delivery_reply_bytes(
        claim,
        report,
        &reception.response_message_id,
        reception.issue,
    )
    .map_err(|_| StoreError::Invalid("réponse guichet non sérialisable"))?;
    let changed = tx
        .execute(
            "UPDATE guichet_receptions\n\
             SET claim_generation = ?1, claim_token = ?2, reply_bytes = ?3\n\
             WHERE issuer_scope = ?4 AND request_id = ?5\n\
               AND claim_generation = ?6 AND claim_token = ?7 AND reply_bytes = ?8",
            params![
                i64::try_from(claim.claim_generation)
                    .map_err(|_| StoreError::Invalid("génération de claim hors borne"))?,
                claim.claim_token,
                reply_bytes,
                reception.issuer_scope,
                reception.request_id,
                i64::try_from(reception.claim_generation)
                    .map_err(|_| StoreError::Corrupt("génération de claim hors borne"))?,
                reception.claim_token,
                reception.reply_bytes,
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed != 1 {
        return Err(StoreError::Conflict("reçu de greffe modifié concurremment"));
    }
    reception.claim_generation = claim.claim_generation;
    reception.claim_token = claim.claim_token.clone();
    reception.reply_bytes = reply_bytes;
    Ok(reception)
}

fn load_guichet_correlation(
    tx: &Transaction<'_>,
    in_reply_to: &str,
    response_message_id: &str,
) -> Result<Option<RecuCorrelation>, StoreError> {
    tx.query_row(
        "SELECT issuer_scope, request_id, lifecycle_event_id, lifecycle_state\n\
         FROM guichet_correlations\n\
         WHERE in_reply_to = ?1 AND response_message_id = ?2",
        params![in_reply_to, response_message_id],
        |row| {
            let state: Option<String> = row.get(3)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                state,
            ))
        },
    )
    .optional()
    .map_err(StoreError::Sql)?
    .map(
        |(issuer_scope, request_id, lifecycle_event_id, lifecycle_state)| {
            Ok(RecuCorrelation {
                issuer_scope,
                request_id,
                in_reply_to: in_reply_to.to_string(),
                response_message_id: response_message_id.to_string(),
                lifecycle_event_id,
                lifecycle_state: lifecycle_state
                    .as_deref()
                    .map(parse_lifecycle_state_name)
                    .transpose()?,
            })
        },
    )
    .transpose()
}

fn upsert_guichet_correlation(
    tx: &Transaction<'_>,
    issuer_scope: &str,
    request_id: &str,
    in_reply_to: &str,
    response_message_id: &str,
) -> Result<RecuCorrelation, StoreError> {
    if let Some(existing) = load_guichet_correlation(tx, in_reply_to, response_message_id)? {
        if existing.issuer_scope != issuer_scope || existing.request_id != request_id {
            return Err(StoreError::Conflict("corrélation guichet déjà attribuée"));
        }
        return Ok(existing);
    }
    let existing_for_request: Option<(String, String)> = tx
        .query_row(
            "SELECT in_reply_to, response_message_id FROM guichet_correlations\n\
             WHERE issuer_scope = ?1 AND request_id = ?2",
            params![issuer_scope, request_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    if existing_for_request.is_some() {
        return Err(StoreError::Conflict(
            "requête guichet corrélée différemment",
        ));
    }
    let inserted = tx
        .execute(
            "INSERT INTO guichet_correlations(\n\
                 in_reply_to, response_message_id, issuer_scope, request_id,\n\
                 lifecycle_event_id, lifecycle_state\n\
             ) VALUES (?1,?2,?3,?4,NULL,NULL)",
            params![in_reply_to, response_message_id, issuer_scope, request_id],
        )
        .map_err(StoreError::Sql)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("corrélation guichet non enregistrée"));
    }
    load_guichet_correlation(tx, in_reply_to, response_message_id)?
        .ok_or(StoreError::Corrupt("corrélation guichet introuvable"))
}

fn update_guichet_aggregates(
    tx: &Transaction<'_>,
    previous_objective: &ObjectifCoordonne,
    objective: &ObjectifCoordonne,
    previous_delegation: &Delegation,
    delegation: &Delegation,
) -> Result<(), StoreError> {
    if previous_objective.etat != EtatObjectif::Clos && objective.etat == EtatObjectif::Clos {
        return Err(StoreError::Invalid("clôture réservée à close_objective"));
    }
    if previous_objective != objective {
        let previous_json = serde_json::to_vec(previous_objective).map_err(StoreError::Json)?;
        let next_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
        let changed = tx
            .execute(
                "UPDATE objectives SET state = ?1, payload_json = ?2\n\
                 WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
                params![
                    objective_state_name(objective.etat),
                    next_json,
                    objective.id.to_string(),
                    objective_state_name(previous_objective.etat),
                    previous_json,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("objectif modifié pendant la greffe"));
        }
    }
    if previous_delegation != delegation {
        let previous_json = serde_json::to_vec(previous_delegation).map_err(StoreError::Json)?;
        let next_json = serde_json::to_vec(delegation).map_err(StoreError::Json)?;
        let changed = tx
            .execute(
                "UPDATE delegations SET state = ?1, payload_json = ?2\n\
                 WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
                params![
                    delegation_state_name(delegation.etat),
                    next_json,
                    delegation.id.to_string(),
                    delegation_state_name(previous_delegation.etat),
                    previous_json,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "délégation modifiée pendant la greffe",
            ));
        }
    }
    Ok(())
}

/// Compare le verdict au mandat chargé DANS la transaction de greffe.
/// Une cible déplacée n'exonère le juré que si son HEAD appartient encore au
/// mandat ou déjà à la nouvelle cible ; un troisième SHA conserve les deux
/// faits dans un motif composé.
fn review_verdict_refusal(
    delegation: &Delegation,
    report: &RapportLivraison,
) -> Option<MotifRefusGreffe> {
    match (&delegation.review_target, &report.review_verdict) {
        (Some(_), None) => Some(MotifRefusGreffe::VerdictRevueRequis),
        (None, Some(_)) => Some(MotifRefusGreffe::VerdictRevueInattendu),
        (None, None) => None,
        (Some(target), Some(evidence)) => {
            if evidence.target_ref != target.target_ref
                || evidence.expected_head != target.expected_head
            {
                Some(MotifRefusGreffe::MandatRevueDivergent)
            } else if evidence.observed_target_head != target.expected_head {
                if evidence.measured_head == target.expected_head
                    || evidence.measured_head == evidence.observed_target_head
                {
                    Some(MotifRefusGreffe::TeteCibleDeplacee)
                } else {
                    Some(MotifRefusGreffe::TeteCibleDeplaceeEtTeteMesureeDivergente)
                }
            } else if evidence.measured_head != target.expected_head {
                Some(MotifRefusGreffe::TeteMesureeDivergente)
            } else {
                None
            }
        }
    }
}

fn operation_name(operation: OperationGuichet) -> &'static str {
    operation.as_sql()
}

fn parse_operation_name(value: &str) -> Result<OperationGuichet, StoreError> {
    OperationGuichet::parse_sql(value).ok_or(StoreError::Corrupt("opération guichet inconnue"))
}

fn issue_name(issue: IssueGreffe) -> &'static str {
    match issue {
        IssueGreffe::Accepted => "accepted",
        IssueGreffe::DemandeDejaTerminale => "request_already_terminal",
        IssueGreffe::Refusee => "refused",
    }
}

fn parse_issue_name(value: &str) -> Result<IssueGreffe, StoreError> {
    match value {
        "accepted" => Ok(IssueGreffe::Accepted),
        "request_already_terminal" => Ok(IssueGreffe::DemandeDejaTerminale),
        "refused" => Ok(IssueGreffe::Refusee),
        _ => Err(StoreError::Corrupt("issue de greffe inconnue")),
    }
}

fn refusal_reason_name(reason: MotifRefusGreffe) -> &'static str {
    reason.as_sql()
}

fn parse_refusal_reason_name(value: &str) -> Result<MotifRefusGreffe, StoreError> {
    MotifRefusGreffe::parse_sql(value).ok_or(StoreError::Corrupt("motif de refus inconnu"))
}

fn lifecycle_state_name(state: EtatRequeteGuichet) -> &'static str {
    match state {
        EtatRequeteGuichet::Answered => "answered",
        EtatRequeteGuichet::Cancelled => "cancelled",
        EtatRequeteGuichet::TimedOut => "timed_out",
    }
}

fn parse_lifecycle_state_name(value: &str) -> Result<EtatRequeteGuichet, StoreError> {
    match value {
        "answered" => Ok(EtatRequeteGuichet::Answered),
        "cancelled" => Ok(EtatRequeteGuichet::Cancelled),
        "timed_out" => Ok(EtatRequeteGuichet::TimedOut),
        _ => Err(StoreError::Corrupt("état terminal guichet inconnu")),
    }
}

fn load_delegations_for_coordination(
    connection: &Connection,
    objectif_id: Uuid,
) -> Result<BTreeMap<Uuid, Delegation>, StoreError> {
    let mut statement = connection
        .prepare(
            "SELECT id, payload_json FROM delegations
             WHERE objective_id = ?1 ORDER BY id",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([objectif_id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(StoreError::Sql)?;
    let mut result = BTreeMap::new();
    for row in rows {
        let (id, payload) = row.map_err(StoreError::Sql)?;
        let delegation: Delegation = serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        let indexed_id = parse_uuid(&id)?;
        if delegation.id != indexed_id || delegation.objectif_id != objectif_id {
            return Err(StoreError::Corrupt(
                "délégation et index de coordination divergents",
            ));
        }
        result.insert(indexed_id, delegation);
    }
    Ok(result)
}

fn canonical_coordination_definition(
    definition: &DefinitionCoordination,
) -> DefinitionCoordination {
    let mut canonical = definition.clone();
    canonical
        .dependencies
        .sort_by_key(|edge| (edge.prerequis_id, edge.dependant_id));
    canonical
        .policies
        .sort_by_key(|policy| policy.delegation_id);
    canonical
        .attentes
        .sort_by_key(|expectation| expectation.attente_id);
    canonical
}

/// `true` si poser `dependent → prerequisite` fermerait un cycle (modèle F28 :
/// un prérequis qui dépend déjà, même transitivement, du dépendant).
fn objective_dependency_creates_cycle(
    tx: &Transaction<'_>,
    dependent: Uuid,
    prerequisite: Uuid,
) -> Result<bool, StoreError> {
    let mut stack = vec![prerequisite];
    let mut seen = BTreeSet::new();
    while let Some(node) = stack.pop() {
        if node == dependent {
            return Ok(true);
        }
        if !seen.insert(node) {
            continue;
        }
        let mut statement = tx
            .prepare(
                "SELECT prerequisite_objective_id FROM objective_dependencies
                 WHERE dependent_objective_id = ?1",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([node.to_string()], |row| row.get::<_, String>(0))
            .map_err(StoreError::Sql)?;
        let mut next = Vec::new();
        for row in rows {
            next.push(parse_uuid(&row.map_err(StoreError::Sql)?)?);
        }
        drop(statement);
        stack.extend(next);
    }
    Ok(false)
}

fn validate_coordination_dag(definition: &DefinitionCoordination) -> Result<(), StoreError> {
    let mut indegrees: BTreeMap<Uuid, usize> = BTreeMap::new();
    let mut adjacency: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for edge in &definition.dependencies {
        indegrees.entry(edge.prerequis_id).or_default();
        *indegrees.entry(edge.dependant_id).or_default() += 1;
        adjacency
            .entry(edge.prerequis_id)
            .or_default()
            .push(edge.dependant_id);
    }
    let mut ready: Vec<Uuid> = indegrees
        .iter()
        .filter_map(|(node, degree)| (*degree == 0).then_some(*node))
        .collect();
    let mut visited = 0;
    while let Some(node) = ready.pop() {
        visited += 1;
        for dependent in adjacency.get(&node).into_iter().flatten() {
            let degree = indegrees
                .get_mut(dependent)
                .ok_or(StoreError::Corrupt("nœud DAG absent"))?;
            *degree -= 1;
            if *degree == 0 {
                ready.push(*dependent);
            }
        }
    }
    if visited != indegrees.len() {
        return Err(StoreError::Invalid("cycle dans les dépendances"));
    }
    Ok(())
}

fn insert_coordination_snapshot(
    tx: &Transaction<'_>,
    snapshot: &StoredCoordinationSnapshot,
) -> Result<(), StoreError> {
    for policy in &snapshot.definition.policies {
        tx.execute(
            "INSERT INTO reassignment_policies(
                 delegation_id, objective_id, policy_version, reminder_threshold,
                 max_reemissions, payload_json
             ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                policy.delegation_id.to_string(),
                policy.objectif_id.to_string(),
                i64::try_from(policy.version)
                    .map_err(|_| StoreError::Invalid("version de politique hors borne"))?,
                i64::from(policy.seuil_relances),
                i64::from(policy.max_reemissions),
                serde_json::to_vec(policy).map_err(StoreError::Json)?,
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    for edge in &snapshot.definition.dependencies {
        tx.execute(
            "INSERT INTO delegation_dependencies(
                 objective_id, prerequisite_id, dependent_id, qualification_mode, payload_json
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                edge.objectif_id.to_string(),
                edge.prerequis_id.to_string(),
                edge.dependant_id.to_string(),
                dependency_mode_name(edge.mode),
                serde_json::to_vec(edge).map_err(StoreError::Json)?,
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    for expectation in &snapshot.definition.attentes {
        tx.execute(
            "INSERT INTO coordination_expectations(
                 expectation_id, objective_id, delegation_id, event_kind,
                 recipient, policy_version, payload_json
             ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                expectation.attente_id.to_string(),
                expectation.objectif_id.to_string(),
                expectation.delegation_id.map(|id| id.to_string()),
                expected_event_name(expectation.kind)?,
                expectation.recipient,
                i64::try_from(expectation.policy_version)
                    .map_err(|_| StoreError::Invalid("version d'attente hors borne"))?,
                serde_json::to_vec(expectation).map_err(StoreError::Json)?,
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    for lineage in &snapshot.lineages {
        tx.execute(
            "INSERT INTO delegation_lineages(
                 delegation_id, objective_id, active_generation, payload_json
             ) VALUES (?1,?2,?3,?4)",
            params![
                lineage.delegation_id.to_string(),
                lineage.objectif_id.to_string(),
                i64::try_from(lineage.generation_active)
                    .map_err(|_| StoreError::Invalid("génération hors borne"))?,
                serde_json::to_vec(lineage).map_err(StoreError::Json)?,
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    for generation in &snapshot.generations {
        generation.verifier().map_err(StoreError::Domain)?;
        tx.execute(
            "INSERT INTO delegation_generations(
                 delegation_id, objective_id, generation, participant_id, state, payload_json
             ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                generation.delegation_id.to_string(),
                generation.objectif_id.to_string(),
                i64::try_from(generation.generation)
                    .map_err(|_| StoreError::Invalid("génération hors borne"))?,
                generation.participant_id,
                generation_state_name(generation.etat),
                serde_json::to_vec(generation).map_err(StoreError::Json)?,
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    for policy in &snapshot.definition.policies {
        let request_id: String = tx
            .query_row(
                "SELECT message_id FROM delegation_outbox WHERE delegation_id = ?1",
                [policy.delegation_id.to_string()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        let episode = EpisodeRelance {
            delegation_id: policy.delegation_id,
            objectif_id: policy.objectif_id,
            generation: 1,
            request_id,
            request_ordinal: 1,
            reminder_count: 0,
            reemissions_used: 0,
            etat: EtatEpisodeRelance::Actif,
        };
        episode
            .verifier(policy.max_reemissions)
            .map_err(StoreError::Domain)?;
        insert_reminder_episode(tx, &episode)?;
    }
    Ok(())
}

fn load_coordination_snapshot(
    connection: &Connection,
    objectif_id: Uuid,
) -> Result<Option<StoredCoordinationSnapshot>, StoreError> {
    let policies: Vec<PolitiqueReassignation> = load_coordination_payloads(
        connection,
        "SELECT payload_json FROM reassignment_policies WHERE objective_id = ?1 ORDER BY delegation_id",
        objectif_id,
    )?;
    let dependencies: Vec<DependanceDelegation> = load_coordination_payloads(
        connection,
        "SELECT payload_json FROM delegation_dependencies WHERE objective_id = ?1 ORDER BY prerequisite_id, dependent_id",
        objectif_id,
    )?;
    let attentes: Vec<AttenteNotification> = load_coordination_payloads(
        connection,
        "SELECT payload_json FROM coordination_expectations WHERE objective_id = ?1 ORDER BY expectation_id",
        objectif_id,
    )?;
    let lineages: Vec<LigneeDelegation> = load_coordination_payloads(
        connection,
        "SELECT payload_json FROM delegation_lineages WHERE objective_id = ?1 ORDER BY delegation_id",
        objectif_id,
    )?;
    let generations: Vec<GenerationDelegation> = load_coordination_payloads(
        connection,
        "SELECT payload_json FROM delegation_generations WHERE objective_id = ?1 ORDER BY delegation_id, generation",
        objectif_id,
    )?;
    if policies.is_empty()
        && dependencies.is_empty()
        && attentes.is_empty()
        && lineages.is_empty()
        && generations.is_empty()
    {
        return Ok(None);
    }
    let definition = DefinitionCoordination {
        objectif_id,
        dependencies,
        policies,
        attentes,
    };
    definition.verifier_bornes().map_err(StoreError::Domain)?;
    validate_coordination_dag(&definition)?;
    let delegations = load_delegations_for_coordination(connection, objectif_id)?;
    let participants: BTreeSet<String> = delegations
        .values()
        .map(|delegation| delegation.participant.clone())
        .collect();
    for policy in &definition.policies {
        policy.verifier(&participants).map_err(StoreError::Domain)?;
    }
    for lineage in &lineages {
        if lineage.objectif_id != objectif_id || lineage.generation_active == 0 {
            return Err(StoreError::Corrupt("lignée de coordination divergente"));
        }
        let current = generations.iter().filter(|generation| {
            generation.delegation_id == lineage.delegation_id
                && generation.generation == lineage.generation_active
        });
        if current.count() != 1 {
            return Err(StoreError::Corrupt(
                "génération courante de la lignée divergente",
            ));
        }
    }
    for generation in &generations {
        generation.verifier().map_err(StoreError::Domain)?;
        if generation.objectif_id != objectif_id
            || !lineages
                .iter()
                .any(|lineage| lineage.delegation_id == generation.delegation_id)
        {
            return Err(StoreError::Corrupt(
                "génération de coordination sans lignée",
            ));
        }
    }
    Ok(Some(StoredCoordinationSnapshot {
        definition,
        lineages,
        generations,
    }))
}

fn load_coordination_payloads<T: for<'de> Deserialize<'de>>(
    connection: &Connection,
    sql: &str,
    objectif_id: Uuid,
) -> Result<Vec<T>, StoreError> {
    let mut statement = connection.prepare(sql).map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([objectif_id.to_string()], |row| row.get::<_, Vec<u8>>(0))
        .map_err(StoreError::Sql)?;
    rows.map(|row| {
        let payload = row.map_err(StoreError::Sql)?;
        serde_json::from_slice(&payload).map_err(StoreError::Json)
    })
    .collect()
}

fn load_active_coordination_context(
    connection: &Connection,
    objectif_id: Uuid,
    delegation_id: Uuid,
    generation: u64,
) -> Result<(GenerationDelegation, PolitiqueReassignation), StoreError> {
    let row: Option<StoredCoordinationContext> = connection
        .query_row(
            "SELECT g.objective_id, g.generation, g.participant_id, g.state,
                    g.payload_json, p.policy_version, p.payload_json
             FROM delegation_lineages l
             JOIN delegation_generations g
               ON g.delegation_id = l.delegation_id
              AND g.generation = l.active_generation
             JOIN reassignment_policies p ON p.delegation_id = l.delegation_id
             WHERE l.delegation_id = ?1 AND l.objective_id = ?2",
            params![delegation_id.to_string(), objectif_id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((
        stored_objective,
        stored_generation,
        participant,
        state,
        generation_bytes,
        policy_version,
        policy_bytes,
    )) = row
    else {
        return Err(StoreError::NotFound(
            "contexte actif de coordination absent",
        ));
    };
    let loaded_generation: GenerationDelegation =
        serde_json::from_slice(&generation_bytes).map_err(StoreError::Json)?;
    let policy: PolitiqueReassignation =
        serde_json::from_slice(&policy_bytes).map_err(StoreError::Json)?;
    if stored_objective != objectif_id.to_string()
        || stored_generation
            != i64::try_from(generation)
                .map_err(|_| StoreError::Invalid("génération hors borne SQLite"))?
        || loaded_generation.objectif_id != objectif_id
        || loaded_generation.delegation_id != delegation_id
        || loaded_generation.generation != generation
        || loaded_generation.participant_id != participant
        || loaded_generation.etat != parse_generation_state(&state)?
        || policy.objectif_id != objectif_id
        || policy.delegation_id != delegation_id
        || policy.version
            != u64::try_from(policy_version)
                .map_err(|_| StoreError::Corrupt("version de politique invalide"))?
    {
        return Err(StoreError::Corrupt(
            "contexte et index de coordination divergents",
        ));
    }
    loaded_generation.verifier().map_err(StoreError::Domain)?;
    Ok((loaded_generation, policy))
}

fn reassignment_request_context_from(
    connection: &Connection,
    request_id: &str,
) -> Result<ReassignmentRequestContext, StoreError> {
    if request_id.trim().is_empty() {
        return Err(StoreError::Invalid("request_id F29 vide"));
    }
    let episode: Option<(String, String, i64, String, i64)> = connection
        .query_row(
            "SELECT g.objective_id,g.delegation_id,g.generation,g.participant_id,o.timeout_secs
             FROM reminder_episodes r
             JOIN delegation_generations g
               ON g.delegation_id=r.delegation_id AND g.generation=r.generation
             JOIN delegation_outbox o ON o.delegation_id=r.delegation_id
             WHERE r.request_id=?1",
            [request_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let row = match episode {
        Some(row) => row,
        None => connection
            .query_row(
                "SELECT g.objective_id,g.delegation_id,g.generation,g.participant_id,o.timeout_secs
                 FROM delegation_outbox o
                 JOIN delegation_generations g
                   ON g.delegation_id=o.delegation_id AND g.generation=1
                 WHERE o.message_id=?1",
                [request_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(StoreError::Sql)?
            .ok_or(StoreError::NotFound("demande suivie F29 inconnue"))?,
    };
    Ok(ReassignmentRequestContext {
        objectif_id: parse_uuid(&row.0)?,
        delegation_id: parse_uuid(&row.1)?,
        generation: u64::try_from(row.2)
            .map_err(|_| StoreError::Corrupt("génération F29 invalide"))?,
        participant: row.3,
        timeout_secs: u64::try_from(row.4)
            .map_err(|_| StoreError::Corrupt("timeout F29 invalide"))?,
    })
}

fn reassignment_batch_for_fact_from(
    connection: &Connection,
    fact: FaitReassignation,
) -> Result<LotReassignation, StoreError> {
    fact.verifier().map_err(StoreError::Domain)?;
    let context = reassignment_request_context_from(connection, &fact.request_id)?;
    let next_deadline_at = fact
        .observed_at
        .checked_add(
            i64::try_from(context.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout F29 hors borne"))?,
        )
        .ok_or(StoreError::Invalid("échéance F29 hors borne"))?;
    let mut facts = vec![fact.clone()];
    if fact.kind != TypeFaitReassignation::DeliveryReport {
        let mut statement = connection
            .prepare(
                "SELECT issuer_scope,request_id,delivery_hash,processed_at
                 FROM guichet_receptions
                 WHERE operation='delivery_report' AND in_reply_to=?1
                 ORDER BY request_id",
            )
            .map_err(StoreError::Sql)?;
        let reports = statement
            .query_map([&fact.request_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .map_err(StoreError::Sql)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(StoreError::Sql)?;
        for (issuer_scope, report_request_id, delivery_hash, processed_at) in reports {
            facts.push(FaitReassignation {
                event_id: format!("delivery-report:{issuer_scope}:{report_request_id}"),
                request_id: fact.request_id.clone(),
                kind: TypeFaitReassignation::DeliveryReport,
                observed_at: processed_at,
                freshness: FraicheurCoordination::Fresh,
                delivery_hash: Some(delivery_hash),
            });
        }
    }
    Ok(LotReassignation {
        objectif_id: context.objectif_id,
        delegation_id: context.delegation_id,
        generation: context.generation,
        issued_at: fact.observed_at,
        next_deadline_at,
        faits: facts,
    })
}

fn apply_coordination_reduction_in_transaction(
    tx: &Transaction<'_>,
    input: &EntreeReductionCoordination,
    observer: &mut impl FnMut(CoordinationCommitPhase) -> Result<(), StoreError>,
) -> Result<StoredCoordinationReduction, StoreError> {
    let (generation, policy) = load_active_coordination_context(
        tx,
        input.objectif_id(),
        input.delegation_id(),
        input.generation(),
    )?;
    let reduction =
        reduire_coordination(&generation, &policy, input).map_err(StoreError::Domain)?;
    let decision_bytes = serde_json::to_vec(&reduction.decision).map_err(StoreError::Json)?;
    let existing_decision: Option<Vec<u8>> = tx
        .query_row(
            "SELECT payload_json FROM active_coordination_decisions WHERE decision_id = ?1",
            [reduction.decision.decision_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    if let Some(existing) = existing_decision {
        if existing != decision_bytes {
            return Err(StoreError::EnvelopeMismatch);
        }
        verify_replayed_coordination_effects(tx, input, &reduction)?;
        return Ok(StoredCoordinationReduction {
            reduction,
            replayed: true,
        });
    }

    if let Some(event) = input.evenement_atteste() {
        let inserted = tx
            .execute(
                "INSERT INTO coordination_events(
                     event_id, request_id, kind, reminder_message_id, recipient,
                     transport_generation, cursor, freshness, observed_at, canonical_bytes
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    event.event_id(),
                    event.request_id(),
                    coordination_event_kind_name(event.kind()),
                    event.reminder_message_id(),
                    event.recipient(),
                    i64::try_from(event.generation())
                        .map_err(|_| StoreError::Invalid("génération transport hors borne"))?,
                    i64::try_from(event.cursor())
                        .map_err(|_| StoreError::Invalid("curseur transport hors borne"))?,
                    coordination_freshness_name(event.freshness()),
                    event.observed_at(),
                    event.canonical_bytes(),
                ],
            )
            .map_err(map_coordination_insert_error)?;
        if inserted != 1 {
            return Err(StoreError::Conflict(
                "événement de coordination non enregistré",
            ));
        }
    }
    observer(CoordinationCommitPhase::AfterEventInsert)?;
    persist_coordination_effects(tx, &generation, &reduction, &decision_bytes, observer)?;
    if let EntreeReductionCoordination::ClotureEvaluee(evaluation) = input {
        open_ready_dependents(
            tx,
            evaluation.delegation_id,
            &evaluation.event_id,
            evaluation.evaluated_at,
            |phase| {
                observer(match phase {
                    DependencyOpeningCommitPhase::Decision => {
                        CoordinationCommitPhase::AfterDependentDecision
                    }
                    DependencyOpeningCommitPhase::Generation => {
                        CoordinationCommitPhase::AfterDependentTransition
                    }
                    DependencyOpeningCommitPhase::Notification => {
                        CoordinationCommitPhase::AfterDependentOutbox
                    }
                })
            },
        )?;
    }
    Ok(StoredCoordinationReduction {
        reduction,
        replayed: false,
    })
}

fn apply_reassignment_batch_in_transaction(
    tx: &Transaction<'_>,
    lot: &LotReassignation,
    observer: &mut impl FnMut(ReassignmentCommitPhase) -> Result<(), StoreError>,
) -> Result<StoredReassignmentReduction, StoreError> {
    let mut nouveaux = Vec::new();
    let mut replay_batches = BTreeSet::new();
    for fait in &lot.faits {
        let payload = serde_json::to_vec(fait).map_err(StoreError::Json)?;
        let existing: Option<(Vec<u8>, String)> = tx
            .query_row(
                "SELECT payload_json, batch_event_id FROM reassignment_events
                 WHERE event_id = ?1",
                [&fait.event_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some((stored, batch_event_id)) = existing {
            if stored != payload {
                return Err(StoreError::EnvelopeMismatch);
            }
            replay_batches.insert(batch_event_id);
        } else {
            nouveaux.push(fait.clone());
        }
    }
    if nouveaux.is_empty() {
        if replay_batches.len() != 1 {
            return Err(StoreError::Conflict("rejeu F29 couvre plusieurs lots"));
        }
        let event_id = replay_batches
            .into_iter()
            .next()
            .ok_or(StoreError::Corrupt("lot F29 rejoué absent"))?;
        let payload: Vec<u8> = tx
            .query_row(
                "SELECT r.payload_json
                 FROM active_coordination_decisions d
                 JOIN reassignment_reductions r ON r.decision_id = d.decision_id
                 WHERE d.delegation_id = ?1 AND d.generation = ?2 AND d.event_id = ?3",
                params![
                    lot.delegation_id.to_string(),
                    i64::try_from(lot.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                    event_id,
                ],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        let reduction: ReductionReassignation =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        return Ok(StoredReassignmentReduction {
            reduction,
            replayed: true,
        });
    }

    let effective = LotReassignation {
        objectif_id: lot.objectif_id,
        delegation_id: lot.delegation_id,
        generation: lot.generation,
        issued_at: lot.issued_at,
        next_deadline_at: lot.next_deadline_at,
        faits: nouveaux,
    };
    let (generation, policy, episode, generations) = load_reassignment_context(tx, &effective)?;
    let reduction = reduire_reassignation(&generation, &policy, &episode, &generations, &effective)
        .map_err(StoreError::Domain)?;
    let batch_event_id = reduction.decision.event_id.clone();
    for fait in &effective.faits {
        let inserted = tx
            .execute(
                "INSERT INTO reassignment_events(
                     event_id, objective_id, delegation_id, generation,
                     batch_event_id, request_id, kind, payload_json
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    fait.event_id,
                    lot.objectif_id.to_string(),
                    lot.delegation_id.to_string(),
                    i64::try_from(lot.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                    batch_event_id,
                    fait.request_id,
                    reassignment_fact_kind_name(fait.kind),
                    serde_json::to_vec(fait).map_err(StoreError::Json)?,
                ],
            )
            .map_err(map_coordination_insert_error)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("fait F29 non enregistré"));
        }
    }
    observer(ReassignmentCommitPhase::AfterEvents)?;

    let decision_bytes = serde_json::to_vec(&reduction.decision).map_err(StoreError::Json)?;
    insert_active_coordination_decision(tx, &reduction.decision, &decision_bytes)?;
    observer(ReassignmentCommitPhase::AfterDecision)?;
    persist_reassignment_generations(tx, &generation, &reduction)?;
    observer(ReassignmentCommitPhase::AfterGenerations)?;

    let (body_bytes, timeout_secs) = load_delegation_request_template(tx, lot.delegation_id)?;
    for effect in &reduction.effets_demandes {
        let outbox = tracked_request_outbox(
            lot,
            &policy,
            effect,
            &body_bytes,
            timeout_secs,
            &batch_event_id,
        )?;
        insert_tracked_request_outbox(tx, &outbox)?;
    }
    observer(ReassignmentCommitPhase::AfterRequestOutboxes)?;
    for notification in &reduction.notifications {
        // SPEC-087 : la chaîne épuisée exige le référent ; l'item part vers
        // la boîte humaine, en plus de la notification à l'agent.
        if notification.kind == TypeNotificationReassignation::InterventionHumaineRequise {
            enqueue_human_inbox_tx(
                tx,
                &format!("chain-exhausted:{}", lot.delegation_id),
                "chain_exhausted",
                &format!(
                    "{{\"objective_id\":\"{}\",\"delegation_id\":\"{}\"}}",
                    lot.objectif_id, lot.delegation_id
                ),
                &format!(
                    "{{\"summary\":\"Chaîne de repli épuisée pour la délégation {} (objectif {}) : intervention requise.\",\"generation\":{}}}",
                    lot.delegation_id, lot.objectif_id, lot.generation
                ),
                &["cancel".to_string(), "ack".to_string()],
                lot.issued_at,
            )?;
        }
        let outbox = reassignment_notification_outbox(lot, &policy, notification, &batch_event_id)?;
        insert_notification_outbox(tx, &outbox)?;
    }
    observer(ReassignmentCommitPhase::AfterNotifications)?;
    let reduction_bytes = serde_json::to_vec(&reduction).map_err(StoreError::Json)?;
    let inserted = tx
        .execute(
            "INSERT INTO reassignment_reductions(decision_id, payload_json) VALUES (?1,?2)",
            params![reduction.decision.decision_id.to_string(), reduction_bytes],
        )
        .map_err(StoreError::Sql)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("réduction F29 non enregistrée"));
    }
    Ok(StoredReassignmentReduction {
        reduction,
        replayed: false,
    })
}

fn load_reassignment_context(
    tx: &Transaction<'_>,
    lot: &LotReassignation,
) -> Result<
    (
        GenerationDelegation,
        PolitiqueReassignation,
        EpisodeRelance,
        Vec<GenerationDelegation>,
    ),
    StoreError,
> {
    let generation_payload: Vec<u8> = tx
        .query_row(
            "SELECT payload_json FROM delegation_generations
             WHERE delegation_id = ?1 AND objective_id = ?2 AND generation = ?3",
            params![
                lot.delegation_id.to_string(),
                lot.objectif_id.to_string(),
                i64::try_from(lot.generation)
                    .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
            ],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    let generation: GenerationDelegation =
        serde_json::from_slice(&generation_payload).map_err(StoreError::Json)?;
    let policy_payload: Vec<u8> = tx
        .query_row(
            "SELECT payload_json FROM reassignment_policies WHERE delegation_id = ?1",
            [lot.delegation_id.to_string()],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    let policy: PolitiqueReassignation =
        serde_json::from_slice(&policy_payload).map_err(StoreError::Json)?;

    let mut statement = tx
        .prepare(
            "SELECT payload_json FROM delegation_generations
             WHERE delegation_id = ?1 ORDER BY generation",
        )
        .map_err(StoreError::Sql)?;
    let generations = statement
        .query_map([lot.delegation_id.to_string()], |row| {
            row.get::<_, Vec<u8>>(0)
        })
        .map_err(StoreError::Sql)?
        .map(|row| serde_json::from_slice(&row.map_err(StoreError::Sql)?).map_err(StoreError::Json))
        .collect::<Result<Vec<GenerationDelegation>, StoreError>>()?;
    drop(statement);

    let active_episode: Option<Vec<u8>> = tx
        .query_row(
            "SELECT payload_json FROM reminder_episodes
             WHERE delegation_id = ?1 AND generation = ?2 AND state = 'actif'
             ORDER BY request_ordinal DESC LIMIT 1",
            params![
                lot.delegation_id.to_string(),
                i64::try_from(lot.generation)
                    .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
            ],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let episode = if let Some(payload) = active_episode {
        serde_json::from_slice(&payload).map_err(StoreError::Json)?
    } else {
        let request_ids: BTreeSet<&str> = lot
            .faits
            .iter()
            .map(|fait| fait.request_id.as_str())
            .collect();
        let mut found = None;
        for request_id in request_ids {
            let payload: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT payload_json FROM reminder_episodes
                     WHERE delegation_id = ?1 AND generation = ?2 AND request_id = ?3",
                    params![
                        lot.delegation_id.to_string(),
                        i64::try_from(lot.generation)
                            .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                        request_id,
                    ],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::Sql)?;
            if payload.is_some() {
                found = payload;
                break;
            }
        }
        if let Some(payload) = found {
            serde_json::from_slice(&payload).map_err(StoreError::Json)?
        } else if generation.generation == 1 && generation.etat.est_active() {
            // Les snapshots créés sous le schéma v9 possèdent déjà leur
            // demande initiale dans delegation_outbox mais pas encore
            // d'épisode F29. L'enrôlement se fait dans la transaction du
            // premier lot, sans reconstruire l'identifiant ni l'enveloppe.
            let request_id: String = tx
                .query_row(
                    "SELECT message_id FROM delegation_outbox WHERE delegation_id = ?1",
                    [lot.delegation_id.to_string()],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            let seeded = EpisodeRelance {
                delegation_id: lot.delegation_id,
                objectif_id: lot.objectif_id,
                generation: 1,
                request_id,
                request_ordinal: 1,
                reminder_count: 0,
                reemissions_used: 0,
                etat: EtatEpisodeRelance::Actif,
            };
            seeded
                .verifier(policy.max_reemissions)
                .map_err(StoreError::Domain)?;
            insert_reminder_episode(tx, &seeded)?;
            seeded
        } else {
            return Err(StoreError::NotFound("épisode F29 absent"));
        }
    };
    Ok((generation, policy, episode, generations))
}

fn persist_reassignment_generations(
    tx: &Transaction<'_>,
    current: &GenerationDelegation,
    reduction: &ReductionReassignation,
) -> Result<(), StoreError> {
    if reduction.source != *current {
        reduction.source.verifier().map_err(StoreError::Domain)?;
        if reduction.source.objectif_id != current.objectif_id
            || reduction.source.delegation_id != current.delegation_id
            || reduction.source.generation != current.generation
            || reduction.source.participant_id != current.participant_id
            || !generation_transition_allowed(current.etat, reduction.source.etat)
        {
            return Err(StoreError::Invalid("transition source F29 invalide"));
        }
        let changed = tx
            .execute(
                "UPDATE delegation_generations SET state = ?1, payload_json = ?2
                 WHERE delegation_id = ?3 AND generation = ?4 AND state = ?5",
                params![
                    generation_state_name(reduction.source.etat),
                    serde_json::to_vec(&reduction.source).map_err(StoreError::Json)?,
                    current.delegation_id.to_string(),
                    i64::try_from(current.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                    generation_state_name(current.etat),
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "génération F29 modifiée concurremment",
            ));
        }
    }

    update_reminder_episode(tx, &reduction.episode_source)?;
    if let Some(next) = &reduction.successeur {
        next.verifier().map_err(StoreError::Domain)?;
        let inserted = tx
            .execute(
                "INSERT INTO delegation_generations(
                     delegation_id, objective_id, generation, participant_id, state, payload_json
                 ) VALUES (?1,?2,?3,?4,?5,?6)",
                params![
                    next.delegation_id.to_string(),
                    next.objectif_id.to_string(),
                    i64::try_from(next.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                    next.participant_id,
                    generation_state_name(next.etat),
                    serde_json::to_vec(next).map_err(StoreError::Json)?,
                ],
            )
            .map_err(map_coordination_insert_error)?;
        if inserted != 1 {
            return Err(StoreError::Conflict("successeur F29 non enregistré"));
        }
        let lineage = LigneeDelegation {
            delegation_id: next.delegation_id,
            objectif_id: next.objectif_id,
            generation_active: next.generation,
        };
        let changed = tx
            .execute(
                "UPDATE delegation_lineages SET active_generation = ?1, payload_json = ?2
                 WHERE delegation_id = ?3 AND active_generation = ?4",
                params![
                    i64::try_from(next.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                    serde_json::to_vec(&lineage).map_err(StoreError::Json)?,
                    next.delegation_id.to_string(),
                    i64::try_from(current.generation)
                        .map_err(|_| StoreError::Invalid("génération F29 hors borne"))?,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict("lignée F29 modifiée concurremment"));
        }
    }
    if let Some(next_episode) = &reduction.episode_successeur {
        insert_reminder_episode(tx, next_episode)?;
    }
    Ok(())
}

fn insert_reminder_episode(
    tx: &Transaction<'_>,
    episode: &EpisodeRelance,
) -> Result<(), StoreError> {
    let inserted = tx
        .execute(
            "INSERT INTO reminder_episodes(
                 delegation_id, generation, request_id, request_ordinal,
                 reminder_count, reemissions_used, state, payload_json
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                episode.delegation_id.to_string(),
                i64::try_from(episode.generation)
                    .map_err(|_| StoreError::Invalid("génération d'épisode hors borne"))?,
                episode.request_id,
                i64::from(episode.request_ordinal),
                i64::from(episode.reminder_count),
                i64::from(episode.reemissions_used),
                reminder_episode_state_name(episode.etat),
                serde_json::to_vec(episode).map_err(StoreError::Json)?,
            ],
        )
        .map_err(map_coordination_insert_error)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("épisode F29 non enregistré"));
    }
    Ok(())
}

fn update_reminder_episode(
    tx: &Transaction<'_>,
    episode: &EpisodeRelance,
) -> Result<(), StoreError> {
    let changed = tx
        .execute(
            "UPDATE reminder_episodes
             SET reminder_count = ?1, reemissions_used = ?2, state = ?3, payload_json = ?4
             WHERE delegation_id = ?5 AND generation = ?6 AND request_id = ?7",
            params![
                i64::from(episode.reminder_count),
                i64::from(episode.reemissions_used),
                reminder_episode_state_name(episode.etat),
                serde_json::to_vec(episode).map_err(StoreError::Json)?,
                episode.delegation_id.to_string(),
                i64::try_from(episode.generation)
                    .map_err(|_| StoreError::Invalid("génération d'épisode hors borne"))?,
                episode.request_id,
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed != 1 {
        return Err(StoreError::Conflict("épisode F29 modifié concurremment"));
    }
    Ok(())
}

fn load_delegation_request_template(
    tx: &Transaction<'_>,
    delegation_id: Uuid,
) -> Result<(Vec<u8>, u64), StoreError> {
    let (body, timeout): (Vec<u8>, i64) = tx
        .query_row(
            "SELECT body_bytes, timeout_secs FROM delegation_outbox WHERE delegation_id = ?1",
            [delegation_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(StoreError::Sql)?;
    Ok((
        body,
        u64::try_from(timeout).map_err(|_| StoreError::Corrupt("timeout F29 invalide"))?,
    ))
}

fn tracked_request_outbox(
    lot: &LotReassignation,
    policy: &PolitiqueReassignation,
    effect: &EffetDemandeSuivie,
    body_bytes: &[u8],
    timeout_secs: u64,
    event_id: &str,
) -> Result<TrackedRequestOutbox, StoreError> {
    let message_bytes = match effect.kind {
        TypeEffetDemandeSuivie::Annuler => serde_json::to_vec(&WrapperToDaemon::CancelRequest {
            id: effect.request_id.clone(),
            sender: crate::MAICIE_IDENTITY.to_string(),
            reason: Some("réassignation Maicie préautorisée".to_string()),
        })
        .map_err(StoreError::Json)?,
        TypeEffetDemandeSuivie::Creer => {
            let body = std::str::from_utf8(body_bytes)
                .map_err(|_| StoreError::Corrupt("instruction F29 non UTF-8"))?;
            serde_json::to_vec(&PublicMessage {
                id: effect.request_id.clone(),
                from: crate::MAICIE_IDENTITY.to_string(),
                to: effect.recipient.clone(),
                body: body.to_string(),
                reply: true,
                hops: 4,
                reply_timeout: Some(timeout_secs),
                deadline_at: effect
                    .deadline_at
                    .and_then(|value| u64::try_from(value).ok()),
                in_reply_to: None,
                references: Vec::new(),
            })
            .map_err(StoreError::Json)?
        }
    };
    Ok(TrackedRequestOutbox {
        effect_id: effect.effect_id,
        issued_at: lot.issued_at,
        objectif_id: lot.objectif_id,
        delegation_id: lot.delegation_id,
        generation: effect.generation,
        event_id: event_id.to_string(),
        policy_version: policy.version,
        kind: effect.kind,
        request_id: effect.request_id.clone(),
        recipient: effect.recipient.clone(),
        message_bytes,
        etat: EtatNotificationOutbox::Prepared,
    })
}

fn insert_tracked_request_outbox(
    tx: &Transaction<'_>,
    outbox: &TrackedRequestOutbox,
) -> Result<(), StoreError> {
    if outbox.issued_at <= 0
        || outbox.generation == 0
        || outbox.policy_version == 0
        || outbox.request_id.trim().is_empty()
        || outbox.recipient.trim().is_empty()
        || outbox.message_bytes.is_empty()
        || outbox.etat != EtatNotificationOutbox::Prepared
    {
        return Err(StoreError::Invalid("outbox de demande F29 invalide"));
    }
    let inserted = tx
        .execute(
            "INSERT INTO tracked_request_outbox(
                 effect_id, issued_at, objective_id, delegation_id, generation,
                 event_id, policy_version, kind, request_id, recipient,
                 message_bytes, state, last_issue_json, terminal
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,'prepared',NULL,0)",
            params![
                outbox.effect_id.to_string(),
                outbox.issued_at,
                outbox.objectif_id.to_string(),
                outbox.delegation_id.to_string(),
                i64::try_from(outbox.generation)
                    .map_err(|_| StoreError::Invalid("génération d'effet hors borne"))?,
                outbox.event_id,
                i64::try_from(outbox.policy_version)
                    .map_err(|_| StoreError::Invalid("version d'effet hors borne"))?,
                tracked_request_kind_name(outbox.kind),
                outbox.request_id,
                outbox.recipient,
                outbox.message_bytes,
            ],
        )
        .map_err(map_coordination_insert_error)?;
    if inserted != 1 {
        return Err(StoreError::Conflict(
            "outbox de demande F29 non enregistrée",
        ));
    }
    Ok(())
}

fn reassignment_notification_outbox(
    lot: &LotReassignation,
    policy: &PolitiqueReassignation,
    notification: &NotificationReassignation,
    event_id: &str,
) -> Result<NotificationOutbox, StoreError> {
    let body = match notification.kind {
        TypeNotificationReassignation::Sortant => "Mission réassignée ou arrêtée",
        TypeNotificationReassignation::Successeur => "Mission de repli ouverte",
        TypeNotificationReassignation::InterventionHumaineRequise => {
            "Chaîne de repli épuisée : intervention humaine requise"
        }
    };
    let message_bytes = serde_json::to_vec(&PublicMessage {
        id: notification.message_id.to_string(),
        from: crate::MAICIE_IDENTITY.to_string(),
        to: notification.recipient.clone(),
        body: body.to_string(),
        reply: false,
        hops: 4,
        reply_timeout: None,
        deadline_at: None,
        in_reply_to: None,
        references: Vec::new(),
    })
    .map_err(StoreError::Json)?;
    let outbox = NotificationOutbox {
        message_id: notification.message_id,
        idempotency_key: format!("notification:{}", notification.message_id),
        issued_at: lot.issued_at,
        objectif_id: lot.objectif_id,
        delegation_id: Some(lot.delegation_id),
        generation: Some(notification.generation),
        event_id: event_id.to_string(),
        policy_version: policy.version,
        recipient: notification.recipient.clone(),
        message_bytes,
        etat: EtatNotificationOutbox::Prepared,
    };
    outbox.verifier().map_err(StoreError::Domain)?;
    Ok(outbox)
}

fn insert_active_coordination_decision(
    tx: &Transaction<'_>,
    decision: &DecisionCoordinationActive,
    decision_bytes: &[u8],
) -> Result<(), StoreError> {
    decision.verifier().map_err(StoreError::Domain)?;
    let inserted = tx
        .execute(
            "INSERT INTO active_coordination_decisions(
                 decision_id, objective_id, delegation_id, generation,
                 event_id, policy_version, kind, payload_json
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                decision.decision_id.to_string(),
                decision.objectif_id.to_string(),
                decision.delegation_id.to_string(),
                i64::try_from(decision.generation)
                    .map_err(|_| StoreError::Invalid("génération de décision hors borne"))?,
                decision.event_id,
                i64::try_from(decision.policy_version)
                    .map_err(|_| StoreError::Invalid("version de décision hors borne"))?,
                active_decision_kind_name(decision.kind),
                decision_bytes,
            ],
        )
        .map_err(map_coordination_insert_error)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("décision active non enregistrée"));
    }
    Ok(())
}

fn open_ready_dependents(
    tx: &Transaction<'_>,
    prerequisite_id: Uuid,
    event_id: &str,
    issued_at: i64,
    mut observer: impl FnMut(DependencyOpeningCommitPhase) -> Result<(), StoreError>,
) -> Result<(), StoreError> {
    open_ready_dependents_with_qualification_mutator(
        tx,
        prerequisite_id,
        event_id,
        issued_at,
        |_| {},
        &mut observer,
    )
}

fn open_ready_dependents_with_qualification_mutator(
    tx: &Transaction<'_>,
    prerequisite_id: Uuid,
    event_id: &str,
    issued_at: i64,
    mut mutate_qualifications: impl FnMut(&mut [QualificationDependance]),
    mut observer: impl FnMut(DependencyOpeningCommitPhase) -> Result<(), StoreError>,
) -> Result<(), StoreError> {
    if event_id.trim().is_empty() || issued_at <= 0 {
        return Err(StoreError::Invalid("fait d'ouverture F28 incomplet"));
    }
    let mut statement = tx
        .prepare(
            "SELECT dependent_id FROM delegation_dependencies
             WHERE prerequisite_id=?1 ORDER BY dependent_id",
        )
        .map_err(StoreError::Sql)?;
    let dependants = statement
        .query_map([prerequisite_id.to_string()], |row| row.get::<_, String>(0))
        .map_err(StoreError::Sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);

    for dependant in dependants {
        let dependant_id = parse_uuid(&dependant)?;
        let generation = load_active_generation(tx, dependant_id)?;
        let mut qualifications = load_dependency_qualifications(tx, &generation)?;
        mutate_qualifications(&mut qualifications);
        let Some(opening) = reduire_ouverture_dependance(&generation, &qualifications, event_id)
            .map_err(StoreError::Domain)?
        else {
            continue;
        };
        persist_dependent_opening(tx, &generation, &opening, issued_at, &mut observer)?;
    }
    Ok(())
}

fn load_active_generation(
    tx: &Transaction<'_>,
    delegation_id: Uuid,
) -> Result<GenerationDelegation, StoreError> {
    let payload: Vec<u8> = tx
        .query_row(
            "SELECT g.payload_json
             FROM delegation_lineages l
             JOIN delegation_generations g
               ON g.delegation_id=l.delegation_id AND g.generation=l.active_generation
             WHERE l.delegation_id=?1",
            [delegation_id.to_string()],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    let generation: GenerationDelegation =
        serde_json::from_slice(&payload).map_err(StoreError::Json)?;
    generation.verifier().map_err(StoreError::Domain)?;
    if generation.delegation_id != delegation_id {
        return Err(StoreError::Corrupt(
            "génération active et index F28 divergents",
        ));
    }
    Ok(generation)
}

fn load_dependency_qualifications(
    tx: &Transaction<'_>,
    dependent: &GenerationDelegation,
) -> Result<Vec<QualificationDependance>, StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT objective_id,prerequisite_id,qualification_mode,payload_json
             FROM delegation_dependencies WHERE dependent_id=?1 ORDER BY prerequisite_id",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([dependent.delegation_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(StoreError::Sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);
    let mut qualifications = Vec::with_capacity(rows.len());
    for (objective_id, prerequisite_id, mode, payload) in rows {
        let edge: DependanceDelegation =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if edge.objectif_id.to_string() != objective_id
            || edge.prerequis_id.to_string() != prerequisite_id
            || edge.dependant_id != dependent.delegation_id
            || dependency_mode_name(edge.mode) != mode
        {
            return Err(StoreError::Corrupt(
                "dépendance F28 et index SQLite divergents",
            ));
        }
        let (hash_greffe, cloture_evaluee) = dependency_qualification(tx, &edge)?;
        qualifications.push(QualificationDependance {
            dependance: edge,
            hash_greffe,
            cloture_evaluee,
        });
    }
    Ok(qualifications)
}

fn dependency_qualification(
    tx: &Transaction<'_>,
    edge: &DependanceDelegation,
) -> Result<(bool, bool), StoreError> {
    let (active_generation, prerequisite_state): (i64, String) = tx
        .query_row(
            "SELECT l.active_generation,g.state
             FROM delegation_lineages l
             JOIN delegation_generations g
               ON g.delegation_id=l.delegation_id AND g.generation=l.active_generation
             WHERE l.delegation_id=?1",
            [edge.prerequis_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(StoreError::Sql)?;
    if prerequisite_state != "ouverte" {
        return Ok((false, false));
    }
    let original_request_id: Option<String> = tx
        .query_row(
            "SELECT message_id FROM delegation_outbox WHERE delegation_id=?1",
            [edge.prerequis_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let mut statement = tx
        .prepare(
            "SELECT delivery_hash,in_reply_to FROM guichet_receptions
             WHERE operation='delivery_report' AND outcome='accepted'
               AND objective_id=?1 AND delegation_id=?2
             ORDER BY request_id",
        )
        .map_err(StoreError::Sql)?;
    let reports = statement
        .query_map(
            params![edge.objectif_id.to_string(), edge.prerequis_id.to_string(),],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .map_err(StoreError::Sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);

    let mut hash_greffe = false;
    let mut cloture_evaluee = false;
    for (delivery_hash, request_id) in reports {
        let (Some(delivery_hash), Some(request_id)) = (delivery_hash, request_id) else {
            return Err(StoreError::Corrupt("rapport F28 incomplet"));
        };
        let episode_generation: Option<i64> = tx
            .query_row(
                "SELECT generation FROM reminder_episodes WHERE request_id=?1",
                [&request_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let request_matches_generation = episode_generation == Some(active_generation)
            || (active_generation == 1 && original_request_id.as_deref() == Some(&request_id));
        if !request_matches_generation {
            continue;
        }
        hash_greffe = true;
        let evaluated: bool = tx
            .query_row(
                "SELECT EXISTS(
                     SELECT 1 FROM evaluated_closure_acts
                     WHERE objective_id=?1 AND delegation_id=?2
                       AND generation=?3 AND delivery_hash=?4
                 )",
                params![
                    edge.objectif_id.to_string(),
                    edge.prerequis_id.to_string(),
                    active_generation,
                    delivery_hash,
                ],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        cloture_evaluee |= evaluated;
    }
    Ok((hash_greffe, cloture_evaluee))
}

fn persist_dependent_opening(
    tx: &Transaction<'_>,
    current: &GenerationDelegation,
    opening: &ReductionOuvertureDelegation,
    issued_at: i64,
    observer: &mut impl FnMut(DependencyOpeningCommitPhase) -> Result<(), StoreError>,
) -> Result<(), StoreError> {
    let decision_bytes = serde_json::to_vec(&opening.decision).map_err(StoreError::Json)?;
    insert_active_coordination_decision(tx, &opening.decision, &decision_bytes)?;
    observer(DependencyOpeningCommitPhase::Decision)?;
    apply_coordination_transition(
        tx,
        current,
        &TransitionCoordinationActive::Generation(opening.generation.clone()),
    )?;
    observer(DependencyOpeningCommitPhase::Generation)?;
    let message = PublicMessage {
        id: opening.notification.message_id.to_string(),
        from: crate::MAICIE_IDENTITY.to_string(),
        to: opening.notification.recipient.clone(),
        body: format!("Délégation {} ouverte", opening.generation.delegation_id),
        reply: false,
        hops: 4,
        reply_timeout: None,
        deadline_at: None,
        in_reply_to: None,
        references: Vec::new(),
    };
    let outbox = NotificationOutbox {
        message_id: opening.notification.message_id,
        idempotency_key: format!("notification:{}", opening.notification.message_id),
        issued_at,
        objectif_id: opening.generation.objectif_id,
        delegation_id: Some(opening.generation.delegation_id),
        generation: Some(opening.generation.generation),
        event_id: opening.decision.event_id.clone(),
        policy_version: DEPENDENCY_POLICY_VERSION,
        recipient: opening.notification.recipient.clone(),
        message_bytes: serde_json::to_vec(&message).map_err(StoreError::Json)?,
        etat: EtatNotificationOutbox::Prepared,
    };
    insert_notification_outbox(tx, &outbox)?;
    observer(DependencyOpeningCommitPhase::Notification)
}

fn persist_coordination_effects(
    tx: &Transaction<'_>,
    current: &GenerationDelegation,
    reduction: &ReductionCoordinationActive,
    decision_bytes: &[u8],
    observer: &mut impl FnMut(CoordinationCommitPhase) -> Result<(), StoreError>,
) -> Result<(), StoreError> {
    insert_active_coordination_decision(tx, &reduction.decision, decision_bytes)?;
    observer(CoordinationCommitPhase::AfterDecisionInsert)?;
    apply_coordination_transition(tx, current, &reduction.transition)?;
    observer(CoordinationCommitPhase::AfterTransition)?;
    for outbox in &reduction.outboxes {
        insert_notification_outbox(tx, outbox)?;
    }
    observer(CoordinationCommitPhase::AfterOutboxes)
}

fn apply_coordination_transition(
    tx: &Transaction<'_>,
    current: &GenerationDelegation,
    transition: &TransitionCoordinationActive,
) -> Result<(), StoreError> {
    match transition {
        TransitionCoordinationActive::Aucune => Ok(()),
        TransitionCoordinationActive::Generation(next) => {
            next.verifier().map_err(StoreError::Domain)?;
            if next.objectif_id != current.objectif_id
                || next.delegation_id != current.delegation_id
                || next.generation != current.generation
                || next.participant_id != current.participant_id
                || !generation_transition_allowed(current.etat, next.etat)
            {
                return Err(StoreError::Invalid("transition de génération invalide"));
            }
            let payload = serde_json::to_vec(next).map_err(StoreError::Json)?;
            let changed = tx
                .execute(
                    "UPDATE delegation_generations SET state = ?1, payload_json = ?2
                     WHERE delegation_id = ?3 AND generation = ?4 AND state = ?5",
                    params![
                        generation_state_name(next.etat),
                        payload,
                        next.delegation_id.to_string(),
                        i64::try_from(next.generation)
                            .map_err(|_| StoreError::Invalid("génération hors borne"))?,
                        generation_state_name(current.etat),
                    ],
                )
                .map_err(StoreError::Sql)?;
            if changed != 1 {
                return Err(StoreError::Conflict("génération modifiée concurremment"));
            }
            Ok(())
        }
        TransitionCoordinationActive::ClotureEvaluee(act) => {
            act.verifier().map_err(StoreError::Domain)?;
            if act.objectif_id() != current.objectif_id
                || act.delegation_id() != current.delegation_id
                || act.generation() != current.generation
            {
                return Err(StoreError::Invalid("acte évalué et génération divergents"));
            }
            let payload = serde_json::to_vec(act).map_err(StoreError::Json)?;
            let inserted = tx
                .execute(
                    "INSERT INTO evaluated_closure_acts(
                         act_id, objective_id, delegation_id, generation,
                         delivery_hash, evaluated_at, payload_json
                     ) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![
                        act.acte_id().to_string(),
                        act.objectif_id().to_string(),
                        act.delegation_id().to_string(),
                        i64::try_from(act.generation())
                            .map_err(|_| StoreError::Invalid("génération d'acte hors borne"))?,
                        act.delivery_hash(),
                        act.evaluated_at(),
                        payload,
                    ],
                )
                .map_err(map_coordination_insert_error)?;
            if inserted != 1 {
                return Err(StoreError::Conflict("acte évalué non enregistré"));
            }
            Ok(())
        }
    }
}

#[derive(Serialize)]
struct ObjectiveClosureMessage<'a> {
    id: String,
    from: &'static str,
    to: &'a str,
    body: String,
    reply: bool,
    hops: u8,
}

fn persist_objective_closure<F>(
    tx: &Transaction<'_>,
    objective: &ObjectifCoordonne,
    expected_state: EtatObjectif,
    decision: &DecisionCoordination,
    issued_at: i64,
    costs: Option<&[CoutMissionAgent]>,
    observer: &mut F,
    control: ControlSnapshot,
) -> Result<(), StoreError>
where
    F: FnMut(ObjectiveClosureCommitPhase) -> Result<(), StoreError>,
{
    if objective.etat != EtatObjectif::Clos
        || decision.kind != TypeDecision::Cloturer
        || decision.objectif_id != objective.id
        || issued_at <= 0
    {
        return Err(StoreError::Invalid("clôture transactionnelle invalide"));
    }
    let objective_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
    let changed = tx
        .execute(
            "UPDATE objectives SET state = 'clos', payload_json = ?1\n\
             WHERE id = ?2 AND state = ?3",
            params![
                objective_json,
                objective.id.to_string(),
                objective_state_name(expected_state),
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed != 1 {
        return Err(StoreError::Conflict("objectif modifié concurremment"));
    }
    observer(ObjectiveClosureCommitPhase::AfterObjectiveUpdate)?;

    let decision_json = serde_json::to_vec(decision).map_err(StoreError::Json)?;
    let inserted = tx
        .execute(
            "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
             VALUES (?1, ?2, ?3, ?4)",
            params![
                decision.id.to_string(),
                objective.id.to_string(),
                decision_state_name(decision.etat),
                decision_json,
            ],
        )
        .map_err(StoreError::Sql)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("décision non enregistrée"));
    }
    observer(ObjectiveClosureCommitPhase::AfterDecisionInsert)?;

    for outbox in objective_closure_outboxes(tx, objective.id, decision, issued_at)? {
        insert_notification_outbox(tx, &outbox)?;
    }
    // Politique 31 : libération des plages dans la même transaction que les notifications.
    release_resource_ranges_for_objective(tx, objective.id)?;
    // Orphelines a_evaluer : solde des délégations ouvertes, même transaction.
    settle_open_delegations_on_objective_closure(tx, objective.id)?;
    persist_objective_costs(tx, objective, issued_at, costs)?;
    // Routines : clôture d'occurrence liée dans la même transaction (manche 4).
    MaicieStore::terminate_occurrences_for_objective_tx(tx, objective.id)?;
    // F37 : déblocage OBJECTIF→OBJECTIF dans la même transaction que 016.
    // Aucun dépendant → zéro écriture supplémentaire (oracle silencieux).
    release_waiting_dependents_on_prerequisite_closure(
        tx,
        objective.id,
        decision,
        issued_at,
        control,
    )?;
    observer(ObjectiveClosureCommitPhase::AfterOutboxes)
}

fn release_resource_ranges_for_objective(
    tx: &Transaction<'_>,
    objective_id: Uuid,
) -> Result<(), StoreError> {
    tx.execute(
        "DELETE FROM resource_range_reservations WHERE objective_id = ?1",
        [objective_id.to_string()],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

/// Solde toute délégation encore ouverte sur un objectif en cours de clôture.
/// État cible : `soldee_par_cloture` — jamais `terminee` (pas de verdict inventé).
/// Dans la même transaction, les enveloppes encore expédiables de ces
/// délégations passent terminales : la reprise ne doit jamais envoyer une
/// mission que le greffe tient pour close.
fn settle_open_delegations_on_objective_closure(
    tx: &Transaction<'_>,
    objective_id: Uuid,
) -> Result<(), StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT id, state, payload_json FROM delegations\n\
             WHERE objective_id = ?1 ORDER BY id",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([objective_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .map_err(StoreError::Sql)?;
    for row in rows {
        let (id, state, payload) = row.map_err(StoreError::Sql)?;
        let mut delegation: Delegation =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if delegation.id.to_string() != id
            || delegation.objectif_id != objective_id
            || delegation.etat != parse_delegation_state(&state)?
        {
            return Err(StoreError::Corrupt("délégation et index SQLite divergents"));
        }
        let previous = delegation.etat;
        match previous {
            EtatDelegation::Terminee
            | EtatDelegation::Annulee
            | EtatDelegation::SoldeeParCloture => continue,
            EtatDelegation::EnAttentePrerequis
            | EtatDelegation::Creee
            | EtatDelegation::AEvaluer => {
                delegation
                    .solder_par_cloture()
                    .map_err(StoreError::Domain)?;
            }
        }
        let next_json = serde_json::to_vec(&delegation).map_err(StoreError::Json)?;
        let changed = tx
            .execute(
                "UPDATE delegations SET state = ?1, payload_json = ?2\n\
                 WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
                params![
                    delegation_state_name(delegation.etat),
                    next_json,
                    id,
                    delegation_state_name(previous),
                    payload,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "délégation modifiée pendant la clôture",
            ));
        }
        terminalize_dispatchable_outboxes_for_settled_delegation(tx, &id)?;
    }
    Ok(())
}

/// Retire de la reprise (`pending_delegation_outboxes`) toute enveloppe encore
/// expédiable d'une délégation soldée par clôture. Ne touche pas la machine à
/// états de la délégation (déjà soldée) — uniquement l'outbox.
fn terminalize_dispatchable_outboxes_for_settled_delegation(
    tx: &Transaction<'_>,
    delegation_id: &str,
) -> Result<(), StoreError> {
    let issue_bytes =
        serde_json::to_vec(&json!({"local": "soldee_par_cloture"})).map_err(StoreError::Json)?;
    tx.execute(
        "UPDATE delegation_outbox\n\
         SET state = 'rejected', terminal = 1, last_issue_json = ?1\n\
         WHERE delegation_id = ?2 AND terminal = 0\n\
           AND state IN ('prepared', 'outcome_unknown')",
        params![issue_bytes, delegation_id],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

fn persist_objective_costs(
    tx: &Transaction<'_>,
    objective: &ObjectifCoordonne,
    closed_at: i64,
    overrides: Option<&[CoutMissionAgent]>,
) -> Result<(), StoreError> {
    let mut windows: BTreeMap<String, i64> = BTreeMap::new();
    let mut statement = tx
        .prepare("SELECT payload_json FROM delegations WHERE objective_id = ?1 ORDER BY id")
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([objective.id.to_string()], |row| row.get::<_, Vec<u8>>(0))
        .map_err(StoreError::Sql)?;
    for row in rows {
        let payload = row.map_err(StoreError::Sql)?;
        let delegation: Delegation = serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        windows
            .entry(delegation.participant)
            .or_insert(objective.cree_at.max(1));
    }
    let mut outbox = tx
        .prepare(
            "SELECT target, MIN(issued_at) FROM delegation_outbox
             WHERE objective_id = ?1 GROUP BY target",
        )
        .map_err(StoreError::Sql)?;
    let outbox_rows = outbox
        .query_map([objective.id.to_string()], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(StoreError::Sql)?;
    for row in outbox_rows {
        let (target, issued_at) = row.map_err(StoreError::Sql)?;
        if issued_at > 0 {
            windows.insert(target, issued_at);
        }
    }
    let mut costs: BTreeMap<String, CoutMissionAgent> = windows
        .into_iter()
        .map(|(agent, from_secs)| {
            let from_secs = if from_secs > 0 {
                from_secs
            } else {
                objective.cree_at.max(1)
            };
            let to_secs = closed_at.max(from_secs);
            (
                agent.clone(),
                CoutMissionAgent::unknown(agent, from_secs, to_secs),
            )
        })
        .collect();
    if let Some(overrides) = overrides {
        for cost in overrides {
            if let Some(slot) = costs.get_mut(&cost.agent) {
                *slot = cost.clone();
            }
        }
    }
    for cost in costs.values() {
        let attested = i64::from(cost.attested);
        tx.execute(
            "INSERT INTO objective_costs (
                 objective_id, agent, from_secs, to_secs, attested,
                 turns, input_tokens, output_tokens,
                 cache_creation_input_tokens, cache_read_input_tokens,
                 facturable_tokens
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                objective.id.to_string(),
                cost.agent,
                cost.from_secs,
                cost.to_secs,
                attested,
                cost.turns.map(|value| value as i64),
                cost.input_tokens.map(|value| value as i64),
                cost.output_tokens.map(|value| value as i64),
                cost.cache_creation_input_tokens.map(|value| value as i64),
                cost.cache_read_input_tokens.map(|value| value as i64),
                cost.facturable_tokens.map(|value| value as i64),
            ],
        )
        .map_err(StoreError::Sql)?;
    }
    Ok(())
}

fn objective_closure_outboxes(
    tx: &Transaction<'_>,
    objective_id: Uuid,
    decision: &DecisionCoordination,
    issued_at: i64,
) -> Result<Vec<NotificationOutbox>, StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT expectation_id, recipient, policy_version, payload_json\n\
             FROM coordination_expectations\n\
             WHERE objective_id = ?1 AND event_kind = 'cloture_objectif'\n\
             ORDER BY expectation_id",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([objective_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(StoreError::Sql)?;
    let mut outboxes = Vec::new();
    for row in rows {
        let (expectation_id, recipient, policy_version, payload) = row.map_err(StoreError::Sql)?;
        let expectation: AttenteNotification =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if expectation.attente_id.to_string() != expectation_id
            || expectation.objectif_id != objective_id
            || expectation.kind != TypeEvenementAttendu::ClotureObjectif
            || expectation.recipient != recipient
            || i64::try_from(expectation.policy_version).ok() != Some(policy_version)
        {
            return Err(StoreError::Corrupt(
                "attente de clôture et index SQLite divergents",
            ));
        }
        let message_id = identifiant_deterministe(
            b"notification-cloture-objectif-v1",
            &[
                objective_id.as_bytes(),
                expectation.attente_id.as_bytes(),
                decision.id.as_bytes(),
            ],
        );
        let event_id = format!("objective-closed:{objective_id}:{}", decision.id);
        let message_bytes = serde_json::to_vec(&ObjectiveClosureMessage {
            id: message_id.to_string(),
            from: crate::MAICIE_IDENTITY,
            to: &expectation.recipient,
            body: format!("Objectif {objective_id} clôturé"),
            reply: false,
            hops: 4,
        })
        .map_err(StoreError::Json)?;
        let generation = expectation
            .delegation_id
            .map(|delegation_id| active_generation_optional(tx, delegation_id))
            .transpose()?
            .flatten();
        let outbox = NotificationOutbox {
            message_id,
            idempotency_key: format!("notification:{message_id}"),
            issued_at,
            objectif_id: objective_id,
            delegation_id: expectation.delegation_id,
            generation,
            event_id,
            policy_version: expectation.policy_version,
            recipient: expectation.recipient,
            message_bytes,
            etat: EtatNotificationOutbox::Prepared,
        };
        outbox.verifier().map_err(StoreError::Domain)?;
        outboxes.push(outbox);
    }
    Ok(outboxes)
}

/// Débloque les dépendants F37 dont tous les prérequis objectifs sont clos.
/// Aucun dépendant → aucune écriture (exigence oracle silencieux).
fn release_waiting_dependents_on_prerequisite_closure(
    tx: &Transaction<'_>,
    closed_prerequisite: Uuid,
    decision: &DecisionCoordination,
    issued_at: i64,
    control: ControlSnapshot,
) -> Result<(), StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT dependent_objective_id FROM objective_dependencies
             WHERE prerequisite_objective_id = ?1 ORDER BY dependent_objective_id",
        )
        .map_err(StoreError::Sql)?;
    let dependents = statement
        .query_map([closed_prerequisite.to_string()], |row| {
            row.get::<_, String>(0)
        })
        .map_err(StoreError::Sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);

    // SPEC-087 : la matérialisation d'une outbox est un effet autonome. En
    // pause, la ligne différée est conservée avec son motif et rejouée par
    // `release_ready_dependents` à la levée.
    let admission = admit_autonomous_effect(AutonomousEffect::DependencyRelease, control);
    for dependent_raw in dependents {
        let dependent_id = parse_uuid(&dependent_raw)?;
        if !all_objective_prerequisites_closed(tx, dependent_id)? {
            continue;
        }
        if let Admission::Deferred { motif } = admission {
            mark_deferred_dispatch_reason(tx, dependent_id, motif)?;
            continue;
        }
        materialize_waiting_dependent(
            tx,
            dependent_id,
            Some(closed_prerequisite),
            &decision.id.to_string(),
            issued_at,
        )?;
    }
    Ok(())
}

/// Pose le motif de différé sur la ligne de dispatch en attente du dépendant.
fn mark_deferred_dispatch_reason(
    tx: &Transaction<'_>,
    dependent_id: Uuid,
    motif: &str,
) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE deferred_delegation_dispatch SET deferred_reason = ?1
         WHERE delegation_id IN (
             SELECT d.id FROM delegations d WHERE d.objective_id = ?2
         )",
        params![motif, dependent_id.to_string()],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

/// Matérialise l'outbox d'un dépendant dont tous les prérequis sont clos.
/// Rend `false` si aucune délégation en attente n'existe pour lui.
fn materialize_waiting_dependent(
    tx: &Transaction<'_>,
    dependent_id: Uuid,
    closed_prerequisite: Option<Uuid>,
    cause_id: &str,
    issued_at: i64,
) -> Result<bool, StoreError> {
    let cause_prerequisite = closed_prerequisite.unwrap_or(dependent_id);
    let Some((objective, mut delegation, deferred)) =
        load_waiting_dependent_bundle(tx, dependent_id)?
    else {
        return Ok(false);
    };
    delegation
        .transition(EtatDelegation::Creee)
        .map_err(StoreError::Domain)?;
    let message_id = Uuid::new_v4();
    delegation
        .finaliser_mandat(message_id)
        .map_err(StoreError::Domain)?;
    let body_bytes = delegation.instruction.as_bytes().to_vec();
    let deadline = issued_at
        .checked_add(
            i64::try_from(deferred.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout hors borne"))?,
        )
        .ok_or(StoreError::Invalid("échéance hors borne"))?;
    let outbox = OutboxDelegation {
        message_id,
        delegation_id: delegation.id,
        target: delegation.participant.clone(),
        body_bytes: body_bytes.clone(),
        reply: deferred.reply,
        timeout_secs: deferred.timeout_secs,
        deadline_contractuelle: deadline,
        body_hash: stable_body_hash(&body_bytes),
        etat: EtatOutboxDelegation::Prepared,
        attempted_at: None,
        retry_until: deferred.retry_until.max(issued_at),
        dedup_retained_until: deferred.dedup_retained_until.max(issued_at),
    };
    let prepared = PreparedDelegation::new(
        objective.clone(),
        delegation.clone(),
        outbox,
        deferred.issuer_scope.clone(),
        issued_at,
        deferred.max_frame_bytes,
    )
    .map_err(StoreError::Outbox)?;
    // L'objectif existe déjà : insert_prepared ferait upsert puis INSERT
    // délégation (conflit). On pose seulement l'outbox + transition.
    let delegation_json = serde_json::to_vec(&delegation).map_err(StoreError::Json)?;
    let changed = tx
        .execute(
            "UPDATE delegations SET state = ?1, payload_json = ?2
             WHERE id = ?3 AND state = ?4",
            params![
                delegation_state_name(EtatDelegation::Creee),
                delegation_json,
                delegation.id.to_string(),
                delegation_state_name(EtatDelegation::EnAttentePrerequis),
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed != 1 {
        return Err(StoreError::Conflict(
            "délégation d'attente modifiée concurremment",
        ));
    }
    tx.execute(
        "INSERT INTO delegation_outbox(\n\
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,\n\
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,\n\
             message_bytes, state, attempted_at, retry_until, dedup_retained_until, terminal\n\
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'prepared',NULL,?13,?14,0)",
        params![
            prepared.outbox.message_id.to_string(),
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            prepared.issuer_scope,
            prepared.issued_at,
            prepared.outbox.target,
            prepared.outbox.body_bytes,
            i64::from(prepared.outbox.reply),
            i64::try_from(prepared.outbox.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
            prepared.outbox.deadline_contractuelle,
            prepared.outbox.body_hash,
            prepared.message_bytes,
            prepared.outbox.retry_until,
            prepared.outbox.dedup_retained_until,
        ],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "UPDATE delegate_idempotency
         SET message_id = ?1, deadline_contractuelle = ?2
         WHERE delegation_id = ?3 AND message_id IS NULL",
        params![message_id.to_string(), deadline, delegation.id.to_string(),],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "DELETE FROM deferred_delegation_dispatch WHERE delegation_id = ?1",
        [delegation.id.to_string()],
    )
    .map_err(StoreError::Sql)?;

    let notify_id = identifiant_deterministe(
        b"notification-deblocage-objectif-v1",
        &[
            cause_prerequisite.as_bytes(),
            dependent_id.as_bytes(),
            cause_id.as_bytes(),
        ],
    );
    let notify_bytes = serde_json::to_vec(&ObjectiveClosureMessage {
        id: notify_id.to_string(),
        from: crate::MAICIE_IDENTITY,
        to: &delegation.participant,
        body: format!(
            "Prérequis {cause_prerequisite} clôturé — délégation {} débloquée",
            delegation.id
        ),
        reply: false,
        hops: 4,
    })
    .map_err(StoreError::Json)?;
    let notify = NotificationOutbox {
        message_id: notify_id,
        idempotency_key: format!("notification:{notify_id}"),
        issued_at,
        objectif_id: dependent_id,
        delegation_id: Some(delegation.id),
        generation: None,
        event_id: format!("objective-unblocked:{dependent_id}:{cause_prerequisite}"),
        policy_version: DEPENDENCY_POLICY_VERSION,
        recipient: delegation.participant.clone(),
        message_bytes: notify_bytes,
        etat: EtatNotificationOutbox::Prepared,
    };
    notify.verifier().map_err(StoreError::Domain)?;
    insert_notification_outbox(tx, &notify)?;
    Ok(true)
}

fn all_objective_prerequisites_closed(
    tx: &Transaction<'_>,
    dependent_id: Uuid,
) -> Result<bool, StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT prerequisite_objective_id FROM objective_dependencies
             WHERE dependent_objective_id = ?1",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([dependent_id.to_string()], |row| row.get::<_, String>(0))
        .map_err(StoreError::Sql)?;
    let mut any = false;
    for row in rows {
        any = true;
        let prereq = parse_uuid(&row.map_err(StoreError::Sql)?)?;
        let state: String = tx
            .query_row(
                "SELECT state FROM objectives WHERE id = ?1",
                [prereq.to_string()],
                |row| row.get(0),
            )
            .map_err(StoreError::Sql)?;
        if state != "clos" {
            return Ok(false);
        }
    }
    Ok(any)
}

struct LoadedDeferred {
    reply: bool,
    timeout_secs: u64,
    retry_until: i64,
    dedup_retained_until: i64,
    max_frame_bytes: usize,
    issuer_scope: String,
}

fn load_waiting_dependent_bundle(
    tx: &Transaction<'_>,
    dependent_id: Uuid,
) -> Result<Option<(ObjectifCoordonne, Delegation, LoadedDeferred)>, StoreError> {
    let objective_row: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [dependent_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((state, payload)) = objective_row else {
        return Ok(None);
    };
    let objective: ObjectifCoordonne =
        serde_json::from_slice(&payload).map_err(StoreError::Json)?;
    if objective.id != dependent_id || objective.etat != parse_objective_state(&state)? {
        return Err(StoreError::Corrupt("objectif dépendant divergent"));
    }
    let delegation_row: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM delegations
             WHERE objective_id = ?1 ORDER BY id LIMIT 1",
            [dependent_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((delegation_state, delegation_payload)) = delegation_row else {
        return Ok(None);
    };
    let delegation: Delegation =
        serde_json::from_slice(&delegation_payload).map_err(StoreError::Json)?;
    if delegation.etat != EtatDelegation::EnAttentePrerequis
        || parse_delegation_state(&delegation_state)? != EtatDelegation::EnAttentePrerequis
    {
        return Ok(None);
    }
    let deferred_row: Option<(i64, i64, i64, i64, i64, String)> = tx
        .query_row(
            "SELECT reply, timeout_secs, retry_until, dedup_retained_until,
                    max_frame_bytes, issuer_scope
             FROM deferred_delegation_dispatch WHERE delegation_id = ?1",
            [delegation.id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((
        reply,
        timeout_secs,
        retry_until,
        dedup_retained_until,
        max_frame_bytes,
        issuer_scope,
    )) = deferred_row
    else {
        return Err(StoreError::Corrupt(
            "délégation en attente sans paramètres de dispatch",
        ));
    };
    Ok(Some((
        objective,
        delegation,
        LoadedDeferred {
            reply: reply != 0,
            timeout_secs: u64::try_from(timeout_secs)
                .map_err(|_| StoreError::Corrupt("timeout différé invalide"))?,
            retry_until,
            dedup_retained_until,
            max_frame_bytes: usize::try_from(max_frame_bytes)
                .map_err(|_| StoreError::Corrupt("max_frame_bytes différé invalide"))?,
            issuer_scope,
        },
    )))
}

fn active_generation_optional(
    tx: &Transaction<'_>,
    delegation_id: Uuid,
) -> Result<Option<u64>, StoreError> {
    tx.query_row(
        "SELECT active_generation FROM delegation_lineages WHERE delegation_id = ?1",
        [delegation_id.to_string()],
        |row| row.get::<_, i64>(0),
    )
    .optional()
    .map_err(StoreError::Sql)?
    .map(|generation| {
        u64::try_from(generation)
            .map_err(|_| StoreError::Corrupt("génération active de notification invalide"))
    })
    .transpose()
}

fn insert_notification_outbox(
    tx: &Transaction<'_>,
    outbox: &NotificationOutbox,
) -> Result<(), StoreError> {
    outbox.verifier().map_err(StoreError::Domain)?;
    if outbox.etat != EtatNotificationOutbox::Prepared {
        return Err(StoreError::Invalid(
            "nouvelle notification hors état prepared",
        ));
    }
    let inserted = tx
        .execute(
            "INSERT INTO notification_outbox(
                 message_id, idempotency_key, issued_at, objective_id, delegation_id,
                 generation, event_id, policy_version, recipient, message_bytes,
                 state, last_issue_json, terminal
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'prepared',NULL,0)",
            params![
                outbox.message_id.to_string(),
                outbox.idempotency_key,
                outbox.issued_at,
                outbox.objectif_id.to_string(),
                outbox.delegation_id.map(|id| id.to_string()),
                outbox
                    .generation
                    .map(i64::try_from)
                    .transpose()
                    .map_err(|_| StoreError::Invalid("génération notification hors borne"))?,
                outbox.event_id,
                i64::try_from(outbox.policy_version)
                    .map_err(|_| StoreError::Invalid("version notification hors borne"))?,
                outbox.recipient,
                outbox.message_bytes,
            ],
        )
        .map_err(map_coordination_insert_error)?;
    if inserted != 1 {
        return Err(StoreError::Conflict("notification non enregistrée"));
    }
    Ok(())
}

fn verify_replayed_coordination_effects(
    tx: &Transaction<'_>,
    input: &EntreeReductionCoordination,
    reduction: &ReductionCoordinationActive,
) -> Result<(), StoreError> {
    if let Some(event) = input.evenement_atteste() {
        let canonical: Option<Vec<u8>> = tx
            .query_row(
                "SELECT canonical_bytes FROM coordination_events WHERE event_id = ?1",
                [event.event_id()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        match canonical {
            Some(bytes) if bytes == event.canonical_bytes() => {}
            Some(_) => return Err(StoreError::EnvelopeMismatch),
            None => return Err(StoreError::Corrupt("événement du rejeu absent")),
        }
    }
    match &reduction.transition {
        TransitionCoordinationActive::Aucune => {}
        TransitionCoordinationActive::Generation(generation) => {
            let stored: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT payload_json FROM delegation_generations
                     WHERE delegation_id = ?1 AND generation = ?2",
                    params![
                        generation.delegation_id.to_string(),
                        i64::try_from(generation.generation)
                            .map_err(|_| StoreError::Invalid("génération hors borne"))?,
                    ],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::Sql)?;
            if stored.as_deref()
                != Some(
                    serde_json::to_vec(generation)
                        .map_err(StoreError::Json)?
                        .as_slice(),
                )
            {
                return Err(StoreError::EnvelopeMismatch);
            }
        }
        TransitionCoordinationActive::ClotureEvaluee(act) => {
            let stored: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT payload_json FROM evaluated_closure_acts WHERE act_id = ?1",
                    [act.acte_id().to_string()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::Sql)?;
            let expected = serde_json::to_vec(act).map_err(StoreError::Json)?;
            if stored.as_deref() != Some(expected.as_slice()) {
                return Err(StoreError::EnvelopeMismatch);
            }
        }
    }
    for outbox in &reduction.outboxes {
        let stored: Option<(String, i64, Vec<u8>, String)> = tx
            .query_row(
                "SELECT idempotency_key, issued_at, message_bytes, state
                 FROM notification_outbox WHERE message_id = ?1",
                [outbox.message_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if stored
            != Some((
                outbox.idempotency_key.clone(),
                outbox.issued_at,
                outbox.message_bytes.clone(),
                notification_state_name(outbox.etat).to_string(),
            ))
        {
            return Err(StoreError::EnvelopeMismatch);
        }
    }
    Ok(())
}

fn coordination_event_kind_name(kind: CoordinationEventKind) -> &'static str {
    match kind {
        CoordinationEventKind::ReminderSent => "reminder_sent",
    }
}

fn coordination_freshness_name(freshness: FraicheurCoordination) -> &'static str {
    match freshness {
        FraicheurCoordination::Fresh => "fresh",
        FraicheurCoordination::Gap => "gap",
        FraicheurCoordination::Ended => "ended",
        FraicheurCoordination::Unavailable => "unavailable",
    }
}

fn active_decision_kind_name(kind: crate::domain::TypeDecisionCoordinationActive) -> &'static str {
    match kind {
        crate::domain::TypeDecisionCoordinationActive::Aucun => "aucun",
        crate::domain::TypeDecisionCoordinationActive::Notifier => "notifier",
        crate::domain::TypeDecisionCoordinationActive::Ouvrir => "ouvrir",
        crate::domain::TypeDecisionCoordinationActive::Reassigner => "reassigner",
        crate::domain::TypeDecisionCoordinationActive::InterventionHumaineRequise => {
            "intervention_humaine_requise"
        }
    }
}

fn generation_transition_allowed(
    current: EtatGenerationDelegation,
    next: EtatGenerationDelegation,
) -> bool {
    matches!(
        (current, next),
        (
            EtatGenerationDelegation::Bloquee,
            EtatGenerationDelegation::Ouverte
        ) | (
            EtatGenerationDelegation::Bloquee | EtatGenerationDelegation::Ouverte,
            EtatGenerationDelegation::Reassignee
                | EtatGenerationDelegation::Annulee
                | EtatGenerationDelegation::InterventionHumaineRequise
        )
    )
}

fn notification_state_name(state: EtatNotificationOutbox) -> &'static str {
    match state {
        EtatNotificationOutbox::Prepared => "prepared",
        EtatNotificationOutbox::OutcomeUnknown => "outcome_unknown",
        EtatNotificationOutbox::Accepted => "accepted",
        EtatNotificationOutbox::Rejected => "rejected",
    }
}

fn parse_notification_state(value: &str) -> Result<EtatNotificationOutbox, StoreError> {
    match value {
        "prepared" => Ok(EtatNotificationOutbox::Prepared),
        "outcome_unknown" => Ok(EtatNotificationOutbox::OutcomeUnknown),
        "accepted" => Ok(EtatNotificationOutbox::Accepted),
        "rejected" => Ok(EtatNotificationOutbox::Rejected),
        _ => Err(StoreError::Corrupt("état notification inconnu")),
    }
}

fn parse_generation_state(value: &str) -> Result<EtatGenerationDelegation, StoreError> {
    match value {
        "bloquee" => Ok(EtatGenerationDelegation::Bloquee),
        "ouverte" => Ok(EtatGenerationDelegation::Ouverte),
        "reassignee" => Ok(EtatGenerationDelegation::Reassignee),
        "annulee" => Ok(EtatGenerationDelegation::Annulee),
        "intervention_humaine_requise" => Ok(EtatGenerationDelegation::InterventionHumaineRequise),
        _ => Err(StoreError::Corrupt("état génération inconnu")),
    }
}

fn map_coordination_insert_error(error: rusqlite::Error) -> StoreError {
    match &error {
        rusqlite::Error::SqliteFailure(code, _) if code.code == ErrorCode::ConstraintViolation => {
            StoreError::Conflict("écriture de coordination dupliquée")
        }
        _ => StoreError::Sql(error),
    }
}

fn dependency_mode_name(mode: crate::domain::ModeQualificationDependance) -> &'static str {
    match mode {
        crate::domain::ModeQualificationDependance::HashGreffe => "hash_greffe",
        crate::domain::ModeQualificationDependance::ClotureEvalueeExigee => {
            "cloture_evaluee_exigee"
        }
    }
}

fn expected_event_name(
    kind: crate::domain::TypeEvenementAttendu,
) -> Result<&'static str, StoreError> {
    match kind {
        crate::domain::TypeEvenementAttendu::ClotureObjectif => Ok("cloture_objectif"),
        crate::domain::TypeEvenementAttendu::OuvertureDelegation => Ok("ouverture_delegation"),
    }
}

fn generation_state_name(state: EtatGenerationDelegation) -> &'static str {
    match state {
        EtatGenerationDelegation::Bloquee => "bloquee",
        EtatGenerationDelegation::Ouverte => "ouverte",
        EtatGenerationDelegation::Reassignee => "reassignee",
        EtatGenerationDelegation::Annulee => "annulee",
        EtatGenerationDelegation::InterventionHumaineRequise => "intervention_humaine_requise",
    }
}

fn reminder_episode_state_name(state: EtatEpisodeRelance) -> &'static str {
    match state {
        EtatEpisodeRelance::Actif => "actif",
        EtatEpisodeRelance::AnnuleAdministrativement => "annule_administrativement",
        EtatEpisodeRelance::Termine => "termine",
    }
}

fn reassignment_fact_kind_name(kind: TypeFaitReassignation) -> &'static str {
    match kind {
        TypeFaitReassignation::DeliveryReport => "delivery_report",
        TypeFaitReassignation::ReminderSent => "reminder_sent",
        TypeFaitReassignation::Answered => "answered",
        TypeFaitReassignation::TimedOut => "timed_out",
        TypeFaitReassignation::AnnulationAdministrative => "annulation_administrative",
    }
}

fn tracked_request_kind_name(kind: TypeEffetDemandeSuivie) -> &'static str {
    match kind {
        TypeEffetDemandeSuivie::Annuler => "annuler",
        TypeEffetDemandeSuivie::Creer => "creer",
    }
}

fn parse_tracked_request_kind(value: &str) -> Result<TypeEffetDemandeSuivie, StoreError> {
    match value {
        "annuler" => Ok(TypeEffetDemandeSuivie::Annuler),
        "creer" => Ok(TypeEffetDemandeSuivie::Creer),
        _ => Err(StoreError::Corrupt("type d'effet F29 inconnu")),
    }
}

// ---------------------------------------------------------------------------
// SPEC-087 : état de contrôle, budget d'objectifs auto-générés, focus, usage
// unique des attestations humaines, décisions humaines et dépôts vers la boîte.
// ---------------------------------------------------------------------------

/// Item à déposer dans la boîte humaine, durable jusqu'au reçu du daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HumanInboxOutboxRow {
    pub dedup_key: String,
    pub kind: String,
    pub subject_json: String,
    pub context: String,
    pub options: Vec<String>,
    pub created_at: i64,
    pub attempts: u32,
}

impl MaicieStore {
    /// Pose l'instantané lu à la relève. Jamais persisté : la commande
    /// suivante relira le daemon.
    pub fn set_control_snapshot(&mut self, snapshot: ControlSnapshot) {
        self.control = snapshot;
    }

    pub fn control_snapshot(&self) -> ControlSnapshot {
        self.control
    }

    /// Occurrences de routine différées par le contrôle du référent.
    pub fn count_control_deferred_occurrences(&self) -> Result<u32, StoreError> {
        self.connection
            .query_row(
                "SELECT COUNT(*) FROM routine_occurrences
                 WHERE state = 'differee'
                   AND reason IN ('pause', 'focus', 'budget', 'controle_inconnu')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count.max(0) as u32)
            .map_err(StoreError::Sql)
    }

    /// Origine automatique d'un objectif. `LegacyUnknown` rend `false` :
    /// l'histoire inconnue reste inconnue et ne compte pas dans le budget.
    pub fn objective_origin_is_auto(&self, objective_id: Uuid) -> Result<bool, StoreError> {
        let payload: Option<Vec<u8>> = self
            .connection
            .query_row(
                "SELECT payload_json FROM objectives WHERE id = ?1",
                [objective_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        let Some(payload) = payload else {
            return Err(StoreError::NotFound("objectif absent"));
        };
        let objective: ObjectifCoordonne =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        Ok(matches!(objective.origin, ObjectiveOrigin::AutoGenerated))
    }

    /// Objectifs ouverts d'origine automatique. Complexité : O(n) sur les
    /// objectifs non clos, n borné en pratique par le plafond et la file de
    /// focus ; le payload porte l'origine, aucune colonne n'est ajoutée.
    pub fn count_open_auto_generated_objectives(&self) -> Result<u32, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT payload_json FROM objectives WHERE state != 'clos'")
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| row.get::<_, Vec<u8>>(0))
            .map_err(StoreError::Sql)?;
        let mut count = 0u32;
        for payload in rows {
            let payload = payload.map_err(StoreError::Sql)?;
            let objective: ObjectifCoordonne =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if matches!(objective.origin, ObjectiveOrigin::AutoGenerated) {
                count = count.saturating_add(1);
            }
        }
        Ok(count)
    }

    /// Dépose durablement un item pour la boîte humaine. Idempotent par
    /// `dedup_key` : un second dépôt de la même clé ne réécrit rien.
    pub fn enqueue_human_inbox(
        &mut self,
        dedup_key: &str,
        kind: &str,
        subject_json: &str,
        context: &str,
        options: &[String],
        now: i64,
    ) -> Result<bool, StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let inserted =
            enqueue_human_inbox_tx(&tx, dedup_key, kind, subject_json, context, options, now)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(inserted)
    }

    pub fn pending_human_inbox(&self) -> Result<Vec<HumanInboxOutboxRow>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT dedup_key, kind, subject_json, context, options_json, created_at, attempts
                 FROM human_inbox_outbox WHERE state = 'prepared' ORDER BY created_at, dedup_key",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                let options_json: String = row.get(4)?;
                Ok(HumanInboxOutboxRow {
                    dedup_key: row.get(0)?,
                    kind: row.get(1)?,
                    subject_json: row.get(2)?,
                    context: row.get(3)?,
                    options: serde_json::from_str(&options_json).unwrap_or_default(),
                    created_at: row.get(5)?,
                    attempts: row.get::<_, i64>(6)?.max(0) as u32,
                })
            })
            .map_err(StoreError::Sql)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(StoreError::Sql)
    }

    pub fn mark_human_inbox_deposited(&mut self, dedup_key: &str) -> Result<(), StoreError> {
        self.connection
            .execute(
                "UPDATE human_inbox_outbox SET state = 'deposited', attempts = attempts + 1
                 WHERE dedup_key = ?1",
                [dedup_key],
            )
            .map_err(StoreError::Sql)?;
        Ok(())
    }

    pub fn mark_human_inbox_attempt(&mut self, dedup_key: &str) -> Result<(), StoreError> {
        self.connection
            .execute(
                "UPDATE human_inbox_outbox SET attempts = attempts + 1 WHERE dedup_key = ?1",
                [dedup_key],
            )
            .map_err(StoreError::Sql)?;
        Ok(())
    }

    pub fn human_decision_applied(&self, decision_id: &str) -> Result<bool, StoreError> {
        self.connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM human_decisions_applied WHERE decision_id = ?1)",
                [decision_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|found| found == 1)
            .map_err(StoreError::Sql)
    }

    /// Enregistre l'application d'une décision dans la même transaction que
    /// son effet. Rend `false` si elle l'était déjà (rejeu).
    fn record_human_decision_applied(
        tx: &Transaction<'_>,
        decision_id: &str,
        item_id: &str,
        effect: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        let inserted = tx
            .execute(
                "INSERT OR IGNORE INTO human_decisions_applied(decision_id, item_id, applied_at, effect)
                 VALUES (?1, ?2, ?3, ?4)",
                params![decision_id, item_id, now, effect],
            )
            .map_err(StoreError::Sql)?;
        Ok(inserted == 1)
    }

    /// Applique une décision « ack » (fermeture sans effet) ou « cancel »
    /// (annulation de la délégation visée) de façon transactionnelle. Une
    /// décision déjà appliquée rend `Replayed`.
    pub fn apply_human_decision(
        &mut self,
        decision_id: &str,
        item_id: &str,
        choice: &str,
        delegation_id: Option<Uuid>,
        now: i64,
    ) -> Result<HumanDecisionApplication, StoreError> {
        if self.human_decision_applied(decision_id)? {
            return Ok(HumanDecisionApplication::Replayed);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let effect = match choice {
            "ack" => "ack",
            "cancel" => {
                let Some(delegation_id) = delegation_id else {
                    return Ok(HumanDecisionApplication::Unsupported);
                };
                cancel_delegation_in_transaction(&tx, delegation_id, now)?;
                "cancel"
            }
            "raise_budget" => "ignored_daemon_side",
            _ => return Ok(HumanDecisionApplication::Unsupported),
        };
        let fresh = Self::record_human_decision_applied(&tx, decision_id, item_id, effect, now)?;
        tx.commit().map_err(StoreError::Sql)?;
        Ok(if fresh {
            HumanDecisionApplication::Applied
        } else {
            HumanDecisionApplication::Replayed
        })
    }

    /// Usage unique d'un message humain : `AlreadyConsumed` s'il a déjà ouvert
    /// un objectif.
    pub fn human_origin_consumption(
        &self,
        message_id: &str,
    ) -> Result<AttestationConsumption, StoreError> {
        let consumed: bool = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM human_origin_consumptions WHERE message_id = ?1)",
                [message_id],
                |row| row.get::<_, i64>(0),
            )
            .map(|found| found == 1)
            .map_err(StoreError::Sql)?;
        Ok(if consumed {
            AttestationConsumption::AlreadyConsumed
        } else {
            AttestationConsumption::NeverConsumed
        })
    }

    pub fn consume_human_origin(
        &mut self,
        message_id: &str,
        objective_id: Uuid,
        now: i64,
    ) -> Result<bool, StoreError> {
        let inserted = self
            .connection
            .execute(
                "INSERT OR IGNORE INTO human_origin_consumptions(message_id, objective_id, consumed_at)
                 VALUES (?1, ?2, ?3)",
                params![message_id, objective_id.to_string(), now],
            )
            .map_err(StoreError::Sql)?;
        Ok(inserted == 1)
    }

    /// Focus actif : l'objectif en position 0, s'il existe.
    pub fn focus_active(&self) -> Result<Option<Uuid>, StoreError> {
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT objective_id FROM focus_queue WHERE position = 0",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        id.as_deref().map(parse_uuid).transpose()
    }

    /// File de focus, position 0 en tête.
    pub fn focus_queue(&self) -> Result<Vec<(Uuid, i64)>, StoreError> {
        let mut statement = self
            .connection
            .prepare("SELECT objective_id, position FROM focus_queue ORDER BY position")
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(StoreError::Sql)?;
        let mut queue = Vec::new();
        for row in rows {
            let (id, position) = row.map_err(StoreError::Sql)?;
            queue.push((parse_uuid(&id)?, position));
        }
        Ok(queue)
    }

    /// Inscrit un focus. `replace` : l'objectif prend la tête et l'ancien
    /// focus recule d'une place ; sinon il rejoint la fin de la file.
    pub fn focus_enqueue(
        &mut self,
        objective_id: Uuid,
        replace: bool,
        now: i64,
    ) -> Result<(), StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        if replace {
            tx.execute("UPDATE focus_queue SET position = position + 1", [])
                .map_err(StoreError::Sql)?;
            tx.execute(
                "INSERT INTO focus_queue(objective_id, position, opened_at) VALUES (?1, 0, ?2)",
                params![objective_id.to_string(), now],
            )
            .map_err(StoreError::Sql)?;
        } else {
            let next: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(position) + 1, 0) FROM focus_queue",
                    [],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            tx.execute(
                "INSERT INTO focus_queue(objective_id, position, opened_at) VALUES (?1, ?2, ?3)",
                params![objective_id.to_string(), next, now],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Ferme le focus courant et promeut la file. Rend le nouveau focus.
    pub fn focus_close_current(&mut self) -> Result<Option<Uuid>, StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        tx.execute("DELETE FROM focus_queue WHERE position = 0", [])
            .map_err(StoreError::Sql)?;
        tx.execute("UPDATE focus_queue SET position = position - 1", [])
            .map_err(StoreError::Sql)?;
        let next: Option<String> = tx
            .query_row(
                "SELECT objective_id FROM focus_queue WHERE position = 0",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        tx.commit().map_err(StoreError::Sql)?;
        next.as_deref().map(parse_uuid).transpose()
    }

    /// Retire un objectif de la file de focus quel que soit son rang.
    pub fn focus_remove(&mut self, objective_id: Uuid) -> Result<(), StoreError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let removed: Option<i64> = tx
            .query_row(
                "SELECT position FROM focus_queue WHERE objective_id = ?1",
                [objective_id.to_string()],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(position) = removed {
            tx.execute(
                "DELETE FROM focus_queue WHERE objective_id = ?1",
                [objective_id.to_string()],
            )
            .map_err(StoreError::Sql)?;
            tx.execute(
                "UPDATE focus_queue SET position = position - 1 WHERE position > ?1",
                [position],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.commit().map_err(StoreError::Sql)
    }

    /// Ouvre atomiquement un focus humain qui attend un agent. Cette voie ne
    /// crée volontairement ni délégation ni outbox : aucune cible ne doit être
    /// inventée pendant que les agents missionnables sont indisponibles.
    ///
    /// Rend l'identifiant existant si le message humain avait déjà été
    /// consommé par une ouverture antérieure. Le guichet rejoue alors son
    /// reçu durable plutôt que de créer un second objectif.
    pub fn open_focus_waiting_for_agent(
        &mut self,
        objective: &ObjectifCoordonne,
        opening_permit: &ObjectiveOpeningPermit,
        human_message_id: &str,
        replace: bool,
        now: i64,
    ) -> Result<Option<Uuid>, StoreError> {
        if human_message_id.trim().is_empty() || now <= 0 {
            return Err(StoreError::Invalid("focus en attente invalide"));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let existing: Option<String> = tx
            .query_row(
                "SELECT objective_id FROM human_origin_consumptions WHERE message_id = ?1",
                [human_message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(StoreError::Sql)?;
        if let Some(existing) = existing {
            return Ok(Some(parse_uuid(&existing)?));
        }
        open_objective(&tx, objective, opening_permit)?;
        let consumed = tx
            .execute(
                "INSERT OR IGNORE INTO human_origin_consumptions(message_id, objective_id, consumed_at)
                 VALUES (?1, ?2, ?3)",
                params![human_message_id, objective.id.to_string(), now],
            )
            .map_err(StoreError::Sql)?;
        if consumed != 1 {
            return Err(StoreError::Conflict("origine humaine déjà consommée"));
        }
        if replace {
            tx.execute("UPDATE focus_queue SET position = position + 1", [])
                .map_err(StoreError::Sql)?;
            tx.execute(
                "INSERT INTO focus_queue(objective_id, position, opened_at) VALUES (?1, 0, ?2)",
                params![objective.id.to_string(), now],
            )
            .map_err(StoreError::Sql)?;
        } else {
            let position: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(position) + 1, 0) FROM focus_queue",
                    [],
                    |row| row.get(0),
                )
                .map_err(StoreError::Sql)?;
            tx.execute(
                "INSERT INTO focus_queue(objective_id, position, opened_at) VALUES (?1, ?2, ?3)",
                params![objective.id.to_string(), position, now],
            )
            .map_err(StoreError::Sql)?;
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(None)
    }

    /// Focus actif sans délégation, dont la durée normale est échue. La
    /// projection ne modifie rien : le dépôt idempotent vers la boîte est
    /// laissé au réconciliateur.
    pub fn overdue_focus_waiting_for_agent(
        &self,
        now: i64,
        normal_secs: u64,
    ) -> Result<Vec<(Uuid, String)>, StoreError> {
        if now <= 0 || normal_secs == 0 {
            return Err(StoreError::Invalid("délai de focus invalide"));
        }
        let mut statement = self
            .connection
            .prepare(
                "SELECT o.id, o.payload_json
                 FROM focus_queue f
                 JOIN objectives o ON o.id = f.objective_id
                 WHERE f.position = 0
                   AND NOT EXISTS (
                     SELECT 1 FROM delegations d WHERE d.objective_id = o.id
                   )",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(StoreError::Sql)?;
        let mut overdue = Vec::new();
        let delay = i64::try_from(normal_secs)
            .map_err(|_| StoreError::Invalid("délai de focus hors borne"))?;
        for row in rows {
            let (id, payload) = row.map_err(StoreError::Sql)?;
            let objective: ObjectifCoordonne =
                serde_json::from_slice(&payload).map_err(StoreError::Json)?;
            if objective.cree_at.saturating_add(delay) <= now {
                overdue.push((parse_uuid(&id)?, objective.but));
            }
        }
        Ok(overdue)
    }

    /// SPEC-087 T024 : ajoute `focus:<objective_id>` aux références du message
    /// des outboxes encore `prepared` de cet objectif, pour que la file
    /// d'exécution du daemon les serve en premier. Les octets sont réécrits
    /// avant tout envoi ; un message déjà parti n'est jamais modifié.
    pub fn add_focus_reference_to_pending_outboxes(
        &mut self,
        objective_id: Uuid,
    ) -> Result<u32, StoreError> {
        let reference = format!("focus:{objective_id}");
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let rows: Vec<(String, Vec<u8>)> = {
            let mut statement = tx
                .prepare(
                    "SELECT message_id, message_bytes FROM delegation_outbox
                     WHERE objective_id = ?1 AND state = 'prepared' AND attempted_at IS NULL",
                )
                .map_err(StoreError::Sql)?;
            let rows = statement
                .query_map([objective_id.to_string()], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })
                .map_err(StoreError::Sql)?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(StoreError::Sql)?
        };
        let mut updated = 0u32;
        for (message_id, bytes) in rows {
            let mut message: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(StoreError::Json)?;
            let Some(object) = message.as_object_mut() else {
                continue;
            };
            let references = object
                .entry("references")
                .or_insert_with(|| serde_json::Value::Array(Vec::new()));
            let Some(array) = references.as_array_mut() else {
                continue;
            };
            if array
                .iter()
                .any(|value| value.as_str() == Some(reference.as_str()))
            {
                continue;
            }
            array.push(serde_json::Value::String(reference.clone()));
            let rewritten = serde_json::to_vec(&message).map_err(StoreError::Json)?;
            tx.execute(
                "UPDATE delegation_outbox SET message_bytes = ?1 WHERE message_id = ?2",
                params![rewritten, message_id],
            )
            .map_err(StoreError::Sql)?;
            updated += 1;
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(updated)
    }

    /// Motifs de différé posés sur les dispatchs en attente (SPEC-087).
    pub fn deferred_dispatch_reasons(&self) -> Result<Vec<(Uuid, String)>, StoreError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT delegation_id, deferred_reason FROM deferred_delegation_dispatch
                 WHERE deferred_reason IS NOT NULL ORDER BY delegation_id",
            )
            .map_err(StoreError::Sql)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(StoreError::Sql)?;
        let mut out = Vec::new();
        for row in rows {
            let (id, reason) = row.map_err(StoreError::Sql)?;
            out.push((parse_uuid(&id)?, reason));
        }
        Ok(out)
    }

    /// Rejoue les dispatchs différés par la pause dont les prérequis sont
    /// clos. Ne fait rien si la garde diffère encore. Rend le nombre
    /// d'outboxes matérialisées.
    pub fn release_ready_dependents(&mut self, now: i64) -> Result<u32, StoreError> {
        if let Admission::Deferred { .. } =
            admit_autonomous_effect(AutonomousEffect::DependencyRelease, self.control)
        {
            return Ok(0);
        }
        let deferred = self.deferred_dispatch_reasons()?;
        if deferred.is_empty() {
            return Ok(0);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(StoreError::Sql)?;
        let mut released = 0u32;
        for (delegation_id, _reason) in deferred {
            let objective_id: Option<String> = tx
                .query_row(
                    "SELECT objective_id FROM delegations WHERE id = ?1",
                    [delegation_id.to_string()],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::Sql)?;
            let Some(objective_id) = objective_id else {
                continue;
            };
            let dependent_id = parse_uuid(&objective_id)?;
            if !all_objective_prerequisites_closed(&tx, dependent_id)? {
                continue;
            }
            if materialize_waiting_dependent(&tx, dependent_id, None, "reprise-controle", now)? {
                released += 1;
            }
        }
        tx.commit().map_err(StoreError::Sql)?;
        Ok(released)
    }
}

/// Dépôt durable vers la boîte humaine, dans la transaction de l'appelant.
/// Idempotent par `dedup_key` : rend `false` si la clé existe déjà.
fn enqueue_human_inbox_tx(
    tx: &Transaction<'_>,
    dedup_key: &str,
    kind: &str,
    subject_json: &str,
    context: &str,
    options: &[String],
    now: i64,
) -> Result<bool, StoreError> {
    if dedup_key.trim().is_empty() || options.is_empty() || now <= 0 {
        return Err(StoreError::Invalid("dépôt humain invalide"));
    }
    let inserted = tx
        .execute(
            "INSERT OR IGNORE INTO human_inbox_outbox(
                 dedup_key, kind, subject_json, context, options_json, created_at, state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'prepared')",
            params![
                dedup_key,
                kind,
                subject_json,
                context,
                serde_json::to_string(options).map_err(StoreError::Json)?,
                now,
            ],
        )
        .map_err(StoreError::Sql)?;
    Ok(inserted == 1)
}

/// Issue de l'application d'une décision humaine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanDecisionApplication {
    Applied,
    Replayed,
    /// Choix non pris en charge par Maicie : la décision reste non acquittée.
    Unsupported,
}

/// Annule une délégation encore vivante ; sans effet si elle est terminale.
fn cancel_delegation_in_transaction(
    tx: &Transaction<'_>,
    delegation_id: Uuid,
    _now: i64,
) -> Result<(), StoreError> {
    let stored: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM delegations WHERE id = ?1",
            [delegation_id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((_, payload)) = stored else {
        return Err(StoreError::NotFound("délégation absente"));
    };
    let mut delegation: Delegation = serde_json::from_slice(&payload).map_err(StoreError::Json)?;
    if delegation.etat.est_terminal() {
        return Ok(());
    }
    delegation.annuler().map_err(StoreError::Domain)?;
    let payload = serde_json::to_vec(&delegation).map_err(StoreError::Json)?;
    tx.execute(
        "UPDATE delegations SET state = ?1, payload_json = ?2 WHERE id = ?3",
        params![
            delegation_state_name(delegation.etat),
            payload,
            delegation_id.to_string()
        ],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "UPDATE delegation_outbox SET terminal = 1, state = 'rejected'
         WHERE delegation_id = ?1 AND terminal = 0",
        [delegation_id.to_string()],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

fn migrate(connection: &mut Connection, allow_upgrade: bool) -> Result<(), StoreError> {
    migrate_to_version(connection, allow_upgrade, SCHEMA_VERSION)
}

fn migrate_to_version(
    connection: &mut Connection,
    allow_upgrade: bool,
    target_version: i64,
) -> Result<(), StoreError> {
    // L'ouverture est un chemin concurrent normal : plusieurs processus
    // Maicie peuvent démarrer avant qu'un seul ait fini de poser le schéma.
    // Le verrou IMMEDIATE couvre donc la lecture de version et toutes les
    // migrations, pour que le second ouvre ensuite un schéma déjà cohérent.
    // Un refus (schéma trop récent ou migration non consentie) sort avant
    // toute écriture : le Drop de la transaction annule le verrou sans
    // mutation durable — oracle : user_version ET sqlite_master inchangés.
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::Sql)?;
    let current_version: i64 = tx
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(StoreError::Sql)?;
    if current_version > target_version {
        return Err(StoreError::UnsupportedSchema {
            found: current_version,
            supported: target_version,
        });
    }
    let schema_populated = database_has_user_schema(&tx)?;
    // Bootstrap sans flag : UNIQUEMENT user_version == 0 ET base vide.
    // Une base peuplée avec user_version remis à 0 n'est PAS neuve — c'est
    // une migration déguisée (porte v0) et exige le même consentement.
    if !allow_upgrade {
        let needs_consent = (current_version > 0 && current_version < target_version)
            || (current_version == 0 && schema_populated);
        if needs_consent {
            return Err(StoreError::MigrationRequired {
                found: current_version,
                supported: target_version,
            });
        }
    }
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (\n\
             version INTEGER PRIMARY KEY,\n\
             applied_at INTEGER NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS maicie_identity (\n\
             singleton INTEGER PRIMARY KEY CHECK(singleton = 1),\n\
             issuer_scope TEXT NOT NULL UNIQUE\n\
         );\n\
         CREATE TABLE IF NOT EXISTS objectives (\n\
             id TEXT PRIMARY KEY,\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegations (\n\
             id TEXT PRIMARY KEY,\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             state TEXT NOT NULL,\n\
             payload_json BLOB NOT NULL\n\
         );\n\
         CREATE TABLE IF NOT EXISTS delegation_outbox (\n\
             message_id TEXT PRIMARY KEY,\n\
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),\n\
             objective_id TEXT NOT NULL REFERENCES objectives(id),\n\
             issuer_scope TEXT NOT NULL,\n\
             issued_at INTEGER NOT NULL,\n\
             target TEXT NOT NULL,\n\
             body_bytes BLOB NOT NULL,\n\
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),\n\
             timeout_secs INTEGER NOT NULL,\n\
             deadline_contractuelle INTEGER NOT NULL,\n\
             body_hash BLOB NOT NULL,\n\
             message_bytes BLOB NOT NULL,\n\
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),\n\
             attempted_at INTEGER,\n\
             retry_until INTEGER NOT NULL,\n\
             dedup_retained_until INTEGER NOT NULL,\n\
             last_issue_json BLOB,\n\
             issue_observed_at INTEGER,\n\
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))\n\
         );\n\
         CREATE INDEX IF NOT EXISTS objectives_open_idx\n\
             ON objectives(state) WHERE state != 'clos';\n\
         CREATE INDEX IF NOT EXISTS delegation_outbox_pending_idx\n\
             ON delegation_outbox(terminal, state, retry_until, message_id);
         CREATE TABLE IF NOT EXISTS delegate_idempotency (
             idempotency_key TEXT PRIMARY KEY,
             canonical_request_bytes BLOB NOT NULL,
             objective_id TEXT NOT NULL UNIQUE REFERENCES objectives(id),
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
             message_id TEXT NOT NULL UNIQUE REFERENCES delegation_outbox(message_id),
             participant TEXT NOT NULL,
             timeout_secs INTEGER NOT NULL CHECK(timeout_secs > 0)
         );
         CREATE TABLE IF NOT EXISTS coordination_decisions (
             id TEXT PRIMARY KEY,
             objective_id TEXT NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('proposed','approved','rejected','applied')),
             payload_json BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS activation_approvals (
             id TEXT PRIMARY KEY,
             command_id TEXT NOT NULL UNIQUE,
             decision_id TEXT NOT NULL UNIQUE REFERENCES coordination_decisions(id),
             objective_id TEXT NOT NULL,
             profile_id TEXT NOT NULL,
             profile_hash BLOB NOT NULL,
             context_hash BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('proposed','approved','consumed')),
             expires_at INTEGER NOT NULL,
             consumed_at INTEGER,
             payload_json BLOB NOT NULL
         );
         CREATE TABLE IF NOT EXISTS activation_outbox (
             command_id TEXT PRIMARY KEY REFERENCES activation_approvals(command_id),
             approval_id TEXT NOT NULL UNIQUE REFERENCES activation_approvals(id),
             spawn_order_bytes BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('dispatching','outcome_unknown','applied')),
             retry_until INTEGER NOT NULL,
             dedup_retained_until INTEGER NOT NULL,
             last_issue_json BLOB,
             issue_observed_at INTEGER,
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
         );
         CREATE INDEX IF NOT EXISTS activation_outbox_pending_idx
             ON activation_outbox(terminal, state, retry_until, command_id);",
    )
    .map_err(StoreError::Sql)?;
    if current_version == 1 {
        migrate_outbox_to_rejected_state(&tx)?;
    }
    if current_version == 3 {
        tx.execute_batch(
            "ALTER TABLE activation_outbox ADD COLUMN last_issue_json BLOB;
             ALTER TABLE activation_outbox ADD COLUMN issue_observed_at INTEGER;",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 6 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS conversation_records (
                 sequence INTEGER PRIMARY KEY,
                 sender TEXT NOT NULL,
                 body_bytes BLOB NOT NULL
             );
             CREATE TRIGGER IF NOT EXISTS conversation_records_append_only_update
                 BEFORE UPDATE ON conversation_records
                 BEGIN SELECT RAISE(ABORT, 'conversation append-only'); END;
             CREATE TRIGGER IF NOT EXISTS conversation_records_append_only_delete
                 BEFORE DELETE ON conversation_records
                 BEGIN SELECT RAISE(ABORT, 'conversation append-only'); END;",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 7 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS guichet_receptions (
                 issuer_scope TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 operation TEXT NOT NULL CHECK(operation IN ('delivery_report','mission_status','deadline_question')),
                 canonical_request_bytes BLOB NOT NULL,
                 objective_id TEXT,
                 delegation_id TEXT,
                 delivery_hash TEXT,
                 in_reply_to TEXT,
                 response_message_id TEXT NOT NULL,
                 claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
                 claim_token TEXT NOT NULL,
                 outcome TEXT NOT NULL CHECK(outcome IN ('accepted','request_already_terminal')),
                 reply_bytes BLOB NOT NULL,
                 decision_id TEXT,
                 processed_at INTEGER NOT NULL,
                 PRIMARY KEY(issuer_scope, request_id),
                 UNIQUE(in_reply_to, response_message_id)
             );
             CREATE TABLE IF NOT EXISTS guichet_lifecycle_events (
                 issuer_scope TEXT NOT NULL,
                 event_id TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('answered','cancelled','timed_out')),
                 observed_at INTEGER NOT NULL,
                 in_reply_to TEXT,
                 response_message_id TEXT,
                 payload_json BLOB NOT NULL,
                 PRIMARY KEY(issuer_scope, event_id),
                 UNIQUE(issuer_scope, request_id)
             );
             CREATE TABLE IF NOT EXISTS guichet_correlations (
                 in_reply_to TEXT NOT NULL,
                 response_message_id TEXT NOT NULL,
                 issuer_scope TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 lifecycle_event_id TEXT,
                 lifecycle_state TEXT CHECK(lifecycle_state IN ('answered','cancelled','timed_out')),
                 PRIMARY KEY(in_reply_to, response_message_id),
                 UNIQUE(issuer_scope, request_id)
             );",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 8 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS guichet_refusal_receptions (
                 issuer_scope TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 canonical_request_bytes BLOB NOT NULL,
                 operation TEXT NOT NULL CHECK(operation IN ('delivery_report','mission_status','deadline_question')),
                 reason TEXT NOT NULL CHECK(reason IN (
                     'delegation_missing','relation_invalid','envelope_mismatch',
                     'review_verdict_required','review_verdict_unexpected',
                     'review_mandate_mismatch','target_head_moved','measured_head_mismatch',
                     'target_head_moved_and_measured_head_mismatch'
                 )),
                 response_message_id TEXT NOT NULL,
                 reply_bytes BLOB NOT NULL,
                 claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
                 claim_token TEXT NOT NULL,
                 processed_at INTEGER NOT NULL,
                 PRIMARY KEY(issuer_scope, request_id, canonical_request_bytes)
             );",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 9 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS coordination_events (
                 event_id TEXT PRIMARY KEY,
                 request_id TEXT NOT NULL,
                 kind TEXT NOT NULL CHECK(kind = 'reminder_sent'),
                 reminder_message_id TEXT NOT NULL,
                 recipient TEXT NOT NULL,
                 transport_generation INTEGER NOT NULL CHECK(transport_generation > 0),
                 cursor INTEGER NOT NULL UNIQUE CHECK(cursor > 0),
                 freshness TEXT NOT NULL CHECK(freshness IN ('fresh','gap','ended','unavailable')),
                 observed_at INTEGER NOT NULL CHECK(observed_at > 0),
                 canonical_bytes BLOB NOT NULL
             );
             CREATE TABLE IF NOT EXISTS coordination_expectations (
                 expectation_id TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT REFERENCES delegations(id),
                 event_kind TEXT NOT NULL CHECK(event_kind IN ('cloture_objectif','ouverture_delegation')),
                 recipient TEXT NOT NULL,
                 policy_version INTEGER NOT NULL CHECK(policy_version > 0),
                 payload_json BLOB NOT NULL,
                 UNIQUE(objective_id, delegation_id, event_kind, recipient, policy_version)
             );
             CREATE TABLE IF NOT EXISTS delegation_dependencies (
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 prerequisite_id TEXT NOT NULL REFERENCES delegations(id),
                 dependent_id TEXT NOT NULL REFERENCES delegations(id),
                 qualification_mode TEXT NOT NULL CHECK(qualification_mode IN ('hash_greffe','cloture_evaluee_exigee')),
                 payload_json BLOB NOT NULL,
                 PRIMARY KEY(prerequisite_id, dependent_id),
                 CHECK(prerequisite_id != dependent_id)
             );
             CREATE INDEX IF NOT EXISTS delegation_dependencies_prerequisite_idx
                 ON delegation_dependencies(prerequisite_id, dependent_id);
             CREATE INDEX IF NOT EXISTS delegation_dependencies_dependent_idx
                 ON delegation_dependencies(dependent_id, prerequisite_id);
             CREATE TABLE IF NOT EXISTS evaluated_closure_acts (
                 act_id TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT NOT NULL REFERENCES delegations(id),
                 generation INTEGER NOT NULL CHECK(generation > 0),
                 delivery_hash TEXT NOT NULL,
                 evaluated_at INTEGER NOT NULL CHECK(evaluated_at > 0),
                 payload_json BLOB NOT NULL,
                 UNIQUE(delegation_id, generation, delivery_hash),
                 FOREIGN KEY(delegation_id, generation)
                     REFERENCES delegation_generations(delegation_id, generation)
             );
             CREATE TABLE IF NOT EXISTS reassignment_policies (
                 delegation_id TEXT PRIMARY KEY REFERENCES delegations(id),
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 policy_version INTEGER NOT NULL CHECK(policy_version > 0),
                 reminder_threshold INTEGER NOT NULL CHECK(reminder_threshold > 0),
                 max_reemissions INTEGER NOT NULL CHECK(max_reemissions BETWEEN 1 AND 8),
                 payload_json BLOB NOT NULL
             );
             CREATE TABLE IF NOT EXISTS delegation_lineages (
                 delegation_id TEXT PRIMARY KEY REFERENCES delegations(id),
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 active_generation INTEGER NOT NULL CHECK(active_generation > 0),
                 payload_json BLOB NOT NULL
             );
             CREATE TABLE IF NOT EXISTS delegation_generations (
                 delegation_id TEXT NOT NULL REFERENCES delegation_lineages(delegation_id),
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 generation INTEGER NOT NULL CHECK(generation > 0),
                 participant_id TEXT NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('bloquee','ouverte','reassignee','annulee','intervention_humaine_requise')),
                 payload_json BLOB NOT NULL,
                 PRIMARY KEY(delegation_id, generation)
             );
             CREATE UNIQUE INDEX IF NOT EXISTS delegation_generations_active_idx
                 ON delegation_generations(delegation_id)
                 WHERE state IN ('bloquee','ouverte');
             CREATE TABLE IF NOT EXISTS reminder_episodes (
                 delegation_id TEXT NOT NULL,
                 generation INTEGER NOT NULL,
                 request_id TEXT NOT NULL,
                 request_ordinal INTEGER NOT NULL CHECK(request_ordinal > 0),
                 reminder_count INTEGER NOT NULL CHECK(reminder_count >= 0),
                 reemissions_used INTEGER NOT NULL CHECK(reemissions_used BETWEEN 0 AND 8),
                 state TEXT NOT NULL CHECK(state IN ('actif','annule_administrativement','termine')),
                 payload_json BLOB NOT NULL,
                 PRIMARY KEY(delegation_id, generation, request_ordinal),
                 UNIQUE(request_id),
                 FOREIGN KEY(delegation_id, generation)
                     REFERENCES delegation_generations(delegation_id, generation)
             );
             CREATE TABLE IF NOT EXISTS active_coordination_decisions (
                 decision_id TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT NOT NULL,
                 generation INTEGER NOT NULL CHECK(generation > 0),
                 event_id TEXT NOT NULL,
                 policy_version INTEGER NOT NULL CHECK(policy_version > 0),
                 kind TEXT NOT NULL CHECK(kind IN ('aucun','notifier','ouvrir','reassigner','intervention_humaine_requise')),
                 payload_json BLOB NOT NULL,
                 UNIQUE(delegation_id, generation, event_id),
                 FOREIGN KEY(delegation_id, generation)
                     REFERENCES delegation_generations(delegation_id, generation)
             );
             CREATE TABLE IF NOT EXISTS notification_outbox (
                 message_id TEXT PRIMARY KEY,
                 idempotency_key TEXT NOT NULL UNIQUE,
                 issued_at INTEGER NOT NULL CHECK(issued_at > 0),
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT,
                 generation INTEGER,
                 event_id TEXT NOT NULL,
                 policy_version INTEGER NOT NULL CHECK(policy_version > 0),
                 recipient TEXT NOT NULL,
                 message_bytes BLOB NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),
                 last_issue_json BLOB,
                 terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0,1)),
                 FOREIGN KEY(delegation_id, generation)
                     REFERENCES delegation_generations(delegation_id, generation)
             );
             CREATE INDEX IF NOT EXISTS notification_outbox_pending_idx
                 ON notification_outbox(terminal, state, message_id);",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version == 9 {
        // Une ancienne notification ne possède aucune preuve permettant de
        // reconstruire son horodatage canonique. La colonne reste donc NULL
        // pour ces lignes et la lecture des pending échoue fermée, au lieu de
        // fabriquer une enveloppe différente au rejeu.
        tx.execute_batch(
            "ALTER TABLE notification_outbox
             ADD COLUMN issued_at INTEGER CHECK(issued_at > 0);",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 11 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS reassignment_events (
                 event_id TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT NOT NULL REFERENCES delegations(id),
                 generation INTEGER NOT NULL CHECK(generation > 0),
                 batch_event_id TEXT NOT NULL,
                 request_id TEXT NOT NULL,
                 kind TEXT NOT NULL CHECK(kind IN (
                     'delivery_report','reminder_sent','answered','timed_out','annulation_administrative'
                 )),
                 payload_json BLOB NOT NULL
             );
             CREATE INDEX IF NOT EXISTS reassignment_events_request_idx
                 ON reassignment_events(delegation_id, generation, request_id, event_id);
             CREATE TABLE IF NOT EXISTS reassignment_reductions (
                 decision_id TEXT PRIMARY KEY REFERENCES active_coordination_decisions(decision_id),
                 payload_json BLOB NOT NULL
             );
             CREATE TABLE IF NOT EXISTS tracked_request_outbox (
                 effect_id TEXT PRIMARY KEY,
                 issued_at INTEGER NOT NULL CHECK(issued_at > 0),
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 delegation_id TEXT NOT NULL REFERENCES delegations(id),
                 generation INTEGER NOT NULL CHECK(generation > 0),
                 event_id TEXT NOT NULL,
                 policy_version INTEGER NOT NULL CHECK(policy_version > 0),
                 kind TEXT NOT NULL CHECK(kind IN ('annuler','creer')),
                 request_id TEXT NOT NULL,
                 recipient TEXT NOT NULL,
                 message_bytes BLOB NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),
                 last_issue_json BLOB,
                 terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0,1)),
                 FOREIGN KEY(delegation_id, generation)
                     REFERENCES delegation_generations(delegation_id, generation)
             );
             CREATE INDEX IF NOT EXISTS tracked_request_outbox_pending_idx
                 ON tracked_request_outbox(terminal, state, effect_id);",
        )
        .map_err(StoreError::Sql)?;
    }
    if current_version < 12 {
        // F36+F37 : arêtes OBJECTIF→OBJECTIF + paramètres de dispatch différé.
        // message_id nullable dans delegate_idempotency : une délégation
        // EnAttentePrerequis n'a pas d'outbox (voie A — pas d'intention bâillonnée).
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS objective_dependencies (
                 dependent_objective_id TEXT NOT NULL REFERENCES objectives(id),
                 prerequisite_objective_id TEXT NOT NULL REFERENCES objectives(id),
                 PRIMARY KEY (dependent_objective_id, prerequisite_objective_id),
                 CHECK (dependent_objective_id != prerequisite_objective_id)
             );
             CREATE INDEX IF NOT EXISTS objective_dependencies_prerequisite_idx
                 ON objective_dependencies(prerequisite_objective_id, dependent_objective_id);
             CREATE TABLE IF NOT EXISTS deferred_delegation_dispatch (
                 delegation_id TEXT PRIMARY KEY REFERENCES delegations(id),
                 reply INTEGER NOT NULL CHECK(reply IN (0, 1)),
                 timeout_secs INTEGER NOT NULL CHECK(timeout_secs > 0),
                 retry_until INTEGER NOT NULL,
                 dedup_retained_until INTEGER NOT NULL,
                 max_frame_bytes INTEGER NOT NULL CHECK(max_frame_bytes > 0),
                 idempotency_key TEXT NOT NULL,
                 issuer_scope TEXT NOT NULL
             );
             ALTER TABLE delegate_idempotency RENAME TO delegate_idempotency_v11;
             CREATE TABLE delegate_idempotency (
                 idempotency_key TEXT PRIMARY KEY,
                 canonical_request_bytes BLOB NOT NULL,
                 objective_id TEXT NOT NULL UNIQUE REFERENCES objectives(id),
                 delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
                 message_id TEXT UNIQUE REFERENCES delegation_outbox(message_id),
                 participant TEXT NOT NULL,
                 timeout_secs INTEGER NOT NULL CHECK(timeout_secs > 0),
                 deadline_contractuelle INTEGER NOT NULL
             );
             INSERT INTO delegate_idempotency(
                 idempotency_key, canonical_request_bytes, objective_id, delegation_id,
                 message_id, participant, timeout_secs, deadline_contractuelle
             )
             SELECT i.idempotency_key, i.canonical_request_bytes, i.objective_id, i.delegation_id,
                    i.message_id, i.participant, i.timeout_secs, o.deadline_contractuelle
             FROM delegate_idempotency_v11 i
             JOIN delegation_outbox o ON o.message_id = i.message_id;
             DROP TABLE delegate_idempotency_v11;",
        )
        .map_err(StoreError::Sql)?;
    }
    // L4 : objective_costs en v13, derrière la v12 réelle (F36+F37).
    if current_version < 13 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS objective_costs (
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 agent TEXT NOT NULL,
                 from_secs INTEGER NOT NULL CHECK(from_secs > 0),
                 to_secs INTEGER NOT NULL CHECK(to_secs >= from_secs),
                 attested INTEGER NOT NULL CHECK(attested IN (0, 1)),
                 turns INTEGER,
                 input_tokens INTEGER,
                 output_tokens INTEGER,
                 cache_creation_input_tokens INTEGER,
                 cache_read_input_tokens INTEGER,
                 facturable_tokens INTEGER,
                 PRIMARY KEY (objective_id, agent)
             );",
        )
        .map_err(StoreError::Sql)?;
    }
    // Politique 31 : registre des plages (comparaison exacte de noms).
    if current_version < 14 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS resource_range_reservations (
                 resource_name TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL REFERENCES objectives(id),
                 reserved_at INTEGER NOT NULL CHECK(reserved_at > 0)
             );
             CREATE INDEX IF NOT EXISTS resource_range_reservations_objective_idx
                 ON resource_range_reservations(objective_id);",
        )
        .map_err(StoreError::Sql)?;
    }
    // Routines : gabarit de delegate + calendrier (M5 / note cursor7).
    if current_version < 15 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS routines (
                 id TEXT PRIMARY KEY,
                 goal TEXT NOT NULL,
                 participant TEXT NOT NULL,
                 period_secs INTEGER NOT NULL CHECK(period_secs >= 60),
                 suite_kind TEXT NOT NULL CHECK(suite_kind IN ('aucune', 'objectif')),
                 suite_objective_id TEXT,
                 depends_on_json TEXT NOT NULL,
                 references_json TEXT NOT NULL,
                 template_hash BLOB NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('proposed', 'active', 'paused')),
                 proposed_at INTEGER NOT NULL CHECK(proposed_at > 0),
                 approved_at INTEGER,
                 paused_at INTEGER,
                 last_bucket INTEGER
             );
             CREATE INDEX IF NOT EXISTS routines_state_idx ON routines(state);
             CREATE TABLE IF NOT EXISTS routine_occurrences (
                 routine_id TEXT NOT NULL REFERENCES routines(id),
                 bucket INTEGER NOT NULL,
                 state TEXT NOT NULL CHECK(state IN ('ouverte', 'sautee', 'differee', 'terminee')),
                 reason TEXT,
                 objective_id TEXT REFERENCES objectives(id),
                 delegation_id TEXT REFERENCES delegations(id),
                 created_at INTEGER NOT NULL CHECK(created_at > 0),
                 PRIMARY KEY (routine_id, bucket)
             );
             CREATE INDEX IF NOT EXISTS routine_occurrences_state_idx
                 ON routine_occurrences(routine_id, state, bucket);",
        )
        .map_err(StoreError::Sql)?;
    }
    // v16 : solde les délégations ouvertes sur objectifs déjà clos.
    if current_version < 16 {
        migrate_orphan_delegations_on_closed_objectives(&tx)?;
    }
    // v17 : élargit le vocabulaire fermé des refus aux preuves de revue.
    if current_version < 17 {
        migrate_review_refusal_reasons_v17(&tx)?;
    }
    // v18 : refus locaux de contrainte, append-only et à motif fermé.
    if current_version < 18 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_delegate_refusals (
                 sequence INTEGER PRIMARY KEY,
                 observed_at INTEGER NOT NULL CHECK(observed_at > 0),
                 reason TEXT NOT NULL CHECK(reason IN (
                     'suite_none_with_unclassified_citation'
                 )),
                 cited_objective_id TEXT NOT NULL REFERENCES objectives(id)
             );
             CREATE INDEX IF NOT EXISTS local_delegate_refusals_reason_idx
                 ON local_delegate_refusals(reason, sequence);
             CREATE TRIGGER IF NOT EXISTS local_delegate_refusals_append_only_update
                 BEFORE UPDATE ON local_delegate_refusals
                 BEGIN SELECT RAISE(ABORT, 'local delegate refusal append-only'); END;
             CREATE TRIGGER IF NOT EXISTS local_delegate_refusals_append_only_delete
                 BEFORE DELETE ON local_delegate_refusals
                 BEGIN SELECT RAISE(ABORT, 'local delegate refusal append-only'); END;",
        )
        .map_err(StoreError::Sql)?;
    }
    // v19 : le vocabulaire autorisé reste fermé par les enums Rust ; la table
    // durable des refus conserve aussi le nom d'une tentative rejetée.
    if target_version >= 19 {
        if current_version < 19 {
            verify_local_delegate_refusals_shape_v18(&tx)?;
            migrate_guichet_refusal_vocabulary_v19(&tx)?;
        }
        verify_guichet_refusal_shape_v19(&tx)?;
    }
    // v20 : la table des reçus ne recopie plus l'enum des opérations dans
    // un CHECK SQL. Rust reste fermé et refuse une valeur inconnue à la
    // lecture ; une nouvelle variante légitime ne requiert plus une seconde
    // liste silencieuse dans le DDL.
    if target_version >= 20 {
        if current_version < 20 {
            verify_guichet_reception_shape_v19(&tx)?;
            migrate_guichet_reception_vocabulary_v20(&tx)?;
        }
        verify_guichet_reception_shape_v20(&tx)?;
    }
    // v21 : références opaques Bridget et dernière projection runtime par
    // délégation. Aucun trigger ne touche objectifs ou décisions.
    // Le garde de schéma lit ce seuil séparé pour prévenir les collisions de migrations.
    #[allow(clippy::collapsible_if)]
    if target_version >= 21 {
        if current_version < 21 {
            tx.execute_batch(
                "CREATE TABLE IF NOT EXISTS delegation_execution_projections (
                 delegation_id TEXT PRIMARY KEY REFERENCES delegations(id),
                 execution_id TEXT NOT NULL UNIQUE,
                 payload_json BLOB NOT NULL,
                 observation_cursor INTEGER NOT NULL CHECK(observation_cursor >= 0),
                 source_generation INTEGER NOT NULL CHECK(source_generation >= 0),
                 observed_at INTEGER NOT NULL CHECK(observed_at >= 0)
             );
             CREATE INDEX IF NOT EXISTS delegation_execution_projections_cursor_idx
                 ON delegation_execution_projections(observation_cursor);",
            )
            .map_err(StoreError::Sql)?;
        }
    }
    // v22 : identité projet Maicie, commande idempotente et outbox locale.
    // La racine demandée reste une intention : Bridget seul la canonise et
    // l'autorise avant d'activer l'identité.
    if target_version >= 22 && current_version < 22 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS project_identities (
                     project_id TEXT PRIMARY KEY,
                     display_name TEXT NOT NULL,
                     status TEXT NOT NULL CHECK(status IN (
                         'pending_binding', 'active', 'registration_conflict', 'disabled'
                     )),
                     created_at INTEGER NOT NULL,
                     updated_at INTEGER NOT NULL,
                     registration_command_id TEXT NOT NULL UNIQUE
                 );
                 CREATE INDEX IF NOT EXISTS project_identities_status_idx
                     ON project_identities(status, updated_at, project_id);
                 CREATE TABLE IF NOT EXISTS project_registration_commands (
                     command_id TEXT PRIMARY KEY,
                     canonical_payload BLOB NOT NULL,
                     proposed_project_id TEXT NOT NULL UNIQUE
                         REFERENCES project_identities(project_id),
                     resolved_project_id TEXT,
                     requested_root TEXT NOT NULL,
                     state TEXT NOT NULL CHECK(state IN (
                         'prepared', 'binding', 'bound', 'failed', 'expired'
                     )),
                     retry_until INTEGER NOT NULL,
                     outcome_json BLOB,
                     outcome_observed_at INTEGER
                 );
                 CREATE INDEX IF NOT EXISTS project_registration_commands_pending_idx
                     ON project_registration_commands(state, retry_until, command_id);
                 CREATE TABLE IF NOT EXISTS project_registration_outbox (
                     command_id TEXT PRIMARY KEY
                         REFERENCES project_registration_commands(command_id),
                     canonical_request BLOB NOT NULL,
                     state TEXT NOT NULL CHECK(state IN (
                         'prepared', 'outcome_unknown', 'applied', 'rejected'
                     ))
                 );
                 CREATE INDEX IF NOT EXISTS project_registration_outbox_pending_idx
                     ON project_registration_outbox(state, command_id);",
        )
        .map_err(StoreError::Sql)?;
    }

    // v23 : profil projet et approbation locale durables. Les payloads ne
    // contiennent que references, digests et generations, jamais un secret.
    if current_version < 23 && target_version >= 23 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS project_profiles (
                 profile_id TEXT PRIMARY KEY,
                 project_id TEXT NOT NULL,
                 state TEXT NOT NULL,
                 payload_json BLOB NOT NULL,
                 approval_json BLOB,
                 updated_at INTEGER NOT NULL CHECK(updated_at >= 0)
             );
             CREATE INDEX IF NOT EXISTS project_profiles_project_idx
                 ON project_profiles(project_id, state, updated_at);",
        )
        .map_err(StoreError::Sql)?;
    }

    // v24 (SPEC-087) : focus du référent, usage unique des attestations
    // d'origine humaine, décisions humaines appliquées, motif de différé des
    // dispatchs en attente, et file de dépôt vers la boîte humaine.
    if current_version < 24 && target_version >= 24 {
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS focus_queue (
                 objective_id TEXT PRIMARY KEY REFERENCES objectives(id),
                 position INTEGER NOT NULL CHECK(position >= 0),
                 opened_at INTEGER NOT NULL CHECK(opened_at > 0)
             );
             CREATE TABLE IF NOT EXISTS human_origin_consumptions (
                 message_id TEXT PRIMARY KEY,
                 objective_id TEXT NOT NULL,
                 consumed_at INTEGER NOT NULL CHECK(consumed_at > 0)
             );
             CREATE TABLE IF NOT EXISTS human_decisions_applied (
                 decision_id TEXT PRIMARY KEY,
                 item_id TEXT NOT NULL,
                 applied_at INTEGER NOT NULL CHECK(applied_at > 0),
                 effect TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS human_inbox_outbox (
                 dedup_key TEXT PRIMARY KEY,
                 kind TEXT NOT NULL,
                 subject_json TEXT NOT NULL,
                 context TEXT NOT NULL,
                 options_json TEXT NOT NULL,
                 created_at INTEGER NOT NULL CHECK(created_at > 0),
                 state TEXT NOT NULL CHECK(state IN ('prepared', 'deposited')),
                 attempts INTEGER NOT NULL DEFAULT 0
             );
             ALTER TABLE deferred_delegation_dispatch ADD COLUMN deferred_reason TEXT;",
        )
        .map_err(StoreError::Sql)?;
    }

    for version in (current_version + 1)..=target_version {
        tx.execute(
            "INSERT OR IGNORE INTO schema_migrations(version, applied_at)\n\
             VALUES (?1, CAST(strftime('%s','now') AS INTEGER))",
            [version],
        )
        .map_err(StoreError::Sql)?;
    }
    tx.pragma_update(None, "user_version", target_version)
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)
}

/// Migration v17 : reconstruit la table pour élargir son CHECK, sans modifier
/// les octets ni le motif des refus historiques.
fn migrate_review_refusal_reasons_v17(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "ALTER TABLE guichet_refusal_receptions RENAME TO guichet_refusal_receptions_v16;
         CREATE TABLE guichet_refusal_receptions (
             issuer_scope TEXT NOT NULL,
             request_id TEXT NOT NULL,
             canonical_request_bytes BLOB NOT NULL,
             operation TEXT NOT NULL CHECK(operation IN ('delivery_report','mission_status','deadline_question')),
             reason TEXT NOT NULL CHECK(reason IN (
                 'delegation_missing','relation_invalid','envelope_mismatch',
                 'review_verdict_required','review_verdict_unexpected',
                 'review_mandate_mismatch','target_head_moved','measured_head_mismatch',
                 'target_head_moved_and_measured_head_mismatch'
             )),
             response_message_id TEXT NOT NULL,
             reply_bytes BLOB NOT NULL,
             claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
             claim_token TEXT NOT NULL,
             processed_at INTEGER NOT NULL,
             PRIMARY KEY(issuer_scope, request_id, canonical_request_bytes)
         );
         INSERT INTO guichet_refusal_receptions(
             issuer_scope, request_id, canonical_request_bytes, operation, reason,
             response_message_id, reply_bytes, claim_generation, claim_token, processed_at
         )
         SELECT issuer_scope, request_id, canonical_request_bytes, operation, reason,
                response_message_id, reply_bytes, claim_generation, claim_token, processed_at
         FROM guichet_refusal_receptions_v16;
         DROP TABLE guichet_refusal_receptions_v16;",
    )
    .map_err(StoreError::Sql)
}

/// Vérifie le vrai comportement v18 avant de marquer v19. La sonde crée ses
/// deux lignes sous savepoint, exige les deux gardes append-only, puis annule
/// toujours l'ensemble — succès comme échec.
fn verify_local_delegate_refusals_shape_v18(tx: &Transaction<'_>) -> Result<(), StoreError> {
    let objectives_before: i64 = tx
        .query_row("SELECT COUNT(*) FROM objectives", [], |row| row.get(0))
        .map_err(|_| StoreError::Corrupt("forme v18 du greffe local incompatible avec v19"))?;
    let refusals_before: i64 = tx
        .query_row("SELECT COUNT(*) FROM local_delegate_refusals", [], |row| {
            row.get(0)
        })
        .map_err(|_| StoreError::Corrupt("forme v18 du greffe local incompatible avec v19"))?;
    tx.execute_batch("SAVEPOINT maicie_v19_preflight_v18")
        .map_err(StoreError::Sql)?;
    let inserted = tx.execute_batch(
        "INSERT INTO objectives(id,state,payload_json)
         VALUES('maicie-v19-preflight-' || lower(hex(randomblob(16))),'ouvert',X'7B7D');
         INSERT INTO local_delegate_refusals(observed_at,reason,cited_objective_id)
         SELECT 1,'suite_none_with_unclassified_citation',id
         FROM objectives WHERE id LIKE 'maicie-v19-preflight-%';",
    );
    let update_rejected = tx
        .execute(
            "UPDATE local_delegate_refusals SET observed_at=2
             WHERE cited_objective_id LIKE 'maicie-v19-preflight-%'",
            [],
        )
        .is_err();
    let delete_rejected = tx
        .execute(
            "DELETE FROM local_delegate_refusals
             WHERE cited_objective_id LIKE 'maicie-v19-preflight-%'",
            [],
        )
        .is_err();
    let cleanup = tx.execute_batch(
        "ROLLBACK TO maicie_v19_preflight_v18;
         RELEASE maicie_v19_preflight_v18;",
    );
    cleanup.map_err(StoreError::Sql)?;
    let objectives_after: i64 = tx
        .query_row("SELECT COUNT(*) FROM objectives", [], |row| row.get(0))
        .map_err(StoreError::Sql)?;
    let refusals_after: i64 = tx
        .query_row("SELECT COUNT(*) FROM local_delegate_refusals", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sql)?;
    if inserted.is_err()
        || !update_rejected
        || !delete_rejected
        || objectives_before != objectives_after
        || refusals_before != refusals_after
    {
        return Err(StoreError::Corrupt(
            "forme v18 du greffe local incompatible avec v19",
        ));
    }
    Ok(())
}

/// Migration v19 : seule la table des refus fédérés est reconstruite. Les
/// contraintes structurelles restent en SQL ; les vocabulaires opération et
/// motif sont validés fail-closed lors de chaque lecture par les enums Rust.
fn migrate_guichet_refusal_vocabulary_v19(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "ALTER TABLE guichet_refusal_receptions RENAME TO guichet_refusal_receptions_v18;
         CREATE TABLE guichet_refusal_receptions (
             issuer_scope TEXT NOT NULL,
             request_id TEXT NOT NULL,
             canonical_request_bytes BLOB NOT NULL,
             operation TEXT NOT NULL,
             reason TEXT NOT NULL,
             response_message_id TEXT NOT NULL,
             reply_bytes BLOB NOT NULL,
             claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
             claim_token TEXT NOT NULL,
             processed_at INTEGER NOT NULL,
             PRIMARY KEY(issuer_scope, request_id, canonical_request_bytes)
         );
         INSERT INTO guichet_refusal_receptions(
             issuer_scope,request_id,canonical_request_bytes,operation,reason,
             response_message_id,reply_bytes,claim_generation,claim_token,processed_at
         )
         SELECT issuer_scope,request_id,canonical_request_bytes,operation,reason,
                response_message_id,reply_bytes,claim_generation,claim_token,processed_at
         FROM guichet_refusal_receptions_v18;
         DROP TABLE guichet_refusal_receptions_v18;",
    )
    .map_err(StoreError::Sql)
}

/// Une base déjà estampillée v19 doit accepter le vocabulaire durable ouvert
/// sans conserver la ligne de sonde. La lecture reste fermée côté Rust.
fn verify_guichet_refusal_shape_v19(tx: &Transaction<'_>) -> Result<(), StoreError> {
    let before: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM guichet_refusal_receptions",
            [],
            |row| row.get(0),
        )
        .map_err(|_| StoreError::Corrupt("forme v19 des refus fédérés incomplète"))?;
    tx.execute_batch("SAVEPOINT maicie_v19_refusal_shape")
        .map_err(StoreError::Sql)?;
    let probe = tx.execute(
        "INSERT INTO guichet_refusal_receptions(
             issuer_scope,request_id,canonical_request_bytes,operation,reason,
             response_message_id,reply_bytes,claim_generation,claim_token,processed_at
         ) VALUES('maicie-v19-shape',lower(hex(randomblob(16))),randomblob(16),
                  'future_operation','future_reason',lower(hex(randomblob(16))),X'7B7D',
                  1,'maicie-v19-shape',1)",
        [],
    );
    let cleanup = tx.execute_batch(
        "ROLLBACK TO maicie_v19_refusal_shape;
         RELEASE maicie_v19_refusal_shape;",
    );
    cleanup.map_err(StoreError::Sql)?;
    let after: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM guichet_refusal_receptions",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    if probe.is_err() || before != after {
        return Err(StoreError::Corrupt(
            "forme v19 des refus fédérés incomplète",
        ));
    }
    Ok(())
}

/// Une vraie v19 porte encore le vocabulaire historique fermé dans le DDL.
/// La sonde prouve simultanément qu'une opération historique entre et que
/// `delegate` n'entre pas, puis annule les deux essais dans tous les cas.
fn verify_guichet_reception_shape_v19(tx: &Transaction<'_>) -> Result<(), StoreError> {
    let before: i64 = tx
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .map_err(|_| StoreError::Corrupt("forme v19 des reçus du guichet incompatible avec v20"))?;
    let suffix = Uuid::new_v4();
    let historical_request = format!("maicie-v20-history-{suffix}");
    let mutation_request = format!("maicie-v20-mutation-{suffix}");
    tx.execute_batch("SAVEPOINT maicie_v20_preflight_v19")
        .map_err(StoreError::Sql)?;
    let historical = tx.execute(
        "INSERT INTO guichet_receptions(
             issuer_scope,request_id,operation,canonical_request_bytes,
             response_message_id,claim_generation,claim_token,outcome,reply_bytes,processed_at
         ) VALUES('maicie-v20-shape',?1,'delivery_report',X'01',?2,1,?3,'accepted',X'02',1)",
        params![
            historical_request,
            format!("response-{suffix}"),
            format!("claim-{suffix}"),
        ],
    );
    let mutation_rejected = tx
        .execute(
            "INSERT INTO guichet_receptions(
                 issuer_scope,request_id,operation,canonical_request_bytes,
                 response_message_id,claim_generation,claim_token,outcome,reply_bytes,processed_at
             ) VALUES('maicie-v20-shape',?1,'delegate',X'03',?2,1,?3,'accepted',X'04',1)",
            params![
                mutation_request,
                format!("response-mutation-{suffix}"),
                format!("claim-mutation-{suffix}"),
            ],
        )
        .is_err();
    tx.execute_batch(
        "ROLLBACK TO maicie_v20_preflight_v19;
         RELEASE maicie_v20_preflight_v19;",
    )
    .map_err(StoreError::Sql)?;
    let after: i64 = tx
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sql)?;
    if historical.is_err() || !mutation_rejected || before != after {
        return Err(StoreError::Corrupt(
            "forme v19 des reçus du guichet incompatible avec v20",
        ));
    }
    Ok(())
}

/// Migration v20 : les contraintes structurelles restent en SQL, tandis que
/// le vocabulaire fermé appartient exclusivement à `OperationGuichet`.
fn migrate_guichet_reception_vocabulary_v20(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "ALTER TABLE guichet_receptions RENAME TO guichet_receptions_v19;
         CREATE TABLE guichet_receptions (
             issuer_scope TEXT NOT NULL,
             request_id TEXT NOT NULL,
             operation TEXT NOT NULL,
             canonical_request_bytes BLOB NOT NULL,
             objective_id TEXT,
             delegation_id TEXT,
             delivery_hash TEXT,
             in_reply_to TEXT,
             response_message_id TEXT NOT NULL,
             claim_generation INTEGER NOT NULL CHECK(claim_generation >= 0),
             claim_token TEXT NOT NULL,
             outcome TEXT NOT NULL CHECK(outcome IN ('accepted','request_already_terminal')),
             reply_bytes BLOB NOT NULL,
             decision_id TEXT,
             processed_at INTEGER NOT NULL,
             PRIMARY KEY(issuer_scope, request_id),
             UNIQUE(in_reply_to, response_message_id)
         );
         INSERT INTO guichet_receptions(
             issuer_scope,request_id,operation,canonical_request_bytes,
             objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
             claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
         )
         SELECT issuer_scope,request_id,operation,canonical_request_bytes,
                objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
                claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
         FROM guichet_receptions_v19;
         DROP TABLE guichet_receptions_v19;",
    )
    .map_err(StoreError::Sql)
}

/// Une v20 accepte le stockage d'un nom futur mais sa lecture Rust le refuse
/// fail-closed. La sonde reste entièrement sous savepoint.
fn verify_guichet_reception_shape_v20(tx: &Transaction<'_>) -> Result<(), StoreError> {
    let before: i64 = tx
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .map_err(|_| StoreError::Corrupt("forme v20 des reçus du guichet incomplète"))?;
    let suffix = Uuid::new_v4();
    let request_id = format!("maicie-v20-unknown-{suffix}");
    tx.execute_batch("SAVEPOINT maicie_v20_reception_shape")
        .map_err(StoreError::Sql)?;
    let inserted = tx.execute(
        "INSERT INTO guichet_receptions(
             issuer_scope,request_id,operation,canonical_request_bytes,
             response_message_id,claim_generation,claim_token,outcome,reply_bytes,processed_at
         ) VALUES('maicie-v20-shape',?1,'future_operation',X'05',?2,1,?3,'accepted',X'06',1)",
        params![
            request_id,
            format!("response-unknown-{suffix}"),
            format!("claim-unknown-{suffix}"),
        ],
    );
    let rejected_on_read = matches!(
        load_guichet_reception(tx, "maicie-v20-shape", &request_id),
        Err(StoreError::Corrupt("opération guichet inconnue"))
    );
    tx.execute_batch(
        "ROLLBACK TO maicie_v20_reception_shape;
         RELEASE maicie_v20_reception_shape;",
    )
    .map_err(StoreError::Sql)?;
    let after: i64 = tx
        .query_row("SELECT COUNT(*) FROM guichet_receptions", [], |row| {
            row.get(0)
        })
        .map_err(StoreError::Sql)?;
    if inserted.is_err() || !rejected_on_read || before != after {
        return Err(StoreError::Corrupt(
            "forme v20 des reçus du guichet incomplète",
        ));
    }
    Ok(())
}

/// Vrai si la base contient déjà un objet de schéma utilisateur.
/// Les tables/index internes `sqlite_*` ne comptent pas : un fichier SQLite
/// fraîchement créé reste « vide » au sens bootstrap.
fn database_has_user_schema(tx: &Transaction<'_>) -> Result<bool, StoreError> {
    let count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master\n\
             WHERE name NOT LIKE 'sqlite_%'",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    Ok(count > 0)
}

/// Migration v16 : solde les délégations ouvertes sur objectifs déjà clos.
/// Même règle que `settle_open_delegations_on_objective_closure` — jamais
/// `terminee` : l'état est `soldee_par_cloture`. Tout-ou-rien : une seule
/// ligne divergente (payload ≠ index) échoue toute la migration.
/// Les outboxes encore expédiables de chaque orpheline sont terminalisées
/// dans la même transaction.
fn migrate_orphan_delegations_on_closed_objectives(tx: &Transaction<'_>) -> Result<(), StoreError> {
    let mut statement = tx
        .prepare(
            "SELECT d.id, d.objective_id, d.state, d.payload_json\n\
             FROM delegations d\n\
             JOIN objectives o ON o.id = d.objective_id\n\
             WHERE o.state = 'clos'\n\
               AND d.state IN ('creee', 'a_evaluer', 'en_attente_prerequis')\n\
             ORDER BY d.id",
        )
        .map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(StoreError::Sql)?;
    let orphans: Vec<_> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);
    for (id, objective_id, state, payload) in orphans {
        let objective_uuid = Uuid::parse_str(&objective_id)
            .map_err(|_| StoreError::Corrupt("objective_id de délégation invalide"))?;
        let mut delegation: Delegation =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if delegation.id.to_string() != id
            || delegation.objectif_id != objective_uuid
            || delegation.etat != parse_delegation_state(&state)?
        {
            return Err(StoreError::Corrupt("délégation et index SQLite divergents"));
        }
        let previous = delegation.etat;
        delegation
            .solder_par_cloture()
            .map_err(StoreError::Domain)?;
        let next_json = serde_json::to_vec(&delegation).map_err(StoreError::Json)?;
        let changed = tx
            .execute(
                "UPDATE delegations SET state = ?1, payload_json = ?2\n\
                 WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
                params![
                    delegation_state_name(delegation.etat),
                    next_json,
                    id,
                    delegation_state_name(previous),
                    payload,
                ],
            )
            .map_err(StoreError::Sql)?;
        if changed != 1 {
            return Err(StoreError::Conflict(
                "délégation orpheline non soldée à la migration",
            ));
        }
        terminalize_dispatchable_outboxes_for_settled_delegation(tx, &id)?;
    }
    Ok(())
}

fn migrate_outbox_to_rejected_state(tx: &Transaction<'_>) -> Result<(), StoreError> {
    tx.execute_batch(
        "DROP INDEX IF EXISTS delegation_outbox_pending_idx;
         ALTER TABLE delegation_outbox RENAME TO delegation_outbox_v1;
         CREATE TABLE delegation_outbox (
             message_id TEXT PRIMARY KEY,
             delegation_id TEXT NOT NULL UNIQUE REFERENCES delegations(id),
             objective_id TEXT NOT NULL REFERENCES objectives(id),
             issuer_scope TEXT NOT NULL,
             issued_at INTEGER NOT NULL,
             target TEXT NOT NULL,
             body_bytes BLOB NOT NULL,
             reply INTEGER NOT NULL CHECK(reply IN (0, 1)),
             timeout_secs INTEGER NOT NULL,
             deadline_contractuelle INTEGER NOT NULL,
             body_hash BLOB NOT NULL,
             message_bytes BLOB NOT NULL,
             state TEXT NOT NULL CHECK(state IN ('prepared','outcome_unknown','accepted','rejected')),
             attempted_at INTEGER,
             retry_until INTEGER NOT NULL,
             dedup_retained_until INTEGER NOT NULL,
             last_issue_json BLOB,
             issue_observed_at INTEGER,
             terminal INTEGER NOT NULL DEFAULT 0 CHECK(terminal IN (0, 1))
         );
         INSERT INTO delegation_outbox(
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
             message_bytes, state, attempted_at, retry_until, dedup_retained_until,
             last_issue_json, issue_observed_at, terminal
         )
         SELECT message_id, delegation_id, objective_id, issuer_scope, issued_at, target,
                body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,
                message_bytes,
                CASE
                    WHEN terminal = 1 AND state = 'outcome_unknown' THEN 'rejected'
                    ELSE state
                END,
                attempted_at, retry_until, dedup_retained_until,
                last_issue_json, issue_observed_at, terminal
         FROM delegation_outbox_v1;
         DROP TABLE delegation_outbox_v1;
         CREATE INDEX delegation_outbox_pending_idx
             ON delegation_outbox(terminal, state, retry_until, message_id);",
    )
    .map_err(StoreError::Sql)
}

fn load_or_create_issuer_scope(connection: &mut Connection) -> Result<String, StoreError> {
    let tx = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(StoreError::Sql)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let candidate = existing.unwrap_or_else(|| format!("maicie-{}", Uuid::new_v4().simple()));
    tx.execute(
        "INSERT OR IGNORE INTO maicie_identity(singleton, issuer_scope) VALUES (1, ?1)",
        [&candidate],
    )
    .map_err(StoreError::Sql)?;
    let scope: String = tx
        .query_row(
            "SELECT issuer_scope FROM maicie_identity WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    tx.commit().map_err(StoreError::Sql)?;
    Ok(scope)
}

fn insert_prepared(
    tx: &Transaction<'_>,
    prepared: &PreparedDelegation,
    opening_permit: &ObjectiveOpeningPermit,
) -> Result<(), StoreError> {
    prepared.delegation.verifier().map_err(StoreError::Domain)?;
    open_objective(tx, &prepared.objective, opening_permit)?;
    let delegation_json = serde_json::to_vec(&prepared.delegation).map_err(StoreError::Json)?;
    tx.execute(
        "INSERT INTO delegations(id, objective_id, state, payload_json) VALUES (?1, ?2, ?3, ?4)",
        params![
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            delegation_state_name(prepared.delegation.etat),
            delegation_json
        ],
    )
    .map_err(StoreError::Sql)?;
    tx.execute(
        "INSERT INTO delegation_outbox(\n\
             message_id, delegation_id, objective_id, issuer_scope, issued_at, target,\n\
             body_bytes, reply, timeout_secs, deadline_contractuelle, body_hash,\n\
             message_bytes, state, attempted_at, retry_until, dedup_retained_until, terminal\n\
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'prepared',NULL,?13,?14,0)",
        params![
            prepared.outbox.message_id.to_string(),
            prepared.delegation.id.to_string(),
            prepared.objective.id.to_string(),
            prepared.issuer_scope,
            prepared.issued_at,
            prepared.outbox.target,
            prepared.outbox.body_bytes,
            i64::from(prepared.outbox.reply),
            i64::try_from(prepared.outbox.timeout_secs)
                .map_err(|_| StoreError::Invalid("timeout_secs hors borne SQLite"))?,
            prepared.outbox.deadline_contractuelle,
            prepared.outbox.body_hash,
            prepared.message_bytes,
            prepared.outbox.retry_until,
            prepared.outbox.dedup_retained_until,
        ],
    )
    .map_err(StoreError::Sql)?;
    Ok(())
}

type RawDelegateResult = (
    Vec<u8>,
    String,
    String,
    Option<String>,
    String,
    i64,
    Vec<u8>,
    i64,
);

fn decode_delegate_result(
    stored: RawDelegateResult,
    expected_canonical_request_bytes: &[u8],
) -> Result<StoredDelegateResult, StoreError> {
    let (
        canonical_request_bytes,
        objective_id,
        delegation_id,
        message_id,
        participant,
        timeout_secs,
        delegation_payload,
        deadline_contractuelle,
    ) = stored;
    if canonical_request_bytes != expected_canonical_request_bytes {
        return Err(StoreError::EnvelopeMismatch);
    }
    if participant.is_empty() || timeout_secs <= 0 {
        return Err(StoreError::Corrupt(
            "résultat de délégation idempotente invalide",
        ));
    }
    let delegation: Delegation =
        serde_json::from_slice(&delegation_payload).map_err(StoreError::Json)?;
    if delegation.id.to_string() != delegation_id || delegation.participant != participant {
        return Err(StoreError::Corrupt(
            "résultat idempotent et délégation divergents",
        ));
    }
    let message_id = match message_id {
        Some(raw) => Some(parse_uuid(&raw)?),
        None => None,
    };
    Ok(StoredDelegateResult {
        objective_id: parse_uuid(&objective_id)?,
        delegation_id: parse_uuid(&delegation_id)?,
        message_id,
        participant,
        duration: delegation.duree,
        timeout_secs: u64::try_from(timeout_secs)
            .map_err(|_| StoreError::Corrupt("timeout idempotent invalide"))?,
        deadline_contractuelle,
    })
}

fn validate_delegate_idempotency_key(key: &str) -> Result<(), StoreError> {
    if key.is_empty() || key.len() > 128 || key.chars().any(char::is_control) {
        return Err(StoreError::Invalid("clé d'idempotence delegate invalide"));
    }
    Ok(())
}

/// Seule frontière capable d'insérer une ligne d'objectif absente.
///
/// Le permit n'est pas une décoration de l'appelant : son origine doit être
/// exactement celle du payload qui sera écrit. La voie de transition d'un
/// objectif existant est séparée et ne contient aucun `INSERT`.
fn open_objective(
    tx: &Transaction<'_>,
    objective: &ObjectifCoordonne,
    opening_permit: &ObjectiveOpeningPermit,
) -> Result<(), StoreError> {
    if objective.etat == EtatObjectif::Clos {
        return Err(StoreError::Invalid("clôture réservée à close_objective"));
    }
    if &objective.origin != opening_permit.origin() {
        return Err(StoreError::Invalid(
            "origine objectif divergente du permit d'ouverture",
        ));
    }
    let id = objective.id.to_string();
    let incoming_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
    let current: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;

    let Some((current_state, current_json)) = current else {
        tx.execute(
            "INSERT INTO objectives(id, state, payload_json) VALUES (?1, ?2, ?3)",
            params![id, objective_state_name(objective.etat), incoming_json],
        )
        .map_err(StoreError::Sql)?;
        return Ok(());
    };

    transition_existing_objective(
        tx,
        objective,
        &id,
        incoming_json,
        current_state,
        current_json,
    )
}

/// Met à jour uniquement une ligne déjà présente. L'absence est un invariant
/// rompu, jamais une invitation implicite à créer sans permit.
fn update_objective(tx: &Transaction<'_>, objective: &ObjectifCoordonne) -> Result<(), StoreError> {
    if objective.etat == EtatObjectif::Clos {
        return Err(StoreError::Invalid("clôture réservée à close_objective"));
    }
    let id = objective.id.to_string();
    let incoming_json = serde_json::to_vec(objective).map_err(StoreError::Json)?;
    let (current_state, current_json): (String, Vec<u8>) = tx
        .query_row(
            "SELECT state, payload_json FROM objectives WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?
        .ok_or(StoreError::NotFound("objectif à mettre à jour absent"))?;

    transition_existing_objective(
        tx,
        objective,
        &id,
        incoming_json,
        current_state,
        current_json,
    )
}

fn transition_existing_objective(
    tx: &Transaction<'_>,
    objective: &ObjectifCoordonne,
    id: &str,
    incoming_json: Vec<u8>,
    current_state: String,
    current_json: Vec<u8>,
) -> Result<(), StoreError> {
    let mut current_objective: ObjectifCoordonne =
        serde_json::from_slice(&current_json).map_err(StoreError::Json)?;
    if parse_objective_state(&current_state)? != current_objective.etat {
        return Err(StoreError::Corrupt(
            "état objectif divergent de son payload",
        ));
    }
    if current_objective == *objective {
        return Ok(());
    }
    current_objective
        .transition(objective.etat, objective.mis_a_jour_at)
        .map_err(|_| StoreError::Conflict("transition objectif interdite"))?;
    if current_objective != *objective {
        return Err(StoreError::Conflict(
            "payload objectif incohérent avec la transition",
        ));
    }
    let changed = tx
        .execute(
            "UPDATE objectives SET state = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
            params![
                objective_state_name(objective.etat),
                incoming_json,
                id,
                current_state,
                current_json
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::Conflict(
            "objectif modifié concurremment pendant la transition",
        ))
    }
}

/// Applique l'effet de coordination d'une issue terminale dans la transaction
/// qui la fige dans l'outbox. Une livraison `accepted` ne vaut pas réponse et
/// n'entraîne donc aucun jugement métier.
fn coordinate_terminal_issue(
    tx: &Transaction<'_>,
    message_id: Uuid,
    issue: &IdempotencyIssue,
    observed_at: i64,
) -> Result<(), StoreError> {
    let Some(outcome) = terminal_issue_outcome(issue) else {
        return Ok(());
    };
    coordinate_delegation_outcome(tx, message_id, observed_at, outcome)
}

/// Un rejet local terminal doit être visible au même titre qu'un refus Bridget
/// afin que la coordination ne reste pas silencieusement en cours.
fn coordinate_local_failure(
    tx: &Transaction<'_>,
    message_id: Uuid,
    reason: LocalFailureReason,
    observed_at: i64,
) -> Result<(), StoreError> {
    let outcome = match reason {
        LocalFailureReason::FrameTooLarge => TerminalDelegationOutcome::LocalFrameTooLarge,
        LocalFailureReason::InvalidEnvelope => TerminalDelegationOutcome::LocalInvalidEnvelope,
    };
    coordinate_delegation_outcome(tx, message_id, observed_at, outcome)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalDelegationOutcome {
    Rejected,
    Cancelled,
    LocalFrameTooLarge,
    LocalInvalidEnvelope,
}

impl TerminalDelegationOutcome {
    fn motif(self) -> &'static str {
        match self {
            Self::Rejected => "issue terminale Bridget : rejet de livraison",
            Self::Cancelled => "issue terminale Bridget : annulation de livraison",
            Self::LocalFrameTooLarge => "échec local : trame de livraison trop grande",
            Self::LocalInvalidEnvelope => "échec local : enveloppe de livraison invalide",
        }
    }

    fn cancels_delegation(self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

fn terminal_issue_outcome(issue: &IdempotencyIssue) -> Option<TerminalDelegationOutcome> {
    match issue {
        IdempotencyIssue::Rejected { category, .. } if category == "cancelled" => {
            Some(TerminalDelegationOutcome::Cancelled)
        }
        IdempotencyIssue::Rejected { .. }
        | IdempotencyIssue::EnvelopeMismatch
        | IdempotencyIssue::IdempotencyExpired
        | IdempotencyIssue::InvalidIssuedAt => Some(TerminalDelegationOutcome::Rejected),
        IdempotencyIssue::Accepted { .. } | IdempotencyIssue::OutcomeUnknown { .. } => None,
    }
}

/// Fige l'audit et les deux agrégats de coordination en une unique
/// transaction. L'identifiant de décision est le `message_id` durable : le
/// traitement reste idempotent après un crash sans créer une seconde décision.
fn coordinate_delegation_outcome(
    tx: &Transaction<'_>,
    message_id: Uuid,
    observed_at: i64,
    outcome: TerminalDelegationOutcome,
) -> Result<(), StoreError> {
    let row: Option<StoredDelegationAggregates> = tx
        .query_row(
            "SELECT o.objective_id, o.delegation_id, obj.state, obj.payload_json,\n\
                    d.state, d.payload_json\n\
             FROM delegation_outbox o\n\
             JOIN objectives obj ON obj.id = o.objective_id\n\
             JOIN delegations d ON d.id = o.delegation_id\n\
             WHERE o.message_id = ?1",
            [message_id.to_string()],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((
        objective_id,
        delegation_id,
        objective_state,
        objective_json,
        delegation_state,
        delegation_json,
    )) = row
    else {
        return Err(StoreError::Corrupt(
            "outbox terminale sans agrégats corrélés",
        ));
    };

    let mut objective: ObjectifCoordonne =
        serde_json::from_slice(&objective_json).map_err(StoreError::Json)?;
    let mut delegation: Delegation =
        serde_json::from_slice(&delegation_json).map_err(StoreError::Json)?;
    if objective.id.to_string() != objective_id
        || objective.etat != parse_objective_state(&objective_state)?
        || delegation.id.to_string() != delegation_id
        || delegation.objectif_id != objective.id
        || delegation.etat != parse_delegation_state(&delegation_state)?
    {
        return Err(StoreError::Corrupt("agrégats de délégation divergents"));
    }

    if objective.etat == EtatObjectif::EnCoordination {
        if observed_at <= 0 {
            return Err(StoreError::Invalid("observed_at invalide"));
        }
        objective
            .transition(EtatObjectif::AEvaluer, observed_at)
            .map_err(StoreError::Domain)?;
    }
    match (delegation.etat, outcome.cancels_delegation()) {
        (EtatDelegation::Creee, true) | (EtatDelegation::AEvaluer, true) => {
            delegation.annuler().map_err(StoreError::Domain)?;
        }
        (EtatDelegation::Creee, false) => delegation
            .transition(EtatDelegation::AEvaluer)
            .map_err(StoreError::Domain)?,
        _ => {}
    }

    let decision = DecisionCoordination {
        id: message_id,
        objectif_id: objective.id,
        kind: TypeDecision::ConstaterIssue,
        proposee_par: "maicie".to_string(),
        etat: EtatDecision::Appliquee,
        motif: outcome.motif().to_string(),
    };
    let decision_json = serde_json::to_vec(&decision).map_err(StoreError::Json)?;
    let existing: Option<Vec<u8>> = tx
        .query_row(
            "SELECT payload_json FROM coordination_decisions WHERE id = ?1",
            [message_id.to_string()],
            |row| row.get(0),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    if let Some(existing) = existing {
        if existing == decision_json {
            return Ok(());
        }
        return Err(StoreError::Conflict(
            "décision d'issue terminale divergente",
        ));
    }

    update_objective(tx, &objective)?;
    upsert_delegation(tx, &delegation)?;
    let inserted = tx
        .execute(
            "INSERT INTO coordination_decisions(id, objective_id, state, payload_json)\n\
             VALUES (?1, ?2, 'applied', ?3)",
            params![
                message_id.to_string(),
                objective.id.to_string(),
                decision_json
            ],
        )
        .map_err(StoreError::Sql)?;
    if inserted != 1 {
        return Err(StoreError::Conflict(
            "décision d'issue terminale non enregistrée",
        ));
    }
    Ok(())
}

fn upsert_delegation(tx: &Transaction<'_>, delegation: &Delegation) -> Result<(), StoreError> {
    let id = delegation.id.to_string();
    let incoming_json = serde_json::to_vec(delegation).map_err(StoreError::Json)?;
    let current: Option<(String, Vec<u8>)> = tx
        .query_row(
            "SELECT state, payload_json FROM delegations WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(StoreError::Sql)?;
    let Some((current_state, current_json)) = current else {
        return Err(StoreError::NotFound("délégation absente"));
    };
    let mut current_delegation: Delegation =
        serde_json::from_slice(&current_json).map_err(StoreError::Json)?;
    if current_delegation.etat != parse_delegation_state(&current_state)? {
        return Err(StoreError::Corrupt(
            "état délégation divergent de son payload",
        ));
    }
    if current_delegation == *delegation {
        return Ok(());
    }
    if delegation.etat == EtatDelegation::Annulee {
        current_delegation.annuler().map_err(StoreError::Domain)?;
    } else {
        current_delegation
            .transition(delegation.etat)
            .map_err(StoreError::Domain)?;
    }
    if current_delegation != *delegation {
        return Err(StoreError::Conflict(
            "payload délégation incohérent avec la transition",
        ));
    }
    let changed = tx
        .execute(
            "UPDATE delegations SET state = ?1, payload_json = ?2\n\
             WHERE id = ?3 AND state = ?4 AND payload_json = ?5",
            params![
                delegation_state_name(delegation.etat),
                incoming_json,
                id,
                current_state,
                current_json
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::Conflict(
            "délégation modifiée concurremment pendant la transition",
        ))
    }
}

fn objective_state_name(state: EtatObjectif) -> &'static str {
    match state {
        EtatObjectif::Ouvert => "ouvert",
        EtatObjectif::EnCoordination => "en_coordination",
        EtatObjectif::AEvaluer => "a_evaluer",
        EtatObjectif::Synthetise => "synthetise",
        EtatObjectif::Clos => "clos",
    }
}

fn routine_state_name(state: EtatRoutine) -> &'static str {
    match state {
        EtatRoutine::Proposed => "proposed",
        EtatRoutine::Active => "active",
        EtatRoutine::Paused => "paused",
    }
}

fn parse_routine_state(value: &str) -> Result<EtatRoutine, StoreError> {
    match value {
        "proposed" => Ok(EtatRoutine::Proposed),
        "active" => Ok(EtatRoutine::Active),
        "paused" => Ok(EtatRoutine::Paused),
        _ => Err(StoreError::Corrupt("état routine inconnu")),
    }
}

fn occurrence_state_name(state: EtatOccurrence) -> &'static str {
    match state {
        EtatOccurrence::Ouverte => "ouverte",
        EtatOccurrence::Sautee => "sautee",
        EtatOccurrence::Differee => "differee",
        EtatOccurrence::Terminee => "terminee",
    }
}

fn parse_occurrence_state(value: &str) -> Result<EtatOccurrence, StoreError> {
    match value {
        "ouverte" => Ok(EtatOccurrence::Ouverte),
        "sautee" => Ok(EtatOccurrence::Sautee),
        "differee" => Ok(EtatOccurrence::Differee),
        "terminee" => Ok(EtatOccurrence::Terminee),
        _ => Err(StoreError::Corrupt("état occurrence inconnu")),
    }
}

fn suite_columns(suite: &SuiteObjective) -> (&'static str, Option<String>) {
    match suite {
        SuiteObjective::Aucune => ("aucune", None),
        SuiteObjective::Objectif(id) => ("objectif", Some(id.to_string())),
    }
}

fn parse_suite_columns(
    kind: &str,
    objective_id: Option<String>,
) -> Result<SuiteObjective, StoreError> {
    match (kind, objective_id) {
        ("aucune", None) => Ok(SuiteObjective::Aucune),
        ("objectif", Some(raw)) => Ok(SuiteObjective::Objectif(parse_uuid(&raw)?)),
        _ => Err(StoreError::Corrupt("suite routine incohérente")),
    }
}

type RoutineRow = (
    String,
    String,
    String,
    i64,
    String,
    Option<String>,
    String,
    String,
    Vec<u8>,
    String,
    i64,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

fn map_routine_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoutineRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
        row.get(7)?,
        row.get(8)?,
        row.get(9)?,
        row.get(10)?,
        row.get(11)?,
        row.get(12)?,
        row.get(13)?,
    ))
}

fn row_to_routine(row: RoutineRow) -> Result<Routine, StoreError> {
    Ok(Routine {
        id: parse_uuid(&row.0)?,
        goal: row.1,
        participant: row.2,
        period_secs: row.3,
        suite: parse_suite_columns(&row.4, row.5)?,
        depends_on: serde_json::from_str(&row.6).map_err(StoreError::Json)?,
        references: serde_json::from_str(&row.7).map_err(StoreError::Json)?,
        template_hash: row.8,
        state: parse_routine_state(&row.9)?,
        proposed_at: row.10,
        approved_at: row.11,
        paused_at: row.12,
        last_bucket: row.13,
    })
}

type OccurrenceRow = (
    String,
    i64,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

fn map_occurrence_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<OccurrenceRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn row_to_occurrence(row: OccurrenceRow) -> Result<RoutineOccurrence, StoreError> {
    Ok(RoutineOccurrence {
        routine_id: parse_uuid(&row.0)?,
        bucket: row.1,
        state: parse_occurrence_state(&row.2)?,
        reason: row.3,
        objective_id: row.4.as_deref().map(parse_uuid).transpose()?,
        delegation_id: row.5.as_deref().map(parse_uuid).transpose()?,
        created_at: row.6,
    })
}

fn parse_objective_state(value: &str) -> Result<EtatObjectif, StoreError> {
    match value {
        "ouvert" => Ok(EtatObjectif::Ouvert),
        "en_coordination" => Ok(EtatObjectif::EnCoordination),
        "a_evaluer" => Ok(EtatObjectif::AEvaluer),
        "synthetise" => Ok(EtatObjectif::Synthetise),
        "clos" => Ok(EtatObjectif::Clos),
        _ => Err(StoreError::Corrupt("état objectif inconnu")),
    }
}

fn delegation_state_name(state: EtatDelegation) -> &'static str {
    state.as_sql()
}

fn parse_delegation_state(value: &str) -> Result<EtatDelegation, StoreError> {
    EtatDelegation::ALL
        .iter()
        .copied()
        .find(|etat| etat.as_sql() == value)
        .ok_or(StoreError::Corrupt("état délégation inconnu"))
}

fn project_profile_status_text(status: ProjectProfileStatus) -> &'static str {
    match status {
        ProjectProfileStatus::Proposed => "proposed",
        ProjectProfileStatus::Resolved => "resolved",
        ProjectProfileStatus::Approved => "approved",
        ProjectProfileStatus::Active => "active",
        ProjectProfileStatus::Stale => "stale",
        ProjectProfileStatus::Disabled => "disabled",
    }
}

fn parse_outbox_state(value: &str) -> Result<EtatOutboxDelegation, StoreError> {
    match value {
        "prepared" => Ok(EtatOutboxDelegation::Prepared),
        "outcome_unknown" => Ok(EtatOutboxDelegation::OutcomeUnknown),
        "accepted" => Ok(EtatOutboxDelegation::Accepted),
        "rejected" => Ok(EtatOutboxDelegation::Rejected),
        _ => Err(StoreError::Corrupt("état outbox inconnu")),
    }
}

fn validate_activation_proposal(
    decision: &DecisionCoordination,
    approval: &ApprobationActivation,
) -> Result<(), StoreError> {
    decision.verifier().map_err(StoreError::Domain)?;
    if decision.kind != TypeDecision::ReveillerProfil
        || decision.etat != EtatDecision::Proposee
        || decision.objectif_id != approval.objective_id
        || approval.profile_id.trim().is_empty()
        || approval.context_scope.trim().is_empty()
        || approval.parameters.trim().is_empty()
        || approval.expires_at <= 0
        || approval.consumed_at.is_some()
        || approval.profile_hash.is_empty()
        || approval.context_hash.is_empty()
        || approval.command_id.is_nil()
    {
        return Err(StoreError::Invalid("proposition d'activation invalide"));
    }
    if approval.actor != "local_human" {
        return Err(StoreError::Domain(DomainError::DonneeInvalide(
            "acteur d'approbation invalide",
        )));
    }
    Ok(())
}

fn validate_approved_spawn_order(
    approval: &ApprobationActivation,
    bytes: &[u8],
) -> Result<(), StoreError> {
    let order: ApprovedSpawnOrder = serde_json::from_slice(bytes).map_err(StoreError::Json)?;
    if order.kind != "SpawnOrder"
        || order.command_id != approval.command_id.to_string()
        || order.agent_type.trim().is_empty()
        || order.issued_at <= 0
        || order.deadline_at <= order.issued_at
        || order.cwd.is_empty()
        || approval.parameters.as_bytes() != bytes
    {
        return Err(StoreError::Invalid(
            "SpawnOrder non conforme à l'approbation",
        ));
    }
    let _ = (order.agent_id, order.persistent);
    Ok(())
}

fn activation_issue(
    command_id: Uuid,
    outcome: &SpawnOutcome,
) -> Result<(EtatActivationOutbox, Vec<u8>), StoreError> {
    let expected = command_id.to_string();
    let value = match outcome {
        SpawnOutcome::Accepted {
            command_id,
            agent_id,
        } => {
            if command_id != &expected {
                return Err(StoreError::Invalid("command_id d'issue divergent"));
            }
            json!({"kind":"accepted","command_id":command_id,"agent_id":agent_id})
        }
        SpawnOutcome::Rejected { command_id, reason } => {
            if command_id != &expected {
                return Err(StoreError::Invalid("command_id d'issue divergent"));
            }
            json!({"kind":"rejected","command_id":command_id,"reason":reason})
        }
        SpawnOutcome::Idempotency(IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id,
        }) => {
            return serde_json::to_vec(&json!({"kind":"outcome_unknown","expires_at":expires_at,"delivery_id":delivery_id}))
                .map(|bytes| (EtatActivationOutbox::OutcomeUnknown, bytes))
                .map_err(StoreError::Json);
        }
        SpawnOutcome::Idempotency(issue) => {
            json!({"kind":"idempotency","issue":encode_issue(issue)})
        }
    };
    serde_json::to_vec(&value)
        .map(|bytes| (EtatActivationOutbox::Applied, bytes))
        .map_err(StoreError::Json)
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decision_state_name(state: EtatDecision) -> &'static str {
    match state {
        EtatDecision::Proposee => "proposed",
        EtatDecision::Approuvee => "approved",
        EtatDecision::Refusee => "rejected",
        EtatDecision::Appliquee => "applied",
    }
}

fn parse_decision_state(value: &str) -> Result<EtatDecision, StoreError> {
    match value {
        "proposed" => Ok(EtatDecision::Proposee),
        "approved" => Ok(EtatDecision::Approuvee),
        "rejected" => Ok(EtatDecision::Refusee),
        "applied" => Ok(EtatDecision::Appliquee),
        _ => Err(StoreError::Corrupt("état décision inconnu")),
    }
}

fn parse_activation_state(value: &str) -> Result<EtatActivationOutbox, StoreError> {
    match value {
        "dispatching" => Ok(EtatActivationOutbox::Dispatching),
        "outcome_unknown" => Ok(EtatActivationOutbox::OutcomeUnknown),
        "applied" => Ok(EtatActivationOutbox::Applied),
        _ => Err(StoreError::Corrupt("état activation inconnu")),
    }
}

fn outbox_state_name(state: EtatOutboxDelegation) -> &'static str {
    match state {
        EtatOutboxDelegation::Prepared => "prepared",
        EtatOutboxDelegation::OutcomeUnknown => "outcome_unknown",
        EtatOutboxDelegation::Accepted => "accepted",
        EtatOutboxDelegation::Rejected => "rejected",
    }
}

fn encode_issue(issue: &IdempotencyIssue) -> Value {
    match issue {
        IdempotencyIssue::Accepted { expires_at } => {
            json!({"kind":"accepted","expires_at":expires_at})
        }
        IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        } => json!({
            "kind":"rejected","category":category,"reason":reason,"expires_at":expires_at
        }),
        IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id,
        } => json!({
            "kind":"outcome_unknown","expires_at":expires_at,"delivery_id":delivery_id
        }),
        IdempotencyIssue::EnvelopeMismatch => json!({"kind":"envelope_mismatch"}),
        IdempotencyIssue::IdempotencyExpired => json!({"kind":"idempotency_expired"}),
        IdempotencyIssue::InvalidIssuedAt => json!({"kind":"invalid_issued_at"}),
    }
}

struct RawPending {
    objective_id: String,
    delegation_id: String,
    message_id: String,
    issuer_scope: String,
    issued_at: i64,
    target: String,
    body_bytes: Vec<u8>,
    reply: i64,
    timeout_secs: i64,
    deadline_contractuelle: i64,
    body_hash: Vec<u8>,
    message_bytes: Vec<u8>,
    state: String,
    attempted_at: Option<i64>,
    retry_until: i64,
    dedup_retained_until: i64,
}

fn raw_pending_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RawPending> {
    Ok(RawPending {
        objective_id: row.get(0)?,
        delegation_id: row.get(1)?,
        message_id: row.get(2)?,
        issuer_scope: row.get(3)?,
        issued_at: row.get(4)?,
        target: row.get(5)?,
        body_bytes: row.get(6)?,
        reply: row.get(7)?,
        timeout_secs: row.get(8)?,
        deadline_contractuelle: row.get(9)?,
        body_hash: row.get(10)?,
        message_bytes: row.get(11)?,
        state: row.get(12)?,
        attempted_at: row.get(13)?,
        retry_until: row.get(14)?,
        dedup_retained_until: row.get(15)?,
    })
}

impl TryFrom<RawPending> for PendingDelegationOutbox {
    type Error = StoreError;

    fn try_from(raw: RawPending) -> Result<Self, Self::Error> {
        let pending = Self {
            objective_id: parse_uuid(&raw.objective_id)?,
            delegation_id: parse_uuid(&raw.delegation_id)?,
            message_id: parse_uuid(&raw.message_id)?,
            issuer_scope: raw.issuer_scope,
            issued_at: raw.issued_at,
            target: raw.target,
            body_bytes: raw.body_bytes,
            reply: match raw.reply {
                0 => false,
                1 => true,
                _ => return Err(StoreError::Corrupt("booléen reply invalide")),
            },
            timeout_secs: u64::try_from(raw.timeout_secs)
                .map_err(|_| StoreError::Corrupt("timeout_secs invalide"))?,
            deadline_contractuelle: raw.deadline_contractuelle,
            body_hash: raw.body_hash,
            message_bytes: raw.message_bytes,
            state: parse_outbox_state(&raw.state)?,
            attempted_at: raw.attempted_at,
            retry_until: raw.retry_until,
            dedup_retained_until: raw.dedup_retained_until,
        };
        pending.validate().map_err(StoreError::Outbox)?;
        Ok(pending)
    }
}

struct RawRecovery {
    pending: RawPending,
    objective_state: String,
    delegation_state: String,
    last_issue_json: Option<Vec<u8>>,
    issue_observed_at: Option<i64>,
}

impl TryFrom<RawRecovery> for RecoverySnapshot {
    type Error = StoreError;

    fn try_from(raw: RawRecovery) -> Result<Self, Self::Error> {
        Ok(Self {
            outbox: raw.pending.try_into()?,
            objective_state: parse_objective_state(&raw.objective_state)?,
            delegation_state: parse_delegation_state(&raw.delegation_state)?,
            last_issue: raw
                .last_issue_json
                .map(|bytes| serde_json::from_slice(&bytes).map_err(StoreError::Json))
                .transpose()?,
            issue_observed_at: raw.issue_observed_at,
        })
    }
}

#[derive(Debug)]
struct StoredProjectIdentity {
    project_id: String,
    display_name: String,
    status: String,
    created_at: i64,
    updated_at: i64,
    registration_command_id: String,
}

fn project_identity_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredProjectIdentity> {
    Ok(StoredProjectIdentity {
        project_id: row.get(0)?,
        display_name: row.get(1)?,
        status: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
        registration_command_id: row.get(5)?,
    })
}

fn decode_project_identity(stored: StoredProjectIdentity) -> Result<ProjectIdentity, StoreError> {
    let status = match stored.status.as_str() {
        "pending_binding" => ProjectIdentityStatus::PendingBinding,
        "active" => ProjectIdentityStatus::Active,
        "registration_conflict" => ProjectIdentityStatus::RegistrationConflict,
        "disabled" => ProjectIdentityStatus::Disabled,
        _ => return Err(StoreError::Corrupt("état identité projet inconnu")),
    };
    if stored.project_id.trim().is_empty()
        || stored.display_name.trim().is_empty()
        || stored.registration_command_id.trim().is_empty()
        || stored.created_at < 0
        || stored.updated_at < stored.created_at
    {
        return Err(StoreError::Corrupt("identité projet durable invalide"));
    }
    Ok(ProjectIdentity {
        project_id: stored.project_id,
        display_name: stored.display_name,
        status,
        created_at: stored.created_at,
        updated_at: stored.updated_at,
        registration_command_id: stored.registration_command_id,
    })
}

#[derive(Debug)]
struct StoredProjectRegistration {
    command_id: String,
    canonical_payload: Vec<u8>,
    proposed_project_id: String,
    resolved_project_id: Option<String>,
    requested_root: String,
    state: String,
    retry_until: i64,
    outcome_json: Option<Vec<u8>>,
    outcome_observed_at: Option<i64>,
}

fn project_registration_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StoredProjectRegistration> {
    Ok(StoredProjectRegistration {
        command_id: row.get(0)?,
        canonical_payload: row.get(1)?,
        proposed_project_id: row.get(2)?,
        resolved_project_id: row.get(3)?,
        requested_root: row.get(4)?,
        state: row.get(5)?,
        retry_until: row.get(6)?,
        outcome_json: row.get(7)?,
        outcome_observed_at: row.get(8)?,
    })
}

fn project_registration_for_command(
    connection: &Connection,
    command_id: &str,
) -> Result<Option<StoredProjectRegistration>, StoreError> {
    connection
        .query_row(
            "SELECT command_id, canonical_payload, proposed_project_id, resolved_project_id,
                    requested_root, state, retry_until, outcome_json, outcome_observed_at
             FROM project_registration_commands WHERE command_id = ?1",
            [command_id],
            project_registration_from_row,
        )
        .optional()
        .map_err(StoreError::Sql)
}

fn project_registration_record_from_stored(
    connection: &Connection,
    stored: StoredProjectRegistration,
) -> Result<ProjectRegistrationRecord, StoreError> {
    let identity = connection
        .query_row(
            "SELECT project_id, display_name, status, created_at, updated_at,
                    registration_command_id
             FROM project_identities WHERE project_id = ?1",
            [&stored.proposed_project_id],
            project_identity_from_row,
        )
        .map_err(StoreError::Sql)
        .and_then(decode_project_identity)?;
    if identity.registration_command_id != stored.command_id {
        return Err(StoreError::Corrupt(
            "identité et commande projet divergentes",
        ));
    }
    let state = ProjectRegistrationState::from_db(&stored.state)?;
    let outcome: Option<ProjectBindOutcome> = stored
        .outcome_json
        .as_deref()
        .map(|bytes| serde_json::from_slice(bytes).map_err(StoreError::Json))
        .transpose()?;
    if outcome.as_ref().is_some_and(|outcome| {
        outcome.command_id != stored.command_id || outcome.project_id != stored.proposed_project_id
    }) {
        return Err(StoreError::Corrupt("issue projet et commande divergentes"));
    }
    if outcome.is_some() != stored.outcome_observed_at.is_some() {
        return Err(StoreError::Corrupt("horodatage issue projet divergent"));
    }
    let outbox_pending: bool = connection
        .query_row(
            "SELECT EXISTS(
                 SELECT 1 FROM project_registration_outbox
                 WHERE command_id = ?1 AND state IN ('prepared', 'outcome_unknown')
             )",
            [&stored.command_id],
            |row| row.get(0),
        )
        .map_err(StoreError::Sql)?;
    Ok(ProjectRegistrationRecord {
        command_id: stored.command_id,
        identity,
        requested_root: stored.requested_root,
        state,
        resolved_project_id: stored.resolved_project_id,
        retry_until: stored.retry_until,
        outbox_pending,
        outcome,
    })
}

fn persist_project_identity(
    connection: &Connection,
    identity: &ProjectIdentity,
) -> Result<(), StoreError> {
    let status = match identity.status {
        ProjectIdentityStatus::PendingBinding => "pending_binding",
        ProjectIdentityStatus::Active => "active",
        ProjectIdentityStatus::RegistrationConflict => "registration_conflict",
        ProjectIdentityStatus::Disabled => "disabled",
    };
    let changed = connection
        .execute(
            "UPDATE project_identities SET status = ?1, updated_at = ?2
             WHERE project_id = ?3 AND registration_command_id = ?4",
            params![
                status,
                identity.updated_at,
                identity.project_id,
                identity.registration_command_id,
            ],
        )
        .map_err(StoreError::Sql)?;
    if changed != 1 {
        return Err(StoreError::Corrupt(
            "identité projet absente à la promotion",
        ));
    }
    Ok(())
}

fn project_registration_state_name(state: ProjectRegistrationState) -> &'static str {
    match state {
        ProjectRegistrationState::Prepared => "prepared",
        ProjectRegistrationState::Binding => "binding",
        ProjectRegistrationState::Bound => "bound",
        ProjectRegistrationState::Failed => "failed",
        ProjectRegistrationState::Expired => "expired",
    }
}

fn validate_project_bind_outcome(outcome: &ProjectBindOutcome) -> Result<(), StoreError> {
    if outcome.contract_version != 1
        || outcome.command_id.trim().is_empty()
        || outcome.project_id.trim().is_empty()
        || outcome.observed_at < 0
    {
        return Err(StoreError::Invalid(
            "issue d'enregistrement projet invalide",
        ));
    }
    match outcome.status {
        ProjectBindStatus::Active => {
            if outcome.binding_generation.unwrap_or_default() == 0
                || outcome.backend != Some(ProjectBackend::Host)
                || outcome.reason.is_some()
                || outcome.existing_project_id.is_some()
                || outcome.existing_binding_generation.is_some()
            {
                return Err(StoreError::Invalid("issue active projet invalide"));
            }
        }
        ProjectBindStatus::RegistrationConflict => {
            if outcome.binding_generation.is_some()
                || outcome.backend.is_some()
                || outcome.reason.is_none()
                || outcome
                    .existing_project_id
                    .as_deref()
                    .is_none_or(str::is_empty)
                || outcome.existing_binding_generation.unwrap_or_default() == 0
            {
                return Err(StoreError::Invalid("issue collision projet invalide"));
            }
        }
        ProjectBindStatus::BindingFailed => {
            if outcome.binding_generation.is_some()
                || outcome.backend.is_some()
                || outcome.reason.is_none()
                || outcome.existing_project_id.is_some()
                || outcome.existing_binding_generation.is_some()
            {
                return Err(StoreError::Invalid("issue échec projet invalide"));
            }
        }
    }
    Ok(())
}

fn validate_project_registration_intent(
    intent: &ProjectRegistrationIntent,
) -> Result<(), StoreError> {
    if intent.command_id.trim().is_empty()
        || intent.proposed_project_id.trim().is_empty()
        || intent.display_name.trim().is_empty()
        || intent.canonical_payload.is_empty()
        || intent.created_at < 0
        || intent.retry_until < intent.created_at
        || intent.requested_root.trim().is_empty()
        || !Path::new(&intent.requested_root).is_absolute()
    {
        return Err(StoreError::Invalid(
            "intention enregistrement projet invalide",
        ));
    }
    Ok(())
}

fn parse_uuid(value: &str) -> Result<Uuid, StoreError> {
    Uuid::parse_str(value).map_err(|_| StoreError::Corrupt("UUID stocké invalide"))
}

fn collect_json_agent_references(value: &serde_json::Value, references: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                collect_json_agent_references(value, references);
            }
        }
        serde_json::Value::Object(object) => {
            for (key, value) in object {
                if matches!(
                    key.as_str(),
                    "participant" | "participant_id" | "agent_id" | "to" | "recipient"
                ) && let Some(reference) = value.as_str()
                {
                    references.insert(reference.to_string());
                }
                collect_json_agent_references(value, references);
            }
        }
        _ => {}
    }
}

fn rewrite_json_agent_references(
    tx: &Transaction<'_>,
    table: &str,
    key_column: &str,
    payload_column: &str,
    mapping: &std::collections::BTreeMap<String, String>,
) -> Result<(), StoreError> {
    let allowed = matches!(
        (table, key_column, payload_column),
        ("delegations", "id", "payload_json")
            | ("delegation_outbox", "message_id", "message_bytes")
            | ("notification_outbox", "message_id", "message_bytes")
            | ("tracked_request_outbox", "effect_id", "message_bytes")
    );
    if !allowed {
        return Err(StoreError::Invalid("table de migration non autorisée"));
    }
    let select = format!("SELECT {key_column}, {payload_column} FROM {table}");
    let mut statement = tx.prepare(&select).map_err(StoreError::Sql)?;
    let rows = statement
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(StoreError::Sql)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::Sql)?;
    drop(statement);

    for (key, payload) in rows {
        let mut value: serde_json::Value =
            serde_json::from_slice(&payload).map_err(StoreError::Json)?;
        if !replace_json_agent_references(&mut value, mapping) {
            continue;
        }
        let bytes = serde_json::to_vec(&value).map_err(StoreError::Json)?;
        let update = format!("UPDATE {table} SET {payload_column} = ?1 WHERE {key_column} = ?2");
        tx.execute(&update, rusqlite::params![bytes, key])
            .map_err(StoreError::Sql)?;
    }
    Ok(())
}

fn replace_json_agent_references(
    value: &mut serde_json::Value,
    mapping: &std::collections::BTreeMap<String, String>,
) -> bool {
    let mut changed = false;
    match value {
        serde_json::Value::Array(values) => {
            for value in values {
                changed |= replace_json_agent_references(value, mapping);
            }
        }
        serde_json::Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                if matches!(
                    key.as_str(),
                    "participant" | "participant_id" | "agent_id" | "to" | "recipient"
                ) && let Some(legacy) = value.as_str()
                    && let Some(agent_id) = mapping.get(legacy)
                {
                    *value = serde_json::Value::String(agent_id.clone());
                    changed = true;
                }
                changed |= replace_json_agent_references(value, mapping);
            }
        }
        _ => {}
    }
    changed
}

#[derive(Debug)]
pub enum StoreError {
    Invalid(&'static str),
    Conflict(&'static str),
    GuichetRefusal(MotifRefusGreffe),
    /// Plage déjà tenue par un autre objectif (comparaison exacte de noms).
    ResourceHeld {
        resource: String,
        holder_objective_id: Uuid,
    },
    EnvelopeMismatch,
    NotFound(&'static str),
    Corrupt(&'static str),
    UnsupportedSchema {
        found: i64,
        supported: i64,
    },
    /// Schéma antérieur au binaire : migration refusée sans consentement.
    MigrationRequired {
        found: i64,
        supported: i64,
    },
    JournalModeUnavailable {
        actual: String,
    },
    Io(std::io::Error),
    Sql(rusqlite::Error),
    Json(serde_json::Error),
    Outbox(OutboxError),
    Domain(DomainError),
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "store invalide : {reason}"),
            Self::Conflict(reason) => write!(formatter, "conflit store : {reason}"),
            Self::GuichetRefusal(reason) => {
                write!(formatter, "refus déterministe du guichet : {reason:?}")
            }
            Self::ResourceHeld {
                resource,
                holder_objective_id,
            } => write!(
                formatter,
                "plage « {resource} » déjà réservée par l'objectif {holder_objective_id}"
            ),
            Self::EnvelopeMismatch => write!(formatter, "enveloppe idempotente divergente"),
            Self::NotFound(reason) => write!(formatter, "store introuvable : {reason}"),
            Self::Corrupt(reason) => write!(formatter, "store corrompu : {reason}"),
            Self::UnsupportedSchema { found, supported } => write!(
                formatter,
                "schéma SQLite {found} non supporté (maximum {supported})"
            ),
            Self::MigrationRequired { found, supported } => write!(
                formatter,
                "schéma SQLite {found} antérieur au binaire (attend {supported}) ; relancer avec : maicie migrate --config <chemin>"
            ),
            Self::JournalModeUnavailable { actual } => write!(
                formatter,
                "journal SQLite WAL indisponible (mode obtenu : {actual})"
            ),
            Self::Io(source) => write!(formatter, "I/O store impossible : {source}"),
            Self::Sql(source) => write!(formatter, "SQLite impossible : {source}"),
            Self::Json(source) => write!(formatter, "JSON store impossible : {source}"),
            Self::Outbox(source) => write!(formatter, "{source}"),
            Self::Domain(source) => write!(formatter, "domaine invalide : {source:?}"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Sql(source) => Some(source),
            Self::Json(source) => Some(source),
            Self::Outbox(source) => Some(source),
            Self::Domain(_) => None,
            Self::Invalid(_)
            | Self::Conflict(_)
            | Self::GuichetRefusal(_)
            | Self::ResourceHeld { .. }
            | Self::EnvelopeMismatch
            | Self::NotFound(_)
            | Self::Corrupt(_)
            | Self::UnsupportedSchema { .. }
            | Self::MigrationRequired { .. }
            | Self::JournalModeUnavailable { .. } => None,
        }
    }
}

#[cfg(test)]
mod coordination_transaction_tests {
    use super::*;
    use crate::domain::{EtatGenerationDelegation, TypeDecisionCoordinationActive};

    #[test]
    fn upsert_interne_refuse_aussi_un_objectif_deja_clos() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE objectives(
                     id TEXT PRIMARY KEY, state TEXT NOT NULL, payload_json BLOB NOT NULL
                 );",
            )
            .unwrap();
        let mut objective =
            ObjectifCoordonne::nouveau("forge", crate::domain::ModeObjectif::Delegue, 10).unwrap();
        objective.clore(11).unwrap();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        assert!(matches!(
            open_objective(&tx, &objective, &ObjectiveOpeningPermit::auto_generated()),
            Err(StoreError::Invalid("clôture réservée à close_objective"))
        ));
        assert_eq!(
            tx.query_row("SELECT COUNT(*) FROM objectives", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn spec_056_permit_et_origine_divergents_refusent_avant_insertion() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE objectives(
                     id TEXT PRIMARY KEY, state TEXT NOT NULL, payload_json BLOB NOT NULL
                 );",
            )
            .unwrap();
        let mut objective = ObjectifCoordonne::nouveau(
            "origine discordante",
            crate::domain::ModeObjectif::Delegue,
            10,
        )
        .unwrap();
        objective.origin = crate::domain::ObjectiveOrigin::LegacyUnknown;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();

        assert!(matches!(
            open_objective(&tx, &objective, &ObjectiveOpeningPermit::auto_generated()),
            Err(StoreError::Invalid(
                "origine objectif divergente du permit d'ouverture"
            ))
        ));
        assert_eq!(
            tx.query_row("SELECT COUNT(*) FROM objectives", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            0,
            "un refus de provenance ne doit créer aucune ligne"
        );
    }

    #[test]
    fn faute_apres_une_vraie_outbox_annule_decision_transition_et_notification() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE active_coordination_decisions(
                     decision_id TEXT PRIMARY KEY, objective_id TEXT NOT NULL,
                     delegation_id TEXT NOT NULL, generation INTEGER NOT NULL,
                     event_id TEXT NOT NULL, policy_version INTEGER NOT NULL,
                     kind TEXT NOT NULL, payload_json BLOB NOT NULL
                 );
                 CREATE TABLE delegation_generations(
                     delegation_id TEXT NOT NULL, generation INTEGER NOT NULL,
                     state TEXT NOT NULL, payload_json BLOB NOT NULL,
                     PRIMARY KEY(delegation_id, generation)
                 );
                 CREATE TABLE notification_outbox(
                     message_id TEXT PRIMARY KEY, idempotency_key TEXT NOT NULL UNIQUE,
                     issued_at INTEGER NOT NULL,
                     objective_id TEXT NOT NULL, delegation_id TEXT, generation INTEGER,
                     event_id TEXT NOT NULL, policy_version INTEGER NOT NULL,
                     recipient TEXT NOT NULL, message_bytes BLOB NOT NULL,
                     state TEXT NOT NULL, last_issue_json BLOB, terminal INTEGER NOT NULL
                 );",
            )
            .unwrap();
        let objectif_id = Uuid::new_v4();
        let delegation_id = Uuid::new_v4();
        let current = GenerationDelegation {
            delegation_id,
            objectif_id,
            generation: 1,
            participant_id: "alice".to_string(),
            etat: EtatGenerationDelegation::Bloquee,
            generation_precedente: None,
            trigger_event_id: None,
        };
        connection
            .execute(
                "INSERT INTO delegation_generations(delegation_id,generation,state,payload_json)
                 VALUES (?1,1,'bloquee',?2)",
                params![
                    delegation_id.to_string(),
                    serde_json::to_vec(&current).unwrap()
                ],
            )
            .unwrap();
        let mut opened = current.clone();
        opened.etat = EtatGenerationDelegation::Ouverte;
        opened.trigger_event_id = Some("event-atomic".to_string());
        let decision = DecisionCoordinationActive {
            decision_id: Uuid::new_v4(),
            objectif_id,
            delegation_id,
            generation: 1,
            event_id: "event-atomic".to_string(),
            policy_version: 1,
            kind: TypeDecisionCoordinationActive::Ouvrir,
            motif: "opening_attested".to_string(),
        };
        let reduction = ReductionCoordinationActive {
            decision: decision.clone(),
            transition: TransitionCoordinationActive::Generation(opened),
            outboxes: vec![NotificationOutbox {
                message_id: Uuid::new_v4(),
                idempotency_key: "event-atomic:alice:1".to_string(),
                issued_at: 1_787_500_100,
                objectif_id,
                delegation_id: Some(delegation_id),
                generation: Some(1),
                event_id: "event-atomic".to_string(),
                policy_version: 1,
                recipient: "alice".to_string(),
                message_bytes: b"notification-v1".to_vec(),
                etat: EtatNotificationOutbox::Prepared,
            }],
        };
        // Oracle de mutation : retirer l'insertion de l'outbox ou déplacer son
        // écriture après l'observateur rend la reprise ci-dessous divergente.
        assert_eq!(reduction.outboxes.len(), 1);
        let decision_bytes = serde_json::to_vec(&decision).unwrap();
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        let result = persist_coordination_effects(
            &tx,
            &current,
            &reduction,
            &decision_bytes,
            &mut |phase| {
                if phase == CoordinationCommitPhase::AfterOutboxes {
                    return Err(StoreError::Conflict("faute après outbox"));
                }
                Ok(())
            },
        );
        assert!(matches!(
            result,
            Err(StoreError::Conflict("faute après outbox"))
        ));
        drop(tx);

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM active_coordination_decisions",
                    [],
                    |row| { row.get::<_, i64>(0) }
                )
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row("SELECT state FROM delegation_generations", [], |row| {
                    row.get::<_, String>(0)
                })
                .unwrap(),
            "bloquee"
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM notification_outbox", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            0
        );

        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .unwrap();
        persist_coordination_effects(&tx, &current, &reduction, &decision_bytes, &mut |_| Ok(()))
            .unwrap();
        tx.commit().unwrap();
        let message_id = reduction.outboxes[0].message_id;
        let mut store = MaicieStore {
            path: PathBuf::from(":memory:"),
            connection,
            issuer_scope: "test_scope_012345678901234567890123".to_string(),
            control: ControlSnapshot::Unread,
        };
        let pending = store.pending_notification_outboxes().unwrap();
        assert_eq!(pending, reduction.outboxes);
        store
            .record_notification_issue(
                message_id,
                &IdempotencyIssue::OutcomeUnknown {
                    expires_at: 100,
                    delivery_id: Some("delivery-1".to_string()),
                },
            )
            .unwrap();
        assert_eq!(
            store.pending_notification_outboxes().unwrap()[0].etat,
            EtatNotificationOutbox::OutcomeUnknown
        );
        let accepted = IdempotencyIssue::Accepted { expires_at: 100 };
        store
            .record_notification_issue(message_id, &accepted)
            .unwrap();
        store
            .record_notification_issue(message_id, &accepted)
            .unwrap();
        assert!(store.pending_notification_outboxes().unwrap().is_empty());
        assert!(
            store
                .record_notification_issue(message_id, &IdempotencyIssue::IdempotencyExpired)
                .is_err()
        );
    }
}

#[cfg(test)]
mod migration_v20_tests {
    use super::*;
    use rusqlite::types::Value;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    fn reception_row(connection: &Connection) -> Vec<Value> {
        connection
            .query_row(
                "SELECT issuer_scope,request_id,operation,canonical_request_bytes,
                        objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
                        claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
                 FROM guichet_receptions WHERE request_id='request-v19-history'",
                [],
                |row| (0..15).map(|index| row.get(index)).collect(),
            )
            .unwrap()
    }

    #[test]
    fn vraie_v19_migre_vers_v20_en_conservant_le_recu_octet_pour_octet() {
        let root = std::env::temp_dir().join(format!(
            "maicie-real-v19-v20-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        let mut connection = Connection::open(&database).unwrap();
        migrate_to_version(&mut connection, false, 19).unwrap();
        connection
            .execute(
                "INSERT INTO guichet_receptions(
                     issuer_scope,request_id,operation,canonical_request_bytes,
                     objective_id,delegation_id,delivery_hash,in_reply_to,response_message_id,
                     claim_generation,claim_token,outcome,reply_bytes,decision_id,processed_at
                 ) VALUES(
                     'scope-v19','request-v19-history','delivery_report',X'000102FF',
                     NULL,NULL,'hash-v19','message-v19','response-v19',
                     3,'claim-v19','accepted',X'7B226F6374657473223A22763139227D',NULL,1787000000
                 )",
                [],
            )
            .unwrap();
        let before = reception_row(&connection);
        let v19_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='guichet_receptions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(v19_sql.contains("CHECK(operation IN"));
        drop(connection);
        fs::set_permissions(&database, fs::Permissions::from_mode(0o600)).unwrap();

        let store = MaicieStore::open_with_migration_consent(&database, true).unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
        drop(store);

        let connection = Connection::open(&database).unwrap();
        assert_eq!(reception_row(&connection), before);
        let v20_sql: String = connection
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='guichet_receptions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!v20_sql.contains("CHECK(operation"));
        let probes: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM guichet_receptions WHERE issuer_scope='maicie-v20-shape'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(probes, 0, "les préflights v20 doivent être rollbackés");
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn operation_inconnue_durable_est_refusee_par_le_vocabulaire_rust() {
        let root = std::env::temp_dir().join(format!(
            "maicie-v20-unknown-operation-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        fs::create_dir(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        drop(MaicieStore::open(&database).unwrap());
        let mut connection = Connection::open(&database).unwrap();
        let tx = connection.transaction().unwrap();
        tx
            .execute(
                "INSERT INTO guichet_receptions(
                     issuer_scope,request_id,operation,canonical_request_bytes,
                     response_message_id,claim_generation,claim_token,outcome,reply_bytes,processed_at
                 ) VALUES('scope-v20','request-unknown','future_operation',X'01',
                          'response-unknown',1,'claim-unknown','accepted',X'02',1)",
                [],
            )
            .unwrap();
        let observed: String = tx
            .query_row(
                "SELECT operation FROM guichet_receptions
                 WHERE issuer_scope='scope-v20' AND request_id='request-unknown'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(observed, "future_operation");
        assert!(matches!(
            load_guichet_reception(&tx, "scope-v20", "request-unknown"),
            Err(StoreError::Corrupt("opération guichet inconnue"))
        ));
        tx.commit().unwrap();
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parseur_rust_refuse_le_nom_inconnu_observe() {
        let observed = "future_operation";
        assert!(matches!(
            parse_operation_name(observed),
            Err(StoreError::Corrupt("opération guichet inconnue"))
        ));
    }
}
