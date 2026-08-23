//! Interface de ligne de commande du compagnon Maicie.
//!
//! La surface reste une projection mince des cas d'usage : elle charge une
//! configuration explicite, consulte l'annuaire public puis délègue la
//! décision durable à `app`. Après le commit, elle délègue l'émission au
//! réconciliateur d'outbox commun : aucun second chemin d'envoi n'existe.

use maicie::app::{
    LocalProfileApproval, ProfileActivationError, ProfileActivationProposalRequest,
    add_participant, approve_profile_activation, close, delegate, delegated_participants,
    propose_profile_activation, remove_participant, status, stored_profile_activation_proposal,
    summarize, DelegateError, DelegateRequest, DelegateResult, DelegationCandidate, ObjectiveError,
};
use maicie::bridget_client::{
    AgentInfo, AttachWindow, BridgetClient, BridgetClientError, BridgetClientLimits,
};
use maicie::config::{ConfigError, MaicieConfig};
use maicie::domain::{
    ClasseDuree, DecisionCoordination, Delegation, EtatFlux, ObjectifCoordonne, SourceSnapshot,
};
use maicie::profiles::{ApprovalProfileView, ProfileError, ResolvedAgentDefinition, approval_view, load_profiles};
use maicie::reconcile::{reconcile_activation_startup_at, reconcile_startup_with_limits, ReconcileError};
use maicie::runtime::{RuntimeNature, RuntimeObservation, RuntimeSignal, RuntimeSubscription};
use maicie::store::{MaicieStore, ObjectiveSnapshot, StoreError};
use maicie::MAICIE_IDENTITY;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const EXIT_USAGE: u8 = 2;
const EXIT_CONFIGURATION: u8 = 3;
const EXIT_BRIDGET: u8 = 4;
const EXIT_DELEGATE: u8 = 5;
const EXIT_STORE: u8 = 6;
const MAX_STATUS_RUNTIME_OBSERVATIONS: usize = 256;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", error.as_json());
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(arguments: Vec<String>) -> Result<String, CliError> {
    let command = parse_command(&arguments)?;
    match command {
        Command::Delegate(delegate_args) => run_delegate(delegate_args),
        Command::Status(status_args) => run_status(status_args),
        Command::Objective(objective_args) => run_objective(objective_args),
        Command::Profile(profile_args) => run_profile(profile_args),
    }
}

fn run_status(arguments: StatusArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let store = open_store_with_reconciliation(&config, BridgetClientLimits::default())?;
    let snapshots = status(&store, arguments.objective_id).map_err(CliError::Objective)?;
    let sources = capture_status_sources(&config, &delegated_participants(&snapshots));
    render_objective_output(
        ObjectiveOutput::Status {
            coordination: snapshots.into_iter().map(SnapshotOutput::from).collect(),
            availability: sources.availability,
            availability_state: sources.availability_state,
            availability_reason: sources.availability_reason,
            transport_snapshot: TransportSnapshotOutput::unknown(),
            runtime: sources.runtime,
            freshness: sources.freshness,
            stream_state: sources.stream_state,
        },
        arguments.json,
    )
}

/// Réalise une capture Attach entièrement éphémère. Elle est limitée par une
/// seule échéance absolue et ne touche jamais SQLite : la sortie décrit donc
/// une observation de cette consultation, pas un cache déguisé en runtime.
fn capture_status_sources(config: &MaicieConfig, participants: &[String]) -> StatusSourcesOutput {
    let Some(budget_ms) = config.status_capture_budget_ms else {
        return StatusSourcesOutput::unknown("budget_capture_non_configure");
    };
    let started_at = match unix_now() {
        Ok(value) => value,
        Err(_) => return StatusSourcesOutput::unknown("horloge_indisponible"),
    };
    let deadline = Instant::now() + Duration::from_millis(budget_ms);
    let client = match BridgetClient::connect_with_limits_until(
        &config.bridget_socket,
        "maicie-status",
        status_limits(deadline),
        deadline,
    ) {
        Ok(client) => client,
        Err(error) => return StatusSourcesOutput::unknown(&capture_reason(&error)),
    };
    let agents = match client.list_agents_until(deadline) {
        Ok(agents) => agents,
        Err(error) => return StatusSourcesOutput::unknown(&capture_reason(&error)),
    };

    let availability = agents
        .iter()
        .filter(|agent| {
            participants
                .iter()
                .any(|participant| participant == &agent.name)
        })
        .map(|agent| AvailabilityOutput::from_agent(agent, started_at))
        .collect::<Vec<_>>();
    let mut runtime = Vec::with_capacity(participants.len());
    for participant in participants {
        if Instant::now() >= deadline {
            runtime.push(RuntimeAgentOutput::unknown(
                participant,
                "budget_capture_epuise",
            ));
            continue;
        }
        let Some(agent) = agents.iter().find(|agent| agent.name == *participant) else {
            runtime.push(RuntimeAgentOutput::unknown(
                participant,
                "agent_absent_annuaire",
            ));
            continue;
        };
        if agent.transport != "acp" {
            runtime.push(RuntimeAgentOutput::unknown(
                participant,
                "transport_non_acp",
            ));
            continue;
        }
        if agent.state != "connected" && agent.state != "dnd" {
            runtime.push(RuntimeAgentOutput::unknown(
                participant,
                "agent_non_connecte",
            ));
            continue;
        }
        runtime.push(capture_runtime_agent(&client, participant, deadline));
    }

    StatusSourcesOutput::from_capture(availability, runtime, started_at)
}

/// Réduit chaque délai filaire à l'échéance globale de la consultation. Le
/// client applique ensuite la durée restante à chaque lecture/écriture.
fn status_limits(deadline: Instant) -> BridgetClientLimits {
    let remaining = deadline.saturating_duration_since(Instant::now());
    BridgetClientLimits {
        connect_timeout: remaining,
        io_timeout: remaining,
        ..BridgetClientLimits::default()
    }
}

