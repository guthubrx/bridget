//! Travaux lourds hors du poste de travail (Spécification 312, Lot 1a).
//! Politique stockée uniquement sur le maître (`heavy-work-policy.json`), baux
//! volatils en mémoire (TTL 60 s), sélection par classe avant score, origine
//! fail-closed et sous-commandes CLI `build-info` et `heavy-work`.

use crate::communication::client::DaemonConnection;
use crate::daemon::DaemonConfig;
use crate::registry::AgentRegistry;
use bridget_transport::protocol::{
    HEAVY_WORK_LEASE_TTL_SECS, HEAVY_WORK_MAX_LEASES_PER_MACHINE, HEAVY_WORK_MAX_WAIT_SECS,
    HEAVY_WORK_MIN_WAIT_SECS, HEAVY_WORK_POLICY_VERSION, HeavyWorkDecision, HeavyWorkExclusion,
    HeavyWorkFallbackAction, HeavyWorkInstallPolicy, HeavyWorkLeaseInfo, HeavyWorkMachinePolicy,
    HeavyWorkMachineStatus, HeavyWorkOrigin, HeavyWorkPolicyFile, HeavyWorkPreflightFailure,
    HeavyWorkRequest, HeavyWorkResponse, HeavyWorkRole, HeavyWorkSshProfile,
    HeavyWorkUnavailableAction,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub(crate) const POLICY_FILENAME: &str = "heavy-work-policy.json";
const POLICY_LOCK_FILENAME: &str = "heavy-work-policy.lock";
const MAX_POLICY_BYTES: u64 = 256 * 1024;
const KNOWN_TARGETS: &[&str] = &["macos-arm64", "linux-x86_64-glibc2.36"];

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn policy_path(namespace_root: &Path) -> PathBuf {
    namespace_root.join(POLICY_FILENAME)
}

fn valid_machine_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
}

fn valid_target(value: &str) -> bool {
    KNOWN_TARGETS.contains(&value)
}

fn valid_ssh_host(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 253
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b':'))
}

fn valid_ssh_user(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with('-')
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
}

fn valid_relative_home_path(value: &str) -> bool {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return false;
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return false;
    }
    let mut has_normal = false;
    for component in path.components() {
        match component {
            Component::Normal(_) => has_normal = true,
            Component::CurDir => {}
            _ => return false,
        }
    }
    has_normal
}

pub(crate) fn validate_ssh_profile(ssh: &HeavyWorkSshProfile) -> Result<(), String> {
    if !valid_ssh_host(&ssh.host) {
        return Err("heavy_work_ssh_host_invalid".into());
    }
    if !valid_ssh_user(&ssh.user) {
        return Err("heavy_work_ssh_user_invalid".into());
    }
    if ssh.port == 0 {
        return Err("heavy_work_ssh_port_invalid".into());
    }
    if !valid_relative_home_path(&ssh.identity_rel) {
        return Err("heavy_work_ssh_identity_rel_invalid".into());
    }
    if !valid_relative_home_path(&ssh.known_hosts_rel) {
        return Err("heavy_work_ssh_known_hosts_rel_invalid".into());
    }
    Ok(())
}

pub(crate) fn validate_policy(policy: &HeavyWorkPolicyFile) -> Result<(), String> {
    if policy.version != HEAVY_WORK_POLICY_VERSION {
        return Err("heavy_work_policy_version_unsupported".into());
    }
    if !(HEAVY_WORK_MIN_WAIT_SECS..=HEAVY_WORK_MAX_WAIT_SECS).contains(&policy.max_wait_secs) {
        return Err("heavy_work_max_wait_out_of_range".into());
    }
    if policy.machines.len() > 64 {
        return Err("heavy_work_machine_limit_exceeded".into());
    }
    for (machine_id, entry) in &policy.machines {
        if !valid_machine_id(machine_id) {
            return Err("heavy_work_machine_id_invalid".into());
        }
        if entry.max_leases == 0 || entry.max_leases > HEAVY_WORK_MAX_LEASES_PER_MACHINE {
            return Err("heavy_work_max_leases_out_of_range".into());
        }
        for target in &entry.targets {
            if !valid_target(target) {
                return Err(format!("heavy_work_target_unknown:{target}"));
            }
        }
        if let Some(ssh) = &entry.ssh {
            validate_ssh_profile(ssh)?;
        }
    }
    Ok(())
}

/// Lit `heavy-work-policy.json` en lecture privée stricte.
/// Retourne `Ok(None)` si le fichier n'existe pas (défaut sûr §3.1).
pub(crate) fn load_policy(namespace_root: &Path) -> Result<Option<HeavyWorkPolicyFile>, String> {
    let path = policy_path(namespace_root);
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("heavy_work_policy_unreadable:{error}")),
    };
    if meta.file_type().is_symlink() || !meta.is_file() {
        return Err("heavy_work_policy_not_regular_file".into());
    }
    if meta.uid() != bridget_transport::fsutil::effective_uid()
        || (meta.permissions().mode() & 0o077) != 0
    {
        return Err("heavy_work_policy_permissions_invalid".into());
    }
    if meta.len() > MAX_POLICY_BYTES {
        return Err("heavy_work_policy_too_large".into());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .map_err(|error| format!("heavy_work_policy_unreadable:{error}"))?;
    let mut bytes = Vec::new();
    std::io::Read::by_ref(&mut file)
        .take(MAX_POLICY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("heavy_work_policy_unreadable:{error}"))?;
    if bytes.len() as u64 > MAX_POLICY_BYTES {
        return Err("heavy_work_policy_too_large".into());
    }
    let policy: HeavyWorkPolicyFile = serde_json::from_slice(&bytes)
        .map_err(|error| format!("heavy_work_policy_invalid_json:{error}"))?;
    validate_policy(&policy)?;
    Ok(Some(policy))
}

