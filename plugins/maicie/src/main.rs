//! Interface de ligne de commande du compagnon Maicie.
//!
//! La surface reste une projection mince des cas d'usage : elle charge une
//! configuration explicite, consulte l'annuaire public puis délègue la
//! décision durable à `app`. Après le commit, elle délègue l'émission au
//! réconciliateur d'outbox commun : aucun second chemin d'envoi n'existe.

use bridget_transport::protocol::ReviewTarget;
use maicie::MAICIE_IDENTITY;
use maicie::app::{
    CatalogueReconcileError, DelegateError, DelegateRequest, DelegateResult, DelegationCandidate,
    LocalProfileApproval, ObjectiveError, ProfileActivationError, ProfileActivationProposalRequest,
    ProjectRegistrationError, ProjectRegistrationRequest, add_participant,
    approve_profile_activation, delegated_participants, prepare_project_registration,
    project_registration_request_bytes, propose_profile_activation, reconcile_catalogue_from_store,
    remove_participant, resolve_project_registration, status, stored_profile_activation_proposal,
    summarize,
};
use maicie::bridget_client::{
    AgentInfo, AttachWindow, BridgetClient, BridgetClientError, BridgetClientLimits,
    DaemonIdentity, ProjectRegistryClient,
};
use maicie::catalogue::{self, AppendOutcome, CatalogueError, CatalogueJournal};
use maicie::config::{ConfigError, MaicieConfig};
use maicie::domain::{
    ClasseDuree, CoutMissionAgent, DecisionCoordination, Delegation, EtatFlux, ObjectifCoordonne,
    ObjectiveOpeningPermit, SourceSnapshot, SuiteObjective,
};
use maicie::greffe_service::{
    GreffeServiceError, append_registre_add, apply_delegate, candidates_from,
    close_objective as close_greffe_objective,
};
use maicie::install_publish::{self, InstallPublishError};
use maicie::profiles::{
    ApprovalProfileView, ProfileError, ResolvedAgentDefinition, approval_view, load_profiles,
};
use maicie::reconcile::{
    CoordinationReconcileAction, CoordinationReconcileReport, ReconcileError,
    reconcile_activation_startup_at, reconcile_coordination_startup_with_limits,
    reconcile_guichet_startup_with_central_service, reconcile_notification_startup_with_limits,
    reconcile_startup_with_limits,
};
use maicie::review_continuity::{
    ReviewContinuityObservation, ReviewContinuityObserver, ReviewContinuityState,
};
use maicie::routines::{
    EtatRoutine, ProposeRoutineRequest, RoutineError, RoutineStatusRow, approve_routine,
    evaluate_routines, pause_routine, propose_routine, resume_routine, routines_status_rows,
};
use maicie::runtime::{RuntimeNature, RuntimeObservation, RuntimeSignal, RuntimeSubscription};
use maicie::store::{
    CompteursRefusDelegationLocale, MaicieStore, ObjectiveSnapshot, ProjectRegistrationState,
    ResourceRangeReservation, SchemaPreflight, StoreError,
};
use maicie::ui_projection::{UiProjectionError, publish_ui_mission_projection_v1};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, IsTerminal, Write};
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
const PROJECT_REGISTRATION_HORIZON_SECS: i64 = 300;
use maicie::{LOCALITY_GUARD_ISSUER_SCOPE, ROUTINES_ISSUER_SCOPE, STATUS_ISSUER_SCOPE};

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
    let (migrate, rest) = peel_migrate_flag(&arguments)?;
    let command = parse_command(&rest)?;
    let projection_config = mission_projection_config(&command);
    let result = match command {
        Command::Delegate(delegate_args) => run_delegate(delegate_args, migrate),
        Command::Status(status_args) => run_status(status_args, migrate),
        Command::Objective(objective_args) => run_objective(objective_args, migrate),
        Command::Profile(profile_args) => run_profile(profile_args, migrate),
        Command::Project(project_args) => run_project(project_args, migrate),
        Command::Registre(registre_args) => run_registre(registre_args, migrate),
        Command::Plage(plage_args) => run_plage(plage_args, migrate),
        Command::Routine(routine_args) => run_routine(routine_args, migrate),
        Command::Preflight(preflight_args) => {
            if migrate {
                return Err(CliError::Usage("preflight n'accepte pas --migrate"));
            }
            run_preflight(preflight_args)
        }
        Command::Migrate(migrate_args) => {
            if migrate {
                return Err(CliError::Usage(
                    "maicie migrate implique déjà le consentement ; retirez --migrate",
                ));
            }
            run_migrate(migrate_args)
        }
    };
    if result.is_ok()
        && let Some(config_path) = projection_config
    {
        publish_ui_mission_projection_v1(config_path).map_err(CliError::Projection)?;
    }
    result
}

/// Chaque commande Maicie qui a atteint une issue réussie rafraîchit le même
/// contrat public atomique. Cette publication n'ajoute aucune boucle résidente.
fn mission_projection_config(command: &Command) -> Option<PathBuf> {
    match command {
        Command::Delegate(arguments) => Some(arguments.config.clone()),
        Command::Status(arguments) => Some(arguments.config.clone()),
        Command::Objective(arguments) => Some(arguments.config.clone()),
        Command::Profile(arguments) => Some(arguments.config.clone()),
        Command::Project(arguments) => Some(arguments.config.clone()),
        Command::Registre(arguments) => Some(arguments.config.clone()),
        Command::Plage(arguments) => Some(arguments.config.clone()),
        Command::Routine(arguments) => Some(arguments.config.clone()),
        Command::Migrate(arguments) => Some(arguments.config.clone()),
        Command::Preflight(_) => None,
    }
}

/// Extrait le consentement explicite de migration (flag global `--migrate`).
/// Le flag peut apparaître avant ou après le verbe ; une duplication est un
/// usage invalide.
fn peel_migrate_flag(arguments: &[String]) -> Result<(bool, Vec<String>), CliError> {
    let mut migrate = false;
    let mut rest = Vec::with_capacity(arguments.len());
    for argument in arguments {
        if argument.as_str() == "--migrate" {
            if migrate {
                return Err(CliError::Usage("option --migrate dupliquée"));
            }
            migrate = true;
        } else {
            rest.push(argument.clone());
        }
    }
    Ok((migrate, rest))
}

fn run_status(arguments: StatusArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let ReconciledStore {
        store,
        coordination: coordination_report,
    } = open_store_with_reconciliation(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?;
    let snapshots = status(&store, arguments.objective_id).map_err(CliError::Objective)?;
    let review_continuity = capture_review_continuity(&config, &store, &snapshots)?;
    let refus_contraintes = store
        .local_delegate_refusal_counts()
        .map_err(CliError::Store)?;
    let sources = capture_status_sources(&config, &delegated_participants(&snapshots));
    render_objective_output(
        ObjectiveOutput::Status {
            coordination: snapshots.into_iter().map(SnapshotOutput::from).collect(),
            review_continuity,
            refus_contraintes,
            availability: sources.availability,
            availability_state: sources.availability_state,
            availability_reason: sources.availability_reason,
            transport_snapshot: TransportSnapshotOutput::unknown(),
            runtime: sources.runtime,
            freshness: sources.freshness,
            stream_state: sources.stream_state,
            coordination_freshness: CoordinationFreshnessOutput::from_report(
                &coordination_report,
                unix_now().ok(),
            ),
        },
        arguments.json,
    )
}

fn capture_review_continuity(
    config: &MaicieConfig,
    store: &MaicieStore,
    snapshots: &[ObjectiveSnapshot],
) -> Result<Vec<ReviewContinuityObservation>, CliError> {
    let repository_root = config
        .review_project
        .as_ref()
        .map(|project| project.repository_root.as_path());
    let verdicts = store.review_verdicts().map_err(CliError::Store)?;
    let verdicts_by_delegation = verdicts
        .iter()
        .map(|verdict| (verdict.delegation_id, verdict))
        .collect::<BTreeMap<_, _>>();
    let mut observer = ReviewContinuityObserver::new(repository_root);
    let mut observations = Vec::new();
    // Complexité : O((d + r) log r), où `d` est le nombre de délégations et
    // `r` le nombre de verdicts ; chaque cible Git distincte n'est lue qu'une fois.
    for snapshot in snapshots {
        for delegation in &snapshot.delegations {
            observations.push(observer.observe(
                delegation,
                verdicts_by_delegation.get(&delegation.id).copied(),
            ));
        }
    }
    Ok(observations)
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
        STATUS_ISSUER_SCOPE,
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
                return RuntimeAgentOutput::unknown(participant, &capture_runtime_reason(&error));
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
                return RuntimeAgentOutput::unknown(participant, &capture_runtime_reason(&error));
            }
        }
    }
}

fn capture_reason(error: &BridgetClientError) -> String {
    match error {
        BridgetClientError::Timeout { .. } => "budget_capture_epuise".to_string(),
        BridgetClientError::ClientRejected { .. }
        | BridgetClientError::VersionUnsupported { .. }
        | BridgetClientError::CapabilityMissing { .. } => "negociation_daemon_refusee".to_string(),
        BridgetClientError::Closed | BridgetClientError::ConnectionUnusable => {
            "liaison_bridget_fermee".to_string()
        }
        BridgetClientError::Decode { .. }
        | BridgetClientError::Protocol(_)
        | BridgetClientError::InvalidEnvelope(_)
        | BridgetClientError::Encode(_) => "reponse_bridget_illisible".to_string(),
        BridgetClientError::Connect { .. }
        | BridgetClientError::Read(_)
        | BridgetClientError::Write(_)
        | BridgetClientError::RemoteNack { .. }
        | BridgetClientError::FrameTooLarge { .. }
        | BridgetClientError::ItemLimitExceeded { .. }
        | BridgetClientError::InvalidLimits(_) => "annuaire_bridget_indisponible".to_string(),
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

fn run_objective(arguments: ObjectiveArgs, migrate: bool) -> Result<String, CliError> {
    let mut store = open_store(&arguments.config, migrate)?;
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
            let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
            let now = unix_now()?;
            let decision =
                close_greffe_objective(&mut store, &config, arguments.objective_id, &reason, now)
                    .map_err(greffe_service_error_for_cli)?;
            ObjectiveOutput::Decision { decision }
        }
    };
    render_objective_output(output, arguments.json)
}

fn open_store(config_path: &PathBuf, migrate: bool) -> Result<MaicieStore, CliError> {
    let config = MaicieConfig::load(config_path).map_err(CliError::Configuration)?;
    open_store_with_reconciliation(
        config_path,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )
    .map(|opened| opened.store)
}

struct ReconciledStore {
    store: MaicieStore,
    coordination: CoordinationReconcileReport,
}

/// Republication **avant** migration. Après une erreur englobante, la version
/// durable décide seule d'une éventuelle restauration : `Err` peut aussi
/// naître après le commit du schéma.
fn open_store_with_migrate_and_publish(
    config_path: &std::path::Path,
    database_path: &std::path::Path,
) -> Result<MaicieStore, CliError> {
    let publication = install_publish::republish_current_exe_before_migrate(database_path)
        .map_err(CliError::InstallPublish)?;
    let migrated = publication.open().map_err(|error| match error {
        install_publish::PublishedMigrationOpenError::Store(error) => CliError::Store(error),
        install_publish::PublishedMigrationOpenError::Install(error) => {
            CliError::InstallPublish(error)
        }
    })?;
    migrated
        .verify_preflight(config_path)
        .map_err(CliError::InstallPublish)
}

/// Unique frontière d'ouverture de la base Maicie configurée par le CLI.
/// L'attestation et l'ouverture consomment la même configuration : aucun
/// rechargement intermédiaire ne peut dissocier le daemon vérifié de la base.
fn open_guarded_maicie_store(
    config_path: &std::path::Path,
    config: &MaicieConfig,
    limits: BridgetClientLimits,
    migrate: bool,
) -> Result<MaicieStore, CliError> {
    require_local_daemon(config, limits)?;
    if migrate {
        open_store_with_migrate_and_publish(config_path, &config.database_path)
    } else {
        MaicieStore::open(&config.database_path).map_err(CliError::Store)
    }
}

/// Toute commande qui ouvre la base rejoue d'abord les outboxes pendantes dans
/// une fenêtre I/O bornée. L'indisponibilité Bridget laisse la ligne durable
/// pending ; les erreurs de contrat restent explicites au CLI.
fn open_store_with_reconciliation(
    config_path: &std::path::Path,
    config: &MaicieConfig,
    limits: BridgetClientLimits,
    migrate: bool,
) -> Result<ReconciledStore, CliError> {
    let mut store = open_guarded_maicie_store(config_path, config, limits, migrate)?;
    reconcile_pending(&mut store, config, limits)?;
    reconcile_activation_startup_at(&mut store, &config.bridget_socket, unix_now()?)
        .map_err(CliError::Reconcile)?;
    reconcile_guichet_startup_with_central_service(&mut store, config, unix_now()?, limits)
        .map_err(CliError::Reconcile)?;
    // Une relève peut créer une outbox de délégation. Elle emprunte aussitôt
    // le même lookup/replay durable que la commande locale, jamais une voie
    // d'envoi spéciale au guichet.
    reconcile_pending(&mut store, config, limits)?;
    // Une commande relève au plus un snapshot borné. Les terminaux du guichet
    // alimentent uniquement F29 ; les événements cursés n'ouvrent jamais F28.
    let coordination =
        reconcile_coordination_startup_with_limits(&mut store, &config.bridget_socket, limits)
            .map_err(CliError::Reconcile)?;
    // Les notifications naissent durablement du réducteur. Leur émission reste
    // le même chemin borné de reprise, jamais une seconde logique d'envoi CLI.
    reconcile_notification_startup_with_limits(&mut store, &config.bridget_socket, limits)
        .map_err(CliError::Reconcile)?;
    // Battement routines : même horloge que la relève (aucune timer Maicie).
    // Court-circuit si aucune active — zéro I/O Bridget, les fixtures CLI
    // mono-séquence et les commandes hors routines restent intactes.
    // Note : une routine `paused` n'est pas dans actives — le saut d'orphelins
    // au resume est couvert DANS resume_routine (pas par ce tick).
    let actives = store
        .list_routines(Some(EtatRoutine::Active))
        .map_err(CliError::Store)?;
    if !actives.is_empty() {
        let now = unix_now()?;
        let issuer_scope = store.issuer_scope().to_string();
        // Annuaire Bridget manquant : tick sans candidats (retente ensuite).
        // Motif explicite — pas unwrap_or_default anonyme (manche 4).
        let candidates =
            list_routine_candidates(config, limits).unwrap_or_else(|_error| Vec::new());
        // Erreurs de stockage remontent ; les blips delegate sont absorbés
        // DANS evaluate_routines (break sans avancer last_bucket).
        evaluate_routines(
            &mut store,
            &config.durations,
            &issuer_scope,
            &candidates,
            now,
        )
        .map_err(CliError::Routine)?;
    }
    Ok(ReconciledStore {
        store,
        coordination,
    })
}

