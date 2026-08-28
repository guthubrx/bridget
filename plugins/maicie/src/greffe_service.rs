//! Services partagés des mutations du greffe central.
//!
//! Cette frontière ne reçoit jamais de chemin depuis une requête fédérée : la
//! configuration du maître fournit seule la base, le journal et la socket.
//! Le CLI local et le guichet appellent les mêmes fonctions d'effet.

use crate::MAICIE_IDENTITY;
use crate::app::{
    CatalogueReconcileError, DelegateError, DelegateRequest, DelegateResult, DelegationCandidate,
    ObjectiveError, close_with_costs, delegate, pin_coordination_policy,
    reconcile_catalogue_from_store,
};
use crate::bridget_client::{
    AgentInfo, BridgetClient, BridgetClientError, BridgetClientLimits, GuichetClaim,
};
use crate::catalogue::{
    AppendOutcome, CatalogueEntry, CatalogueError, CatalogueJournal, parse_closed_line,
};
use crate::config::MaicieConfig;
use crate::domain::guichet::{
    DelegateMutationStatus, MutationReply, RegistreAddMutationStatus, RequeteCanonique,
    RequeteGuichet,
};
use crate::domain::{
    CoutMissionAgent, CoutMissionCompteurs, DecisionCoordination, ObjectiveOpeningPermit,
};
use crate::store::{MaicieStore, StoreError, StoredGuichetReply};
use bridget_transport::greffe_authorization::{
    GreffeAuthorizationGate, GreffeAuthorizationRefusal, GreffeEffectAuthorization,
    GreffeMutationAction,
};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistreAddResult {
    pub constat_id: String,
    pub outcome: AppendOutcome,
}

#[derive(Debug)]
pub enum GreffeServiceError {
    Invalid(&'static str),
    CatalogueAbsent,
    Catalogue(CatalogueError),
    CatalogueReconcile(CatalogueReconcileError),
    Objective(ObjectiveError),
    Store(StoreError),
    Bridget(BridgetClientError),
    Delegate(DelegateError),
    Authorization(GreffeAuthorizationRefusal),
}

impl fmt::Display for GreffeServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(reason) => write!(formatter, "service du greffe invalide : {reason}"),
            Self::CatalogueAbsent => {
                formatter.write_str("catalogue_path absent de la configuration du greffe central")
            }
            Self::Catalogue(error) => write!(formatter, "journal du greffe impossible : {error}"),
            Self::CatalogueReconcile(error) => {
                write!(formatter, "réconciliation du journal impossible : {error}")
            }
            Self::Objective(error) => write!(formatter, "clôture du greffe impossible : {error}"),
            Self::Store(error) => write!(formatter, "stockage du greffe impossible : {error}"),
            Self::Bridget(error) => write!(formatter, "annuaire Bridget indisponible : {error}"),
            Self::Delegate(error) => write!(formatter, "délégation du greffe impossible : {error}"),
            Self::Authorization(error) => {
                write!(formatter, "autorisation du greffe refusée : {error}")
            }
        }
    }
}

impl std::error::Error for GreffeServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bridget(error) => Some(error),
            Self::Delegate(error) => Some(error),
            Self::Authorization(error) => Some(error),
            Self::Catalogue(error) => Some(error),
            Self::CatalogueReconcile(error) => Some(error),
            Self::Objective(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::CatalogueAbsent | Self::Invalid(_) => None,
        }
    }
}