/// Écriture atomique avec verrou, sauvegarde horodatée `0600` et relecture.
pub(crate) fn write_policy_atomic(
    namespace_root: &Path,
    policy: &HeavyWorkPolicyFile,
) -> Result<(PathBuf, Option<PathBuf>), String> {
    validate_policy(policy)?;
    let bytes = serde_json::to_vec_pretty(policy)
        .map_err(|error| format!("heavy_work_policy_encode_failed:{error}"))?;
    let mut data = bytes;
    data.push(b'\n');
    if data.len() as u64 > MAX_POLICY_BYTES {
        return Err("heavy_work_policy_too_large".into());
    }
    let lock_path = namespace_root.join(POLICY_LOCK_FILENAME);
    let lock_file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&lock_path)
        .map_err(|error| format!("heavy_work_policy_lock_failed:{error}"))?;
    use std::os::unix::io::AsRawFd;
    // SAFETY: descripteur ouvert valide détenu par `lock_file`.
    if unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX) } != 0 {
        return Err("heavy_work_policy_lock_failed".into());
    }
    let target = policy_path(namespace_root);
    let backup = if target.exists() {
        let backup_path = namespace_root.join(format!("{POLICY_FILENAME}.bak-{}", now_unix()));
        let existing = fs::read(&target)
            .map_err(|error| format!("heavy_work_policy_backup_failed:{error}"))?;
        let mut bfile = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&backup_path)
            .map_err(|error| format!("heavy_work_policy_backup_failed:{error}"))?;
        bfile
            .write_all(&existing)
            .and_then(|_| bfile.sync_all())
            .map_err(|error| format!("heavy_work_policy_backup_failed:{error}"))?;
        Some(backup_path)
    } else {
        None
    };
    let tmp = namespace_root.join(format!(".{POLICY_FILENAME}.tmp-{}", Uuid::new_v4()));
    let write_res = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&tmp)
            .map_err(|error| format!("heavy_work_policy_write_failed:{error}"))?;
        file.write_all(&data)
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("heavy_work_policy_write_failed:{error}"))?;
        fs::rename(&tmp, &target)
            .map_err(|error| format!("heavy_work_policy_rename_failed:{error}"))?;
        Ok::<(), String>(())
    })();
    if let Err(error) = write_res {
        let _ = fs::remove_file(&tmp);
        return Err(error);
    }
    // Relecture de vérification après remplacement.
    let reread = load_policy(namespace_root)?;
    if reread.as_ref() != Some(policy) {
        return Err("heavy_work_policy_readback_mismatch".into());
    }
    Ok((target, backup))
}

#[derive(Default)]
pub(crate) struct HeavyWorkLeaseManager {
    leases: HashMap<String, HeavyWorkLeaseInfo>,
}

impl HeavyWorkLeaseManager {
    pub(crate) fn purge_expired(&mut self, now: u64) {
        self.leases.retain(|_, lease| lease.expires_at_unix > now);
    }

    pub(crate) fn active_count_for(&self, machine_id: &str) -> usize {
        self.leases
            .values()
            .filter(|lease| lease.machine_id == machine_id)
            .count()
    }

    pub(crate) fn handle_request(
        &mut self,
        request: HeavyWorkRequest,
        namespace_root: &Path,
        master_host: &str,
        registry: &AgentRegistry,
    ) -> HeavyWorkResponse {
        self.handle_request_at(request, namespace_root, master_host, registry, now_unix())
    }

