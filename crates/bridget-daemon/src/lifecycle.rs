//! Frontière des ordres de cycle de vie et préparation contrôlée du spawn.
//!
//! Ce module ne lance aucun processus : T906 consomme `PreparedSpawn` avec le
//! bootstrap supervisé. Il concentre déjà les gardes et leur ordre afin que
//! chaque refus terminal passe par la même saga idempotente que le succès.

use crate::fleet::{
    FleetError, FleetSupervisor, SpawnLease, SpawnOrder, SpawnSubmission, SpawnWaiter,
};
use crate::idempotency::SpawnCommandIssue;
use crate::registry::{
    AgentDefinition, AgentRegistry, allow_api_key_value, forbidden_environment_variable,
};
use bridget_transport::SpawnRefusal;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const BASELINE_ENV: &[&str] = &["HOME", "PATH", "USER", "LANG", "TMPDIR"];
const FALLBACK_PATH: &str = "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

pub type SourceEnvironment = BTreeMap<String, OsString>;

#[derive(Debug, Clone)]
pub struct PreparedSpawn {
    pub lease: SpawnLease,
    pub agent_type: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: SourceEnvironment,
}

#[derive(Debug, Clone)]
pub enum SpawnDecision {
    Ready(PreparedSpawn),
    Await(SpawnWaiter),
    Accepted { name: String },
    Rejected(SpawnRefusal),
    EnvelopeMismatch,
}

pub fn source_environment() -> SourceEnvironment {
    std::env::vars_os()
        .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
        .collect()
}

/// Applique le lookup/rejeu idempotent avant toute garde mutable. Une
/// réservation neuve est ensuite soit préparée pour T906, soit terminée avec
/// l'un des onze motifs fermés du contrat.
pub fn submit_spawn(
    supervisor: &FleetSupervisor,
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    now: i64,
    recovering: bool,
) -> Result<SpawnDecision, FleetError> {
    if recovering && !supervisor.knows_command(&order.command_id) {
        return Ok(SpawnDecision::Rejected(SpawnRefusal::DaemonRecovering));
    }
    match supervisor.request_spawn(order, now)? {
        SpawnSubmission::Start(lease) => {
            let prepared = match prepare_spawn(registry, source, order, lease.clone()) {
                Ok(prepared) => prepared,
                Err(reason) => {
                    let (category, detail) = refusal_record(&reason);
                    supervisor.fail(&lease, category, detail)?;
                    return Ok(SpawnDecision::Rejected(reason));
                }
            };
            supervisor.mark_starting(&lease, now)?;
            Ok(SpawnDecision::Ready(prepared))
        }
        SpawnSubmission::Await(waiter) => Ok(SpawnDecision::Await(waiter)),
        SpawnSubmission::Terminal(issue) => Ok(decision_from_issue(issue, supervisor.quota())),
        SpawnSubmission::EnvelopeMismatch => Ok(SpawnDecision::EnvelopeMismatch),
        SpawnSubmission::IdempotencyExpired => {
            Ok(SpawnDecision::Rejected(SpawnRefusal::IdempotencyExpired))
        }
    }
}

fn prepare_spawn(
    registry: &AgentRegistry,
    source: &SourceEnvironment,
    order: &SpawnOrder,
    lease: SpawnLease,
) -> Result<PreparedSpawn, SpawnRefusal> {
    let definition = registry
        .get(&order.agent_type)
        .map_err(|_| SpawnRefusal::UnknownType)?;
    if let Some(variable) = forbidden_environment_variable(
        definition,
        allow_api_key_value(
            source
                .get("BRIDGET_ALLOW_API_KEY")
                .and_then(|value| value.to_str()),
        ),
        |variable| source.contains_key(variable),
    ) {
        return Err(SpawnRefusal::BillingGuard { variable });
    }
    if definition.protocol != "acp" {
        return Err(SpawnRefusal::NegotiationFailed {
            detail: format!("le protocole '{}' n'est pas ACP", definition.protocol),
        });
    }
    if !order.cwd.is_dir() {
        return Err(SpawnRefusal::CwdGone);
    }
    let env = build_environment(definition, source)?;
    if !command_exists(&definition.command, &env) {
        return Err(SpawnRefusal::CommandMissing {
            command: definition.command.clone(),
            registry: registry.source().display().to_string(),
        });
    }
    Ok(PreparedSpawn {
        lease,
        agent_type: order.agent_type.clone(),
        command: definition.command.clone(),
        args: definition.args.clone(),
        cwd: order.cwd.clone(),
        env,
    })
}

pub fn build_environment(
    definition: &AgentDefinition,
    source: &SourceEnvironment,
) -> Result<SourceEnvironment, SpawnRefusal> {
    let home = source.get("HOME").ok_or_else(|| SpawnRefusal::EnvUnfit {
        detail: "HOME absent de l'environnement du daemon".to_string(),
    })?;
    if !Path::new(home).is_absolute() {
        return Err(SpawnRefusal::EnvUnfit {
            detail: "HOME n'est pas absolu".to_string(),
        });
    }
    let mut env = SourceEnvironment::new();
    for name in BASELINE_ENV {
        if let Some(value) = source.get(*name) {
            env.insert((*name).to_string(), value.clone());
        }
    }
    env.entry("PATH".to_string())
        .or_insert_with(|| OsString::from(FALLBACK_PATH));
    env.entry("TMPDIR".to_string())
        .or_insert_with(|| OsString::from("/tmp"));
    for name in &definition.pass_env {
        if let Some(value) = source.get(name) {
            env.insert(name.clone(), value.clone());
        }
    }
    Ok(env)
}