/// Projette l'annuaire public dans les faits minimaux du sélecteur Maicie.
/// Cette règle est commune au CLI et aux claims fédérés.
pub fn candidates_from(config: &MaicieConfig, agents: &[AgentInfo]) -> Vec<DelegationCandidate> {
    let mut candidates = config
        .profiles
        .iter()
        .filter_map(|profile| {
            let agent_name = profile.agent_name.as_deref().unwrap_or(&profile.id);
            let agent = agents.iter().find(|agent| agent.name == agent_name)?;
            Some(DelegationCandidate {
                name: agent.name.clone(),
                tags: profile.tags.clone(),
                available: matches!(agent.state.as_str(), "connected" | "dnd"),
                dnd: agent.state == "dnd",
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| left.name.cmp(&right.name));
    candidates
}

/// Applique une délégation et fige, le cas échéant, la politique de
/// coordination déclarée. L'appelant reste responsable du dispatch de
/// l'outbox déjà durable.
pub fn apply_delegate(
    store: &mut MaicieStore,
    config: &MaicieConfig,
    candidates: &[DelegationCandidate],
    request: &DelegateRequest<'_>,
) -> Result<DelegateResult, DelegateError> {
    let result = delegate(
        store,
        config.durations,
        MAICIE_IDENTITY,
        candidates,
        request,
    )?;
    if let (Some(policies), DelegateResult::Created(created)) =
        (&config.coordination_policies, &result)
    {
        pin_coordination_policy(store, policies, created)?;
    }
    Ok(result)
}

/// Applique une mutation déjà parsée depuis le guichet puis persiste son reçu
/// terminal. Un reçu existant gagne avant toute nouvelle I/O ou écriture.
pub fn apply_guichet_mutation(
    store: &mut MaicieStore,
    config: &MaicieConfig,
    limits: BridgetClientLimits,
    claim: &GuichetClaim,
    canonical: &RequeteCanonique,
    response_message_id: &str,
    now: i64,
) -> Result<StoredGuichetReply, GreffeServiceError> {
    if let Some(stored) = store
        .replay_guichet_mutation(claim, canonical)
        .map_err(GreffeServiceError::Store)?
    {
        return Ok(stored);
    }
    let authorization_gate = GreffeAuthorizationGate::from_environment();
    let reply = match &canonical.request {
        RequeteGuichet::Delegate(request) => {
            let client = BridgetClient::connect_with_limits(
                &config.bridget_socket,
                store.issuer_scope(),
                limits,
            )
            .map_err(GreffeServiceError::Bridget)?;
            let agents = client.list_agents().map_err(GreffeServiceError::Bridget)?;
            let candidates = candidates_from(config, &agents);
            let retry_until = now
                .checked_add(client.negotiated().horizon_secs)
                .ok_or(GreffeServiceError::Invalid("horizon Bridget hors borne"))?;
            let delegate_request = DelegateRequest {
                goal: &request.goal,
                opening_permit: ObjectiveOpeningPermit::auto_generated(),
                explicit_target: request.explicit_target.as_deref(),
                required_tags: &request.required_tags,
                duration: request.duration,
                reply: false,
                constat_id: None,
                review_target: request.review_target.as_ref(),
                suite: request.suite.clone(),
                depends_on: &request.depends_on,
                references: &request.references,
                idempotency_key: &canonical.request_id,
                now,
                retry_until,
                dedup_retained_until: retry_until,
                max_frame_bytes: client.limits().max_frame_bytes,
            };
            let result = authorization_gate
                .authorize_effect_then(
                    GreffeEffectAuthorization {
                        attestation: claim.authorization_attestation.as_ref(),
                        action: GreffeMutationAction::Delegate,
                        issuer_scope: &canonical.issuer_scope,
                        request_id: &canonical.request_id,
                        request_issued_at: canonical.issued_at,
                        canonical_request: &claim.canonical_request,
                        observed_at: now,
                    },
                    |_| apply_delegate(store, config, &candidates, &delegate_request),
                )
                .map_err(GreffeServiceError::Authorization)?
                .map_err(GreffeServiceError::Delegate)?;
            match result {
                DelegateResult::Created(created) => MutationReply::Delegate {
                    status: DelegateMutationStatus::Created,
                    objective_id: Some(created.objective_id.to_string()),
                    delegation_id: Some(created.delegation_id.to_string()),
                    message_id: created.message_id.map(|id| id.to_string()),
                    participant: Some(created.participant),
                    candidates: Vec::new(),
                    waiting_on_prerequisites: created.waiting_on_prerequisites,
                    replayed: created.replayed,
                },
                DelegateResult::Candidates(candidates) => MutationReply::Delegate {
                    status: DelegateMutationStatus::SelectionRequired,
                    objective_id: None,
                    delegation_id: None,
                    message_id: None,
                    participant: None,
                    candidates,
                    waiting_on_prerequisites: false,
                    replayed: false,
                },
            }
        }
        RequeteGuichet::RegistreAdd(request) => {
            let result = authorization_gate
                .authorize_effect_then(
                    GreffeEffectAuthorization {
                        attestation: claim.authorization_attestation.as_ref(),
                        action: GreffeMutationAction::RegistreAdd,
                        issuer_scope: &canonical.issuer_scope,
                        request_id: &canonical.request_id,
                        request_issued_at: canonical.issued_at,
                        canonical_request: &claim.canonical_request,
                        observed_at: now,
                    },
                    |_| append_registre_add(store, config, &request.line),
                )
                .map_err(GreffeServiceError::Authorization)??;
            MutationReply::RegistreAdd {
                status: match result.outcome {
                    AppendOutcome::Appended => RegistreAddMutationStatus::Appended,
                    AppendOutcome::IdempotentNoop => RegistreAddMutationStatus::IdempotentNoop,
                },
                constat_id: result.constat_id,
            }
        }
        RequeteGuichet::ObjectiveClose(request) => {
            let (decision, replayed) = authorization_gate
                .authorize_effect_then(
                    GreffeEffectAuthorization {
                        attestation: claim.authorization_attestation.as_ref(),
                        action: GreffeMutationAction::ObjectiveClose,
                        issuer_scope: &canonical.issuer_scope,
                        request_id: &canonical.request_id,
                        request_issued_at: canonical.issued_at,
                        canonical_request: &claim.canonical_request,
                        observed_at: now,
                    },
                    |_| {
                        close_objective_idempotent(
                            store,
                            config,
                            request.objective_id,
                            &request.reason,
                            now,
                            &canonical.request_id,
                        )
                    },
                )
                .map_err(GreffeServiceError::Authorization)??;
            MutationReply::ObjectiveClose {
                objective_id: request.objective_id.to_string(),
                decision_id: decision.id.to_string(),
                replayed,
            }
        }
        RequeteGuichet::DeliveryReport(_)
        | RequeteGuichet::MissionStatus { .. }
        | RequeteGuichet::DeadlineQuestion { .. } => {
            return Err(GreffeServiceError::Invalid(
                "opération consultative envoyée au service de mutation",
            ));
        }
    };
    store
        .persist_guichet_mutation(claim, canonical, response_message_id, now, &reply)
        .map_err(GreffeServiceError::Store)
}

/// Parse puis ajoute une ligne fermée au journal déclaré par le maître. Aucun
/// chemin fourni par l'appelant ne traverse cette fonction.
pub fn append_registre_add(
    store: &MaicieStore,
    config: &MaicieConfig,
    line: &str,
) -> Result<RegistreAddResult, GreffeServiceError> {
    let entry = parse_closed_line(line.trim()).map_err(GreffeServiceError::Catalogue)?;
    let CatalogueEntry::Add(add) = entry else {
        return Err(GreffeServiceError::Catalogue(CatalogueError::Format(
            "registre add n'accepte qu'une ligne kind=add fermée".to_string(),
        )));
    };
    let constat_id = add.id.clone();
    let path = config
        .catalogue_path
        .as_ref()
        .ok_or(GreffeServiceError::CatalogueAbsent)?;
    let mut journal = CatalogueJournal::open(path).map_err(GreffeServiceError::Catalogue)?;
    reconcile_catalogue_from_store(store, &mut journal)
        .map_err(GreffeServiceError::CatalogueReconcile)?;
    let outcome = journal
        .append_add(add)
        .map_err(GreffeServiceError::Catalogue)?;
    Ok(RegistreAddResult {
        constat_id,
        outcome,
    })
}

/// Clôt l'objectif avec les mêmes coûts attestés (ou inconnus) que le CLI.
pub fn close_objective(
    store: &mut MaicieStore,
    config: &MaicieConfig,
    objective_id: Uuid,
    reason: &str,
    now: i64,
) -> Result<DecisionCoordination, GreffeServiceError> {
    let costs = collect_mission_costs(config, store, objective_id, now);
    close_with_costs(store, objective_id, reason, now, costs).map_err(GreffeServiceError::Objective)
}

/// Variante fédérée : la clé du dépôt rend la clôture rejouable même si le
/// processus tombe après le commit métier mais avant le reçu guichet.
pub fn close_objective_idempotent(
    store: &mut MaicieStore,
    config: &MaicieConfig,
    objective_id: Uuid,
    reason: &str,
    now: i64,
    idempotency_key: &str,
) -> Result<(DecisionCoordination, bool), GreffeServiceError> {
    if reason.trim().is_empty() || now <= 0 {
        return Err(GreffeServiceError::Objective(ObjectiveError::Invalid(
            "motif ou horodatage absent",
        )));
    }
    let costs = collect_mission_costs(config, store, objective_id, now);
    store
        .close_objective_idempotent_with_costs(objective_id, reason, now, costs, idempotency_key)
        .map_err(GreffeServiceError::Store)
}

/// Interroge le ledger Bridget pour chaque agent délégué. Indisponibilité ou
/// absence d'échantillon signifie « inconnu », jamais zéro inventé.
pub fn collect_mission_costs(
    config: &MaicieConfig,
    store: &MaicieStore,
    objective_id: Uuid,
    closed_at: i64,
) -> Vec<CoutMissionAgent> {
    let Ok(windows) = store.delegation_cost_windows(objective_id) else {
        return Vec::new();
    };
    let client = BridgetClient::connect_with_limits(
        &config.bridget_socket,
        crate::USAGE_ISSUER_SCOPE,
        BridgetClientLimits::default(),
    )
    .ok();
    windows
        .into_iter()
        .map(|(agent, from_secs)| {
            let from_secs = from_secs.max(1);
            let to_secs = closed_at.max(from_secs);
            match client
                .as_ref()
                .and_then(|client| client.usage_window(&agent, from_secs, to_secs).ok())
                .flatten()
            {
                Some(aggregate) => CoutMissionAgent::attested(
                    agent,
                    from_secs,
                    to_secs,
                    CoutMissionCompteurs {
                        turns: aggregate.turns,
                        input_tokens: aggregate.input_tokens,
                        output_tokens: aggregate.output_tokens,
                        cache_creation_input_tokens: aggregate.cache_creation_input_tokens,
                        cache_read_input_tokens: aggregate.cache_read_input_tokens,
                    },
                ),
                None => CoutMissionAgent::unknown(agent, from_secs, to_secs),
            }
        })
        .collect()
}