    pub(crate) fn handle_request_at(
        &mut self,
        request: HeavyWorkRequest,
        namespace_root: &Path,
        master_host: &str,
        registry: &AgentRegistry,
        now: u64,
    ) -> HeavyWorkResponse {
        self.purge_expired(now);
        match request {
            HeavyWorkRequest::Renew { lease_id } => {
                if let Some(lease) = self.leases.get_mut(&lease_id) {
                    lease.expires_at_unix = now.saturating_add(HEAVY_WORK_LEASE_TTL_SECS);
                    HeavyWorkResponse::Renewed {
                        lease_id,
                        expires_at_unix: lease.expires_at_unix,
                    }
                } else {
                    HeavyWorkResponse::Error {
                        code: "heavy_work_lease_not_found".into(),
                        message: "Ce bail n'existe plus ou a expiré (TTL 60 s).".into(),
                    }
                }
            }
            HeavyWorkRequest::Release { lease_id } => {
                let released = self.leases.remove(&lease_id).is_some();
                HeavyWorkResponse::Released { lease_id, released }
            }
            HeavyWorkRequest::Status => {
                let policy = match load_policy(namespace_root) {
                    Ok(policy) => policy,
                    Err(code) => {
                        return HeavyWorkResponse::Error {
                            code,
                            message:
                                "La politique heavy-work-policy.json est invalide ou illisible."
                                    .into(),
                        };
                    }
                };
                let mut leases: Vec<HeavyWorkLeaseInfo> = self.leases.values().cloned().collect();
                leases.sort_by(|a, b| a.lease_id.cmp(&b.lease_id));
                let Some(policy) = policy else {
                    return HeavyWorkResponse::Status {
                        configured: false,
                        master_host: master_host.to_string(),
                        max_wait_secs: bridget_transport::protocol::HEAVY_WORK_DEFAULT_WAIT_SECS,
                        user: None,
                        mission: None,
                        machines: Vec::new(),
                        leases,
                    };
                };
                let machines = policy
                    .machines
                    .iter()
                    .map(|(machine_id, entry)| HeavyWorkMachineStatus {
                        machine_id: machine_id.clone(),
                        role: entry.role,
                        targets: entry.targets.clone(),
                        max_leases: entry.max_leases,
                        active_leases: self.active_count_for(machine_id),
                        compilation_capacity: compilation_capacity_for(registry, machine_id),
                        install_policy: entry.install_policy,
                        has_ssh_profile: entry.ssh.is_some(),
                    })
                    .collect();
                HeavyWorkResponse::Status {
                    configured: true,
                    master_host: master_host.to_string(),
                    max_wait_secs: policy.max_wait_secs,
                    user: Some(policy.user),
                    mission: Some(policy.mission),
                    machines,
                    leases,
                }
            }
            HeavyWorkRequest::Plan {
                caller_machine,
                target,
                origin,
                requested_host,
                exclusions,
            } => self.decide(
                namespace_root,
                registry,
                &caller_machine,
                &target,
                origin,
                None,
                requested_host.as_deref(),
                &exclusions,
                now,
            ),
            HeavyWorkRequest::Acquire {
                caller_machine,
                target,
                origin,
                repo_owner,
                requested_host,
                exclusions,
            } => self.decide(
                namespace_root,
                registry,
                &caller_machine,
                &target,
                origin,
                Some(&repo_owner),
                requested_host.as_deref(),
                &exclusions,
                now,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn decide(
        &mut self,
        namespace_root: &Path,
        registry: &AgentRegistry,
        caller_machine: &str,
        target: &str,
        origin: HeavyWorkOrigin,
        acquire_repo_owner: Option<&str>,
        requested_host: Option<&str>,
        exclusions: &[HeavyWorkExclusion],
        now: u64,
    ) -> HeavyWorkResponse {
        if !valid_machine_id(caller_machine) {
            return HeavyWorkResponse::Error {
                code: "heavy_work_caller_invalid".into(),
                message: "Nom de machine appelante invalide.".into(),
            };
        }
        if !valid_target(target) {
            return HeavyWorkResponse::Error {
                code: "heavy_work_target_unknown".into(),
                message: format!(
                    "Cible inconnue « {target} ». Cibles admises : {}.",
                    KNOWN_TARGETS.join(", ")
                ),
            };
        }
        let policy = match load_policy(namespace_root) {
            Ok(Some(policy)) => policy,
            Ok(None) => {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::PolicyAbsent {
                        reason: "heavy-work-policy.json absent sur le maître : déport inactif, mode discret local sous garde cible."
                            .into(),
                    },
                };
            }
            Err(code) => {
                return HeavyWorkResponse::Error {
                    code,
                    message: "La politique heavy-work-policy.json est invalide ou illisible."
                        .into(),
                };
            }
        };

        let origin_policy = match origin {
            HeavyWorkOrigin::User => &policy.user,
            HeavyWorkOrigin::Mission => &policy.mission,
        };
        let caller_role = policy
            .machines
            .get(caller_machine)
            .map(|m| m.role)
            .unwrap_or(HeavyWorkRole::Protected);

        // Cas --host explicite (§3.3.3) : respecte toujours le rôle `exclue`.
        if let Some(host) = requested_host {
            if !valid_machine_id(host) {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Refused {
                        code: "heavy_work_host_invalid".into(),
                        case_kind: "explicit_host".into(),
                        message: format!("Machine demandée « {host} » invalide."),
                        action: "Indiquer un machine_id déclaré dans heavy-work-policy.json."
                            .into(),
                    },
                };
            }
            let Some(entry) = policy.machines.get(host) else {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Refused {
                        code: "heavy_work_host_unknown".into(),
                        case_kind: "explicit_host".into(),
                        message: format!(
                            "La machine « {host} » n'est pas déclarée dans heavy-work-policy.json."
                        ),
                        action: "Déclarer cette machine via `bridget heavy-work policy` sur le maître, ou fournir les paramètres SSH explicites sans daemon."
                            .into(),
                    },
                };
            };
            if entry.role == HeavyWorkRole::Excluded {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Refused {
                        code: "heavy_work_host_excluded".into(),
                        case_kind: "explicit_host".into(),
                        message: format!(
                            "La machine « {host} » a le rôle « exclue » dans heavy-work-policy.json."
                        ),
                        action: "Choisir une autre machine ou modifier son rôle sur le maître."
                            .into(),
                    },
                };
            }
            if host != caller_machine && entry.ssh.is_none() {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Refused {
                        code: "heavy_work_ssh_profile_missing".into(),
                        case_kind: "explicit_host".into(),
                        message: format!(
                            "Aucun profil SSH déclaré pour « {host} » dans heavy-work-policy.json."
                        ),
                        action: "Configurer le profil SSH de cette machine via `bridget heavy-work policy` sur le maître."
                            .into(),
                    },
                };
            }
            let active = self.active_count_for(host);
            if active >= usize::from(entry.max_leases) {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Wait {
                        retry_after_secs: 5,
                        max_wait_secs: policy.max_wait_secs,
                        candidates: vec![host.to_string()],
                        fallback_after_wait: if caller_role == HeavyWorkRole::Excluded {
                            HeavyWorkFallbackAction::Refuse
                        } else {
                            origin_policy.fallback_after_wait
                        },
                        case_kind: "A".into(),
                        reason: format!(
                            "La machine demandée « {host} » a déjà {active}/{} bail(s) actif(s).",
                            entry.max_leases
                        ),
                    },
                };
            }
            return self.grant_machine(
                host,
                entry,
                caller_machine,
                target,
                origin,
                acquire_repo_owner,
                now,
                format!("Machine « {host} » imposée par --host."),
            );
        }

        // Sélection automatique (§3.2.3) : filtre de classe avant tout score.
        let excluded_map: HashMap<&str, HeavyWorkPreflightFailure> = exclusions
            .iter()
            .map(|e| (e.machine_id.as_str(), e.reason))
            .collect();

        let open_capable: Vec<(&String, &HeavyWorkMachinePolicy)> = policy
            .machines
            .iter()
            .filter(|(id, m)| {
                m.role == HeavyWorkRole::Open
                    && m.targets.iter().any(|t| t == target)
                    && (id.as_str() == caller_machine || m.ssh.is_some())
            })
            .collect();

        let mut available: Vec<(&String, &HeavyWorkMachinePolicy, bool, u8)> = open_capable
            .iter()
            .filter(|(id, _)| !excluded_map.contains_key(id.as_str()))
            .map(|(id, m)| {
                let free = self.active_count_for(id) < usize::from(m.max_leases);
                let cap = compilation_capacity_for(registry, id);
                (*id, *m, free, cap)
            })
            .collect();

        // Tri déterministe (§3.2.3.3) : bail libre → compilation_capacity décroissant → machine_id croissant.
        available.sort_by(|a, b| {
            b.2.cmp(&a.2)
                .then_with(|| b.3.cmp(&a.3))
                .then_with(|| a.0.cmp(b.0))
        });

        if let Some((machine_id, entry, true, cap)) = available.first().copied() {
            let reason = format!(
                "Machine ouverte « {machine_id} » choisie pour {target} (bail libre, compilation_capacity={cap})."
            );
            return self.grant_machine(
                machine_id,
                entry,
                caller_machine,
                target,
                origin,
                acquire_repo_owner,
                now,
                reason,
            );
        }

        // Aucune machine ouverte libre : distinguer cas A et cas B (§3.2.3.5).
        let busy_candidates: Vec<String> = available
            .iter()
            .filter(|(_, _, free, _)| !free)
            .map(|(id, _, _, _)| (*id).clone())
            .collect();
        let unreachable_candidates: Vec<String> = open_capable
            .iter()
            .filter(|(id, _)| {
                matches!(
                    excluded_map.get(id.as_str()),
                    Some(HeavyWorkPreflightFailure::Unreachable | HeavyWorkPreflightFailure::Busy)
                )
            })
            .map(|(id, _)| (*id).clone())
            .collect();
        let has_case_a = !busy_candidates.is_empty() || !unreachable_candidates.is_empty();

        if has_case_a {
            let mut candidates = busy_candidates;
            for c in unreachable_candidates {
                if !candidates.contains(&c) {
                    candidates.push(c);
                }
            }
            candidates.sort();
            let reason = format!(
                "Machine(s) ouverte(s) pour {target} occupée(s) ou injoignable(s) : {}.",
                candidates.join(", ")
            );
            match origin_policy.on_busy_or_unreachable {
                HeavyWorkUnavailableAction::Wait => {
                    return HeavyWorkResponse::Decision {
                        decision: HeavyWorkDecision::Wait {
                            retry_after_secs: 5,
                            max_wait_secs: policy.max_wait_secs,
                            candidates,
                            fallback_after_wait: if caller_role == HeavyWorkRole::Excluded {
                                HeavyWorkFallbackAction::Refuse
                            } else {
                                origin_policy.fallback_after_wait
                            },
                            case_kind: "A".into(),
                            reason,
                        },
                    };
                }
                HeavyWorkUnavailableAction::Fallback => {
                    return self.apply_fallback(
                        caller_machine,
                        caller_role,
                        target,
                        origin_policy.fallback_after_wait,
                        "A",
                        reason,
                    );
                }
            }
        }

        let reason = if open_capable.is_empty() {
            format!("Aucune machine ouverte ne déclare la cible {target}.")
        } else {
            format!(
                "Toutes les machines ouvertes déclarant {target} ont échoué sur un prérequis absent."
            )
        };
        self.apply_fallback(
            caller_machine,
            caller_role,
            target,
            origin_policy.on_no_open_target,
            "B",
            reason,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn grant_machine(
        &mut self,
        machine_id: &str,
        entry: &HeavyWorkMachinePolicy,
        caller_machine: &str,
        target: &str,
        origin: HeavyWorkOrigin,
        acquire_repo_owner: Option<&str>,
        now: u64,
        reason: String,
    ) -> HeavyWorkResponse {
        let (lease_id, expires_at_unix) = if let Some(repo_owner) = acquire_repo_owner {
            let id = Uuid::new_v4().to_string();
            let expires = now.saturating_add(HEAVY_WORK_LEASE_TTL_SECS);
            self.leases.insert(
                id.clone(),
                HeavyWorkLeaseInfo {
                    lease_id: id.clone(),
                    machine_id: machine_id.to_string(),
                    caller_machine: caller_machine.to_string(),
                    target: target.to_string(),
                    origin,
                    repo_owner: repo_owner.to_string(),
                    acquired_at_unix: now,
                    expires_at_unix: expires,
                },
            );
            (Some(id), Some(expires))
        } else {
            (None, None)
        };

        let decision = if machine_id == caller_machine {
            HeavyWorkDecision::RunLocalLease {
                lease_id,
                expires_at_unix,
                machine_id: machine_id.to_string(),
                reason,
            }
        } else {
            let Some(ssh) = entry.ssh.clone() else {
                return HeavyWorkResponse::Decision {
                    decision: HeavyWorkDecision::Refused {
                        code: "heavy_work_ssh_profile_missing".into(),
                        case_kind: "B".into(),
                        message: format!("Profil SSH absent pour « {machine_id} »."),
                        action: "Déclarer le profil SSH de cette machine sur le maître.".into(),
                    },
                };
            };
            HeavyWorkDecision::RunRemote {
                lease_id,
                expires_at_unix,
                machine_id: machine_id.to_string(),
                ssh,
                reason,
            }
        };
        HeavyWorkResponse::Decision { decision }
    }

    fn apply_fallback(
        &self,
        caller_machine: &str,
        caller_role: HeavyWorkRole,
        target: &str,
        fallback: HeavyWorkFallbackAction,
        case_kind: &str,
        reason: String,
    ) -> HeavyWorkResponse {
        if caller_role == HeavyWorkRole::Excluded {
            return HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Refused {
                    code: "heavy_work_caller_excluded".into(),
                    case_kind: case_kind.into(),
                    message: format!(
                        "{reason} Le poste appelant « {caller_machine} » a le rôle « exclue » : tout repli local est interdit."
                    ),
                    action: "Rendre disponible une machine ouverte pour cette cible ou utiliser --host <machine>."
                        .into(),
                },
            };
        }
        match fallback {
            HeavyWorkFallbackAction::Discrete => HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::FallbackDiscrete {
                    case_kind: case_kind.into(),
                    reason,
                },
            },
            HeavyWorkFallbackAction::Refuse => HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Refused {
                    code: format!("heavy_work_refused_case_{}", case_kind.to_lowercase()),
                    case_kind: case_kind.into(),
                    message: format!(
                        "{reason} La politique refuse le repli sur le poste protégé pour {target}."
                    ),
                    action: "Attendre ou allumer une machine ouverte capable, ou relancer explicitement avec --local ou --host <machine>."
                        .into(),
                },
            },
        }
    }
}