fn command_exists(command: &str, env: &SourceEnvironment) -> bool {
    let is_executable = |path: &Path| {
        path.metadata()
            .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    };
    if command.contains('/') {
        return is_executable(Path::new(command));
    }
    env.get("PATH")
        .and_then(|path| path.to_str())
        .is_some_and(|path| {
            path.split(':')
                .map(Path::new)
                .map(|dir| dir.join(command))
                .any(|candidate| is_executable(&candidate))
        })
}

fn refusal_record(reason: &SpawnRefusal) -> (&'static str, String) {
    let category = match reason {
        SpawnRefusal::UnknownType => "unknown_type",
        SpawnRefusal::CommandMissing { .. } => "command_missing",
        SpawnRefusal::BillingGuard { .. } => "billing_guard",
        SpawnRefusal::NameActive => "name_active",
        SpawnRefusal::EnvUnfit { .. } => "env_unfit",
        SpawnRefusal::CwdGone => "cwd_gone",
        SpawnRefusal::NegotiationFailed { .. } => "negotiation_failed",
        SpawnRefusal::SpawnTimeout => "spawn_timeout",
        SpawnRefusal::QuotaExceeded { .. } => "quota_exceeded",
        SpawnRefusal::DaemonRecovering => "daemon_recovering",
        SpawnRefusal::IdempotencyExpired => "idempotency_expired",
    };
    (
        category,
        serde_json::to_string(reason).expect("SpawnRefusal est toujours sérialisable"),
    )
}