/// Le registre Maicie appartient à la machine du daemon joint. La sonde est
/// donc faite avant l'ouverture SQLite : une identité absente ou distante ne
/// peut ni créer une base locale ni y rejouer une outbox.
fn require_local_daemon(
    config: &MaicieConfig,
    limits: BridgetClientLimits,
) -> Result<(), CliError> {
    let mut client = BridgetClient::connect_with_limits(
        &config.bridget_socket,
        LOCALITY_GUARD_ISSUER_SCOPE,
        limits,
    )
    .map_err(|error| daemon_identity_refusal(&error))?;
    let identity = client
        .daemon_identity()
        .map_err(|error| daemon_identity_refusal(&error))?;
    let local_host = bridget_core::local_host();

    if !bridget_core::host_is_attested(&local_host) {
        return Err(CliError::DaemonStoreLocality(
            "cette machine n'est pas attestée ; le registre Maicie suit le daemon joint. Exécutez la commande sur la machine qui héberge le daemon après avoir rétabli son nom de machine".to_string(),
        ));
    }
    if !bridget_core::host_is_attested(&identity.host) {
        return Err(CliError::DaemonStoreLocality(
            "la machine du daemon n'est pas attestée ; le registre Maicie suit le daemon joint. Exécutez la commande sur la machine qui héberge le daemon après avoir rétabli son nom de machine".to_string(),
        ));
    }
    if daemon_store_is_local(&identity, &local_host) {
        return Ok(());
    }

    Err(CliError::DaemonStoreLocality(format!(
        "le daemon joint s'exécute sur {} tandis que cette commande s'exécute sur {}; le registre Maicie suit le daemon joint. Exécutez la commande sur la machine qui héberge le daemon",
        sanitize_terminal(&identity.host),
        sanitize_terminal(&local_host),
    )))
}

fn daemon_store_is_local(identity: &DaemonIdentity, local_host: &str) -> bool {
    bridget_core::host_is_attested(&identity.host)
        && bridget_core::host_is_attested(local_host)
        && identity.host == local_host
}

fn daemon_identity_refusal(error: &BridgetClientError) -> CliError {
    CliError::DaemonStoreLocality(format!(
        "{} ; aucune écriture SQLite n'a été ouverte. Réessayez depuis la machine qui héberge le daemon après avoir rétabli son attestation",
        daemon_identity_failure_detail(error),
    ))
}

fn daemon_identity_failure_detail(error: &BridgetClientError) -> &'static str {
    match error {
        BridgetClientError::Closed | BridgetClientError::Timeout { .. } => {
            "rapport d'identité absent"
        }
        BridgetClientError::ClientRejected { .. } => "demande d'identité refusée par le daemon",
        BridgetClientError::Protocol(_) | BridgetClientError::Decode { .. } => {
            "rapport d'identité invalide"
        }
        _ => "rapport d'identité indisponible",
    }
}

fn list_routine_candidates(
    config: &MaicieConfig,
    limits: BridgetClientLimits,
) -> Result<Vec<DelegationCandidate>, CliError> {
    let client =
        BridgetClient::connect_with_limits(&config.bridget_socket, ROUTINES_ISSUER_SCOPE, limits)
            .map_err(CliError::Bridget)?;
    let agents = client.list_agents().map_err(CliError::Bridget)?;
    Ok(candidates_from(config, &agents))
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

fn run_project(arguments: ProjectArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = open_guarded_maicie_store(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?;
    let (mut record, canonical_request) = match arguments.action {
        ProjectAction::Register {
            display_name,
            requested_root,
            project_id,
            command_id,
        } => {
            let issued_at = unix_now()?;
            let prepared = prepare_project_registration(
                &mut store,
                &ProjectRegistrationRequest {
                    command_id: command_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    project_id: project_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
                    display_name,
                    requested_root,
                    issued_at,
                    deadline_at: issued_at + PROJECT_REGISTRATION_HORIZON_SECS,
                },
            )
            .map_err(CliError::ProjectRegistration)?;
            (prepared.record, prepared.canonical_request)
        }
        ProjectAction::Resume { command_id } => {
            let record = store
                .project_registration(&command_id)
                .map_err(CliError::Store)?
                .ok_or(CliError::Usage("commande project inconnue"))?;
            let canonical_request = project_registration_request_bytes(&store, &command_id)
                .map_err(CliError::ProjectRegistration)?
                .ok_or(CliError::Usage("outbox project absente"))?;
            (record, canonical_request)
        }
    };

    if record.outcome.is_none() {
        let mut client =
            ProjectRegistryClient::connect(&config.bridget_socket, store.issuer_scope())
                .map_err(CliError::Bridget)?;
        let outcome = client
            .bind_exact_bytes(&canonical_request)
            .map_err(CliError::Bridget)?;
        record = resolve_project_registration(&mut store, &outcome)
            .map_err(CliError::ProjectRegistration)?;
    }
    render_project_registration_output(&record, arguments.json)
}

#[derive(Serialize)]
struct ProjectRegistrationOutput<'a> {
    command_id: &'a str,
    project_id: &'a str,
    state: &'static str,
    resolved_project_id: Option<&'a str>,
    outcome: Option<&'a bridget_transport::protocol::ProjectBindOutcome>,
    next_action: &'static str,
}

fn render_project_registration_output(
    record: &maicie::store::ProjectRegistrationRecord,
    json: bool,
) -> Result<String, CliError> {
    let state = match record.state {
        ProjectRegistrationState::Prepared => "prepared",
        ProjectRegistrationState::Binding => "binding",
        ProjectRegistrationState::Bound => "bound",
        ProjectRegistrationState::Failed => "failed",
        ProjectRegistrationState::Expired => "expired",
    };
    let next_action = if record.outcome.is_some() {
        "none"
    } else {
        "project resume --command-id"
    };
    let output = ProjectRegistrationOutput {
        command_id: &record.command_id,
        project_id: &record.identity.project_id,
        state,
        resolved_project_id: record.resolved_project_id.as_deref(),
        outcome: record.outcome.as_ref(),
        next_action,
    };
    if json {
        serde_json::to_string(&output)
            .map_err(|_| CliError::Usage("sortie project JSON indisponible"))
    } else {
        Ok(format!(
            "project command_id={} project_id={} state={} resolved_project_id={} next_action={}",
            output.command_id,
            output.project_id,
            output.state,
            output.resolved_project_id.unwrap_or("-"),
            output.next_action,
        ))
    }
}

fn run_delegate(arguments: DelegateArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let limits = BridgetClientLimits::default();
    let mut store =
        open_store_with_reconciliation(&arguments.config, &config, limits, migrate)?.store;
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
        opening_permit: ObjectiveOpeningPermit::auto_generated(),
        explicit_target: arguments.target.as_deref(),
        required_tags: &arguments.required_tags,
        duration: arguments.duration,
        // Maicie est un client public durable, pas un wrapper enregistré :
        // demander une réponse Bridget serait refusé avant livraison. La
        // corrélation de réponse attend T015b/Subscribe, sans la simuler ici.
        reply: false,
        constat_id: arguments.constat_id.as_deref(),
        review_target: arguments.review_target.as_ref(),
        suite: arguments.suite.clone(),
        depends_on: &arguments.depends_on,
        references: &arguments.references,
        idempotency_key: &idempotency_key,
        now,
        retry_until,
        dedup_retained_until: retry_until,
        max_frame_bytes: client.limits().max_frame_bytes,
    };
    let result = apply_delegate(&mut store, &config, &candidates, &request).map_err(|error| {
        delegate_error_for_cli(error, &config.profiles, &agents, &arguments.config)
    })?;
    // La transaction `delegate` est déjà commitée ici. T008 effectue ensuite
    // lookup puis replay des octets persistés, sans reconstruire le message.
    reconcile_pending(&mut store, &config, limits)?;
    render_output(DelegateOutput::from(result), arguments.json)
        .map_err(|_| CliError::Delegate(DelegateError::Invalid("sortie JSON indisponible")))
}

/// Expose le consentement local US4 sans jamais lancer de processus. La
/// proposition ne fait qu'écrire l'approbation ; l'approbation ne produit que
/// l'outbox, ensuite reprise par le protocole public Bridget.
fn run_profile(arguments: ProfileArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = open_store_with_reconciliation(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?
    .store;
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
            let (screen, profile_hash) =
                approval_screen(&config, &profile_id, &arguments.definition)?;
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
            let proposal =
                stored_profile_activation_proposal(&store, approval_id, retry_until, retry_until)
                    .map_err(CliError::ProfileActivation)?;
            let (screen, profile_hash) = approval_screen(
                &config,
                &proposal.approval.profile_id,
                &arguments.definition,
            )?;
            let screen = ApprovalScreenOutput::from(screen);
            confirm_local_profile_approval(approval_id, &screen)?;
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
                    screen,
                },
                false,
            )
        }
    }
}

/// L'approbation n'est volontairement disponible que depuis un vrai terminal
/// local. Les options CLI sont donc insuffisantes à elles seules : l'humain
/// voit les champs neutralisés puis tape une confirmation explicite.
fn confirm_local_profile_approval(
    approval_id: Uuid,
    screen: &ApprovalScreenOutput,
) -> Result<(), CliError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(CliError::Usage(
            "approbation = terminal interactif uniquement",
        ));
    }
    let args = serde_json::to_string(&screen.args)
        .map_err(|_| CliError::Usage("arguments d'approbation illisibles"))?;
    let forbidden_env = serde_json::to_string(&screen.forbidden_env)
        .map_err(|_| CliError::Usage("environnement d'approbation illisible"))?;
    let mut output = io::stdout().lock();
    writeln!(output, "Approbation locale du profil :")
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    writeln!(output, "  profil={}", screen.display_name)
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    writeln!(
        output,
        "  type={} modèle={} effort={}",
        screen.agent_type, screen.model, screen.effort
    )
    .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    writeln!(output, "  command={}", screen.command)
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    writeln!(output, "  args={args}")
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    writeln!(output, "  forbidden_env={forbidden_env}")
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    write!(output, "Tapez oui pour approuver {approval_id} : ")
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    output
        .flush()
        .map_err(|_| CliError::Usage("écran d'approbation indisponible"))?;
    drop(output);

    let mut confirmation = String::new();
    io::stdin()
        .read_line(&mut confirmation)
        .map_err(|_| CliError::Usage("confirmation locale illisible"))?;
    if confirmation.trim() != "oui" {
        return Err(CliError::Usage("approbation locale refusée"));
    }
    Ok(())
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
    let definition_bytes =
        fs::read(definition_path).map_err(|_| CliError::Usage("définition résolue illisible"))?;
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
            let message = delegation
                .message_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "—".to_string());
            format!(
                "objectif={objective_id} délégation={} participant={} message_id={message} état={} replayed={replayed}",
                delegation.id, delegation.participant, delegation.coordination_state,
            )
        }
        DelegateOutput::Candidates { candidates } => format!("candidats={}", candidates.join(",")),
    })
}

/// Précise un refus de cible explicite avec les deux inscriptions distinctes.
/// Cette projection CLI ne change pas la décision métier : les replays restent
/// traités par `delegate` avant cette explication.
fn delegate_error_for_cli(
    error: DelegateError,
    profiles: &[maicie::config::ProfileConfig],
    agents: &[AgentInfo],
    config_path: &std::path::Path,
) -> CliError {
    let DelegateError::TargetUnavailable(target) = error else {
        return CliError::Delegate(error);
    };
    if target == MAICIE_IDENTITY {
        return CliError::TargetIsPilot(target);
    }
    let Some(agent) = agents.iter().find(|agent| agent.name == target) else {
        return CliError::TargetUnknownBridget(target);
    };
    let has_profile = profiles
        .iter()
        .any(|profile| profile.agent_name.as_deref().unwrap_or(&profile.id) == agent.name);
    if !has_profile {
        return CliError::TargetMissingMaicieProfile {
            target,
            config_path: config_path.to_path_buf(),
        };
    }
    match agent.state.as_str() {
        "dnd" => CliError::TargetUnavailableState {
            target,
            state: "dnd".to_string(),
        },
        "connected" => CliError::TargetEligibilityDivergence(target),
        state => CliError::TargetUnavailableState {
            target,
            state: state.to_string(),
        },
    }
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
    Project(ProjectArgs),
    Registre(RegistreArgs),
    Plage(PlageArgs),
    Routine(RoutineArgs),
    /// Gate sans écriture, destiné au chemin d'installation/activation.
    Preflight(PreflightArgs),
    /// Consentement explicite : applique les migrations de schéma.
    Migrate(MigrateArgs),
}

#[derive(Debug)]
struct PreflightArgs {
    config: PathBuf,
    json: bool,
}

#[derive(Debug)]
struct MigrateArgs {
    config: PathBuf,
}

#[derive(Debug)]
struct PlageArgs {
    config: PathBuf,
    action: PlageAction,
}

#[derive(Debug)]
enum PlageAction {
    Reserve {
        resource: String,
        objective_id: Uuid,
    },
    List,
}

#[derive(Debug)]
struct RegistreArgs {
    config: PathBuf,
    action: RegistreAction,
}