fn compilation_capacity_for(registry: &AgentRegistry, machine_id: &str) -> u8 {
    registry
        .routing_policy()
        .and_then(|policy| {
            policy
                .machine_policies
                .get(machine_id)
                .and_then(|m| m.compilation_capacity)
        })
        .unwrap_or(0)
}

/// Résout l'origine sans option (§3.1, fail-closed) :
/// toute trace d'agent, de session T3 ou de marqueur délégué vaut `Mission`.
pub(crate) fn resolve_origin() -> HeavyWorkOrigin {
    use crate::mcp_identity::{IdentityError, SystemProcessTree};
    let delegated = crate::environment::Namespace::from_environment()
        .ok()
        .is_some_and(|ns| {
            crate::mcp_identity::has_delegated_marker(
                &ns.root.join("delegated-pids"),
                std::process::id(),
                &SystemProcessTree,
            )
        });
    let native = crate::mcp_identity::resolve_current_identity();
    let mcp = crate::mcp_identity::resolve_current_mcp_identity();
    let t3 = crate::t3code_mcp::resolve_current();
    classify_origin(
        delegated,
        !matches!(native, Err(IdentityError::IdentityNotFound)),
        !matches!(mcp, Err(IdentityError::IdentityNotFound)),
        t3.is_some(),
    )
}

pub(crate) fn classify_origin(
    delegated_marker: bool,
    native_agent_present: bool,
    mcp_identity_present: bool,
    t3_session_present: bool,
) -> HeavyWorkOrigin {
    if delegated_marker || native_agent_present || mcp_identity_present || t3_session_present {
        HeavyWorkOrigin::Mission
    } else {
        HeavyWorkOrigin::User
    }
}

