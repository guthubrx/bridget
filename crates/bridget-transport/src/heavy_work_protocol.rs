//! Contrat fermé des travaux lourds hors poste (Spécification 312, Lot 1a).
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const HEAVY_WORK_POLICY_VERSION: u16 = 1;
pub const HEAVY_WORK_LEASE_TTL_SECS: u64 = 60;
pub const HEAVY_WORK_MIN_WAIT_SECS: u64 = 60;
pub const HEAVY_WORK_MAX_WAIT_SECS: u64 = 3600;
pub const HEAVY_WORK_DEFAULT_WAIT_SECS: u64 = 600;
pub const HEAVY_WORK_MAX_LEASES_PER_MACHINE: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkRole {
    #[serde(rename = "ouverte")]
    Open,
    #[serde(rename = "protégée", alias = "protegee")]
    Protected,
    #[serde(rename = "exclue")]
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkOrigin {
    User,
    Mission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkUnavailableAction {
    #[serde(rename = "attendre")]
    Wait,
    #[serde(rename = "repli")]
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkFallbackAction {
    #[serde(rename = "discret")]
    Discrete,
    #[serde(rename = "refuser")]
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkInstallPolicy {
    #[serde(rename = "demander")]
    Ask,
    #[serde(rename = "outils_du_catalogue")]
    CatalogTools,
    #[serde(rename = "jamais")]
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeavyWorkPreflightFailure {
    Unreachable,
    Busy,
    PrerequisiteMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkSshProfile {
    pub host: String,
    pub user: String,
    pub port: u16,
    /// Chemin relatif à `$HOME` sur la machine appelante (sans `..` ni lien).
    pub identity_rel: String,
    /// Chemin relatif à `$HOME` sur la machine appelante (sans `..` ni lien).
    pub known_hosts_rel: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkMachinePolicy {
    pub role: HeavyWorkRole,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<HeavyWorkSshProfile>,
    #[serde(default = "default_max_leases")]
    pub max_leases: u8,
    #[serde(default = "default_install_policy")]
    pub install_policy: HeavyWorkInstallPolicy,
}

fn default_max_leases() -> u8 {
    1
}

fn default_install_policy() -> HeavyWorkInstallPolicy {
    HeavyWorkInstallPolicy::Ask
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkOriginPolicy {
    pub on_busy_or_unreachable: HeavyWorkUnavailableAction,
    pub on_no_open_target: HeavyWorkFallbackAction,
    pub fallback_after_wait: HeavyWorkFallbackAction,
}

impl HeavyWorkOriginPolicy {
    pub fn default_user() -> Self {
        Self {
            on_busy_or_unreachable: HeavyWorkUnavailableAction::Wait,
            on_no_open_target: HeavyWorkFallbackAction::Discrete,
            fallback_after_wait: HeavyWorkFallbackAction::Discrete,
        }
    }

    pub fn default_mission() -> Self {
        Self {
            on_busy_or_unreachable: HeavyWorkUnavailableAction::Wait,
            on_no_open_target: HeavyWorkFallbackAction::Refuse,
            fallback_after_wait: HeavyWorkFallbackAction::Refuse,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkPolicyFile {
    pub version: u16,
    #[serde(default = "default_max_wait_secs")]
    pub max_wait_secs: u64,
    #[serde(default = "HeavyWorkOriginPolicy::default_user")]
    pub user: HeavyWorkOriginPolicy,
    #[serde(default = "HeavyWorkOriginPolicy::default_mission")]
    pub mission: HeavyWorkOriginPolicy,
    #[serde(default)]
    pub machines: BTreeMap<String, HeavyWorkMachinePolicy>,
}

fn default_max_wait_secs() -> u64 {
    HEAVY_WORK_DEFAULT_WAIT_SECS
}

impl Default for HeavyWorkPolicyFile {
    fn default() -> Self {
        Self {
            version: HEAVY_WORK_POLICY_VERSION,
            max_wait_secs: HEAVY_WORK_DEFAULT_WAIT_SECS,
            user: HeavyWorkOriginPolicy::default_user(),
            mission: HeavyWorkOriginPolicy::default_mission(),
            machines: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkExclusion {
    pub machine_id: String,
    pub reason: HeavyWorkPreflightFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeavyWorkRequest {
    Status,
    Plan {
        caller_machine: String,
        target: String,
        origin: HeavyWorkOrigin,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requested_host: Option<String>,
        #[serde(default)]
        exclusions: Vec<HeavyWorkExclusion>,
    },
    Acquire {
        caller_machine: String,
        target: String,
        origin: HeavyWorkOrigin,
        repo_owner: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        requested_host: Option<String>,
        #[serde(default)]
        exclusions: Vec<HeavyWorkExclusion>,
    },
    Renew {
        lease_id: String,
    },
    Release {
        lease_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkLeaseInfo {
    pub lease_id: String,
    pub machine_id: String,
    pub caller_machine: String,
    pub target: String,
    pub origin: HeavyWorkOrigin,
    pub repo_owner: String,
    pub acquired_at_unix: u64,
    pub expires_at_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeavyWorkMachineStatus {
    pub machine_id: String,
    pub role: HeavyWorkRole,
    pub targets: Vec<String>,
    pub max_leases: u8,
    pub active_leases: usize,
    pub compilation_capacity: u8,
    pub install_policy: HeavyWorkInstallPolicy,
    pub has_ssh_profile: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeavyWorkDecision {
    PolicyAbsent {
        reason: String,
    },
    RunRemote {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lease_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at_unix: Option<u64>,
        machine_id: String,
        ssh: HeavyWorkSshProfile,
        reason: String,
    },
    RunLocalLease {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lease_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expires_at_unix: Option<u64>,
        machine_id: String,
        reason: String,
    },
    Wait {
        retry_after_secs: u64,
        max_wait_secs: u64,
        candidates: Vec<String>,
        fallback_after_wait: HeavyWorkFallbackAction,
        case_kind: String,
        reason: String,
    },
    FallbackDiscrete {
        case_kind: String,
        reason: String,
    },
    Refused {
        code: String,
        case_kind: String,
        message: String,
        action: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "snake_case", deny_unknown_fields)]
pub enum HeavyWorkResponse {
    Status {
        configured: bool,
        master_host: String,
        max_wait_secs: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        user: Option<HeavyWorkOriginPolicy>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mission: Option<HeavyWorkOriginPolicy>,
        machines: Vec<HeavyWorkMachineStatus>,
        leases: Vec<HeavyWorkLeaseInfo>,
    },
    Decision {
        decision: HeavyWorkDecision,
    },
    Renewed {
        lease_id: String,
        expires_at_unix: u64,
    },
    Released {
        lease_id: String,
        released: bool,
    },
    Error {
        code: String,
        message: String,
    },
}
