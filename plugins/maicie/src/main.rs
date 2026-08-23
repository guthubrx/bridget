//! Interface de ligne de commande du compagnon Maicie.
//!
//! La surface reste une projection mince des cas d'usage : elle charge une
//! configuration explicite, consulte l'annuaire public puis délègue la
//! décision durable à `app`. Elle n'envoie jamais elle-même une délégation.

use maicie::MAICIE_IDENTITY;
use maicie::app::{DelegateError, DelegateRequest, DelegateResult, DelegationCandidate, delegate};
use maicie::bridget_client::{AgentInfo, BridgetClient, BridgetClientError, BridgetClientLimits};
use maicie::config::{ConfigError, MaicieConfig};
use maicie::domain::ClasseDuree;
use maicie::store::{MaicieStore, StoreError};
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
    }
}

fn run_delegate(arguments: DelegateArgs) -> Result<String, CliError> {
    let config = MaicieConfig::load(&arguments.config).map_err(CliError::Configuration)?;
    let mut store = MaicieStore::open(&config.database_path).map_err(CliError::Store)?;
    let limits = BridgetClientLimits::default();
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
        reply: true,
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
            let agent = agents.iter().find(|agent| agent.name == profile.id)?;
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

fn parse_command(arguments: &[String]) -> Result<Command, CliError> {
    let Some((verb, tail)) = arguments.split_first() else {
        return Err(CliError::Usage("commande attendue : delegate"));
    };
    match verb.as_str() {
        "delegate" => parse_delegate(tail).map(Command::Delegate),
        _ => Err(CliError::Usage("commande inconnue : delegate attendu")),
    }
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

#[derive(Debug)]
enum CliError {
    Usage(&'static str),
    Configuration(ConfigError),
    Bridget(BridgetClientError),
    Delegate(DelegateError),
    Store(StoreError),
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => EXIT_USAGE,
            Self::Configuration(_) => EXIT_CONFIGURATION,
            Self::Bridget(_) => EXIT_BRIDGET,
            Self::Delegate(DelegateError::Store(_)) | Self::Store(_) => EXIT_STORE,
            Self::Delegate(_) => EXIT_DELEGATE,
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
            Self::Delegate(DelegateError::Invalid(_)) => "delegate_invalid",
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
            Self::Store(error) => error.fmt(formatter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, DelegateOutput, parse_command};

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
}