fn capture_runtime_agent(
    client: &BridgetClient,
    participant: &str,
    deadline: Instant,
) -> RuntimeAgentOutput {
    let mut subscription =
        match RuntimeSubscription::open_until(client, participant, AttachWindow::Today, deadline) {
            Ok(subscription) => subscription,
            Err(error) => {
                return RuntimeAgentOutput::unknown(participant, &capture_runtime_reason(&error))
            }
        };
    let subscription_id = subscription.subscription_id().to_string();
    let mut observations = Vec::new();
    loop {
        if observations.len() >= MAX_STATUS_RUNTIME_OBSERVATIONS {
            return RuntimeAgentOutput {
                agent: participant.to_string(),
                source: "acp_subscription",
                subscription_id: Some(subscription_id),
                stream_state: EtatFlux::Unavailable,
                captured_at: None,
                reason: Some("limite_evenements_capture".to_string()),
                observations,
            };
        }
        match subscription.next_signal_until(deadline) {
            Ok(RuntimeSignal::Observation(observation)) => {
                observations.push(RuntimeObservationOutput::from(observation));
            }
            Ok(RuntimeSignal::SnapshotCaughtUp { .. }) => {
                return RuntimeAgentOutput {
                    agent: participant.to_string(),
                    source: "acp_subscription",
                    subscription_id: Some(subscription_id),
                    stream_state: subscription.stream_state(),
                    captured_at: unix_now().ok(),
                    reason: None,
                    observations,
                };
            }
            Ok(RuntimeSignal::Gap { .. } | RuntimeSignal::JournalReadError { .. }) => {
                return RuntimeAgentOutput {
                    agent: participant.to_string(),
                    source: "acp_subscription",
                    subscription_id: Some(subscription_id),
                    stream_state: EtatFlux::Gap,
                    captured_at: unix_now().ok(),
                    reason: Some("flux_incomplet".to_string()),
                    observations,
                };
            }
            Ok(RuntimeSignal::End { .. }) => {
                return RuntimeAgentOutput {
                    agent: participant.to_string(),
                    source: "acp_subscription",
                    subscription_id: Some(subscription_id),
                    stream_state: EtatFlux::Ended,
                    captured_at: unix_now().ok(),
                    reason: Some("abonnement_termine".to_string()),
                    observations,
                };
            }
            Err(error) => {
                return RuntimeAgentOutput::unknown(participant, &capture_runtime_reason(&error))
            }
        }
    }
}

fn capture_reason(error: &BridgetClientError) -> String {
    match error {
        BridgetClientError::Timeout { .. } => "budget_capture_epuise".to_string(),
        _ => "annuaire_bridget_indisponible".to_string(),
    }
}

fn capture_runtime_reason(error: &maicie::runtime::RuntimeError) -> String {
    match error {
        maicie::runtime::RuntimeError::Transport(BridgetClientError::Timeout { .. }) => {
            "budget_capture_epuise".to_string()
        }
        maicie::runtime::RuntimeError::Ended => "abonnement_termine".to_string(),
        _ => "abonnement_indisponible".to_string(),
    }
}

fn run_objective(arguments: ObjectiveArgs) -> Result<String, CliError> {
    let mut store = open_store(&arguments.config)?;
    let output = match arguments.action {
        ObjectiveAction::AddParticipant { participant } => {
            let decision = add_participant(&mut store, arguments.objective_id, &participant)
                .map_err(CliError::Objective)?;
            ObjectiveOutput::Decision { decision }
        }
        ObjectiveAction::RemoveParticipant {
            participant,
            reason,
        } => {
            let decision =
                remove_participant(&mut store, arguments.objective_id, &participant, &reason)
                    .map_err(CliError::Objective)?;
            ObjectiveOutput::Decision { decision }
        }
        ObjectiveAction::Summarize => {
            let snapshot =
                summarize(&store, arguments.objective_id).map_err(CliError::Objective)?;
            ObjectiveOutput::Summary {
                coordination: SnapshotOutput::from(snapshot),
            }
        }
        ObjectiveAction::Close { reason } => {
            let decision = close(&mut store, arguments.objective_id, &reason, unix_now()?)
                .map_err(CliError::Objective)?;
            ObjectiveOutput::Decision { decision }
        }
    };
    render_objective_output(output, arguments.json)
}

fn open_store(config_path: &PathBuf) -> Result<MaicieStore, CliError> {
    let config = MaicieConfig::load(config_path).map_err(CliError::Configuration)?;
    open_store_with_reconciliation(&config, BridgetClientLimits::default())
}

/// Toute commande qui ouvre la base rejoue d'abord les outboxes pendantes dans
/// une fenêtre I/O bornée. L'indisponibilité Bridget laisse la ligne durable
/// pending ; les erreurs de contrat restent explicites au CLI.
fn open_store_with_reconciliation(
    config: &MaicieConfig,
    limits: BridgetClientLimits,
) -> Result<MaicieStore, CliError> {
    let mut store = MaicieStore::open(&config.database_path).map_err(CliError::Store)?;
    reconcile_pending(&mut store, config, limits)?;
    reconcile_activation_startup_at(&mut store, &config.bridget_socket, unix_now()?)
        .map_err(CliError::Reconcile)?;
    Ok(store)
}

fn reconcile_pending(
    store: &mut MaicieStore,
    config: &MaicieConfig,
    limits: BridgetClientLimits,
) -> Result<(), CliError> {
    reconcile_startup_with_limits(store, &config.bridget_socket, limits)
        .map(|_| ())
        .map_err(CliError::Reconcile)
}

