//! Ramasse-copies : purge des `/tmp/bridget-*` orphelins âgés, alerte disque,
//! et inventaire dry-run des `target/` de worktrees déjà mergés.
//!
//! Discipline alignée sur `purge_orphan_mcp_configs` : best-effort au boot,
//! log nominatif, jamais d'effacement sous un processus vivant (fail-closed
//! si la sonde propriétaire est incertaine).

use log::{info, warn};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Âge minimal avant qu'une copie `/tmp/bridget-*` orpheline soit éligible.
pub const ORPHAN_MIN_AGE_SECS: u64 = 24 * 60 * 60;

/// Seuil d'alerte : en dessous, WARN visible (boot, reaper, who/status).
pub const DISK_WARN_FREE_BYTES: u64 = 20 * 1024 * 1024 * 1024;

/// Préfixe des copies de travail à considérer sous `/tmp`.
pub const BRIDGET_TMP_PREFIX: &str = "bridget-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PurgeDecision {
    Delete,
    Spare { reason: &'static str },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeReport {
    pub deleted: Vec<PathBuf>,
    pub spared: Vec<(PathBuf, &'static str)>,
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedWorktreeTarget {
    pub worktree: PathBuf,
    pub branch: Option<String>,
    pub target_dir: PathBuf,
    pub size_bytes: Option<u64>,
}

/// Décision pure : âge + propriétaires connus.
///
/// `owner_pids = None` → propriétaire inconnu → épargne (fail-closed).
/// `owner_pids = Some([])` → orphelin confirmé.
/// `owner_pids = Some([..])` → vivant → épargne.
pub fn decide_purge(age_secs: u64, owner_pids: Option<&[u32]>, min_age_secs: u64) -> PurgeDecision {
    match owner_pids {
        None => PurgeDecision::Spare {
            reason: "proprietaire_inconnu",
        },
        Some(pids) if !pids.is_empty() => PurgeDecision::Spare {
            reason: "processus_vivant",
        },
        Some(_) if age_secs < min_age_secs => PurgeDecision::Spare {
            reason: "copie_du_jour",
        },
        Some(_) => PurgeDecision::Delete,
    }
}

/// Message d'alerte si l'espace libre du volume est sous le seuil.
pub fn disk_pressure_warning(free_bytes: u64, threshold_bytes: u64) -> Option<String> {
    if free_bytes >= threshold_bytes {
        return None;
    }
    let free_gi = free_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let threshold_gi = threshold_bytes / (1024 * 1024 * 1024);
    Some(format!(
        "WARN disque: {free_gi:.1} Gi libres (< {threshold_gi} Gi) — saturation récurrente des copies de travail"
    ))
}

/// Espace libre disponible pour un utilisateur non-root sur le volume de `path`.
pub fn free_bytes_for(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    // SAFETY: pointeur CString valide le temps de l'appel ; struct zeroed.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return None;
        }
        // Casts volontaires : largeur de f_bavail/f_frsize varie selon l'OS.
        #[allow(clippy::unnecessary_cast)]
        {
            Some(stat.f_bavail as u64 * stat.f_frsize as u64)
        }
    }
}

/// WARN nominatif si le volume de `path` est sous le seuil.
pub fn warn_if_disk_low(path: &Path) {
    let Some(free) = free_bytes_for(path) else {
        return;
    };
    if let Some(message) = disk_pressure_warning(free, DISK_WARN_FREE_BYTES) {
        warn!("{message}");
    }
}

/// Texte à afficher dans who/status (aucune panique auto).
pub fn disk_warning_for_display(path: &Path) -> Option<String> {
    let free = free_bytes_for(path)?;
    disk_pressure_warning(free, DISK_WARN_FREE_BYTES)
}

/// Purge au boot : `/tmp/bridget-*` orphelins et âgés > 24 h.
pub fn purge_orphan_bridget_tmp(tmp_dir: &Path) -> PurgeReport {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    purge_orphan_bridget_tmp_at(tmp_dir, now, ORPHAN_MIN_AGE_SECS, &LiveOwnerProbe)
}

/// Cœur testable du ramassage.
pub fn purge_orphan_bridget_tmp_at<P: OwnerProbe>(
    tmp_dir: &Path,
    now_unix: u64,
    min_age_secs: u64,
    probe: &P,
) -> PurgeReport {
    let mut report = PurgeReport::default();
    let entries = match scan_bridget_tmp(tmp_dir, now_unix) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                "ramasse-copies: lecture {} impossible: {error}",
                tmp_dir.display()
            );
            return report;
        }
    };

    for entry in entries {
        let owners = probe.owner_pids(&entry.path);
        let decision = decide_purge(entry.age_secs, owners.as_deref(), min_age_secs);
        match decision {
            PurgeDecision::Spare { reason } => {
                report.spared.push((entry.path, reason));
            }
            PurgeDecision::Delete => {
                let path = entry.path;
                let remove_result = if entry.is_dir {
                    fs::remove_dir_all(&path)
                } else {
                    fs::remove_file(&path)
                };
                match remove_result {
                    Ok(()) => {
                        info!("ramasse-copies: orphelin âgé supprimé {}", path.display());
                        report.deleted.push(path);
                    }
                    Err(error) => {
                        warn!(
                            "ramasse-copies: suppression impossible {}: {error}",
                            path.display()
                        );
                        report.errors.push((path, error.to_string()));
                    }
                }
            }
        }
    }
    report
}