/// `bridget build-info --json` : sans daemon, sans namespace, arguments stricts.
pub(crate) fn run_build_info(args: &[String]) -> Result<String, String> {
    if args.len() != 1 || args[0] != "--json" {
        return Err("build-info : usage exact : bridget build-info --json".into());
    }
    let payload = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_id": crate::build_info::BUILD_ID,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
    });
    serde_json::to_string(&payload).map_err(|error| error.to_string())
}

fn send_heavy_work_rpc(request: HeavyWorkRequest) -> Result<HeavyWorkResponse, String> {
    let config = DaemonConfig::default();
    let mut conn = DaemonConnection::connect_until(
        &config.socket_path,
        Instant::now() + Duration::from_secs(5),
    )
    .map_err(|error| format!("daemon_unreachable:{error}"))?;
    let response = conn
        .exchange(&WrapperToDaemon::HeavyWork { request })
        .map_err(|error| format!("daemon_rpc_failed:{error}"))?;
    match response {
        DaemonToWrapper::HeavyWorkResult { response } => Ok(response),
        other => Err(format!("daemon_unexpected_response:{other:?}")),
    }
}

fn parse_exclusion(raw: &str) -> Result<HeavyWorkExclusion, String> {
    let (machine_id, motif) = raw
        .split_once(':')
        .ok_or_else(|| format!("--exclude attendu <machine>:<motif>, reçu « {raw} »"))?;
    if !valid_machine_id(machine_id) {
        return Err(format!("machine_id invalide dans --exclude : {machine_id}"));
    }
    let reason = match motif {
        "unreachable" | "injoignable" => HeavyWorkPreflightFailure::Unreachable,
        "busy" | "occupee" | "occupée" => HeavyWorkPreflightFailure::Busy,
        "prerequisite_missing" | "prerequis" | "prérequis" => {
            HeavyWorkPreflightFailure::PrerequisiteMissing
        }
        _ => {
            return Err(format!(
                "motif --exclude inconnu « {motif} » (attendu: unreachable | busy | prerequisite_missing)"
            ));
        }
    };
    Ok(HeavyWorkExclusion {
        machine_id: machine_id.to_string(),
        reason,
    })
}

fn parse_role(raw: &str) -> Result<HeavyWorkRole, String> {
    match raw {
        "ouverte" | "open" => Ok(HeavyWorkRole::Open),
        "protégée" | "protegee" | "protected" => Ok(HeavyWorkRole::Protected),
        "exclue" | "excluded" => Ok(HeavyWorkRole::Excluded),
        _ => Err(format!(
            "rôle invalide « {raw} » (attendu: ouverte | protégée | exclue)"
        )),
    }
}

fn parse_unavailable_action(raw: &str) -> Result<HeavyWorkUnavailableAction, String> {
    match raw {
        "attendre" | "wait" => Ok(HeavyWorkUnavailableAction::Wait),
        "repli" | "fallback" => Ok(HeavyWorkUnavailableAction::Fallback),
        _ => Err(format!(
            "action cas A invalide « {raw} » (attendu: attendre | repli)"
        )),
    }
}

fn parse_fallback_action(raw: &str) -> Result<HeavyWorkFallbackAction, String> {
    match raw {
        "discret" | "discrete" => Ok(HeavyWorkFallbackAction::Discrete),
        "refuser" | "refuse" => Ok(HeavyWorkFallbackAction::Refuse),
        _ => Err(format!(
            "action de repli invalide « {raw} » (attendu: discret | refuser)"
        )),
    }
}

fn parse_install_policy(raw: &str) -> Result<HeavyWorkInstallPolicy, String> {
    match raw {
        "demander" | "ask" => Ok(HeavyWorkInstallPolicy::Ask),
        "outils_du_catalogue" | "catalog_tools" => Ok(HeavyWorkInstallPolicy::CatalogTools),
        "jamais" | "never" => Ok(HeavyWorkInstallPolicy::Never),
        _ => Err(format!(
            "politique d'installation invalide « {raw} » (attendu: demander | outils_du_catalogue | jamais)"
        )),
    }
}

fn require_local_master_authority(namespace_root: &Path) -> Result<(), String> {
    let registry_path = namespace_root.join("agents.json");
    let local_machine = crate::build_info::local_host();
    let authority = crate::console_ui::resolve_registry_authority(
        &registry_path,
        &local_machine,
        true,
        &local_machine,
        Some(&registry_path),
    );
    if !authority.writable {
        return Err(format!(
            "heavy_work_policy_master_only : écriture réservée au maître local ({})",
            authority.owner_badge
        ));
    }
    Ok(())
}