fn run_delegate(arguments: DelegateArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let limits = BridgetClientLimits::default();
    let mut store = open_store_with_reconciliation(&config, limits)?;
    let client =
        BridgetClient::connect_with_limits(&config.bridget_socket, store.issuer_scope(), limits)
            .map_err(CliError::Bridget)?;
    let now = unix_now()?;
    let retry_until = now
        .checked_add(client.negotiated().horizon_secs)
        .ok_or(CliError::Usage("horizon Bridget hors borne"))?;
    let agents = client.list_agents().map_err(CliError::Bridget)?;
    let candidates = candidates_from(&config, &agents);
    let idempotency_key = arguments
        .idempotency_key
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let request = DelegateRequest {
        goal: &arguments.goal,
        explicit_target: arguments.target.as_deref(),
        required_tags: &arguments.required_tags,
        duration: arguments.duration,
        // Maicie est un client public durable, pas un wrapper enregistré :
        // demander une réponse Bridget serait refusé avant livraison. La
        // corrélation de réponse attend T015b/Subscribe, sans la simuler ici.
        reply: false,
        idempotency_key: &idempotency_key,
        now,
        retry_until,
        dedup_retained_until: retry_until,
        max_frame_bytes: client.limits().max_frame_bytes,
    };
    let result = delegate(
        &mut store,
        config.durations,
        MAICIE_IDENTITY,
        &candidates,
        &request,
    )
    .map_err(CliError::Delegate)?;
    // La transaction `delegate` est déjà commitée ici. T008 effectue ensuite
    // lookup puis replay des octets persistés, sans reconstruire le message.
    reconcile_pending(&mut store, &config, limits)?;
    render_output(DelegateOutput::from(result), arguments.json)
        .map_err(|_| CliError::Delegate(DelegateError::Invalid("sortie JSON indisponible")))
}

/// Expose le consentement local US4 sans jamais lancer de processus. La
/// proposition ne fait qu'écrire l'approbation ; l'approbation ne produit que
/// l'outbox, ensuite reprise par le protocole public Bridget.
fn run_profile(arguments: ProfileArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = open_store_with_reconciliation(&config, BridgetClientLimits::default())?;
    let now = unix_now()?;
    match arguments.action {
        ProfileAction::Propose {
            objective_id,
            profile_id,
            context_scope,
            cwd,
            persistent,
            reason,
        } => {
            let (screen, profile_hash) = approval_screen(&config, &profile_id, &arguments.definition)?;
            let retry_until = deadline_from(now, config.durations.long_secs)?;
            let proposal = propose_profile_activation(
                &mut store,
                &ProfileActivationProposalRequest {
                    objective_id,
                    profile_id: &screen.profile.id,
                    agent_type: &screen.profile.agent_type,
                    profile_hash: &profile_hash,
                    resolved_definition_digest: &screen.definition_digest,
                    context_scope: &context_scope,
                    cwd: &cwd,
                    persistent,
                    now,
                    spawn_deadline_at: deadline_from(now, config.durations.normal_secs)?,
                    approval_expires_at: deadline_from(now, config.durations.normal_secs)?,
                    retry_until,
                    dedup_retained_until: retry_until,
                    reason: &reason,
                },
            )
            .map_err(CliError::ProfileActivation)?;
            render_profile_output(
                ProfileOutput::Proposed {
                    approval_id: proposal.approval.id,
                    command_id: proposal.approval.command_id,
                    expires_at: proposal.approval.expires_at,
                    screen: ApprovalScreenOutput::from(screen),
                },
                arguments.json,
            )
        }
        ProfileAction::Approve { approval_id } => {
            let retry_until = deadline_from(now, config.durations.long_secs)?;
            let proposal = stored_profile_activation_proposal(
                &store,
                approval_id,
                retry_until,
                retry_until,
            )
            .map_err(CliError::ProfileActivation)?;
            let (screen, profile_hash) = approval_screen(
                &config,
                &proposal.approval.profile_id,
                &arguments.definition,
            )?;
            let activation = approve_profile_activation(
                &mut store,
                &proposal,
                &LocalProfileApproval {
                    approval_id,
                    now,
                    profile_hash: &profile_hash,
                    resolved_definition_digest: &screen.definition_digest,
                },
            )
            .map_err(CliError::ProfileActivation)?;
            // Une approbation arrive après la passe de démarrage : rejouer la
            // même routine ici rend son SpawnOrder éligible sans inventer un
            // second chemin d'émission.
            reconcile_activation_startup_at(&mut store, &config.bridget_socket, now)
                .map_err(CliError::Reconcile)?;
            render_profile_output(
                ProfileOutput::Approved {
                    approval_id,
                    command_id: activation.command_id,
                    actor: "local_human",
                    screen: ApprovalScreenOutput::from(screen),
                },
                arguments.json,
            )
        }
    }
}

fn approval_screen(
    config: &MaicieConfig,
    profile_id: &str,
    definition_path: &PathBuf,
) -> Result<(ApprovalProfileView, Vec<u8>), CliError> {
    let loaded = load_profiles(&config.profiles).map_err(CliError::Profile)?;
    let profile = loaded
        .into_iter()
        .find(|profile| profile.id == profile_id)
        .ok_or(CliError::Usage("profil inconnu"))?;
    let profile_config = config
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .ok_or(CliError::Usage("profil inconnu"))?;
    let definition_bytes = fs::read(definition_path)
        .map_err(|_| CliError::Usage("définition résolue illisible"))?;
    let definition: ResolvedAgentDefinition = serde_json::from_slice(&definition_bytes)
        .map_err(|_| CliError::Usage("définition résolue invalide"))?;
    let profile_bytes = serde_json::to_vec(profile_config)
        .map_err(|_| CliError::Usage("profil non sérialisable"))?;
    let profile_hash = Sha256::digest(profile_bytes).to_vec();
    approval_view(profile, definition)
        .map(|view| (view, profile_hash))
        .map_err(CliError::Profile)
}

fn deadline_from(now: i64, seconds: u64) -> Result<i64, CliError> {
    now.checked_add(i64::try_from(seconds).map_err(|_| CliError::Usage("délai hors borne"))?)
        .ok_or(CliError::Usage("échéance hors borne"))
}