fn decision_from_issue(issue: SpawnCommandIssue, quota: usize) -> SpawnDecision {
    match issue {
        SpawnCommandIssue::Connected { name, .. } => SpawnDecision::Accepted { name },
        SpawnCommandIssue::Cancelled { reason } if reason == "spawn_timeout" => {
            SpawnDecision::Rejected(SpawnRefusal::SpawnTimeout)
        }
        SpawnCommandIssue::Cancelled { reason } => {
            SpawnDecision::Rejected(SpawnRefusal::NegotiationFailed { detail: reason })
        }
        SpawnCommandIssue::Failed { category, reason } => {
            if let Ok(refusal) = serde_json::from_str::<SpawnRefusal>(&reason) {
                return SpawnDecision::Rejected(refusal);
            }
            let refusal = match category.as_str() {
                "unknown_type" => SpawnRefusal::UnknownType,
                "command_missing" => SpawnRefusal::CommandMissing {
                    command: reason,
                    registry: "registre de l'issue initiale".to_string(),
                },
                "billing_guard" => SpawnRefusal::BillingGuard { variable: reason },
                "name_active" => SpawnRefusal::NameActive,
                "env_unfit" => SpawnRefusal::EnvUnfit { detail: reason },
                "cwd_gone" => SpawnRefusal::CwdGone,
                "spawn_timeout" => SpawnRefusal::SpawnTimeout,
                "quota_exceeded" => SpawnRefusal::QuotaExceeded { limit: quota },
                "daemon_recovering" => SpawnRefusal::DaemonRecovering,
                "idempotency_expired" => SpawnRefusal::IdempotencyExpired,
                _ => SpawnRefusal::NegotiationFailed { detail: reason },
            };
            SpawnDecision::Rejected(refusal)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desired_state::DesiredStateStore;
    use crate::fleet::FleetConfig;
    use std::fs;

    const NOW: i64 = 2_000_000;

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t905-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn supervisor(root: &Path, quota: usize) -> FleetSupervisor {
        fs::create_dir_all(root).unwrap();
        FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota,
                persistent_horizon_secs: 3600,
                ephemeral_horizon_secs: 300,
                issued_at_tolerance_secs: 30,
            },
        )
        .unwrap()
    }

    fn registry(command: &str, protocol: &str, forbidden: &[&str]) -> AgentRegistry {
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": command,
                        "protocol": protocol,
                        "forbidden_env": forbidden,
                        "pass_env": ["SPECIAL_AUTH"]
                    }
                }
            })
            .to_string(),
            "/tmp/t905-agents.json",
        )
        .unwrap()
    }

    fn source(home: &Path) -> SourceEnvironment {
        BTreeMap::from([
            ("HOME".to_string(), home.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("fr_FR.UTF-8")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
            ("SPECIAL_AUTH".to_string(), OsString::from("présent")),
        ])
    }

    fn order(root: &Path, id: &str, name: &str) -> SpawnOrder {
        SpawnOrder {
            agent_type: "fixture".to_string(),
            requested_name: Some(name.to_string()),
            cwd: root.to_path_buf(),
            persistent: false,
            command_id: id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 10,
        }
    }

    fn rejection(decision: SpawnDecision) -> SpawnRefusal {
        match decision {
            SpawnDecision::Rejected(reason) => reason,
            other => panic!("refus attendu, obtenu: {other:?}"),
        }
    }

    #[test]
    fn environnement_construit_ne_copie_que_baseline_et_pass_env() {
        let root = root("env");
        fs::create_dir_all(&root).unwrap();
        let definition = registry("/bin/sh", "acp", &[])
            .get("fixture")
            .unwrap()
            .clone();
        let mut source = source(&root);
        source.insert("SECRET_INATTENDU".to_string(), OsString::from("non"));
        let env = build_environment(&definition, &source).unwrap();
        assert_eq!(env.get("SPECIAL_AUTH"), Some(&OsString::from("présent")));
        assert!(!env.contains_key("SECRET_INATTENDU"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel() {
        // Huit refus indépendants ; NameActive et QuotaExceeded nécessitent
        // une génération témoin active et sont exercés plus bas.
        for (label, expected) in [
            ("unknown", SpawnRefusal::UnknownType),
            (
                "missing",
                SpawnRefusal::CommandMissing {
                    command: "/commande/introuvable".to_string(),
                    registry: "/tmp/t905-agents.json".to_string(),
                },
            ),
            (
                "billing",
                SpawnRefusal::BillingGuard {
                    variable: "API_KEY".to_string(),
                },
            ),
            (
                "env",
                SpawnRefusal::EnvUnfit {
                    detail: "HOME absent de l'environnement du daemon".to_string(),
                },
            ),
            ("cwd", SpawnRefusal::CwdGone),
            (
                "negotiation",
                SpawnRefusal::NegotiationFailed {
                    detail: "le protocole 'tmux' n'est pas ACP".to_string(),
                },
            ),
            ("timeout", SpawnRefusal::SpawnTimeout),
            ("expired", SpawnRefusal::IdempotencyExpired),
        ] {
            let root = root(label);
            fs::create_dir_all(&root).unwrap();
            let supervisor = supervisor(&root, 2);
            let mut request = order(&root, &format!("command-{label}"), "agent-a");
            let mut env = source(&root);
            let registry = match label {
                "unknown" => {
                    request.agent_type = "absent".to_string();
                    registry("/bin/sh", "acp", &[])
                }
                "missing" => registry("/commande/introuvable", "acp", &[]),
                "billing" => {
                    env.insert("API_KEY".to_string(), OsString::from("secret"));
                    registry("/bin/sh", "acp", &["API_KEY"])
                }
                "env" => {
                    env.remove("HOME");
                    registry("/bin/sh", "acp", &[])
                }
                "cwd" => {
                    request.cwd = root.join("disparu");
                    registry("/bin/sh", "acp", &[])
                }
                "negotiation" => registry("/bin/sh", "tmux", &[]),
                "timeout" => {
                    request.issued_at = NOW - 2;
                    request.deadline_at = NOW - 1;
                    registry("/bin/sh", "acp", &[])
                }
                "expired" => {
                    request.issued_at = NOW - 400;
                    request.deadline_at = NOW - 390;
                    registry("/bin/sh", "acp", &[])
                }
                _ => unreachable!(),
            };
            let actual = rejection(
                submit_spawn(&supervisor, &registry, &env, &request, NOW, false).unwrap(),
            );
            assert_eq!(actual, expected, "famille {label}");
            let replay = rejection(
                submit_spawn(&supervisor, &registry, &env, &request, NOW, false).unwrap(),
            );
            assert_eq!(replay, expected, "rejeu divergent pour {label}");
            assert_eq!(supervisor.active_count(), 0, "résidu actif pour {label}");
            let _ = fs::remove_dir_all(root);
        }

        let nq_root = root("name-quota");
        fs::create_dir_all(&nq_root).unwrap();
        let nq_supervisor = supervisor(&nq_root, 1);
        let nq_registry = registry("/bin/sh", "acp", &[]);
        let env = source(&nq_root);
        assert!(matches!(
            submit_spawn(
                &nq_supervisor,
                &nq_registry,
                &env,
                &order(&nq_root, "command-first", "agent-a"),
                NOW,
                false,
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(&nq_root, "command-name", "agent-a"),
                    NOW,
                    false,
                )
                .unwrap()
            ),
            SpawnRefusal::NameActive
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(&nq_root, "command-quota", "agent-b"),
                    NOW,
                    false,
                )
                .unwrap()
            ),
            SpawnRefusal::QuotaExceeded { limit: 1 }
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        let _ = fs::remove_dir_all(nq_root);

        let root = root("recovering");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry("/bin/sh", "acp", &[]);
        assert_eq!(
            rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &source(&root),
                    &order(&root, "command-recovering", "agent-r"),
                    NOW,
                    true,
                )
                .unwrap()
            ),
            SpawnRefusal::DaemonRecovering
        );
        assert_eq!(supervisor.active_count(), 0);
        assert!(!supervisor.knows_command("command-recovering"));
        let _ = fs::remove_dir_all(root);
    }
}