#[derive(Debug)]
enum RegistreAction {
    /// Vue humaine d'autorité : projection pure, aucune écriture.
    List {
        /// Déplie le corps des constats fermés (traités).
        fermes: bool,
        /// Déplie le corps des constats réfutés.
        refutes: bool,
        /// Affiche aussi les entrées encore en attente de qualification.
        attente: bool,
        /// Historique des fermetures erronées puis rectifiées.
        rectifies: bool,
    },
    /// Append d'une ligne JSON fermée `add` (idempotent aux octets identiques).
    Add { line: String },
    /// Migration d'un corpus prose intermédiaire vers `pending_qualification`.
    Migrer { depuis: PathBuf },
    /// Qualification humaine : append d'un `add` au texte verbatim du pending.
    Qualifier {
        pending_id: String,
        severity: catalogue::Severity,
        source_kind: catalogue::MissionSourceKind,
        source_id: String,
        source_failed: bool,
        date: String,
    },
    /// Transcription FR-1711 : sévérité dérivée si fait couvert, sinon pending.
    Consign {
        fait: String,
        source_id: String,
        date: String,
        text: String,
    },
    /// Traité : ce fut vrai, ce ne l'est plus.
    Fermer {
        constat_id: String,
        raison: catalogue::RaisonFermeture,
        reference: String,
        date: String,
    },
    /// Réfuté : ce ne fut jamais vrai.
    Refuter {
        constat_id: String,
        raison: catalogue::RaisonRefutation,
        reference: String,
        date: String,
    },
    /// Rectifie une transition erronée : delivered→open, historique conservé.
    Rectifier {
        constat_id: String,
        raison: catalogue::RaisonRectification,
        reference: String,
        date: String,
    },
    /// Requalifié : sévérité et/ou nature — reste ouvert.
    Requalifier {
        constat_id: String,
        from: Option<catalogue::Severity>,
        to: Option<catalogue::Severity>,
        nature_from: Option<catalogue::EntryNature>,
        nature_to: Option<catalogue::EntryNature>,
        raison: catalogue::RaisonRequalification,
        reference: Option<String>,
        date: String,
    },
}

#[derive(Debug)]
struct DelegateArgs {
    config: PathBuf,
    goal: String,
    target: Option<String>,
    required_tags: Vec<String>,
    duration: ClasseDuree,
    constat_id: Option<String>,
    review_target: Option<ReviewTarget>,
    suite: SuiteObjective,
    depends_on: Vec<Uuid>,
    references: Vec<Uuid>,
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
struct ProjectArgs {
    config: PathBuf,
    action: ProjectAction,
    json: bool,
}

#[derive(Debug)]
enum ProjectAction {
    Register {
        display_name: String,
        requested_root: String,
        project_id: Option<String>,
        command_id: Option<String>,
    },
    Resume {
        command_id: String,
    },
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
        "project" => parse_project(tail).map(Command::Project),
        "registre" => parse_registre(tail).map(Command::Registre),
        "plage" => parse_plage(tail).map(Command::Plage),
        "routine" => parse_routine(tail).map(Command::Routine),
        "preflight" => parse_preflight(tail).map(Command::Preflight),
        "migrate" => parse_migrate(tail).map(Command::Migrate),
        _ => Err(CliError::Usage(
            "commande inconnue : delegate, status, objective, profile, project, registre, plage, routine, preflight ou migrate",
        )),
    }
}

fn parse_project(arguments: &[String]) -> Result<ProjectArgs, CliError> {
    let Some((action, tail)) = arguments.split_first() else {
        return Err(CliError::Usage(
            "action project obligatoire : register ou resume",
        ));
    };
    let mut config = None;
    let mut display_name = None;
    let mut requested_root = None;
    let mut project_id = None;
    let mut command_id = None;
    let mut json = false;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
            "--name" => set_once_string(
                &mut display_name,
                next_value(tail, &mut index, "--name")?,
                "name",
            )?,
            "--root" => set_once_string(
                &mut requested_root,
                next_value(tail, &mut index, "--root")?,
                "root",
            )?,
            "--project-id" => set_once_string(
                &mut project_id,
                next_value(tail, &mut index, "--project-id")?,
                "project-id",
            )?,
            "--command-id" => set_once_string(
                &mut command_id,
                next_value(tail, &mut index, "--command-id")?,
                "command-id",
            )?,
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option project inconnue")),
        }
        index += 1;
    }
    let config = config.ok_or(CliError::Usage("--config est obligatoire"))?;
    let action = match action.as_str() {
        "register" => ProjectAction::Register {
            display_name: display_name.ok_or(CliError::Usage("--name est obligatoire"))?,
            requested_root: requested_root.ok_or(CliError::Usage("--root est obligatoire"))?,
            project_id,
            command_id,
        },
        "resume" => {
            if display_name.is_some() || requested_root.is_some() || project_id.is_some() {
                return Err(CliError::Usage(
                    "project resume n'accepte ni --name, ni --root, ni --project-id",
                ));
            }
            ProjectAction::Resume {
                command_id: command_id
                    .ok_or(CliError::Usage("project resume exige --command-id"))?,
            }
        }
        _ => {
            return Err(CliError::Usage(
                "action project inconnue : register ou resume",
            ));
        }
    };
    Ok(ProjectArgs {
        config,
        action,
        json,
    })
}

fn parse_preflight(arguments: &[String]) -> Result<PreflightArgs, CliError> {
    let mut config = None;
    let mut json = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                set_once_path(&mut config, next_value(arguments, &mut index, "--config")?)?
            }
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            _ => return Err(CliError::Usage("option preflight inconnue")),
        }
        index += 1;
    }
    Ok(PreflightArgs {
        config: config.ok_or(CliError::Usage("--config est obligatoire"))?,
        json,
    })
}

#[derive(Serialize)]
struct SchemaPreflightOutput {
    kind: &'static str,
    state: &'static str,
    database_schema: Option<i64>,
    binary_schema: i64,
    write_schema_compatible: bool,
    bootstrap_required: bool,
}

impl From<SchemaPreflight> for SchemaPreflightOutput {
    fn from(report: SchemaPreflight) -> Self {
        Self {
            kind: "schema_preflight",
            state: if report.bootstrap_required {
                "bootstrap_ready"
            } else {
                "compatible"
            },
            database_schema: report.database_version,
            binary_schema: report.supported_version,
            write_schema_compatible: true,
            bootstrap_required: report.bootstrap_required,
        }
    }
}

fn run_preflight(arguments: PreflightArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let output = SchemaPreflightOutput::from(
        MaicieStore::schema_preflight(&config.database_path).map_err(CliError::Store)?,
    );
    if arguments.json {
        return serde_json::to_string(&output)
            .map_err(|_| CliError::Usage("sortie preflight JSON indisponible"));
    }
    Ok(format!(
        "préflight schéma={} base={} binaire={} schéma-écriture=compatible bootstrap={}",
        output.state,
        output
            .database_schema
            .map(|version| version.to_string())
            .unwrap_or_else(|| "absente".to_string()),
        output.binary_schema,
        if output.bootstrap_required {
            "oui"
        } else {
            "non"
        },
    ))
}

fn parse_migrate(arguments: &[String]) -> Result<MigrateArgs, CliError> {
    let mut config = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                set_once_path(&mut config, next_value(arguments, &mut index, "--config")?)?
            }
            _ => return Err(CliError::Usage("option migrate inconnue")),
        }
        index += 1;
    }
    Ok(MigrateArgs {
        config: config.ok_or(CliError::Usage("--config est obligatoire"))?,
    })
}

fn run_migrate(arguments: MigrateArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let store = open_guarded_maicie_store(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        true,
    )?;
    let version = store.schema_version().map_err(CliError::Store)?;
    Ok(format!("schéma migré vers {version}"))
}

#[derive(Debug)]
struct RoutineArgs {
    config: PathBuf,
    json: bool,
    action: RoutineAction,
}

#[derive(Debug)]
enum RoutineAction {
    Propose {
        goal: String,
        participant: String,
        period_secs: i64,
        suite: SuiteObjective,
        depends_on: Vec<Uuid>,
        references: Vec<Uuid>,
    },
    Approve {
        routine_id: Uuid,
    },
    List,
    Pause {
        routine_id: Uuid,
    },
    Resume {
        routine_id: Uuid,
    },
    Show {
        routine_id: Uuid,
    },
}

fn parse_routine(arguments: &[String]) -> Result<RoutineArgs, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage(
            "action routine obligatoire : propose|approve|list|pause|resume|show",
        ));
    };
    let mut config = None;
    let mut json = false;
    let mut goal = None;
    let mut participant = None;
    let mut period_secs = None;
    let mut suite = None;
    let mut depends_on = Vec::new();
    let mut references = Vec::new();
    let mut routine_id = None;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
            "--json" => {
                if json {
                    return Err(CliError::Usage("option --json dupliquée"));
                }
                json = true;
            }
            "--goal" => {
                set_once_string(&mut goal, next_value(tail, &mut index, "--goal")?, "goal")?
            }
            "--to" => set_once_string(
                &mut participant,
                next_value(tail, &mut index, "--to")?,
                "to",
            )?,
            "--period-secs" => {
                if period_secs.is_some() {
                    return Err(CliError::Usage("option --period-secs dupliquée"));
                }
                let raw = next_value(tail, &mut index, "--period-secs")?;
                period_secs = Some(
                    raw.parse::<i64>()
                        .map_err(|_| CliError::Usage("--period-secs entier attendu"))?,
                );
            }
            "--suite" => {
                if suite.is_some() {
                    return Err(CliError::Usage("option --suite dupliquée"));
                }
                suite = Some(parse_suite(next_value(tail, &mut index, "--suite")?)?);
            }
            "--depends-on" => {
                depends_on.push(parse_objective_id(next_value(
                    tail,
                    &mut index,
                    "--depends-on",
                )?)?);
            }
            "--reference" => {
                references.push(parse_objective_id(next_value(
                    tail,
                    &mut index,
                    "--reference",
                )?)?);
            }
            "--id" => {
                if routine_id.is_some() {
                    return Err(CliError::Usage("option --id dupliquée"));
                }
                routine_id = Some(parse_objective_id(next_value(tail, &mut index, "--id")?)?);
            }
            other if other.starts_with("--") => {
                return Err(CliError::Usage("option routine inconnue"));
            }
            _ => {
                return Err(CliError::Usage("option routine inconnue"));
            }
        }
        index += 1;
    }
    let config = config.ok_or(CliError::Usage("--config est obligatoire"))?;
    let action = match verb.as_str() {
        "propose" => {
            let suite = suite.ok_or(CliError::Usage(
                "--suite est obligatoire (--suite aucune|<objectif-id>)",
            ))?;
            RoutineAction::Propose {
                goal: goal.ok_or(CliError::Usage("--goal obligatoire"))?,
                participant: participant.ok_or(CliError::Usage("--to obligatoire"))?,
                period_secs: period_secs.ok_or(CliError::Usage("--period-secs obligatoire"))?,
                suite,
                depends_on,
                references,
            }
        }
        "approve" => RoutineAction::Approve {
            routine_id: routine_id.ok_or(CliError::Usage("--id obligatoire"))?,
        },
        "list" => RoutineAction::List,
        "pause" => RoutineAction::Pause {
            routine_id: routine_id.ok_or(CliError::Usage("--id obligatoire"))?,
        },
        "resume" => RoutineAction::Resume {
            routine_id: routine_id.ok_or(CliError::Usage("--id obligatoire"))?,
        },
        "show" => RoutineAction::Show {
            routine_id: routine_id.ok_or(CliError::Usage("--id obligatoire"))?,
        },
        _ => {
            return Err(CliError::Usage(
                "action routine inconnue : propose|approve|list|pause|resume|show",
            ));
        }
    };
    Ok(RoutineArgs {
        config,
        json,
        action,
    })
}

fn run_routine(arguments: RoutineArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = open_store_with_reconciliation(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?
    .store;
    let now = unix_now()?;
    match arguments.action {
        RoutineAction::Propose {
            goal,
            participant,
            period_secs,
            suite,
            depends_on,
            references,
        } => {
            let routine = propose_routine(
                &mut store,
                &ProposeRoutineRequest {
                    goal: &goal,
                    participant: &participant,
                    period_secs,
                    suite,
                    depends_on: &depends_on,
                    references: &references,
                    now,
                },
            )
            .map_err(CliError::Routine)?;
            render_routine_output(
                RoutineOutput::Proposed {
                    routine_id: routine.id,
                    template_hash_hex: hex_hash(&routine.template_hash),
                    period_secs: routine.period_secs,
                    participant: routine.participant,
                },
                arguments.json,
            )
        }
        RoutineAction::Approve { routine_id } => {
            let routine = store
                .load_routine(routine_id)
                .map_err(CliError::Store)?
                .ok_or(CliError::Routine(RoutineError::NotFound(routine_id)))?;
            // Hash recalculé + refus AVANT l'écran (vigilance piégée).
            let expected_hash = routine_approval_preflight(&routine)?;
            confirm_local_routine_approval(routine_id, &routine, &expected_hash)?;
            let approved = approve_routine(&mut store, routine_id, &expected_hash, now)
                .map_err(CliError::Routine)?;
            render_routine_output(
                RoutineOutput::Approved {
                    routine_id: approved.id,
                    state: "active",
                },
                arguments.json,
            )
        }
        RoutineAction::List => {
            let rows = routines_status_rows(&store).map_err(CliError::Routine)?;
            render_routine_output(
                RoutineOutput::List {
                    routines: rows.into_iter().map(RoutineStatusOutput::from).collect(),
                },
                arguments.json,
            )
        }
        RoutineAction::Pause { routine_id } => {
            let paused = pause_routine(&mut store, routine_id, now).map_err(CliError::Routine)?;
            render_routine_output(
                RoutineOutput::State {
                    routine_id: paused.id,
                    state: "paused",
                },
                arguments.json,
            )
        }
        RoutineAction::Resume { routine_id } => {
            let resumed = resume_routine(&mut store, routine_id, now).map_err(CliError::Routine)?;
            render_routine_output(
                RoutineOutput::State {
                    routine_id: resumed.id,
                    state: "active",
                },
                arguments.json,
            )
        }
        RoutineAction::Show { routine_id } => {
            let rows = routines_status_rows(&store).map_err(CliError::Routine)?;
            let row = rows
                .into_iter()
                .find(|row| row.routine_id == routine_id)
                .ok_or(CliError::Routine(RoutineError::NotFound(routine_id)))?;
            render_routine_output(
                RoutineOutput::Show {
                    routine: RoutineStatusOutput::from(row),
                },
                arguments.json,
            )
        }
    }
}

fn confirm_local_routine_approval(
    routine_id: Uuid,
    routine: &maicie::routines::Routine,
    expected_hash: &[u8],
) -> Result<(), CliError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(CliError::Usage(
            "approbation routine = terminal interactif uniquement",
        ));
    }
    print!(
        "{}",
        format_routine_approval_screen(routine_id, routine, expected_hash)
    );
    print!("Confirmer l'activation (oui) : ");
    io::stdout()
        .flush()
        .map_err(|_| CliError::Usage("stdout indisponible"))?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|_| CliError::Usage("stdin indisponible"))?;
    if line.trim() != "oui" {
        return Err(CliError::Usage("approbation routine refusée"));
    }
    Ok(())
}

