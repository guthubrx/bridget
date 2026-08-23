//! Interface de ligne de commande du compagnon Maicie.
//!
//! La surface reste une projection mince des cas d'usage : elle charge une
//! configuration explicite, consulte l'annuaire public puis délègue la
//! décision durable à `app`. Après le commit, elle délègue l'émission au
//! réconciliateur d'outbox commun : aucun second chemin d'envoi n'existe.

use maicie::app::{
    add_participant, close, delegate, remove_participant, status, summarize, DelegateError,
    DelegateRequest, DelegateResult, DelegationCandidate, ObjectiveError,
};
use maicie::bridget_client::{AgentInfo, BridgetClient, BridgetClientError, BridgetClientLimits};
use maicie::config::{ConfigError, MaicieConfig};
use maicie::domain::{ClasseDuree, DecisionCoordination, Delegation, ObjectifCoordonne};
use maicie::reconcile::{reconcile_startup_with_limits, ReconcileError};
use maicie::store::{MaicieStore, ObjectiveSnapshot, StoreError};
use maicie::MAICIE_IDENTITY;
use serde::Serialize;
use std::env;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const EXIT_USAGE: u8 = 2;
const EXIT_CONFIGURATION: u8 = 3;
const EXIT_BRIDGET: u8 = 4;
const EXIT_DELEGATE: u8 = 5;
const EXIT_STORE: u8 = 6;

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
    }
}

fn run_status(arguments: StatusArgs) -> Result<String, CliError> {
    let store = open_store(&arguments.config)?;
    let snapshots = status(&store, arguments.objective_id).map_err(CliError::Objective)?;
    render_objective_output(
        ObjectiveOutput::Status {
            coordination: snapshots.into_iter().map(SnapshotOutput::from).collect(),
            transport_snapshot: "unknown",
            runtime: "unknown",
            freshness: "unknown",
            stream_state: "unavailable",
        },
        arguments.json,
    )
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

fn parse_command(arguments: &[String]) -> Result<Command, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("commande attendue : delegate"));
    };
    match verb.as_str() {
        "delegate" => parse_delegate(tail).map(Command::Delegate),
        "status" => parse_status(tail).map(Command::Status),
        "objective" => parse_objective(tail).map(Command::Objective),
        _ => Err(CliError::Usage("commande inconnue : delegate attendu")),
    }
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
    timeout_secs: u64,
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
                    timeout_secs: created.timeout_secs,
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
        transport_snapshot: &'static str,
        runtime: &'static str,
        freshness: &'static str,
        stream_state: &'static str,
    },
    Decision {
        decision: DecisionCoordination,
    },
    Summary {
        coordination: SnapshotOutput,
    },
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
        ObjectiveOutput::Status { coordination, .. } => {
            format!("objectifs={}", coordination.len())
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

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    Configuration(ConfigError),
    Bridget(BridgetClientError),
    Delegate(DelegateError),
    Objective(ObjectiveError),
    Store(StoreError),
    Reconcile(ReconcileError),
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
            Self::Reconcile(_) => EXIT_BRIDGET,
            Self::Delegate(_) => EXIT_DELEGATE,
            Self::Objective(_) => EXIT_DELEGATE,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{candidates_from, parse_command, Command, DelegateOutput};
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
            profiles: vec![ProfileConfig {
                id: "code-review".to_string(),
                agent_name: Some("coderBridget".to_string()),
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