fn render_output(output: DelegateOutput, json: bool) -> Result<String, serde_json::Error> {
    if json {
        return serde_json::to_string(&output);
    }
    Ok(match output {
        DelegateOutput::Created {
            objective_id,
            delegations,
            replayed,
            ..
        } => {
            let delegation = &delegations[0];
            format!(
                "objectif={objective_id} délégation={} participant={} message_id={} état=prepared replayed={replayed}",
                delegation.id, delegation.participant, delegation.message_id
            )
        }
        DelegateOutput::Candidates { candidates } => format!("candidats={}", candidates.join(",")),
    })
}

fn candidates_from(config: &MaicieConfig, agents: &[AgentInfo]) -> Vec<DelegationCandidate> {
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

fn unix_now() -> Result<i64, CliError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CliError::Usage("horloge système antérieure à Unix"))?;
    i64::try_from(duration.as_secs()).map_err(|_| CliError::Usage("horodatage hors borne"))
}

#[derive(Debug)]
enum Command {
    Delegate(DelegateArgs),
    Status(StatusArgs),
    Objective(ObjectiveArgs),
    Profile(ProfileArgs),
}

#[derive(Debug)]
struct DelegateArgs {
    config: PathBuf,
    goal: String,
    target: Option<String>,
    required_tags: Vec<String>,
    duration: ClasseDuree,
    idempotency_key: Option<String>,
    json: bool,
}

#[derive(Debug)]
struct StatusArgs {
    config: PathBuf,
    objective_id: Option<Uuid>,
    json: bool,
}

#[derive(Debug)]
struct ObjectiveArgs {
    config: PathBuf,
    objective_id: Uuid,
    action: ObjectiveAction,
    json: bool,
}

#[derive(Debug)]
enum ObjectiveAction {
    AddParticipant { participant: String },
    RemoveParticipant { participant: String, reason: String },
    Summarize,
    Close { reason: String },
}

#[derive(Debug)]
struct ProfileArgs {
    config: PathBuf,
    definition: PathBuf,
    action: ProfileAction,
    json: bool,
}

#[derive(Debug)]
enum ProfileAction {
    Propose {
        objective_id: Uuid,
        profile_id: String,
        context_scope: String,
        cwd: String,
        persistent: bool,
        reason: String,
    },
    /// Cette variante n'est constructible que par la sous-commande locale et
    /// son drapeau `--confirm`. Elle n'appartient à aucun protocole Bridget ou
    /// MCP : le domaine fixe ensuite toujours actor=local_human.
    Approve { approval_id: Uuid },
}

fn parse_command(arguments: &[String]) -> Result<Command, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("commande attendue : delegate"));
    };
    match verb.as_str() {
        "delegate" => parse_delegate(tail).map(Command::Delegate),
        "status" => parse_status(tail).map(Command::Status),
        "objective" => parse_objective(tail).map(Command::Objective),
        "profile" => parse_profile(tail).map(Command::Profile),
        _ => Err(CliError::Usage("commande inconnue : delegate attendu")),
    }
}

fn parse_profile(arguments: &[String]) -> Result<ProfileArgs, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("action profile obligatoire"));
    };
    let (action, config, definition, json) = match verb.as_str() {
        "propose" => parse_profile_propose(tail)?,
        "approve" => parse_profile_approve(tail)?,
        _ => return Err(CliError::Usage("action profile inconnue")),
    };
    Ok(ProfileArgs {
        config: config.ok_or(CliError::Usage("--config est obligatoire"))?,
        definition: definition.ok_or(CliError::Usage("--definition est obligatoire"))?,
        action,
        json,
    })
}

fn parse_profile_propose(
    arguments: &[String],
) -> Result<(ProfileAction, Option<PathBuf>, Option<PathBuf>, bool), CliError> {
    let Some((objective_id, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("identifiant objectif obligatoire"));
    };
    let Some((profile_id, tail)) = tail.split_first() else {
        return Err(CliError::Usage("identifiant profil obligatoire"));
    };
    let objective_id = parse_objective_id(objective_id)?;
    let mut config = None;
    let mut definition = None;
    let mut context_scope = None;
    let mut cwd = None;
    let mut reason = None;
    let mut persistent = false;
    let mut json = false;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(
                &mut config,
                next_value(tail, &mut index, "--config")?,
            )?,
            "--definition" => set_once_path(
                &mut definition,
                next_value(tail, &mut index, "--definition")?,
            )?,
            "--context-scope" => set_once_string(
                &mut context_scope,
                next_value(tail, &mut index, "--context-scope")?,
                "context-scope",
            )?,
            "--cwd" => set_once_string(&mut cwd, next_value(tail, &mut index, "--cwd")?, "cwd")?,
            "--reason" => set_once_string(
                &mut reason,
                next_value(tail, &mut index, "--reason")?,
                "reason",
            )?,
            "--persistent" => {
                if persistent {
                    return Err(CliError::Usage("option --persistent dupliquée"));
                }
                persistent = true;
            }
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option profile propose inconnue")),
        }
        index += 1;
    }
    Ok((
        ProfileAction::Propose {
            objective_id,
            profile_id: profile_id.to_string(),
            context_scope: context_scope
                .ok_or(CliError::Usage("--context-scope est obligatoire"))?,
            cwd: cwd.ok_or(CliError::Usage("--cwd est obligatoire"))?,
            persistent,
            reason: reason.ok_or(CliError::Usage("--reason est obligatoire"))?,
        },
        config,
        definition,
        json,
    ))
}

fn parse_profile_approve(
    arguments: &[String],
) -> Result<(ProfileAction, Option<PathBuf>, Option<PathBuf>, bool), CliError> {
    let Some((approval_id, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("identifiant approbation obligatoire"));
    };
    let approval_id = Uuid::parse_str(approval_id)
        .map_err(|_| CliError::Usage("identifiant approbation UUID invalide"))?;
    let mut config = None;
    let mut definition = None;
    let mut confirmed = false;
    let mut json = false;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(
                &mut config,
                next_value(tail, &mut index, "--config")?,
            )?,
            "--definition" => set_once_path(
                &mut definition,
                next_value(tail, &mut index, "--definition")?,
            )?,
            "--confirm" => {
                if confirmed {
                    return Err(CliError::Usage("option --confirm dupliquée"));
                }
                confirmed = true;
            }
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option profile approve inconnue")),
        }
        index += 1;
    }
    if !confirmed {
        return Err(CliError::Usage("--confirm est obligatoire"));
    }
    Ok((ProfileAction::Approve { approval_id }, config, definition, json))
}