/// Recalcule le hash scellé et refuse AVANT tout écran si le gabarit a divergé.
/// Extrait pour qu'un oracle puisse tuer le retrait de cette ligne (MUT-A).
fn routine_approval_preflight(routine: &maicie::routines::Routine) -> Result<Vec<u8>, CliError> {
    let expected_hash = maicie::routines::sealed_template_hash(routine);
    if expected_hash != routine.template_hash {
        return Err(CliError::Routine(RoutineError::Invalid("gabarit altéré")));
    }
    Ok(expected_hash)
}

/// Texte d'écran ADR 011 : les SIX champs scellés + les deux empreintes.
/// Testable sans TTY (véracité de l'interface, pas seulement la garde).
fn format_routine_approval_screen(
    routine_id: Uuid,
    routine: &maicie::routines::Routine,
    expected_hash: &[u8],
) -> String {
    let suite_label = match &routine.suite {
        maicie::domain::SuiteObjective::Aucune => "aucune".to_string(),
        maicie::domain::SuiteObjective::Objectif(id) => id.to_string(),
    };
    let depends = if routine.depends_on.is_empty() {
        "—".to_string()
    } else {
        routine
            .depends_on
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    let references = if routine.references.is_empty() {
        "—".to_string()
    } else {
        routine
            .references
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    format!(
        "Approbation locale de la routine\n\
           id={routine_id}\n\
           goal={goal}\n\
           participant={participant}\n\
           period_secs={period}\n\
           suite={suite}\n\
           depends_on={depends}\n\
           references={references}\n\
           hash_stocke={stocke}\n\
           hash_recalcule={recalc} (scellé sur goal,participant,period_secs,suite,depends_on,references — tous affichés ci-dessus)\n",
        goal = sanitize_terminal(&routine.goal),
        participant = routine.participant,
        period = routine.period_secs,
        suite = suite_label,
        depends = depends,
        references = references,
        stocke = hex_hash(&routine.template_hash),
        recalc = hex_hash(expected_hash),
    )
}

fn hex_hash(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RoutineOutput {
    Proposed {
        routine_id: Uuid,
        template_hash_hex: String,
        period_secs: i64,
        participant: String,
    },
    Approved {
        routine_id: Uuid,
        state: &'static str,
    },
    State {
        routine_id: Uuid,
        state: &'static str,
    },
    List {
        routines: Vec<RoutineStatusOutput>,
    },
    Show {
        routine: RoutineStatusOutput,
    },
}

#[derive(Serialize)]
struct RoutineStatusOutput {
    routine_id: Uuid,
    state: EtatRoutine,
    period_secs: i64,
    participant: String,
    last_bucket: Option<i64>,
    open_occurrence: Option<maicie::routines::RoutineOccurrence>,
    recent_sautee: Vec<maicie::routines::RoutineOccurrence>,
    recent_differee: Vec<maicie::routines::RoutineOccurrence>,
}

impl From<RoutineStatusRow> for RoutineStatusOutput {
    fn from(row: RoutineStatusRow) -> Self {
        Self {
            routine_id: row.routine_id,
            state: row.state,
            period_secs: row.period_secs,
            participant: row.participant,
            last_bucket: row.last_bucket,
            open_occurrence: row.open_occurrence,
            recent_sautee: row.recent_sautee,
            recent_differee: row.recent_differee,
        }
    }
}

fn render_routine_output(output: RoutineOutput, json: bool) -> Result<String, CliError> {
    if json {
        return serde_json::to_string(&output)
            .map_err(|_| CliError::Usage("sortie routine JSON indisponible"));
    }
    Ok(match output {
        RoutineOutput::Proposed {
            routine_id,
            template_hash_hex,
            period_secs,
            participant,
        } => format!(
            "routine={} état=proposed participant={} period_secs={} hash={}",
            routine_id, participant, period_secs, template_hash_hex
        ),
        RoutineOutput::Approved { routine_id, state }
        | RoutineOutput::State { routine_id, state } => {
            format!("routine={} état={}", routine_id, state)
        }
        RoutineOutput::List { routines } => format!(
            "routines={} sautee={} differee={}",
            routines.len(),
            routines
                .iter()
                .map(|row| row.recent_sautee.len())
                .sum::<usize>(),
            routines
                .iter()
                .map(|row| row.recent_differee.len())
                .sum::<usize>(),
        ),
        RoutineOutput::Show { routine } => format!(
            "routine={} état={:?} sautee={} differee={} ouverte={}",
            routine.routine_id,
            routine.state,
            routine.recent_sautee.len(),
            routine.recent_differee.len(),
            routine.open_occurrence.is_some(),
        ),
    })
}

fn parse_plage(arguments: &[String]) -> Result<PlageArgs, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage(
            "action plage obligatoire : reserve ou list",
        ));
    };
    let mut config = None;
    let mut resource = None;
    let mut objective_id = None;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
            "--ressource" => set_once_string(
                &mut resource,
                next_value(tail, &mut index, "--ressource")?,
                "ressource",
            )?,
            "--objective-id" => {
                let raw = next_value(tail, &mut index, "--objective-id")?;
                if objective_id.is_some() {
                    return Err(CliError::Usage("option --objective-id dupliquée"));
                }
                objective_id = Some(
                    Uuid::parse_str(raw).map_err(|_| CliError::Usage("--objective-id invalide"))?,
                );
            }
            _ => return Err(CliError::Usage("option plage inconnue")),
        }
        index += 1;
    }
    let config = config.ok_or(CliError::Usage("--config est obligatoire"))?;
    let action = match verb.as_str() {
        "reserve" => {
            let resource = resource.ok_or(CliError::Usage("--ressource est obligatoire"))?;
            if resource.is_empty() {
                return Err(CliError::Usage("--ressource ne peut pas être vide"));
            }
            let objective_id =
                objective_id.ok_or(CliError::Usage("--objective-id est obligatoire"))?;
            PlageAction::Reserve {
                resource,
                objective_id,
            }
        }
        "list" => {
            if resource.is_some() || objective_id.is_some() {
                return Err(CliError::Usage(
                    "list n'accepte ni --ressource ni --objective-id",
                ));
            }
            PlageAction::List
        }
        _ => {
            return Err(CliError::Usage("action plage inconnue : reserve ou list"));
        }
    };
    Ok(PlageArgs { config, action })
}

fn run_plage(arguments: PlageArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = open_guarded_maicie_store(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?;
    match arguments.action {
        PlageAction::Reserve {
            resource,
            objective_id,
        } => {
            let now = unix_now()?;
            let reserved = store
                .reserve_resource_range(&resource, objective_id, now)
                .map_err(CliError::Store)?;
            Ok(format!(
                "plage réservée ressource={} objectif={} reserved_at={}",
                reserved.resource, reserved.objective_id, reserved.reserved_at
            ))
        }
        PlageAction::List => {
            let rows = store.list_resource_ranges().map_err(CliError::Store)?;
            Ok(render_plage_list(&rows))
        }
    }
}

fn render_plage_list(rows: &[ResourceRangeReservation]) -> String {
    if rows.is_empty() {
        return "plages=0".to_string();
    }
    let mut lines = vec![format!("plages={}", rows.len())];
    for row in rows {
        lines.push(format!(
            "ressource={} objectif={} reserved_at={}",
            row.resource, row.objective_id, row.reserved_at
        ));
    }
    lines.join("\n")
}

fn parse_registre(arguments: &[String]) -> Result<RegistreArgs, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage(
            "action registre obligatoire : list, add, migrer, qualifier, consign, fermer, refuter, rectifier ou requalifier",
        ));
    };
    let mut config = None;
    let mut line = None;
    let mut depuis = None;
    let mut pending_id = None;
    let mut severity = None;
    let mut source_kind = None;
    let mut source_id = None;
    let mut date = None;
    let mut fermes = false;
    let mut refutes = false;
    let mut attente = false;
    let mut rectifies = false;
    let mut source_failed = false;
    let mut fait = None;
    let mut text = None;
    let mut constat_id = None;
    let mut raison = None;
    let mut reference = None;
    let mut de = None;
    let mut vers = None;
    let mut nature_de = None;
    let mut nature_vers = None;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
            "--line" => {
                set_once_string(&mut line, next_value(tail, &mut index, "--line")?, "line")?
            }
            "--depuis" => set_once_path(&mut depuis, next_value(tail, &mut index, "--depuis")?)?,
            "--pending" => set_once_string(
                &mut pending_id,
                next_value(tail, &mut index, "--pending")?,
                "pending",
            )?,
            "--severity" => set_once_string(
                &mut severity,
                next_value(tail, &mut index, "--severity")?,
                "severity",
            )?,
            "--source-kind" => set_once_string(
                &mut source_kind,
                next_value(tail, &mut index, "--source-kind")?,
                "source-kind",
            )?,
            "--source-id" => set_once_string(
                &mut source_id,
                next_value(tail, &mut index, "--source-id")?,
                "source-id",
            )?,
            "--date" => {
                set_once_string(&mut date, next_value(tail, &mut index, "--date")?, "date")?
            }
            "--fait" => {
                set_once_string(&mut fait, next_value(tail, &mut index, "--fait")?, "fait")?
            }
            "--text" => {
                set_once_string(&mut text, next_value(tail, &mut index, "--text")?, "text")?
            }
            "--constat" => set_once_string(
                &mut constat_id,
                next_value(tail, &mut index, "--constat")?,
                "constat",
            )?,
            "--raison" => set_once_string(
                &mut raison,
                next_value(tail, &mut index, "--raison")?,
                "raison",
            )?,
            "--ref" => set_once_string(
                &mut reference,
                next_value(tail, &mut index, "--ref")?,
                "ref",
            )?,
            "--de" => set_once_string(&mut de, next_value(tail, &mut index, "--de")?, "de")?,
            "--vers" => {
                set_once_string(&mut vers, next_value(tail, &mut index, "--vers")?, "vers")?
            }
            "--nature-de" => set_once_string(
                &mut nature_de,
                next_value(tail, &mut index, "--nature-de")?,
                "nature-de",
            )?,
            "--nature-vers" => set_once_string(
                &mut nature_vers,
                next_value(tail, &mut index, "--nature-vers")?,
                "nature-vers",
            )?,
            "--fermes" => {
                if fermes {
                    return Err(CliError::Usage("option --fermes dupliquée"));
                }
                fermes = true;
            }
            "--refutes" => {
                if refutes {
                    return Err(CliError::Usage("option --refutes dupliquée"));
                }
                refutes = true;
            }
            "--attente" => {
                if attente {
                    return Err(CliError::Usage("option --attente dupliquée"));
                }
                attente = true;
            }
            "--rectifies" => {
                if rectifies {
                    return Err(CliError::Usage("option --rectifies dupliquée"));
                }
                rectifies = true;
            }
            "--source-failed" => {
                if source_failed {
                    return Err(CliError::Usage("option --source-failed dupliquée"));
                }
                source_failed = true;
            }
            _ => return Err(CliError::Usage("option registre inconnue")),
        }
        index += 1;
    }
    let config = config.ok_or(CliError::Usage("--config est obligatoire"))?;
    let action = match verb.as_str() {
        "list" => {
            if line.is_some()
                || depuis.is_some()
                || pending_id.is_some()
                || fait.is_some()
                || text.is_some()
            {
                return Err(CliError::Usage(
                    "registre list n'accepte que --config, --fermes, --refutes, --rectifies et --attente",
                ));
            }
            RegistreAction::List {
                fermes,
                refutes,
                attente,
                rectifies,
            }
        }
        "add" => {
            if depuis.is_some()
                || attente
                || fermes
                || refutes
                || rectifies
                || pending_id.is_some()
                || fait.is_some()
            {
                return Err(CliError::Usage("options incompatibles avec registre add"));
            }
            RegistreAction::Add {
                line: line.ok_or(CliError::Usage("--line est obligatoire pour registre add"))?,
            }
        }
        "migrer" => {
            if line.is_some()
                || attente
                || fermes
                || refutes
                || rectifies
                || pending_id.is_some()
                || fait.is_some()
            {
                return Err(CliError::Usage(
                    "options incompatibles avec registre migrer",
                ));
            }
            RegistreAction::Migrer {
                depuis: depuis.ok_or(CliError::Usage(
                    "--depuis est obligatoire pour registre migrer",
                ))?,
            }
        }
        "qualifier" => {
            if line.is_some() || depuis.is_some() || attente || fait.is_some() {
                return Err(CliError::Usage(
                    "options incompatibles avec registre qualifier",
                ));
            }
            let severity = parse_severity(&severity.ok_or(CliError::Usage(
                "--severity est obligatoire pour registre qualifier",
            ))?)?;
            let source_kind = parse_source_kind(&source_kind.ok_or(CliError::Usage(
                "--source-kind est obligatoire pour registre qualifier",
            ))?)?;
            RegistreAction::Qualifier {
                pending_id: pending_id.ok_or(CliError::Usage(
                    "--pending est obligatoire pour registre qualifier",
                ))?,
                severity,
                source_kind,
                source_id: source_id.ok_or(CliError::Usage(
                    "--source-id est obligatoire pour registre qualifier",
                ))?,
                source_failed,
                date: date.ok_or(CliError::Usage(
                    "--date est obligatoire pour registre qualifier",
                ))?,
            }
        }
        "consign" => {
            if line.is_some()
                || depuis.is_some()
                || attente
                || pending_id.is_some()
                || severity.is_some()
                || source_kind.is_some()
                || source_failed
            {
                return Err(CliError::Usage(
                    "registre consign : pas de --severity (dérivée ou absente) ; options --fait --source-id --date --text",
                ));
            }
            RegistreAction::Consign {
                fait: fait.ok_or(CliError::Usage(
                    "--fait est obligatoire pour registre consign",
                ))?,
                source_id: source_id.ok_or(CliError::Usage(
                    "--source-id est obligatoire pour registre consign",
                ))?,
                date: date.ok_or(CliError::Usage(
                    "--date est obligatoire pour registre consign",
                ))?,
                text: text.ok_or(CliError::Usage(
                    "--text est obligatoire pour registre consign",
                ))?,
            }
        }
        "fermer" => {
            let raison_raw = raison.ok_or(CliError::Usage(
                "--raison typée obligatoire pour registre fermer",
            ))?;
            let raison = catalogue::RaisonFermeture::parse(&raison_raw).ok_or(CliError::Usage(
                "raison fermer : corrige_en_production|corrige_par_lot|absorbe_par_autre_constat|objectif_clos",
            ))?;
            RegistreAction::Fermer {
                constat_id: constat_id.ok_or(CliError::Usage("--constat obligatoire"))?,
                raison,
                reference: reference.ok_or(CliError::Usage("--ref obligatoire"))?,
                date: date.ok_or(CliError::Usage("--date obligatoire"))?,
            }
        }
        "refuter" => {
            let raison_raw = raison.ok_or(CliError::Usage(
                "--raison typée obligatoire pour registre refuter",
            ))?;
            let raison = catalogue::RaisonRefutation::parse(&raison_raw).ok_or(CliError::Usage(
                "raison refuter : charge_fausse_mesuree|hors_perimetre|deja_couvert|erreur_de_lecture",
            ))?;
            RegistreAction::Refuter {
                constat_id: constat_id.ok_or(CliError::Usage("--constat obligatoire"))?,
                raison,
                reference: reference.ok_or(CliError::Usage("--ref obligatoire (mesure:N/M)"))?,
                date: date.ok_or(CliError::Usage("--date obligatoire"))?,
            }
        }
        "rectifier" => {
            let raison_raw = raison.ok_or(CliError::Usage(
                "--raison typée obligatoire pour registre rectifier",
            ))?;
            let raison =
                catalogue::RaisonRectification::parse(&raison_raw).ok_or(CliError::Usage(
                    "raison rectifier : fermeture_erronee|solde_mission_errone|refutation_erronee",
                ))?;
            RegistreAction::Rectifier {
                constat_id: constat_id.ok_or(CliError::Usage("--constat obligatoire"))?,
                raison,
                reference: reference.ok_or(CliError::Usage("--ref obligatoire"))?,
                date: date.ok_or(CliError::Usage("--date obligatoire"))?,
            }
        }
        "requalifier" => {
            let raison_raw = raison.ok_or(CliError::Usage(
                "--raison typée obligatoire pour registre requalifier",
            ))?;
            let raison =
                catalogue::RaisonRequalification::parse(&raison_raw).ok_or(CliError::Usage(
                    "raison requalifier : severite_ajustee|perimetre_affine|nature_reclassee",
                ))?;
            let from = match de {
                Some(raw) => Some(parse_severity(&raw)?),
                None => None,
            };
            let to = match vers {
                Some(raw) => Some(parse_severity(&raw)?),
                None => None,
            };
            if from.is_some() != to.is_some() {
                return Err(CliError::Usage(
                    "requalifier : --de et --vers ensemble ou absents",
                ));
            }
            let nature_from = match nature_de {
                Some(raw) => Some(parse_entry_nature(&raw)?),
                None => None,
            };
            let nature_to = match nature_vers {
                Some(raw) => Some(parse_entry_nature(&raw)?),
                None => None,
            };
            if nature_from.is_some() != nature_to.is_some() {
                return Err(CliError::Usage(
                    "requalifier : --nature-de et --nature-vers ensemble ou absents",
                ));
            }
            if from.is_none() && nature_from.is_none() {
                return Err(CliError::Usage(
                    "requalifier : fournir --de/--vers et/ou --nature-de/--nature-vers",
                ));
            }
            RegistreAction::Requalifier {
                constat_id: constat_id.ok_or(CliError::Usage("--constat obligatoire"))?,
                from,
                to,
                nature_from,
                nature_to,
                raison,
                reference,
                date: date.ok_or(CliError::Usage("--date obligatoire"))?,
            }
        }
        _ => {
            return Err(CliError::Usage(
                "action registre inconnue : list, add, migrer, qualifier, consign, fermer, refuter, rectifier ou requalifier",
            ));
        }
    };
    Ok(RegistreArgs { config, action })
}