fn run_policy_cli(args: &[String]) -> Result<String, String> {
    let config = DaemonConfig::default();
    let namespace_root = config
        .db_path
        .parent()
        .ok_or("namespace_root_unavailable")?;
    let action = args.first().map(String::as_str).unwrap_or("show");
    match action {
        "show" => {
            let policy = load_policy(namespace_root)?;
            let out = serde_json::json!({
                "configured": policy.is_some(),
                "path": policy_path(namespace_root),
                "policy": policy,
            });
            serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
        }
        "init" | "set-machine" | "set-global" => {
            require_local_master_authority(namespace_root)?;
            let mut apply = false;
            let mut policy = load_policy(namespace_root)?.unwrap_or_default();
            let mut i = 1usize;
            if action == "init" {
                let mut workstation: Option<String> = None;
                while i < args.len() {
                    match args[i].as_str() {
                        "--apply" => {
                            apply = true;
                            i += 1;
                        }
                        "--preview" | "--json" => i += 1,
                        "--workstation" if i + 1 < args.len() => {
                            workstation = Some(args[i + 1].clone());
                            i += 2;
                        }
                        other => {
                            return Err(format!(
                                "heavy-work policy init : argument inconnu {other}"
                            ));
                        }
                    }
                }
                if let Some(ws) = workstation {
                    if !valid_machine_id(&ws) {
                        return Err(format!("machine_id invalide : {ws}"));
                    }
                    policy
                        .machines
                        .entry(ws)
                        .or_insert_with(|| HeavyWorkMachinePolicy {
                            role: HeavyWorkRole::Protected,
                            targets: Vec::new(),
                            ssh: None,
                            max_leases: 1,
                            install_policy: HeavyWorkInstallPolicy::Ask,
                        });
                }
            } else if action == "set-machine" {
                let mut machine_id: Option<String> = None;
                let mut role: Option<HeavyWorkRole> = None;
                let mut targets: Option<Vec<String>> = None;
                let mut max_leases: Option<u8> = None;
                let mut install_policy: Option<HeavyWorkInstallPolicy> = None;
                let mut ssh_host: Option<String> = None;
                let mut ssh_user: Option<String> = None;
                let mut ssh_port: Option<u16> = None;
                let mut ssh_identity: Option<String> = None;
                let mut ssh_known_hosts: Option<String> = None;
                let mut clear_ssh = false;
                while i < args.len() {
                    match args[i].as_str() {
                        "--apply" => {
                            apply = true;
                            i += 1;
                        }
                        "--preview" | "--json" => i += 1,
                        "--clear-ssh" => {
                            clear_ssh = true;
                            i += 1;
                        }
                        "--machine" if i + 1 < args.len() => {
                            machine_id = Some(args[i + 1].clone());
                            i += 2;
                        }
                        "--role" if i + 1 < args.len() => {
                            role = Some(parse_role(&args[i + 1])?);
                            i += 2;
                        }
                        "--targets" if i + 1 < args.len() => {
                            let list: Vec<String> = if args[i + 1].trim().is_empty() {
                                Vec::new()
                            } else {
                                args[i + 1]
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect()
                            };
                            targets = Some(list);
                            i += 2;
                        }
                        "--max-leases" if i + 1 < args.len() => {
                            max_leases = Some(
                                args[i + 1]
                                    .parse::<u8>()
                                    .map_err(|_| "heavy_work_max_leases_invalid")?,
                            );
                            i += 2;
                        }
                        "--install-policy" if i + 1 < args.len() => {
                            install_policy = Some(parse_install_policy(&args[i + 1])?);
                            i += 2;
                        }
                        "--ssh-host" if i + 1 < args.len() => {
                            ssh_host = Some(args[i + 1].clone());
                            i += 2;
                        }
                        "--ssh-user" if i + 1 < args.len() => {
                            ssh_user = Some(args[i + 1].clone());
                            i += 2;
                        }
                        "--ssh-port" if i + 1 < args.len() => {
                            ssh_port = Some(
                                args[i + 1]
                                    .parse::<u16>()
                                    .map_err(|_| "heavy_work_ssh_port_invalid")?,
                            );
                            i += 2;
                        }
                        "--ssh-identity-rel" if i + 1 < args.len() => {
                            ssh_identity = Some(args[i + 1].clone());
                            i += 2;
                        }
                        "--ssh-known-hosts-rel" if i + 1 < args.len() => {
                            ssh_known_hosts = Some(args[i + 1].clone());
                            i += 2;
                        }
                        other => {
                            return Err(format!(
                                "heavy-work policy set-machine : argument inconnu {other}"
                            ));
                        }
                    }
                }
                let machine_id = machine_id.ok_or("--machine requis")?;
                let entry =
                    policy
                        .machines
                        .entry(machine_id)
                        .or_insert_with(|| HeavyWorkMachinePolicy {
                            role: HeavyWorkRole::Protected,
                            targets: Vec::new(),
                            ssh: None,
                            max_leases: 1,
                            install_policy: HeavyWorkInstallPolicy::Ask,
                        });
                if let Some(r) = role {
                    entry.role = r;
                }
                if let Some(t) = targets {
                    entry.targets = t;
                }
                if let Some(l) = max_leases {
                    entry.max_leases = l;
                }
                if let Some(p) = install_policy {
                    entry.install_policy = p;
                }
                if clear_ssh {
                    entry.ssh = None;
                } else if ssh_host.is_some()
                    || ssh_user.is_some()
                    || ssh_port.is_some()
                    || ssh_identity.is_some()
                    || ssh_known_hosts.is_some()
                {
                    let prev = entry.ssh.clone();
                    entry.ssh = Some(HeavyWorkSshProfile {
                        host: ssh_host
                            .or_else(|| prev.as_ref().map(|s| s.host.clone()))
                            .ok_or("--ssh-host requis")?,
                        user: ssh_user
                            .or_else(|| prev.as_ref().map(|s| s.user.clone()))
                            .ok_or("--ssh-user requis")?,
                        port: ssh_port
                            .or_else(|| prev.as_ref().map(|s| s.port))
                            .unwrap_or(22),
                        identity_rel: ssh_identity
                            .or_else(|| prev.as_ref().map(|s| s.identity_rel.clone()))
                            .ok_or("--ssh-identity-rel requis")?,
                        known_hosts_rel: ssh_known_hosts
                            .or_else(|| prev.as_ref().map(|s| s.known_hosts_rel.clone()))
                            .ok_or("--ssh-known-hosts-rel requis")?,
                    });
                }
            } else {
                // set-global
                while i < args.len() {
                    match args[i].as_str() {
                        "--apply" => {
                            apply = true;
                            i += 1;
                        }
                        "--preview" | "--json" => i += 1,
                        "--max-wait-secs" if i + 1 < args.len() => {
                            policy.max_wait_secs = args[i + 1]
                                .parse::<u64>()
                                .map_err(|_| "heavy_work_max_wait_invalid")?;
                            i += 2;
                        }
                        "--user-case-a" if i + 1 < args.len() => {
                            policy.user.on_busy_or_unreachable =
                                parse_unavailable_action(&args[i + 1])?;
                            i += 2;
                        }
                        "--user-case-b" if i + 1 < args.len() => {
                            policy.user.on_no_open_target = parse_fallback_action(&args[i + 1])?;
                            i += 2;
                        }
                        "--user-fallback" if i + 1 < args.len() => {
                            policy.user.fallback_after_wait = parse_fallback_action(&args[i + 1])?;
                            i += 2;
                        }
                        "--mission-case-a" if i + 1 < args.len() => {
                            policy.mission.on_busy_or_unreachable =
                                parse_unavailable_action(&args[i + 1])?;
                            i += 2;
                        }
                        "--mission-case-b" if i + 1 < args.len() => {
                            policy.mission.on_no_open_target = parse_fallback_action(&args[i + 1])?;
                            i += 2;
                        }
                        "--mission-fallback" if i + 1 < args.len() => {
                            policy.mission.fallback_after_wait =
                                parse_fallback_action(&args[i + 1])?;
                            i += 2;
                        }
                        other => {
                            return Err(format!(
                                "heavy-work policy set-global : argument inconnu {other}"
                            ));
                        }
                    }
                }
            }
            validate_policy(&policy)?;
            if apply {
                let (path, backup) = write_policy_atomic(namespace_root, &policy)?;
                let out = serde_json::json!({
                    "applied": true,
                    "path": path,
                    "backup": backup,
                    "policy": policy,
                });
                serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
            } else {
                let out = serde_json::json!({
                    "applied": false,
                    "preview": true,
                    "path": policy_path(namespace_root),
                    "policy": policy,
                });
                serde_json::to_string_pretty(&out).map_err(|e| e.to_string())
            }
        }
        other => Err(format!(
            "heavy-work policy : sous-commande inconnue « {other} » (attendu: show | init | set-machine | set-global)"
        )),
    }
}