fn parse_status(arguments: &[String]) -> Result<StatusArgs, CliError> {
    let mut config = None;
    let mut objective_id = None;
    let mut json = false;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        match argument.as_str() {
            "--config" => {
                set_once_path(&mut config, next_value(arguments, &mut index, "--config")?)?
            }
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            value if !value.starts_with("--") && objective_id.is_none() => {
                objective_id = Some(parse_objective_id(value)?);
            }
            _ => return Err(CliError::Usage("option status inconnue")),
        }
        index += 1;
    }
    Ok(StatusArgs {
        config: config.ok_or(CliError::Usage("--config est obligatoire"))?,
        objective_id,
        json,
    })
}

fn parse_objective(arguments: &[String]) -> Result<ObjectiveArgs, CliError> {
    let Some((id, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("identifiant objectif obligatoire"));
    };
    let objective_id = parse_objective_id(id)?;
    let Some((verb, tail)) = tail.split_first() else {
        return Err(CliError::Usage("action objectif obligatoire"));
    };
    let (action, config, json) = match verb.as_str() {
        "add-participant" => {
            let Some((participant, flags)) = tail.split_first() else {
                return Err(CliError::Usage("participant obligatoire"));
            };
            let (config, json, reason) = parse_objective_flags(flags, false)?;
            if reason.is_some() {
                return Err(CliError::Usage("--reason interdit pour add-participant"));
            }
            (
                ObjectiveAction::AddParticipant {
                    participant: participant.to_string(),
                },
                config,
                json,
            )
        }
        "remove-participant" => {
            let Some((participant, flags)) = tail.split_first() else {
                return Err(CliError::Usage("participant obligatoire"));
            };
            let (config, json, reason) = parse_objective_flags(flags, true)?;
            (
                ObjectiveAction::RemoveParticipant {
                    participant: participant.to_string(),
                    reason: reason.ok_or(CliError::Usage("--reason est obligatoire"))?,
                },
                config,
                json,
            )
        }
        "summarize" => {
            let (config, json, reason) = parse_objective_flags(tail, false)?;
            if reason.is_some() {
                return Err(CliError::Usage("--reason interdit pour summarize"));
            }
            (ObjectiveAction::Summarize, config, json)
        }
        "close" => {
            let (config, json, reason) = parse_objective_flags(tail, true)?;
            (
                ObjectiveAction::Close {
                    reason: reason.ok_or(CliError::Usage("--reason est obligatoire"))?,
                },
                config,
                json,
            )
        }
        _ => return Err(CliError::Usage("action objectif inconnue")),
    };
    Ok(ObjectiveArgs {
        config: config.ok_or(CliError::Usage("--config est obligatoire"))?,
        objective_id,
        action,
        json,
    })
}

fn parse_objective_flags(
    arguments: &[String],
    allow_reason: bool,
) -> Result<(Option<PathBuf>, bool, Option<String>), CliError> {
    let mut config = None;
    let mut reason = None;
    let mut json = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                set_once_path(&mut config, next_value(arguments, &mut index, "--config")?)?
            }
            "--reason" if allow_reason => set_once_string(
                &mut reason,
                next_value(arguments, &mut index, "--reason")?,
                "reason",
            )?,
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option objectif inconnue")),
        }
        index += 1;
    }
    Ok((config, json, reason))
}

fn parse_objective_id(value: &str) -> Result<Uuid, CliError> {
    Uuid::parse_str(value).map_err(|_| CliError::Usage("identifiant objectif UUID invalide"))
}

fn parse_delegate(arguments: &[String]) -> Result<DelegateArgs, CliError> {
    let mut config = None;
    let mut goal = None;
    let mut target = None;
    let mut duration = ClasseDuree::Normale;
    let mut idempotency_key = None;
    let mut required_tags = Vec::new();
    let mut json = false;
    let mut index = 0;
    while index < arguments.len() {
        let argument = &arguments[index];
        match argument.as_str() {
            "--config" => {
                set_once_path(&mut config, next_value(arguments, &mut index, "--config")?)?
            }
            "--goal" => set_once_string(
                &mut goal,
                next_value(arguments, &mut index, "--goal")?,
                "goal",
            )?,
            "--to" => set_once_string(
                &mut target,
                next_value(arguments, &mut index, "--to")?,
                "to",
            )?,
            "--duration" => {
                duration = parse_duration(next_value(arguments, &mut index, "--duration")?)?;
            }
            "--idempotency-key" => set_once_string(
                &mut idempotency_key,
                next_value(arguments, &mut index, "--idempotency-key")?,
                "idempotency-key",
            )?,
            "--tag" => required_tags.push(next_value(arguments, &mut index, "--tag")?.to_string()),
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option delegate inconnue")),
        }
        index += 1;
    }
    let config = config.ok_or(CliError::Usage("--config est obligatoire"))?;
    let goal = goal.ok_or(CliError::Usage("--goal est obligatoire"))?;
    Ok(DelegateArgs {
        config,
        goal,
        target,
        required_tags,
        duration,
        idempotency_key,
        json,
    })
}

fn next_value<'a>(
    arguments: &'a [String],
    index: &mut usize,
    option: &'static str,
) -> Result<&'a str, CliError> {
    *index += 1;
    arguments
        .get(*index)
        .map(String::as_str)
        .filter(|value| !value.starts_with("--"))
        .ok_or(CliError::Usage(option))
}