#[derive(Debug, Clone)]
struct BridgetTmpEntry {
    path: PathBuf,
    age_secs: u64,
    is_dir: bool,
}

fn scan_bridget_tmp(tmp_dir: &Path, now_unix: u64) -> io::Result<Vec<BridgetTmpEntry>> {
    let entries = match fs::read_dir(tmp_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(BRIDGET_TMP_PREFIX) {
            continue;
        }
        // Socket / pid de production : ne pas toucher au socket daemon courant.
        if name == "bridget.sock" || name.ends_with(".sock") || name.ends_with(".pid") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let age = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| now_unix.saturating_sub(d.as_secs()))
            .unwrap_or(0);
        out.push(BridgetTmpEntry {
            path: entry.path(),
            age_secs: age,
            is_dir: meta.is_dir(),
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Sonde des PIDs qui détiennent encore une copie.
pub trait OwnerProbe {
    /// `None` = incertitude (épargner). `Some(pids)` = liste connue (vide = orphelin).
    fn owner_pids(&self, path: &Path) -> Option<Vec<u32>>;
}

/// Sonde réelle : lsof + PID encodé dans le nom + cmdline ps.
pub struct LiveOwnerProbe;

impl OwnerProbe for LiveOwnerProbe {
    fn owner_pids(&self, path: &Path) -> Option<Vec<u32>> {
        let mut pids = Vec::new();

        if let Some(name_pid) = pid_suffix_from_name(path)
            && process_alive(name_pid)
        {
            pids.push(name_pid);
        }

        match lsof_pids(path) {
            Ok(found) => {
                for pid in found {
                    if !pids.contains(&pid) {
                        pids.push(pid);
                    }
                }
            }
            Err(_) => {
                // lsof indisponible ou en échec → fail-closed sauf si on a déjà
                // un propriétaire nommé vivant (alors la protection tient).
                if pids.is_empty() {
                    return None;
                }
            }
        }

        if let Ok(mentioned) = cmdline_pids_mentioning(path) {
            for pid in mentioned {
                if !pids.contains(&pid) {
                    pids.push(pid);
                }
            }
        }

        Some(pids)
    }
}

fn pid_suffix_from_name(path: &Path) -> Option<u32> {
    let name = path.file_name()?.to_str()?;
    let tail = name.rsplit('-').next()?;
    let pid: u32 = tail.parse().ok()?;
    // Éviter les faux positifs (années, compteurs courts).
    if pid >= 100 { Some(pid) } else { None }
}

fn process_alive(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

fn lsof_pids(path: &Path) -> io::Result<Vec<u32>> {
    let output = Command::new("lsof").args(["-t", "--"]).arg(path).output()?;
    // lsof exit 1 = aucun processus : orphelin confirmé.
    if !output.status.success() && !output.stdout.is_empty() {
        return Err(io::Error::other("lsof a échoué"));
    }
    let mut pids = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Ok(pid) = line.trim().parse::<u32>()
            && !pids.contains(&pid)
        {
            pids.push(pid);
        }
    }
    Ok(pids)
}

fn cmdline_pids_mentioning(path: &Path) -> io::Result<Vec<u32>> {
    let needle = path.to_string_lossy();
    let output = Command::new("/bin/ps")
        .args(["-axo", "pid=,command="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("ps a échoué"));
    }
    let mut pids = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let line = line.trim();
        let Some((pid_s, command)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        if !command.contains(needle.as_ref()) {
            continue;
        }
        if let Ok(pid) = pid_s.trim().parse::<u32>() {
            pids.push(pid);
        }
    }
    Ok(pids)
}

/// Inventaire dry-run : `target/` des worktrees dont la branche est ancêtre de HEAD.
pub fn list_merged_worktree_targets(repo_root: &Path) -> io::Result<Vec<MergedWorktreeTarget>> {
    let worktrees_dir = repo_root.join(".worktrees");
    if !worktrees_dir.is_dir() {
        return Ok(Vec::new());
    }

    let head = git_stdout(repo_root, &["rev-parse", "HEAD"])?;
    let head = head.trim();
    let mut out = Vec::new();

    for entry in fs::read_dir(&worktrees_dir)?.flatten() {
        let worktree = entry.path();
        if !worktree.is_dir() {
            continue;
        }
        let target_dir = worktree.join("target");
        if !target_dir.is_dir() {
            continue;
        }
        let branch = git_stdout(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "HEAD");
        let tip = match git_stdout(&worktree, &["rev-parse", "HEAD"]) {
            Ok(tip) => tip.trim().to_string(),
            Err(_) => continue,
        };
        // Mergé = tip ancêtre de main/HEAD du dépôt principal.
        let merged = git_stdout(
            repo_root,
            &["merge-base", "--is-ancestor", tip.as_str(), head],
        )
        .is_ok();
        if !merged {
            continue;
        }
        let size_bytes = dir_size_bytes(&target_dir).ok();
        out.push(MergedWorktreeTarget {
            worktree,
            branch,
            target_dir,
            size_bytes,
        });
    }
    out.sort_by(|a, b| a.worktree.cmp(&b.worktree));
    Ok(out)
}

fn git_stdout(cwd: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new("git").args(args).current_dir(cwd).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!("git {} a échoué", args.join(" "))));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn dir_size_bytes(path: &Path) -> io::Result<u64> {
    let mut total = 0u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current)?.flatten() {
            let meta = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => continue,
            };
            if meta.is_dir() {
                stack.push(entry.path());
            } else {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

pub fn render_cleanup_dry_run(targets: &[MergedWorktreeTarget]) -> String {
    let mut out = String::new();
    out.push_str("bridget cleanup --dry-run (aucune suppression)\n");
    if targets.is_empty() {
        out.push_str("Aucune cible target/ de worktree mergé trouvée.\n");
        return out;
    }
    for target in targets {
        let size = match target.size_bytes {
            Some(bytes) => format!("{:.1} Gi", bytes as f64 / (1024.0 * 1024.0 * 1024.0)),
            None => "?".into(),
        };
        let branch = target.branch.as_deref().unwrap_or("(détaché)");
        out.push_str(&format!(
            "  {}  branch={}  size={}\n",
            target.target_dir.display(),
            branch,
            size
        ));
    }
    out.push_str("Décision utilisateur requise — pas d'effacement auto de target/.\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::OpenOptionsExt;

    struct MapProbe {
        map: HashMap<PathBuf, Option<Vec<u32>>>,
    }

    impl OwnerProbe for MapProbe {
        fn owner_pids(&self, path: &Path) -> Option<Vec<u32>> {
            self.map.get(path).cloned().unwrap_or(Some(vec![]))
        }
    }

    fn set_mtime(path: &Path, unix_secs: i64) {
        use std::io::Write;
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        let _ = (&file).write_all(b"x");
        let times = libc::timeval {
            tv_sec: unix_secs,
            tv_usec: 0,
        };
        let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        // SAFETY: chemin CString valide ; tableau de 2 timeval (atime, mtime).
        unsafe {
            let tv = [times, times];
            assert_eq!(libc::utimes(c_path.as_ptr(), tv.as_ptr()), 0);
        }
    }

    #[test]
    fn orpheline_agee_est_ramassee() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS + 1, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Delete
        );
    }

    #[test]
    fn copie_du_jour_est_epargnee() {
        assert_eq!(
            decide_purge(60, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "copie_du_jour"
            }
        );
    }

    #[test]
    fn processus_vivant_est_epargne() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, Some(&[4242]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "processus_vivant"
            }
        );
    }

    #[test]
    fn proprietaire_inconnu_fail_closed() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, None, ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "proprietaire_inconnu"
            }
        );
    }

    #[test]
    fn seuil_franchi_produit_warn() {
        let message = disk_pressure_warning(5 * 1024 * 1024 * 1024, DISK_WARN_FREE_BYTES).unwrap();
        assert!(message.contains("WARN disque"));
        assert!(message.contains("< 20 Gi"));
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES, DISK_WARN_FREE_BYTES).is_none());
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES + 1, DISK_WARN_FREE_BYTES).is_none());
    }

    #[test]
    fn purge_reelle_respecte_age_et_vivant() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let aged = root.join("bridget-aged-orphan");
        let today = root.join("bridget-today");
        let live = root.join("bridget-live-owner");
        let now = 1_700_000_000i64;
        set_mtime(&aged, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);
        set_mtime(&today, now - 120);
        set_mtime(&live, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);

        let mut map = HashMap::new();
        map.insert(aged.clone(), Some(vec![]));
        map.insert(today.clone(), Some(vec![]));
        map.insert(live.clone(), Some(vec![7]));
        let probe = MapProbe { map };

        let report = purge_orphan_bridget_tmp_at(&root, now as u64, ORPHAN_MIN_AGE_SECS, &probe);

        assert!(!aged.exists(), "orpheline âgée doit être ramassée");
        assert!(today.exists(), "copie du jour doit être épargnée");
        assert!(live.exists(), "copie d'un vivant doit être épargnée");
        assert_eq!(report.deleted, vec![aged]);
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &today && *r == "copie_du_jour")
        );
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &live && *r == "processus_vivant")
        );

        let _ = fs::remove_dir_all(&root);
    }
}