pub(crate) fn run_cli(args: &[String]) -> Result<String, String> {
    let Some(sub) = args.first().map(String::as_str) else {
        return Err(
            "heavy-work : attendu status | plan | acquire | renew | release | policy".into(),
        );
    };
    let rest = &args[1..];
    match sub {
        "policy" => run_policy_cli(rest),
        "status" => {
            for arg in rest {
                if arg != "--json" {
                    return Err(format!("heavy-work status : argument inconnu {arg}"));
                }
            }
            let response = send_heavy_work_rpc(HeavyWorkRequest::Status)?;
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "plan" | "acquire" => {
            let mut target: Option<String> = None;
            let mut repo_owner: Option<String> = None;
            let mut requested_host: Option<String> = None;
            let mut exclusions: Vec<HeavyWorkExclusion> = Vec::new();
            let mut i = 0usize;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--json" | "--dry-run" => i += 1,
                    "--target" if i + 1 < rest.len() => {
                        target = Some(rest[i + 1].clone());
                        i += 2;
                    }
                    "--repo-owner" if i + 1 < rest.len() => {
                        repo_owner = Some(rest[i + 1].clone());
                        i += 2;
                    }
                    "--host" if i + 1 < rest.len() => {
                        requested_host = Some(rest[i + 1].clone());
                        i += 2;
                    }
                    "--exclude" if i + 1 < rest.len() => {
                        exclusions.push(parse_exclusion(&rest[i + 1])?);
                        i += 2;
                    }
                    other => return Err(format!("heavy-work {sub} : argument inconnu {other}")),
                }
            }
            let target = target.ok_or("--target requis")?;
            let caller_machine = crate::build_info::local_host();
            let origin = resolve_origin();
            let request = if sub == "plan" {
                HeavyWorkRequest::Plan {
                    caller_machine,
                    target,
                    origin,
                    requested_host,
                    exclusions,
                }
            } else {
                let repo_owner = repo_owner.ok_or("--repo-owner requis pour acquire")?;
                HeavyWorkRequest::Acquire {
                    caller_machine,
                    target,
                    origin,
                    repo_owner,
                    requested_host,
                    exclusions,
                }
            };
            let response = send_heavy_work_rpc(request)?;
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "renew" | "release" => {
            let mut lease_id: Option<String> = None;
            let mut i = 0usize;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--json" => i += 1,
                    "--lease" if i + 1 < rest.len() => {
                        lease_id = Some(rest[i + 1].clone());
                        i += 2;
                    }
                    other => return Err(format!("heavy-work {sub} : argument inconnu {other}")),
                }
            }
            let lease_id = lease_id.ok_or("--lease requis")?;
            let request = if sub == "renew" {
                HeavyWorkRequest::Renew { lease_id }
            } else {
                HeavyWorkRequest::Release { lease_id }
            };
            let response = send_heavy_work_rpc(request)?;
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        other => Err(format!(
            "heavy-work : sous-commande inconnue « {other} » (attendu: status | plan | acquire | renew | release | policy)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bridget312-{tag}-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
        dir
    }

    fn ssh(host: &str) -> HeavyWorkSshProfile {
        HeavyWorkSshProfile {
            host: host.into(),
            user: "moi".into(),
            port: 22,
            identity_rel: ".ssh/id_ed25519".into(),
            known_hosts_rel: ".ssh/known_hosts".into(),
        }
    }

    fn empty_registry(dir: &Path) -> AgentRegistry {
        let path = dir.join("agents.json");
        fs::write(&path, r#"{"agents":{}}"#).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        AgentRegistry::load_from_path(path).unwrap()
    }

    #[test]
    fn spec312_build_info_strict_json_without_daemon() {
        let out = run_build_info(&["--json".into()]).unwrap();
        let value: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["build_id"], crate::build_info::BUILD_ID);
        assert_eq!(value["os"], std::env::consts::OS);
        assert_eq!(value["arch"], std::env::consts::ARCH);
        assert!(run_build_info(&[]).is_err());
        assert!(run_build_info(&["--json".into(), "--extra".into()]).is_err());
    }

    #[test]
    fn spec312_origin_classification_is_fail_closed() {
        assert_eq!(
            classify_origin(false, false, false, false),
            HeavyWorkOrigin::User
        );
        for (d, n, m, t) in [
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (false, false, false, true),
        ] {
            assert_eq!(classify_origin(d, n, m, t), HeavyWorkOrigin::Mission);
        }
    }

    #[test]
    fn spec312_policy_absent_gives_safe_default_and_validation_rejects_bad_ssh() {
        let dir = temp_dir("absent");
        let reg = empty_registry(&dir);
        let mut mgr = HeavyWorkLeaseManager::default();
        let resp = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::Mission,
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1000,
        );
        assert!(matches!(
            resp,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::PolicyAbsent { .. }
            }
        ));

        let mut bad = HeavyWorkPolicyFile::default();
        bad.machines.insert(
            "m1".into(),
            HeavyWorkMachinePolicy {
                role: HeavyWorkRole::Open,
                targets: vec!["macos-arm64".into()],
                ssh: Some(HeavyWorkSshProfile {
                    identity_rel: "../secret".into(),
                    ..ssh("10.0.0.2")
                }),
                max_leases: 1,
                install_policy: HeavyWorkInstallPolicy::Ask,
            },
        );
        assert!(validate_policy(&bad).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn spec312_class_filter_leases_expiry_mixed_preflight_and_excluded_caller() {
        let dir = temp_dir("leases");
        let reg = empty_registry(&dir);
        let mut policy = HeavyWorkPolicyFile::default();
        policy.machines.insert(
            "monordinateur".into(),
            HeavyWorkMachinePolicy {
                role: HeavyWorkRole::Protected,
                targets: vec!["macos-arm64".into()],
                ssh: None,
                max_leases: 1,
                install_policy: HeavyWorkInstallPolicy::Never,
            },
        );
        policy.machines.insert(
            "macbook-m1".into(),
            HeavyWorkMachinePolicy {
                role: HeavyWorkRole::Open,
                targets: vec!["macos-arm64".into()],
                ssh: Some(ssh("macbook-m1.local")),
                max_leases: 1,
                install_policy: HeavyWorkInstallPolicy::Ask,
            },
        );
        policy.machines.insert(
            "macbook-m2".into(),
            HeavyWorkMachinePolicy {
                role: HeavyWorkRole::Open,
                targets: vec!["macos-arm64".into()],
                ssh: Some(ssh("macbook-m2.local")),
                max_leases: 1,
                install_policy: HeavyWorkInstallPolicy::Ask,
            },
        );
        policy.machines.insert(
            "excluded-box".into(),
            HeavyWorkMachinePolicy {
                role: HeavyWorkRole::Excluded,
                targets: vec!["macos-arm64".into()],
                ssh: Some(ssh("excluded.local")),
                max_leases: 1,
                install_policy: HeavyWorkInstallPolicy::Never,
            },
        );
        let (path, backup) = write_policy_atomic(&dir, &policy).unwrap();
        assert!(path.is_file());
        assert!(backup.is_none());

        let mut mgr = HeavyWorkLeaseManager::default();
        // 1. Acquisition choisit macbook-m1 (ordre alphabétique à capacité égale)
        let first = mgr.handle_request_at(
            HeavyWorkRequest::Acquire {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                repo_owner: "a".repeat(40),
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1000,
        );
        let lease1 = match first {
            HeavyWorkResponse::Decision {
                decision:
                    HeavyWorkDecision::RunRemote {
                        lease_id: Some(id),
                        machine_id,
                        expires_at_unix: Some(exp),
                        ..
                    },
            } => {
                assert_eq!(machine_id, "macbook-m1");
                assert_eq!(exp, 1060);
                id
            }
            other => panic!("attendu RunRemote sur macbook-m1, reçu {other:?}"),
        };

        // 2. Deuxième acquisition bascule sur macbook-m2
        let second = mgr.handle_request_at(
            HeavyWorkRequest::Acquire {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::Mission,
                repo_owner: "a".repeat(40),
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1010,
        );
        assert!(matches!(
            second,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::RunRemote { ref machine_id, .. }
            } if machine_id == "macbook-m2"
        ));

        // 3. Troisième acquisition : cas A (Wait)
        let third = mgr.handle_request_at(
            HeavyWorkRequest::Acquire {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::Mission,
                repo_owner: "a".repeat(40),
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1020,
        );
        assert!(matches!(
            third,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Wait { ref case_kind, .. }
            } if case_kind == "A"
        ));

        // 4. Expiration du bail 1 après 60 s (t = 1061) → macbook-m1 de nouveau libre
        let after_expiry = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1061,
        );
        assert!(matches!(
            after_expiry,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::RunRemote { ref machine_id, .. }
            } if machine_id == "macbook-m1"
        ));
        assert!(matches!(
            mgr.handle_request_at(
                HeavyWorkRequest::Renew { lease_id: lease1 },
                &dir,
                "cartae",
                &reg,
                1061,
            ),
            HeavyWorkResponse::Error { ref code, .. } if code == "heavy_work_lease_not_found"
        ));

        // 5. Motifs mixtes de pré-vérification : un unreachable + un prerequisite_missing => cas A
        let mixed = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: None,
                exclusions: vec![
                    HeavyWorkExclusion {
                        machine_id: "macbook-m1".into(),
                        reason: HeavyWorkPreflightFailure::Unreachable,
                    },
                    HeavyWorkExclusion {
                        machine_id: "macbook-m2".into(),
                        reason: HeavyWorkPreflightFailure::PrerequisiteMissing,
                    },
                ],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            mixed,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Wait { ref case_kind, .. }
            } if case_kind == "A"
        ));

        // 6. Tous prerequisite_missing => cas B (user = discret, mission = refuser)
        let both_missing_user = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: None,
                exclusions: vec![
                    HeavyWorkExclusion {
                        machine_id: "macbook-m1".into(),
                        reason: HeavyWorkPreflightFailure::PrerequisiteMissing,
                    },
                    HeavyWorkExclusion {
                        machine_id: "macbook-m2".into(),
                        reason: HeavyWorkPreflightFailure::PrerequisiteMissing,
                    },
                ],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            both_missing_user,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::FallbackDiscrete { ref case_kind, .. }
            } if case_kind == "B"
        ));

        let both_missing_mission = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::Mission,
                requested_host: None,
                exclusions: vec![
                    HeavyWorkExclusion {
                        machine_id: "macbook-m1".into(),
                        reason: HeavyWorkPreflightFailure::PrerequisiteMissing,
                    },
                    HeavyWorkExclusion {
                        machine_id: "macbook-m2".into(),
                        reason: HeavyWorkPreflightFailure::PrerequisiteMissing,
                    },
                ],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            both_missing_mission,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Refused { ref case_kind, .. }
            } if case_kind == "B"
        ));

        // 7. --host sur machine `exclue` refusé ; appelant `exclue` refuse repli discret
        let host_excluded = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "monordinateur".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: Some("excluded-box".into()),
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            host_excluded,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Refused { ref code, .. }
            } if code == "heavy_work_host_excluded"
        ));

        let caller_excluded = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "excluded-box".into(),
                target: "linux-x86_64-glibc2.36".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            caller_excluded,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::Refused { ref code, .. }
            } if code == "heavy_work_caller_excluded"
        ));

        // 8. Appelant choisi lui-même machine ouverte => RunLocalLease
        let self_open = mgr.handle_request_at(
            HeavyWorkRequest::Plan {
                caller_machine: "macbook-m1".into(),
                target: "macos-arm64".into(),
                origin: HeavyWorkOrigin::User,
                requested_host: None,
                exclusions: vec![],
            },
            &dir,
            "cartae",
            &reg,
            1080,
        );
        assert!(matches!(
            self_open,
            HeavyWorkResponse::Decision {
                decision: HeavyWorkDecision::RunLocalLease { ref machine_id, .. }
            } if machine_id == "macbook-m1"
        ));

        let _ = fs::remove_dir_all(&dir);
    }
}