fn set_once_string(
    slot: &mut Option<String>,
    value: &str,
    option: &'static str,
) -> Result<(), CliError> {
    if slot.replace(value.to_string()).is_some() {
        return Err(CliError::Usage(option));
    }
    Ok(())
}

fn set_once_path(slot: &mut Option<PathBuf>, value: &str) -> Result<(), CliError> {
    if slot.replace(PathBuf::from(value)).is_some() {
        return Err(CliError::Usage("config"));
    }
    Ok(())
}

fn parse_duration(value: &str) -> Result<ClasseDuree, CliError> {
    match value {
        "courte" => Ok(ClasseDuree::Courte),
        "normale" => Ok(ClasseDuree::Normale),
        "longue" => Ok(ClasseDuree::Longue),
        _ => Err(CliError::Usage("duration : courte, normale ou longue")),
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DelegateOutput {
    Created {
        objective_id: Uuid,
        state: &'static str,
        delegations: Vec<DelegationOutput>,
        replayed: bool,
    },
    Candidates {
        candidates: Vec<String>,
    },
}

#[derive(Serialize)]
struct DelegationOutput {
    id: Uuid,
    participant: String,
    message_id: Uuid,
    coordination_state: &'static str,
    duration: ClasseDuree,
    timeout_secs: u64,
    deadline_contractuelle: i64,
}

impl From<DelegateResult> for DelegateOutput {
    fn from(result: DelegateResult) -> Self {
        match result {
            DelegateResult::Created(created) => Self::Created {
                objective_id: created.objective_id,
                state: "en_coordination",
                delegations: vec![DelegationOutput {
                    id: created.delegation_id,
                    participant: created.participant,
                    message_id: created.message_id,
                    coordination_state: "prepared",
                    duration: created.duration,
                    timeout_secs: created.timeout_secs,
                    deadline_contractuelle: created.deadline_contractuelle,
                }],
                replayed: created.replayed,
            },
            DelegateResult::Candidates(candidates) => Self::Candidates { candidates },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ObjectiveOutput {
    Status {
        coordination: Vec<SnapshotOutput>,
        availability: Vec<AvailabilityOutput>,
        availability_state: EtatFlux,
        availability_reason: Option<String>,
        transport_snapshot: TransportSnapshotOutput,
        runtime: Vec<RuntimeAgentOutput>,
        freshness: FreshnessOutput,
        stream_state: EtatFlux,
    },
    Decision {
        decision: DecisionCoordination,
    },
    Summary {
        coordination: SnapshotOutput,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ProfileOutput {
    Proposed {
        approval_id: Uuid,
        command_id: Uuid,
        expires_at: i64,
        screen: ApprovalScreenOutput,
    },
    Approved {
        approval_id: Uuid,
        command_id: Uuid,
        actor: &'static str,
        screen: ApprovalScreenOutput,
    },
}

/// Écran local de consentement. Chaque champ potentiellement contrôlé par une
/// configuration ou une définition résolue est neutralisé avant tout rendu
/// terminal : aucun ESC, saut de ligne ou contrôle C1 ne peut y être exécuté.
#[derive(Serialize)]
struct ApprovalScreenOutput {
    display_name: String,
    agent_type: String,
    model: String,
    effort: String,
    command: String,
    args: Vec<String>,
    forbidden_env: Vec<String>,
    definition_digest: String,
}

impl From<ApprovalProfileView> for ApprovalScreenOutput {
    fn from(view: ApprovalProfileView) -> Self {
        Self {
            display_name: sanitize_terminal(&view.profile.display_name),
            agent_type: sanitize_terminal(&view.profile.agent_type),
            model: sanitize_terminal(&view.profile.model),
            effort: sanitize_terminal(&view.profile.effort),
            command: sanitize_terminal(&view.command),
            args: view.args.iter().map(|arg| sanitize_terminal(arg)).collect(),
            forbidden_env: view
                .forbidden_env
                .iter()
                .map(|value| sanitize_terminal(value))
                .collect(),
            definition_digest: sanitize_terminal(&view.definition_digest),
        }
    }
}

fn sanitize_terminal(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn render_profile_output(output: ProfileOutput, json: bool) -> Result<String, CliError> {
    if json {
        return serde_json::to_string(&output)
            .map_err(|_| CliError::Usage("sortie profile JSON indisponible"));
    }
    match output {
        ProfileOutput::Proposed {
            approval_id,
            command_id,
            expires_at,
            screen,
        } => Ok(format!(
            "profil={} type={} modèle={} effort={} commande={} args={} approval_id={} command_id={} expire_at={}",
            screen.display_name,
            screen.agent_type,
            screen.model,
            screen.effort,
            screen.command,
            screen.args.join(" "),
            approval_id,
            command_id,
            expires_at,
        )),
        ProfileOutput::Approved {
            approval_id,
            command_id,
            actor,
            screen,
        } => Ok(format!(
            "profil={} commande={} args={} approval_id={} command_id={} actor={}",
            screen.display_name,
            screen.command,
            screen.args.join(" "),
            approval_id,
            command_id,
            actor,
        )),
    }
}

/// Fait d'annuaire Bridget, séparé de l'activité ACP. Il est daté de la
/// consultation et ne prétend pas décrire une activité de l'agent.
#[derive(Serialize)]
struct AvailabilityOutput {
    agent: String,
    state: String,
    transport: String,
    observed_at: i64,
    source: &'static str,
}

impl AvailabilityOutput {
    fn from_agent(agent: &AgentInfo, observed_at: i64) -> Self {
        Self {
            agent: agent.name.clone(),
            state: agent.state.clone(),
            transport: agent.transport.clone(),
            observed_at,
            source: "bridget",
        }
    }
}

/// Le contrat public disponible ne publie pas le statut de demande corrélé.
/// Ce champ est donc un inconnu explicite et non une absence silencieuse.
#[derive(Serialize)]
struct TransportSnapshotOutput {
    state: &'static str,
    reason: &'static str,
}

impl TransportSnapshotOutput {
    fn unknown() -> Self {
        Self {
            state: "unknown",
            reason: "request_status_public_unavailable",
        }
    }
}

#[derive(Serialize)]
struct FreshnessOutput {
    state: EtatFlux,
    observed_at: Option<i64>,
    reason: Option<String>,
}

#[derive(Serialize)]
struct RuntimeAgentOutput {
    agent: String,
    source: &'static str,
    subscription_id: Option<String>,
    stream_state: EtatFlux,
    captured_at: Option<i64>,
    reason: Option<String>,
    observations: Vec<RuntimeObservationOutput>,
}

impl RuntimeAgentOutput {
    fn unknown(agent: &str, reason: &str) -> Self {
        Self {
            agent: agent.to_string(),
            source: "acp_subscription",
            subscription_id: None,
            stream_state: EtatFlux::Unavailable,
            captured_at: None,
            reason: Some(reason.to_string()),
            observations: Vec::new(),
        }
    }
}

#[derive(Serialize)]
struct RuntimeObservationOutput {
    agent: String,
    source: SourceSnapshot,
    nature: &'static str,
    observed_at: String,
    session_id: String,
    message_id: Option<String>,
    subscription_id: String,
    seq: u64,
    proof_ref: String,
    stream_state: EtatFlux,
    details: serde_json::Value,
}

impl From<RuntimeObservation> for RuntimeObservationOutput {
    fn from(observation: RuntimeObservation) -> Self {
        Self {
            agent: observation.agent,
            source: observation.source,
            nature: runtime_nature_name(observation.nature),
            observed_at: observation.observed_at,
            session_id: observation.session_id,
            message_id: observation.message_id,
            subscription_id: observation.subscription_id,
            seq: observation.seq,
            proof_ref: observation.proof_ref,
            stream_state: observation.stream_state,
            details: observation.details,
        }
    }
}

fn runtime_nature_name(nature: RuntimeNature) -> &'static str {
    match nature {
        RuntimeNature::Disponibilite => "disponibilite",
        RuntimeNature::Tour => "tour",
        RuntimeNature::Outil => "outil",
        RuntimeNature::Idle => "idle",
        RuntimeNature::PermissionAutoDecidee => "permission_auto_decidee",
    }
}

struct StatusSourcesOutput {
    availability: Vec<AvailabilityOutput>,
    availability_state: EtatFlux,
    availability_reason: Option<String>,
    runtime: Vec<RuntimeAgentOutput>,
    freshness: FreshnessOutput,
    stream_state: EtatFlux,
}

impl StatusSourcesOutput {
    fn unknown(reason: &str) -> Self {
        Self {
            availability: Vec::new(),
            availability_state: EtatFlux::Unavailable,
            availability_reason: Some(reason.to_string()),
            runtime: Vec::new(),
            freshness: FreshnessOutput {
                state: EtatFlux::Unavailable,
                observed_at: None,
                reason: Some(reason.to_string()),
            },
            stream_state: EtatFlux::Unavailable,
        }
    }

    fn from_capture(
        availability: Vec<AvailabilityOutput>,
        runtime: Vec<RuntimeAgentOutput>,
        observed_at: i64,
    ) -> Self {
        let stream_state = aggregate_stream_state(&runtime);
        let reason = match stream_state {
            EtatFlux::Fresh => None,
            EtatFlux::Gap => Some("flux_incomplet".to_string()),
            EtatFlux::Ended => Some("abonnement_termine".to_string()),
            EtatFlux::Unavailable => Some("observation_incomplete".to_string()),
        };
        Self {
            availability,
            availability_state: EtatFlux::Fresh,
            availability_reason: None,
            runtime,
            freshness: FreshnessOutput {
                state: stream_state,
                observed_at: Some(observed_at),
                reason,
            },
            stream_state,
        }
    }
}

fn aggregate_stream_state(runtime: &[RuntimeAgentOutput]) -> EtatFlux {
    if runtime.is_empty()
        || runtime
            .iter()
            .any(|agent| agent.stream_state == EtatFlux::Unavailable)
    {
        return EtatFlux::Unavailable;
    }
    if runtime
        .iter()
        .any(|agent| agent.stream_state == EtatFlux::Gap)
    {
        return EtatFlux::Gap;
    }
    if runtime
        .iter()
        .any(|agent| agent.stream_state == EtatFlux::Ended)
    {
        return EtatFlux::Ended;
    }
    EtatFlux::Fresh
}

#[derive(Serialize)]
struct SnapshotOutput {
    objective: ObjectifCoordonne,
    delegations: Vec<Delegation>,
    decisions: Vec<DecisionCoordination>,
    /// Issue durable enregistrée par Maicie, distincte du snapshot transport.
    remises_locales: Vec<maicie::store::RemiseLocale>,
}

impl From<ObjectiveSnapshot> for SnapshotOutput {
    fn from(snapshot: ObjectiveSnapshot) -> Self {
        Self {
            objective: snapshot.objective,
            delegations: snapshot.delegations,
            decisions: snapshot.decisions,
            remises_locales: snapshot.remises_locales,
        }
    }
}

fn render_objective_output(output: ObjectiveOutput, json: bool) -> Result<String, CliError> {
    if json {
        return serde_json::to_string(&output)
            .map_err(|_| CliError::Objective(ObjectiveError::Invalid("sortie JSON indisponible")));
    }
    Ok(match output {
        ObjectiveOutput::Status {
            coordination,
            availability,
            availability_state,
            availability_reason,
            transport_snapshot,
            runtime,
            freshness,
            stream_state,
        } => {
            let auto_permissions = runtime
                .iter()
                .flat_map(|agent| agent.observations.iter())
                .filter(|observation| observation.nature == "permission_auto_decidee")
                .count();
            format!(
                "objectifs={} disponibilité={} état_disponibilité={} motif_disponibilité={} snapshot_transport={} runtime={} permissions_auto_décidées={} fraîcheur={} flux={}",
                coordination.len(),
                availability.len(),
                flux_name(availability_state),
                availability_reason.as_deref().unwrap_or("aucun"),
                transport_snapshot.state,
                runtime.len(),
                auto_permissions,
                flux_name(freshness.state),
                flux_name(stream_state),
            )
        }
        ObjectiveOutput::Decision { decision } => format!(
            "décision={} objectif={} état=applied",
            decision.id, decision.objectif_id
        ),
        ObjectiveOutput::Summary { coordination } => format!(
            "objectif={} délégations={} décisions={}",
            coordination.objective.id,
            coordination.delegations.len(),
            coordination.decisions.len()
        ),
    })
}

fn flux_name(state: EtatFlux) -> &'static str {
    match state {
        EtatFlux::Fresh => "fresh",
        EtatFlux::Gap => "gap",
        EtatFlux::Ended => "ended",
        EtatFlux::Unavailable => "unavailable",
    }
}

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    Configuration(ConfigError),
    Bridget(BridgetClientError),
    Delegate(DelegateError),
    Objective(ObjectiveError),
    Store(StoreError),
    Reconcile(ReconcileError),
    Profile(ProfileError),
    ProfileActivation(ProfileActivationError),
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => EXIT_USAGE,
            Self::Configuration(_) => EXIT_CONFIGURATION,
            Self::Bridget(_) => EXIT_BRIDGET,
            Self::Delegate(DelegateError::Store(_)) | Self::Store(_) => EXIT_STORE,
            Self::Reconcile(ReconcileError::Store(_)) => EXIT_STORE,
            Self::Objective(ObjectiveError::Store(_)) => EXIT_STORE,
            Self::ProfileActivation(ProfileActivationError::Store(_)) => EXIT_STORE,
            Self::Reconcile(_) => EXIT_BRIDGET,
            Self::Delegate(_) => EXIT_DELEGATE,
            Self::Objective(_) => EXIT_DELEGATE,
            Self::Profile(_) | Self::ProfileActivation(_) => EXIT_DELEGATE,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage",
            Self::Configuration(_) => "configuration",
            Self::Bridget(_) => "bridget",
            Self::Delegate(DelegateError::EnvelopeMismatch) => "envelope_mismatch",
            Self::Delegate(DelegateError::TargetUnavailable(_)) => "target_unavailable",
            Self::Delegate(DelegateError::Store(_)) | Self::Store(_) => "store",
            Self::Reconcile(ReconcileError::Store(_)) => "store",
            Self::Reconcile(_) => "bridget",
            Self::Delegate(DelegateError::Invalid(_)) => "delegate_invalid",
            Self::Objective(ObjectiveError::NotFound(_)) => "objective_not_found",
            Self::Objective(ObjectiveError::Store(_)) => "store",
            Self::Objective(ObjectiveError::Invalid(_)) => "objective_invalid",
            Self::Profile(_) => "profile_invalid",
            Self::ProfileActivation(ProfileActivationError::Store(_)) => "store",
            Self::ProfileActivation(_) => "profile_activation_invalid",
        }
    }

    fn as_json(&self) -> String {
        serde_json::json!({"error": {"code": self.code(), "message": self.to_string()}}).to_string()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(detail) => write!(formatter, "usage invalide : {detail}"),
            Self::Configuration(error) => error.fmt(formatter),
            Self::Bridget(error) => error.fmt(formatter),
            Self::Delegate(error) => error.fmt(formatter),
            Self::Objective(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
            Self::Reconcile(error) => error.fmt(formatter),
            Self::Profile(error) => error.fmt(formatter),
            Self::ProfileActivation(error) => error.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{candidates_from, parse_command, sanitize_terminal, Command, DelegateOutput};
    use maicie::bridget_client::AgentInfo;
    use maicie::config::{DurationClasses, MaicieConfig, ProfileConfig};
    use std::path::PathBuf;

    #[test]
    fn delegate_exige_les_options_structurantes() {
        assert!(parse_command(&["delegate".to_string()]).is_err());
        let command = parse_command(&[
            "delegate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--goal".to_string(),
            "audit".to_string(),
            "--json".to_string(),
        ])
        .unwrap();
        assert!(matches!(command, Command::Delegate(_)));
    }

    #[test]
    fn candidates_sont_json_deterministe() {
        let output = serde_json::to_string(&DelegateOutput::Candidates {
            candidates: vec!["a".to_string(), "b".to_string()],
        })
        .unwrap();
        assert_eq!(output, r#"{"kind":"candidates","candidates":["a","b"]}"#);
    }

    #[test]
    fn ecran_d_approbation_ne_restitue_aucun_caractere_de_controle() {
        assert_eq!(sanitize_terminal("nom\n\u{1b}[2J"), "nom\\n\\u{1b}[2J");
    }

    #[test]
    fn candidat_joint_le_nom_runtime_plutot_que_le_slug_du_profil() {
        let config = MaicieConfig {
            version: 1,
            bridget_socket: PathBuf::from("/tmp/bridget.sock"),
            database_path: PathBuf::from("/tmp/maicie.sqlite3"),
            durations: DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            status_capture_budget_ms: None,
            profiles: vec![ProfileConfig {
                id: "code-review".to_string(),
                agent_name: Some("coderBridget".to_string()),
                agent_type: None,
                model: None,
                effort: None,
                display_name: "Code review".to_string(),
                tags: vec!["review".to_string()],
                personality_ref: "profiles/reviewer.md".to_string(),
                tools: Vec::new(),
                spawn_order_ref: "agents/reviewer".to_string(),
            }],
        };
        let agents = vec![AgentInfo {
            name: "coderBridget".to_string(),
            agent_type: "codex".to_string(),
            connection_id: "conn-1".to_string(),
            host: "local".to_string(),
            transport: "acp".to_string(),
            os: "macos".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: None,
            effort: None,
        }];

        let candidates = candidates_from(&config, &agents);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name, "coderBridget");
        assert_eq!(candidates[0].tags, ["review"]);
    }
}