fn parse_severity(value: &str) -> Result<catalogue::Severity, CliError> {
    match value {
        "blocker" => Ok(catalogue::Severity::Blocker),
        "major" => Ok(catalogue::Severity::Major),
        "minor" => Ok(catalogue::Severity::Minor),
        "info" => Ok(catalogue::Severity::Info),
        _ => Err(CliError::Usage("severity : blocker, major, minor ou info")),
    }
}

fn parse_entry_nature(value: &str) -> Result<catalogue::EntryNature, CliError> {
    catalogue::EntryNature::parse(value)
        .ok_or(CliError::Usage("nature : constat, regle ou resultat"))
}

fn parse_source_kind(value: &str) -> Result<catalogue::MissionSourceKind, CliError> {
    match value {
        "mission" => Ok(catalogue::MissionSourceKind::Mission),
        "incident" => Ok(catalogue::MissionSourceKind::Incident),
        "review" => Ok(catalogue::MissionSourceKind::Review),
        "gate" => Ok(catalogue::MissionSourceKind::Gate),
        _ => Err(CliError::Usage(
            "source-kind : mission, incident, review ou gate",
        )),
    }
}

fn run_registre(arguments: RegistreArgs, migrate: bool) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let store = open_guarded_maicie_store(
        &arguments.config,
        &config,
        BridgetClientLimits::default(),
        migrate,
    )?;
    if let RegistreAction::Add { line } = &arguments.action {
        let result =
            append_registre_add(&store, &config, line).map_err(greffe_service_error_for_cli)?;
        return Ok(match result.outcome {
            AppendOutcome::Appended => "registre add: appended".to_string(),
            AppendOutcome::IdempotentNoop => "registre add: idempotent_noop".to_string(),
        });
    }
    let catalogue_path = config.catalogue_path.ok_or(CliError::Usage(
        "catalogue_path absent de la configuration : registre exige un journal déclaré",
    ))?;
    let mut journal = CatalogueJournal::open(&catalogue_path).map_err(CliError::Catalogue)?;
    // T1710 : réconciliation idempotente au fil des commandes catalogue — jamais
    // en boucle résidente. Une clôture durable manquée est rattrapée ici.
    reconcile_catalogue_from_store(&store, &mut journal).map_err(CliError::CatalogueReconcile)?;
    match arguments.action {
        RegistreAction::List {
            fermes,
            refutes,
            attente,
            rectifies,
        } => {
            let parsed = journal.read_journal().map_err(CliError::Catalogue)?;
            if let Some(warning) = parsed.torn_tail_warning {
                eprintln!("avertissement: {warning}");
            }
            let view = catalogue::project_registre(&parsed.entries);
            let mut rendered = catalogue::render_registre_list_sections(
                &view,
                catalogue::RegistreListSections {
                    fermes,
                    refutes,
                    attente,
                    rectifies,
                },
            );
            let costs = store.all_mission_costs().map_err(CliError::Store)?;
            rendered.push_str(&render_mission_costs_section(&costs));
            Ok(rendered)
        }
        RegistreAction::Add { .. } => unreachable!("registre add traité par le service partagé"),
        RegistreAction::Migrer { depuis } => {
            let report = journal
                .migrate_prose_file(&depuis)
                .map_err(CliError::Catalogue)?;
            Ok(format!(
                "registre migrer: {} lues, {} appended, {} skipped",
                report.read, report.appended, report.skipped
            ))
        }
        RegistreAction::Qualifier {
            pending_id,
            severity,
            source_kind,
            source_id,
            source_failed,
            date,
        } => {
            let mission_source = catalogue::MissionSource {
                kind: source_kind,
                id: source_id,
                failed: if matches!(source_kind, catalogue::MissionSourceKind::Gate) {
                    if source_failed {
                        Some(true)
                    } else {
                        return Err(CliError::Usage("une source gate exige --source-failed"));
                    }
                } else if source_failed {
                    return Err(CliError::Usage(
                        "--source-failed n'est admis que pour source-kind=gate",
                    ));
                } else {
                    None
                },
            };
            let outcome = journal
                .qualify_pending(&pending_id, severity, mission_source, date)
                .map_err(CliError::Catalogue)?;
            Ok(match outcome {
                AppendOutcome::Appended => "registre qualifier: appended".to_string(),
                AppendOutcome::IdempotentNoop => "registre qualifier: idempotent_noop".to_string(),
            })
        }
        RegistreAction::Consign {
            fait,
            source_id,
            date,
            text,
        } => {
            let fact = catalogue::ObservedFact {
                kind: fait,
                source_id,
                date,
                text,
            };
            let (transcription, outcome) = journal
                .consign_observed_fact(&fact)
                .map_err(CliError::Catalogue)?;
            let kind = match &transcription {
                catalogue::TranscriptionOutcome::CoveredAdd(add) => {
                    let severity = match add.severity {
                        catalogue::Severity::Blocker => "blocker",
                        catalogue::Severity::Major => "major",
                        catalogue::Severity::Minor => "minor",
                        catalogue::Severity::Info => "info",
                    };
                    format!("add/{severity}")
                }
                catalogue::TranscriptionOutcome::Pending(_) => "pending_qualification".to_string(),
            };
            Ok(match outcome {
                AppendOutcome::Appended => format!("registre consign: appended ({kind})"),
                AppendOutcome::IdempotentNoop => {
                    format!("registre consign: idempotent_noop ({kind})")
                }
            })
        }
        RegistreAction::Fermer {
            constat_id,
            raison,
            reference,
            date,
        } => {
            maicie::preuve::ensure_reference_fermeture_at_write(&reference).map_err(|error| {
                CliError::Catalogue(CatalogueError::TransitionInvalide(error.to_string()))
            })?;
            let outcome = journal
                .close_constat_attested(&constat_id, raison, &reference, &date)
                .map_err(CliError::Catalogue)?;
            Ok(match outcome {
                AppendOutcome::Appended => "registre fermer: appended".into(),
                AppendOutcome::IdempotentNoop => "registre fermer: idempotent_noop".into(),
            })
        }
        RegistreAction::Refuter {
            constat_id,
            raison,
            reference,
            date,
        } => {
            maicie::preuve::ensure_reference_refutation_at_write(&reference).map_err(|error| {
                CliError::Catalogue(CatalogueError::TransitionInvalide(error.to_string()))
            })?;
            let outcome = journal
                .refute_constat_attested(&constat_id, raison, &reference, &date)
                .map_err(CliError::Catalogue)?;
            Ok(match outcome {
                AppendOutcome::Appended => "registre refuter: appended".into(),
                AppendOutcome::IdempotentNoop => "registre refuter: idempotent_noop".into(),
            })
        }
        RegistreAction::Rectifier {
            constat_id,
            raison,
            reference,
            date,
        } => {
            maicie::preuve::ensure_reference_fermeture_at_write(&reference).map_err(|error| {
                CliError::Catalogue(CatalogueError::TransitionInvalide(error.to_string()))
            })?;
            let outcome = journal
                .rectify_constat_attested(&constat_id, raison, &reference, &date)
                .map_err(CliError::Catalogue)?;
            Ok(match outcome {
                AppendOutcome::Appended => "registre rectifier: appended".into(),
                AppendOutcome::IdempotentNoop => "registre rectifier: idempotent_noop".into(),
            })
        }
        RegistreAction::Requalifier {
            constat_id,
            from,
            to,
            nature_from,
            nature_to,
            raison,
            reference,
            date,
        } => {
            if let Some(ref raw) = reference {
                maicie::preuve::ensure_reference_fermeture_at_write(raw).map_err(|error| {
                    CliError::Catalogue(CatalogueError::TransitionInvalide(error.to_string()))
                })?;
            }
            let severity = match (from, to) {
                (Some(a), Some(b)) => Some((a, b)),
                (None, None) => None,
                _ => {
                    return Err(CliError::Usage(
                        "requalifier : --de et --vers ensemble ou absents",
                    ));
                }
            };
            let nature = match (nature_from, nature_to) {
                (Some(a), Some(b)) => Some((a, b)),
                (None, None) => None,
                _ => {
                    return Err(CliError::Usage(
                        "requalifier : --nature-de et --nature-vers ensemble ou absents",
                    ));
                }
            };
            let outcome = journal
                .requalify_constat(
                    &constat_id,
                    severity,
                    nature,
                    raison,
                    reference.as_deref(),
                    &date,
                )
                .map_err(CliError::Catalogue)?;
            Ok(match outcome {
                AppendOutcome::Appended => "registre requalifier: appended".into(),
                AppendOutcome::IdempotentNoop => "registre requalifier: idempotent_noop".into(),
            })
        }
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
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
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
    let json = false;
    let mut index = 0;
    while index < tail.len() {
        match tail[index].as_str() {
            "--config" => set_once_path(&mut config, next_value(tail, &mut index, "--config")?)?,
            "--definition" => set_once_path(
                &mut definition,
                next_value(tail, &mut index, "--definition")?,
            )?,
            _ => return Err(CliError::Usage("option profile approve inconnue")),
        }
        index += 1;
    }
    Ok((
        ProfileAction::Approve { approval_id },
        config,
        definition,
        json,
    ))
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
    let mut constat_id = None;
    let mut review_ref = None;
    let mut expected_head = None;
    let mut suite = None;
    let mut depends_on = Vec::new();
    let mut references = Vec::new();
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
            "--constat-id" => set_once_string(
                &mut constat_id,
                next_value(arguments, &mut index, "--constat-id")?,
                "constat-id",
            )?,
            "--review-ref" => set_once_string(
                &mut review_ref,
                next_value(arguments, &mut index, "--review-ref")?,
                "review-ref",
            )?,
            "--expected-head" => set_once_string(
                &mut expected_head,
                next_value(arguments, &mut index, "--expected-head")?,
                "expected-head",
            )?,
            "--suite" => {
                if suite.is_some() {
                    return Err(CliError::Usage("option --suite dupliquée"));
                }
                suite = Some(parse_suite(next_value(arguments, &mut index, "--suite")?)?);
            }
            "--depends-on" => {
                depends_on.push(parse_objective_id(next_value(
                    arguments,
                    &mut index,
                    "--depends-on",
                )?)?);
            }
            "--reference" => {
                references.push(parse_objective_id(next_value(
                    arguments,
                    &mut index,
                    "--reference",
                )?)?);
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
    let suite = suite.ok_or(CliError::Usage(
        "--suite est obligatoire (--suite <objectif-id> ou --suite aucune)",
    ))?;
    let review_target = match (review_ref, expected_head) {
        (None, None) => None,
        (Some(target_ref), Some(expected_head)) => {
            let target = ReviewTarget {
                target_ref,
                expected_head,
            };
            if !target.is_valid() {
                return Err(CliError::Usage(
                    "cible de revue invalide : <remote>/<branche> et SHA de 40 hexadécimaux minuscules attendus",
                ));
            }
            Some(target)
        }
        _ => {
            return Err(CliError::Usage(
                "--review-ref et --expected-head doivent être fournis ensemble",
            ));
        }
    };
    Ok(DelegateArgs {
        config,
        goal,
        target,
        required_tags,
        duration,
        constat_id,
        review_target,
        suite,
        depends_on,
        references,
        idempotency_key,
        json,
    })
}

fn parse_suite(value: &str) -> Result<SuiteObjective, CliError> {
    if value == "aucune" {
        Ok(SuiteObjective::Aucune)
    } else {
        Ok(SuiteObjective::Objectif(parse_objective_id(value)?))
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    message_id: Option<Uuid>,
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
                    coordination_state: if created.waiting_on_prerequisites {
                        "en_attente_prerequis"
                    } else {
                        "prepared"
                    },
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
        review_continuity: Vec<ReviewContinuityObservation>,
        refus_contraintes: CompteursRefusDelegationLocale,
        availability: Vec<AvailabilityOutput>,
        availability_state: EtatFlux,
        availability_reason: Option<String>,
        transport_snapshot: TransportSnapshotOutput,
        runtime: Vec<RuntimeAgentOutput>,
        freshness: FreshnessOutput,
        stream_state: EtatFlux,
        coordination_freshness: CoordinationFreshnessOutput,
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

/// Fraîcheur de la relève 016 de cette invocation. Elle ne remplace aucun fait
/// local : `stale` et `unavailable` expliquent pourquoi la coordination n'a
/// pas appliqué d'effet automatique pendant cette passe.
#[derive(Serialize)]
struct CoordinationFreshnessOutput {
    state: &'static str,
    observed_at: Option<i64>,
    reason: Option<String>,
}

impl CoordinationFreshnessOutput {
    fn from_report(report: &CoordinationReconcileReport, observed_at: Option<i64>) -> Self {
        let action = report.actions.last();
        match action {
            Some(CoordinationReconcileAction::SnapshotAtteint { .. })
            | Some(CoordinationReconcileAction::EvenementApplique { .. })
            | Some(CoordinationReconcileAction::TerminalApplique { .. }) => Self {
                state: "fresh",
                observed_at,
                reason: None,
            },
            Some(CoordinationReconcileAction::Gap { reason, .. }) => Self {
                state: "stale",
                observed_at,
                reason: Some(reason.clone()),
            },
            Some(CoordinationReconcileAction::Unavailable { reason }) => Self {
                state: "unavailable",
                observed_at,
                reason: Some(reason.clone()),
            },
            Some(CoordinationReconcileAction::TransportIndisponible) => Self {
                state: "unavailable",
                observed_at: None,
                reason: Some("transport_indisponible".to_string()),
            },
            Some(CoordinationReconcileAction::BudgetEpuise) => Self {
                state: "unavailable",
                observed_at: None,
                reason: Some("budget_releve_epuise".to_string()),
            },
            None => Self {
                state: "unavailable",
                observed_at: None,
                reason: Some("aucune_observation_coordination".to_string()),
            },
        }
    }
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
        RuntimeNature::Tour => "tour",
        RuntimeNature::Outil => "outil",
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
    /// Coûts portés à la clôture (attestés ou inconnus). Vide si ouvert.
    costs: Vec<CoutMissionAgent>,
}

impl From<ObjectiveSnapshot> for SnapshotOutput {
    fn from(snapshot: ObjectiveSnapshot) -> Self {
        Self {
            objective: snapshot.objective,
            delegations: snapshot.delegations,
            decisions: snapshot.decisions,
            remises_locales: snapshot.remises_locales,
            costs: snapshot.costs,
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
            review_continuity,
            refus_contraintes,
            availability,
            availability_state,
            availability_reason,
            transport_snapshot,
            runtime,
            freshness,
            stream_state,
            coordination_freshness,
        } => {
            let auto_permissions = runtime
                .iter()
                .flat_map(|agent| agent.observations.iter())
                .filter(|observation| observation.nature == "permission_auto_decidee")
                .count();
            let count = |state| {
                review_continuity
                    .iter()
                    .filter(|observation| observation.state == state)
                    .count()
            };
            let mut rendered = format!(
                "objectifs={} refus_contradiction_suite={} disponibilité={} état_disponibilité={} motif_disponibilité={} snapshot_transport={} runtime={} permissions_auto_décidées={} fraîcheur={} flux={} coordination_fraîcheur={} coordination_motif={} revue_cible_absente={} revue_verdict_absent={} revue_ancetre={} revue_reecrite={} revue_inobservable={} coûts={}",
                coordination.len(),
                refus_contraintes.suite_aucune_avec_citation_non_classee,
                availability.len(),
                flux_name(availability_state),
                availability_reason.as_deref().unwrap_or("aucun"),
                transport_snapshot.state,
                runtime.len(),
                auto_permissions,
                flux_name(freshness.state),
                flux_name(stream_state),
                coordination_freshness.state,
                coordination_freshness.reason.as_deref().unwrap_or("aucun"),
                count(ReviewContinuityState::TargetAbsent),
                count(ReviewContinuityState::VerdictAbsent),
                count(ReviewContinuityState::StillAncestor),
                count(ReviewContinuityState::Rewritten),
                count(ReviewContinuityState::Unobservable),
                render_costs_summary(
                    &coordination
                        .iter()
                        .flat_map(|snapshot| snapshot.costs.iter())
                        .cloned()
                        .collect::<Vec<_>>(),
                ),
            );
            for observation in review_continuity.iter().filter(|observation| {
                matches!(
                    observation.state,
                    ReviewContinuityState::Rewritten | ReviewContinuityState::Unobservable
                )
            }) {
                rendered.push_str(&format!(
                    "\n{} delegation={} cible={} sha_jugé={} tête_observée={} motif={}",
                    if observation.state == ReviewContinuityState::Rewritten {
                        "ALERTE_VERDICT_REECRIT"
                    } else {
                        "VERDICT_INOBSERVABLE"
                    },
                    observation.delegation_id,
                    observation.target_ref.as_deref().unwrap_or("absente"),
                    observation.reviewed_head.as_deref().unwrap_or("absent"),
                    observation.observed_head.as_deref().unwrap_or("absente"),
                    observation
                        .reason
                        .map(|reason| reason.as_str())
                        .unwrap_or("aucun"),
                ));
            }
            rendered
        }
        ObjectiveOutput::Decision { decision } => format!(
            "décision={} objectif={} état=applied",
            decision.id, decision.objectif_id
        ),
        ObjectiveOutput::Summary { coordination } => format!(
            "objectif={} délégations={} décisions={} coûts={}",
            coordination.objective.id,
            coordination.delegations.len(),
            coordination.decisions.len(),
            render_costs_summary(&coordination.costs),
        ),
    })
}

fn render_costs_summary(costs: &[CoutMissionAgent]) -> String {
    if costs.is_empty() {
        return "aucun".to_string();
    }
    costs
        .iter()
        .map(|cost| {
            if cost.attested {
                format!(
                    "{}:facturable={} cache_read={} tours={}",
                    cost.agent,
                    cost.facturable_tokens.unwrap_or(0),
                    cost.cache_read_input_tokens.unwrap_or(0),
                    cost.turns.unwrap_or(0),
                )
            } else {
                format!("{}:inconnu", cost.agent)
            }
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn render_mission_costs_section(costs: &[CoutMissionAgent]) -> String {
    let mut out = String::from("--- coûts des missions closes ---\n");
    if costs.is_empty() {
        out.push_str("(aucun coût porté)\n");
        return out;
    }
    for cost in costs {
        if cost.attested {
            out.push_str(&format!(
                "agent={} attested=true tours={} facturable={} cache_read={} fenêtre={}-{}\n",
                cost.agent,
                cost.turns.unwrap_or(0),
                cost.facturable_tokens.unwrap_or(0),
                cost.cache_read_input_tokens.unwrap_or(0),
                cost.from_secs,
                cost.to_secs,
            ));
        } else {
            out.push_str(&format!(
                "agent={} attested=false coût=inconnu fenêtre={}-{}\n",
                cost.agent, cost.from_secs, cost.to_secs,
            ));
        }
    }
    out
}

fn flux_name(state: EtatFlux) -> &'static str {
    match state {
        EtatFlux::Fresh => "fresh",
        EtatFlux::Gap => "gap",
        EtatFlux::Ended => "ended",
        EtatFlux::Unavailable => "unavailable",
    }
}

fn greffe_service_error_for_cli(error: GreffeServiceError) -> CliError {
    match error {
        GreffeServiceError::CatalogueAbsent => CliError::Usage(
            "catalogue_path absent de la configuration : registre exige un journal déclaré",
        ),
        GreffeServiceError::Catalogue(error) => CliError::Catalogue(error),
        GreffeServiceError::CatalogueReconcile(error) => CliError::CatalogueReconcile(error),
        GreffeServiceError::Objective(error) => CliError::Objective(error),
        GreffeServiceError::Store(error) => CliError::Store(error),
        GreffeServiceError::Bridget(error) => CliError::Bridget(error),
        GreffeServiceError::Delegate(error) => CliError::Delegate(error),
        GreffeServiceError::Authorization(_) => CliError::Usage(
            bridget_transport::greffe_authorization::GREFFE_AUTHORIZATION_PUBLIC_REFUSAL,
        ),
        GreffeServiceError::Invalid(reason) => CliError::Usage(reason),
    }
}

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    Configuration(ConfigError),
    Catalogue(CatalogueError),
    CatalogueReconcile(CatalogueReconcileError),
    Bridget(BridgetClientError),
    DaemonStoreLocality(String),
    Delegate(DelegateError),
    TargetUnknownBridget(String),
    TargetMissingMaicieProfile {
        target: String,
        config_path: PathBuf,
    },
    TargetIsPilot(String),
    TargetUnavailableState {
        target: String,
        state: String,
    },
    TargetEligibilityDivergence(String),
    Objective(ObjectiveError),
    ProjectRegistration(ProjectRegistrationError),
    Store(StoreError),
    Reconcile(ReconcileError),
    Profile(ProfileError),
    ProfileActivation(ProfileActivationError),
    Routine(RoutineError),
    InstallPublish(InstallPublishError),
    Projection(UiProjectionError),
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => EXIT_USAGE,
            Self::Configuration(_) | Self::Catalogue(_) => EXIT_CONFIGURATION,
            Self::Bridget(_) | Self::DaemonStoreLocality(_) => EXIT_BRIDGET,
            Self::Delegate(DelegateError::Store(_))
            | Self::Store(_)
            | Self::CatalogueReconcile(CatalogueReconcileError::Store(_)) => EXIT_STORE,
            Self::Reconcile(ReconcileError::Store(_)) => EXIT_STORE,
            Self::Objective(ObjectiveError::Store(_)) => EXIT_STORE,
            Self::ProjectRegistration(ProjectRegistrationError::Store(_)) => EXIT_STORE,
            Self::ProfileActivation(ProfileActivationError::Store(_)) => EXIT_STORE,
            Self::Routine(RoutineError::Store(_)) => EXIT_STORE,
            Self::CatalogueReconcile(CatalogueReconcileError::Catalogue(_)) => EXIT_CONFIGURATION,
            Self::CatalogueReconcile(_) => EXIT_CONFIGURATION,
            Self::Reconcile(_) => EXIT_BRIDGET,
            Self::Delegate(_) => EXIT_DELEGATE,
            Self::TargetUnknownBridget(_)
            | Self::TargetMissingMaicieProfile { .. }
            | Self::TargetIsPilot(_)
            | Self::TargetUnavailableState { .. }
            | Self::TargetEligibilityDivergence(_) => EXIT_DELEGATE,
            Self::Objective(_) => EXIT_DELEGATE,
            Self::ProjectRegistration(_) => EXIT_DELEGATE,
            Self::Profile(_) | Self::ProfileActivation(_) | Self::Routine(_) => EXIT_DELEGATE,
            Self::InstallPublish(_) | Self::Projection(_) => EXIT_STORE,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Usage(_) => "usage",
            Self::Configuration(_) => "configuration",
            Self::Catalogue(_) => "catalogue",
            Self::CatalogueReconcile(CatalogueReconcileError::Store(_)) => "store",
            Self::CatalogueReconcile(CatalogueReconcileError::Catalogue(_)) => "catalogue",
            Self::CatalogueReconcile(_) => "catalogue_reconcile",
            Self::Bridget(_) => "bridget",
            Self::DaemonStoreLocality(_) => "daemon_store_not_local",
            Self::Delegate(DelegateError::EnvelopeMismatch) => "envelope_mismatch",
            Self::Delegate(DelegateError::ContrainteRefusee { .. }) => {
                "delegate_constraint_refused"
            }
            Self::Delegate(DelegateError::TargetUnavailable(_)) => "target_unavailable",
            Self::TargetIsPilot(_)
            | Self::TargetUnavailableState { .. }
            | Self::TargetEligibilityDivergence(_) => "target_unavailable",
            Self::TargetUnknownBridget(_) => "target_unknown_bridget",
            Self::TargetMissingMaicieProfile { .. } => "target_missing_maicie_profile",
            Self::Delegate(DelegateError::Store(_)) | Self::Store(_) => "store",
            Self::Reconcile(ReconcileError::Store(_)) => "store",
            Self::Reconcile(_) => "bridget",
            Self::Delegate(DelegateError::Invalid(_)) => "delegate_invalid",
            Self::Objective(ObjectiveError::NotFound(_)) => "objective_not_found",
            Self::Objective(ObjectiveError::Store(_)) => "store",
            Self::Objective(ObjectiveError::Invalid(_)) => "objective_invalid",
            Self::ProjectRegistration(ProjectRegistrationError::Store(_)) => "store",
            Self::ProjectRegistration(_) => "project_registration_invalid",
            Self::Profile(_) => "profile_invalid",
            Self::ProfileActivation(ProfileActivationError::Store(_)) => "store",
            Self::ProfileActivation(_) => "profile_activation_invalid",
            Self::Routine(RoutineError::Store(_)) => "store",
            Self::Routine(RoutineError::NotFound(_)) => "routine_not_found",
            Self::Routine(_) => "routine_invalid",
            Self::InstallPublish(_) => "install_publish",
            Self::Projection(_) => "mission_projection",
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
            Self::Catalogue(error) => error.fmt(formatter),
            Self::CatalogueReconcile(error) => error.fmt(formatter),
            Self::Bridget(error) => error.fmt(formatter),
            Self::Projection(error) => error.fmt(formatter),
            Self::DaemonStoreLocality(detail) => write!(formatter, "écriture refusée : {detail}"),
            Self::Delegate(error) => error.fmt(formatter),
            Self::TargetUnknownBridget(target) => write!(
                formatter,
                "agent inconnu de Bridget : {}; vérifiez son inscription et sa connexion",
                sanitize_terminal(target)
            ),
            Self::TargetMissingMaicieProfile {
                target,
                config_path,
            } => write!(
                formatter,
                "agent Bridget sans profil Maicie : {}; ajoutez un profil dans {} avec \"agent_name\": \"{}\"",
                sanitize_terminal(target),
                config_path.display(),
                sanitize_terminal(target)
            ),
            Self::TargetIsPilot(target) => write!(
                formatter,
                "cible indisponible : {} (condition : cible réservée au pilote)",
                sanitize_terminal(target)
            ),
            Self::TargetUnavailableState { target, state } if state == "busy" => write!(
                formatter,
                "cible indisponible : {} (condition : state=busy; tour en cours, réessayer après sa fin)",
                sanitize_terminal(target)
            ),
            Self::TargetUnavailableState { target, state } if state == "dnd" => write!(
                formatter,
                "cible indisponible : {} (condition : state=dnd; ne pas déranger)",
                sanitize_terminal(target)
            ),
            Self::TargetUnavailableState { target, state } => write!(
                formatter,
                "cible indisponible : {} (condition : state={})",
                sanitize_terminal(target),
                sanitize_terminal(state)
            ),
            Self::TargetEligibilityDivergence(target) => write!(
                formatter,
                "cible indisponible : {} (condition : divergence d'éligibilité; toutes les gardes observées sont satisfaites)",
                sanitize_terminal(target)
            ),
            Self::Objective(error) => error.fmt(formatter),
            Self::ProjectRegistration(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
            Self::Reconcile(error) => error.fmt(formatter),
            Self::Profile(error) => error.fmt(formatter),
            Self::ProfileActivation(error) => error.fmt(formatter),
            Self::Routine(error) => error.fmt(formatter),
            Self::InstallPublish(error) => error.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CliError, Command, DelegateError, DelegateOutput, MigrateArgs, PlageAction, PlageArgs,
        RegistreAction, RegistreArgs, SchemaPreflightOutput, candidates_from, capture_reason,
        daemon_identity_failure_detail, daemon_store_is_local, delegate_error_for_cli,
        format_routine_approval_screen, open_store_with_reconciliation, parse_command,
        peel_migrate_flag, routine_approval_preflight, run, run_migrate, run_plage, run_registre,
        sanitize_terminal,
    };
    use bridget_transport::protocol::ReviewTarget;
    use maicie::bridget_client::{AgentInfo, BridgetClientError, DaemonIdentity};
    use maicie::config::{DurationClasses, MaicieConfig, ProfileConfig};
    use maicie::domain::SuiteObjective;
    use maicie::routines::{EtatRoutine, Routine, sealed_template_hash};
    use maicie::store::{SCHEMA_VERSION, SchemaPreflight};
    use serde_json::json;
    use std::fs;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::{fs::PermissionsExt, net::UnixListener};
    use std::path::{Path, PathBuf};
    use std::thread;
    use uuid::Uuid;

    fn agent_info(name: &str, state: &str, domain: Option<&str>) -> AgentInfo {
        AgentInfo {
            name: name.to_string(),
            agent_type: "codex".to_string(),
            connection_id: format!("conn-{name}"),
            host: "fixture".to_string(),
            transport: "fixture".to_string(),
            os: "linux".to_string(),
            state: state.to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: domain.map(str::to_string),
            model: None,
            effort: None,
        }
    }

    fn profile_config(agent_name: &str) -> ProfileConfig {
        ProfileConfig {
            id: format!("profil-{agent_name}"),
            agent_name: Some(agent_name.to_string()),
            agent_type: None,
            model: None,
            effort: None,
            display_name: agent_name.to_string(),
            tags: vec!["review".to_string()],
            personality_ref: "profiles/reviewer.md".to_string(),
            tools: Vec::new(),
            spawn_order_ref: "agents/reviewer".to_string(),
        }
    }

    #[test]
    fn refus_avant_ecran_sur_gabarit_altere() {
        let id = Uuid::new_v4();
        let mut routine = Routine {
            id,
            goal: "ronde".into(),
            participant: "prospective".into(),
            period_secs: 60,
            suite: SuiteObjective::Aucune,
            depends_on: vec![],
            references: vec![],
            template_hash: vec![0x00],
            state: EtatRoutine::Proposed,
            proposed_at: 1,
            approved_at: None,
            paused_at: None,
            last_bucket: None,
        };
        routine.template_hash = sealed_template_hash(&routine);
        // Mutant du contenu sans retoucher le hash stocké → vigilance piégée
        // si l'écran s'affichait. Le préflight DOIT refuser avant.
        routine.goal = "autre goal".into();
        let err = routine_approval_preflight(&routine).expect_err("gabarit altéré");
        assert!(
            err.to_string().contains("gabarit altéré"),
            "refus pré-écran attendu, obtenu : {err}"
        );
        // Contrôle positif : gabarit intact → Ok (l'écran pourrait s'afficher).
        routine.goal = "ronde".into();
        routine.template_hash = sealed_template_hash(&routine);
        assert!(routine_approval_preflight(&routine).is_ok());
    }

    /// Oracle fédéré : une machine distante peut partager n'importe quel
    /// chemin local. Seul l'hôte attesté décide ; le contrôle positif local
    /// empêche une garde qui refuserait systématiquement.
    #[test]
    fn garde_daemon_exige_un_hote_atteste_localement() {
        let federated_same_path = DaemonIdentity {
            host: "machine-distante".to_string(),
            db_path: "/home/moi/.cache/bridget/bridget.db".to_string(),
        };
        assert!(
            !daemon_store_is_local(&federated_same_path, "machine-locale"),
            "hôtes distincts doivent rester fédérés, quel que soit le chemin du daemon"
        );

        let local_same_path = DaemonIdentity {
            host: "machine-locale".to_string(),
            db_path: "/home/moi/.cache/bridget/bridget.db".to_string(),
        };
        assert!(
            daemon_store_is_local(&local_same_path, "machine-locale"),
            "contrôle positif : un hôte attesté identique doit franchir la garde"
        );

        let unknown_host = DaemonIdentity {
            host: bridget_core::HOTE_NON_ATTESTE.to_string(),
            db_path: "/home/moi/.cache/bridget/bridget.db".to_string(),
        };
        assert!(
            !daemon_store_is_local(&unknown_host, "machine-locale"),
            "le repli HOTE_NON_ATTESTE n'atteste aucune machine"
        );
        assert!(
            !daemon_store_is_local(&local_same_path, bridget_core::HOTE_NON_ATTESTE),
            "le repli local n'atteste aucune machine non plus"
        );
    }

    /// Les trois échecs ne sont pas un succès local implicite. Ils ont tous
    /// le même verdict d'écriture (refus), mais leur détail reste observable
    /// pour distinguer un daemon muet, une matrice de rôle cassée et un fil
    /// incompatible.
    #[test]
    fn garde_daemon_n_aplatit_pas_les_echecs_d_attestation() {
        assert_eq!(
            daemon_identity_failure_detail(&BridgetClientError::Closed),
            "rapport d'identité absent"
        );
        assert_eq!(
            daemon_identity_failure_detail(&BridgetClientError::ClientRejected {
                reason: serde_json::json!("MessageOutsideClientRole"),
            }),
            "demande d'identité refusée par le daemon"
        );
        assert_eq!(
            daemon_identity_failure_detail(&BridgetClientError::Protocol(
                "DaemonIdentityReport invalide".to_string(),
            )),
            "rapport d'identité invalide"
        );
    }

    #[test]
    fn capture_reason_expose_le_refus_reel_du_daemon() {
        let error = BridgetClientError::ClientRejected {
            reason: serde_json::json!({"code": "invalid_issuer_scope"}),
        };
        assert_eq!(capture_reason(&error), "negociation_daemon_refusee");
        assert_ne!(capture_reason(&error), "annuaire_bridget_indisponible");
    }

    /// Le refus fédéré doit précéder l'ouverture SQLite. Ce témoin couvre le
    /// mutant qui déplacerait la garde après `open_maicie_store` : le message
    /// resterait un refus, mais la base locale aurait déjà été créée.
    #[test]
    fn garde_federee_refuse_avant_d_ouvrir_sqlite() {
        let root = std::env::temp_dir().join(format!("maicie-locality-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("répertoire temporaire");
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
            .expect("permissions du répertoire temporaire");
        let socket = root.join("bridget.sock");
        let database = root.join("maicie.sqlite3");
        let listener = UnixListener::bind(&socket).expect("socket daemon témoin");
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion client");
            let reader_stream = stream.try_clone().expect("clone lecteur");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();

            reader.read_line(&mut line).expect("role client");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&line).expect("role JSON"),
                json!({"type":"RoleHandshake","role":"client"})
            );
            writeln!(writer, "{}", json!({"type":"RoleAccepted","role":"client"}))
                .expect("RoleAccepted");
            writer.flush().expect("flush RoleAccepted");

            line.clear();
            reader.read_line(&mut line).expect("ClientHello");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&line)
                    .expect("ClientHello JSON")
                    .get("type"),
                Some(&json!("ClientHello"))
            );
            writeln!(
                writer,
                "{}",
                json!({
                    "type":"ClientWelcome",
                    "version":1,
                    "horizon_secs":60,
                    "issued_at_tolerance_secs":0,
                    "capabilities":["send_idempotent","lookup"]
                })
            )
            .expect("ClientWelcome");
            writer.flush().expect("flush ClientWelcome");

            line.clear();
            reader.read_line(&mut line).expect("demande identité");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&line).expect("identité JSON"),
                json!({"type":"DaemonIdentityRequest"})
            );
            writeln!(
                writer,
                "{}",
                json!({
                    "type":"DaemonIdentityReport",
                    "host":"machine-federee-temoin",
                    "db_path":"/var/lib/bridget/bridget.db"
                })
            )
            .expect("rapport identité");
            writer.flush().expect("flush rapport identité");
        });

        let config = MaicieConfig {
            version: 1,
            bridget_socket: socket,
            database_path: database.clone(),
            durations: DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            status_capture_budget_ms: None,
            catalogue_path: None,
            coordination_policies: None,
            review_project: None,
            profiles: Vec::new(),
        };
        let error = match open_store_with_reconciliation(
            &config.database_path,
            &config,
            maicie::bridget_client::BridgetClientLimits::default(),
            false,
        ) {
            Ok(_) => panic!("un daemon fédéré doit être refusé"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("écriture refusée"),
            "le refus doit être explicite, obtenu : {error}"
        );
        assert!(
            !database.exists(),
            "la base SQLite locale ne doit pas être créée avant le refus"
        );

        server.join().expect("daemon témoin");
        fs::remove_dir_all(root).expect("nettoyage témoin");
    }

    fn config_sans_daemon(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
        fs::create_dir_all(root).expect("répertoire temporaire");
        fs::set_permissions(root, fs::Permissions::from_mode(0o700))
            .expect("permissions du répertoire temporaire");
        let database = root.join("maicie.sqlite3");
        let catalogue = root.join("catalogue.jsonl");
        let config_path = root.join("config.json");
        let config = MaicieConfig {
            version: 1,
            bridget_socket: root.join("daemon-absent.sock"),
            database_path: database.clone(),
            durations: DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            status_capture_budget_ms: None,
            catalogue_path: Some(catalogue.clone()),
            coordination_policies: None,
            review_project: None,
            profiles: Vec::new(),
        };
        fs::write(
            &config_path,
            serde_json::to_vec_pretty(&config).expect("configuration JSON"),
        )
        .expect("écriture configuration");
        (config_path, database, catalogue)
    }

    /// Oracle de couverture runtime : les trois entrées qui contournaient le
    /// helper historique doivent toutes refuser avant SQLite ou le journal.
    #[test]
    fn garde_federee_couvre_chaque_entree_directe_du_store() {
        let mut failures = Vec::new();
        for (index, entry) in ["migrate", "plage-list", "registre-list"]
            .into_iter()
            .enumerate()
        {
            // Les sockets Unix sont bornées à 103 octets sur macOS. Garder la
            // fixture courte même si TMPDIR est déjà un chemin assez long.
            let root = std::env::temp_dir().join(format!("m54-{index}-{}", Uuid::new_v4()));
            let (config, database, catalogue) = config_sans_daemon(&root);
            let result = match entry {
                "migrate" => run_migrate(MigrateArgs { config }),
                "plage-list" => run_plage(
                    PlageArgs {
                        config,
                        action: PlageAction::List,
                    },
                    false,
                ),
                "registre-list" => run_registre(
                    RegistreArgs {
                        config,
                        action: RegistreAction::List {
                            fermes: false,
                            refutes: false,
                            attente: false,
                            rectifies: false,
                        },
                    },
                    false,
                ),
                _ => unreachable!("inventaire fermé"),
            };
            let refused = matches!(&result, Err(CliError::DaemonStoreLocality(_)));
            let database_created = database.exists();
            let catalogue_created = catalogue.exists();
            if !refused || database_created || catalogue_created {
                failures.push(format!(
                    "{entry}: result={result:?} sqlite={database_created} catalogue={catalogue_created}"
                ));
            }
            fs::remove_dir_all(root).expect("nettoyage témoin");
        }
        assert!(
            failures.is_empty(),
            "{} sur 3 entrées contournent la garde : {}",
            failures.len(),
            failures.join(" ; ")
        );
    }

    /// Oracle structurel : une nouvelle ouverture directe doit rendre le banc
    /// rouge, même si les trois entrées connues restent correctement gardées.
    #[test]
    fn inventaire_des_ouvertures_sqlite_reste_derriere_l_unique_helper_garde() {
        let source = include_str!("main.rs");
        let needle = ["MaicieStore", "::open"].concat();
        let lines = source.lines().collect::<Vec<_>>();
        let sites = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(&needle))
            .map(|(index, _)| {
                let declaration = lines[..=index]
                    .iter()
                    .rev()
                    .find_map(|line| line.trim_start().strip_prefix("fn "))
                    .expect("appel d'ouverture hors fonction");
                let function = declaration
                    .split_once('(')
                    .map(|(name, _)| name)
                    .unwrap_or(declaration)
                    .trim();
                format!("{function}:{}", index + 1)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sites
                .iter()
                .map(|site| site.split_once(':').unwrap().0)
                .collect::<Vec<_>>(),
            vec!["open_guarded_maicie_store"],
            "sites d'ouverture directe hors helper gardé : {sites:?}"
        );
    }

    #[test]
    fn ecran_approbation_routine_affiche_les_six_champs_scelles() {
        let id = Uuid::new_v4();
        let dep = Uuid::new_v4();
        let reference = Uuid::new_v4();
        let suite_obj = Uuid::new_v4();
        let hash = vec![0xab_u8, 0xcd];
        let routine = Routine {
            id,
            goal: "ronde".into(),
            participant: "prospective".into(),
            period_secs: 60,
            suite: SuiteObjective::Objectif(suite_obj),
            depends_on: vec![dep],
            references: vec![reference],
            template_hash: hash.clone(),
            state: EtatRoutine::Proposed,
            proposed_at: 1,
            approved_at: None,
            paused_at: None,
            last_bucket: None,
        };
        let screen = format_routine_approval_screen(id, &routine, &hash);
        for key in [
            "goal=",
            "participant=",
            "period_secs=",
            "suite=",
            "depends_on=",
            "references=",
        ] {
            assert!(screen.contains(key), "écran doit montrer {key}");
        }
        assert!(screen.contains(&dep.to_string()));
        assert!(screen.contains(&reference.to_string()));
        assert!(screen.contains(&suite_obj.to_string()));
        assert!(
            screen.contains(
                "scellé sur goal,participant,period_secs,suite,depends_on,references — tous affichés ci-dessus"
            ),
            "libellé hash ne doit pas mentir sur les champs sources"
        );
        assert!(
            !screen.contains("depuis les champs affichés"),
            "ancien libellé ambigu interdit"
        );
    }

    #[test]
    fn delegate_exige_les_options_structurantes() {
        assert!(parse_command(&["delegate".to_string()]).is_err());
        let sans_suite = parse_command(&[
            "delegate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--goal".to_string(),
            "audit".to_string(),
            "--json".to_string(),
        ]);
        assert!(
            sans_suite
                .err()
                .is_some_and(|error| error.to_string().contains("--suite")),
            "F36 : omission de --suite refusée"
        );
        let command = parse_command(&[
            "delegate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--goal".to_string(),
            "audit".to_string(),
            "--suite".to_string(),
            "aucune".to_string(),
            "--json".to_string(),
        ])
        .unwrap();
        assert!(matches!(command, Command::Delegate(_)));
    }

    #[test]
    fn delegate_exige_une_cible_de_revue_atomique_et_canonique() {
        let base = vec![
            "delegate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--goal".to_string(),
            "audit".to_string(),
            "--suite".to_string(),
            "aucune".to_string(),
        ];
        let mut partial = base.clone();
        partial.extend(["--review-ref".to_string(), "origin/fix/review".to_string()]);
        assert!(parse_command(&partial).is_err());

        let mut invalid = base.clone();
        invalid.extend([
            "--review-ref".to_string(),
            "origin/fix/review".to_string(),
            "--expected-head".to_string(),
            "A".repeat(40),
        ]);
        assert!(parse_command(&invalid).is_err());

        let mut valid = base;
        valid.extend([
            "--review-ref".to_string(),
            "origin/fix/review".to_string(),
            "--expected-head".to_string(),
            "a".repeat(40),
        ]);
        let Command::Delegate(arguments) = parse_command(&valid).unwrap() else {
            panic!("commande delegate attendue")
        };
        assert_eq!(
            arguments.review_target,
            Some(ReviewTarget {
                target_ref: "origin/fix/review".to_string(),
                expected_head: "a".repeat(40),
            })
        );
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
            catalogue_path: None,
            coordination_policies: None,
            review_project: None,
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

    #[test]
    fn cible_absente_de_bridget_est_distinguee_d_un_profil_absent() {
        let error = delegate_error_for_cli(
            DelegateError::TargetUnavailable("cursorbridget".to_string()),
            &[],
            &[],
            std::path::Path::new("/tmp/maicie.json"),
        );

        assert_eq!(error.code(), "target_unknown_bridget");
        assert_eq!(
            error.to_string(),
            "agent inconnu de Bridget : cursorbridget; vérifiez son inscription et sa connexion"
        );
    }

    #[test]
    fn agent_connecte_sans_profil_indique_le_champ_a_ajouter() {
        let agents = vec![AgentInfo {
            name: "cursorbridget".to_string(),
            agent_type: "cursor".to_string(),
            connection_id: "conn-cursor".to_string(),
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
        let error = delegate_error_for_cli(
            DelegateError::TargetUnavailable("cursorbridget".to_string()),
            &[],
            &agents,
            std::path::Path::new("/tmp/maicie.json"),
        );

        assert_eq!(error.code(), "target_missing_maicie_profile");
        assert_eq!(
            error.to_string(),
            "agent Bridget sans profil Maicie : cursorbridget; ajoutez un profil dans /tmp/maicie.json avec \"agent_name\": \"cursorbridget\""
        );
    }

    #[test]
    fn test_023_refus_busy_json_nomme_la_condition_reelle_et_le_reessai() {
        let agents = vec![agent_info("relecteur", "busy", Some("lot-temporaire"))];
        let profiles = vec![profile_config("relecteur")];

        let error = delegate_error_for_cli(
            DelegateError::TargetUnavailable("relecteur".to_string()),
            &profiles,
            &agents,
            std::path::Path::new("/tmp/maicie.json"),
        );
        let rendered: serde_json::Value =
            serde_json::from_str(&error.as_json()).expect("erreur JSON valide");

        assert_eq!(error.code(), "target_unavailable");
        assert_eq!(
            rendered
                .pointer("/error/message")
                .and_then(|value| value.as_str()),
            Some(
                "cible indisponible : relecteur (condition : state=busy; tour en cours, réessayer après sa fin)"
            )
        );
        assert!(
            !rendered
                .pointer("/error/message")
                .and_then(|value| value.as_str())
                .unwrap()
                .contains("domaine"),
            "le domaine observé n'est pas la condition exécutée"
        );
    }

    #[test]
    fn test_023_toutes_les_gardes_de_cible_nomment_leur_condition() {
        let dnd = agent_info("dormeur", "dnd", Some("bridget"));
        let stopped = agent_info("arrete", "stopped", Some("bridget"));
        let sans_profil = agent_info("sans-profil", "busy", Some("bridget"));
        let profiles = vec![profile_config("dormeur"), profile_config("arrete")];
        let agents = vec![dnd, stopped, sans_profil];

        let cases = [
            (
                "dormeur",
                "target_unavailable",
                "cible indisponible : dormeur (condition : state=dnd; ne pas déranger)",
            ),
            (
                "arrete",
                "target_unavailable",
                "cible indisponible : arrete (condition : state=stopped)",
            ),
            (
                "maicie",
                "target_unavailable",
                "cible indisponible : maicie (condition : cible réservée au pilote)",
            ),
            (
                "sans-profil",
                "target_missing_maicie_profile",
                "agent Bridget sans profil Maicie : sans-profil; ajoutez un profil dans /tmp/maicie.json avec \"agent_name\": \"sans-profil\"",
            ),
            (
                "absent",
                "target_unknown_bridget",
                "agent inconnu de Bridget : absent; vérifiez son inscription et sa connexion",
            ),
        ];
        for (target, expected_code, expected_message) in cases {
            let error = delegate_error_for_cli(
                DelegateError::TargetUnavailable(target.to_string()),
                &profiles,
                &agents,
                std::path::Path::new("/tmp/maicie.json"),
            );
            assert_eq!(error.code(), expected_code, "code de {target}");
            assert_eq!(error.to_string(), expected_message, "condition de {target}");
        }
    }

    #[test]
    fn test_023_domaine_non_decisionnel_et_refus_incoherent_nomme() {
        let profiles = vec![profile_config("hors-domaine")];
        let agents = vec![agent_info(
            "hors-domaine",
            "connected",
            Some("chantier-voisin"),
        )];
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
            catalogue_path: None,
            coordination_policies: None,
            review_project: None,
            profiles: profiles.clone(),
        };

        let candidates = candidates_from(&config, &agents);
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].available);
        assert!(!candidates[0].dnd);

        let impossible = delegate_error_for_cli(
            DelegateError::TargetUnavailable("hors-domaine".to_string()),
            &profiles,
            &agents,
            std::path::Path::new("/tmp/maicie.json"),
        );
        assert_eq!(
            impossible.to_string(),
            "cible indisponible : hors-domaine (condition : divergence d'éligibilité; toutes les gardes observées sont satisfaites)"
        );
    }

    #[test]
    fn registre_list_et_add_passent_par_la_commande_mince() {
        let list = parse_command(&[
            "registre".to_string(),
            "list".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
        ])
        .unwrap();
        assert!(matches!(
            list,
            Command::Registre(RegistreArgs {
                action: RegistreAction::List {
                    fermes: false,
                    refutes: false,
                    attente: false,
                    rectifies: false,
                },
                ..
            })
        ));
        let add = parse_command(&[
            "registre".to_string(),
            "add".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--line".to_string(),
            r#"{"v":1,"kind":"add","id":"c1","date":"2026-08-24T04:00:00+02:00","mission_source":{"kind":"incident","id":"i1"},"severity":"info","text":"x"}"#.to_string(),
        ])
        .unwrap();
        assert!(matches!(
            add,
            Command::Registre(RegistreArgs {
                action: RegistreAction::Add { .. },
                ..
            })
        ));
        assert!(parse_command(&["registre".to_string(), "list".to_string()]).is_err());
        let migrer = parse_command(&[
            "registre".to_string(),
            "migrer".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--depuis".to_string(),
            "/tmp/prose.jsonl".to_string(),
        ])
        .unwrap();
        assert!(matches!(
            migrer,
            Command::Registre(RegistreArgs {
                action: RegistreAction::Migrer { .. },
                ..
            })
        ));
        let attente = parse_command(&[
            "registre".to_string(),
            "list".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--attente".to_string(),
        ])
        .unwrap();
        assert!(matches!(
            attente,
            Command::Registre(RegistreArgs {
                action: RegistreAction::List {
                    fermes: false,
                    refutes: false,
                    attente: true,
                    rectifies: false,
                },
                ..
            })
        ));
        let qualifier = parse_command(&[
            "registre".to_string(),
            "qualifier".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--pending".to_string(),
            "pending:doc#1".to_string(),
            "--severity".to_string(),
            "major".to_string(),
            "--source-kind".to_string(),
            "incident".to_string(),
            "--source-id".to_string(),
            "i1".to_string(),
            "--date".to_string(),
            "2026-08-24T06:00:00+02:00".to_string(),
        ])
        .unwrap();
        assert!(matches!(
            qualifier,
            Command::Registre(RegistreArgs {
                action: RegistreAction::Qualifier { .. },
                ..
            })
        ));
    }

    #[test]
    fn parse_migrate_exige_config() {
        assert!(parse_command(&["migrate".to_string()]).is_err());
        let command = parse_command(&[
            "migrate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
        ])
        .unwrap();
        assert!(matches!(command, Command::Migrate(_)));
    }

    #[test]
    fn preflight_est_une_commande_sans_consentement_de_migration() {
        let command = parse_command(&[
            "preflight".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
            "--json".to_string(),
        ])
        .unwrap();
        assert!(matches!(command, Command::Preflight(_)));
        let error = run(vec![
            "preflight".to_string(),
            "--migrate".to_string(),
            "--config".to_string(),
            "/tmp/maicie.json".to_string(),
        ])
        .unwrap_err();
        assert!(error.to_string().contains("n'accepte pas --migrate"));
    }

    #[test]
    fn preflight_json_nomine_la_compatibilite_d_ecriture() {
        let output = SchemaPreflightOutput::from(SchemaPreflight {
            database_version: Some(SCHEMA_VERSION),
            supported_version: SCHEMA_VERSION,
            bootstrap_required: false,
        });
        assert_eq!(
            serde_json::to_string(&output).unwrap(),
            format!(
                "{{\"kind\":\"schema_preflight\",\"state\":\"compatible\",\"database_schema\":{SCHEMA_VERSION},\"binary_schema\":{SCHEMA_VERSION},\"write_schema_compatible\":true,\"bootstrap_required\":false}}"
            )
        );
    }

    #[test]
    fn peel_migrate_flag_accepte_une_seule_occurrence() {
        let (migrate, rest) =
            peel_migrate_flag(&["--migrate".to_string(), "status".to_string()]).unwrap();
        assert!(migrate);
        assert_eq!(rest, vec!["status".to_string()]);
        assert!(peel_migrate_flag(&["--migrate".to_string(), "--migrate".to_string()]).is_err());
    }

    #[test]
    fn toutes_les_portees_emetteur_atteignent_le_minimum_du_daemon() {
        // Mesure du 28/08 : les quatre portees literales de Maicie faisaient 12
        // a 21 caracteres alors que le daemon en exige 22 depuis le 22/08. Tout
        // ClientHello etait donc refuse en invalid_issuer_scope, observe a la
        // socket, et le refus remontait deguise en « annuaire indisponible ».
        // Aucune commande CLI ne pouvait negocier : la garde de localite n'a
        // jamais fonctionne depuis sa naissance.
        // Raccourcir une portee sous le minimum tue ce temoin.
        for portee in [
            maicie::LOCALITY_GUARD_ISSUER_SCOPE,
            maicie::STATUS_ISSUER_SCOPE,
            maicie::ROUTINES_ISSUER_SCOPE,
            maicie::USAGE_ISSUER_SCOPE,
        ] {
            assert!(
                portee.len() >= maicie::MIN_ISSUER_SCOPE_LEN,
                "portee « {portee} » : {} caracteres, le daemon en exige {} et refuse en invalid_issuer_scope",
                portee.len(),
                maicie::MIN_ISSUER_SCOPE_LEN
            );
            assert!(
                portee
                    .bytes()
                    .all(|octet: u8| octet.is_ascii_alphanumeric() || matches!(octet, b'-' | b'_')),
                "portee « {portee} » : caractere hors alphabet accepte par le daemon"
            );
        }
    }
}
